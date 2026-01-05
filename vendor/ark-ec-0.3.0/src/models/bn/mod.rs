use crate::{
    models::{ModelParameters, SWModelParameters},
    PairingEngine,
};
use ark_ff::fields::{
    fp12_2over3over2::{Fp12, Fp12Parameters},
    fp2::Fp2Parameters,
    fp6_3over2::Fp6Parameters,
    Field, Fp2, PrimeField, SquareRootField,
};
use num_traits::One;

use core::marker::PhantomData;

pub enum TwistType {
    M,
    D,
}

pub trait BnParameters: 'static {
    // The absolute value of the BN curve parameter `X` (as in `q = 36 X^4 + 36 X^3 + 24 X^2 + 6 X + 1`).
    const X: &'static [u64];
    // Whether or not `X` is negative.
    const X_IS_NEGATIVE: bool;

    // The absolute value of `6X + 2`.
    const ATE_LOOP_COUNT: &'static [i8];

    const TWIST_TYPE: TwistType;
    const TWIST_MUL_BY_Q_X: Fp2<Self::Fp2Params>;
    const TWIST_MUL_BY_Q_Y: Fp2<Self::Fp2Params>;
    type Fp: PrimeField + SquareRootField + Into<<Self::Fp as PrimeField>::BigInt>;
    type Fp2Params: Fp2Parameters<Fp = Self::Fp>;
    type Fp6Params: Fp6Parameters<Fp2Params = Self::Fp2Params>;
    type Fp12Params: Fp12Parameters<Fp6Params = Self::Fp6Params>;
    type G1Parameters: SWModelParameters<BaseField = Self::Fp>;
    type G2Parameters: SWModelParameters<
        BaseField = Fp2<Self::Fp2Params>,
        ScalarField = <Self::G1Parameters as ModelParameters>::ScalarField,
    >;
}

pub mod g1;
pub mod g2;

pub use self::{
    g1::{G1Affine, G1Prepared, G1Projective},
    g2::{G2Affine, G2Prepared, G2Projective},
};

#[derive(Derivative)]
#[derivative(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Bn<P: BnParameters>(PhantomData<fn() -> P>);

impl<P: BnParameters> Bn<P> {
    // Evaluate the line function at point p.
    fn ell(f: &mut Fp12<P::Fp12Params>, coeffs: &g2::EllCoeff<Fp2<P::Fp2Params>>, p: &G1Affine<P>) {
        let mut c0 = coeffs.0;
        let mut c1 = coeffs.1;
        let mut c2 = coeffs.2;

        match P::TWIST_TYPE {
            TwistType::M => {
                c2.mul_assign_by_fp(&p.y);
                c1.mul_assign_by_fp(&p.x);
                f.mul_by_014(&c0, &c1, &c2);
            }
            TwistType::D => {
                c0.mul_assign_by_fp(&p.y);
                c1.mul_assign_by_fp(&p.x);
                f.mul_by_034(&c0, &c1, &c2);
            }
        }
    }

    fn exp_by_neg_x(mut f: Fp12<P::Fp12Params>) -> Fp12<P::Fp12Params> {
        f = f.cyclotomic_exp(&P::X);
        if !P::X_IS_NEGATIVE {
            f.conjugate();
        }
        f
    }
}

impl<P: BnParameters> PairingEngine for Bn<P> {
    type Fr = <P::G1Parameters as ModelParameters>::ScalarField;
    type G1Projective = G1Projective<P>;
    type G1Affine = G1Affine<P>;
    type G1Prepared = G1Prepared<P>;
    type G2Projective = G2Projective<P>;
    type G2Affine = G2Affine<P>;
    type G2Prepared = G2Prepared<P>;
    type Fq = P::Fp;
    type Fqe = Fp2<P::Fp2Params>;
    type Fqk = Fp12<P::Fp12Params>;

    fn miller_loop<'a, I>(i: I) -> Self::Fqk
    where
        I: IntoIterator<Item = &'a (Self::G1Prepared, Self::G2Prepared)>,
    {
        let mut pairs = vec![];
        for (p, q) in i {
            if !p.is_zero() && !q.is_zero() {
                pairs.push((p, q.ell_coeffs.iter()));
            }
        }

        let mut f = Self::Fqk::one();

        for i in (1..P::ATE_LOOP_COUNT.len()).rev() {
            if i != P::ATE_LOOP_COUNT.len() - 1 {
                f.square_in_place();
            }

            for (p, ref mut coeffs) in &mut pairs {
                Self::ell(&mut f, coeffs.next().unwrap(), &p.0);
            }

            let bit = P::ATE_LOOP_COUNT[i - 1];
            match bit {
                1 => {
                    for &mut (p, ref mut coeffs) in &mut pairs {
                        Self::ell(&mut f, coeffs.next().unwrap(), &p.0);
                    }
                }
                -1 => {
                    for &mut (p, ref mut coeffs) in &mut pairs {
                        Self::ell(&mut f, coeffs.next().unwrap(), &p.0);
                    }
                }
                _ => continue,
            }
        }

        if P::X_IS_NEGATIVE {
            f.conjugate();
        }

        for &mut (p, ref mut coeffs) in &mut pairs {
            Self::ell(&mut f, coeffs.next().unwrap(), &p.0);
        }

        for &mut (p, ref mut coeffs) in &mut pairs {
            Self::ell(&mut f, coeffs.next().unwrap(), &p.0);
        }

        f
    }

    #[allow(clippy::let_and_return)]
    fn final_exponentiation(f: &Self::Fqk) -> Option<Self::Fqk> {
        // Easy part: result = elt^((q^6-1)*(q^2+1)).
        // Follows, e.g., Beuchat et al page 9, by computing result as follows:
        //   elt^((q^6-1)*(q^2+1)) = (conj(elt) * elt^(-1))^(q^2+1)

        // f1 = r.conjugate() = f^(p^6)
        let mut f1 = *f;
        f1.conjugate();

        f.inverse().map(|mut f2| {
            // f2 = f^(-1);
            // r = f^(p^6 - 1)
            let mut r = f1 * &f2;

            // f2 = f^(p^6 - 1)
            f2 = r;
            // r = f^((p^6 - 1)(p^2))
            r.frobenius_map(2);

            // r = f^((p^6 - 1)(p^2) + (p^6 - 1))
            // r = f^((p^6 - 1)(p^2 + 1))
            r *= &f2;

            // Hard part follows Laura Fuentes-Castaneda et al. "Faster hashing to G2"
            // by computing:
            //
            // result = elt^(q^3 * (12*z^3 + 6z^2 + 4z - 1) +
            //               q^2 * (12*z^3 + 6z^2 + 6z) +
            //               q   * (12*z^3 + 6z^2 + 4z) +
            //               1   * (12*z^3 + 12z^2 + 6z + 1))
            // which equals
            //
            // result = elt^( 2z * ( 6z^2 + 3z + 1 ) * (q^4 - q^2 + 1)/r ).

            // Reduce stack pressure by containing many temporaries inside an inner scope
            // and returning only the small set we need for later steps.
            // More aggressive reduction: allocate some intermediates on the heap
            // to reduce the maximum stack frame size during SBF compilation.
            use ark_std::boxed::Box;

            let (mut it, j, n) = {
                let a = Box::new(Self::exp_by_neg_x(r)); // boxed y0
                let b = Box::new(a.cyclotomic_square()); // boxed y1
                let c = Box::new(b.cyclotomic_square()); // boxed y2
                let d = Box::new((*c) * &*b); // boxed y3
                let e = Box::new(Self::exp_by_neg_x(*d)); // boxed y4
                let _f = Box::new(e.cyclotomic_square()); // boxed y5
                let g = Box::new(Self::exp_by_neg_x(*_f)); // boxed y6

                // Operate using boxed values to keep temporaries on heap
                let mut d = *d; // bring into local, reuse after
                let mut g = *g;
                d.conjugate();
                g.conjugate();
                let h = g * &*e; // y7
                let mut it = h * &d; // y8
                let j = it * &*b; // y9
                let k = it * &*e; // y10
                let l = k * &r; // y11
                let mut m = j; // y12
                m.frobenius_map(1);
                let n = m * &l; // y13
                it.frobenius_map(2);
                (it, j, n)
            };

            // Continue with smaller live set
            let o = it * &n; // y14
            let mut t = { let mut tmp = r; tmp.conjugate(); tmp * &j }; // y15
            t.frobenius_map(3);
            let y16 = t * &o;

            y16
        })
    }
}

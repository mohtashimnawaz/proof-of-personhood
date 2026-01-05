use ark_bn254::Bn254;
use ark_groth16::{generate_random_parameters, create_random_proof};
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;
use verifier::verify_groth16_native;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use rand::Rng;

#[derive(Clone)]
struct SimpleCircuit {
    pub a: u64,
    pub b: u64,
}

impl ark_relations::r1cs::ConstraintSynthesizer<ark_bn254::Fr> for SimpleCircuit {
    fn generate_constraints(self, cs: ark_relations::r1cs::ConstraintSystemRef<ark_bn254::Fr>) -> Result<(), ark_relations::r1cs::SynthesisError> {
        let a_var = ark_r1cs_std::fields::fp::FpVar::new_input(cs.clone(), || Ok(ark_bn254::Fr::from(self.a))).unwrap();
        let b_var = ark_r1cs_std::fields::fp::FpVar::new_witness(cs.clone(), || Ok(ark_bn254::Fr::from(self.b))).unwrap();
        let prod = &a_var * &b_var;
        let expected = ark_r1cs_std::fields::fp::FpVar::new_input(cs, || Ok(ark_bn254::Fr::from(self.a * self.b))).unwrap();
        prod.enforce_equal(&expected)?;
        Ok(())
    }
}

#[test]
fn test_native_verify() {
    let rng = &mut rand::thread_rng();
    let circuit = SimpleCircuit { a: 4, b: 9 };
    let params = generate_random_parameters::<Bn254, _, _>(circuit.clone(), rng).unwrap();
    let proof = create_random_proof(circuit, &params, rng).unwrap();

    let pub_inp = vec![ark_bn254::Fr::from(4u64), ark_bn254::Fr::from(36u64)];

    let ok = verify_groth16_native(&params.vk, &proof, &pub_inp).unwrap();
    assert!(ok);
}

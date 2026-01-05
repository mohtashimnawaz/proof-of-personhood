use anyhow::Result;
use ark_bn254::Bn254;
use ark_groth16::{generate_random_parameters, create_random_proof, prepare_verifying_key, verify_proof, Proof, VerifyingKey};
use ark_relations::r1cs::ConstraintSynthesizer;
use ark_relations::r1cs::ConstraintSystemRef;
use ark_relations::r1cs::SynthesisError;
use ark_r1cs_std::alloc::AllocVar;
use ark_ff::UniformRand;
use ark_serialize::CanonicalSerialize;
use std::io::Write;
use std::fs::File;

#[derive(Clone)]
struct SimpleCircuit {
    pub a: u64,
    pub b: u64,
}

impl ConstraintSynthesizer<<Bn254 as ark_ff::Field>::Fr> for SimpleCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<<Bn254 as ark_ff::Field>::Fr>) -> Result<(), SynthesisError> {
        use ark_r1cs_std::fields::fp::FpVar;
        let a_var = FpVar::new_input(cs.clone(), || Ok(<Bn254 as ark_ff::Field>::from(self.a))).unwrap();
        let b_var = FpVar::new_witness(cs.clone(), || Ok(<Bn254 as ark_ff::Field>::from(self.b))).unwrap();
        let prod = &a_var * &b_var;
        let expected = FpVar::new_input(cs, || Ok(<Bn254 as ark_ff::Field>::from(self.a * self.b))).unwrap();
        prod.enforce_equal(&expected)?;
        Ok(())
    }
}

fn main() -> Result<()> {
    // Example: prove that a * b == a*b (trivial)
    let rng = &mut rand::thread_rng();
    let circuit = SimpleCircuit { a: 3, b: 11 };

    println!("Generating parameters...");
    let params = generate_random_parameters::<Bn254, _, _>(circuit.clone(), rng)?;

    println!("Creating proof...");
    let proof = create_random_proof(circuit, &params, rng)?;

    println!("Serializing...");
    // Serialize verifying key, proof, and public inputs
    let mut vk_file = File::create("vk.bin")?;
    params.vk.serialize_unchecked(&mut vk_file)?;

    let mut proof_file = File::create("proof.bin")?;
    proof.serialize_unchecked(&mut proof_file)?;

    let mut pub_file = File::create("public_inputs.bin")?;
    // public inputs: a and a*b
    let fr_a = <Bn254 as ark_ff::Field>::from(3u64);
    let fr_ab = <Bn254 as ark_ff::Field>::from(33u64);
    fr_a.serialize_unchecked(&mut pub_file)?;
    fr_ab.serialize_unchecked(&mut pub_file)?;

    println!("Done: vk.bin, proof.bin, public_inputs.bin written");
    Ok(())
}

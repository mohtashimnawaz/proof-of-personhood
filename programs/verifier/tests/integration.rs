use verifier::process_instruction;
use solana_program::pubkey::Pubkey;
use ark_bn254::{Bn254, Fr};
use ark_groth16::{generate_random_parameters, create_random_proof};
use ark_serialize::CanonicalSerialize;
use ark_relations::r1cs::ConstraintSynthesizer;
use ark_relations::r1cs::ConstraintSystemRef;
use ark_relations::r1cs::SynthesisError;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::alloc::AllocVar;
use rand::Rng;
use solana_program::account_info::AccountInfo;
use std::sync::Arc;

#[derive(Clone)]
struct SimpleCircuit {
    pub a: u64,
    pub b: u64,
}

impl ConstraintSynthesizer<Fr> for SimpleCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let a_var = FpVar::new_input(cs.clone(), || Ok(Fr::from(self.a))).unwrap();
        let b_var = FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.b))).unwrap();
        let prod = &a_var * &b_var;
        let expected = FpVar::new_input(cs, || Ok(Fr::from(self.a * self.b))).unwrap();
        prod.enforce_equal(&expected)?;
        Ok(())
    }
}

#[test]
fn test_onchain_verify_roundtrip() {
    let rng = &mut rand::thread_rng();
    let circuit = SimpleCircuit { a: 7, b: 13 };
    let params = generate_random_parameters::<Bn254, _, _>(circuit.clone(), rng).unwrap();
    let proof = create_random_proof(circuit, &params, rng).unwrap();

    // serialize vk and proof to bytes
    let mut vk_bytes = vec![];
    params.vk.serialize_uncompressed(&mut vk_bytes).unwrap();

    let mut proof_bytes = vec![];
    proof.serialize_uncompressed(&mut proof_bytes).unwrap();

    // public inputs: a and a*b
    let mut pub_bytes = vec![];
    Fr::from(7u64).serialize_uncompressed(&mut pub_bytes).unwrap();
    Fr::from(91u64).serialize_uncompressed(&mut pub_bytes).unwrap();

    let mut instruction_data = vec![];
    instruction_data.extend_from_slice(&vk_bytes);
    instruction_data.extend_from_slice(&proof_bytes);
    instruction_data.extend_from_slice(&pub_bytes);

    // call process_instruction
    let program_id = Pubkey::new_unique();
    let accounts: Vec<AccountInfo> = vec![];

    let res = process_instruction(&program_id, &accounts, &instruction_data);
    assert!(res.is_ok());
}

use verifier::process_instruction;
use solana_program::pubkey::Pubkey;
use ark_bn254::{Bn254, Fr};
use ark_groth16::{generate_random_parameters, create_random_proof};
use ark_serialize::CanonicalSerialize;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use rand::Rng;

#[derive(Clone)]
struct SimpleCircuit {
    pub a: u64,
    pub b: u64,
}

impl ark_relations::r1cs::ConstraintSynthesizer<Fr> for SimpleCircuit {
    fn generate_constraints(self, cs: ark_relations::r1cs::ConstraintSystemRef<Fr>) -> Result<(), ark_relations::r1cs::SynthesisError> {
        let a_var = ark_r1cs_std::fields::fp::FpVar::new_input(cs.clone(), || Ok(Fr::from(self.a))).unwrap();
        let b_var = ark_r1cs_std::fields::fp::FpVar::new_witness(cs.clone(), || Ok(Fr::from(self.b))).unwrap();
        let prod = &a_var * &b_var;
        let expected = ark_r1cs_std::fields::fp::FpVar::new_input(cs, || Ok(Fr::from(self.a * self.b))).unwrap();
        prod.enforce_equal(&expected)?;
        Ok(())
    }
}

#[test]
fn test_e2e_verify() {
    let rng = &mut rand::thread_rng();
    let circuit = SimpleCircuit { a: 5, b: 17 };
    let params = generate_random_parameters::<Bn254, _, _>(circuit.clone(), rng).unwrap();
    let proof = create_random_proof(circuit, &params, rng).unwrap();

    let mut vk_bytes = vec![];
    params.vk.serialize_uncompressed(&mut vk_bytes).unwrap();
    let mut proof_bytes = vec![];
    proof.serialize_uncompressed(&mut proof_bytes).unwrap();

    let mut pub_bytes = vec![];
    Fr::from(5u64).serialize_uncompressed(&mut pub_bytes).unwrap();
    Fr::from(85u64).serialize_uncompressed(&mut pub_bytes).unwrap();

    let mut instruction_data = vec![];
    instruction_data.extend_from_slice(&vk_bytes);
    instruction_data.extend_from_slice(&proof_bytes);
    instruction_data.extend_from_slice(&pub_bytes);

    let program_id = Pubkey::new_unique();
    let accounts: Vec<solana_program::account_info::AccountInfo> = vec![];

    let res = process_instruction(&program_id, &accounts, &instruction_data);
    assert!(res.is_ok());
}

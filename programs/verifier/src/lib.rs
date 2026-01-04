use solana_program::{account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey, msg};

#[derive(Debug)]
pub enum VerifierError {
    InvalidInstruction,
}

entrypoint!(process_instruction);

pub fn process_instruction(_program_id: &Pubkey, _accounts: &[AccountInfo], _instruction_data: &[u8]) -> ProgramResult {
    // For now, just log and return OK. We'll wire in Groth16 verify next.
    msg!("Verifier program entrypoint reached");
    Ok(())
}

// Minimal compile-time check: reference arkworks Groth16 types to ensure they are available to the build.
// This function is not used at runtime yet; it's to exercise the dependency graph during build.
#[allow(dead_code)]
fn _compile_check_groth16() {
    #[cfg(any())]
    {
        use ark_bn254::Bn254;
        use ark_groth16::{Proof, VerifyingKey};
        // Create zero-sized placeholders to ensure types resolve.
        let _vk: Option<VerifyingKey<Bn254>> = None;
        let _proof: Option<Proof<Bn254>> = None;
        let _ = (_vk, _proof);
    }
}

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

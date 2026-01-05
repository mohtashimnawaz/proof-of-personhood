use solana_program::{account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey, msg, program_error::ProgramError};
use ark_bn254::Bn254;
use ark_groth16::{Proof, VerifyingKey, verify_proof};
use ark_ff::PrimeField;
use ark_serialize::CanonicalDeserialize;

#[derive(Debug)]
pub enum VerifierError {
    InvalidInstruction,
    DeserializationFailed,
    VerificationFailed,
    InvalidProofFormat,
}

impl From<VerifierError> for ProgramError {
    fn from(e: VerifierError) -> Self {
        match e {
            VerifierError::InvalidInstruction => ProgramError::InvalidInstructionData,
            VerifierError::DeserializationFailed => ProgramError::InvalidArgument,
            VerifierError::VerificationFailed => ProgramError::Custom(1),
            VerifierError::InvalidProofFormat => ProgramError::Custom(2),
        }
    }
}

entrypoint!(process_instruction);

/// Instruction format:
/// [0:32] - Verifying Key (248 bytes for BN254 Groth16)
/// [248:360] - Proof (128 bytes for BN254 Groth16)
/// [360:] - Public inputs (variable length, each input is 32 bytes)
pub fn process_instruction(_program_id: &Pubkey, _accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    msg!("Verifier program invoked");

    // Minimum size: VK (248) + Proof (128) + at least 1 public input (32) = 408 bytes
    if instruction_data.len() < 408 {
        msg!("Invalid instruction data length: {}", instruction_data.len());
        return Err(VerifierError::InvalidInstruction.into());
    }

    // Parse components from instruction data
    let vk_bytes = &instruction_data[0..248];
    let proof_bytes = &instruction_data[248..376];
    let public_inputs_bytes = &instruction_data[376..];

    // Deserialize verifying key
    let vk: VerifyingKey<Bn254> = CanonicalDeserialize::deserialize_uncompressed(vk_bytes)
        .map_err(|_| {
            msg!("Failed to deserialize verifying key");
            VerifierError::DeserializationFailed
        })?;

    // Deserialize proof
    let proof: Proof<Bn254> = CanonicalDeserialize::deserialize_uncompressed(proof_bytes)
        .map_err(|_| {
            msg!("Failed to deserialize proof");
            VerifierError::InvalidProofFormat
        })?;

    // Deserialize public inputs (each is a field element)
    if public_inputs_bytes.len() % 32 != 0 {
        msg!("Invalid public inputs size: must be multiple of 32");
        return Err(VerifierError::InvalidProofFormat.into());
    }

    let num_inputs = public_inputs_bytes.len() / 32;
    let mut public_inputs = Vec::new();
    for i in 0..num_inputs {
        let input_bytes = &public_inputs_bytes[i*32..(i+1)*32];
        let input = <Bn254 as ark_ff::Field>::BigInt::deserialize_uncompressed(input_bytes)
            .map_err(|_| {
                msg!("Failed to deserialize public input {}", i);
                VerifierError::DeserializationFailed
            })?;
        let fe = <Bn254 as ark_ff::Field>::from_bigint(input)
            .ok_or_else(|| {
                msg!("Public input {} is out of field range", i);
                VerifierError::DeserializationFailed
            })?;
        public_inputs.push(fe);
    }

    msg!("Verifying proof with {} public inputs", public_inputs.len());

    // Perform Groth16 verification
    let is_valid = verify_proof(&vk, &proof, &public_inputs)
        .map_err(|_| {
            msg!("Proof verification failed");
            VerifierError::VerificationFailed
        })?;

    if is_valid {
        msg!("Proof verified successfully");
        Ok(())
    } else {
        msg!("Proof verification returned false");
        Err(VerifierError::VerificationFailed.into())
    }
}

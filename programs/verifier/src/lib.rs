use solana_program::{account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey, msg, program_error::ProgramError};
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Proof, VerifyingKey, verify_proof};
use ark_serialize::CanonicalDeserialize;
use ark_ff::PrimeField;
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

    // Use a byte-slice reader to sequentially deserialize vk, proof, and public inputs
    let mut bytes: &[u8] = instruction_data;

    // Deserialize verifying key
    let vk = <VerifyingKey<Bn254> as ark_serialize::CanonicalDeserialize>::deserialize_uncompressed(&mut bytes)
        .map_err(|_| {
            msg!("Failed to deserialize verifying key");
            VerifierError::DeserializationFailed
        })?;

    // Deserialize proof
    let proof = <Proof<Bn254> as ark_serialize::CanonicalDeserialize>::deserialize_uncompressed(&mut bytes)
        .map_err(|_| {
            msg!("Failed to deserialize proof");
            VerifierError::InvalidProofFormat
        })?;

    // Deserialize public inputs until EOF
    let mut public_inputs = Vec::new();
    while !bytes.is_empty() {
        let fe = <Fr as ark_serialize::CanonicalDeserialize>::deserialize_uncompressed(&mut bytes)
            .map_err(|_| {
                msg!("Failed to deserialize public input at offset");
                VerifierError::DeserializationFailed
            })?;
        public_inputs.push(fe);
    }

    msg!("Verifying proof with {} public inputs", public_inputs.len());

    // Prepare verifying key and perform Groth16 verification
    use ark_groth16::prepare_verifying_key;
    let pvk = prepare_verifying_key(&vk);
    let is_valid = verify_proof(&pvk, &proof, &public_inputs)
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

/// Helper for native verification (used in tests and by higher-level callers)
#[cfg(not(target_os = "solana"))]
pub fn verify_groth16_native(vk: &VerifyingKey<Bn254>, proof: &Proof<Bn254>, public_inputs: &[Fr]) -> Result<bool, VerifierError> {
    use ark_groth16::prepare_verifying_key;
    let pvk = prepare_verifying_key(vk);
    verify_proof(&pvk, proof, public_inputs).map_err(|_| VerifierError::VerificationFailed)
}

// On BPF we omit the native helper to keep code size small
#[cfg(target_os = "solana")]
pub fn verify_groth16_native(_vk: &VerifyingKey<Bn254>, _proof: &Proof<Bn254>, _public_inputs: &[Fr]) -> Result<bool, VerifierError> {
    Err(VerifierError::VerificationFailed)
}

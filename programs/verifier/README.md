# Solana BPF Groth16 Verifier (PoC)

This program will implement an on-chain Groth16 verifier in Rust for Solana BPF.

Planned steps:
- Select pairing-friendly curve compatible with Circom/groth16 (BN254 is typical for snarkjs)
- Verify that `ark-groth16` and required `ark-` crates can be compiled for the BPF target (`bpfel-unknown-unknown`) with `no_std` or carefully selected features
- Implement a `verify` instruction that accepts a verification key, a proof, and public inputs and runs Groth16 verifier
- Add tests and measure compute units on devnet; optimize as needed

Notes:
- This is a production-focused effort and will require performance tuning and careful dependency selection.

#!/usr/bin/env bash
set -euxo pipefail

# Requirements: circom (v2), snarkjs (v0.7)

CIRCUIT=multiplier
OUTDIR=circuits/${CIRCUIT}_build
mkdir -p ${OUTDIR}

# Compile
circom circuits/${CIRCUIT}.circom --r1cs --wasm --sym -o ${OUTDIR}

# Powers of Tau
snarkjs powersoftau new bn128 12 ${OUTDIR}/pot12_0000.ptau -v
snarkjs powersoftau contribute ${OUTDIR}/pot12_0000.ptau ${OUTDIR}/pot12_0001.ptau --name="first contribution" -v

# Setup
snarkjs groth16 setup ${OUTDIR}/${CIRCUIT}.r1cs ${OUTDIR}/pot12_0001.ptau ${OUTDIR}/${CIRCUIT}_zkey
snarkjs zkey export verificationkey ${OUTDIR}/${CIRCUIT}_zkey ${OUTDIR}/verification_key.json

# Create witness
node ${OUTDIR}/${CIRCUIT}_js/generate_witness.js ${OUTDIR}/${CIRCUIT}_js/${CIRCUIT}.wasm circuits/input.json ${OUTDIR}/witness.wtns

# Prove
snarkjs groth16 prove ${OUTDIR}/${CIRCUIT}_zkey ${OUTDIR}/witness.wtns ${OUTDIR}/proof.json ${OUTDIR}/public.json

# Export binary files for verifier consumption
node - <<'NODE'
const fs = require('fs');
const path = require('path');
const out = 'circuits/multiplier_build';
const vk = JSON.parse(fs.readFileSync(path.join(out, 'verification_key.json')));
const proof = JSON.parse(fs.readFileSync(path.join(out, 'proof.json')));
const pub = JSON.parse(fs.readFileSync(path.join(out, 'public.json')));
fs.writeFileSync(path.join(out, 'vk.json'), JSON.stringify(vk));
fs.writeFileSync(path.join(out, 'proof.bin'), Buffer.from(JSON.stringify(proof)));
fs.writeFileSync(path.join(out, 'public.json'), JSON.stringify(pub));
console.log('Wrote vk.json, proof.bin, public.json');
NODE

echo 'Done'

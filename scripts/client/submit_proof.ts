import { execSync } from 'child_process';
import { readFileSync } from 'fs';
import path from 'path';
import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, sendAndConfirmTransaction } from '@solana/web3.js';

async function main() {
  const proverDir = path.resolve(__dirname, '../../programs/prover');
  console.log('Running prover to generate vk/proof/public inputs...');
  execSync('cargo run --release -p prover', { stdio: 'inherit', cwd: proverDir });

  const vkPath = path.join(proverDir, 'vk.bin');
  const proofPath = path.join(proverDir, 'proof.bin');
  const pubPath = path.join(proverDir, 'public_inputs.bin');

  const vk = readFileSync(vkPath);
  const proof = readFileSync(proofPath);
  const pubInputs = readFileSync(pubPath);

  const instructionData = Buffer.concat([vk, proof, pubInputs]);

  const programIdEnv = process.env.VERIFIER_PROGRAM_ID;
  if (!programIdEnv) {
    console.error('Please set VERIFIER_PROGRAM_ID env var to the deployed verifier program id');
    process.exit(1);
  }
  const programId = new PublicKey(programIdEnv);

  const connection = new Connection(process.env.RPC_URL || 'http://127.0.0.1:8899', 'confirmed');
  const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(require('fs').readFileSync(process.env.SOLANA_KEYPAIR || process.env.HOME + '/.config/solana/id.json', 'utf8'))));

  const ix = new TransactionInstruction({ keys: [], programId, data: instructionData });
  const tx = new Transaction().add(ix);

  console.log('Sending transaction to verifier program', programId.toString());
  const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
  console.log('Transaction confirmed:', sig);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

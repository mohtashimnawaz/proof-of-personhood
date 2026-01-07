import { utils, getPublicKey as nobleGetPublicKey, sign as edSign, verify as edVerify } from 'noble-ed25519';
import * as bbs from '@mattrglobal/bbs-signatures';

// Simple in-memory key stores for prototype. Swap to HSM/KMS in prod.
let privateKey: Uint8Array | null = null;
let publicKeyHex: string | null = null;
let blsKeyPair: any = null; // internal BLS keypair from mattr

export async function generateKeyPair() {
  // noble-ed25519 utils.randomPrivateKey returns Uint8Array
  privateKey = utils.randomPrivateKey();
  const pub = await nobleGetPublicKey(privateKey);
  publicKeyHex = typeof pub === 'string' ? pub : Buffer.from(pub).toString('hex');
  return { privateKey: Buffer.from(privateKey).toString('hex'), publicKey: publicKeyHex };
}

function stableStringify(obj: any): string {
  // simple canonicalization: sort object keys recursively
  if (obj === null || typeof obj !== 'object') return JSON.stringify(obj);
  if (Array.isArray(obj)) return '[' + obj.map(stableStringify).join(',') + ']';
  const keys = Object.keys(obj).sort();
  return '{' + keys.map(k => JSON.stringify(k) + ':' + stableStringify(obj[k])).join(',') + '}';
}

export async function getPublicKeyInfo() {
  if (!publicKeyHex) throw new Error('keypair not generated');
  return { alg: 'ed25519', kty: 'OKP', publicKeyHex };
}

export async function signCredential(credential: Record<string, any>) {
  if (!privateKey) throw new Error('keypair not generated');
  const payload = stableStringify(credential);
  const sig = await edSign(Buffer.from(payload), Buffer.from(privateKey));
  return Buffer.from(sig).toString('hex');
}

export async function verifyCredentialSignature(credential: Record<string, any>, signatureHex: string) {
  if (!publicKeyHex) throw new Error('keypair not generated');
  const payload = stableStringify(credential);
  const sig = Buffer.from(signatureHex, 'hex');
  const pub = Buffer.from(publicKeyHex, 'hex');
  return await edVerify(sig, Buffer.from(payload), pub);
}

// ------------------ BBS+ support (prototype) ------------------

function credentialToMessages(credential: Record<string, any>): string[] {
  // Deterministic ordering: issuer, subject, then claims sorted by key, then issuedAt
  const msgs: string[] = [];
  msgs.push(String(credential.issuer || ''));
  msgs.push(String(credential.subject || ''));
  const claims = credential.claims || {};
  const keys = Object.keys(claims).sort();
  for (const k of keys) {
    msgs.push(String(claims[k]));
  }
  msgs.push(String(credential.issuedAt || ''));
  return msgs;
}

export async function generateBbsKeyPair() {
  blsKeyPair = await bbs.generateBls12381KeyPair();
  const pub = Buffer.from(blsKeyPair.publicKey).toString('hex');
  return { blsPublicKeyHex: pub };
}

export async function signBbsCredential(credential: Record<string, any>) {
  if (!blsKeyPair) throw new Error('BLS keypair not generated');
  const messages = credentialToMessages(credential);
  // Convert the bls keypair to a BBS keypair suitable for this message count
  const bbsKeyPair = await bbs.bls12381toBbs({ keyPair: blsKeyPair, messageCount: messages.length });
  const signature = await bbs.sign({ keyPair: bbsKeyPair, messages });
  return Buffer.from(signature).toString('hex');
}

export async function verifyBbsCredential(credential: Record<string, any>, signatureHex: string) {
  if (!blsKeyPair) throw new Error('BLS keypair not generated');
  const messages = credentialToMessages(credential);
  const bbsKeyPair = await bbs.bls12381toBbs({ keyPair: blsKeyPair, messageCount: messages.length });
  const sig = Buffer.from(signatureHex, 'hex');
  // bbs.verify returns boolean
  return await bbs.verify({ publicKey: bbsKeyPair.publicKey, messages, signature: sig });
}

// Placeholder: Implement blinded BBS+ flows here in the future.
export const blindSignUnavailable = () => ({ error: 'blinded BBS+ issuance not implemented in prototype' });

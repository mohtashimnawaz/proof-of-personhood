import { utils, getPublicKey, sign, verify } from 'noble-ed25519';

// Simple in-memory key store for prototype. Swap to HSM/KMS in prod.
let privateKey: Uint8Array | null = null;
let publicKeyHex: string | null = null;

export async function generateKeyPair() {
  // noble-ed25519 utils.randomPrivateKey returns Uint8Array
  privateKey = utils.randomPrivateKey();
  const pub = await getPublicKey(privateKey);
  publicKeyHex = Buffer.from(pub).toString('hex');
  return { privateKey: Buffer.from(privateKey).toString('hex'), publicKey: publicKeyHex };
}

function stableStringify(obj: any): string {
  // simple canonicalization: sort object keys recursively
  if (obj === null || typeof obj !== 'object') return JSON.stringify(obj);
  if (Array.isArray(obj)) return '[' + obj.map(stableStringify).join(',') + ']';
  const keys = Object.keys(obj).sort();
  return '{' + keys.map(k => JSON.stringify(k) + ':' + stableStringify(obj[k])).join(',') + '}';
}

export async function getPublicKey() {
  if (!publicKeyHex) throw new Error('keypair not generated');
  return { alg: 'ed25519', kty: 'OKP', publicKeyHex };
}

export async function signCredential(credential: Record<string, any>) {
  if (!privateKey) throw new Error('keypair not generated');
  const payload = stableStringify(credential);
  const sig = await sign(Buffer.from(payload), Buffer.from(privateKey));
  return Buffer.from(sig).toString('hex');
}

export async function verifyCredentialSignature(credential: Record<string, any>, signatureHex: string) {
  if (!publicKeyHex) throw new Error('keypair not generated');
  const payload = stableStringify(credential);
  const sig = Buffer.from(signatureHex, 'hex');
  const pub = Buffer.from(publicKeyHex, 'hex');
  return await verify(sig, Buffer.from(payload), pub);
}

// Placeholder: Implement blinded BBS+ flows here in the future.
export const blindSignUnavailable = () => ({ error: 'blinded BBS+ issuance not implemented in prototype' });

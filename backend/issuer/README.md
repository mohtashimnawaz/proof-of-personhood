# Issuer prototype (backend)

This is a minimal prototype of the Issuer service used for the Proof-of-Personhood flow.

Features:
- In-memory ed25519 keypair (for prototype purposes)
- `GET /health` - health check
- `GET /keys` - returns the issuer public key
- `POST /issue` - issues a signed credential (JSON + ed25519 signature)
- **BBS+ endpoints (prototype)**:
  - `GET /bbs/keys` - returns the issuer BLS public key (hex)
  - `POST /bbs/issue` - issues a credential signed with **BBS+** (non-blinded flow in this prototype)
  - `POST /bbs/verify` - verifies a credential + BBS+ signature

Blinded issuance (for selective disclosure) is **not implemented yet** and will be added in a follow-up task.

Run locally:
- npm install
- npm run dev

Example curl:

Issue a credential:

curl -X POST http://localhost:4001/issue -H "Content-Type: application/json" -d '{"subject":"user:alice","claims":{"age":30}}'

Notes:
- This uses an ed25519 signature for fast prototyping. We will swap to BBS+ blinded issuance for selective disclosure and ZK proofs in the next iteration.
- Key management is in-memory here; production must use KMS/HSM and rotate keys.

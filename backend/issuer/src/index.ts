import express from 'express';
import bodyParser from 'body-parser';
import cors from 'cors';
import { generateKeyPair, getPublicKeyInfo, signCredential, blindSignUnavailable, generateBbsKeyPair, signBbsCredential, verifyBbsCredential } from './issuer';

const app = express();
app.use(cors());
app.use(bodyParser.json());

app.get('/health', (_req, res) => res.json({ status: 'ok' }));

app.get('/keys', async (_req, res) => {
  try {
    const k = await getPublicKeyInfo();
    res.json(k);
  } catch (e: any) {
    res.status(500).json({ error: e.message });
  }
});

app.get('/bbs/keys', async (_req, res) => {
  try {
    const k = await generateBbsKeyPair();
    res.json(k);
  } catch (e: any) {
    res.status(500).json({ error: e.message });
  }
});

app.post('/issue', async (req, res) => {
  // Request body: { subject: string, claims: { ... } }
  try {
    const body = req.body;
    if (!body || !body.subject || !body.claims) return res.status(400).json({ error: 'subject and claims are required' });

    const credential = {
      issuer: 'urn:issuer:local-prototype',
      subject: body.subject,
      claims: body.claims,
      issuedAt: new Date().toISOString()
    };

    const signature = await signCredential(credential);
    res.json({ credential, signature, alg: 'ed25519-prototype' });
  } catch (e: any) {
    res.status(500).json({ error: e.message });
  }
});

app.post('/bbs/issue', async (req, res) => {
  try {
    const body = req.body;
    if (!body || !body.subject || !body.claims) return res.status(400).json({ error: 'subject and claims are required' });
    const credential = {
      issuer: 'urn:issuer:local-prototype',
      subject: body.subject,
      claims: body.claims,
      issuedAt: new Date().toISOString()
    };
    const signature = await signBbsCredential(credential);
    res.json({ credential, signature, alg: 'bbs+ (prototype)' });
  } catch (e: any) {
    res.status(500).json({ error: e.message });
  }
});

app.post('/bbs/verify', async (req, res) => {
  try {
    const { credential, signature } = req.body;
    if (!credential || !signature) return res.status(400).json({ error: 'credential and signature are required' });
    const ok = await verifyBbsCredential(credential, signature);
    res.json(ok);
  } catch (e: any) {
    res.status(500).json({ error: e.message });
  }
});

app.post('/issue/blind-init', (_req, res) => {
  res.status(501).json(blindSignUnavailable());
});

app.post('/issue/blind-sign', (_req, res) => {
  res.status(501).json(blindSignUnavailable());
});

const PORT = process.env.PORT || 4001;

(async function init() {
  await generateKeyPair();
  await generateBbsKeyPair();
  if (process.env.NODE_ENV !== 'test') {
    app.listen(PORT, () => console.log(`Issuer prototype running on port ${PORT}`));
  }
})();

export default app;

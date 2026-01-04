import express from 'express';
import bodyParser from 'body-parser';
import cors from 'cors';
import { generateKeyPair, getPublicKey, signCredential, blindSignUnavailable } from './issuer';

const app = express();
app.use(cors());
app.use(bodyParser.json());

app.get('/health', (_req, res) => res.json({ status: 'ok' }));

app.get('/keys', async (_req, res) => {
  try {
    const k = await getPublicKey();
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

app.post('/issue/blind-init', (_req, res) => {
  res.status(501).json(blindSignUnavailable());
});

app.post('/issue/blind-sign', (_req, res) => {
  res.status(501).json(blindSignUnavailable());
});

const PORT = process.env.PORT || 4001;

async function start() {
  await generateKeyPair();
  app.listen(PORT, () => console.log(`Issuer prototype running on port ${PORT}`));
}

start();

export default app;

import request from 'supertest';
import app from '../src/index';

describe('BBS+ endpoints', () => {
  it('returns bbs keys', async () => {
    const res = await request(app).get('/bbs/keys');
    expect(res.status).toBe(200);
    expect(res.body).toHaveProperty('blsPublicKeyHex');
  });

  it('issues a BBS credential and verifies it', async () => {
    const payload = { subject: 'user:bob', claims: { age: 25, country: 'US' } };
    const issueRes = await request(app).post('/bbs/issue').send(payload);
    expect(issueRes.status).toBe(200);
    expect(issueRes.body).toHaveProperty('credential');
    expect(issueRes.body).toHaveProperty('signature');

    const verifyRes = await request(app).post('/bbs/verify').send({ credential: issueRes.body.credential, signature: issueRes.body.signature });
    expect(verifyRes.status).toBe(200);
    expect(verifyRes.body.verified).toBe(true);
  });
});

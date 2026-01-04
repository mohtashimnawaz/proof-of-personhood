import request from 'supertest';
import app from '../src/index';

describe('Issuer prototype', () => {
  it('health check', async () => {
    const res = await request(app).get('/health');
    expect(res.status).toBe(200);
    expect(res.body.status).toBe('ok');
  });

  it('returns keys', async () => {
    const res = await request(app).get('/keys');
    expect(res.status).toBe(200);
    expect(res.body).toHaveProperty('publicKeyHex');
  });

  it('issues a credential and signature', async () => {
    const payload = { subject: 'user:alice', claims: { age: 30, country: 'US' } };
    const res = await request(app).post('/issue').send(payload);
    expect(res.status).toBe(200);
    expect(res.body).toHaveProperty('credential');
    expect(res.body).toHaveProperty('signature');
    expect(res.body.credential.subject).toBe('user:alice');
  });
});

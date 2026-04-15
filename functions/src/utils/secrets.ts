import { SecretManagerServiceClient } from "@google-cloud/secret-manager";

const client = new SecretManagerServiceClient();

const cache = new Map<string, string>();

async function accessSecret(secretId: string): Promise<string> {
  if (cache.has(secretId)) {
    return cache.get(secretId)!;
  }
  const [version] = await client.accessSecretVersion({
    name: `${secretId}/versions/latest`,
  });
  const value = version.payload?.data?.toString() ?? "";
  cache.set(secretId, value);
  return value;
}

export async function getJwtSecret(): Promise<string> {
  const secretId = process.env.JWT_SECRET_ID;
  if (!secretId) throw new Error("JWT_SECRET_ID not set");
  return accessSecret(secretId);
}

export async function getVapidKeys(): Promise<{
  privateKey: string;
  publicKey: string;
}> {
  const privateSecretId = process.env.VAPID_PRIVATE_SECRET_ID;
  const publicSecretId = process.env.VAPID_PUBLIC_SECRET_ID;
  if (!privateSecretId || !publicSecretId) {
    throw new Error("VAPID secret IDs not set");
  }
  const [privateKey, publicKey] = await Promise.all([
    accessSecret(privateSecretId),
    accessSecret(publicSecretId),
  ]);
  return { privateKey, publicKey };
}

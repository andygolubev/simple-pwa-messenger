/** Typed API client for the encrypted messenger Cloud Functions API. */

const BASE_URL = import.meta.env.VITE_API_BASE_URL as string;

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function request<T>(
  path: string,
  options: RequestInit & { jwt?: string } = {}
): Promise<T> {
  const { jwt, ...rest } = options;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(jwt ? { Authorization: `Bearer ${jwt}` } : {}),
  };
  const res = await fetch(`${BASE_URL}${path}`, { ...rest, headers });
  const body = await res.json();
  if (!res.ok) {
    throw new ApiError(res.status, (body as { error?: string }).error ?? "Unknown error");
  }
  return body as T;
}

export interface AuthResponse {
  jwt: string;
  uid: string;
  isNewUser: boolean;
}

export async function authGoogle(idToken: string): Promise<AuthResponse> {
  return request<AuthResponse>("/auth/google", {
    method: "POST",
    body: JSON.stringify({ idToken }),
  });
}

export interface IdentityKeysPayload {
  identityPublicKey: string;
  signingPublicKey: string;
}

export async function publishIdentityKeys(
  jwt: string,
  payload: IdentityKeysPayload
): Promise<{ ok: boolean }> {
  return request<{ ok: boolean }>("/keys/identity", {
    method: "POST",
    jwt,
    body: JSON.stringify(payload),
  });
}

export interface SignedPreKeyUpload {
  id: number;
  publicKey: string;
  signature: string;
}

export interface OneTimePreKeyUpload {
  id: number;
  publicKey: string;
}

export async function uploadPrekeys(
  jwt: string,
  signedPreKey?: SignedPreKeyUpload,
  oneTimePreKeys?: OneTimePreKeyUpload[]
): Promise<{ ok: boolean; oneTimePreKeyCount: number }> {
  return request<{ ok: boolean; oneTimePreKeyCount: number }>("/keys/prekeys", {
    method: "POST",
    jwt,
    body: JSON.stringify({ signedPreKey, oneTimePreKeys }),
  });
}

export interface PreKeyBundle {
  identityPublicKey: string;
  signingPublicKey: string;
  signedPreKey: SignedPreKeyUpload;
  oneTimePreKey: OneTimePreKeyUpload | null;
}

export async function getPreKeyBundle(jwt: string, uid: string): Promise<PreKeyBundle> {
  return request<PreKeyBundle>(`/keys/bundle?uid=${encodeURIComponent(uid)}`, { jwt });
}

export async function getOwnPreKeyCount(jwt: string, uid: string): Promise<number> {
  const res = await request<{ oneTimePreKeyCount: number }>(
    `/keys/bundle?uid=${encodeURIComponent(uid)}&countOnly=true`,
    { jwt }
  );
  return res.oneTimePreKeyCount;
}

export interface SendMessagePayload {
  roomId: string;
  recipientUid: string;
  type: "x3dh_initial" | "ratchet";
  header: string;
  ciphertext: string;
  ephemeralPublic?: string;
  usedOneTimePreKeyId?: number;
}

export async function sendMessage(
  jwt: string,
  payload: SendMessagePayload
): Promise<{ messageId: string; serverTimestamp: string }> {
  return request<{ messageId: string; serverTimestamp: string }>("/chat/send", {
    method: "POST",
    jwt,
    body: JSON.stringify(payload),
  });
}

export interface EncryptedMessage {
  messageId: string;
  senderUid: string;
  type: "x3dh_initial" | "ratchet";
  header: string;
  ciphertext: string;
  ephemeralPublic?: string;
  usedOneTimePreKeyId?: number;
  createdAt: string;
  serverTimestamp: string;
}

export async function getChatHistory(
  jwt: string,
  roomId: string,
  options: { limit?: number; after?: string } = {}
): Promise<{ messages: EncryptedMessage[]; nextCursor: string | null }> {
  const params = new URLSearchParams({ roomId });
  if (options.limit) params.set("limit", String(options.limit));
  if (options.after) params.set("after", options.after);
  return request<{ messages: EncryptedMessage[]; nextCursor: string | null }>(
    `/chat/history?${params}`,
    { jwt }
  );
}

export async function pollRooms(
  jwt: string,
  since: string
): Promise<{ rooms: Array<{ roomId: string; participants: string[]; lastMessageAt: string }> }> {
  return request<{
    rooms: Array<{ roomId: string; participants: string[]; lastMessageAt: string }>;
  }>(`/chat/poll?since=${encodeURIComponent(since)}`, { jwt });
}

export async function subscribePush(
  jwt: string,
  subscription: PushSubscription
): Promise<{ deviceId: string }> {
  const raw = subscription.toJSON();
  return request<{ deviceId: string }>("/push/subscribe", {
    method: "POST",
    jwt,
    body: JSON.stringify({
      endpoint: subscription.endpoint,
      keys: raw.keys,
      userAgent: navigator.userAgent,
    }),
  });
}

export async function unsubscribePush(
  jwt: string,
  endpoint: string
): Promise<{ ok: boolean }> {
  return request<{ ok: boolean }>("/push/subscribe", {
    method: "DELETE",
    jwt,
    body: JSON.stringify({ endpoint }),
  });
}

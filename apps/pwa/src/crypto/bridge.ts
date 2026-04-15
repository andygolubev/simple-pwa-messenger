/**
 * Thin bridge between the WASM crypto module and application logic.
 * All crypto operations that touch private key material happen here.
 *
 * The WASM module is loaded lazily on first use to avoid blocking startup.
 */

export interface WasmModule {
  generateIdentity: () => IdentityBundle;
  generateSignedPrekey: (signingPrivate: string, preKeyId: number) => SignedPreKey;
  generateOneTimePrekeys: (startId: number, count: number) => OneTimePreKey[];
  x3dhInitiate: (
    myIdentityPrivate: string,
    theirIdentityPublic: string,
    theirSigningPublic: string,
    theirSignedPrekey: string,
    theirSignedPrekeySignature: string,
    theirOneTimePrekey?: string
  ) => X3dhInitiateResult;
  x3dhRespond: (
    myIdentityPrivate: string,
    mySignedPrekeyPrivate: string,
    myOneTimePrekeyPrivate: string | undefined,
    theirIdentityPublic: string,
    theirEphemeralPublic: string
  ) => X3dhRespondResult;
  ratchetEncrypt: (sessionState: string, plaintext: string) => RatchetEncryptResult;
  ratchetDecrypt: (
    sessionState: string,
    header: string,
    ciphertext: string
  ) => RatchetDecryptResult;
  verifyIdentitySignature: (
    signingPublic: string,
    message: string,
    signature: string
  ) => boolean;
}

export interface IdentityBundle {
  identityPrivateKey: string;
  identityPublicKey: string;
  signingPrivateKey: string;
  signingPublicKey: string;
}

export interface SignedPreKey {
  id: number;
  publicKey: string;
  signature: string;
  privateKey: string;
}

export interface OneTimePreKey {
  id: number;
  publicKey: string;
  privateKey: string;
}

export interface X3dhInitiateResult {
  sessionState: string;
  ephemeralPublic: string;
  usedOneTimePreKeyId?: number;
}

export interface X3dhRespondResult {
  sessionState: string;
}

export interface RatchetEncryptResult {
  updatedSessionState: string;
  message: { header: string; ciphertext: string };
}

export interface RatchetDecryptResult {
  updatedSessionState: string;
  plaintext: string;
}

let wasmModule: WasmModule | null = null;

export async function loadCrypto(): Promise<WasmModule> {
  if (wasmModule) return wasmModule;

  // Dynamic import of the wasm-pack generated module
  // The module is built by `crates/messenger-crypto/build-wasm.sh` and output to `apps/pwa/pkg/`
  const wasm = await import("../../../pkg/messenger_crypto") as unknown as WasmModule;
  wasmModule = wasm;
  return wasm;
}

export async function generateIdentity(): Promise<IdentityBundle> {
  const crypto = await loadCrypto();
  return crypto.generateIdentity();
}

export async function generateSignedPrekey(
  signingPrivate: string,
  preKeyId: number
): Promise<SignedPreKey> {
  const crypto = await loadCrypto();
  return crypto.generateSignedPrekey(signingPrivate, preKeyId);
}

export async function generateOneTimePrekeys(
  startId: number,
  count: number
): Promise<OneTimePreKey[]> {
  const crypto = await loadCrypto();
  return crypto.generateOneTimePrekeys(startId, count);
}

export async function x3dhInitiate(
  myIdentityPrivate: string,
  theirIdentityPublic: string,
  theirSigningPublic: string,
  theirSignedPrekey: string,
  theirSignedPrekeySignature: string,
  theirOneTimePrekey?: string
): Promise<X3dhInitiateResult> {
  const crypto = await loadCrypto();
  return crypto.x3dhInitiate(
    myIdentityPrivate,
    theirIdentityPublic,
    theirSigningPublic,
    theirSignedPrekey,
    theirSignedPrekeySignature,
    theirOneTimePrekey
  );
}

export async function x3dhRespond(
  myIdentityPrivate: string,
  mySignedPrekeyPrivate: string,
  myOneTimePrekeyPrivate: string | undefined,
  theirIdentityPublic: string,
  theirEphemeralPublic: string
): Promise<X3dhRespondResult> {
  const crypto = await loadCrypto();
  return crypto.x3dhRespond(
    myIdentityPrivate,
    mySignedPrekeyPrivate,
    myOneTimePrekeyPrivate,
    theirIdentityPublic,
    theirEphemeralPublic
  );
}

export async function ratchetEncrypt(
  sessionState: string,
  plaintext: string
): Promise<RatchetEncryptResult> {
  const crypto = await loadCrypto();
  return crypto.ratchetEncrypt(sessionState, plaintext);
}

export async function ratchetDecrypt(
  sessionState: string,
  header: string,
  ciphertext: string
): Promise<RatchetDecryptResult> {
  const crypto = await loadCrypto();
  return crypto.ratchetDecrypt(sessionState, header, ciphertext);
}

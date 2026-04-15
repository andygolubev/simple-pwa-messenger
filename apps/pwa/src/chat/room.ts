/**
 * Room ID derivation and session management for 1:1 encrypted chats.
 *
 * roomId = SHA-256(sort([uid1, uid2]).join(":"))  — matches server logic.
 */

export async function deriveRoomId(uid1: string, uid2: string): Promise<string> {
  const sorted = [uid1, uid2].sort().join(":");
  const encoded = new TextEncoder().encode(sorted);
  const hashBuffer = await crypto.subtle.digest("SHA-256", encoded);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
}

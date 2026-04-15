/**
 * API client contract tests — verify request shapes and error handling.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock fetch globally
const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

// Set required env var
vi.stubGlobal("import.meta", { env: { VITE_API_BASE_URL: "https://api.example.com" } });

function mockJsonResponse(body: unknown, status = 200) {
  return Promise.resolve({
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(body),
  });
}

// Re-import after mocking global
let authGoogle: typeof import("../api/client").authGoogle;
let ApiError: typeof import("../api/client").ApiError;

beforeEach(async () => {
  vi.resetModules();
  mockFetch.mockReset();
  const mod = await import("../api/client");
  authGoogle = mod.authGoogle;
  ApiError = mod.ApiError;
});

describe("authGoogle", () => {
  it("sends POST with idToken and returns jwt/uid/isNewUser", async () => {
    mockFetch.mockReturnValueOnce(
      mockJsonResponse({ jwt: "token123", uid: "user-1", isNewUser: false })
    );

    const result = await authGoogle("google-id-token");

    expect(mockFetch).toHaveBeenCalledWith(
      "https://api.example.com/auth/google",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ idToken: "google-id-token" }),
      })
    );
    expect(result.jwt).toBe("token123");
    expect(result.uid).toBe("user-1");
    expect(result.isNewUser).toBe(false);
  });

  it("throws ApiError on 401", async () => {
    mockFetch.mockReturnValueOnce(
      mockJsonResponse({ error: "Invalid token" }, 401)
    );

    await expect(authGoogle("bad-token")).rejects.toBeInstanceOf(ApiError);
  });
});

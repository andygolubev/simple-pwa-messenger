import express from "express";
import request from "supertest";
import { app } from "../src/index";

// Mock external dependencies
jest.mock("../src/utils/secrets", () => ({
  getJwtSecret: jest.fn().mockResolvedValue("test-secret-min-32-chars-long!!"),
  getVapidKeys: jest.fn().mockResolvedValue({
    publicKey: "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nksh8U",
    privateKey: "UUxI4O8-FbRouAevSmBQ6co550gBRqK-hLRJEJMGbUA",
  }),
}));

jest.mock("firebase-admin", () => ({
  initializeApp: jest.fn(),
  auth: jest.fn().mockReturnValue({
    verifyIdToken: jest.fn().mockResolvedValue({
      uid: "user-123",
      email: "test@example.com",
      name: "Test User",
    }),
  }),
}));

jest.mock("../src/utils/firestore", () => {
  const mockSet = jest.fn().mockResolvedValue(undefined);
  const mockGet = jest.fn().mockResolvedValue({ exists: false });
  const mockDoc = jest.fn().mockReturnValue({
    set: mockSet,
    get: mockGet,
  });
  const mockCollection = jest.fn().mockReturnValue({ doc: mockDoc });
  return {
    getDb: jest.fn().mockReturnValue({ collection: mockCollection }),
  };
});

describe("POST /auth/google", () => {
  it("returns 400 if idToken is missing", async () => {
    const res = await request(app).post("/auth/google").send({});
    expect(res.status).toBe(400);
    expect(res.body.error).toMatch(/idToken/);
  });

  it("returns jwt, uid, and isNewUser on valid token", async () => {
    const res = await request(app)
      .post("/auth/google")
      .send({ idToken: "valid-google-token" });
    expect(res.status).toBe(200);
    expect(res.body.uid).toBe("user-123");
    expect(typeof res.body.jwt).toBe("string");
    expect(typeof res.body.isNewUser).toBe("boolean");
  });
});

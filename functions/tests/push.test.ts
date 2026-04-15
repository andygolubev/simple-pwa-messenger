import request from "supertest";
import jwt from "jsonwebtoken";
import { app } from "../src/index";

const JWT_SECRET = "test-secret-min-32-chars-long!!";

jest.mock("../src/utils/secrets", () => ({
  getJwtSecret: jest.fn().mockResolvedValue("test-secret-min-32-chars-long!!"),
  getVapidKeys: jest.fn().mockResolvedValue({
    publicKey: "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nksh8U",
    privateKey: "UUxI4O8-FbRouAevSmBQ6co550gBRqK-hLRJEJMGbUA",
  }),
}));

const mockDeviceRef = {
  set: jest.fn().mockResolvedValue(undefined),
  get: jest.fn().mockResolvedValue({ exists: true }),
  delete: jest.fn().mockResolvedValue(undefined),
};

jest.mock("../src/utils/firestore", () => ({
  getDb: jest.fn().mockReturnValue({
    collection: jest.fn().mockReturnValue({
      doc: jest.fn().mockReturnValue({
        collection: jest.fn().mockReturnValue({
          doc: jest.fn().mockReturnValue(mockDeviceRef),
        }),
      }),
    }),
  }),
}));

function makeAuthHeader(uid: string): string {
  const token = jwt.sign({ uid }, JWT_SECRET, { expiresIn: "1h" });
  return `Bearer ${token}`;
}

describe("POST /push/subscribe", () => {
  it("returns 401 without auth", async () => {
    const res = await request(app).post("/push/subscribe").send({});
    expect(res.status).toBe(401);
  });

  it("returns 400 if endpoint or keys missing", async () => {
    const res = await request(app)
      .post("/push/subscribe")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({ endpoint: "https://push.example.com/sub" });
    expect(res.status).toBe(400);
  });

  it("stores subscription and returns deviceId", async () => {
    const res = await request(app)
      .post("/push/subscribe")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({
        endpoint: "https://push.example.com/sub/abc",
        keys: { p256dh: "testP256Key", auth: "testAuth" },
      });
    expect(res.status).toBe(200);
    expect(typeof res.body.deviceId).toBe("string");
  });
});

describe("DELETE /push/subscribe", () => {
  it("removes subscription", async () => {
    const res = await request(app)
      .delete("/push/subscribe")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({ endpoint: "https://push.example.com/sub/abc" });
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
  });

  it("returns 400 if endpoint missing", async () => {
    const res = await request(app)
      .delete("/push/subscribe")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({});
    expect(res.status).toBe(400);
  });
});

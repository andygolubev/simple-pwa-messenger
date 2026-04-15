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

const mockMessages = [
  {
    id: "msg-1",
    data: () => ({
      messageId: "msg-1",
      senderUid: "user-1",
      type: "ratchet",
      header: "hdr",
      ciphertext: "ct",
      createdAt: "2024-01-01T00:00:00Z",
      serverTimestamp: "2024-01-01T00:00:00Z",
    }),
  },
];

const mockAdd = jest.fn().mockResolvedValue({ id: "new-msg-id" });
const mockRunTransaction = jest.fn(async (cb: (tx: unknown) => Promise<void>) => {
  const tx = {
    get: jest.fn().mockResolvedValue({
      exists: true,
      data: () => ({ oneTimePreKeys: [{ id: 1, publicKey: "k1" }] }),
    }),
    update: jest.fn(),
    set: jest.fn(),
  };
  await cb(tx);
});

const mockMessagesQuery = {
  orderBy: jest.fn().mockReturnThis(),
  limit: jest.fn().mockReturnThis(),
  startAfter: jest.fn().mockReturnThis(),
  get: jest.fn().mockResolvedValue({ docs: mockMessages }),
};

const mockRoomsSnap = {
  docs: [
    {
      id: "room-abc",
      data: () => ({
        participants: ["user-1", "user-2"],
        lastMessageAt: "2024-01-02T00:00:00Z",
      }),
    },
  ],
};

const mockDb = {
  collection: jest.fn((name: string) => {
    if (name === "users") {
      return {
        doc: jest.fn().mockReturnValue({
          get: jest.fn().mockResolvedValue({ exists: true }),
        }),
      };
    }
    if (name === "rooms") {
      return {
        doc: jest.fn().mockReturnValue({
          set: jest.fn().mockResolvedValue(undefined),
          get: jest.fn().mockResolvedValue({
            exists: true,
            data: () => ({ participants: ["user-1", "user-2"] }),
          }),
          collection: jest.fn().mockReturnValue({
            add: mockAdd,
            ...mockMessagesQuery,
          }),
        }),
        where: jest.fn().mockReturnThis(),
        orderBy: jest.fn().mockReturnThis(),
        get: jest.fn().mockResolvedValue(mockRoomsSnap),
      };
    }
    if (name === "preKeyBundles") {
      return {
        doc: jest.fn().mockReturnValue({
          get: jest.fn().mockResolvedValue({ exists: true, data: () => ({}) }),
          update: jest.fn().mockResolvedValue(undefined),
        }),
      };
    }
    if (name === "pushSubscriptions") {
      return {
        doc: jest.fn().mockReturnValue({
          collection: jest.fn().mockReturnValue({
            get: jest.fn().mockResolvedValue({ docs: [] }),
          }),
        }),
      };
    }
    return { doc: jest.fn().mockReturnValue({}) };
  }),
  runTransaction: mockRunTransaction,
};

jest.mock("../src/utils/firestore", () => ({
  getDb: jest.fn().mockReturnValue(mockDb),
}));

function makeAuthHeader(uid: string): string {
  const token = jwt.sign({ uid }, JWT_SECRET, { expiresIn: "1h" });
  return `Bearer ${token}`;
}

describe("POST /chat/send", () => {
  it("returns 401 without auth", async () => {
    const res = await request(app).post("/chat/send").send({});
    expect(res.status).toBe(401);
  });

  it("returns 400 if required fields missing", async () => {
    const res = await request(app)
      .post("/chat/send")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({ roomId: "room-1" });
    expect(res.status).toBe(400);
  });

  it("stores ratchet message and returns messageId", async () => {
    const res = await request(app)
      .post("/chat/send")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({
        roomId: "room-abc",
        recipientUid: "user-2",
        type: "ratchet",
        header: "base64hdr",
        ciphertext: "base64ct",
      });
    expect(res.status).toBe(200);
    expect(res.body.messageId).toBe("new-msg-id");
    expect(res.body.serverTimestamp).toBeDefined();
  });

  it("stores x3dh_initial message and returns messageId", async () => {
    const res = await request(app)
      .post("/chat/send")
      .set("Authorization", makeAuthHeader("user-1"))
      .send({
        roomId: "room-abc",
        recipientUid: "user-2",
        type: "x3dh_initial",
        header: "base64hdr",
        ciphertext: "base64ct",
        ephemeralPublic: "base64eph",
        usedOneTimePreKeyId: 1,
      });
    expect(res.status).toBe(200);
    expect(res.body.messageId).toBeDefined();
  });
});

describe("GET /chat/history", () => {
  it("returns 400 without roomId", async () => {
    const res = await request(app)
      .get("/chat/history")
      .set("Authorization", makeAuthHeader("user-1"));
    expect(res.status).toBe(400);
  });

  it("returns messages for a room", async () => {
    const res = await request(app)
      .get("/chat/history?roomId=room-abc")
      .set("Authorization", makeAuthHeader("user-1"));
    expect(res.status).toBe(200);
    expect(Array.isArray(res.body.messages)).toBe(true);
  });
});

describe("GET /chat/poll", () => {
  it("returns 400 without since", async () => {
    const res = await request(app)
      .get("/chat/poll")
      .set("Authorization", makeAuthHeader("user-1"));
    expect(res.status).toBe(400);
  });

  it("returns rooms with new activity", async () => {
    const res = await request(app)
      .get("/chat/poll?since=2024-01-01T00:00:00Z")
      .set("Authorization", makeAuthHeader("user-1"));
    expect(res.status).toBe(200);
    expect(Array.isArray(res.body.rooms)).toBe(true);
  });
});

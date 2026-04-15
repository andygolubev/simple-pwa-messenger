import * as ff from "@google-cloud/functions-framework";
import express from "express";
import { handleAuthGoogle } from "./handlers/auth";
import { handlePublishIdentityKeys, handleUploadPrekeys, handleGetPreKeyBundle } from "./handlers/keys";
import { handleSendMessage, handleGetHistory, handlePollRooms } from "./handlers/chat";
import { handleSubscribePush, handleUnsubscribePush } from "./handlers/push";
import { requireAuth } from "./middleware/auth";

const app = express();
app.use(express.json());

// Auth
app.post("/auth/google", handleAuthGoogle);

// Keys (authenticated)
app.post("/keys/identity", requireAuth, handlePublishIdentityKeys);
app.post("/keys/prekeys", requireAuth, handleUploadPrekeys);
app.get("/keys/bundle", requireAuth, handleGetPreKeyBundle);

// Chat (authenticated)
app.post("/chat/send", requireAuth, handleSendMessage);
app.get("/chat/history", requireAuth, handleGetHistory);
app.get("/chat/poll", requireAuth, handlePollRooms);

// Push (authenticated)
app.post("/push/subscribe", requireAuth, handleSubscribePush);
app.delete("/push/subscribe", requireAuth, handleUnsubscribePush);

// Health check
app.get("/healthz", (_req, res) => res.json({ ok: true }));

ff.http("handler", app);

export { app };

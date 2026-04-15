import { Request, Response, NextFunction } from "express";
import jwt from "jsonwebtoken";
import { getJwtSecret } from "../utils/secrets";

export interface AuthenticatedRequest extends Request {
  uid?: string;
}

export async function requireAuth(
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): Promise<void> {
  const authHeader = req.headers.authorization;
  if (!authHeader || !authHeader.startsWith("Bearer ")) {
    res.status(401).json({ error: "Missing or malformed Authorization header" });
    return;
  }

  const token = authHeader.slice(7);
  try {
    const secret = await getJwtSecret();
    const payload = jwt.verify(token, secret) as { uid: string };
    req.uid = payload.uid;
    next();
  } catch {
    res.status(401).json({ error: "Invalid or expired token" });
  }
}

export async function mintJwt(uid: string): Promise<string> {
  const secret = await getJwtSecret();
  return jwt.sign({ uid }, secret, { expiresIn: "1h" });
}

import React, { createContext, useContext, useEffect, useState } from "react";
import * as db from "../store/db";
import { authGoogle } from "../api/client";
import { bootstrapKeys, checkAndReplenishPrekeys } from "./keyBootstrap";
import { subscribeToPush } from "../push/subscription";

interface AuthState {
  uid: string | null;
  jwt: string | null;
  loading: boolean;
}

interface AuthContextValue extends AuthState {
  loginWithGoogle: (idToken: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<AuthState>({ uid: null, jwt: null, loading: true });

  useEffect(() => {
    // Restore persisted session on app start
    db.loadAuthState()
      .then((auth) => {
        if (auth) {
          setState({ uid: auth.uid, jwt: auth.jwt, loading: false });
          // Background replenishment check
          checkAndReplenishPrekeys(auth.uid, auth.jwt).catch(console.error);
        } else {
          setState({ uid: null, jwt: null, loading: false });
        }
      })
      .catch(() => setState({ uid: null, jwt: null, loading: false }));
  }, []);

  async function loginWithGoogle(idToken: string): Promise<void> {
    const response = await authGoogle(idToken);

    if (response.isNewUser) {
      await bootstrapKeys(response.uid, response.jwt);
    } else {
      await checkAndReplenishPrekeys(response.uid, response.jwt);
    }

    // JWT is valid for 1 hour
    await db.saveAuthState({
      uid: response.uid,
      jwt: response.jwt,
      expiresAt: Date.now() + 60 * 60 * 1000,
    });

    setState({ uid: response.uid, jwt: response.jwt, loading: false });

    // Subscribe to push notifications (best-effort)
    subscribeToPush(response.jwt).catch(console.error);
  }

  async function logout(): Promise<void> {
    await db.clearAuthState();
    setState({ uid: null, jwt: null, loading: false });
  }

  return (
    <AuthContext.Provider value={{ ...state, loginWithGoogle, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}

import React from "react";
import { useAuth } from "./auth/AuthContext";

export function App() {
  const { uid, loading, loginWithGoogle, logout } = useAuth();

  if (loading) {
    return <div style={{ padding: 24 }}>Loading…</div>;
  }

  if (!uid) {
    return <LoginPage onLogin={loginWithGoogle} />;
  }

  return <MessengerPage uid={uid} onLogout={logout} />;
}

function LoginPage({ onLogin }: { onLogin: (token: string) => Promise<void> }) {
  const [error, setError] = React.useState<string | null>(null);
  const [busy, setBusy] = React.useState(false);

  async function handleSignIn() {
    setError(null);
    setBusy(true);
    try {
      // In a real app, initiate Google OAuth flow here and obtain idToken.
      // For now we show a placeholder input for the token.
      const token = (document.getElementById("id-token") as HTMLInputElement)?.value;
      if (!token) {
        setError("Please enter a Google ID token");
        return;
      }
      await onLogin(token);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Login failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div style={{ padding: 24, maxWidth: 400, margin: "80px auto", fontFamily: "sans-serif" }}>
      <h1>Encrypted Messenger</h1>
      <p>End-to-end encrypted using the Signal Protocol.</p>
      <div style={{ marginTop: 24 }}>
        <input
          id="id-token"
          type="text"
          placeholder="Google ID token"
          style={{ width: "100%", padding: 8, marginBottom: 8 }}
        />
        <button onClick={handleSignIn} disabled={busy} style={{ width: "100%", padding: 10 }}>
          {busy ? "Signing in…" : "Sign in with Google"}
        </button>
      </div>
      {error && <p style={{ color: "red", marginTop: 12 }}>{error}</p>}
    </div>
  );
}

function MessengerPage({ uid, onLogout }: { uid: string; onLogout: () => Promise<void> }) {
  return (
    <div style={{ padding: 24, fontFamily: "sans-serif" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h2>Messenger</h2>
        <button onClick={onLogout}>Sign out</button>
      </div>
      <p style={{ color: "#666" }}>Signed in as: {uid}</p>
      <p style={{ marginTop: 24, color: "#999" }}>
        Chat interface goes here. Select a contact to start an encrypted conversation.
      </p>
    </div>
  );
}

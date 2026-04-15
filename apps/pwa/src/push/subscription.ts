/**
 * Web Push subscription management.
 *
 * Privacy guarantee: push payload contains only notification hints (title, body, roomId).
 * The server never sends plaintext message content via push.
 */

import * as api from "../api/client";

const VAPID_PUBLIC_KEY = import.meta.env.VITE_VAPID_PUBLIC_KEY as string;

function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
  const raw = window.atob(base64);
  return Uint8Array.from(Array.from(raw).map((c) => c.charCodeAt(0)));
}

export async function registerServiceWorker(): Promise<ServiceWorkerRegistration> {
  if (!("serviceWorker" in navigator)) {
    throw new Error("Service workers are not supported in this browser");
  }
  return navigator.serviceWorker.register("/sw.js");
}

export async function subscribeToPush(jwt: string): Promise<void> {
  const registration = await registerServiceWorker();
  await navigator.serviceWorker.ready;

  // Check for existing subscription
  const existing = await registration.pushManager.getSubscription();
  if (existing) {
    // Already subscribed — re-register with server (idempotent)
    await api.subscribePush(jwt, existing);
    return;
  }

  if (!VAPID_PUBLIC_KEY) {
    console.warn("VITE_VAPID_PUBLIC_KEY not set — push notifications disabled");
    return;
  }

  const subscription = await registration.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: urlBase64ToUint8Array(VAPID_PUBLIC_KEY),
  });

  await api.subscribePush(jwt, subscription);
}

export async function unsubscribeFromPush(jwt: string): Promise<void> {
  if (!("serviceWorker" in navigator)) return;

  const registration = await navigator.serviceWorker.getRegistration();
  if (!registration) return;

  const subscription = await registration.pushManager.getSubscription();
  if (!subscription) return;

  await api.unsubscribePush(jwt, subscription.endpoint);
  await subscription.unsubscribe();
}

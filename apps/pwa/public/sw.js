/**
 * Service Worker for the encrypted messenger PWA.
 *
 * Handles:
 * - Web Push notifications (privacy-preserving: payload contains only hints)
 * - Cache-first strategy for static assets
 * - Notification click routing to the relevant room
 */

const CACHE_NAME = "messenger-v1";
const PRECACHE_URLS = ["/", "/index.html", "/manifest.json"];

// ─── Install & activate ───────────────────────────────────────────────────────

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(PRECACHE_URLS))
  );
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)))
    )
  );
  self.clients.claim();
});

// ─── Fetch (cache-first for navigation, network-first for API) ────────────────

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (url.pathname.startsWith("/auth/") || url.pathname.startsWith("/keys/") ||
      url.pathname.startsWith("/chat/") || url.pathname.startsWith("/push/")) {
    // API requests: always network
    return;
  }
  event.respondWith(
    caches.match(event.request).then((cached) => cached ?? fetch(event.request))
  );
});

// ─── Push notifications ───────────────────────────────────────────────────────

self.addEventListener("push", (event) => {
  if (!event.data) return;

  let data = {};
  try {
    data = event.data.json();
  } catch {
    // Non-JSON push payload — show a generic notification
  }

  const title = data.title ?? "New message";
  const body = data.body ?? "You have a new encrypted message";
  const roomId = data.roomId;

  event.waitUntil(
    self.registration.showNotification(title, {
      body,
      icon: "/icon-192.png",
      badge: "/icon-192.png",
      tag: roomId ? `room-${roomId}` : "messenger",
      data: { roomId },
    })
  );
});

// ─── Notification click ───────────────────────────────────────────────────────

self.addEventListener("notificationclick", (event) => {
  event.notification.close();

  const roomId = event.notification.data?.roomId;
  const targetUrl = roomId ? `/?room=${roomId}` : "/";

  event.waitUntil(
    self.clients.matchAll({ type: "window", includeUncontrolled: true }).then((clients) => {
      const existing = clients.find((c) => c.url.includes(self.location.origin));
      if (existing) {
        existing.focus();
        existing.navigate(targetUrl);
      } else {
        self.clients.openWindow(targetUrl);
      }
    })
  );
});

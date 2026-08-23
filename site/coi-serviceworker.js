if (typeof window === "undefined") {
  self.addEventListener("install", () => self.skipWaiting());
  self.addEventListener("activate", (event) => event.waitUntil(self.clients.claim()));
  self.addEventListener("fetch", (event) => {
    if (event.request.cache === "only-if-cached" && event.request.mode !== "same-origin") return;
    event.respondWith(fetch(event.request).then((response) => {
      if (response.status === 0) return response;
      const headers = new Headers(response.headers);
      headers.set("Cross-Origin-Embedder-Policy", "require-corp");
      headers.set("Cross-Origin-Opener-Policy", "same-origin");
      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers,
      });
    }));
  });
} else if (!window.crossOriginIsolated && "serviceWorker" in navigator) {
  navigator.serviceWorker.addEventListener("controllerchange", () => location.reload());
  navigator.serviceWorker.register(document.currentScript.src).then((registration) => {
    if (registration.active && !navigator.serviceWorker.controller) location.reload();
  }).catch((error) => console.error("cross-origin isolation failed", error));
}

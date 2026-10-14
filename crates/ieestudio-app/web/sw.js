// Guarda la aplicación en el navegador para que abra al instante y funcione
// sin internet. La versión cambia con cada publicación; al cambiar, se
// descarga la nueva y se borra la anterior.
const VERSION = "__VERSION__";
const CACHE = "ieestudio-" + VERSION;
const ARCHIVOS = [
  "./",
  "./index.html",
  "./ieestudio-app.js",
  "./ieestudio-app_bg.wasm",
  "./manifest.webmanifest",
  "./icono.svg",
  "./icono-180.png",
  "./icono-192.png",
  "./icono-512.png",
  "./icono-maskable-512.png",
];

self.addEventListener("install", (evento) => {
  evento.waitUntil(caches.open(CACHE).then((c) => c.addAll(ARCHIVOS)).then(() => self.skipWaiting()));
});

self.addEventListener("activate", (evento) => {
  evento.waitUntil(
    caches.keys()
      .then((claves) => Promise.all(claves.filter((k) => k.startsWith("ieestudio-") && k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (evento) => {
  if (evento.request.method !== "GET") return;
  evento.respondWith(
    caches.match(evento.request, { ignoreSearch: true }).then((guardado) => guardado || fetch(evento.request))
  );
});

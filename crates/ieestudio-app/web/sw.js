// Guarda la aplicación en el navegador para que abra al instante y funcione
// sin internet. La versión cambia con cada publicación; al cambiar, se
// descarga la nueva y se borra la anterior. El proyecto no vive aquí sino en
// el almacenamiento local del sitio, y este archivo nunca lo toca.
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

// «reload» pide cada archivo al servidor sin pasar por la caché HTTP del
// navegador: si no, una versión nueva podía guardar el programa anterior
// (GitHub Pages lo deja en caché diez minutos) o, peor, mezclar el .js nuevo
// con el .wasm viejo.
self.addEventListener("install", (evento) => {
  evento.waitUntil(
    caches.open(CACHE)
      .then((c) => c.addAll(ARCHIVOS.map((url) => new Request(url, { cache: "reload" }))))
      .then(() => self.skipWaiting())
  );
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

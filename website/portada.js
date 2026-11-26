// Portada de ieestudio.com: escena 3D del tablero, simbología, vistas del
// programa, flujo de trabajo, hojas de la memoria y visor de imágenes.
// La escena se carga después del contenido y solo si hay WebGL; sin ella la
// portada muestra el alzado del tablero y todo lo demás funciona igual.

const reducido = matchMedia("(prefers-reduced-motion: reduce)").matches;
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));

// ---------------------------------------------------------------- simbología
const ficha = $("#ficha-pieza");
const botonesPieza = $$(".simbologia button[data-pieza]");
let escena = null;
let actual = null;

function seleccionar(id) {
  actual = actual === id ? null : id;
  botonesPieza.forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.pieza === actual)));
  $$(".globo").forEach((g) => g.classList.toggle("activo", g.dataset.pieza === actual));
  const t = actual && $(`template[data-pieza="${actual}"]`);
  if (t) {
    ficha.replaceChildren(t.content.cloneNode(true));
    ficha.hidden = false;
  } else {
    ficha.hidden = true;
    ficha.replaceChildren();
  }
  escena && escena.seleccionar(actual);
  dibujarLineas();
}
botonesPieza.forEach((b) => b.addEventListener("click", () => seleccionar(b.dataset.pieza)));
$$(".globo").forEach((g) => g.addEventListener("click", () => seleccionar(g.dataset.pieza)));

// Globos con su línea de referencia, como en un plano de ensamble.
const capa = $("#globos");
const lineas = capa && $("svg", capa);
const SEP = { principal: [-100, -60], medicion: [110, -50], derivados: [-100, -30], barras: [120, 30], conductores: [-40, 95], canalizacion: [-90, 10], tierra: [130, 30] };
let ultimas = null;
function dibujarLineas(pos = ultimas) {
  if (!pos || !lineas) return;
  ultimas = pos;
  let svg = "";
  for (const g of $$(".globo", capa)) {
    const k = g.dataset.pieza;
    const a = pos[k];
    if (!a) continue;
    const [dx, dy] = SEP[k] || [100, 0];
    const x = a.x + dx, y = a.y + dy;
    g.style.left = x + "px";
    g.style.top = y + "px";
    const len = Math.hypot(dx, dy) || 1;
    const x1 = x - (dx / len) * 15, y1 = y - (dy / len) * 15;
    const cls = k === actual ? ' class="activo-linea"' : "";
    svg += `<line${cls} x1="${x1.toFixed(1)}" y1="${y1.toFixed(1)}" x2="${a.x.toFixed(1)}" y2="${a.y.toFixed(1)}"/><circle class="ancla" cx="${a.x.toFixed(1)}" cy="${a.y.toFixed(1)}" r="2.5"/>`;
  }
  lineas.innerHTML = svg;
}

// Punta con el circuito bajo el puntero.
const punta = $("#punta");
function pasar(info) {
  if (!punta) return;
  if (!info) { punta.classList.remove("ver"); return; }
  punta.innerHTML = `<b>${info.id}</b> ${info.desc}<br>${info.polos}P-${info.amp} A`;
  punta.style.left = info.x + "px";
  punta.style.top = info.y + "px";
  punta.classList.add("ver");
}

// ---------------------------------------------------------------- carga de la escena
function hayWebGL() {
  try {
    const c = document.createElement("canvas");
    return !!(c.getContext("webgl2") || c.getContext("webgl"));
  } catch (e) {
    return false;
  }
}
async function cargarEscena() {
  const cont = $("#escena");
  if (!cont || !hayWebGL()) return;
  const ahorro = navigator.connection && navigator.connection.saveData;
  if (ahorro) return;
  try {
    await Promise.race([
      Promise.all([document.fonts.load("26px 'Plex Mono'"), document.fonts.load("600 40px Manrope")]),
      new Promise((r) => setTimeout(r, 1500)),
    ]);
    const { iniciar } = await import("./escena.js");
    const compacto = matchMedia("(max-width: 900px), (pointer: coarse)").matches;
    escena = iniciar({ contenedor: cont, reducido, compacto, alSeleccionar: (id) => { if (id !== actual) seleccionar(id); }, alPasar: pasar });
    escena.alColocar(dibujarLineas);
    await escena.primerCuadro;
    cont.classList.add("lista");
    if (actual) escena.seleccionar(actual);
  } catch (e) {
    // Sin escena, la portada conserva el alzado del tablero.
    console.warn("IEEStudio: la escena 3D no se pudo cargar; se muestra el dibujo estático.", e);
  }
}
const arrancar = () => ("requestIdleCallback" in window ? requestIdleCallback(cargarEscena, { timeout: 1200 }) : setTimeout(cargarEscena, 200));
if (document.readyState === "complete") arrancar();
else addEventListener("load", arrancar, { once: true });

// ---------------------------------------------------------------- pestañas accesibles
function pestanas(lista, alElegir) {
  const tabs = $$('[role="tab"]', lista);
  const elegir = (t, enfocar) => {
    tabs.forEach((x) => {
      const si = x === t;
      x.setAttribute("aria-selected", String(si));
      x.tabIndex = si ? 0 : -1;
    });
    if (enfocar) t.focus();
    alElegir(t);
  };
  tabs.forEach((t, i) => {
    t.addEventListener("click", () => elegir(t));
    t.addEventListener("keydown", (e) => {
      const sig = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 }[e.key];
      if (sig) { e.preventDefault(); elegir(tabs[(i + sig + tabs.length) % tabs.length], true); }
      if (e.key === "Home") { e.preventDefault(); elegir(tabs[0], true); }
      if (e.key === "End") { e.preventDefault(); elegir(tabs[tabs.length - 1], true); }
    });
  });
}

// Vistas del programa: cambia la captura de la ventana y la del teléfono.
const MOVIL = { inicio: "inicio", proyecto: "proyecto", tableros: "tableros", procedimiento: "tableros", unifilar: "unifilar", revisar: "revisar", memoria: "memoria", rapido: "rapido" };
const vistas = $(".vistas");
if (vistas) {
  pestanas(vistas, (t) => {
    const v = t.dataset.vista;
    $$("#ventana-lienzo img").forEach((img) => img.classList.toggle("ver", img.dataset.vista === v));
    $$(".telefono img").forEach((img) => img.classList.toggle("ver", img.dataset.movil === MOVIL[v]));
    $("#ventana-lienzo").setAttribute("aria-labelledby", t.id);
  });
}

// Flujo de trabajo.
const pasos = $(".pasos");
if (pasos) {
  pestanas(pasos, (t) => {
    $$(".detalle-paso").forEach((d) => (d.hidden = d.id !== t.getAttribute("aria-controls")));
  });
}

// ---------------------------------------------------------------- hojas de la memoria
const hojas = $("#hojas");
if (hojas) {
  const botones = $$(".indice-memoria button");
  const papeles = $$(".hoja", hojas);
  const frente = (i) => {
    botones.forEach((b) => b.setAttribute("aria-pressed", String(+b.dataset.hoja === i)));
    papeles.forEach((p) => p.classList.toggle("al-frente", +p.dataset.hoja === i));
    hojas.classList.add("abiertas");
  };
  botones.forEach((b) => b.addEventListener("click", () => frente(+b.dataset.hoja)));
  papeles.forEach((p) => p.addEventListener("click", () => {
    const img = $("img", p);
    ampliar(img.dataset.grande || img.currentSrc || img.src, img.alt);
  }));
  // Las hojas se despliegan una sola vez, cuando la sección entra en pantalla.
  if (!reducido && "IntersectionObserver" in window) {
    const io = new IntersectionObserver(([e]) => {
      if (e.isIntersecting) { hojas.classList.add("abiertas"); io.disconnect(); }
    }, { rootMargin: "0px 0px -30% 0px" });
    io.observe(hojas);
  } else {
    hojas.classList.add("abiertas");
  }
}

// ---------------------------------------------------------------- visor
const visor = $("#visor");
function ampliar(src, alt) {
  if (!visor || typeof visor.showModal !== "function") { window.open(src, "_blank", "noopener"); return; }
  const img = $("#visor-img");
  img.src = src;
  img.alt = alt || "";
  $("#visor-texto").textContent = alt || "";
  visor.showModal();
}
if (visor) {
  $("[data-cerrar]", visor).addEventListener("click", () => visor.close());
  visor.addEventListener("click", (e) => { if (e.target === visor) visor.close(); });
}
$$("[data-ampliar]").forEach((b) => b.addEventListener("click", () => {
  const img = $(`${b.dataset.ampliar} img.ver`);
  if (img) ampliar(img.currentSrc || img.src, img.alt);
}));

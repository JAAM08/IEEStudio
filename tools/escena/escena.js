// Escena 3D de la portada de ieestudio.com: un tablero de distribución abierto
// con sus interruptores, barras, conductores, canalización, puesta a tierra y
// medición. Las piezas corresponden a lo que calcula IEEStudio y los circuitos
// son los del proyecto de ejemplo que trae la aplicación (TG-1, casa
// habitación).
//
// Se empaqueta con `npm run build` en website/escena.js. No se carga si el
// navegador no tiene WebGL: la portada muestra entonces el dibujo estático.
import {
  ACESFilmicToneMapping,
  AmbientLight,
  BoxGeometry,
  BufferAttribute,
  CanvasTexture,
  CatmullRomCurve3,
  Color,
  CylinderGeometry,
  DirectionalLight,
  Group,
  Mesh,
  MeshPhysicalMaterial,
  MeshStandardMaterial,
  PCFShadowMap,
  PerspectiveCamera,
  PlaneGeometry,
  PMREMGenerator,
  Raycaster,
  RepeatWrapping,
  Scene,
  SRGBColorSpace,
  TubeGeometry,
  Vector2,
  Vector3,
  WebGLRenderer,
} from "three";
import { RoundedBoxGeometry } from "three/addons/geometries/RoundedBoxGeometry.js";
import { RoomEnvironment } from "three/addons/environments/RoomEnvironment.js";

const AMBAR = new Color("#E5B84B");
const AZUL = new Color("#5489C7");

// Proyecto de ejemplo de IEEStudio: tablero TG-1, 3F-4H 220Y/127 V.
// [id, descripción, polos, interruptor (A), fases, posiciones en el tablero]
// Las posiciones son las que propone el programa y escribe en el unifilar
// (crates/ieestudio-project/src/positions.rs): nones a la izquierda y pares a
// la derecha, y cada renglón es de una fase (A, B, C, A…). Por eso C-03 (A y C)
// ocupa los espacios 5 y 7, y la columna derecha queda C-06, C-05, C-04.
// Comprobado con `ieestudio dxf` del ejemplo: tools/verificar-sitio.py.
const CIRCUITOS = [
  ["C-01", "Alumbrado planta baja", 1, 15, "A", [1]],
  ["C-02", "Contactos cocina", 1, 20, "B", [3]],
  ["C-03", "Aire acondicionado", 2, 25, "AC", [5, 7]],
  ["C-04", "Contactos sala", 1, 15, "C", [6]],
  ["C-05", "Alumbrado exterior", 1, 15, "B", [4]],
  ["C-06", "Bomba de agua", 1, 25, "A", [2]],
];
// Lo que muestra el medidor: corriente
// nominal y caída del alimentador de TG-1 tal como las escribe el unifilar
// («In: 20.22 A», «%e: 0.68»). No es una medición: es el resultado del cálculo.
const ALIMENTADOR = { corriente: "20.22", caida: "0.68" };
const ESPACIOS_POR_LADO = 7;
// Lado (-1 izquierda, 1 derecha) y renglón de un espacio numerado.
const ladoDe = (n) => (n % 2 === 1 ? -1 : 1);
const renglonDe = (n) => Math.floor((n - 1) / 2);

const COLOR_FASE = { A: "#1d1f23", B: "#a8443a", C: "#3f6ea8" };
const NEUTRO = "#d6d3cb";
const TIERRA = "#3d8a5a";

// Medidas del tablero, en metros.
const W = 0.56, H = 0.82, D = 0.12, T = 0.012;
const PASO = 0.034; // altura de un polo
const Y0 = 0.15; // primer espacio

export function iniciar({ contenedor, reducido = false, compacto = false, alSeleccionar, alPasar }) {
  const renderer = new WebGLRenderer({ antialias: !compacto, alpha: true, powerPreference: "low-power" });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, compacto ? 1 : 1.5));
  renderer.outputColorSpace = SRGBColorSpace;
  renderer.toneMapping = ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.05;
  renderer.shadowMap.enabled = !compacto;
  renderer.shadowMap.type = PCFShadowMap;
  contenedor.appendChild(renderer.domElement);
  renderer.domElement.setAttribute("aria-hidden", "true");

  const scene = new Scene();
  const pmrem = new PMREMGenerator(renderer);
  scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
  scene.environmentIntensity = 0.32;

  const camera = new PerspectiveCamera(26, 1, 0.05, 20);
  const objetivo = new Vector3(0.0, -0.02, 0.04);

  // ---------------------------------------------------------------- materiales
  const gabinete = new MeshPhysicalMaterial({ color: "#2a2e35", metalness: 0.55, roughness: 0.42, clearcoat: 0.35, clearcoatRoughness: 0.5 });
  const placa = new MeshStandardMaterial({ color: "#1b1e24", metalness: 0.2, roughness: 0.85 });
  const cobre = new MeshStandardMaterial({ color: "#c9874f", metalness: 1, roughness: 0.32 });
  const estanado = new MeshStandardMaterial({ color: "#b9bcc1", metalness: 1, roughness: 0.3 });
  const acero = new MeshStandardMaterial({ color: "#8f96a0", metalness: 0.95, roughness: 0.34 });
  const cuerpo = new MeshPhysicalMaterial({ color: "#2b2e34", metalness: 0.05, roughness: 0.55, clearcoat: 0.25 });
  const frente = new MeshStandardMaterial({ color: "#3a3e46", metalness: 0.05, roughness: 0.5 });
  const tapa = new MeshStandardMaterial({ color: "#1f2227", metalness: 0.1, roughness: 0.75 });
  const palanca = () => new MeshStandardMaterial({ color: "#8d939c", roughness: 0.4, metalness: 0.1, emissive: AMBAR.clone(), emissiveIntensity: 0 });

  // ---------------------------------------------------------------- muro
  const muro = new Mesh(new PlaneGeometry(9, 6), new MeshStandardMaterial({ color: "#16181d", roughness: 0.96, map: reticula() }));
  muro.position.set(0, 0, -0.002);
  muro.receiveShadow = true;
  scene.add(muro);

  // ---------------------------------------------------------------- gabinete
  const tablero = new Group();
  scene.add(tablero);
  const caja = (w, h, d, mat, x, y, z, r = 0) => {
    const g = r ? new RoundedBoxGeometry(w, h, d, 2, r) : new BoxGeometry(w, h, d);
    const m = new Mesh(g, mat);
    m.position.set(x, y, z);
    m.castShadow = m.receiveShadow = !compacto;
    tablero.add(m);
    return m;
  };
  caja(W, H, T, gabinete, 0, 0, T / 2); // fondo
  caja(T, H, D, gabinete, -W / 2 + T / 2, 0, D / 2, 0.004);
  caja(T, H, D, gabinete, W / 2 - T / 2, 0, D / 2, 0.004);
  caja(W, T, D, gabinete, 0, H / 2 - T / 2, D / 2, 0.004);
  caja(W, T, D, gabinete, 0, -H / 2 + T / 2, D / 2, 0.004);
  caja(W - 2 * T - 0.04, H - 2 * T - 0.06, 0.004, placa, 0, 0, T + 0.002);

  // Puerta abierta con el directorio de circuitos por dentro.
  const bisagra = new Group();
  bisagra.position.set(W / 2, 0, D);
  bisagra.rotation.y = 1.9;
  tablero.add(bisagra);
  const puerta = new Mesh(new RoundedBoxGeometry(W, H, 0.014, 2, 0.005), gabinete);
  puerta.position.set(-W / 2, 0, 0.007);
  puerta.castShadow = !compacto;
  bisagra.add(puerta);
  const directorio = new Mesh(new PlaneGeometry(0.3, 0.4), new MeshStandardMaterial({ map: textoDirectorio(), roughness: 0.9 }));
  directorio.position.set(-W / 2, 0.04, -0.0005);
  directorio.rotation.y = Math.PI;
  bisagra.add(directorio);

  // ---------------------------------------------------------------- barras
  const zBarra = 0.032;
  const barras = new Group();
  tablero.add(barras);
  ["A", "B", "C"].forEach((f, i) => {
    const b = new Mesh(new BoxGeometry(0.011, 0.36, 0.006), cobre);
    b.position.set((i - 1) * 0.019, Y0 - 0.13, zBarra);
    b.castShadow = !compacto;
    barras.add(b);
  });

  // Interruptor principal arriba, de tres polos, y el medidor a su lado.
  const principal = new Group();
  tablero.add(principal);
  const pCuerpo = new Mesh(new RoundedBoxGeometry(0.15, 0.1, 0.07, 3, 0.008), cuerpo);
  pCuerpo.castShadow = !compacto;
  principal.add(pCuerpo);
  const pPalanca = new Mesh(new RoundedBoxGeometry(0.05, 0.022, 0.03, 2, 0.004), palanca());
  pPalanca.position.set(0, 0, 0.044);
  principal.add(pPalanca);
  principal.position.set(0, 0.285, 0.05);

  const medidor = new Group();
  tablero.add(medidor);
  const mCaja = new Mesh(new RoundedBoxGeometry(0.096, 0.1, 0.05, 2, 0.006), cuerpo);
  medidor.add(mCaja);
  const pantalla = new Mesh(new PlaneGeometry(0.08, 0.06), new MeshStandardMaterial({
    color: "#000", emissive: "#ffffff", emissiveMap: textoMedidor(), emissiveIntensity: 1.1, roughness: 0.25,
  }));
  pantalla.position.set(0, 0.008, 0.0252);
  medidor.add(pantalla);
  medidor.position.set(0.172, 0.285, 0.04);

  // ---------------------------------------------------------------- interruptores
  const interruptores = [];
  const yEspacio = (k) => Y0 - k * PASO;
  CIRCUITOS.forEach(([id, desc, polos, amp, fase, posiciones]) => {
    const lado = ladoDe(posiciones[0]);
    const k = renglonDe(posiciones[0]); // primer renglón que ocupa
    const y = yEspacio(k) - ((polos - 1) * PASO) / 2;
    const g = new Group();
    const c = new Mesh(new RoundedBoxGeometry(0.095, polos * PASO - 0.003, 0.064, 2, 0.005), cuerpo);
    c.castShadow = !compacto;
    g.add(c);
    const cara = new Mesh(new BoxGeometry(0.05, polos * PASO - 0.008, 0.004), frente);
    cara.position.set(lado * 0.012, 0, 0.032);
    g.add(cara);
    const pm = palanca();
    const p = new Mesh(new RoundedBoxGeometry(0.014, polos * PASO - 0.012, 0.018, 2, 0.003), pm);
    p.position.set(lado * 0.012, 0, 0.04);
    g.add(p);
    g.position.set(lado * 0.0745, y, 0.07);
    g.userData = { id, desc, polos, amp, fase, lado, posiciones, base: g.position.z, palanca: pm, alto: 0, tipo: "interruptor" };
    c.userData.interruptor = g;
    cara.userData.interruptor = g;
    p.userData.interruptor = g;
    tablero.add(g);
    interruptores.push(g);
  });
  // Espacios libres con su tapa.
  const ocupado = new Set(CIRCUITOS.flatMap((c) => c[5]));
  for (let n = 1; n <= ESPACIOS_POR_LADO * 2; n++) {
    if (ocupado.has(n)) continue;
    const t = new Mesh(new BoxGeometry(0.095, PASO - 0.004, 0.006), tapa);
    t.position.set(ladoDe(n) * 0.0745, yEspacio(renglonDe(n)), 0.1);
    tablero.add(t);
  }

  // ---------------------------------------------------------------- neutro y tierra
  const yBarrasBajas = -0.255;
  const neutro = new Mesh(new BoxGeometry(0.17, 0.014, 0.016), estanado);
  neutro.position.set(-0.1, yBarrasBajas, 0.04);
  tablero.add(neutro);
  const tierra = new Mesh(new BoxGeometry(0.13, 0.014, 0.016), cobre);
  tierra.position.set(0.12, yBarrasBajas, 0.04);
  tablero.add(tierra);
  for (let i = 0; i < 8; i++) {
    const t = new Mesh(new CylinderGeometry(0.004, 0.004, 0.006, 10), acero);
    t.rotation.x = Math.PI / 2;
    t.position.set(i < 5 ? -0.17 + i * 0.035 : 0.075 + (i - 5) * 0.045, yBarrasBajas, 0.05);
    tablero.add(t);
  }

  // ---------------------------------------------------------------- canalización
  const tubos = new Group();
  tablero.add(tubos);
  const xTubos = [-0.13, 0.0, 0.13];
  const tubo = (x, y0, y1, r) => {
    const t = new Mesh(new CylinderGeometry(r, r, Math.abs(y1 - y0), 28, 1, true), acero);
    t.position.set(x, (y0 + y1) / 2, D / 2);
    t.castShadow = !compacto;
    tubos.add(t);
    const anillo = new Mesh(new CylinderGeometry(r + 0.007, r + 0.007, 0.022, 28), acero);
    anillo.position.set(x, y0 + Math.sign(y1 - y0) * 0.011, D / 2);
    tubos.add(anillo);
  };
  xTubos.forEach((x) => tubo(x, -H / 2, -2.6, 0.026));
  tubo(0, H / 2, 2.6, 0.032);

  // ---------------------------------------------------------------- conductores
  const conductores = [];
  const tiempo = { value: 0 };
  const cable = (puntos, color, r, grupo, circuito) => {
    const curva = new CatmullRomCurve3(puntos.map((p) => new Vector3(...p)), false, "centripetal", 0.5);
    const largo = curva.getLength();
    const geo = new TubeGeometry(curva, Math.max(32, Math.round(largo * 90)), r, 7, false);
    const uv = geo.attributes.uv;
    const along = new Float32Array(uv.count);
    for (let i = 0; i < uv.count; i++) along[i] = uv.getX(i) * largo;
    geo.setAttribute("along", new BufferAttribute(along, 1));
    const mat = new MeshPhysicalMaterial({ color, roughness: 0.48, metalness: 0, clearcoat: 0.25, clearcoatRoughness: 0.6 });
    const u = { intensidad: { value: grupo === "alimentador" ? 0.55 : 0.32 }, color: { value: grupo === "tierra" ? AZUL.clone() : AMBAR.clone() } };
    mat.onBeforeCompile = (sh) => {
      sh.uniforms.uTime = tiempo;
      sh.uniforms.uInt = u.intensidad;
      sh.uniforms.uCol = u.color;
      sh.vertexShader = sh.vertexShader
        .replace("#include <common>", "#include <common>\nattribute float along;\nvarying float vAlong;")
        .replace("#include <begin_vertex>", "#include <begin_vertex>\nvAlong = along;");
      sh.fragmentShader = sh.fragmentShader
        .replace("#include <common>", "#include <common>\nuniform float uTime;\nuniform float uInt;\nuniform vec3 uCol;\nvarying float vAlong;")
        .replace(
          "#include <emissivemap_fragment>",
          "#include <emissivemap_fragment>\nfloat ph = fract(vAlong * 3.2 - uTime * 0.55);\nfloat band = smoothstep(0.0, 0.05, ph) * (1.0 - smoothstep(0.05, 0.22, ph));\ntotalEmissiveRadiance += uCol * band * uInt * 2.4;"
        );
    };
    const m = new Mesh(geo, mat);
    m.castShadow = !compacto;
    m.userData = { circuito, grupo, u };
    tablero.add(m);
    conductores.push(m);
    return m;
  };

  // Alimentador: entra por el tubo superior al interruptor principal y de ahí a las barras.
  const zc = 0.065;
  [["A", -0.018], ["B", 0], ["C", 0.018]].forEach(([f, dx]) => {
    cable([[dx * 0.6, 1.6, D / 2], [dx * 0.6, H / 2 - 0.02, D / 2], [dx, 0.36, zc], [dx * 2.4, 0.335, 0.05]], COLOR_FASE[f], 0.0065, "alimentador", "ALIM");
    cable([[dx * 2.4, 0.235, 0.05], [dx, 0.2, 0.04], [(f === "A" ? -1 : f === "B" ? 0 : 1) * 0.019, Y0 + 0.02, zBarra]], COLOR_FASE[f], 0.0055, "alimentador", "ALIM");
  });
  // El neutro baja por el canal izquierdo para no tapar el medidor.
  cable([[-0.03, 1.6, D / 2], [-0.03, H / 2 - 0.02, D / 2], [-0.12, 0.33, 0.06], [-0.212, 0.1, 0.06], [-0.212, -0.2, 0.06], [-0.12, yBarrasBajas + 0.014, 0.05], [-0.09, yBarrasBajas, 0.05]], NEUTRO, 0.0055, "neutro", "ALIM");

  // Derivados: salen del extremo exterior de cada interruptor, bajan por el canal lateral y entran al tubo.
  interruptores.forEach((g, i) => {
    const { id, fase, lado } = g.userData;
    const x0 = g.position.x + lado * 0.05;
    const y = g.position.y;
    const xc = lado * (0.2 - (i % 3) * 0.011);
    const xt = lado < 0 ? xTubos[0] : xTubos[2];
    const fin = (dx) => [
      [x0, y, 0.07], [x0 + lado * 0.03, y - 0.008, 0.075], [xc, y - 0.05, 0.07],
      [xc, -0.3, 0.065], [xt + dx + lado * 0.02, -0.36, D / 2], [xt + dx, -H / 2 + 0.01, D / 2], [xt + dx, -1.8, D / 2],
    ];
    const colorFase = COLOR_FASE[fase[0]] || COLOR_FASE.A;
    cable(fin(((i % 3) - 1) * 0.008), colorFase, 0.0042, "derivado", id);
    if (fase.length > 1) cable(fin(((i % 3) - 1) * 0.008 + 0.006), COLOR_FASE[fase[1]], 0.0042, "derivado", id);
  });
  // Neutros y tierras de los derivados hacia el tubo central.
  for (let i = 0; i < 3; i++) {
    cable([[-0.16 + i * 0.03, yBarrasBajas, 0.05], [-0.15 + i * 0.03, -0.3, 0.06], [-0.012 + i * 0.008, -0.37, D / 2], [-0.012 + i * 0.008, -H / 2, D / 2], [-0.012 + i * 0.008, -1.8, D / 2]], NEUTRO, 0.004, "neutro", "N");
    cable([[0.09 + i * 0.04, yBarrasBajas, 0.05], [0.08 + i * 0.03, -0.3, 0.06], [0.004 + i * 0.008, -0.37, D / 2], [0.004 + i * 0.008, -H / 2, D / 2], [0.004 + i * 0.008, -1.8, D / 2]], TIERRA, 0.004, "tierra", "T");
  }

  // ---------------------------------------------------------------- luces
  scene.add(new AmbientLight("#9fb3cf", 0.18));
  const clave = new DirectionalLight("#fff0d8", 2.6);
  clave.position.set(-1.1, 1.5, 2.1);
  clave.target.position.set(0, -0.05, 0);
  clave.castShadow = !compacto;
  clave.shadow.mapSize.set(1024, 1024);
  clave.shadow.camera.left = -0.9; clave.shadow.camera.right = 0.9;
  clave.shadow.camera.top = 0.9; clave.shadow.camera.bottom = -0.9;
  clave.shadow.camera.near = 0.5; clave.shadow.camera.far = 5;
  clave.shadow.bias = -0.0004;
  clave.shadow.normalBias = 0.01;
  clave.shadow.radius = 5;
  scene.add(clave, clave.target);
  const contra = new DirectionalLight(AZUL, 1.4);
  contra.position.set(1.8, 0.5, 0.9);
  scene.add(contra);
  const relleno = new DirectionalLight("#ffffff", 0.35);
  relleno.position.set(0.3, -1, 1.5);
  scene.add(relleno);

  // ---------------------------------------------------------------- piezas señalables
  // Punto de cada globo, en coordenadas del tablero.
  const piezas = {
    principal: { punto: new Vector3(0, 0.3, 0.09), objetos: [principal], foco: new Vector3(0.02, 0.24, 0.05) },
    derivados: { punto: new Vector3(-0.11, 0.08, 0.105), objetos: interruptores, foco: new Vector3(-0.02, 0.08, 0.05) },
    barras: { punto: new Vector3(0.0, 0.0, 0.04), objetos: [barras], foco: new Vector3(0, 0.05, 0.04) },
    conductores: { punto: new Vector3(-0.2, -0.18, 0.07), objetos: conductores, foco: new Vector3(-0.08, -0.12, 0.05) },
    canalizacion: { punto: new Vector3(0.13, -0.52, D / 2 + 0.026), objetos: [tubos], foco: new Vector3(0.02, -0.35, 0.05) },
    tierra: { punto: new Vector3(0.12, yBarrasBajas + 0.008, 0.048), objetos: [tierra], foco: new Vector3(0.06, -0.2, 0.05) },
    medicion: { punto: new Vector3(0.2, 0.335, 0.05), objetos: [medidor], foco: new Vector3(0.15, 0.27, 0.05), dist: 1.25 },
  };

  // ---------------------------------------------------------------- cámara e interacción
  const puntero = new Vector2(0, 0);
  const suave = new Vector2(0, 0);
  const vista = { az: -0.42, el: 0.08, dist: 3.0, foco: objetivo.clone(), destino: objetivo.clone(), distDestino: 2.25 };
  let seleccion = null;
  let sobre = null;
  let ancho = 1, alto = 1, desplazamiento = 0;

  const ray = new Raycaster();
  const ndc = new Vector2();
  const seleccionables = [];
  interruptores.forEach((g) => g.children.forEach((m) => seleccionables.push(m)));
  conductores.forEach((m) => seleccionables.push(m));
  [principal, medidor, tubos, barras].forEach((g) => g.traverse((m) => m.isMesh && seleccionables.push(m)));
  seleccionables.push(tierra);

  const piezaDe = (obj) => {
    if (obj.userData.interruptor) return { pieza: "derivados", interruptor: obj.userData.interruptor };
    if (obj.userData.circuito) {
      const c = obj.userData.circuito;
      if (obj.userData.grupo === "tierra" || c === "T") return { pieza: "tierra" };
      const g = interruptores.find((x) => x.userData.id === c);
      return g ? { pieza: "conductores", interruptor: g } : { pieza: "conductores" };
    }
    if (obj === tierra) return { pieza: "tierra" };
    for (const [k, v] of Object.entries(piezas)) {
      for (const o of v.objetos) {
        let hit = false;
        o.traverse && o.traverse((m) => { if (m === obj) hit = true; });
        if (hit) return { pieza: k };
      }
    }
    return null;
  };

  const elemento = renderer.domElement;
  const onMove = (e) => {
    if (e.pointerType === "touch") return;
    const r = elemento.getBoundingClientRect();
    puntero.set(((e.clientX - r.left) / r.width) * 2 - 1, -(((e.clientY - r.top) / r.height) * 2 - 1));
    ndc.copy(puntero);
    ray.setFromCamera(ndc, camera);
    const hit = ray.intersectObjects(seleccionables, false)[0];
    const info = hit ? piezaDe(hit.object) : null;
    const g = info && info.interruptor ? info.interruptor : null;
    if (g !== sobre) {
      sobre = g;
      elemento.style.cursor = info ? "pointer" : "";
      alPasar && alPasar(g ? { id: g.userData.id, desc: g.userData.desc, polos: g.userData.polos, amp: g.userData.amp, x: e.clientX - r.left, y: e.clientY - r.top } : null);
    } else if (g && alPasar) {
      alPasar({ id: g.userData.id, desc: g.userData.desc, polos: g.userData.polos, amp: g.userData.amp, x: e.clientX - r.left, y: e.clientY - r.top });
    }
    if (!g) elemento.style.cursor = info ? "pointer" : "";
    pedir();
  };
  const onLeave = (e) => {
    if (e && e.pointerType === "touch") return;
    puntero.set(0, 0);
    sobre = null;
    alPasar && alPasar(null);
    pedir();
  };
  // Un toque o clic elige la pieza. En pantallas táctiles no hay «pasar el
  // puntero»: el toque sobre un interruptor también muestra su circuito unos segundos.
  let ocultar = 0;
  const onClick = (e) => {
    const r = elemento.getBoundingClientRect();
    ndc.set(((e.clientX - r.left) / r.width) * 2 - 1, -(((e.clientY - r.top) / r.height) * 2 - 1));
    ray.setFromCamera(ndc, camera);
    const hit = ray.intersectObjects(seleccionables, false)[0];
    const info = hit ? piezaDe(hit.object) : null;
    if (info) alSeleccionar && alSeleccionar(info.pieza);
    if (e.pointerType === "touch" || tactil) {
      const g = info && info.interruptor ? info.interruptor : null;
      sobre = g;
      alPasar && alPasar(g ? { id: g.userData.id, desc: g.userData.desc, polos: g.userData.polos, amp: g.userData.amp, x: e.clientX - r.left, y: e.clientY - r.top } : null);
      clearTimeout(ocultar);
      ocultar = setTimeout(() => { sobre = null; alPasar && alPasar(null); pedir(); }, 2500);
      pedir();
    }
  };
  let tactil = false;
  elemento.addEventListener("pointerdown", (e) => { tactil = e.pointerType === "touch"; });
  elemento.addEventListener("pointermove", onMove);
  elemento.addEventListener("pointerleave", onLeave);
  elemento.addEventListener("click", onClick);

  // El tablero se recorre a la derecha en pantallas anchas para dejar el texto a la izquierda.
  const medir = () => {
    ancho = contenedor.clientWidth || 1;
    alto = contenedor.clientHeight || 1;
    renderer.setSize(ancho, alto, false);
    camera.aspect = ancho / alto;
    desplazamiento = ancho > 900 ? -ancho * 0.07 : 0;
    if (desplazamiento) camera.setViewOffset(ancho, alto, desplazamiento, -alto * 0.02, ancho, alto);
    else camera.clearViewOffset();
    vista.distDestino = ancho > 900 ? 3.0 : ancho > 600 ? 2.9 : 3.3;
    if (!seleccion) vista.dist = vista.distDestino;
    camera.updateProjectionMatrix();
    pedir();
  };
  const ro = new ResizeObserver(medir);
  ro.observe(contenedor);

  // ---------------------------------------------------------------- bucle
  let activo = true, visible = true, pendiente = false, ultimo = 0, t = 0;
  const v = new Vector3();
  const animar = (ahora) => {
    pendiente = false;
    const dt = Math.min(0.05, ultimo ? (ahora - ultimo) / 1000 : 0.016);
    ultimo = ahora;
    const k = reducido ? 1 : 1 - Math.pow(0.0016, dt);
    suave.lerp(puntero, reducido ? 0 : k * 0.6);
    if (!reducido) t += dt;
    tiempo.value = t;

    vista.foco.lerp(vista.destino, k);
    vista.dist += (vista.distDestino - vista.dist) * k;
    const az = vista.az + suave.x * 0.16;
    const el = vista.el + suave.y * 0.07;
    camera.position.set(
      vista.foco.x + Math.sin(az) * Math.cos(el) * vista.dist,
      vista.foco.y + Math.sin(el) * vista.dist,
      vista.foco.z + Math.cos(az) * Math.cos(el) * vista.dist
    );
    camera.lookAt(vista.foco);
    clave.position.set(-1.1 + suave.x * 0.9, 1.5 + suave.y * 0.5, 2.1);

    // Interruptor bajo el puntero: se adelanta y su circuito se enciende.
    let moviendo = false;
    for (const g of interruptores) {
      const meta = g === sobre || (seleccion === "derivados" && !sobre) ? 1 : 0;
      const d = meta - g.userData.alto;
      if (Math.abs(d) > 0.002) moviendo = true;
      g.userData.alto += d * (reducido ? 1 : k * 1.4);
      g.position.z = g.userData.base + g.userData.alto * 0.012;
      g.userData.palanca.emissiveIntensity = g.userData.alto * 0.9;
    }
    pPalanca.material.emissiveIntensity = seleccion === "principal" ? 0.9 : 0.25;
    for (const m of conductores) {
      const { circuito, grupo, u } = m.userData;
      let meta = grupo === "alimentador" ? 0.5 : 0.22;
      if (sobre && circuito === sobre.userData.id) meta = 1.6;
      else if (sobre) meta *= 0.35;
      if (seleccion === "conductores" && !sobre) meta = grupo === "derivado" || grupo === "alimentador" ? 1 : meta;
      if (seleccion === "tierra" && grupo === "tierra") meta = 1.4;
      if (reducido) meta *= 0.55;
      u.intensidad.value += (meta - u.intensidad.value) * (reducido ? 1 : k);
    }

    renderer.render(scene, camera);
    colocarGlobos();

    const quieto = Math.abs(vista.dist - vista.distDestino) < 0.001 && vista.foco.distanceTo(vista.destino) < 0.0005 && suave.distanceTo(puntero) < 0.001 && !moviendo;
    if (activo && visible && (!reducido || !quieto)) {
      if (!reducido) pedir();
    }
  };
  const pedir = () => {
    if (pendiente || !activo || !visible) return;
    pendiente = true;
    requestAnimationFrame(animar);
  };

  // Posición en pantalla de cada globo (la capa HTML dibuja los globos y sus líneas).
  const pantallaDe = {};
  let escuchaGlobos = null;
  const colocarGlobos = () => {
    for (const [k, p] of Object.entries(piezas)) {
      v.copy(p.punto).project(camera);
      pantallaDe[k] = { x: (v.x * 0.5 + 0.5) * ancho, y: (-v.y * 0.5 + 0.5) * alto };
    }
    escuchaGlobos && escuchaGlobos(pantallaDe);
  };

  const io = new IntersectionObserver(([e]) => {
    visible = e.isIntersecting;
    if (visible) { ultimo = 0; pedir(); }
  });
  io.observe(contenedor);
  const onVis = () => { activo = !document.hidden; if (activo) { ultimo = 0; pedir(); } };
  document.addEventListener("visibilitychange", onVis);

  medir();
  return {
    seleccionar(id) {
      seleccion = id && piezas[id] ? id : null;
      vista.destino.copy(seleccion ? piezas[seleccion].foco : objetivo);
      const cerca = seleccion && piezas[seleccion].dist;
      vista.distDestino = seleccion ? (cerca || 2.1) * (ancho > 900 ? 1 : 1.2) : ancho > 900 ? 3.0 : ancho > 600 ? 2.9 : 3.3;
      pedir();
    },
    alColocar(fn) { escuchaGlobos = fn; colocarGlobos(); },
    piezas: Object.keys(piezas),
    destruir() {
      activo = false;
      ro.disconnect(); io.disconnect();
      clearTimeout(ocultar);
      document.removeEventListener("visibilitychange", onVis);
      renderer.dispose();
      elemento.remove();
    },
    primerCuadro: new Promise((res) => requestAnimationFrame(() => requestAnimationFrame(res))),
  };
}

// Retícula de plano en el muro: líneas finas cada 5 cm y gruesas cada 25 cm.
function reticula() {
  const c = document.createElement("canvas");
  c.width = c.height = 512;
  const g = c.getContext("2d");
  g.fillStyle = "#17191e";
  g.fillRect(0, 0, 512, 512);
  for (let i = 0; i <= 10; i++) {
    const p = i * 51.2;
    g.strokeStyle = i % 5 === 0 ? "rgba(84,137,199,0.22)" : "rgba(84,137,199,0.09)";
    g.lineWidth = i % 5 === 0 ? 2 : 1;
    g.beginPath(); g.moveTo(p, 0); g.lineTo(p, 512); g.moveTo(0, p); g.lineTo(512, p); g.stroke();
  }
  const t = new CanvasTexture(c);
  t.wrapS = t.wrapT = RepeatWrapping;
  t.repeat.set(9 / 0.5, 6 / 0.5);
  t.colorSpace = SRGBColorSpace;
  t.anisotropy = 4;
  return t;
}

// Directorio de circuitos pegado por dentro de la puerta, como en obra.
function textoDirectorio() {
  const c = document.createElement("canvas");
  c.width = 600; c.height = 800;
  const g = c.getContext("2d");
  g.fillStyle = "#e9e6df";
  g.fillRect(0, 0, 600, 800);
  g.strokeStyle = "#2b2f36";
  g.lineWidth = 3;
  g.strokeRect(24, 24, 552, 752);
  g.fillStyle = "#17191e";
  g.font = "600 40px Manrope, sans-serif";
  g.fillText("Tablero TG-1", 52, 98);
  g.font = "26px 'Plex Mono', monospace";
  g.fillStyle = "#4a4f57";
  g.fillText("3F-4H  220Y/127 V", 52, 142);
  g.lineWidth = 1.5;
  [...CIRCUITOS].sort((a, b) => a[0].localeCompare(b[0])).forEach(([id, desc, polos, amp], i) => {
    const y = 214 + i * 82;
    g.strokeStyle = "#b9b5ab";
    g.beginPath(); g.moveTo(52, y - 44); g.lineTo(548, y - 44); g.stroke();
    g.fillStyle = "#17191e";
    g.font = "26px 'Plex Mono', monospace";
    g.fillText(id, 52, y);
    g.font = "500 26px Manrope, sans-serif";
    g.fillText(desc, 152, y);
    g.fillStyle = "#4a4f57";
    g.font = "22px 'Plex Mono', monospace";
    g.fillText(`${polos}P-${amp} A`, 152, y + 28);
  });
  const t = new CanvasTexture(c);
  t.colorSpace = SRGBColorSpace;
  t.anisotropy = 4;
  return t;
}

// Pantalla del medidor: el resultado del alimentador de TG-1 con la misma
// notación del unifilar (In corriente nominal, %e caída del tramo).
function textoMedidor() {
  const c = document.createElement("canvas");
  c.width = 256; c.height = 192;
  const g = c.getContext("2d");
  g.fillStyle = "#07090b";
  g.fillRect(0, 0, 256, 192);
  g.fillStyle = "#858b94";
  g.font = "22px 'Plex Mono', monospace";
  g.fillText("ALIM. TG-1", 20, 38);
  g.fillStyle = "#E5B84B";
  g.font = "26px 'Plex Mono', monospace";
  g.fillText("In", 20, 96);
  g.font = "48px 'Plex Mono', monospace";
  g.fillText(ALIMENTADOR.corriente, 64, 98);
  g.font = "24px 'Plex Mono', monospace";
  g.fillText("A", 214, 98);
  g.fillStyle = "#5489C7";
  g.font = "26px 'Plex Mono', monospace";
  g.fillText(`%e ${ALIMENTADOR.caida}`, 20, 158);
  const t = new CanvasTexture(c);
  t.colorSpace = SRGBColorSpace;
  return t;
}


# Genera las páginas de website/ con la misma navegación y el mismo pie.
#
#   python3 tools/paginas.py website
#
# Todo el texto describe lo que el programa hace hoy. Antes de agregar una
# función o una cifra, compruébala en el código o en docs/roadmap.md. Las
# imágenes son capturas y hojas reales del programa (ver website/README.md).
import os, sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else "website"
GH = "https://github.com/JAAM08/IEEStudio"
SITIO = "https://ieestudio.com/"
DESC_INICIO = ("Programa gratuito y de código abierto para calcular, diseñar y documentar instalaciones "
               "eléctricas de baja tensión conforme a la NOM-001-SEDE-2012: conductores, protecciones, "
               "caída de tensión, tubería, cuadro de cargas, unifilar y memoria de cálculo.")

# (destino, nombre, sección de la portada que lo marca como activo)
NAV = [
    ("#producto", "Producto", "producto"),
    ("funciones/", "Herramientas", "herramientas"),
    ("guia/", "Documentación", "documentacion"),
    ("codigo-abierto/", "Comunidad", "comunidad"),
]


def icon(p, name, cls=""):
    c = f' class="{cls}"' if cls else ""
    return f'<svg{c} aria-hidden="true"><use href="{p}iconos.svg#{name}"/></svg>'


def head(p, title, desc, current, extra_head=""):
    full = "IEEStudio — Ingeniería eléctrica con precisión digital" if not title else f"{title} · IEEStudio"
    links = []
    for href, name, sec in NAV:
        if href.startswith("#"):
            url = f"{p}{href}" if p else href
            if p:
                url = f"{p}{href}"
        else:
            url = f"{p}{href}"
        cur = ' aria-current="page"' if href == current else ""
        links.append(f'      <a href="{url}" data-seccion="{sec}"{cur}>{name}</a>')
    links = "\n".join(links)
    return f"""<!doctype html>
<html lang="es-MX">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>{full}</title>
<meta name="description" content="{desc}">
<link rel="canonical" href="{SITIO}{current if not current.startswith('#') else ''}">
<meta name="theme-color" content="#101216">
<meta name="color-scheme" content="dark">
<link rel="icon" href="{p}app/icono.svg" type="image/svg+xml">
<link rel="icon" href="{p}favicon.ico" sizes="any">
<link rel="apple-touch-icon" href="{p}app/icono-180.png">
<meta property="og:type" content="website">
<meta property="og:locale" content="es_MX">
<meta property="og:site_name" content="IEEStudio">
<meta property="og:title" content="{full}">
<meta property="og:description" content="{desc}">
<meta property="og:image" content="{SITIO}imagenes/compartir.jpg">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:url" content="{SITIO}{current if not current.startswith('#') else ''}">
<meta name="twitter:card" content="summary_large_image">
<link rel="preload" href="{p}fuentes/Manrope-Variable.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="{p}estilos.css">{extra_head}
<script>document.documentElement.classList.add("js")</script>
</head>
<body>
<a class="sr" href="#contenido">Ir al contenido</a>

<header class="nav" id="nav">
  <div class="wrap">
    <a class="marca" href="{p or './'}"><img src="{p}app/icono.svg" alt="" width="28" height="28">IEEStudio</a>
    <button class="menu" type="button" aria-label="Menú" aria-expanded="false" aria-controls="enlaces">{icon(p, "i-menu")}</button>
    <nav class="nav-links" id="enlaces" aria-label="Principal">
{links}
      <a class="nav-git" href="{GH}">{icon(p, "i-git")}<span>GitHub</span></a>
      <a class="btn btn-ambar btn-chico" href="{p}descargar/"{' aria-current="page"' if current == "descargar/" else ""}>Descargar</a>
    </nav>
  </div>
</header>

<main id="contenido">
"""


def foot(p, scripts=""):
    return f"""</main>

<footer>
  <div class="wrap">
    <a class="marca" href="{p or './'}"><img src="{p}app/icono.svg" alt="" width="26" height="26">IEEStudio</a>
    <nav aria-label="Pie">
      <a href="{p}app/">Abrir la app</a>
      <a href="{p}funciones/">Herramientas</a>
      <a href="{p}guia/">Documentación</a>
      <a href="{p}descargar/">Descargar</a>
      <a href="{p}codigo-abierto/">Comunidad</a>
      <a href="{GH}/issues">Reportar un problema</a>
    </nav>
    <p>Software libre bajo licencias MIT o Apache-2.0. IEEStudio no está afiliado a la Secretaría de Energía, a la CFE ni a ningún organismo de normalización; la NOM-001-SEDE-2012 se cita por artículo y su texto no se reproduce. Las imágenes del programa son capturas y documentos reales de IEEStudio; el modelo 3D de la portada es un dibujo propio.</p>
  </div>
</footer>

<script src="{p}sitio.js" defer></script>{scripts}
</body>
</html>
"""


def write(rel, html):
    # Las capturas de escritorio llevan su versión al doble de resolución.
    import re
    html = re.sub(r'<img src="([^"]*imagenes/app/[a-z]+)\.webp" width="1440"',
                  r'<img src="\1.webp" srcset="\1.webp 1x, \1@2x.webp 2x" width="1440"', html)
    path = os.path.join(ROOT, rel, "index.html") if rel else os.path.join(ROOT, "index.html")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(html)


# ---------------------------------------------------------------- dibujo del tablero
# Posiciones de los interruptores del proyecto de ejemplo, como las propone el
# programa en el unifilar (nones a la izquierda, pares a la derecha). Son las
# mismas de tools/escena/escena.js; tools/verificar-sitio.py las compara con
# `ieestudio dxf`.
POSICIONES = [("C-01", 1, [1]), ("C-02", 1, [3]), ("C-03", 2, [5, 7]), ("C-04", 1, [6]), ("C-05", 1, [4]), ("C-06", 1, [2])]

# Alzado frontal del mismo tablero de la escena 3D (escala 1:2, cotas en mm).
# Sirve de respaldo mientras carga la escena, cuando no hay WebGL y en el cierre.
def plano_tablero(cls="", titulo="Alzado frontal del tablero TG-1 del proyecto de ejemplo"):
    rows = []
    for et, polos, pos in POSICIONES:
        n = pos[0]
        lado = -1 if n % 2 else 1
        k = (n - 1) // 2
        x = 149 if lado < 0 else 223.5
        y = 181.5 + k * 17
        h = polos * 17 - 1.5
        rows.append(f'<rect x="{x}" y="{y}" width="47.5" height="{h}" rx="2"/>')
        hx = x + (40 if lado < 0 else 3.5)
        rows.append(f'<rect class="c" x="{hx}" y="{y + 4}" width="4" height="{h - 8}" rx="1"/>')
        tx = x - 6 if lado < 0 else x + 53.5
        anchor = "end" if lado < 0 else "start"
        rows.append(f'<text x="{tx}" y="{y + h / 2 + 3.5}" text-anchor="{anchor}">{et}</text>')
    ocupados = {n for _, _, pos in POSICIONES for n in pos}
    for n in range(1, 15):
        if n in ocupados:
            continue
        x = 149 if n % 2 else 223.5
        rows.append(f'<rect class="f" x="{x}" y="{181.5 + (n - 1) // 2 * 17 + 1}" width="47.5" height="14"/>')
    brk_svg = "\n    ".join(rows)
    return f"""<svg class="plano-tablero {cls}" viewBox="0 0 440 600" role="img" aria-label="{titulo}">
  <title>{titulo}</title>
  <g>
    <rect class="g" x="70" y="60" width="280" height="410" rx="3"/>
    <rect class="f" x="86" y="76" width="248" height="378"/>
    <rect x="194" y="0" width="32" height="60"/>
    <path class="f" d="M190 52h40M190 58h40"/>
    <rect x="132" y="470" width="26" height="110"/><rect x="197" y="470" width="26" height="110"/><rect x="262" y="470" width="26" height="110"/>
    <path class="f" d="M128 472h34M193 472h34M258 472h34"/>
    <rect x="172.5" y="97.5" width="75" height="50" rx="4"/>
    <rect class="rc" x="197.5" y="117" width="25" height="11" rx="2"/>
    <text x="210" y="92" text-anchor="middle">3P-25</text>
    <rect x="276" y="101" width="37.5" height="42.5" rx="3"/>
    <rect class="c" x="281" y="106" width="27.5" height="20" rx="1"/>
    <rect x="198" y="165" width="5.5" height="180"/><rect x="207.25" y="165" width="5.5" height="180"/><rect x="216.5" y="165" width="5.5" height="180"/>
    {brk_svg}
    <rect x="117.5" y="389" width="85" height="7"/><rect x="237.5" y="389" width="65" height="7"/>
    <text x="160" y="410" text-anchor="middle">N</text><text x="270" y="410" text-anchor="middle">T</text>
  </g>
  <g class="cota">
    <path d="M70 34v20M350 34v20M70 40h280M380 60h20M380 470h20M394 60v410"/>
    <path d="M74 37l-4 3 4 3M346 37l4 3-4 3M391 64l3-4 3 4M391 466l3 4 3-4"/>
  </g>
  <text x="210" y="34" text-anchor="middle">560</text>
  <text x="404" y="268" transform="rotate(90 404 268)" text-anchor="middle">820</text>
  <text x="70" y="596">TG-1  3F-4H  220Y/127 V</text>
</svg>"""


# ---------------------------------------------------------------- portada
p = ""

PIEZAS = [
    ("principal", "Interruptor principal",
     "La carga del alimentador sale del artículo 220 con sus factores de demanda; el interruptor es el normalizado inmediato superior, o el que fijes, y se verifica contra el conductor.",
     "220-40, 220-42; 240-6(a), 240-4", "funciones/#calculos"),
    ("derivados", "Interruptores derivados",
     "Cada circuito lleva el interruptor normalizado inmediato superior a su corriente de diseño. Pasa el puntero sobre uno, o tócalo: es un circuito del proyecto de ejemplo, en la posición que le da su unifilar.",
     "210-20(a); 240-6(a); 240-4(d)", "funciones/#calculos"),
    ("barras", "Barras y fases",
     "Reparte los circuitos de un tablero entre las fases A, B y C, muestra el desbalance y propone el reparto más parejo; se puede deshacer.",
     "Cuadro de cargas por tablero", "guia/#paso-4"),
    ("conductores", "Conductores",
     "El menor calibre que cumple por temperatura de terminales, ampacidad corregida por temperatura y agrupamiento, protección y caída de tensión. Paralelos, un juego por tubo.",
     "110-14(c); Tabla 310-15(b)(16); 310-10(h)", "funciones/#calculos"),
    ("canalizacion", "Canalización",
     "Tubo conduit EMT, IMC, RMC y PVC con las dimensiones reales de cada conductor aislado: ocupación de 53, 31 y 40 %, niples al 60 % y atascamiento.",
     "Capítulo 10, Tablas 1, 4 y 5", "funciones/#calculos"),
    ("tierra", "Puesta a tierra",
     "Conductor de puesta a tierra de cada circuito y alimentador, con el incremento proporcional si las fases se sobredimensionan, y conductor al electrodo de la acometida.",
     "Tabla 250-122; 250-122(b); Tabla 250-66", "funciones/#calculos"),
    ("medicion", "Caída de tensión",
     "Por impedancia, con la resistencia y reactancia de la Tabla 9, por tramo y acumulada desde la fuente. La pantalla no es una medición: muestra lo que IEEStudio calcula para el alimentador de TG-1 del ejemplo, In 20.22 A y %e 0.68, como en su unifilar.",
     "Capítulo 10, Tabla 9; 210-19(a)(1) nota 4; 215-2(a)(1) nota 2", "funciones/#calculos"),
]

simbologia = "\n".join(
    f'          <li><button type="button" data-pieza="{k}" aria-pressed="false" aria-controls="ficha-pieza"><span class="n">{i}</span>{n}</button></li>'
    for i, (k, n, *_r) in enumerate(PIEZAS, 1))
fichas = "\n".join(
    f'      <template data-pieza="{k}"><h3>{n}</h3><p>{t}</p><p class="ref">NOM-001-SEDE-2012: {r}</p><a href="{l}">Ver la herramienta</a></template>'
    for k, n, t, r, l in PIEZAS)
globos = "".join(f'<button class="globo" type="button" tabindex="-1" aria-hidden="true" data-pieza="{k}">{i}</button>'
                 for i, (k, *_r) in enumerate(PIEZAS, 1))

VISTAS = [
    ("inicio", "Inicio", "Al abrir eliges entre dos experiencias independientes: Proyecto completo o Cálculo rápido de un circuito.", "Pantalla de inicio de IEEStudio con Proyecto completo y Cálculo rápido."),
    ("proyecto", "Proyecto", "Datos de la obra, suministro y la tabla de tableros con su estado. Lo que capturas aquí va a la portada de la memoria.", "Vista Proyecto con el resumen del proyecto de ejemplo."),
    ("tableros", "Tableros y circuitos", "El cuadro de cargas se recalcula con cada cambio: corriente, calibre, tierra, tubo, interruptor, caída de tensión y estado de cada circuito.", "Cuadro de cargas del tablero TG-1 con el estado de cada circuito y la ficha del circuito C-04."),
    ("procedimiento", "Procedimiento", "Cada resultado con su fórmula, los valores sustituidos y el artículo de la norma; lo mismo que va en la memoria.", "Panel de procedimiento del circuito C-04 con la corriente, la caída de tensión y sus referencias."),
    ("unifilar", "Unifilar", "El diagrama se dibuja desde el proyecto: acometida, alimentadores, tableros y circuitos. Se exporta a DXF.", "Diagrama unifilar del proyecto de ejemplo dibujado por IEEStudio."),
    ("revisar", "Revisar", "Todo lo que falta antes de entregar, en un solo lugar y con su arreglo: lo que no cumple, lo sin evaluar y lo fuera de alcance.", "Vista Revisar con lo que no cumple, lo que falta evaluar y lo que queda fuera de alcance."),
    ("memoria", "Memoria", "La vista previa de la memoria de cálculo, con los datos de firma y la lista de lo que conviene revisar antes de entregar.", "Vista previa de la memoria de cálculo en IEEStudio."),
    ("rapido", "Cálculo rápido", "Un circuito suelto, sin crear proyecto: corriente, interruptor, conductor, tierra, tubería y caída, con alternativas si no cumple.", "Cálculo rápido de un circuito con sus resultados."),
]
vistas_btn = "\n".join(
    f'            <button type="button" role="tab" id="tab-{k}" aria-controls="ventana-lienzo" aria-selected="{"true" if k == "tableros" else "false"}" tabindex="{0 if k == "tableros" else -1}" data-vista="{k}"><b>{n}</b><span>{d}</span></button>'
    for i, (k, n, d, a) in enumerate(VISTAS))
vistas_img = "\n".join(
    f'              <img src="imagenes/app/{k}.webp" srcset="imagenes/app/{k}.webp 1x, imagenes/app/{k}@2x.webp 2x" width="1440" height="900" alt="{a}" data-vista="{k}"{" class=\"ver\"" if k == "tableros" else ""} loading="lazy">'
    for i, (k, n, d, a) in enumerate(VISTAS))
MOVIL = {"inicio": "inicio", "proyecto": "proyecto", "tableros": "tableros", "procedimiento": "resultados", "unifilar": "unifilar", "revisar": "revisar", "memoria": "memoria", "rapido": "rapido"}
movil_img = "\n".join(
    f'            <img src="imagenes/app/movil-{m}.webp" width="390" height="844" alt="" data-movil="{m}"{" class=\"ver\"" if m == "tableros" else ""} loading="lazy">'
    for m in sorted(set(MOVIL.values())))

# Módulos de herramientas. Cada uno con su dibujo propio.
def dib(svg, vb="0 0 400 200"):
    return f'<svg class="dib" viewBox="{vb}" preserveAspectRatio="xMidYMid meet" aria-hidden="true">{svg}</svg>'

D_CAIDA = dib("""
<path class="fino" d="M40 60h330"/><text x="40" y="50">V fuente</text>
<path d="M40 60 L370 108" class="ambar"/><path class="pulso" d="M40 60 L370 108"/>
<text x="374" y="112" class="a">V carga</text>
<path class="fino" d="M370 60v48"/><text x="378" y="88">ΔV</text>
<g transform="translate(0 70)">
<path d="M40 120h40M80 112h40v16h-40zM120 120h52"/><text x="88" y="104">R</text>
<path d="M172 120c6-14 12-14 18 0s12 14 18 0 12-14 18 0 12 14 18 0"/><text x="190" y="104">X</text>
<path d="M244 120h126"/><circle class="relleno" cx="40" cy="120" r="3"/><circle cx="370" cy="120" r="10"/>
<path class="pulso" d="M40 120h330"/>
<path class="fino" d="M40 150h330M40 145v10M370 145v10"/><text x="196" y="168">L</text>
</g>
<text x="40" y="24" class="valor">ΔV = 2 · I · L · (R cos θ + X sen θ) / n</text>""", "0 0 460 250")

D_COND = dib("""
<circle cx="110" cy="100" r="64" class="tenue"/><circle cx="110" cy="100" r="48"/><circle cx="110" cy="100" r="30" class="ambar"/>
<circle class="relleno-ambar" cx="110" cy="100" r="30" opacity=".18"/>
<path class="fino" d="M164 60h40M158 100h46M140 125h64"/>
<text x="210" y="64">aislamiento</text><text x="210" y="104">cobre o aluminio</text><text x="210" y="129">60, 75 o 90 °C</text>
<text x="40" y="190" class="valor">I ≤ ampacidad × Ft × Fa</text>""", "0 0 400 200")

D_ITM = dib("""
<path class="fino" d="M30 120h340"/>
<g>""" + "".join(f'<path d="M{40 + i * 42} 112v16"/><text x="{40 + i * 42}" y="146" text-anchor="middle">{a}</text>' for i, a in enumerate([15, 20, 25, 30, 35, 40, 45, 50])) + """</g>
<path class="ambar" d="M124 70v50"/><text x="124" y="60" text-anchor="middle" class="a">I diseño</text>
<g class="marca-itm" style="--dx: 0px"><path d="M124 92 L166 92" class="tenue"/><circle cx="166" cy="120" r="7" class="ambar"/><circle class="relleno-ambar" cx="166" cy="120" r="3"/></g>
<text x="166" y="176" text-anchor="middle" class="valor">inmediato superior</text>""", "0 0 400 200")

D_TUBO = dib("""
<circle cx="200" cy="100" r="74"/><circle cx="200" cy="100" r="66" class="fino"/>
<circle cx="176" cy="84" r="20" class="relleno-gris"/><circle cx="222" cy="80" r="20" class="relleno-gris"/><circle cx="200" cy="122" r="20" class="relleno-gris"/>
<circle cx="236" cy="124" r="11" class="relleno-gris"/>
<text x="290" y="60">40 %</text><text x="290" y="78" class="a">ocupación</text>""", "0 0 400 200")

D_CUADRO = dib("""
<text x="30" y="56">A</text><text x="30" y="96">B</text><text x="30" y="136">C</text>
<rect class="relleno barra-fase" x="50" y="44" width="250" height="16" rx="3" opacity=".75" style="--bal: .88"/>
<rect class="relleno barra-fase" x="50" y="84" width="180" height="16" rx="3" opacity=".55" style="--bal: 1.22"/>
<rect class="relleno barra-fase" x="50" y="124" width="205" height="16" rx="3" opacity=".65" style="--bal: 1.07"/>
<path class="ambar" d="M270 34v116" stroke-dasharray="3 4"/><text x="276" y="172" class="a">promedio</text>""", "0 0 400 190")

D_MOTOR = dib("""
<circle cx="120" cy="100" r="46"/><text x="120" y="108" text-anchor="middle" class="valor" style="font-size:22px">M</text>
<path d="M120 20v34M120 146v24"/><path class="pulso" d="M120 20v34"/>
<path class="fino" d="M200 70h150M200 100h150M200 130h150"/>
<text x="200" y="64">I plena carga (tablas)</text><text x="200" y="94" class="a">conductor 125 %</text><text x="200" y="124">protección y sobrecarga</text>""", "0 0 400 190")

D_TRAFO = dib("""
<circle cx="150" cy="100" r="40"/><circle cx="200" cy="100" r="40"/>
<path d="M60 100h50M240 100h100"/><path class="pulso" d="M60 100h50"/><path class="pulso" d="M240 100h100"/>
<text x="60" y="88">primario</text><text x="270" y="88">secundario</text>
<text x="175" y="170" text-anchor="middle" class="valor">≤ 600 V</text>""", "0 0 400 190")

D_TIERRA = dib("""
<path d="M200 30v80"/><path d="M160 110h80M172 126h56M186 142h28"/>
<path class="pulso" d="M200 30v80"/>
<path class="fino" d="M70 60h130"/><text x="70" y="52">equipo</text>
<text x="250" y="70" class="a">250-122</text><text x="250" y="90">electrodo 250-66</text>""", "0 0 400 180")

D_FALLA = dib("""
<path d="M60 40h280"/><text x="60" y="30">transformador</text>
<path d="M120 40v50M120 90h60M240 40v90M240 130h60"/>
<rect x="104" y="90" width="32" height="20" rx="3"/><rect x="224" y="130" width="32" height="20" rx="3"/>
<path class="pulso" d="M60 40h280"/>
<text x="150" y="104" class="a">kA</text><text x="270" y="144" class="a">kA</text>
<text x="60" y="180">punto a punto, tablero por tablero</text>""", "0 0 400 190")

D_RAPIDO = dib("""
<rect x="90" y="30" width="220" height="140" rx="8"/><path class="fino" d="M90 58h220"/>
<path class="fino" d="M110 84h80M110 108h120M110 132h60"/>
<rect class="relleno-ambar" x="226" y="140" width="70" height="18" rx="4" opacity=".9"/>
<text x="110" y="49">1 circuito</text><text x="232" y="153" style="fill:#15130c">PDF</text>""", "0 0 400 200")

MODULOS = [
    ("m-caida", "Caída de tensión", D_CAIDA,
     "Por impedancia, con la resistencia y la reactancia de la Tabla 9 según el factor de potencia, para circuitos monofásicos y trifásicos, por tramo y acumulada desde la fuente.",
     "Te dice la longitud máxima que cumple y qué calibre la resuelve.",
     "Capítulo 10, Tabla 9; 210-19(a)(1), nota 4; 215-2(a)(1), nota 2", "guia/#paso-5"),
    ("m-conductores", "Conductores", D_COND,
     "El menor calibre que cumple por terminales, ampacidad corregida por temperatura ambiente y agrupamiento, protección y caída de tensión. Cobre o aluminio.",
     "Te dice cuál de los criterios decidió el calibre.",
     "110-14(c); Tablas 310-15(b)(16), (b)(2)(a) y (b)(3)(a); 310-10(h)", "funciones/#calculos"),
    ("m-protecciones", "Interruptores termomagnéticos", D_ITM,
     "El valor normalizado inmediato superior a la corriente de diseño, o el que tú fijes, verificado contra el conductor y sus límites para calibres pequeños.",
     "Si fijas uno, te dice cuál habría propuesto y por qué.",
     "210-20(a), 215-3; 240-6(a); 240-4, 240-4(d)", "guia/#paso-5"),
    ("m-tuberia", "Tubería conduit", D_TUBO,
     "EMT, IMC, RMC y PVC con el área real de cada conductor aislado, la ocupación permitida, niples y atascamiento.",
     "Diámetro comercial propuesto por circuito.",
     "Capítulo 10, Tablas 1, 4 y 5", "funciones/#calculos"),
    ("m-cuadro", "Cuadro de cargas", D_CUADRO,
     "Circuitos por tablero con su carga, fases y estado; carga del alimentador por el artículo 220 y desbalance entre fases.",
     "Balanceo automático que se puede deshacer; ida y vuelta con Excel.",
     "220-12, 220-40, 220-42, 220-44", "guia/#paso-4"),
    ("m-motores", "Motores y aire acondicionado", D_MOTOR,
     "Corriente de las tablas 430-248 y 430-250, conductor al 125 %, protección, relevador de sobrecarga y motocompresores por su placa.",
     "El motor mayor se considera en el alimentador.",
     "430-22, 430-24, 430-32, 430-52; 440-22, 440-32", "funciones/#calculos"),
    ("m-unifilar", "Diagrama unifilar", '<img src="imagenes/app/unifilar-detalle.webp" width="1200" height="892" alt="Detalle del diagrama unifilar que dibuja IEEStudio para el proyecto de ejemplo." loading="lazy">',
     "Se dibuja desde el proyecto con su simbología: acometida, transformador, alimentadores, tableros y circuitos con interruptor, conductores, tubería y caída.",
     "Modos Revisión y Plano, con notas y cajetín; exportación a DXF para AutoCAD.",
     "Generado por el programa", "guia/#paso-7"),
    ("m-transformador", "Transformadores", D_TRAFO,
     "Transformadores de 600 V o menos dentro de la instalación: primario y secundario, protección máxima y tensión propia del tablero que alimentan.",
     "La caída se cuenta desde el secundario.",
     "450-3(b), Tabla 450-3(b); 240-21(c)", "guia/#paso-3"),
    ("m-tierra", "Puesta a tierra", D_TIERRA,
     "Conductor de puesta a tierra de equipos por la protección del circuito y conductor al electrodo por el mayor conductor de la acometida.",
     "Con el incremento proporcional si las fases crecen.",
     "Tabla 250-122, 250-122(b); Tabla 250-66", "funciones/#calculos"),
    ("m-falla", "Capacidad interruptiva", D_FALLA,
     "Con los datos de la fuente, revisa cada tablero con una falla estimada como cota superior, descontando los alimentadores y sumando la aportación de motores. No es un estudio de cortocircuito.",
     "Se puede apagar; entonces queda anotada como fuera de alcance.",
     "110-9, 110-10", "funciones/#calculos"),
    ("m-rapido", "Cálculo rápido", D_RAPIDO,
     "Un circuito sin crear proyecto, con su hoja en PDF y alternativas ya calculadas si no cumple.",
     "Si te sirve, pasa al tablero seleccionado con un clic.",
     "Mismas reglas que el proyecto", "guia/#rapido"),
]
modulos_html = ""
for cls, nombre, svg, desc, beneficio, ref, doc in MODULOS:
    modulos_html += f"""        <article class="modulo {cls} revela">
          <div class="dibujo">{svg}</div>
          <div class="texto">
            <h3>{nombre}</h3>
            <p>{desc}</p>
            <p class="beneficio">{beneficio}</p>
            <p class="ref">{ref}</p>
            <a class="doc" href="{doc}">Documentación<span class="sr"> de {nombre.lower()}</span></a>
          </div>
        </article>
"""

PASOS = [
    ("captura", "Introduce los datos", "Obra, suministro, tableros y circuitos. Cada dato se captura una vez, en el nivel al que pertenece.",
     "Agregar circuito pregunta primero qué vas a conectar: alumbrado, contactos, motor, aire acondicionado u otra carga.",
     ["Datos de la obra y del suministro en la vista Proyecto.", "Tableros con su alimentación, conexión, longitud y áreas.", "Circuitos nuevos sin potencia ni longitud: quedan sin evaluar hasta que los capturas."],
     "imagenes/app/proyecto.webp", "Vista Proyecto del proyecto de ejemplo."),
    ("calculo", "Ejecuta los cálculos", "No hay botón de calcular: cada cambio recalcula el proyecto completo.",
     "Corriente, conductor, interruptor, tierra, tubería y caída de tensión de cada tramo, con la carga del alimentador por el artículo 220.",
     ["El motor no redondea: solo se redondea lo que se muestra.", "Si una tabla no cubre el caso, el resultado queda sin evaluar.", "El procedimiento muestra la fórmula, los valores y la referencia."],
     "imagenes/app/procedimiento.webp", "Panel de procedimiento con las fórmulas y los valores sustituidos."),
    ("revision", "Revisa los resultados", "Cada verificación dice si cumple, no cumple o no se pudo evaluar, y por qué.",
     "La vista Revisar junta lo que falta del proyecto con su arreglo, como «Usar 10 AWG», y separa lo que queda fuera de alcance.",
     ["Ningún resultado se marca conforme si falta un dato.", "Lo que la norma pide y el programa no verifica se anota.", "La aprobación profesional nunca la da el programa."],
     "imagenes/app/revisar.webp", "Vista Revisar del proyecto de ejemplo."),
    ("documento", "Genera la documentación", "La memoria de cálculo en PDF, el cuadro de cargas en Excel y el unifilar en DXF salen del mismo modelo.",
     "La memoria lleva los datos de la obra, criterios, cuadro de cargas, unifilar, observaciones y el procedimiento de cada tramo.",
     ["Responsable, cédula y revisión en cada hoja.", "Observaciones con todo lo que no cumple o no se evaluó.", "El .iees abre igual en el navegador y en Windows."],
     "imagenes/app/memoria.webp", "Vista Memoria con la vista previa del PDF."),
]
pasos_btn = "\n".join(
    f'          <button class="paso" type="button" role="tab" id="paso-{k}" aria-controls="detalle-{k}" aria-selected="{"true" if i == 0 else "false"}" tabindex="{0 if i == 0 else -1}"><span class="nodo">{i + 1}</span><h3>{t}</h3><p>{s}</p></button>'
    for i, (k, t, s, *_r) in enumerate(PASOS))
pasos_det = ""
for i, (k, t, s, d, lis, img, alt) in enumerate(PASOS):
    li = "".join(f"<li>{x}</li>" for x in lis)
    pasos_det += f"""        <div class="detalle-paso" role="tabpanel" id="detalle-{k}" aria-labelledby="paso-{k}"{"" if i == 0 else " hidden"}>
          <div><h3>{t}</h3><p>{d}</p><ul>{li}</ul></div>
          <figure><img src="{img}" width="1440" height="900" alt="{alt}" loading="lazy"></figure>
        </div>
"""

# Hojas reales de la memoria del proyecto de ejemplo (cómo se generan: website/README.md).
HOJAS = [
    ("1", "Portada, datos y criterios", "Primera hoja de la memoria: datos de la obra, criterios de diseño y resumen."),
    ("2", "Cuadro de cargas", "Hoja de la memoria con la carga del alimentador y el cuadro de cargas del tablero TG-1."),
    ("3", "Diagrama unifilar", "Hoja de la memoria con el diagrama unifilar, su simbología, notas y cajetín."),
    ("4", "Observaciones", "Hoja de la memoria con las observaciones y el inicio del procedimiento del alimentador."),
    ("9", "Procedimiento de cada tramo", "Hoja de la memoria con el procedimiento de cálculo de un circuito y sus referencias."),
    ("13", "Firmas", "Última hoja de la memoria con los espacios para firmas."),
]
hojas_btn = "\n".join(
    f'            <li><button type="button" data-hoja="{i}" aria-pressed="{"true" if i == 0 else "false"}">{n}<span class="pg">hoja {h} de 13</span></button></li>'
    for i, (h, n, a) in enumerate(HOJAS))
nh = len(HOJAS)
hojas_img = "\n".join(
    f'            <button class="hoja{" al-frente" if i == 0 else ""}" type="button" data-hoja="{i}" style="--i:{nh - 1 - i};--abs:{abs(nh - 1 - i - (nh - 1) / 2)};z-index:{nh - i}" aria-label="Ampliar: {n}"><img src="imagenes/memoria/hoja-{h}.webp" data-grande="imagenes/memoria/hoja-{h}-grande.webp" width="1020" height="1320" alt="{a}" loading="lazy"></button>'
    for i, (h, n, a) in enumerate(HOJAS))

DOCS_USO = [
    ("guia/#video", "Guía en video", "Un recorrido de pocos minutos por un proyecto completo.", "guía"),
    ("guia/#pasos", "Paso a paso", "De la acometida a la memoria, con capturas de cada vista.", "guía"),
    ("guia/#rapido", "Cálculo rápido", "Un circuito suelto sin crear proyecto.", "guía"),
    ("guia/#atajos", "Atajos de teclado", "Vistas, guardar, exportar y la paleta de comandos.", "guía"),
    ("guia/#preguntas", "Preguntas frecuentes", "Privacidad, archivos, teléfono y aprobación.", "guía"),
]
DOCS_TEC = [
    (f"{GH}/blob/main/docs/engineering-methodology.md", "Metodología de ingeniería", "Cómo se calcula, qué se verifica y qué no.", "GitHub"),
    (f"{GH}/blob/main/docs/normative-sources.md", "Fuentes normativas", "Qué tablas y artículos usa el programa y de dónde salen.", "GitHub"),
    (f"{GH}/tree/main/data/nom-001-sede-2012", "Tablas de la norma en datos", "Los CSV que usa el motor, revisables en una hoja de cálculo.", "GitHub"),
    (f"{GH}/tree/main/docs/decisions", "Decisiones de diseño", "El porqué de cada criterio, registro por registro.", "GitHub"),
    (f"{GH}/blob/main/docs/architecture.md", "Arquitectura", "Cómo se organiza el código en crates.", "GitHub"),
]
def indice(items):
    return "\n".join(
        f'            <li><a href="{u}"><span><span class="t">{t}</span><span class="d">{d}</span></span><span class="puntos" aria-hidden="true"></span><span class="dest">{x}</span></a></li>'
        for u, t, d, x in items)

ld = """
<script type="application/ld+json">
{"@context":"https://schema.org","@type":"SoftwareApplication","name":"IEEStudio","url":"https://ieestudio.com/",
"applicationCategory":"EngineeringApplication","operatingSystem":"Windows 10, Windows 11, navegador web",
"description":"Cálculo de instalaciones eléctricas de baja tensión conforme a la NOM-001-SEDE-2012, con cuadro de cargas, diagrama unifilar y memoria de cálculo.",
"inLanguage":"es","isAccessibleForFree":true,"license":"https://github.com/JAAM08/IEEStudio#licencia",
"offers":{"@type":"Offer","price":"0","priceCurrency":"MXN"},
"author":{"@type":"Person","name":"Juan Antonio Aguilar Martínez","url":"https://github.com/JAAM08"},
"codeRepository":"https://github.com/JAAM08/IEEStudio"}
</script>"""

inicio = head(p, "", DESC_INICIO, "", ld) + f"""  <section class="hero" id="inicio" aria-labelledby="titulo">
    <div class="escena" id="escena">
      {plano_tablero("respaldo")}
    </div>
    <div class="globos" id="globos"><svg aria-hidden="true"></svg>{globos}<div class="punta" id="punta" role="status"></div></div>

    <div class="wrap hero-texto">
      <p class="estado"><span class="led" aria-hidden="true"></span><span>En desarrollo activo. Versión <span class="version-tag">de prueba</span></span></p>
      <h1 id="titulo">Ingeniería eléctrica. <span>Con precisión digital.</span></h1>
      <p class="lead">Una plataforma de herramientas para calcular, diseñar y documentar instalaciones eléctricas de baja tensión conforme a la NOM-001-SEDE-2012, con cada paso a la vista.</p>
      <div class="acciones">
        <a class="btn btn-ambar" href="app/">{icon(p, "i-play")}Explorar IEEStudio</a>
        <a class="btn btn-linea" href="#herramientas">Conocer las herramientas</a>
      </div>
      <p class="nota">Gratis y de código abierto. Funciona en el navegador o se <a href="descargar/">instala en Windows</a>.</p>
    </div>

    <aside class="simbologia" aria-labelledby="simb-titulo">
      <h2 id="simb-titulo">Simbología del tablero</h2>
      <ol>
{simbologia}
      </ol>
      <div class="ficha-pieza" id="ficha-pieza" aria-live="polite" hidden></div>
{fichas}
    </aside>

    <dl class="cajetin" aria-label="Datos del modelo">
      <div><dt>Proyecto</dt><dd>Casa habitación (ejemplo)</dd></div>
      <div><dt>Tablero</dt><dd>TG-1, 3F-4H 220Y/127 V</dd></div>
      <div><dt>Norma</dt><dd>NOM-001-SEDE-2012</dd></div>
      <div><dt>Versión</dt><dd class="version-tag">de prueba</dd></div>
    </dl>
  </section>

  <section class="lamina" id="producto" aria-labelledby="t-producto">
    <div class="wrap">
      <p class="rotulo"><b>01</b>El programa</p>
      <div class="cabeza">
        <h2 id="t-producto">La aplicación, tal como es.</h2>
        <p class="intro">Capturas del programa con el proyecto de ejemplo que trae: una casa habitación con un tablero y seis circuitos. Lo que ves aquí es lo que abres en el navegador o en Windows.</p>
      </div>
      <div class="programa">
        <div class="vistas" role="tablist" aria-label="Vistas del programa" aria-orientation="vertical">
{vistas_btn}
        </div>
        <div class="mesa">
          <div class="ventana">
            <div class="ventana-barra">
              <span class="ctl" aria-hidden="true"><i></i><i></i><i></i></span>
              <span class="titulo"><img src="app/icono.svg" alt="">IEEStudio</span>
              <button class="ampliar" type="button" data-ampliar="#ventana-lienzo">Ampliar</button>
            </div>
            <div class="lienzo" id="ventana-lienzo" role="tabpanel" aria-live="polite">
{vistas_img}
            </div>
          </div>
          <div class="telefono" aria-hidden="true">
{movil_img}
          </div>
        </div>
      </div>
    </div>
  </section>

  <section class="lamina clara" id="herramientas" aria-labelledby="t-herramientas">
    <div class="wrap">
      <p class="rotulo"><b>02</b>Herramientas</p>
      <div class="cabeza">
        <h2 id="t-herramientas">Lo que lleva una instalación de baja tensión.</h2>
        <p class="intro">Cada herramienta cita el artículo o la tabla de la NOM-001-SEDE-2012 que la respalda. Si falta un dato o la tabla no cubre el caso, no se supone un valor: el resultado queda sin evaluar.</p>
      </div>
      <div class="modulos">
{modulos_html}      </div>
    </div>
  </section>

  <section class="lamina" id="flujo" aria-labelledby="t-flujo">
    <div class="wrap">
      <p class="rotulo"><b>03</b>Cómo funciona</p>
      <div class="cabeza">
        <h2 id="t-flujo">De los datos de la obra a la memoria firmada.</h2>
        <p class="intro">Un solo modelo del proyecto: el cuadro de cargas, el unifilar y la memoria salen de los mismos datos y se actualizan juntos.</p>
      </div>
      <div class="flujo" id="flujo-diagrama">
        <div class="bus" aria-hidden="true">
          <svg viewBox="0 0 1000 120" preserveAspectRatio="none">
            <path class="base" d="M22 82H1000"/>
            <path class="carga" pathLength="1" d="M22 82H1000"/>
          </svg>
        </div>
        <div class="pasos" role="tablist" aria-label="Pasos del flujo de trabajo">
{pasos_btn}
        </div>
{pasos_det}      </div>
    </div>
  </section>

  <section class="lamina clara" id="memoria" aria-labelledby="t-memoria">
    <div class="wrap memoria">
      <div>
        <p class="rotulo"><b>04</b>Memoria técnica</p>
        <h2 id="t-memoria">La memoria se arma sola, hoja por hoja.</h2>
        <p class="intro">Un PDF con lo que pide una revisión: datos de la obra, criterios de diseño, cuadro de cargas, unifilar, observaciones y el procedimiento de cada tramo con su referencia. Estas son hojas reales del proyecto de ejemplo.</p>
        <ol class="indice-memoria">
{hojas_btn}
        </ol>
      </div>
      <div class="hojas" id="hojas">
{hojas_img}
        <p class="leyenda">Memoria del proyecto de ejemplo, generada por IEEStudio</p>
      </div>
    </div>
  </section>

  <section class="lamina" id="filosofia" aria-labelledby="t-filosofia">
    <div class="wrap">
      <p class="rotulo"><b>05</b>Filosofía</p>
      <h2 id="t-filosofia" class="sr">Filosofía del proyecto</h2>
      <div class="filosofia">
        <p class="manifiesto revela">Un programa de ingeniería tiene que poder revisarse. <span>Por eso IEEStudio muestra cada fórmula, cita cada tabla y dice con claridad lo que no sabe, en lugar de esconderlo detrás de un resultado verde.</span></p>
        <dl class="principios revela">
          <div><dt>sin redondeo</dt><dd><b>Calcula con precisión completa</b>Solo redondea lo que se muestra. Cualquier número se puede comprobar a mano.</dd></div>
          <div><dt>sin suposiciones</dt><dd><b>Nunca inventa un dato</b>Si la tabla no cubre el caso o falta un valor, el resultado queda sin evaluar y dice qué falta.</dd></div>
          <div><dt>datos abiertos</dt><dd><b>La norma vive en tablas revisables</b>Ningún valor normativo está escrito en el código: las tablas están en archivos CSV y cada regla tiene sus pruebas.</dd></div>
          <div><dt>firma humana</dt><dd><b>La aprobación es del ingeniero</b>El programa verifica contra la norma, pero nunca marca un proyecto como aprobado.</dd></div>
        </dl>
      </div>
    </div>
  </section>

  <section class="lamina clara" id="documentacion" aria-labelledby="t-docs">
    <div class="wrap">
      <p class="rotulo"><b>06</b>Documentación</p>
      <div class="cabeza">
        <h2 id="t-docs">Aprende a usarlo y revisa cómo calcula.</h2>
        <p class="intro">La guía explica el uso con capturas y video. La documentación técnica, en el repositorio, explica los criterios de cálculo y sus fuentes.</p>
      </div>
      <div class="docs">
        <div>
          <h3>Uso del programa</h3>
          <ul class="indice">
{indice(DOCS_USO)}
          </ul>
        </div>
        <div>
          <h3>Referencia técnica</h3>
          <ul class="indice">
{indice(DOCS_TEC)}
          </ul>
        </div>
      </div>
    </div>
  </section>

  <section class="lamina" id="comunidad" aria-labelledby="t-comunidad">
    <div class="wrap">
      <p class="rotulo"><b>07</b>Comunidad y desarrollo</p>
      <div class="cabeza">
        <h2 id="t-comunidad">Un proyecto abierto, en desarrollo.</h2>
        <p class="intro">El código, las tablas y los casos de prueba son públicos. Las versiones se compilan en GitHub desde ese código; nadie sube un instalador a mano.</p>
      </div>
      <div class="comunidad">
        <div class="panel repo">
          <h3>{icon(p, "i-git")}Repositorio</h3>
          <a class="url" href="{GH}">github.com/JAAM08/IEEStudio</a>
          <dl>
            <div><dt>Licencia</dt><dd>MIT o Apache-2.0</dd></div>
            <div><dt>Lenguaje</dt><dd>Rust y egui</dd></div>
            <div><dt>Plataformas</dt><dd>Windows y navegador</dd></div>
            <div><dt>Norma</dt><dd>NOM-001-SEDE-2012</dd></div>
          </dl>
          <ul class="contribuir">
            <li><a href="{GH}/issues/new?title=Error%3A%20">{icon(p, "i-fail")}<span><b>Reportar un error</b><span>Si es de cálculo, adjunta el .iees o los datos del circuito.</span></span></a></li>
            <li><a href="{GH}/issues/new?title=Propuesta%3A%20">{icon(p, "i-steps")}<span><b>Proponer una función</b><span>Qué necesitas calcular y con qué referencia.</span></span></a></li>
            <li><a href="{GH}/blob/main/CONTRIBUTING.md">{icon(p, "i-code")}<span><b>Contribuir</b><span>Cómo preparar un cambio y sus pruebas.</span></span></a></li>
          </ul>
        </div>
        <div class="panel">
          <h3>{icon(p, "i-down")}Versiones publicadas</h3>
          <ul class="lista-vivo" id="versiones"><li class="cargando">Consultando GitHub… <a href="{GH}/releases">Ver versiones</a></li></ul>
        </div>
        <div class="panel">
          <h3>{icon(p, "i-build")}Cambios recientes</h3>
          <ul class="lista-vivo" id="cambios"><li class="cargando">Consultando GitHub… <a href="{GH}/commits/main">Ver historial</a></li></ul>
        </div>
      </div>
    </div>
  </section>

  <section class="cierre" id="descargar" aria-labelledby="t-descargar">
    <div class="wrap cierre-grid">
      <div>
        <p class="rotulo"><b>08</b>Descarga</p>
        <h2 id="t-descargar">Calcula tu siguiente instalación. <span>En el navegador o en Windows.</span></h2>
        <p class="version">Versión de prueba: revisa los resultados antes de usarlos en un proyecto.</p>
        <div class="acciones">
          <a class="btn btn-ambar" href="app/">{icon(p, "i-play")}Abrir en el navegador</a>
          <a class="btn btn-linea descargar" href="{GH}/releases">{icon(p, "i-down")}Descargar para Windows</a>
        </div>
        <div class="plataformas">
          <div><h3>Navegador</h3><p>Chrome, Edge o Firefox recientes. Se instala como app y funciona sin internet.</p></div>
          <div><h3>Windows</h3><p>Windows 10 u 11 de 64 bits. Instalador con la línea de comandos incluida.</p></div>
          <div><h3>Linux y otros</h3><p>Se compila desde el <a href="{GH}">código fuente</a> con Rust.</p></div>
        </div>
      </div>
      <div class="alzado">{plano_tablero("", "Alzado frontal del tablero de la portada, con sus cotas en milímetros")}</div>
    </div>
  </section>

  <dialog class="visor" id="visor" aria-label="Imagen ampliada">
    <div class="visor-barra"><span id="visor-texto"></span><button type="button" data-cerrar>Cerrar</button></div>
    <img id="visor-img" alt="">
  </dialog>
""" + foot(p, '\n<script type="module" src="portada.js"></script>')
write("", inicio)


# ---------------------------------------------------------------- páginas interiores
def cabecera(p, rotulo, titulo, intro, extra=""):
    return f"""  <section class="cabecera">
    <div class="wrap">
      <p class="rotulo">{rotulo}</p>
      <h1>{titulo}</h1>
      <p class="intro">{intro}</p>{extra}
    </div>
  </section>
"""


def botones(p):
    return f"""<a class="btn btn-ambar" href="{p}app/">{icon(p, "i-play")}Abrir en el navegador</a>
          <a class="btn btn-linea descargar" href="{GH}/releases">{icon(p, "i-down")}Descargar para Windows</a>"""


def proc_card(p):
    return f"""<figure class="proc" aria-label="Procedimiento del circuito C-04 del proyecto de ejemplo">
        <div class="proc-head"><b>C-04 Contactos sala</b><span>1,500 W, 127 V, 35 m, 12 AWG</span></div>
        <ol>
          <li>
            <h3>Corriente nominal</h3>
            <p class="f">I = P / (V · fp)</p>
            <p class="s">I = 1,500 / (127 · 0.90) = 13.12 A</p>
          </li>
          <li>
            <h3>Caída de tensión</h3>
            <p class="f">ΔV = 2 · I · L · (R · cos θ + X · sen θ) / n</p>
            <p class="s">ΔV = 2 · 13.12 · 0.0350 · (6.6 · 0.90 + 0.223 · 0.436) / 1 = 5.55 V</p>
            <p class="ref">NOM-001-SEDE-2012: Capítulo 10, Tabla 9</p>
          </li>
          <li>
            <h3>Caída acumulada desde la fuente</h3>
            <p class="s">e total = 0.68 + 4.37 = <span class="mal">5.05 %</span></p>
            <p class="n">5.05 % es mayor que el límite de 5.00 %.</p>
            <p class="ref">NOM-001-SEDE-2012: 210-19(a)(1), nota 4; 215-2(a)(1), nota 2</p>
          </li>
        </ol>
        <div class="proc-foot">
          <span class="veredicto">{icon(p, "i-fail")}No cumple</span>
          <span>Con <b>10 AWG</b> cumple; se aplica con un clic.</span>
        </div>
      </figure>"""


# ---------------------------------------------------------------- herramientas
p = "../"
calc = [
    ("Conductores", "Menor calibre por terminales, temperatura, agrupamiento, protección y caída de tensión. Conductores en paralelo, un juego por tubo, cuando hacen falta."),
    ("Protecciones", "Interruptor normalizado inmediato superior a la corriente de diseño, o el que tú fijes, verificado contra el conductor."),
    ("Caída de tensión", "Por impedancia con la Tabla 9, por tramo y acumulada desde la fuente, con la longitud máxima permitida."),
    ("Carga del alimentador", "Artículo 220: alumbrado por área según el tipo de inmueble, contactos, aparatos fijos y cocina, con sus factores de demanda."),
    ("Motores y clima", "Corrientes de las tablas 430-248 y 430-250, conductor al 125 %, interruptor, relevador de sobrecarga y el motor mayor en el alimentador."),
    ("Transformadores", "Hasta 600 V: primario y secundario, protección de la Tabla 450-3(b) y tensión propia del tablero."),
    ("Puesta a tierra", "Conductor de puesta a tierra de equipos (Tabla 250-122) y conductor al electrodo de la acometida (Tabla 250-66)."),
    ("Tubería conduit", "EMT, IMC, RMC y PVC: ocupación de 53, 31 y 40 %, niples al 60 % y verificación de atascamiento."),
    ("Capacidad interruptiva", "Revisión por tablero con la falla estimada como cota superior, punto a punto desde la fuente. Se puede apagar; no es un estudio de cortocircuito."),
    ("Balanceo de fases", "Reparte los circuitos de un tablero entre las fases para dejarlo lo más balanceado posible, y se puede deshacer."),
    ("Cálculo rápido", "Un circuito sin armar proyecto, con su hoja en PDF y alternativas ya calculadas si no cumple."),
    ("Excel y DXF", "El cuadro de cargas va y viene de Excel; el unifilar se exporta a DXF para AutoCAD."),
]
calc_html = "\n".join(f"        <div><dt>{a}</dt><dd>{b}</dd></div>" for a, b in calc)
funciones = head(p, "Herramientas", "Qué calcula IEEStudio, cómo muestra cada paso con su artículo de la NOM-001-SEDE-2012 y qué entrega: cuadro de cargas, diagrama unifilar y memoria de cálculo.", "funciones/") + cabecera(
    p, "Herramientas", "Nada sale de una caja negra.",
    "Cada resultado trae su fórmula, los valores sustituidos y el artículo de la norma. Si algo no cumple te dice por qué y cómo corregirlo; si falta un dato, no lo inventa.",
) + f"""
  <section class="bloque">
    <div class="wrap">
      <h2>El procedimiento de cada circuito.</h2>
      <div class="dividido">
      {proc_card(p)}
        <div class="puntos-lista">
          <div>
            <h3>La fórmula, los valores y el artículo</h3>
            <p>Cada circuito y alimentador muestra su procedimiento con los valores sustituidos y la tabla o artículo que lo respalda. Es lo mismo que va en la memoria.</p>
          </div>
          <div>
            <h3>Te dice qué criterio decidió</h3>
            <p>Si el calibre lo determinó la ampacidad, la protección o la caída de tensión, lo dice. Si algo no cumple, propone alternativas ya calculadas.</p>
          </div>
          <div>
            <h3>No redondea por ti</h3>
            <p>Calcula con precisión completa y solo redondea lo que se muestra. Puedes comprobar cualquier número a mano.</p>
          </div>
        </div>
      </div>

      <div class="veredictos">
        <div class="veredicto-c ok">
          <span class="nombre">{icon(p, "i-ok")}Cumple</span>
          <p>Se calculó con todos sus datos y pasó cada verificación.</p>
          <p class="eg">C-01: caída de <span class="mono">2.76 %</span> de <span class="mono">3.00 %</span> permitido.</p>
        </div>
        <div class="veredicto-c mal">
          <span class="nombre">{icon(p, "i-fail")}No cumple</span>
          <p>Al menos una verificación falla. Dice cuál y cómo corregirla.</p>
          <p class="eg">C-04: caída acumulada de <span class="mono">5.05 %</span> de <span class="mono">5.00 %</span>.</p>
        </div>
        <div class="veredicto-c pend">
          <span class="nombre">{icon(p, "i-pending")}Sin evaluar</span>
          <p>Falta un dato o la norma no cubre el caso. Nunca supone un valor.</p>
          <p class="eg">C-06: falta la corriente de placa del motor para el relevador de sobrecarga.</p>
        </div>
      </div>
      <p class="aprobacion"><b>La aprobación es tuya.</b> IEEStudio verifica contra la norma, pero nunca marca un proyecto como aprobado: esa firma la da el ingeniero responsable.</p>
    </div>
  </section>

  <section class="bloque" id="entregables">
    <div class="wrap">
      <h2>Del cuadro de cargas a la memoria.</h2>
      <div class="exhibe">
        <div class="exhibe-media"><div class="marco"><img src="{p}imagenes/app/tableros.webp" width="1440" height="900" alt="Cuadro de cargas con el estado de cada circuito." loading="lazy"></div></div>
        <div class="exhibe-texto">
          <h3>Cuadro de cargas que se calcula solo</h3>
          <p>Cada circuito con su carga, corriente, calibre, tierra, tubería, interruptor y caída de tensión, y el alimentador con la carga del artículo 220. El filtro «Pendientes» deja solo lo que falta.</p>
          <ul><li>Carga por fase y desbalance del tablero.</li><li>Caída acumulada desde la fuente.</li></ul>
        </div>
      </div>
      <div class="exhibe inv">
        <div class="exhibe-media"><div class="marco"><img src="{p}imagenes/app/unifilar.webp" width="1440" height="900" alt="Diagrama unifilar dibujado por IEEStudio." loading="lazy"></div></div>
        <div class="exhibe-texto">
          <h3>El unifilar se dibuja solo</h3>
          <p>Acometida, alimentadores, tableros y circuitos con su interruptor, conductores, tubería y caída de tensión. Cambia un dato y el diagrama se actualiza.</p>
          <ul><li>Se exporta a DXF para seguir dibujando en AutoCAD.</li><li>Va incluido en la memoria de cálculo.</li></ul>
        </div>
      </div>
      <div class="exhibe">
        <div class="exhibe-media"><div class="par-hojas">
          <img src="{p}imagenes/memoria/hoja-1.webp" width="1020" height="1320" alt="Primera hoja de la memoria: datos de la obra, criterios de diseño y resumen." loading="lazy">
          <img src="{p}imagenes/memoria/hoja-2.webp" width="1020" height="1320" alt="Hoja de la memoria con el cuadro de cargas." loading="lazy">
        </div></div>
        <div class="exhibe-texto">
          <h3>La memoria, lista para revisar y firmar</h3>
          <p>Un PDF con los datos de la obra, los criterios de diseño, el cuadro de cargas, el diagrama unifilar, las observaciones y el procedimiento de cada tramo.</p>
          <ul><li>Responsable, cédula y revisión en cada hoja.</li><li>Espacio para firmas al final.</li></ul>
        </div>
      </div>
      <div class="exhibe inv">
        <div class="exhibe-media"><div class="marco"><img src="{p}imagenes/app/rapido.webp" width="1440" height="900" alt="Cálculo rápido de un circuito." loading="lazy"></div></div>
        <div class="exhibe-texto">
          <h3>Un circuito, sin armar proyecto</h3>
          <p>El cálculo rápido te da corriente, interruptor, conductor, tierra, tubería y caída de tensión de un solo circuito, con su hoja en PDF. Si te sirve, lo agregas al proyecto con un clic.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="bloque" id="calculos">
    <div class="wrap">
      <h2>Lo que calcula hoy.</h2>
      <p class="intro">Instalaciones de baja tensión conforme a la NOM-001-SEDE-2012. El programa está en desarrollo: lo que no aparece aquí todavía no lo calcula.</p>
      <dl class="calculos">
{calc_html}
      </dl>
    </div>
  </section>

  <section class="bloque">
    <div class="wrap">
      <div class="banda">
        <div>
          <h2>Las tablas de la norma, en datos abiertos.</h2>
          <p>Las tablas que usa el programa están en archivos que cualquiera puede revisar, cotejadas contra una captura independiente. El código no tiene valores de la norma escritos a mano.</p>
        </div>
        <a class="btn btn-linea" href="{p}codigo-abierto/#verificacion">Cómo se verifica</a>
      </div>
    </div>
  </section>

  <section class="bloque">
    <div class="wrap">
      <h2>Míralo con el proyecto de ejemplo.</h2>
      <p class="intro">Abre la app con el ejemplo o empieza el tuyo desde cero.</p>
      <div class="acciones mt">
        {botones(p)}
      </div>
    </div>
  </section>
""" + foot(p)
write("funciones", funciones)

# ---------------------------------------------------------------- documentación (guía)
steps = [
    ("paso-1", "Abre IEEStudio", "Al abrir eliges entre dos experiencias: <strong>Proyecto completo</strong> —uno nuevo, uno que ya tienes o el ejemplo, una casa habitación con un tablero y seis circuitos— y <strong>Cálculo rápido</strong>, para un circuito suelto sin crear un proyecto.",
     ["Arriba están las cinco vistas: Proyecto, Tableros y circuitos, Unifilar, Revisar y Memoria (<kbd>F2</kbd> a <kbd>F6</kbd>).", "Lo que eliges en una vista lo siguen las demás."]),
    ("paso-2", "Datos de la obra y la acometida", "En la vista Proyecto capturas el cliente, la ubicación, quién elabora, el tipo de inmueble y el suministro. Arriba, el resumen dice cómo va el proyecto.",
     ["Los datos de la obra van a la portada de la memoria; no cambian el cálculo.", "Los límites de caída de tensión se ajustan aquí."]),
    ("paso-3", "Tableros y carga del alimentador", "En Tableros y circuitos elige un tablero: en su ficha, a la derecha, capturas de dónde se alimenta, su conexión, la longitud del alimentador y las áreas que atiende. Con eso se calcula la carga según el artículo 220, con el detalle de cada grupo en el procedimiento.",
     ["Si el tablero se alimenta por un transformador, márcalo y captura sus datos.", "Las áreas definen la carga mínima de alumbrado (Tabla 220-12)."]),
    ("paso-4", "Circuitos en el cuadro de cargas", "Con «Agregar circuito» eliges qué vas a conectar y capturas su potencia, fases y longitud en la ficha. El cuadro de cargas se recalcula con cada cambio: corriente, calibre, tierra, tubo, interruptor, caída de tensión y estado.",
     ["Ctrl+D duplica el circuito seleccionado y Supr lo elimina.", "«Esencial» y «Completo» cambian las columnas del cuadro.", "«Balancear fases» reparte los circuitos entre A, B y C."]),
    ("paso-5", "Revisa los resultados", "En la ficha, la pestaña Resultados muestra lo que calculó el programa para el elemento elegido. Si algo no cumple, arriba de la ficha está el arreglo, como «Usar 10 AWG»; si fijaste un calibre o un interruptor, te dice qué habría propuesto y por qué.",
     ["Puedes fijar calibre, interruptor y conductores por fase, o dejar que IEEStudio proponga el menor que cumple."]),
    ("paso-6", "Revisa la normativa", "La pestaña Normativa lista cada verificación —ampacidad, protección, caída, tubería— con su estado y el artículo que aplica. La vista Revisar junta todo lo que falta del proyecto, con su arreglo; lo que la norma pide y el programa no verifica aparece como fuera de alcance.",
     ["El procedimiento (<kbd>F7</kbd>) muestra abajo la fórmula y los valores de cada paso."]),
    ("paso-7", "El diagrama unifilar", "La vista Unifilar muestra el diagrama dibujado a partir del proyecto; se acerca con Ctrl y la rueda o pellizcando, y se recorre arrastrando. No se dibuja a mano: cualquier cambio en el cuadro de cargas se refleja aquí.",
     ["«Exportar DXF», para seguir en AutoCAD."]),
    ("paso-8", "La memoria de cálculo", "La vista Memoria muestra la vista previa con los datos de la obra, el cuadro de cargas, el unifilar, las observaciones y el procedimiento. A la derecha capturas la firma y revisas «Antes de entregar». Expórtala a PDF con Ctrl+E.",
     ["Las observaciones listan todo lo que no cumple o no se evaluó."]),
    ("paso-9", "Guarda y exporta", "En Windows, Guardar escribe un archivo .iees en tu carpeta. En el navegador, el proyecto se guarda solo en ese navegador y «Descargar proyecto» te da el .iees para tener tu copia.",
     ["El cuadro de cargas se exporta a Excel y se importa de vuelta.", "<kbd>Ctrl</kbd> <kbd>K</kbd> busca cualquier comando, tablero o circuito.", "Un .iees abre igual en la app en línea y en la de Windows."]),
]
steps_html = ""
for k, (img, title, text, bullets) in enumerate(steps, 1):
    lis = "".join(f"<li>{b}</li>" for b in bullets)
    steps_html += f"""        <div class="guia-paso" id="{img}">
          <div>
            <span class="num">{k}</span>
            <h3>{title}</h3>
            <p>{text}</p>
            <ul>{lis}</ul>
          </div>
          <figure><img src="{p}imagenes/guia/{img}.webp" width="1600" height="1000" alt="{title} en IEEStudio; el recuadro marca la zona del paso." loading="lazy"></figure>
        </div>
"""
keys = [
    ("<kbd>Ctrl</kbd> <kbd>N</kbd>", "Nuevo proyecto"),
    ("<kbd>Ctrl</kbd> <kbd>O</kbd>", "Abrir un proyecto (en el navegador, usa Archivo, Abrir)"),
    ("<kbd>Ctrl</kbd> <kbd>S</kbd>", "Guardar; en el navegador, descargar el proyecto"),
    ("<kbd>Ctrl</kbd> <kbd>E</kbd>", "Exportar la memoria en PDF"),
    ("<kbd>Ctrl</kbd> <kbd>Z</kbd> / <kbd>Ctrl</kbd> <kbd>Y</kbd>", "Deshacer y rehacer"),
    ("<kbd>Ctrl</kbd> <kbd>D</kbd>", "Duplicar el circuito seleccionado"),
    ("<kbd>Supr</kbd>", "Eliminar el circuito seleccionado"),
    ("<kbd>Ctrl</kbd> <kbd>K</kbd>", "Paleta de comandos: busca cualquier comando, tablero o circuito"),
    ("<kbd>F2</kbd> a <kbd>F8</kbd>", "<kbd>F2</kbd> a <kbd>F6</kbd>: las vistas Proyecto, Tableros y circuitos, Unifilar, Revisar y Memoria; <kbd>F7</kbd>, el procedimiento; <kbd>F8</kbd>, el cálculo rápido. En el navegador, F5 recarga la página: usa la barra de vistas."),
]
keys_html = "\n".join(f"        <tr><td>{a}</td><td>{b}</td></tr>" for a, b in keys)
faq = [
    ("¿Cuesta algo?", "No. IEEStudio es gratuito y de código abierto, también para uso profesional. No pide cuenta ni registro."),
    ("¿Mis proyectos se suben a internet?", "No. En la versión en línea todo se calcula en tu navegador y el proyecto se guarda en ese mismo navegador. En Windows se guarda en tus carpetas. Ningún proyecto pasa por un servidor."),
    ("¿Si borro los datos del navegador pierdo el proyecto?", "El proyecto que tenías abierto, sí. Por eso conviene descargar el .iees de vez en cuando: es tu copia y la puedes abrir en cualquier computadora."),
    ("¿La memoria que genera ya está aprobada?", "No. IEEStudio verifica contra la norma y lo dice claramente, pero la aprobación la da el ingeniero responsable que revisa y firma. El programa nunca marca un proyecto como aprobado."),
    ("¿Qué hago si algo dice «sin evaluar»?", "Lee el motivo en la pestaña Normativa o en las observaciones de la memoria: casi siempre falta un dato, como la corriente de placa de un motor. Captúralo y se evalúa."),
    ("¿Calcula el cortocircuito?", "Revisa la capacidad interruptiva de cada tablero con una falla estimada como cota superior; no es un estudio completo de cortocircuito. La revisión se puede apagar y entonces queda anotada como fuera de alcance."),
    ("¿Funciona en el celular?", "Sí. En el teléfono la app usa una barra inferior con las cinco vistas, listas que se tocan y la ficha a pantalla completa; en una tableta, el cuadro completo con cajones para el navegador y la ficha. Todavía no se ha probado en un teléfono real con iOS; si algo no responde, repórtalo."),
    ("¿Por qué Windows dice «Windows protegió su PC»?", "Porque el instalador todavía no tiene firma digital. Elige «Más información» y «Ejecutar de todas formas». La versión en línea no tiene ese aviso."),
    ("Encontré un error o me falta algo", f'Repórtalo en <a href="{GH}/issues">GitHub</a>. Si es un error de cálculo, incluye el archivo .iees o los datos del circuito.'),
]
faq_html = "\n".join(f"        <details><summary>{q}</summary><p>{a}</p></details>" for q, a in faq)
guia = head(p, "Documentación", "Guía de uso de IEEStudio en video y paso a paso: proyecto, tableros, circuitos, resultados, unifilar y memoria de cálculo; atajos y preguntas frecuentes.", "guia/") + cabecera(
    p, "Documentación", "Un proyecto completo, paso a paso.",
    "Mira el recorrido en video o sigue los pasos con la app abierta. Todo lo que ves aquí funciona igual en el navegador y en Windows.",
    extra=f"""
      <div class="acciones"><a class="btn btn-ambar" href="{p}app/">{icon(p, "i-play")}Abrir la app para seguirla</a><a class="btn btn-linea" href="#referencia">Referencia técnica</a></div>""",
) + f"""
  <section class="bloque" id="video">
    <div class="wrap">
      <div class="video">
        <video controls preload="none" poster="{p}video/guia.jpg" playsinline>
          <source src="{p}video/guia.mp4" type="video/mp4">
          <track kind="captions" src="{p}video/guia.vtt" srclang="es" label="Español">
        </video>
      </div>
      <p class="nota-video">Narración con voz sintética y subtítulos en español. El video se grabó con una versión anterior de la interfaz; los pasos de abajo muestran la actual.</p>
    </div>
  </section>

  <section class="bloque" id="pasos">
    <div class="wrap">
      <h2>De la acometida a la memoria.</h2>
      <div class="guia">
{steps_html}      </div>
    </div>
  </section>

  <section class="bloque" id="rapido">
    <div class="wrap">
      <div class="exhibe">
        <div class="exhibe-media"><div class="marco"><img src="{p}imagenes/guia/paso-rapido.webp" width="1600" height="1000" alt="Cálculo rápido de un circuito con sus recomendaciones." loading="lazy"></div></div>
        <div class="exhibe-texto">
          <h2>Cálculo rápido de un circuito.</h2>
          <p>Desde la pantalla de inicio, o con <kbd>F8</kbd> dentro de un proyecto, calculas un circuito suelto: sistema, carga, longitud y conductor. Si no cumple, te da alternativas ya calculadas que aplicas con un clic; su hoja se exporta en PDF, y dentro de un proyecto «Agregar al proyecto» lo pasa al tablero seleccionado.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="bloque" id="atajos">
    <div class="wrap">
      <h2>Atajos de teclado.</h2>
      <table class="atajos">
{keys_html}
      </table>
    </div>
  </section>

  <section class="bloque" id="preguntas">
    <div class="wrap">
      <h2>Preguntas frecuentes.</h2>
      <div class="faq">
{faq_html}
      </div>
    </div>
  </section>

  <section class="bloque" id="referencia">
    <div class="wrap">
      <h2>Referencia técnica.</h2>
      <p class="intro">Criterios de cálculo, fuentes y decisiones, en el repositorio.</p>
      <div class="docs">
        <div>
          <ul class="indice">
{indice(DOCS_TEC[:3])}
          </ul>
        </div>
        <div>
          <ul class="indice">
{indice(DOCS_TEC[3:])}
          </ul>
        </div>
      </div>
    </div>
  </section>
""" + foot(p)
write("guia", guia)

# ---------------------------------------------------------------- descargar
descargar = head(p, "Descargar", "Usa IEEStudio en el navegador sin instalar nada, instálalo como aplicación o descarga el instalador para Windows.", "descargar/") + cabecera(
    p, "Descargar", "En línea o en Windows, el mismo programa.",
    "Los mismos cálculos en las dos versiones. Un proyecto que empiezas en una lo terminas en la otra.",
    extra='\n      <p class="version">Consultando la versión más reciente…</p>',
) + f"""
  <section class="bloque">
    <div class="wrap">
      <div class="vias">
        <article class="via">
          <div class="via-arte" aria-hidden="true">
            <div class="ventana">
              <div class="ventana-barra"><span class="ctl"><i></i><i></i><i></i></span><span class="titulo">ieestudio.com/app</span><span></span></div>
              <img src="{p}imagenes/app/unifilar.webp" width="1440" height="900" alt="" loading="lazy">
            </div>
          </div>
          <div class="via-cuerpo">
            <h2>En el navegador</h2>
            <ul>
              <li>Sin instalar nada: abre la página y empieza a calcular.</li>
              <li>Tu proyecto se guarda solo en el navegador y lo descargas como archivo <code>.iees</code> cuando quieras.</li>
              <li>Todo se calcula en tu computadora. Ningún proyecto se sube a un servidor.</li>
              <li>Se instala como aplicación y funciona sin internet.</li>
            </ul>
            <a class="btn btn-ambar" href="{p}app/">{icon(p, "i-play")}Abrir en el navegador</a>
          </div>
        </article>
        <article class="via">
          <div class="via-arte" aria-hidden="true">
            <div class="ventana">
              <div class="ventana-barra"><span class="ctl"><i></i><i></i><i></i></span><span class="titulo"><img src="{p}app/icono.svg" alt="">IEEStudio</span><span></span></div>
              <img src="{p}imagenes/app/rapido.webp" width="1440" height="900" alt="" loading="lazy">
            </div>
          </div>
          <div class="via-cuerpo">
            <h2>En Windows</h2>
            <ul>
              <li>Instalador para Windows 10 y 11 de 64 bits.</li>
              <li>Los proyectos <code>.iees</code> abren con doble clic y se guardan directo en tus carpetas.</li>
              <li>Incluye la línea de comandos y el servidor para agentes de IA (MCP).</li>
              <li>Funciona sin internet desde el primer día.</li>
            </ul>
            <a class="btn btn-linea descargar" href="{GH}/releases">{icon(p, "i-down")}Descargar para Windows</a>
          </div>
        </article>
      </div>

      <div class="como">
        <div>
          <h3>Instalar en Windows</h3>
          <ol>
            <li><b>Descarga el instalador</b> con el botón de arriba.</li>
            <li><b>Ábrelo.</b> Si Windows muestra «Windows protegió su PC», elige «Más información» y luego «Ejecutar de todas formas». Sale porque el instalador aún no tiene firma digital.</li>
            <li><b>Sigue el asistente</b> y abre IEEStudio desde el menú Inicio.</li>
            <li><b>Si el acceso directo aparece sin ícono</b>, revisa en Seguridad de Windows, Historial de protección, que el antivirus no haya quitado el programa, y permítelo.</li>
          </ol>
        </div>
        <div>
          <h3>Instalar la app desde el navegador</h3>
          <ol>
            <li><b>Abre <a href="{p}app/">ieestudio.com/app</a></b> en Chrome o Edge y espera a que cargue la primera vez.</li>
            <li><b>Busca el ícono de instalar</b> en la barra de direcciones, o en el menú del navegador elige «Instalar IEEStudio».</li>
            <li><b>Listo:</b> queda con su propio ícono y ventana, y abre sin internet.</li>
            <li><b>En iPad o iPhone</b>, desde Safari: Compartir, «Agregar a inicio».</li>
          </ol>
        </div>
      </div>

      <table class="ficha-tecnica">
        <tr><th>Navegador</th><td>Chrome, Edge o Firefox recientes. La primera carga descarga la aplicación completa; después queda guardada.</td></tr>
        <tr><th>Windows</th><td>Windows 10 u 11 de 64 bits.</td></tr>
        <tr><th>Linux y otros</th><td>Se compila desde el <a href="{GH}">código fuente</a> con Rust.</td></tr>
        <tr><th>Versiones anteriores</th><td>Todas están en la <a href="{GH}/releases">página de versiones</a> de GitHub, con sus notas.</td></tr>
      </table>
    </div>
  </section>
""" + foot(p)
write("descargar", descargar)

# ---------------------------------------------------------------- comunidad (código abierto)
credits = [
    ("IBM Plex Mono", "IBM", "SIL Open Font License 1.1", "https://github.com/IBM/plex"),
    ("Manrope", "Mikhail Sharanda y colaboradores", "SIL Open Font License 1.1", "https://github.com/sharanda/manrope"),
    ("three.js (escena 3D de la portada)", "Autores de three.js", "MIT", "https://github.com/mrdoob/three.js"),
]
credits_html = "\n".join(f'        <tr><td>{a}</td><td>{b}</td><td>{c}</td><td><a href="{d}">Original</a></td></tr>' for a, b, c, d in credits)
codigo = head(p, "Comunidad", "Cómo se verifica IEEStudio: código y tablas públicos, pruebas, compilación en GitHub, firma de código, privacidad y cómo contribuir.", "codigo-abierto/") + cabecera(
    p, "Comunidad", "Puedes revisar cómo calcula cada número.",
    "El código, las tablas de la norma y los casos de prueba son públicos. Nadie tiene que creerle al programa: se puede comprobar.",
    extra=f"""
      <div class="acciones"><a class="btn btn-linea" href="{GH}">{icon(p, "i-git")}Ver el repositorio</a><a class="btn btn-linea" href="{GH}/issues">Reportar un problema</a></div>""",
) + f"""
  <section class="bloque">
    <div class="wrap">
      <div class="confianza">
        <div>
          {icon(p, "i-code")}
          <h3>Código y tablas públicos</h3>
          <p>El código, las tablas de la norma en CSV y los casos de prueba están <a href="{GH}">en GitHub</a>. Cada regla cita su cláusula.</p>
        </div>
        <div>
          {icon(p, "i-build")}
          <h3>Compilado a la vista</h3>
          <p>El instalador y la versión en línea se compilan en GitHub Actions desde el código público. Nadie los sube a mano.</p>
        </div>
        <div>
          {icon(p, "i-lock")}
          <h3>Tus proyectos son tuyos</h3>
          <p>El programa no envía información a ningún lado. Este sitio no usa cookies ni rastreadores; solo consulta a GitHub la versión publicada.</p>
        </div>
        <div>
          {icon(p, "i-free")}
          <h3>Gratis, sin cuentas</h3>
          <p>Licencias MIT o Apache-2.0. Úsalo en los proyectos de tu trabajo sin registrarte ni pagar.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="bloque" id="verificacion">
    <div class="wrap">
      <h2>Cómo se comprueba cada cálculo.</h2>
      <div class="dos">
        <div>
          <h3>Tres tipos de prueba por regla</h3>
          <p>Cada regla de cálculo tiene una prueba matemática (la fórmula da lo que debe), una de referencia (un caso resuelto a mano) y una normativa (aplica la tabla y el artículo correctos). Se ejecutan en cada cambio, en Windows y en Linux.</p>
        </div>
        <div>
          <h3>Casos revisables en una hoja de cálculo</h3>
          <p>Los casos de referencia están en archivos CSV que un ingeniero puede abrir en Excel, revisar contra su propio cálculo y firmar como revisados.</p>
        </div>
        <div>
          <h3>Tablas cotejadas</h3>
          <p>Las tablas de la NOM-001-SEDE-2012 se validan por consistencia interna y se cotejaron contra una captura independiente. El repositorio no contiene el texto de la norma.</p>
        </div>
        <div>
          <h3>Nunca supone un dato</h3>
          <p>Si una tabla no cubre un caso, el programa lo dice y lo marca como sin evaluar en lugar de usar un valor aproximado.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="bloque" id="contribuir">
    <div class="wrap">
      <h2>Participa en el desarrollo.</h2>
      <div class="comunidad">
        <div class="panel repo">
          <h3>{icon(p, "i-git")}Repositorio</h3>
          <a class="url" href="{GH}">github.com/JAAM08/IEEStudio</a>
          <ul class="contribuir">
            <li><a href="{GH}/issues/new?title=Error%3A%20">{icon(p, "i-fail")}<span><b>Reportar un error</b><span>Si es de cálculo, adjunta el .iees o los datos del circuito.</span></span></a></li>
            <li><a href="{GH}/issues/new?title=Propuesta%3A%20">{icon(p, "i-steps")}<span><b>Proponer una función</b><span>Qué necesitas calcular y con qué referencia.</span></span></a></li>
            <li><a href="{GH}/blob/main/CONTRIBUTING.md">{icon(p, "i-code")}<span><b>Contribuir</b><span>Cómo preparar un cambio y sus pruebas.</span></span></a></li>
          </ul>
        </div>
        <div class="panel">
          <h3>{icon(p, "i-down")}Versiones publicadas</h3>
          <ul class="lista-vivo" id="versiones"><li class="cargando">Consultando GitHub… <a href="{GH}/releases">Ver versiones</a></li></ul>
        </div>
        <div class="panel">
          <h3>{icon(p, "i-build")}Cambios recientes</h3>
          <ul class="lista-vivo" id="cambios"><li class="cargando">Consultando GitHub… <a href="{GH}/commits/main">Ver historial</a></li></ul>
        </div>
      </div>
    </div>
  </section>

  <section class="bloque" id="firma">
    <div class="wrap">
      <div class="firma">
        <div>
          <h3>Política de firma de código</h3>
          <p class="en">Free code signing provided by <a href="https://about.signpath.io">SignPath.io</a>, certificate by <a href="https://signpath.org">SignPath Foundation</a>.</p>
          <p class="estado-firma">Solicitud pendiente. Mientras tanto el instalador se publica sin firma y Windows puede mostrar «Windows protegió su PC». La versión en línea no lo necesita.</p>
        </div>
        <ul>
          <li>Los instaladores se compilan en GitHub Actions desde el código público del repositorio; ningún binario se compila ni se sube a mano.</li>
          <li>Autor y responsable de aprobar cada versión firmada: <a href="https://github.com/JAAM08">Juan Antonio Aguilar Martínez</a>.</li>
          <li>Privacidad: el programa no transfiere información a otros sistemas en red, salvo que el usuario lo pida expresamente.</li>
        </ul>
      </div>
    </div>
  </section>

  <section class="bloque" id="creditos">
    <div class="wrap">
      <h2>Créditos.</h2>
      <p class="intro">Las capturas son de IEEStudio y las hojas, de la memoria que genera con su proyecto de ejemplo. El modelo 3D de la portada y los dibujos técnicos son propios del proyecto. Este sitio no usa fotografías de stock.</p>
      <table class="creditos">
        <tr><th>Recurso</th><th>Autor</th><th>Licencia</th><th></th></tr>
{credits_html}
      </table>
    </div>
  </section>
""" + foot(p)
write("codigo-abierto", codigo)
print("ok")

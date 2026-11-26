#!/usr/bin/env python3
"""Revisa el sitio de website/ antes de publicarlo.

    python3 tools/verificar-sitio.py [--cli target/release/ieestudio]

Comprueba que:
- las páginas coinciden con lo que genera tools/paginas.py;
- todo enlace, imagen, ancla e ícono local existe;
- todo archivo de website/ se usa y no hay fotografías de stock;
- cada página tiene título, descripción, canonical y metadatos para compartir,
  y el sitemap, robots.txt y los datos estructurados son coherentes;
- con --cli (o si encuentra target/release/ieestudio o target/debug/ieestudio):
  las posiciones de los interruptores y los valores del medidor de la escena
  3D y del alzado coinciden con lo que calcula el programa para el ejemplo.

Sale con error si algo falla. No modifica website/.
"""
import html
import json
import os
import re
import subprocess
import sys
import tempfile
from urllib.parse import urljoin, urlparse

RAIZ = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
WEB = os.path.join(RAIZ, "website")
SITIO = "https://ieestudio.com/"
PAGINAS = ["index.html", "funciones/index.html", "guia/index.html", "descargar/index.html", "codigo-abierto/index.html"]
# Archivos que no se enlazan desde las páginas y aun así deben estar.
SIN_ENLACE = {"imagenes/compartir.jpg", "README.md", ".htaccess", "robots.txt", "sitemap.xml", "fuentes/OFL-IBM-Plex.txt", "fuentes/OFL-Manrope.txt"}
STOCK = re.compile(r"fotos/|rawpixel|flickr|unsplash|pexels|shutterstock|istock|gettyimages|pixabay", re.I)

errores = []


def error(msg):
    errores.append(msg)


def leer(ruta, codificacion="utf-8"):
    with open(ruta, encoding=codificacion) as f:
        return f.read()


# ------------------------------------------------------------------ generador
with tempfile.TemporaryDirectory() as tmp:
    subprocess.run([sys.executable, os.path.join(RAIZ, "tools", "paginas.py"), tmp], check=True, stdout=subprocess.DEVNULL)
    for p in PAGINAS:
        if leer(os.path.join(tmp, p)) != leer(os.path.join(WEB, p)):
            error(f"{p} no coincide con tools/paginas.py: vuelve a generar las páginas")

# ------------------------------------------------------------------ enlaces y recursos
sprite = set(re.findall(r'id="([^"]+)"', leer(os.path.join(WEB, "iconos.svg"))))
ids = {p: re.findall(r'\sid="([^"]+)"', leer(os.path.join(WEB, p))) for p in PAGINAS}
usados = set()
for p in PAGINAS:
    texto = leer(os.path.join(WEB, p))
    for i in set(ids[p]):
        if ids[p].count(i) > 1:
            error(f"{p}: id repetido «{i}»")
    if STOCK.search(texto):
        error(f"{p}: referencia a fotografías de stock")
    for atributo, valor in re.findall(r'\s(href|src|srcset|data-grande|poster)="([^"]+)"', texto):
        partes = valor.split(",") if atributo == "srcset" else [valor]
        for parte in partes:
            url = html.unescape(parte.strip().split(" ")[0])
            if url.startswith(("http://", "https://", "mailto:")) or url.startswith("#") and not url[1:]:
                continue
            completo = urlparse(urljoin("/" + p, url))
            ruta = completo.path.lstrip("/")
            if ruta.startswith("app/"):
                continue  # la aplicación la agrega tools/sitio-web.sh
            if ruta == "" or ruta.endswith("/"):
                ruta += "index.html"
            if not os.path.isfile(os.path.join(WEB, ruta)):
                error(f"{p}: no existe {url}")
                continue
            usados.add(ruta)
            ancla = completo.fragment
            if ancla:
                if ruta == "iconos.svg":
                    if ancla not in sprite:
                        error(f"{p}: el ícono #{ancla} no está en iconos.svg")
                elif ruta in ids and ancla not in ids[ruta]:
                    error(f"{p}: el ancla {url} no existe")

# Lo que cargan las hojas de estilo y los scripts.
for extra in ("estilos.css", "sitio.js", "portada.js"):
    texto = leer(os.path.join(WEB, extra))
    for url in re.findall(r'url\("([^"]+)"\)|import\("\./([^"]+)"\)', texto):
        url = next(u for u in url if u)
        if not os.path.isfile(os.path.join(WEB, url)):
            error(f"{extra}: no existe {url}")
        usados.add(url)
    if STOCK.search(texto):
        error(f"{extra}: referencia a fotografías de stock")
# portada.js arma las rutas de las capturas de teléfono con data-movil.
for archivo in os.listdir(os.path.join(WEB, "imagenes", "app")):
    if archivo.startswith("movil-") and f'data-movil="{archivo[6:-5]}"' in leer(os.path.join(WEB, "index.html")):
        usados.add(f"imagenes/app/{archivo}")

for base, _, archivos in os.walk(WEB):
    for a in archivos:
        rel = os.path.relpath(os.path.join(base, a), WEB).replace(os.sep, "/")
        if rel in SIN_ENLACE or rel in usados:
            continue
        error(f"website/{rel} no se usa en ninguna página")

# ------------------------------------------------------------------ SEO
mapa = set(re.findall(r"<loc>([^<]+)</loc>", leer(os.path.join(WEB, "sitemap.xml"))))
if f"Sitemap: {SITIO}sitemap.xml" not in leer(os.path.join(WEB, "robots.txt")):
    error("robots.txt no declara el sitemap")
for p in PAGINAS:
    texto = leer(os.path.join(WEB, p))
    esperado = SITIO + p.replace("index.html", "")
    reglas = {
        "idioma": r'<html lang="es',
        "título": r"<title>[^<]{10,70}</title>",
        "descripción": r'<meta name="description" content="[^"]{60,320}">',
        "canonical": rf'<link rel="canonical" href="{re.escape(esperado)}">',
        "og:url": rf'<meta property="og:url" content="{re.escape(esperado)}">',
        "og:title": r'<meta property="og:title" content="[^"]+">',
        "og:description": r'<meta property="og:description" content="[^"]+">',
        "og:image": rf'<meta property="og:image" content="{re.escape(SITIO)}imagenes/compartir\.jpg">',
        "viewport": r'<meta name="viewport"',
        "h1 único": None,
    }
    for nombre, patron in reglas.items():
        if patron and not re.search(patron, texto):
            error(f"{p}: falta o está mal {nombre}")
    if len(re.findall(r"<h1[\s>]", texto)) != 1:
        error(f"{p}: debe tener un solo h1")
    if esperado not in mapa:
        error(f"sitemap.xml no incluye {esperado}")
    for bloque in re.findall(r'<script type="application/ld\+json">(.*?)</script>', texto, re.S):
        try:
            datos = json.loads(bloque)
            if datos.get("url") != SITIO or datos.get("@type") != "SoftwareApplication":
                error(f"{p}: datos estructurados incompletos")
        except ValueError as e:
            error(f"{p}: los datos estructurados no son JSON válido ({e})")
for loc in mapa:
    ruta = loc.replace(SITIO, "")
    if not ruta.startswith("app/") and not os.path.isfile(os.path.join(WEB, ruta, "index.html") if ruta else os.path.join(WEB, "index.html")):
        error(f"sitemap.xml apunta a {loc}, que no existe")

# ------------------------------------------------------------------ escena contra el programa
cli = None
if "--cli" in sys.argv:
    cli = sys.argv[sys.argv.index("--cli") + 1]
else:
    for candidato in ("target/release/ieestudio", "target/debug/ieestudio"):
        if os.path.isfile(os.path.join(RAIZ, candidato)):
            cli = os.path.join(RAIZ, candidato)
            break
if cli:
    with tempfile.TemporaryDirectory() as tmp:
        iees = os.path.join(tmp, "casa.iees")
        subprocess.run([cli, "ejemplo", iees], check=True, stdout=subprocess.DEVNULL)
        subprocess.run([cli, "dxf", iees, os.path.join(tmp, "casa.dxf")], check=True, stdout=subprocess.DEVNULL)
        datos = json.loads(subprocess.run([cli, "calcular", iees, "--json"], check=True, capture_output=True, text=True).stdout)
        lineas = leer(os.path.join(tmp, "casa.dxf"), "cp1252").splitlines()  # DXF R12
    # Pares código-valor del DXF; los textos son el código 1.
    textos = [lineas[i + 1].strip() for i in range(0, len(lineas) - 1, 2) if lineas[i].strip() == "1"]
    # En el unifilar, cada circuito lleva arriba su posición, su interruptor y
    # su fase, y luego «C-01  L = 18 m».
    programa = {}
    for i, t in enumerate(textos):
        m = re.match(r"(C-\d+)\W.*L = ", t)
        if m:
            for j in range(i - 1, max(i - 7, -1), -1):
                if re.fullmatch(r"\d+(,\d+)*", textos[j]):
                    programa[m.group(1)] = [int(n) for n in textos[j].split(",")]
                    break
    escena = leer(os.path.join(RAIZ, "tools", "escena", "escena.js"))
    en_escena = {c: [int(n) for n in pos.split(",")] for c, pos in re.findall(r'\["(C-\d+)", "[^"]+", \d, \d+, "[A-C]+", \[([\d, ]+)\]\]', escena)}
    paginas_py = leer(os.path.join(RAIZ, "tools", "paginas.py"))
    en_alzado = {c: [int(n) for n in pos.split(",")] for c, pos in re.findall(r'\("(C-\d+)", \d, \[([\d, ]+)\]\)', paginas_py)}
    if not programa:
        error("no se encontraron las posiciones en el DXF del ejemplo")
    if en_escena != programa:
        error(f"posiciones de la escena {en_escena} distintas de las del programa {programa}")
    if en_alzado != programa:
        error(f"posiciones del alzado {en_alzado} distintas de las del programa {programa}")
    run = datos["panels"][0]["feeder"]["run"]
    corriente, caida = f"{run['nominal_a']:.2f}", f"{run['drop_percent']:.2f}"
    m = re.search(r'ALIMENTADOR = \{ corriente: "([\d.]+)", caida: "([\d.]+)" \}', escena)
    if not m or m.groups() != (corriente, caida):
        error(f"el medidor dice {m.groups() if m else '?'}, el programa calcula In {corriente} A y %e {caida}")
    for pagina in ("index.html",):
        if f"In {corriente} A y %e {caida}" not in leer(os.path.join(WEB, pagina)):
            error(f"{pagina}: el texto del medidor no cita In {corriente} A y %e {caida}")
    print(f"Escena contra el programa: posiciones {programa}, In {corriente} A, %e {caida}")
else:
    print("Sin la línea de comandos compilada: se omite la comparación de la escena con el programa")

if errores:
    print("\n".join("ERROR: " + e for e in errores))
    sys.exit(1)
print(f"Sitio en orden: {len(PAGINAS)} páginas, {len(usados)} recursos usados.")

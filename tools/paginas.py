# Genera las páginas de website/ con la misma navegación y el mismo pie.
import os, sys

ROOT = sys.argv[1]
GH = "https://github.com/JAAM08/IEEStudio"

PAGES = [
    ("funciones/", "Funciones"),
    ("guia/", "Guía"),
    ("descargar/", "Descargar"),
    ("codigo-abierto/", "Código abierto"),
]


def icon(p, name, cls=""):
    c = f' class="{cls}"' if cls else ""
    return f'<svg{c} aria-hidden="true"><use href="{p}iconos.svg#{name}"/></svg>'


def head(p, title, desc, current):
    full = "IEEStudio" if not title else f"{title} · IEEStudio"
    links = "\n".join(
        f'      <a href="{p}{href}"{" aria-current=\"page\"" if href == current else ""}>{name}</a>'
        for href, name in PAGES
    )
    return f"""<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{full}</title>
<meta name="description" content="{desc}">
<meta name="theme-color" content="#121418">
<link rel="icon" href="{p}app/icono.svg" type="image/svg+xml">
<link rel="icon" href="{p}favicon.ico" sizes="any">
<link rel="apple-touch-icon" href="{p}app/icono-180.png">
<meta property="og:title" content="{full}">
<meta property="og:description" content="{desc}">
<meta property="og:image" content="https://ieestudio.com/imagenes/compartir.jpg">
<meta property="og:url" content="https://ieestudio.com/{current}">
<link rel="preload" href="{p}fuentes/IBMPlexSansCondensed-Bold.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="{p}estilos.css">
</head>
<body>

<header class="nav" id="nav">
  <div class="wrap">
    <a class="brand" href="{p or './'}"><img src="{p}app/icono.svg" alt="" width="30" height="30">IEEStudio</a>
    <nav class="nav-links" aria-label="Páginas">
{links}
      <a class="nav-git" href="{GH}">{icon(p, "i-git")}<span>GitHub</span></a>
      <a class="btn btn-volt menu-app" href="{p}app/">Abrir la app</a>
    </nav>
    <button class="menu-toggle" type="button" aria-label="Menú" aria-expanded="false">{icon(p, "i-menu")}</button>
    <a class="btn btn-volt btn-small" href="{p}app/">Abrir la app</a>
  </div>
</header>

<main>
"""


def foot(p):
    return f"""</main>

<footer>
  <div class="wrap">
    <a class="brand" href="{p or './'}"><img src="{p}app/icono.svg" alt="" width="26" height="26">IEEStudio</a>
    <nav aria-label="Enlaces">
      <a href="{p}app/">Abrir la app</a>
      <a href="{p}funciones/">Funciones</a>
      <a href="{p}guia/">Guía</a>
      <a href="{p}descargar/">Descargar</a>
      <a href="{p}codigo-abierto/">Código abierto</a>
      <a href="{GH}/issues">Reportar un problema</a>
    </nav>
    <p>Software libre bajo licencias MIT o Apache-2.0. IEEStudio no está afiliado a la Secretaría de Energía ni a ningún organismo de normalización; la NOM-001-SEDE-2012 se cita por artículo y su texto no se reproduce. Fotografías de dominio público; ver <a href="{p}codigo-abierto/#creditos">créditos</a>.</p>
  </div>
</footer>

<script src="{p}sitio.js"></script>
</body>
</html>
"""


def proc_card(p, full):
    if full:
        steps = """          <li>
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
            <p class="s">e total = 0.68 + 4.37 = <span class="bad">5.05 %</span></p>
            <p class="n">5.05 % es mayor que el límite de 5.00 %.</p>
            <p class="ref">NOM-001-SEDE-2012: 210-19(a)(1), nota 4; 215-2(a)(1), nota 2</p>
          </li>"""
        head_ = "1,500 W, 127 V, 35 m, 12 AWG"
        fix = "Con <b>10 AWG</b> cumple. Se aplica con un clic."
        cls = "proc"
    else:
        steps = """          <li>
            <h3>Caída de tensión</h3>
            <p class="s">ΔV = 2 · 13.12 · 0.0350 · (6.6 · 0.90 + 0.223 · 0.436) = 5.55 V</p>
            <p class="ref">NOM-001-SEDE-2012: Capítulo 10, Tabla 9</p>
          </li>
          <li>
            <h3>Caída acumulada</h3>
            <p class="s">0.68 + 4.37 = <span class="bad">5.05 %</span>, límite 5.00 %</p>
          </li>"""
        head_ = "1,500 W, 35 m, 12 AWG"
        fix = "Con <b>10 AWG</b> cumple"
        cls = "float float-proc proc"
    return f"""<figure class="{cls}" aria-label="Procedimiento del circuito C-04">
        <div class="proc-head"><b>C-04 Contactos sala</b><span>{head_}</span></div>
        <ol>
{steps}
        </ol>
        <div class="proc-foot">
          <span class="verdict">{icon(p, "i-fail", "icon")}No cumple</span>
          <span class="fix">{fix}</span>
        </div>
      </figure>"""


def dl_buttons(p, line=True):
    second = "btn btn-line descargar" if line else "btn btn-volt descargar"
    return f"""<a class="btn btn-volt" href="{p}app/">{icon(p, "i-play")}Abrir en el navegador</a>
          <a class="{second}" href="{GH}/releases">{icon(p, "i-down")}Descargar para Windows</a>"""


def page_head(p, kicker, title, intro, photo=None, extra=""):
    ph = ""
    cls = "page-head"
    if photo:
        src, alt = photo
        cls += " has-photo"
        ph = f'\n  <div class="photo"><img src="{p}imagenes/fotos/{src}" alt="{alt}" width="1600" height="1000"></div>'
    return f"""  <section class="{cls}">{ph}
    <div class="wrap">
      <p class="kicker">{kicker}</p>
      <h1>{title}</h1>
      <p class="intro">{intro}</p>{extra}
    </div>
  </section>
"""


def write(rel, html):
    path = os.path.join(ROOT, rel, "index.html") if rel else os.path.join(ROOT, "index.html")
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open(path, "w").write(html)


# ------------------------------------------------------------------ inicio
p = ""
inicio = head(p, "", "Programa gratuito y de código abierto para calcular instalaciones eléctricas de baja tensión conforme a la NOM-001-SEDE-2012. Úsalo en el navegador o instálalo en Windows.", "") + f"""  <section class="hero">
    <div class="wrap hero-copy">
      <h1><span>Calcula la instalación.</span><span class="second">Entrega la memoria.</span></h1>
      <div class="hero-side">
        <p class="lead">IEEStudio calcula instalaciones eléctricas de baja tensión conforme a la NOM-001-SEDE-2012 y arma la memoria de cálculo con cada paso a la vista. Gratis y de código abierto.</p>
        <div class="actions">
          {dl_buttons(p)}
        </div>
        <p class="release version">Versión de prueba: revisa los resultados antes de usarlos en un proyecto.</p>
      </div>
    </div>

    <div class="stage">
      <div class="glow" aria-hidden="true"></div>
      <div class="laptop">
        <div class="screen">
          <img src="imagenes/app-calculo.webp" width="1440" height="900" alt="IEEStudio con el cuadro de cargas de una casa habitación: cada circuito con su corriente, calibre, interruptor, caída de tensión y estado." fetchpriority="high">
        </div>
        <div class="base" aria-hidden="true"></div>
      </div>
      {proc_card(p, False)}
      <img class="float float-paper" src="imagenes/memoria-1.webp" width="1020" height="1320" alt="Primera hoja de la memoria de cálculo en PDF." loading="lazy">
    </div>
  </section>

  <section class="block">
    <div class="wrap who">
      <div class="who-photo"><img src="imagenes/fotos/tablero-electricista.webp" width="1024" height="681" alt="Electricista con casco revisando un tablero de distribución." loading="lazy"></div>
      <div>
        <p class="kicker">Para quién es</p>
        <h2>Para quien diseña, calcula y firma la instalación</h2>
        <p class="intro">Ingenieros, contratistas y residentes de obra que preparan la memoria de cálculo de una instalación residencial, comercial o industrial en baja tensión.</p>
        <ul>
          <li>{icon(p, "i-check")}<span><b>Un solo modelo del proyecto</b>El cuadro de cargas, el unifilar y la memoria salen de los mismos datos y se actualizan juntos.</span></li>
          <li>{icon(p, "i-check")}<span><b>La norma, artículo por artículo</b>Cada resultado cita la tabla o el artículo de la NOM-001-SEDE-2012 que lo respalda.</span></li>
          <li>{icon(p, "i-check")}<span><b>Sin licencias ni cuentas</b>Gratis para uso profesional, en el navegador o en Windows.</span></li>
        </ul>
      </div>
    </div>
  </section>

  <section class="block">
    <div class="wrap">
      <p class="kicker">Conoce IEEStudio</p>
      <h2>Empieza por donde quieras</h2>
      <div class="tiles">
        <a class="tile" href="funciones/">
          <div class="art"><img src="imagenes/app-unifilar.webp" width="1440" height="900" alt="" loading="lazy"></div>
          <div class="body"><h3>Funciones</h3><p>Qué calcula, cómo muestra el procedimiento y qué entrega: cuadro de cargas, unifilar y memoria.</p><span class="go">Ver funciones</span></div>
        </a>
        <a class="tile" href="guia/">
          <div class="art"><img src="video/guia.jpg" width="1600" height="900" alt="" loading="lazy"><span class="play"><span>{icon(p, "i-triangle")}</span></span></div>
          <div class="body"><h3>Guía en video</h3><p>Un recorrido de pocos minutos por un proyecto completo, y la guía paso a paso.</p><span class="go">Ver la guía</span></div>
        </a>
        <a class="tile" href="descargar/">
          <div class="art"><img src="imagenes/app-rapido.webp" width="1440" height="900" alt="" loading="lazy"></div>
          <div class="body"><h3>En línea o en Windows</h3><p>Úsalo en el navegador sin instalar nada o descarga el instalador para Windows.</p><span class="go">Ver opciones</span></div>
        </a>
      </div>
    </div>
  </section>

  <section class="block" style="padding-top:0">
    <div class="wrap">
      <div class="photo-band">
        <img src="imagenes/fotos/clemas.webp" width="1024" height="576" alt="Tablillas de conexión con conductores en un tablero de control." loading="lazy">
        <div class="content">
          <p class="kicker">Código abierto</p>
          <h2>Puedes revisar cómo calcula cada número</h2>
          <p>El código, las tablas de la norma y los casos de prueba son públicos. El instalador y la versión en línea se compilan a la vista en GitHub.</p>
          <div class="actions"><a class="btn btn-line" href="codigo-abierto/">Cómo se verifica</a></div>
        </div>
      </div>
    </div>
  </section>

  <section class="closing">
    <div class="wrap">
      <h2>Pruébalo ahora,<br>sin instalar nada.</h2>
      <p>IEEStudio abre con un proyecto de ejemplo ya cargado: una casa habitación con su cuadro de cargas, su unifilar y su memoria.</p>
      <div class="actions">
        {dl_buttons(p)}
      </div>
    </div>
  </section>
""" + foot(p)
write("", inicio)

# ------------------------------------------------------------------ funciones
p = "../"
calc = [
    ("Conductores", "Menor calibre por terminales, temperatura, agrupamiento, protección y caída de tensión. Conductores en paralelo, un juego por tubo, cuando hacen falta."),
    ("Protecciones", "Interruptor normalizado inmediato superior a la corriente de diseño, o el que tú fijes, verificado contra el conductor."),
    ("Caída de tensión", "Por impedancia con la Tabla 9, por tramo y acumulada desde la fuente, con la longitud máxima permitida."),
    ("Carga del alimentador", "Artículo 220: alumbrado por área según el tipo de inmueble, contactos, aparatos fijos y cocina, con sus factores de demanda."),
    ("Cortocircuito", "Corriente de falla en cada tablero por el método de impedancias, con aportación de motores, y capacidad interruptiva de cada interruptor."),
    ("Motores y clima", "Corrientes de las tablas 430-248 y 430-250, conductor al 125 %, interruptor, relevador de sobrecarga y el motor mayor en el alimentador."),
    ("Transformadores", "Hasta 600 V: primario y secundario, protección de la Tabla 450-3(b), tensión propia del tablero y falla a través del transformador."),
    ("Puesta a tierra", "Conductor de puesta a tierra de equipos (Tabla 250-122) y conductor al electrodo de la acometida (Tabla 250-66)."),
    ("Tubería conduit", "EMT, IMC, RMC y PVC: ocupación de 53, 31 y 40 %, niples al 60 % y verificación de atascamiento."),
    ("Balanceo de fases", "Reparte los circuitos de un tablero entre las fases para dejarlo lo más balanceado posible, y se puede deshacer."),
    ("Cálculo rápido", "Un circuito sin armar proyecto, con su hoja en PDF y alternativas ya calculadas si no cumple."),
    ("Excel y DXF", "El cuadro de cargas va y viene de Excel; el unifilar se exporta a DXF para AutoCAD."),
]
calc_html = "\n".join(f"        <div><dt>{a}</dt><dd>{b}</dd></div>" for a, b in calc)
funciones = head(p, "Funciones", "Qué calcula IEEStudio, cómo muestra cada paso con su artículo de la NOM-001-SEDE-2012 y qué entrega: cuadro de cargas, diagrama unifilar y memoria de cálculo.", "funciones/") + page_head(
    p, "Funciones", "Nada sale de una caja negra",
    "Cada resultado trae su fórmula, los valores sustituidos y el artículo de la norma. Si algo no cumple te dice por qué y cómo corregirlo; si falta un dato, no lo inventa.",
    ("interruptores.webp", "Interruptores termomagnéticos en un tablero."),
) + f"""
  <section class="block">
    <div class="wrap">
      <p class="kicker">Cada paso a la vista</p>
      <h2>El procedimiento de cada circuito</h2>
      <div class="split">
      {proc_card(p, True)}
        <div class="points">
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

      <div class="verdicts">
        <div class="v ok">
          <span class="name">{icon(p, "i-ok", "icon")}Cumple</span>
          <p>Se calculó con todos sus datos y pasó cada verificación.</p>
          <p class="eg">C-01: caída de <span class="mono">2.76 %</span> de <span class="mono">3.00 %</span> permitido.</p>
        </div>
        <div class="v fail">
          <span class="name">{icon(p, "i-fail", "icon")}No cumple</span>
          <p>Al menos una verificación falla. Dice cuál y cómo corregirla.</p>
          <p class="eg">C-04: caída acumulada de <span class="mono">5.05 %</span> de <span class="mono">5.00 %</span>.</p>
        </div>
        <div class="v pending">
          <span class="name">{icon(p, "i-pending", "icon")}No evaluado</span>
          <p>Falta un dato o la norma no cubre el caso. Nunca supone un valor.</p>
          <p class="eg">C-06: falta la corriente de placa del motor para el relevador de sobrecarga.</p>
        </div>
      </div>
      <p class="approval"><b>La aprobación es tuya.</b> IEEStudio verifica contra la norma, pero nunca marca un proyecto como aprobado: esa firma la da el ingeniero responsable.</p>
    </div>
  </section>

  <section class="block" id="entregables">
    <div class="wrap">
      <p class="kicker">Lo que entrega</p>
      <h2>Del cuadro de cargas a la memoria</h2>
      <div class="show section-gap" style="margin-top:56px">
        <div class="show-media">
          <div class="frame"><img src="{p}imagenes/app-calculo.webp" width="1440" height="900" alt="Cuadro de cargas con el estado de cada circuito." loading="lazy"></div>
        </div>
        <div class="show-text">
          <h3>Cuadro de cargas que se calcula solo</h3>
          <p>Cada circuito con su carga, corriente, calibre, tierra, tubería, interruptor y caída de tensión, y el alimentador con la carga del artículo 220. Filtra lo que no cumple con un clic.</p>
          <ul>
            <li>Carga por fase y desbalance del tablero.</li>
            <li>Caída acumulada desde la fuente.</li>
          </ul>
        </div>
      </div>
      <div class="show flip">
        <div class="show-media">
          <div class="frame"><img src="{p}imagenes/app-unifilar.webp" width="1440" height="900" alt="Diagrama unifilar dibujado por IEEStudio." loading="lazy"></div>
        </div>
        <div class="show-text">
          <h3>El unifilar se dibuja solo</h3>
          <p>Acometida, alimentadores, tableros y circuitos con su interruptor, conductores, tubería, caída de tensión y cortocircuito. Cambia un dato y el diagrama se actualiza.</p>
          <ul>
            <li>Se exporta a DXF para seguir dibujando en AutoCAD.</li>
            <li>Va incluido en la memoria de cálculo.</li>
          </ul>
        </div>
      </div>
      <div class="show">
        <div class="show-media">
          <div class="papers">
            <img src="{p}imagenes/memoria-1.webp" width="1020" height="1320" alt="Primera hoja de la memoria: datos de la obra, criterios de diseño y resumen." loading="lazy">
            <img src="{p}imagenes/memoria-2.webp" width="1020" height="1320" alt="Hoja de la memoria con el cuadro de cargas." loading="lazy">
          </div>
        </div>
        <div class="show-text">
          <h3>La memoria, lista para revisar y firmar</h3>
          <p>Un PDF con los datos de la obra, los criterios de diseño, el cuadro de cargas, el diagrama unifilar, las observaciones y el procedimiento de cada tramo.</p>
          <ul>
            <li>Responsable, cédula y revisión en cada hoja.</li>
            <li>Espacio para firmas al final.</li>
          </ul>
        </div>
      </div>
      <div class="show flip">
        <div class="show-media">
          <div class="frame"><img src="{p}imagenes/app-rapido.webp" width="1440" height="900" alt="Cálculo rápido de un circuito." loading="lazy"></div>
        </div>
        <div class="show-text">
          <h3>Un circuito, sin armar proyecto</h3>
          <p>El cálculo rápido te da corriente, interruptor, conductor, tierra, tubería y caída de tensión de un solo circuito, con su hoja en PDF. Si te sirve, lo agregas al proyecto con un clic.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="block" id="calculos">
    <div class="wrap">
      <p class="kicker">Qué calcula</p>
      <h2>Lo que lleva una instalación de baja tensión</h2>
      <dl class="calc">
{calc_html}
      </dl>
    </div>
  </section>

  <section class="block" style="padding-top:0">
    <div class="wrap">
      <div class="photo-band">
        <img src="{p}imagenes/fotos/subestacion.webp" width="1024" height="768" alt="Transformador de potencia en una subestación." loading="lazy">
        <div class="content">
          <p class="kicker">NOM-001-SEDE-2012</p>
          <h2>Las tablas de la norma, en datos abiertos</h2>
          <p>Las tablas que usa el programa están en archivos que cualquiera puede revisar, cotejadas contra una captura independiente. El código no tiene valores de la norma escritos a mano.</p>
          <div class="actions"><a class="btn btn-line" href="{p}codigo-abierto/">Cómo se verifica</a></div>
        </div>
      </div>
    </div>
  </section>

  <section class="closing">
    <div class="wrap">
      <h2>Míralo con un<br>proyecto real.</h2>
      <p>Abre la app con el proyecto de ejemplo o empieza el tuyo desde cero.</p>
      <div class="actions">
        {dl_buttons(p)}
      </div>
    </div>
  </section>
""" + foot(p)
write("funciones", funciones)

# ------------------------------------------------------------------ guía
steps = [
    ("paso-1", "Abre IEEStudio", "La app abre con un proyecto de ejemplo: una casa habitación con un tablero y seis circuitos. A la izquierda está el navegador del proyecto; haz clic en cualquier elemento y las demás vistas lo siguen.",
     ["Para empezar el tuyo: Archivo, Nuevo proyecto.", "El ejemplo siempre está en Archivo, Abrir el proyecto de ejemplo."]),
    ("paso-2", "Datos de la obra y la acometida", "Selecciona el proyecto para capturar el cliente, la ubicación, quién elaboró y quién firma, el tipo de inmueble, el sistema de la acometida y los datos de cortocircuito de la compañía o del transformador.",
     ["Los datos de la obra van a la portada de la memoria; no cambian el cálculo.", "Los límites de caída de tensión se ajustan aquí."]),
    ("paso-3", "Tableros y carga del alimentador", "En cada tablero eliges de dónde se alimenta, su conexión, la longitud del alimentador y las áreas que atiende. Con eso se calcula la carga según el artículo 220, con el detalle de cada grupo en el procedimiento.",
     ["Si el tablero se alimenta por un transformador, márcalo y captura sus datos.", "Las áreas definen la carga mínima de alumbrado (Tabla 220-12)."]),
    ("paso-4", "Circuitos en el cuadro de cargas", "Agrega circuitos con «+ Circuito» y captura su tipo de carga, potencia, fases y longitud. El cuadro de cargas se recalcula con cada cambio: corriente, calibre, tierra, tubo, interruptor, caída de tensión y estado.",
     ["Ctrl+D duplica el circuito seleccionado y Supr lo elimina.", "«Balancear fases» reparte los circuitos entre A, B y C."]),
    ("paso-5", "Revisa los resultados", "En Propiedades, la pestaña Resultados muestra lo que calculó el programa para el circuito seleccionado. Si fijaste un calibre o un interruptor, te dice qué habría propuesto y por qué.",
     ["Puedes fijar calibre, interruptor y conductores por fase, o dejar que IEEStudio proponga el menor que cumple."]),
    ("paso-6", "Revisa la normativa", "La pestaña Normativa lista cada verificación —ampacidad, protección, caída, tubería, capacidad interruptiva— con su estado y el artículo que aplica.",
     ["Abajo, el Procedimiento muestra la fórmula y los valores de cada paso."]),
    ("paso-7", "El diagrama unifilar", "El espacio de trabajo Diagrama muestra el unifilar dibujado a partir del proyecto. No se dibuja a mano: cualquier cambio en el cuadro de cargas se refleja aquí.",
     ["Archivo, Exportar unifilar a DXF, para seguir en AutoCAD."]),
    ("paso-8", "La memoria de cálculo", "El espacio Documentación muestra la vista previa de la memoria con los datos de la obra, el cuadro de cargas, el unifilar, las observaciones y el procedimiento. Expórtala a PDF con Ctrl+E.",
     ["Las observaciones listan todo lo que no cumple o no se evaluó."]),
    ("paso-9", "Guarda y exporta", "En Windows, Guardar escribe un archivo .iees en tu carpeta. En el navegador, el proyecto se guarda solo en ese navegador y «Descargar proyecto» te da el .iees para tener tu copia.",
     ["El cuadro de cargas se exporta a Excel y se importa de vuelta.", "Un .iees abre igual en la app en línea y en la de Windows."]),
]
steps_html = ""
for k, (img, title, text, bullets) in enumerate(steps, 1):
    lis = "".join(f"<li>{b}</li>" for b in bullets)
    steps_html += f"""        <div class="guide-step" id="{img}">
          <div>
            <span class="num">{k}</span>
            <h3>{title}</h3>
            <p>{text}</p>
            <ul>{lis}</ul>
          </div>
          <figure><img src="{p}imagenes/guia/{img}.webp" width="1600" height="1000" alt="{title} en IEEStudio." loading="lazy"></figure>
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
    ("<kbd>F2</kbd> a <kbd>F8</kbd>", "Mostrar u ocultar paneles: Proyecto, Cuadro de cargas, Unifilar, Memoria, Procedimiento, Propiedades y Cálculo rápido. En el navegador, F5 recarga la página: usa el menú Ventana."),
]
keys_html = "\n".join(f"        <tr><td>{a}</td><td>{b}</td></tr>" for a, b in keys)
faq = [
    ("¿Cuesta algo?", "No. IEEStudio es gratuito y de código abierto, también para uso profesional. No pide cuenta ni registro."),
    ("¿Mis proyectos se suben a internet?", "No. En la versión en línea todo se calcula en tu navegador y el proyecto se guarda en ese mismo navegador. En Windows se guarda en tus carpetas. Ningún proyecto pasa por un servidor."),
    ("¿Si borro los datos del navegador pierdo el proyecto?", "El proyecto que tenías abierto, sí. Por eso conviene descargar el .iees de vez en cuando: es tu copia y la puedes abrir en cualquier computadora."),
    ("¿La memoria que genera ya está aprobada?", "No. IEEStudio verifica contra la norma y lo dice claramente, pero la aprobación la da el ingeniero responsable que revisa y firma. El programa nunca marca un proyecto como aprobado."),
    ("¿Qué hago si algo dice «no evaluado»?", "Lee el motivo en la pestaña Normativa o en las observaciones de la memoria: casi siempre falta un dato, como la corriente de placa de un motor. Captúralo y se evalúa."),
    ("¿Funciona en el celular?", "Abre, pero está pensado para pantallas grandes. En una computadora o tableta es mucho más cómodo."),
    ("¿Por qué Windows dice «Windows protegió su PC»?", "Porque el instalador todavía no tiene firma digital; está en trámite. Elige «Más información» y «Ejecutar de todas formas». La versión en línea no tiene ese aviso."),
    ("Encontré un error o me falta algo", f'Repórtalo en <a href="{GH}/issues">GitHub</a>. Si es un error de cálculo, incluye el archivo .iees o los datos del circuito.'),
]
faq_html = "\n".join(f"        <details><summary>{q}</summary><p>{a}</p></details>" for q, a in faq)
guia = head(p, "Guía", "Guía de uso de IEEStudio en video y paso a paso: proyecto, tableros, circuitos, resultados, unifilar y memoria de cálculo.", "guia/") + page_head(
    p, "Guía", "Un proyecto completo, paso a paso",
    "Mira el recorrido en video o sigue los pasos con la app abierta. Todo lo que ves aquí funciona igual en el navegador y en Windows.",
    extra=f"""
      <div class="actions"><a class="btn btn-volt" href="{p}app/">{icon(p, "i-play")}Abrir la app para seguirla</a></div>""",
) + f"""
  <section class="block" id="video">
    <div class="wrap">
      <div class="video">
        <video controls preload="none" poster="{p}video/guia.jpg" playsinline>
          <source src="{p}video/guia.mp4" type="video/mp4">
          <track kind="captions" src="{p}video/guia.vtt" srclang="es" label="Español">
        </video>
      </div>
      <p class="video-note">Narración con voz sintética y subtítulos integrados en español. El proyecto del video es el ejemplo que trae la app.</p>
    </div>
  </section>

  <section class="block" style="padding-top:0">
    <div class="wrap">
      <p class="kicker">Paso a paso</p>
      <h2>De la acometida a la memoria</h2>
      <div class="guide">
{steps_html}      </div>
    </div>
  </section>

  <section class="block">
    <div class="wrap">
      <div class="show">
        <div class="show-media">
          <div class="frame"><img src="{p}imagenes/guia/paso-rapido.webp" width="1600" height="1000" alt="Cálculo rápido de un circuito con sus recomendaciones." loading="lazy"></div>
        </div>
        <div class="show-text">
          <p class="kicker">Atajo</p>
          <h3>Cálculo rápido de un circuito</h3>
          <p>Con <kbd>F8</kbd> o Herramientas, Cálculo rápido, calculas un circuito suelto: sistema, carga, longitud y conductor. Si no cumple, te da alternativas ya calculadas que aplicas con un clic, y con «Agregar al proyecto» lo pasas al tablero seleccionado.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="block" id="atajos">
    <div class="wrap">
      <p class="kicker">Teclado</p>
      <h2>Atajos</h2>
      <table class="keys">
{keys_html}
      </table>
    </div>
  </section>

  <section class="block" id="preguntas">
    <div class="wrap">
      <p class="kicker">Preguntas frecuentes</p>
      <h2>Lo que más se pregunta</h2>
      <div class="faq">
{faq_html}
      </div>
    </div>
  </section>
""" + foot(p)
write("guia", guia)

# ------------------------------------------------------------------ descargar
descargar = head(p, "Descargar", "Usa IEEStudio en el navegador sin instalar nada, instálalo como aplicación o descarga el instalador para Windows.", "descargar/") + page_head(
    p, "Descargar", "En línea o en Windows, el mismo programa",
    "Los mismos cálculos en las dos versiones. Un proyecto que empiezas en una lo terminas en la otra.",
    ("tuberia.webp", "Tramos de tubería conduit EMT y dobladoras en un almacén de obra."),
    extra='\n      <p class="release version" style="margin-top:20px">Consultando la versión más reciente…</p>',
) + f"""
  <section class="block">
    <div class="wrap">
      <div class="ways" style="margin-top:0">
        <article class="way">
          <div class="way-art" aria-hidden="true">
            <div class="chrome">
              <div class="chrome-bar"><span class="dots"><i></i><i></i><i></i></span><span class="url">ieestudio.com<b>/app</b></span></div>
              <img class="shot" src="{p}imagenes/app-unifilar.webp" width="1440" height="900" alt="" loading="lazy">
            </div>
          </div>
          <div class="way-body">
            <h3>En el navegador</h3>
            <ul>
              <li>Sin instalar nada: abre la página y empieza a calcular.</li>
              <li>Tu proyecto se guarda solo en el navegador y lo descargas como archivo <code>.iees</code> cuando quieras.</li>
              <li>Todo se calcula en tu computadora. Ningún proyecto se sube a un servidor.</li>
              <li>Se instala como aplicación y funciona sin internet.</li>
            </ul>
            <a class="btn btn-volt" href="{p}app/">{icon(p, "i-play")}Abrir en el navegador</a>
          </div>
        </article>
        <article class="way">
          <div class="way-art" aria-hidden="true">
            <div class="chrome">
              <div class="chrome-bar"><span class="title"><img src="{p}app/icono.svg" alt="">IEEStudio</span><span class="win"><span>—</span><span>☐</span><span>✕</span></span></div>
              <img class="shot" src="{p}imagenes/app-rapido.webp" width="1440" height="900" alt="" loading="lazy">
            </div>
          </div>
          <div class="way-body">
            <h3>En Windows</h3>
            <ul>
              <li>Instalador para Windows 10 y 11 de 64 bits.</li>
              <li>Los proyectos <code>.iees</code> abren con doble clic y se guardan directo en tus carpetas.</li>
              <li>Incluye la línea de comandos y el servidor para agentes de IA (MCP).</li>
              <li>Funciona sin internet desde el primer día.</li>
            </ul>
            <a class="btn btn-line descargar" href="{GH}/releases">{icon(p, "i-down")}Descargar para Windows</a>
          </div>
        </article>
      </div>

      <div class="howto">
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

      <table class="specs section-gap">
        <tr><th>Navegador</th><td>Chrome, Edge o Firefox recientes. La primera carga descarga unos 16 MB; después queda guardada.</td></tr>
        <tr><th>Windows</th><td>Windows 10 u 11 de 64 bits. Unos 90 MB instalado.</td></tr>
        <tr><th>Linux y otros</th><td>Se compila desde el <a href="{GH}">código fuente</a> con Rust.</td></tr>
        <tr><th>Versiones anteriores</th><td>Todas están en la <a href="{GH}/releases">página de versiones</a> de GitHub, con sus notas.</td></tr>
      </table>
    </div>
  </section>
""" + foot(p)
write("descargar", descargar)

# ------------------------------------------------------------------ código abierto
credits = [
    ("Tablero de distribución con electricista", "CANEurope, vía Flickr", "CC0 1.0 (dominio público)", "https://www.flickr.com/photos/101564557@N06/37439260981"),
    ("Tablillas de conexión", "rawpixel", "CC0 1.0 (dominio público)", "https://www.rawpixel.com/image/5904900/photo-image-public-domain-line-free"),
    ("Interruptores termomagnéticos", "rawpixel", "CC0 1.0 (dominio público)", "https://www.rawpixel.com/image/5921188/photo-image-public-domain-technology-free"),
    ("Subestación eléctrica", "rawpixel", "CC0 1.0 (dominio público)", "https://www.rawpixel.com/image/5902727/photo-image-background-public-domain-free"),
    ("Tubería conduit EMT", "Darryl, vía WordPress Photo Directory", "CC0 1.0 (dominio público)", "https://wordpress.org/photos/photo/4361f02677/"),
    ("Charola con cable 350 MCM", "Darryl, vía WordPress Photo Directory", "CC0 1.0 (dominio público)", "https://wordpress.org/photos/photo/63761f0310/"),
]
credits_html = "\n".join(f'        <tr><td>{a}</td><td>{b}</td><td>{c}</td><td><a href="{d}">Original</a></td></tr>' for a, b, c, d in credits)
codigo = head(p, "Código abierto", "Cómo se verifica IEEStudio: código y tablas públicos, pruebas, compilación en GitHub, política de firma de código y privacidad.", "codigo-abierto/") + page_head(
    p, "Código abierto", "Puedes revisar cómo calcula cada número",
    "El código, las tablas de la norma y los casos de prueba son públicos. Nadie tiene que creerle al programa: se puede comprobar.",
    ("charola.webp", "Charola portacables con cable de potencia en una subestación."),
    extra=f"""
      <div class="actions"><a class="btn btn-line" href="{GH}">{icon(p, "i-git")}Ver el repositorio</a></div>""",
) + f"""
  <section class="block">
    <div class="wrap">
      <div class="trust" style="margin-top:0">
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
          <p>El programa no envía información a ningún lado. Este sitio no usa cookies ni rastreadores.</p>
        </div>
        <div>
          {icon(p, "i-free")}
          <h3>Gratis, sin cuentas</h3>
          <p>Licencias MIT o Apache-2.0. Úsalo en los proyectos de tu trabajo sin registrarte ni pagar.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="block" id="verificacion">
    <div class="wrap">
      <p class="kicker">Verificación</p>
      <h2>Cómo se comprueba cada cálculo</h2>
      <div class="two">
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
          <p>Si una tabla no cubre un caso, el programa lo dice y lo marca como no evaluado en lugar de usar un valor aproximado.</p>
        </div>
      </div>
    </div>
  </section>

  <section class="block" id="firma">
    <div class="wrap">
      <div class="signing" style="margin-top:0">
        <div>
          <h3>Política de firma de código</h3>
          <p class="en">Free code signing provided by <a href="https://about.signpath.io">SignPath.io</a>, certificate by <a href="https://signpath.org">SignPath Foundation</a>.</p>
          <p class="state">Solicitud en trámite. Mientras tanto el instalador se publica sin firma y Windows puede mostrar «Windows protegió su PC». La versión en línea no lo necesita.</p>
        </div>
        <ul>
          <li>Los instaladores se compilan en GitHub Actions desde el código público del repositorio; ningún binario se compila ni se sube a mano.</li>
          <li>Autor y responsable de aprobar cada versión firmada: <a href="https://github.com/JAAM08">Juan Antonio Aguilar Martínez</a>.</li>
          <li>Privacidad: el programa no transfiere información a otros sistemas en red, salvo que el usuario lo pida expresamente.</li>
        </ul>
      </div>
    </div>
  </section>

  <section class="block" id="creditos">
    <div class="wrap">
      <p class="kicker">Créditos</p>
      <h2>Imágenes de este sitio</h2>
      <p class="intro">Las capturas son de IEEStudio y las hojas, de la memoria que genera. Las fotografías son de dominio público:</p>
      <table class="credits">
        <tr><th>Fotografía</th><th>Autor</th><th>Licencia</th><th></th></tr>
{credits_html}
      </table>
      <p class="intro" style="font-size:1rem">Tipografía IBM Plex, licencia SIL Open Font License. Íconos y personaje del video, propios del proyecto.</p>
    </div>
  </section>

  <section class="closing">
    <div class="wrap">
      <h2>¿Quieres contribuir?</h2>
      <p>Reporta errores, propone mejoras o revisa los casos de prueba. Todo pasa en GitHub.</p>
      <div class="actions">
        <a class="btn btn-volt" href="{GH}/issues">Reportar un problema</a>
        <a class="btn btn-line" href="{GH}">{icon(p, "i-git")}Ver el código</a>
      </div>
    </div>
  </section>
""" + foot(p)
write("codigo-abierto", codigo)
print("ok")

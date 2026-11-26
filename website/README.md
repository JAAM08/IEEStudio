# Sitio ieestudio.com

`website/` es el sitio informativo. `tools/sitio-web.sh` lo junta con la
aplicación compilada para el navegador y deja el sitio completo en
`target/sitio`:

```
target/sitio/
  index.html                 Portada: escena 3D, programa, herramientas, flujo,
                             memoria, filosofía, documentación, comunidad, descarga
  funciones/                 Herramientas: qué calcula y qué entrega
  guia/                      Documentación: video, paso a paso, atajos, preguntas
  descargar/                 En línea, Windows e instalación como app
  codigo-abierto/            Comunidad: verificación, firma de código, créditos
  estilos.css, sitio.js      Comunes a todas las páginas
  portada.js, escena.js      Solo de la portada
  iconos.svg, imagenes/, fuentes/, video/, robots.txt, sitemap.xml
  app/                       La aplicación (WebAssembly, PWA), aparte del sitio
```

Las páginas comparten navegación y pie; `python3 tools/paginas.py website` las
genera. Se pueden editar a mano, pero volver a generarlas reemplaza esos
cambios. El criterio de diseño está en `docs/decisions/0036-sitio-web-rediseno.md`.

## Sistema visual

Carbón `#17191E` y carbón profundo `#101216` de fondo, ámbar eléctrico
`#E5B84B` como acento de marca, azul técnico `#5489C7` para datos y
conexiones, violeta `#8E7CC3` solo como acento secundario, gris técnico
`#5F6368` y blanco `#F4F5F7`. Manrope para el texto e IBM Plex Mono para
valores y referencias. Cada sección es una «lámina» numerada, como un juego
de planos; los dibujos de las herramientas usan trazo técnico sobre retícula.

## Escena 3D

Fuente en `tools/escena/escena.js`; se empaqueta con three.js en
`website/escena.js`:

```
cd tools/escena && npm install && npm run build
```

Es geometría hecha en código, sin modelos externos. Los circuitos, sus
interruptores y posiciones son los del proyecto de ejemplo del programa. Sin
WebGL, la portada muestra el alzado del mismo tablero en SVG (`plano_tablero`
en `tools/paginas.py`).

## Política de imágenes

El sitio muestra el programa real: capturas de la aplicación, hojas de la
memoria que genera, el modelo 3D y dibujos técnicos propios, y el ícono. No se
usan fotografías de stock, interfaces inventadas ni resultados simulados. Las
capturas no se retocan.

| Recurso | Origen | Versión |
| --- | --- | --- |
| `imagenes/app/*.webp`, `*@2x.webp` | `ieestudio-app --captura` a escala 2, tema oscuro, 1440×900 | `main` después de la fase 0 (R7) |
| `imagenes/app/movil-*.webp` | lo mismo a 390×844 | `main` después de la fase 0 (R7) |
| `imagenes/app/unifilar-detalle.webp` | recorte de `unifilar@2x` | `main` después de la fase 0 (R7) |
| `imagenes/memoria/hoja-*.webp` | `ieestudio memoria` del ejemplo, hojas 1, 2, 3, 4, 9 y 13 | `main` después de la fase 0 (R7) |
| `imagenes/guia/*.webp` | capturas con la zona de cada paso resaltada | anterior cercana a R7 |
| `video/` | guía en video (`tools/video/README.md`) | anterior a R7; el sitio lo dice |
| `imagenes/compartir.jpg` | portada del sitio con la escena 3D | este rediseño |

Para volver a generar las capturas de `imagenes/app/` (Linux, con xvfb):

```
cargo build --release -p ieestudio-app -p ieestudio-cli
cap() { out=$1; shift; WINIT_X11_SCALE_FACTOR=2 xvfb-run -a -s "-screen 0 3200x2000x24" \
  target/release/ieestudio-app --captura "$out" "$@"; }
cap proyecto.png --vista proyecto --tamano 1440x900
cap tableros.png --vista tableros --seleccion C-04 --tamano 1440x900
cap procedimiento.png --vista tableros --seleccion C-04 --procedimiento --tamano 1440x900
cap unifilar.png --vista unifilar --tamano 1440x900
cap revisar.png --vista revisar --tamano 1440x900
cap memoria.png --vista memoria --tamano 1440x900
cap rapido.png --rapido --tamano 1440x900
cap inicio.png --inicio --tamano 1440x900
# y las mismas a --tamano 390x844 con el prefijo movil-
```

Cada PNG de 2880×1800 se guarda como `nombre@2x.webp` y reducido a 1440×900
como `nombre.webp`. Las hojas de la memoria salen de
`ieestudio ejemplo casa.iees && ieestudio memoria casa.iees` con `pdftoppm` a
1020×1320 (`hoja-N.webp`) y 2040×2640 (`hoja-N-grande.webp`).

## Publicación

El sitio se publica en GitHub Pages: cada cambio en `main` que pasa las
pruebas actualiza ieestudio.com, y «Run workflow» en la pestaña Actions lo
vuelve a publicar. El dominio se compró en Hostinger y solo apunta a GitHub
Pages con sus registros DNS; no hay hosting contratado.

La integración continua arma el sitio en cada cambio (artefacto
`ieestudio-sitio`) y lo adjunta a cada versión como
`IEEStudio-<versión>-sitio-web.zip`. Si algún día se contrata hosting, basta
con extraer ese zip en `public_html`: el `.htaccess` declara el tipo de los
archivos `.wasm`. En cualquier caso hace falta HTTPS para que la aplicación se
pueda instalar.

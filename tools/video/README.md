# Video de la guía

Herramientas para volver a producir `website/video/guia.mp4` cuando cambie la
interfaz. Nada de esto se usa para compilar el programa.

1. `guion.py`: lo que se dice, los subtítulos, los clics en la app y las zonas
   que se resaltan, en coordenadas de una ventana de 1440 × 810.
2. `personaje.js`: dibuja el personaje (original del proyecto) con la boca
   cerrada, entreabierta y abierta, y los ojos abiertos y cerrados.
3. La voz se genera con [Piper](https://github.com/rhasspy/piper) y la voz
   `es_MX-claude-high`, un archivo por parte del guion.
4. `grabar.js`: abre la app con Playwright a 1920 × 1080, hace los clics y
   anota cuándo empieza cada parte y cuándo la pantalla ya respondió.
5. `montar.py`: recorta lo que la app tardó en responder, coloca la voz,
   dibuja el personaje (la boca sigue a la voz), los subtítulos, los
   resaltados, la portada y el cierre, y escribe el MP4 y los subtítulos en
   VTT. Después se normaliza el volumen con `ffmpeg -af loudnorm`.

Requiere el sitio servido en `http://localhost:8765` (`tools/sitio-web.sh` y
`python3 -m http.server 8765` dentro de `target/sitio`), Node con Playwright,
Python con Pillow y NumPy, `piper-tts` y ffmpeg.

`tools/paginas.py` genera las páginas de `website/` con la misma navegación y
el mismo pie; se puede editar el HTML a mano o cambiar el script y volver a
generarlas con `python3 tools/paginas.py website`.

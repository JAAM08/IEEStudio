# 0036. Rediseño de ieestudio.com: sitio estático, escena 3D propia y solo imágenes reales

**Estado:** propuesta (26 de noviembre de 2026), en revisión en su PR. No cambia
la aplicación ni el motor de cálculo: solo `website/` y las herramientas que lo
generan.

## Contexto

El dueño del proyecto pidió rediseñar el sitio informativo con una identidad
propia («Technical editorial × interactive engineering»), elementos 3D reales,
capturas auténticas del programa y sin fotografías de stock. El sitio era HTML y
CSS estáticos generados por `tools/paginas.py`, publicados por la integración
continua en GitHub Pages junto con la aplicación en `/app`.

## Decisión

- **Se conserva la arquitectura.** Sitio estático sin framework, generado por
  `tools/paginas.py`; mismo despliegue (`tools/sitio-web.sh`, GitHub Pages) y
  mismas direcciones (`/funciones/`, `/guia/`, `/descargar/`,
  `/codigo-abierto/`, `/app/`), para no romper enlaces. La navegación las nombra
  Herramientas, Documentación, Descargar y Comunidad.
- **Escena 3D con three.js, empaquetada.** La portada muestra un tablero
  construido con geometría en código (`tools/escena/escena.js`), sin modelos
  descargados. Sus circuitos, interruptores y posiciones son los del proyecto de
  ejemplo del programa (TG-1, posiciones del unifilar). `npm run build` en
  `tools/escena` genera `website/escena.js` (three.js r187 incluido, sin CDN).
  Se carga después del contenido, solo con WebGL, y se pausa fuera de pantalla
  o con la pestaña oculta; con `prefers-reduced-motion` no hay pulsos ni
  paralaje. Sin WebGL, o mientras carga, se ve el alzado del mismo tablero en
  SVG. La simbología es una lista de botones: todo lo que hace la escena se
  puede hacer con el teclado y sin ella.
- **Solo recursos propios.** Capturas reales de la aplicación (`--captura` a
  escala 2), hojas reales de la memoria del proyecto de ejemplo, dibujos
  técnicos propios y el ícono del programa. Se retiran las fotografías de stock.
  Las capturas no se retocan; los números que cita el texto salen del proyecto
  de ejemplo (`ieestudio calcular`).
- **Sin cifras inventadas.** Versiones y cambios recientes se consultan a la API
  pública de GitHub en el navegador (con 10 minutos de caché); si falla, quedan
  los enlaces.

## Consecuencias

- Al cambiar la interfaz hay que volver a generar las capturas (ver
  `website/README.md`). Las de la guía (`imagenes/guia/`) y el video son de una
  versión cercana anterior y quedan señaladas como tales.
- La escena agrega unos 150 kB comprimidos que solo se descargan con WebGL y
  después de la carga inicial.

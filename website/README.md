# Sitio ieestudio.com

`website/` es el sitio informativo. `tools/sitio-web.sh` lo junta con la
aplicación compilada para el navegador y deja el sitio completo en
`target/sitio`:

```
target/sitio/
  index.html                 Inicio
  funciones/                 Qué calcula y qué entrega
  guia/                      Video y guía paso a paso, atajos, preguntas
  descargar/                 En línea, Windows e instalación como app
  codigo-abierto/            Verificación, firma de código, privacidad, créditos
  estilos.css, sitio.js, iconos.svg, imagenes/, fuentes/, video/
  app/                       La aplicación (WebAssembly, PWA), aparte del sitio
```

Las páginas comparten navegación y pie; `tools/paginas.py` las genera. Se
pueden editar a mano, pero volver a generarlas reemplaza esos cambios.

La integración continua arma el sitio en cada cambio (artefacto
`ieestudio-sitio`) y lo adjunta a cada versión como
`IEEStudio-<versión>-sitio-web.zip`.

## Publicación

El sitio se publica en GitHub Pages: cada cambio en `main` que pasa las
pruebas actualiza ieestudio.com, y «Run workflow» en la pestaña Actions lo
vuelve a publicar. El dominio se compró en Hostinger y solo apunta a GitHub
Pages con sus registros DNS; no hay hosting contratado.

Si algún día se contrata hosting, basta con extraer el zip del sitio en
`public_html`: el `.htaccess` declara el tipo de los archivos `.wasm`. En
cualquier caso hace falta HTTPS para que la aplicación se pueda instalar.

## Imágenes y video

- `imagenes/`: capturas de la aplicación en el navegador a 2x con el proyecto
  de ejemplo, en WebP; `imagenes/guia/` las mismas con la zona de cada paso
  resaltada; las hojas de la memoria son del PDF que genera el programa.
- `imagenes/fotos/`: fotografías de dominio público (CC0); autores y enlaces
  en la página Código abierto, sección Créditos.
- `video/`: la guía en video, sus subtítulos y su portada. Cómo se produce:
  `tools/video/README.md`.

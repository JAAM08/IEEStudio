# Página de ieestudio.com

`website/` es la página de inicio. `tools/sitio-web.sh` la junta con la
aplicación compilada para el navegador y deja el sitio completo en
`target/sitio`:

```
target/sitio/
  index.html, .htaccess, favicon.ico, imagenes/, fuentes/   ← esta carpeta
  app/                                                      ← la aplicación (WebAssembly, PWA)
```

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

## Capturas

Las imágenes de `imagenes/` son capturas de la aplicación en el navegador a
2x, con el proyecto de ejemplo, convertidas a WebP; las de la memoria son
hojas del PDF que genera el programa.

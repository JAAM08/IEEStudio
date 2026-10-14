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

## Publicarlo en Hostinger

En el Administrador de archivos, carpeta `public_html` del dominio: subir el
zip y extraerlo ahí, de modo que `index.html` y la carpeta `app` queden
directo en `public_html`. El `.htaccess` declara el tipo de los archivos
`.wasm` y hace que una versión nueva llegue en cuanto se publica. Hace falta
HTTPS (SSL) para que la aplicación se pueda instalar.

## Capturas

Las imágenes de `imagenes/` son capturas de la aplicación en el navegador a
2x, con el proyecto de ejemplo, convertidas a WebP; las de la memoria son
hojas del PDF que genera el programa.

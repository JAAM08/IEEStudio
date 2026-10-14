# Página de ieestudio.com

Página estática, sin dependencias: `index.html`, `imagenes/`, `fuentes/` y
`favicon.ico`. El botón de descarga consulta a GitHub la versión más reciente
con instalador, así que no hay que editarla en cada versión.

Para publicarla en Hostinger: Administrador de archivos, carpeta
`public_html` del dominio, subir `ieestudio-sitio.zip` y extraerlo ahí (que
`index.html` quede directo en `public_html`).

Las capturas se generan con
`cargo run -p ieestudio-app -- casa.iees --captura imagenes/principal.png`
(y `--espacio diagrama`, `--panel rapido`) a partir de `ieestudio ejemplo`.

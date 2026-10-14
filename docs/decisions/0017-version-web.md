# 0017. La aplicación en el navegador

**Estado:** aceptada (octubre de 2026).

## Contexto

La interfaz se escribió con egui (decisión 0002) en parte para poder llevarla
al navegador. Quien desconfía de un instalador sin firma, o no puede instalar
programas en la computadora del trabajo, debe poder usar IEEStudio sin
instalar nada, con los mismos cálculos.

## Decisión

- **Es el mismo programa.** `ieestudio-app` se compila a WebAssembly
  (`wasm32-unknown-unknown`) y corre completo en el navegador: cálculo,
  procedimiento, unifilar, memoria en PDF, Excel y DXF. No hay servidor: los
  proyectos nunca salen de la computadora del usuario. Las tablas y las
  tipografías ya iban dentro del programa, así que no hay archivos aparte.
- **Archivos.** En el navegador no hay carpetas. Abrir es elegir un archivo con
  el selector del navegador, que llega un momento después; guardar y exportar
  es descargar. El módulo `files` esconde la diferencia: lo que se abre llega
  siempre a un buzón que la aplicación atiende en cada cuadro, en escritorio y
  en el navegador.
- **El proyecto se guarda solo en el navegador** (almacenamiento local), junto
  con el acomodo de la interfaz, y se retoma al volver a abrir la página.
  Descargar el `.iees` sigue siendo la forma de tener una copia propia.
- **Lo que no existe en el navegador:** «Guardar como», «Abrir reciente» y
  «Salir»; la línea de comandos y el servidor MCP siguen siendo de escritorio.
- **Aplicación instalable (PWA).** Manifiesto e íconos para instalarla desde el
  navegador, y un service worker que guarda la aplicación para que abra al
  instante y sin internet. Cada compilación tiene su propia versión de caché.
- **Publicación.** `tools/sitio-web.sh` arma ieestudio.com en `target/sitio`:
  la página de inicio (`website/`) y la aplicación en `/app`. La integración
  continua lo compila después de las pruebas y lo adjunta a cada versión.

## Consecuencias

- La primera carga es pesada: el motor de PDF (Typst) es la mayor parte del
  programa. Después queda guardada en el navegador.
- La interfaz está pensada para pantallas grandes; en el celular abre, con un
  aviso de que es más cómoda en computadora o tableta.
- Cualquier código nuevo que lea o escriba archivos pasa por `files`; usar
  `std::fs` o `rfd` directamente rompe la versión del navegador, y la
  integración continua lo detecta al compilarla.

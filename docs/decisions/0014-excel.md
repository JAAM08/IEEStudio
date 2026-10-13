# 0014. Cuadro de cargas de ida y vuelta con Excel

**Estado:** aceptada (octubre de 2026).

## Contexto

Muchos proyectos ya existen en una hoja de cálculo, y capturar cien circuitos
uno por uno en la interfaz es lento. A la vez, el cliente o el revisor suelen
pedir el cuadro de cargas en Excel.

## Decisión

- **Un libro con tres hojas.** «Tableros» y «Cuadro de cargas» llevan los datos
  que captura el usuario; «Resultados», lo que calculó el programa, con las
  observaciones de lo que no cumple o no se evaluó. Las listas (tipo de carga,
  conexión, carga continua) llevan validación de datos en Excel.
- **Las columnas se reconocen por su encabezado**, sin importar el orden ni
  mayúsculas. Solo «Tablero» e «ID» son obligatorias; una columna que no está
  deja el dato como estaba o con su valor de omisión. La conexión acepta «1F»,
  «3F-4H» y similares, y las fases «R S T».
- **Importar nunca borra.** Agrega o actualiza tableros y circuitos por su ID;
  lo que no está en el libro queda igual. Un tablero que no existe se crea.
- **Las filas malas no detienen la importación.** Se informan con su número de
  fila y la razón, y el resto entra. En la aplicación, la importación se puede
  deshacer completa desde el mismo aviso.
- **Calibre fijado** se valida contra las tablas del perfil al importar.
- `rust_xlsxwriter` escribe y `calamine` lee; los dos en Rust puro, sin
  depender de Excel instalado.

## Lo que no se hizo todavía

Las áreas por tipo de la Tabla 220-12 (se exporta e importa el total con el
tipo del inmueble), los datos de cortocircuito del alimentador y las
decisiones de diseño distintas del calibre y el interruptor se capturan en la
aplicación.

# 0019. Rediseño de la interfaz, fase 1: escritorio

**Estado:** aceptada (noviembre de 2026). Amplía la 0010.

## Contexto

La interfaz por paneles (0010) ya tenía espacios de trabajo, paneles
acoplables, navegador y propiedades en Datos / Resultados / Normativa. Lo que
faltaba era jerarquía: el resumen del tablero se leía como una fila de texto
pequeño, las acciones sobre un circuito estaban mezcladas con las del
tablero, y la referencia normativa de cada resultado quedaba escondida en la
pestaña Normativa, aunque la trazabilidad es la razón de ser del programa.

El rediseño toma como columna vertebral el navegador y las propiedades
ligadas a la selección de Revit; de Photoshop, la barra contextual; de
Premiere, los espacios por tarea y su restablecimiento; de AutoCAD, el
acceso por teclado; de Fluent 2, las medidas. Se conservan la paleta, IBM
Plex y el amarillo solo para marca, foco y acción principal. La selección
sigue siendo neutra.

## Decisión

- **Pantalla de inicio.** Al arrancar sin proyecto que retomar: Nuevo, Abrir,
  Ejemplo, Cálculo rápido y recientes. También desde Archivo › Inicio.
- **Cálculo rápido y Buscar en la barra superior**, fuera de los menús: el
  cálculo de un circuito suelto es lo que más se usa en obra.
- **Encabezado del tablero.** Nombre, sistema y de dónde se alimenta; conteo
  por estado; cifras grandes con su unidad separadas por líneas finas, sin
  tarjetas: carga conectada y calculada, corriente, factor de potencia,
  **caída máxima** del tablero (en rojo si no cumple) y carga por fase en kVA.
- **Barra contextual** sobre la tabla, con el elemento seleccionado: duplicar,
  eliminar, cambiar de fase (A, B, C o el par), fijar o liberar el calibre y
  abrir su procedimiento. Las acciones del tablero (agregar circuito,
  balancear) se quedan en la barra de herramientas.
- **Trazabilidad visible.** Cada sección de Resultados (protección,
  conductor, tubería, caída) muestra a la derecha el estado de las
  verificaciones que la respaldan y sus artículos; al pasar el cursor se lee
  el detalle y al tocarlos se abre la pestaña Normativa. Lo demás que se
  verificó aparece en «Otras verificaciones» con su referencia.
- **Heredado frente a propio.** Lo que toma el valor del proyecto o lo
  propone IEEStudio se ve en gris («Cobre · del proyecto», «Lo propone
  IEEStudio»); un valor propio lleva al lado un botón para volver al heredado.
- **Paleta de comandos (Ctrl+K).** Comandos, paneles, espacios de trabajo y
  cualquier tablero o circuito por su ID o descripción, sin importar acentos.

## Lo que no entra en esta fase

Tablet y teléfono (fases 2 y 3). Antes hay que probar en un teléfono real
cómo se comporta egui con el teclado virtual, el desplazamiento y la
accesibilidad en el navegador; del resultado depende cómo se hacen.

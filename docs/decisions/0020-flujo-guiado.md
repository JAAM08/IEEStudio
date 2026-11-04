# 0020. Vista guiada en cinco pasos

**Estado:** aceptada (noviembre de 2026). Cambia la vista de omisión de la
0010; la 0019 sigue valiendo dentro de la vista avanzada.

## Contexto

Con la 0019 la interfaz quedó más clara, pero seguía siendo la misma
estructura: siete paneles acoplables, una tabla de quince columnas y un
panel de propiedades largo. Quien abre el programa por primera vez no sabe
por dónde empezar, y el dueño del proyecto pidió un rediseño que de verdad
facilitara su manejo. Se revisaron maquetas antes de programar.

## Decisión

- **Dos vistas sobre el mismo proyecto.** La **guiada** es la de omisión; la
  **avanzada** son los paneles acoplables de siempre. Se cambia con un botón
  de la barra superior o desde Ctrl+K, y la elección se guarda. Ninguna
  duplica datos ni cálculos: las dos editan el mismo `Doc`.
- **Cinco pasos en la barra superior:** Proyecto → Tableros → Circuitos →
  Revisar → Memoria. Un paso con lo mínimo se marca con ✓; Revisar muestra
  cuántos pendientes hay desde cualquier paso. El orden se sugiere, no se
  obliga: se salta a cualquier paso, y el pie lleva al anterior y al
  siguiente.
- **Proyecto:** datos de la obra; tipo de inmueble y sistema como tarjetas;
  los criterios de diseño resumidos en una línea y plegados.
- **Tableros:** el árbol de alimentación (fuente → tableros → subtableros)
  con la ficha del tablero al lado.
- **Circuitos:** la tabla separa **lo que se captura** (tipo, carga, fase,
  longitud) de **lo que resuelve IEEStudio** (cable, interruptor, caída
  total, estado), con nueve columnas. La ficha dice en palabras qué no
  cumple, con su referencia, y ofrece el arreglo cuando lo hay («Usar 10
  AWG»); debajo, los datos esenciales y, plegados, las opciones avanzadas y
  el procedimiento.
- **Revisar:** los tres niveles del veredicto, todo lo pendiente en una
  lista (primero lo que no cumple) con su arreglo e «Ir al circuito», lo que
  queda fuera de alcance, los avisos y el unifilar ajustado al ancho.
- **Memoria:** datos de firma, la lista «Antes de entregar», la vista previa
  y los botones de PDF, Excel y DXF.
- **Siempre a la mano:** Archivo, deshacer y rehacer, Cálculo rápido (en la
  vista guiada abre como ventana flotante), Buscar (Ctrl+K, que también
  lleva a cualquier paso) y el cambio de tema.

## Lo que no entra todavía

Editar celdas directo en la tabla y plantillas de circuito: se dejaron para
después de probar este flujo. Tablet y teléfono siguen pendientes de la
prueba de egui en un teléfono real; el flujo por pasos ya está pensado para
llevarse ahí como una vista por paso con la barra de pasos abajo.

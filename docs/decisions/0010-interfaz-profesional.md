# 0010. Interfaz por paneles acoplables y espacios de trabajo

**Estado:** aceptada (octubre de 2026).

## Contexto

La primera interfaz tenía tres vistas fijas y la tabla, el procedimiento y
las recomendaciones competían por atención. Se pidió una interfaz con la
filosofía de las aplicaciones profesionales (Adobe, Revit): un solo proyecto
como fuente de verdad y varias vistas sobre él.

## Decisión

- **Paneles acoplables** (egui_dock): Proyecto, Cuadro de cargas, Unifilar,
  Memoria, Procedimiento y Propiedades. Se arrastran, se dividen, se cierran y
  se vuelven a abrir desde el menú Ventana (F2 a F8).
- **Espacios de trabajo** Cálculo, Diagrama y Documentación, a la derecha de
  la barra de menús. El acomodo, el tema, el acento y las columnas se guardan
  al cerrar.
- **Navegador del proyecto** en árbol, en lugar de un menú lateral de
  navegación: el menú de 20 entradas de la propuesta repetía elementos y
  anunciaba funciones inexistentes.
- **Propiedades en tres pestañas:** Datos (lo que captura el usuario),
  Resultados (lo que calcula VoltForge; el calibre se marca «propuesto» o
  «fijado por ti») y Normativa (verificaciones con su cláusula).
- **Cálculo rápido** como herramienta en ventana flotante, marcada «fuera del
  proyecto».
- **Memoria** con vista previa de páginas blancas dentro del entorno oscuro.
- **Color:** el acento (amarillo eléctrico) solo para marca, foco y
  selección; la fila seleccionada es gris neutro con una barra de acento. Los
  estados tienen colores propios y siempre llevan ícono y texto: verde
  cumple, rojo no cumple, ámbar anaranjado sin evaluar.
- **Densidad:** tipografía de 13 px, controles de 22 px, sin tarjetas de
  indicadores; el resumen del tablero es una franja de una línea.

## Lo que no se hizo, a propósito

- Campos sin cálculo detrás (capacidad interruptiva, curva del interruptor):
  sugerirían una verificación que no existe.
- Una tarjeta de «cumple con la normativa aplicable»: contradice el veredicto
  en tres niveles (0009).
- Un editor vectorial libre para el unifilar: el diagrama sale del proyecto;
  dibujar a mano permitiría contradecir el cuadro de cargas.

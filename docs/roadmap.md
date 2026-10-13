# Estado y pendientes

## Hecho

- Tablas de la NOM-001-SEDE-2012 en CSV, validadas y cotejadas contra una
  captura independiente; referencias y parámetros del perfil en datos.
- Motor: corriente, caída de tensión, conductor, interruptor, puesta a tierra
  y tubería, con pruebas matemáticas, de referencia y normativas, y casos de
  ingeniería en `fixtures/`.
- Proyecto con subtableros, cuadro de cargas, caída acumulada, procedimiento
  paso a paso con referencias y veredicto en tres niveles.
- Cálculo rápido de un circuito con recomendaciones.
- Diagrama unifilar automático.
- Memoria de cálculo y hoja de cálculo rápido en PDF.
- Aplicación de escritorio con paneles acoplables, espacios de trabajo,
  navegador del proyecto, propiedades en Datos / Resultados / Normativa,
  vista previa de la memoria y preferencias guardadas; línea de comandos con
  salida JSON.
- Integración continua en Windows y Linux que publica los ejecutables.
- Motores y motocompresores (artículos 430 y 440): corriente por tabla,
  conductor, interruptor, sobrecarga, placa de equipos de aire acondicionado y
  alimentadores con motores (ver la decisión 0011). También en el cálculo
  rápido. Interruptor fijado por el usuario en cualquier tramo.
- Carga de los alimentadores según el artículo 220: tipo de inmueble, áreas
  por tablero, mínimas de alumbrado y de contactos, y factores de demanda por
  grupo (ver la decisión 0012).
- Cortocircuito en cada tablero y capacidad interruptiva de cada interruptor
  (110-9, 110-10; ver la decisión 0013).
- Datos de la obra en la memoria (cliente, ubicación, revisión, responsable
  y cédula) y cuadro de cargas de ida y vuelta con Excel (decisión 0014).
- Conductor al electrodo de puesta a tierra de la acometida (Tabla 250-66,
  con el área equivalente de los paralelos) y balanceo automático de fases
  por tablero (todas las combinaciones hasta 10 circuitos; búsqueda local
  arriba de eso), que se puede deshacer.
- Servidor MCP para agentes de IA (`voltforge mcp`): proyecto de ejemplo,
  cálculo de proyecto y de un circuito, memoria, Excel, DXF y reglas.
- Unifilar en DXF (R12, en capas) y botón para pasar un cálculo rápido al
  tablero seleccionado.
- Deshacer y rehacer (Ctrl+Z, Ctrl+Y). Conductores en paralelo propuestos
  por el programa, un juego por tubo (decisión 0015).
- Instalador de Windows (Inno Setup) con ícono, accesos directos y la
  extensión `.vfp`; una etiqueta `v*` publica la versión en GitHub. El
  programa abre el último proyecto usado y recuerda los recientes.

## Pendiente, en orden

Alcance de la versión 1.0: instalaciones residenciales, comerciales e
industriales en baja tensión, con motores.

1. Revisión de los casos de `fixtures/` por un ingeniero (`reviewed_by`).
   Requisito para la beta.

## Después de la 1.0

Transformador y acometida en media tensión (artículo 450); lo que
falta del artículo 220 (estufas y secadoras, calefacción, neutro del
alimentador, métodos opcionales); falla de fase a tierra; coordinación de
protecciones; charola; sistemas fotovoltaicos; sistemas de tierra y otros
perfiles normativos.

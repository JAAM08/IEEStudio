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
- Instalador de Windows (Inno Setup) con ícono, accesos directos y la
  extensión `.vfp`; una etiqueta `v*` publica la versión en GitHub. El
  programa abre el último proyecto usado y recuerda los recientes.

## Pendiente, en orden

Alcance de la versión 1.0: instalaciones residenciales, comerciales e
industriales en baja tensión, con motores.

1. Motores y aire acondicionado (artículos 430 y 440).
2. Cálculo de la carga según el artículo 220: alumbrado por m², contactos,
   factores de demanda.
3. Cortocircuito y capacidad interruptiva (110-9 y 110-10).
4. Datos del proyecto en la memoria (cliente, ubicación, responsable, cédula,
   revisión) e importar y exportar el cuadro de cargas en Excel.
5. Transformador y acometida en media tensión (artículo 450); conductor del
   electrodo de puesta a tierra (250-66); balanceo automático de fases;
   paralelos automáticos dentro del proyecto; deshacer y rehacer; unifilar en
   DXF; pasar un cálculo rápido al proyecto.
6. Servidor MCP para agentes.
7. Revisión de los casos de `fixtures/` por un ingeniero (`reviewed_by`).
   Requisito para la beta.

## Fuera del alcance de la versión 0.1

Reglas de motores y de climatización, cortocircuito y capacidad interruptiva,
coordinación, transformador y media tensión, charola, sistema fotovoltaico,
sistemas de tierra y otros perfiles normativos.

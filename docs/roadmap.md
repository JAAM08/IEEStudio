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

## Pendiente, en orden

1. Deshacer y rehacer. Exportar el unifilar a DXF para AutoCAD. Agregar y quitar tableros desde la
   interfaz, guardar el tema elegido, pasar un cálculo rápido a un proyecto y
   alternativas (paralelo, longitud máxima) también dentro del proyecto.
2. Instalador de Windows, con la reducción del tamaño del ejecutable.
3. Servidor MCP para agentes.
4. Revisión de los casos de `fixtures/` por un ingeniero (`reviewed_by`).

## Fuera del alcance de la versión 0.1

Reglas de motores y de climatización, cortocircuito y capacidad interruptiva,
coordinación, transformador y media tensión, charola, sistema fotovoltaico,
sistemas de tierra y otros perfiles normativos.

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
- Cortocircuito retirado del programa (decisión 0018): la capacidad
  interruptiva queda fuera de alcance y el veredicto lo dice como «Conforme
  en lo evaluado».
- Datos de la obra en la memoria (cliente, ubicación, revisión, responsable
  y cédula) y cuadro de cargas de ida y vuelta con Excel (decisión 0014).
- Transformadores de 600 V o menos dentro del proyecto: tensión por tablero,
  primario y secundario dimensionados con la capacidad del transformador,
  protección de la Tabla 450-3(b) y caída desde el secundario (decisión 0016).
- Conductor al electrodo de puesta a tierra de la acometida (Tabla 250-66,
  con el área equivalente de los paralelos) y balanceo automático de fases
  por tablero (todas las combinaciones hasta 10 circuitos; búsqueda local
  arriba de eso), que se puede deshacer.
- Servidor MCP para agentes de IA (`ieestudio mcp`): proyecto de ejemplo,
  cálculo de proyecto y de un circuito, memoria, Excel, DXF y reglas.
- Unifilar en DXF (R12, en capas) y botón para pasar un cálculo rápido al
  tablero seleccionado.
- Deshacer y rehacer (Ctrl+Z, Ctrl+Y). Conductores en paralelo propuestos
  por el programa, un juego por tubo (decisión 0015).
- Instalador de Windows (Inno Setup) con ícono, accesos directos y la
  extensión `.iees`; una etiqueta `v*` publica la versión en GitHub. El
  programa abre el último proyecto usado y recuerda los recientes.
- La aplicación completa en el navegador (WebAssembly), instalable como PWA y
  sin internet, y la página de ieestudio.com que la publica en `/app`
  (decisión 0017). El programa avisa en pantalla si no puede arrancar, en
  escritorio y en el navegador.

- Rediseño de la interfaz, fase 1 (decisión 0019): pantalla de inicio,
  encabezado del tablero, barra contextual, trazabilidad visible en
  Resultados, valores heredados en gris y paleta de comandos (Ctrl+K).
- Vista guiada en cinco pasos como vista de omisión (decisión 0020):
  Proyecto, Tableros, Circuitos, Revisar y Memoria; la vista avanzada con
  paneles queda a un clic.
- Agregar circuito por tipo de carga (decisión 0021): «¿Qué vas a
  conectar?» con motor y aire acondicionado a la vista, ficha que pide
  primero los datos del equipo, símbolos por tipo y aviso en Revisar cuando
  un circuito parece motor o aire acondicionado.
- Unifilar con el formato de un plano (decisión 0022): bloques de acometida,
  transformador, alimentadores y protecciones, recuadro y encabezado de cada
  tablero, símbolos por carga, subtableros abajo, filas, y modos Revisión y
  Plano con simbología, notas y cajetín.
- Capacidad interruptiva con la falla estimada por bus infinito (decisión
  0023): dato de la suministradora o subestación, tolerancia de la
  impedancia, aportación de motores a la vista, «cumple con un método
  conservador» o «no concluyente»; datos de catálogo de acometida,
  subestación, transformadores, tableros e interruptores, y posiciones
  propuestas en el tablero.

## Pendiente, en orden


- Rediseño, fases 2 y 3: tablet y teléfono, llevando la vista guiada a una
  vista por paso. Antes, una prueba en un teléfono real de egui en el
  navegador (teclado virtual, desplazamiento).
- Por decidir tras usar la vista guiada: editar celdas directo en la tabla y
  plantillas de circuito.
- Volver a tomar las capturas de la guía del sitio con la interfaz nueva.
- Cotejar los símbolos del unifilar con el texto de la NMX-J-136-ANCE-2019 y
  agregar la tabla de impedancias de la NMX-J-116 para transformadores sin
  dato de placa.
- Unifilar fase 3: bloques con atributos en el DXF y tamaños de hoja.

Alcance de la versión 1.0: instalaciones residenciales, comerciales e
industriales en baja tensión, con motores.

1. Revisión de los casos de `fixtures/` por un ingeniero (`reviewed_by`).
   Requisito para la beta.

## Después de la 1.0

Acometida y transformadores en media tensión (Tabla 450-3(a)); lo que
falta del artículo 220 (estufas y secadoras, calefacción, neutro del
alimentador, métodos opcionales); charola; sistemas fotovoltaicos y otros
perfiles normativos.

## Suite eléctrica

Programas aparte, reimplementados desde las normas públicas (decisión 0018):

- **Cortocircuito**: falla trifásica y de fase a tierra, coordinación de
  protecciones. Al existir, IEEStudio importará la corriente de falla de cada
  tablero para volver a verificar la capacidad interruptiva (110-9).
- **Malla de tierras**: sistemas de tierra y potenciales de paso y contacto.

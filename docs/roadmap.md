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
- Sistema y conexión coherentes (decisión 0024): el sistema decide qué
  conexiones existen, el cálculo rechaza las imposibles, la interfaz solo
  ofrece las que hay con su tensión, las fases se eligen con botones y no se
  preguntan cuando están determinadas, y los proyectos guardados con una
  combinación imposible se corrigen con un botón.
- Estados claros (decisión 0025): el estado de cada circuito es el de su
  cálculo, la capacidad interruptiva es una verificación por tablero con su
  dato a la mano, y el número de pendientes es el mismo en todas las
  pantallas. La tabla de circuitos y la barra de pasos caben sin cortarse.
- Unifilar a pantalla completa en la vista guiada, desde Revisar y Memoria,
  con Revisión / Plano y exportar; el dibujo se guarda entre cuadros y solo
  se rehace cuando el proyecto cambia.
- Auditoría del motor de falla (decisiones 0026, 0027 y 0028): cada dato en su
  nivel y sin valores supuestos; estados Válido, No concluyente, Sin evaluar
  y Dato inválido con «Cómo se obtuvo»; motores según su ubicación, a través
  de la impedancia de su transformador; estudio por partes; avisos de
  transformador capturado dos veces con confirmación. Informe en
  `auditoria-modelo-y-falla.md`.
- Pendientes de esa auditoría (decisión 0033): la conexión del secundario
  sale de su topología declarada (17); circuitos y alimentadores nuevos
  nacen sin potencia ni longitud y 0 m es dato inválido, formato 6 (19). Los
  cuatro estados en cada verificación de los tramos (20) quedan para el
  rediseño de interfaz.
- Auditoría independiente de Fable (`auditoria-fable-pr11.md`): aprobado con
  observaciones H-01 a H-18. Plan de trabajo hacia la 1.0 en `plan-1.0.md`.
- Etapa 0 del plan: cada hallazgo de las dos auditorías tiene un caso
  reproducible (`tools/auditoria.sh`, `etapa-0-linea-base.md`) y las
  decisiones D-1 a D-8, analizadas en `decisiones-pendientes-d1-d8.md`, quedaron
  aprobadas en la decisión 0029.
- Rediseño 2026, etapa R1 (`rediseno-2026-r1-linea-base.md`): Revisar ya no
  se cierra en ventanas angostas y los paneles de los pasos pasan abajo
  cuando no caben; la versión web nueva llega completa y avisa para recargar;
  capturas en cinco tamaños (`tools/capturas.sh`) y mediciones de la vista
  previa de la memoria y del teclado del teléfono.
- Rediseño 2026, etapa R2 (`rediseno-2026-r2-sistema-visual.md`): sistema
  visual nuevo con amarillo eléctrico, azul técnico y morado lavanda sobre
  obsidiana y grafito, en tema oscuro y claro; etiquetas de estado uniformes
  y esquinas en tres medidas. Sin cambios de cálculo.
- Rediseño 2026, etapa R3 (`rediseno-2026-r3-marco-unico.md`): un solo
  marco en escritorio, sin modo guiado ni avanzado: barra superior con las
  cinco vistas, navegador e inspector fijos, procedimiento en un panel
  inferior, cálculo rápido flotante y «Siguiente» en la barra de estado.
- Rediseño 2026, etapa R4 (`rediseno-2026-r4-captura-unificada.md`): vista
  Proyecto con resumen, tabla de tableros y cuatro bloques de captura; un
  cuadro con juegos «Esencial» y «Completo» y filtro «Pendientes»; un árbol
  con «Agregar»; una ficha con lo que no cumple, su arreglo y los ajustes
  manuales; verificación de capacidad interruptiva con confirmación;
  importación de Excel en tres pasos.
- Rediseño 2026, etapa R4.5 (`rediseno-2026-r4-5-lenguaje-visual.md` y
  `sistema-visual.md`): lenguaje visual para todo IEEStudio con escala
  tipográfica de cinco tamaños, retícula de 4 y 8 px, tres superficies con
  sombra solo en lo flotante, barra de vistas en cápsula, navegador integrado
  al cromo, ficha en grupos de campos y barra de estado discreta. Obligatorio
  para R5, R6 y R7.
- Rediseño 2026, etapa R5 (`rediseno-2026-r5-unifilar-y-memoria.md`):
  Memoria convierte en imagen solo las páginas a la vista (seis como mucho),
  así el proyecto de 522 páginas abre con 553 MiB en vez de más de 5.4 GB;
  ficha de Memoria a la derecha con firma, «Antes de entregar» y
  exportaciones en grupos de campos; unifilar con controles flotantes en
  cápsula, zoom con Ctrl y la rueda o pellizco sobre el cursor, arrastre, y
  sin dibujar lo que queda fuera de la ventana.
- Rediseño 2026, etapa R6 (`rediseno-2026-r6-tablet-y-telefono.md`):
  pantalla de inicio con dos experiencias, «Proyecto completo» y «Cálculo
  rápido» (esta en su propia pantalla, sin proyecto); teléfono con barra
  inferior, listas de tableros y circuitos, ficha a pantalla completa y hoja
  inferior en el unifilar; tableta con el cuadro completo y cajones para el
  navegador y la ficha; modo táctil de 44 px. Navegación y desplazamiento
  validados con toques en Chromium con un teléfono emulado.
- Rediseño 2026, etapa R7 (`rediseno-2026-r7-retiro-y-cierre.md` y decisión
  0034): Revisar rehecha con la ficha al lado (cajón en tableta, pantalla
  completa en teléfono), contadores y lista armada una vez por cambio; se
  retiran `guided.rs`, `egui_dock`, las opciones de línea de comandos de
  antes y el aviso de «pantallas grandes»; repaso final del sistema visual;
  capturas y texto nuevos de la guía del sitio; tabla final de rendimiento.
  Con esto termina el rediseño 2026.

## Pendiente, en orden


El orden y el detalle están en `plan-1.0.md`. En resumen:

- Etapa 1: cerrar los riesgos de cálculo y de dictamen (H-01 a H-07, H-10 y
  H-11) con los criterios de la decisión 0029.
- Etapa 2: hecha (H-12 a H-18 y pendientes 17 y 19); el 20 se diferió al
  rediseño (decisión 0033).
- Etapa 4: unifilar en el navegador y cadena de publicación (versión
  visible, actualización del service worker, imágenes del sitio).
- Etapa 5: rendimiento, incluido el tamaño del `.wasm` (50.8 MB) y la
  composición de la memoria que bloquea la ventana (1.3 s en escritorio y
  2.9 s en el navegador con 522 páginas). Revisar con 386 circuitos quedó en
  7.8 ms por cuadro en R7.
- Etapas 7 y 8: la interfaz adaptativa para escritorio, tableta y teléfono
  quedó hecha con el rediseño 2026 (R1 a R7, decisión 0034); falta la prueba
  en un teléfono real (abajo).
- Por decidir: editar celdas directo en la tabla y plantillas de circuito.
- Probar la interfaz en un teléfono y una tableta reales, iOS Safari incluido
  (lista en la sección 9 de `rediseno-2026-r7-retiro-y-cierre.md`).
- Rehacer el video de la guía del sitio con la interfaz nueva.
- Confirmar con la NMX-J-136-ANCE-2019 los símbolos del unifilar (cotejados
  con la edición 2007) y con el texto de la NMX-J-116-ANCE la tabla de
  impedancias (transcrita de una lámina). Agregar la de transformadores tipo
  pedestal (NMX-J-285).
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

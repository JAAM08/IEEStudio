# IEEStudio — Plan de trabajo hacia la 1.0

**Revisión del plan:** 12 de noviembre de 2026.  
**Rama base revisada:** `main`, commit `d4665fbcf638002670184280ca844c20772d458c` (PR #11).  
**Objetivo:** cerrar las observaciones técnicas de Opus/Fable, estabilizar la entrega de escritorio y navegador, y después definir e implementar una única interfaz adaptativa antes de publicar la 1.0.

Este documento reemplaza la secuencia anterior. Claude debe leerlo completo, revisar el estado real de `main` antes de cada etapa y trabajar una etapa a la vez. No debe dar por hecho que una tarea sigue pendiente si ya está integrada. Tampoco debe adelantar el rediseño visual: primero se estabilizan el modelo, los resultados y la publicación.

## 1. Decisiones del dueño del proyecto

1. IEEStudio debe ser utilizable en escritorio, tablet y teléfono antes de la 1.0.
2. Debe existir **una sola interfaz adaptativa**, no dos aplicaciones o modos separados llamados «Guiado» y «Avanzado». El usuario nuevo debe encontrar un recorrido sugerido; el usuario experto debe poder trabajar sin pasos forzados. Se conserva un solo modelo `Doc`, una sola selección y una sola fuente de resultados.
3. El flujo Proyecto → Tableros → Circuitos → Revisar → Memoria se conserva como orientación, checklist o navegación contextual, pero no como una segunda interfaz independiente ni como un asistente que obliga a avanzar en orden.
4. El unifilar nuevo (formato de plano de la decisión 0022) debe verse igual en escritorio y navegador. No se acepta que la versión web muestre un renderer anterior.
5. No se publicará una nueva Release mientras las observaciones críticas de Fable sigan abiertas.
6. Los cambios de cálculo deben ser técnicamente trazables y verificables. No se inventan datos ni valores normativos. Si el método no permite concluir, el resultado debe decirlo.
7. El rediseño visual se revisa y aprueba antes de implementarlo. La dirección acordada es una interfaz profesional, adaptable y de densidad equilibrada, con carbón + amarillo eléctrico + IBM Plex; referencias de refinamiento Apple, herramientas técnicas Autodesk y organización Adobe, sin copiar literalmente sus interfaces.
8. El dueño revisa el comportamiento con explicaciones en español, enlaces y capturas; no necesita revisar código.

## 2. Estado real comprobado en GitHub

### 2.1 Base de código

- Repositorio: `JAAM08/IEEStudio`; rama principal `main`.
- Rust + egui/eframe; escritorio y WebAssembly comparten `ieestudio-app`.
- El PR #11 está integrado en `main` en `d4665fb`; incluye las decisiones 0026, 0027 y 0028, fuente única/topología, estados y trazabilidad, y propagación de motores por zonas.
- La auditoría de Opus documenta 20 hallazgos; 16 marcados corregidos y cuatro pendientes. Ver `docs/auditoria-modelo-y-falla.md`.
- La auditoría independiente de Fable se conserva como `docs/auditoria-fable-pr11.md`. Su dictamen es **APROBADO CON OBSERVACIONES**: confirma las 16 correcciones, pero identifica 18 hallazgos adicionales o persistentes (H-01 a H-18), con tres prioritarios.
- Las decisiones 0024 y 0025 (sistema/conexión y claridad del estado propio/capacidad interruptiva por tablero) ya aparecen en el roadmap como trabajo hecho. Verificar su estado en el código; no rehacerlas sin un fallo reproducible.

### 2.2 CI, instalador, sitio y Releases

La ejecución de GitHub Actions **43148932851**, asociada al commit `d4665fb`, terminó correctamente:
- pruebas, formato y clippy en Linux y Windows;
- compilación de ejecutables;
- generación y prueba automatizada del instalador de Windows;
- clippy y compilación WebAssembly;
- generación del sitio, artefacto `ieestudio-sitio` y despliegue de GitHub Pages.

Esto demuestra que el commit se compiló y que el instalador se generó y pasó la prueba automatizada de instalación/arranque en CI. **No significa que el instalador esté publicado como Release ni que Antonio haya validado la aplicación manualmente.**

La última Release visible al revisar el repositorio sigue siendo `v0.3.0-alpha.3`, publicada el 14 de octubre de 2026. Contiene instalador y paquete del sitio, pero no los cambios del PR #11. No crear una etiqueta o Release nueva hasta cerrar las etapas técnicas y verificar la entrega.

El flujo de CI actual funciona así:
- push a `main`: pruebas y despliegue de ieestudio.com;
- `workflow_dispatch`: permite volver a ejecutar el workflow y publicar el sitio;
- etiqueta `v*`: crea/actualiza la Release y adjunta el instalador; el job web adjunta también el ZIP del sitio.
- Los artefactos de Actions caducan; una Release conserva los binarios y ZIP publicados.

Archivos relevantes: `.github/workflows/ci.yml`, `tools/sitio-web.sh`, `website/README.md`, `installer/ieestudio.iss`, `crates/ieestudio-app/web/`.

### 2.3 Unifilar de la aplicación web

En el código de `main`:
- El renderer del unifilar vive en `crates/ieestudio-diagrams/src/lib.rs`.
- `crates/ieestudio-app/src/panels/mod.rs` llama a `single_line_with` y guarda el resultado en caché por revisión y modo.
- `guided.rs` solicita el diagrama a `shared.diagram(doc, shared.diagram_mode)`.
- La aplicación web se compila desde el mismo crate `ieestudio-app` y enlaza el mismo crate `ieestudio-diagrams`; no se encontró un renderer separado exclusivo de WebAssembly en las búsquedas realizadas.

Por eso, si el navegador muestra el unifilar de la primera versión mientras el escritorio muestra el nuevo, **no asumir que falta copiar un archivo de diseño**. Primero demostrar cuál build y renderer está ejecutando. Posibles causas que se deben discriminar: versión WebAssembly antigua, caché del navegador/service worker, ruta/panel que no usa el renderer compartido, modo Revisión/Plano confundido, o un defecto de integración. No declarar resuelto con solo recompilar.

La aplicación tiene un service worker (`crates/ieestudio-app/web/sw.js`) que crea una caché con versión derivada del hash del WASM. La prueba web debe verificar actualización real en una instalación PWA existente y en una sesión limpia.

## 3. Reglas obligatorias para Claude

1. **Leer antes de tocar:** `CLAUDE.md`, `docs/architecture.md`, `docs/engineering-methodology.md`, `docs/roadmap.md`, `docs/auditoria-modelo-y-falla.md`, `docs/auditoria-fable-pr11.md`, decisiones 0017, 0020, 0022–0028 y las decisiones pertinentes al módulo que se cambie.
2. **No tocar `main` directamente.** Una rama por etapa o subetapa, un PR dirigido a `main`. No apilar PRs dependientes entre sí. Esperar a que se integre la etapa previa.
3. **No cambiar cálculos por intuición.** Toda decisión de ingeniería o normativa debe tener una decisión documentada, hipótesis explícitas, fuente identificable y pruebas. Cuando requiera aprobación de Antonio, presentar alternativas, impacto y recomendación antes de implementar.
4. **No inventar valores.** Datos ausentes siguen ausentes. Los valores normativos viven en `data/<perfil>/*.csv`; no codificar tablas o factores en la interfaz ni en el motor.
5. **Modelo único.** La interfaz, CLI, memoria, Excel, DXF y WebAssembly deben consumir el mismo proyecto y los mismos resultados. No duplicar motores ni reglas en UI.
6. **Compatibilidad de archivos:** toda migración debe leer proyectos anteriores, conservar sus datos y emitir un diagnóstico claro. No prometer que versiones antiguas abren archivos nuevos salvo que exista prueba real. Nunca descartar silenciosamente campos desconocidos o datos inválidos.
7. **Escritorio y navegador son paridad funcional.** Todo cambio de modelo, cálculo, diagrama, exportación o interacción debe probarse en ambos targets. Acceso a archivos por el módulo `files`, según decisión 0017; no usar `std::fs` ni `rfd` en rutas WebAssembly.
8. **Pruebas obligatorias antes de cada commit:**
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo clippy --target wasm32-unknown-unknown -p ieestudio-app -- -D warnings`
   - compilación release de escritorio y WebAssembly cuando aplique.
9. **Pruebas visuales:** capturas de tema claro y oscuro, tamaños 1440×900, 1280×800, 1024×768, tablet 768×1024 y teléfono 390×844. Revisar las imágenes antes de enviarlas. No basta con que compile.
10. **No usar el equipo de Antonio por control remoto.** Entregar instaladores, enlaces y pasos breves para que él pruebe.
11. **Reportar cada etapa en español:** comportamiento visible, archivos o módulos importantes, pruebas ejecutadas y resultado, capturas, riesgos y pendientes, enlace al PR.
12. **No modificar releases o publicar versiones** como efecto secundario de un PR. Solo la etapa de publicación puede crear etiqueta/Release y requiere autorización expresa de Antonio.
13. **No iniciar el rediseño final** hasta que las etapas 1–4 estén cerradas y el dueño confirme el cierre técnico.
14. Si el alcance crece, registrar el hallazgo y priorizarlo; no esconderlo ni introducir una refactorización masiva sin PR separado.

## 4. Plan de ejecución

### Etapa 0 — Congelar la línea base y preparar la auditoría

**Objetivo:** dejar claro qué commit se está corrigiendo y hacer reproducibles los hallazgos.

1. Confirmar que el checkout está en `main` y parte de `d4665fb` o de su descendiente.
2. Revisar el informe Opus y el informe Fable. Marcar cada hallazgo como: corregido, pendiente, no reproducido o requiere decisión de Antonio.
3. Incorporar `docs/auditoria-fable-pr11.md` al repositorio y enlazarlo desde `docs/roadmap.md` y `docs/decisions/README.md` si procede.
4. Preparar fixtures mínimos para H-01 a H-18 y para los pendientes 17–20 de la auditoría de Opus.
5. Guardar una copia de referencia de los proyectos antiguos y actuales usados para probar compatibilidad.
6. Registrar una tabla inicial de pruebas, tiempos y capturas de escritorio/WebAssembly.

**Listo cuando:** cada hallazgo tiene un caso reproducible o una nota de por qué no se reproduce; se conoce el SHA de la base y se puede repetir la batería de pruebas.

### Etapa 1 — Cerrar primero los riesgos de cálculo y de dictamen

**Prioridad P0. No iniciar rediseño antes de completar esta etapa.**

#### 1A. H-01 — Falla fase-neutro en sistemas 1F-3H 240/120 V

El informe Fable demuestra que el programa puede mostrar 3.86 kA y «Cumple» para un interruptor de 5 kA, aunque el escenario de fase-neutro podría ser 5.1–6.4 kA. La etiqueta «cota superior» no es defendible si no se ha evaluado esa condición.

- Revisar el método aplicable a transformadores monofásicos con derivación central y los casos de falla fase-fase y fase-neutro.
- No introducir automáticamente un multiplicador 1.5 como si fuera un requisito normativo. Fable lo propone como recomendación de ingeniería y cita el método punto a punto Bussmann; hay que cotejar la fuente, sus condiciones y la base del cálculo.
- Recomendación provisional: hasta que Antonio apruebe el método y se verifique su fuente, el estudio de capacidad interruptiva de 1F-3H debe ser **No concluyente** cuando la fase-neutro pueda gobernar. No debe dar «Cumple» ni «Conforme» por una falla fase-fase menor.
- Si se aprueba una estimación conservadora, guardarla en el perfil con fuente y limitaciones, no como constante escondida.
- Pruebas: el caso de 25 kVA, 3 %, 240/120 V; acometida y transformador intermedio; interruptores de 5 kA y 10 kA; verificar el estado del resultado y el texto de trazabilidad.

#### 1B. H-02 — Tensión de referencia del dato de falla de CFE/suministradora

- El modelo debe identificar la tensión en la que está expresada la corriente de falla disponible.
- Si el dato corresponde a media tensión, no copiar sus kA directamente al secundario. Modelar el paso por el transformador con impedancias y relación de tensiones correctas.
- Validar el dato contra la impedancia y capacidad del transformador cuando ambos estén disponibles; detectar contradicciones y explicar qué dato prevalece o por qué el estudio queda incompleto.
- No inferir que un dato de 5 kA está en baja tensión si el usuario no lo especificó.
- Pruebas: dato de 5 kA en el primario de 13.2 kV con transformación a 480 V; dato expresado en el secundario; dato superior a la falla de bus infinito; ausencia de tensión de referencia.

#### 1C. H-03 — Cargas sospechosas de ser motores

- Si `review.rs` identifica una carga que parece motor o compresor, pero el usuario la dejó como «Otra carga», no se debe dar un resultado concluyente de capacidad interruptiva si esa aportación puede modificarlo.
- La confirmación «Es otra carga» debe persistir dentro del proyecto, tener trazabilidad y eliminar el aviso en Revisar, memoria y CLI.
- Si el usuario confirma que sí es motor, ofrecer la conversión conservando datos y recalcular.
- El estado de la falla debe distinguir entre sospecha sin resolver, motor confirmado sin datos y motor confirmado con datos.
- Pruebas: caso Bomba 50 hp capturada como Otra carga; misma carga convertida a Motor; confirmación persistida al guardar/cerrar/reabrir.

#### 1D. H-04 y H-05 — Coherencia de conexiones, sistemas e impedancias

> **Decidido en la 0030** (12 de noviembre de 2026): topología declarada, relación de tensiones con 1 % del perfil, conexión contradictoria como «Dato inválido» y carga no lineal de tres estados entran en 1D. El intervalo de impedancia, la nota 3 y los bancos de monofásicos (N-1 a N-3) esperan la tabla de la NMX-J-116-ANCE-2017.

- Unificar el criterio de trifásico en `System::has_three_phases` o función equivalente; eliminar umbrales contradictorios en `fault.rs`, `system.rs` y `calc.rs`.
- Detectar una conexión de transformador que contradiga el sistema eléctrico del punto correspondiente.
- Detectar impedancias fuera del rango válido de la tabla o clase de equipo; no inventar límites: deben venir de fuente aprobada y estar documentados.
- Pruebas: relación de tensión 1.88; relación 2.1; transformador monofásico asociado a sistema 220Y/127 V; impedancia anómala; cada caso debe mostrar error/advertencia con estado correcto y no generar un «Cumple» falso.

> **Ajuste del 12 de noviembre de 2026 (aprobado por el dueño).** El
> objetivo de la revisión de cortocircuito es elegir bien los interruptores,
> con el menor número de datos. Por eso:
>
> - **1E se reduce** a H-07, H-10 y H-11: dictamen y textos, sin datos nuevos.
>   H-06 (capacidad del transformador propio) queda en pausa porque agrega
>   la pregunta «¿es propio o de la suministradora?».
> - **N-1 a N-3** de la 0030 (intervalo de la NMX-J-116, nota 3 y bancos de
>   monofásicos) quedan en pausa: son casos poco comunes. Mientras tanto,
>   esos casos piden la impedancia de placa.
> - **Nueva etapa 1F: falla en cada tablero descontando los conductores**
>   (método punto a punto, decisión 0031, aceptada e implementada). No pide
>   datos nuevos y es la parte que más ayuda a elegir interruptores.
> - **Cálculo de cortocircuito opcional por proyecto** (pedido del dueño, 12
>   de noviembre): encendido por omisión; apagado, la capacidad
>   interruptiva queda «fuera de alcance» y la memoria lo anota.
> - **El alcance de cortocircuito se congela después de 1F.** Un caso que no
>   cubra queda «No concluyente: requiere estudio de cortocircuito».
> - **En el rediseño de interfaz:** la captura del cortocircuito se reduce a
>   «tengo el dato de la suministradora» o «tengo el transformador», con lo
>   demás oculto hasta que haga falta, y la carga no lineal se pregunta al
>   crear el alimentador. Antes del rediseño, una prueba de uso con alguien
>   que no conozca el programa y una campaña de validación contra cálculos
>   de ingenieros.

#### 1E. H-06, H-07, H-10 y H-11 — Capacidad, dictamen y presentación consistente

- H-06: evaluar la capacidad del transformador de acometida cuando sea equipo propio y haya datos de carga y placa suficientes. Distinguir claramente transformador de CFE y transformador del usuario. No emitir verificación si faltan datos.
- H-07: el dictamen debe usar «Conforme en lo evaluado» y explicar el alcance. No afirmar cumplimiento integral de NOM-001-SEDE-2012 por el hecho de que las reglas implementadas hayan pasado.
- H-10: el unifilar debe mostrar el origen real de la falla por zona. Si no hay estimación, mostrar «Icc SIN ESTIMAR» y la causa; no imprimir «dato de suministradora», «bus infinito» o «impedancia reducida 10 %» si no corresponde al caso.
- H-11: conservar «Dato inválido» como tal en Revisar, propiedades, memoria, unifilar y CLI; no convertirlo en «Sin evaluar» ni «Fuera de alcance».
- Pruebas cruzadas: misma instalación y dato inválido inspeccionados en interfaz, PDF, DXF/diagrama y CLI.

**Listo cuando:** ningún escenario P0 produce «Cumple» o «Conforme» con una falla potencialmente subestimada, con dato de referencia ambiguo o con motor sospechoso sin resolver. Todos los outputs expresan el mismo origen y estado.

### Etapa 2 — Completar los pendientes de la auditoría Opus y los hallazgos menores de Fable

**Objetivo:** cerrar el modelo de conexiones, los estados y la integridad de captura.

#### 2A. Pendientes 17–20 de `docs/auditoria-modelo-y-falla.md`

1. **17 — Conexión propia del secundario del transformador.** Revisar el modelo `Panel.transformer`: el secundario no debe heredar automáticamente la conexión del primario si el transformador puede tener otra configuración. Añadir el dato explícito donde proceda, validaciones y migración de proyectos.
2. **18 — Falla fase-neutro 120/240 V.** Se resuelve dentro de H-01; no crear dos métodos independientes.
3. **19 — Datos iniciales de circuitos/alimentadores.** Eliminar valores automáticos supuestos de 1,000 W y 10 m. Campos desconocidos vacíos; longitud 0 m debe rechazarse o tener significado explícito aprobado. Un circuito nuevo no puede aparentar cálculo completo sin datos.
4. **20 — Estados de conductor, protección, caída y tierra.** Migrar a estados de evaluación diferenciados: Válido, No concluyente, Sin evaluar y Dato inválido, sin confundirlos con Cumple/No cumple. La verificación normativa y el veredicto permanecen separados.

#### 2B. Hallazgos menores H-12 a H-18

- H-12: unificar el conteo de motores incompletos a través de transformadores y en la memoria.
- H-13: corregir la referencia de `references.csv` que todavía dice «en cada tablero».
- H-14: agregar fixtures de referencia para las reglas nuevas de propagación y transformadores; dejar `reviewed_by` vacío hasta revisión humana real, no inventar una firma.
- H-15: corregir la nota de compresores que omite que su corriente también entra al estudio de falla.
- H-16: detectar transformadores 1:1 comparando las tensiones de la conexión real, incluido 127 → 127 V.
- H-17: cuando un valor introducido se rechace (cero, negativo o texto), marcar el campo y explicar por qué; no ignorarlo silenciosamente.
- H-18: documentar la suma de magnitudes de impedancia y su diferencia menor de 0.2 %; evaluar si debe calcularse impedancia compleja. No llamar «cota superior» a un método que no garantice matemáticamente esa propiedad.

#### 2C. Reglas de captura

- Circuitos y alimentadores nuevos deben nacer sin potencia ni longitud asumidas.
- Longitud cero: decidir si representa conexión sin tramo, dato faltante o valor inválido; no tratarla indistintamente.
- Los proyectos anteriores deben abrir sin perder valores existentes; si un valor por omisión antiguo estaba guardado, no borrarlo automáticamente.
- Revisar el resto de formularios para evitar pedir datos que el sistema ya deduce.

**Listo cuando:** todos los hallazgos 17–20 y H-12–H-18 están corregidos o explícitamente diferidos con motivo y aceptación de Antonio; los estados se conservan igual en todas las salidas.

### Etapa 3 — Decisiones de ingeniería que debe aprobar Antonio

Claude debe preparar una nota breve por decisión con alternativas, pros/contras, ejemplo numérico, referencias exactas y recomendación. **No debe elegir silenciosamente por el usuario.**

| Decisión | Recomendación inicial para Antonio | Condición antes de implementar |
|---|---|---|
| D-1 Falla fase-neutro | Recomendación conservadora: marcar No concluyente mientras no se valide el método. Evaluar el factor 1.5 citado por Fable, sin asumir que es norma. | Revisar la fuente Bussmann, sus hipótesis y el método aplicable al sistema 1F-3H. |
| D-2 Tensión de falla CFE | Capturar explícitamente la tensión de referencia y transformar el dato entre primario/secundario. | Definir qué datos de transformador son obligatorios y cómo se trata un dato ambiguo. |
| D-3 Sospecha de motor | No concluyente hasta que el usuario clasifique; guardar confirmación. | Aprobar texto y persistencia de «Es otra carga». |
| D-4 Falla en transformador intermedio | No cambiar aún la decisión 0028. Comparar bus infinito local con propagación de la falla aguas arriba y determinar cuál es conservadora y físicamente consistente. | Presentar cálculo cotejado para el caso de 72.9 kA frente a 9.49 kA indicado por Fable, explicar bases y supuestos, y aprobar el método. |
| D-5 Transformador de acometida | Verificar capacidad del transformador propio; distinguirlo del equipo de CFE. | Definir dato «propio/CFE», potencia de carga aplicable y criterio normativo. |
| D-6 Dictamen | Adoptar «Conforme en lo evaluado» siempre. | Aplicar en UI, PDF, CLI y cualquier exportación. |
| D-7 Formato `.iees` | Migrar a versión 4 si los campos nuevos no son legibles por versiones previas. | Añadir migración v3→v4, pruebas de archivos históricos y documentar compatibilidad hacia atrás/adelante sin promesas falsas. |
| D-8 NMX-J-116 | Cotejar tabla de impedancias y tolerancia con el documento normativo original. | No afirmar validación normativa hasta tener revisión profesional y referencia trazable. |

También deben revisarse con fuente vigente NOM-001-SEDE-2012, NMX-J-116, NMX-J-136-ANCE-2019 e IEC 60909 cuando aplique. La aprobación de un algoritmo por pruebas matemáticas no equivale a aprobación normativa.

**Listo cuando:** Antonio aprobó las decisiones que afectan el método, y cada decisión aceptada está registrada en `docs/decisions/` y `docs/roadmap.md`.

### Etapa 4 — Verificación del unifilar web y de la cadena de publicación

**Prioridad alta. Resolver antes del rediseño final.**

El reporte de Antonio: al abrir IEEStudio en el navegador, el unifilar todavía parece el de la primera versión, mientras que el diseño nuevo se trabajó en escritorio.

#### 4A. Diagnóstico reproducible

1. Confirmar SHA de `main`, ejecución de Actions y URL de Pages publicada.
2. En el mismo proyecto de prueba, abrir el unifilar en escritorio y en navegador con el mismo modo (Revisión o Plano).
3. Añadir temporalmente una identificación de build visible en «Acerca de» o en diagnóstico técnico: versión, SHA corto y fecha/identificador de build. No mostrar texto de desarrollo al usuario final en la Release.
4. Verificar que la ruta web usa `Shared::diagram` y el mismo `single_line_with` de `ieestudio-diagrams`; comprobar que no exista una ruta alternativa o una imagen estática confundida con el diagrama vivo.
5. Comparar el resultado de `ieestudio-diagrams` y sus elementos/hotspots entre targets para el mismo fixture.
6. Revisar el caché `diagram_cache`: invalidación por `Doc::revision`, modo y opciones. Evitar que la revisión o modo quede obsoleto.
7. Probar en navegador limpio, navegador con service worker previo, PWA instalada y después de actualizar. Registrar qué versión de `ieestudio-app_bg.wasm` y `sw.js` se descarga.
8. Comprobar que el service worker instala la caché nueva y elimina la anterior; si es necesario, implementar una estrategia de actualización que no deje al usuario ejecutando indefinidamente el WASM anterior.
9. No borrar datos de proyectos del usuario al actualizar la PWA. La invalidación debe afectar recursos de aplicación, no almacenamiento local de proyectos.

#### 4B. Corrección y aceptación

- Si era caché, corregir versión/activación y documentar cómo forzar actualización segura.
- Si era un build antiguo, corregir pipeline/despliegue y añadir verificación del SHA publicado.
- Si era un renderer diferente, eliminar la duplicación y conectar la web al renderer compartido.
- Si era el modo equivocado, aclarar selector y persistencia de Revisión/Plano.
- Si el renderer compartido produce el diagrama nuevo pero visualmente sigue siendo la primera versión, revisar el código del renderer y el commit de la decisión 0022; no sustituirlo con capturas ni imágenes estáticas.

**Listo cuando:** para el mismo fixture, escritorio y navegador muestran el mismo diagrama actualizado; una PWA que tenía instalada la versión anterior recibe la nueva tras el ciclo normal de actualización; las capturas y el SHA desplegado quedan en el PR.

### Etapa 5 — Rendimiento y robustez

Partir de mediciones antes de optimizar. Usar un generador de proyecto de prueba reproducible: 12 tableros, 3 niveles, 400 circuitos, motores y transformador.

Medir:
- tiempo de cálculo del proyecto;
- duración de un cuadro de interfaz y del render del unifilar;
- tiempo y memoria de generación de PDF;
- tamaño del WASM comprimido y sin comprimir;
- tiempo de carga inicial y de reapertura desde caché.

Acciones:
1. Mantener el unifilar cacheado por revisión y opciones; no reconstruirlo en cada cuadro.
2. Dibujar solo elementos visibles en diagramas y tablas cuando el tamaño del proyecto lo amerite.
3. Evitar copias de resultados dentro de funciones de dibujo.
4. Evitar recálculos repetidos al editar; medir primero y evaluar cálculo diferido o parcial solo si los números lo justifican.
5. Generación PDF: escritorio fuera del hilo UI si el backend lo permite; navegador sin bloquear la interacción, con progreso/estado. No afirmar que existe Web Worker si Typst o dependencias no lo soportan.
6. Reutilizar plantillas y recursos PDF; cachear por revisión/opciones.
7. Dividir módulos grandes en cambios mecánicos separados, sin mezclar refactor con correcciones de cálculo.
8. No introducir optimizaciones que cambien resultados.

Metas iniciales a validar, no promesas:
- interfaz estable a 60 fps en proyecto de prueba, evitando trabajo sostenido superior a 16 ms por frame;
- PDF en escritorio por debajo de 5 s en el proyecto de referencia o documentar el límite;
- ninguna generación/exportación debe congelar el navegador sin mensaje de progreso;
- registrar tamaño de WASM y carga real antes/después.

**Listo cuando:** PR incluye tabla antes/después, pruebas y capturas, y cada meta está marcada alcanzada o no alcanzada con evidencia.

### Etapa 6 — Entrega de escritorio, web y versiones

**No crear Release hasta que las etapas 1–5 estén aceptadas.**

1. Comprobar que CI ejecuta pruebas Windows/Linux, clippy WebAssembly, build web, build release y prueba de instalador.
2. Validar el instalador generado: nombre, versión, firma si la hubiera, accesos directos, asociación `.iees`, desinstalación y actualización sobre la versión anterior.
3. Probar instalación limpia y actualización desde `v0.3.0-alpha.3` en una máquina de prueba Windows. Confirmar que los proyectos y preferencias sobreviven.
4. Probar ejecutable portátil si se distribuye por separado; documentar requisitos.
5. Generar sitio con `tools/sitio-web.sh`; probar /app, PWA, archivos, PDF, Excel, DXF, proyecto local y offline.
6. Verificar las páginas informativas y actualizar las capturas/guía solo cuando la interfaz se haya estabilizado.
7. Preparar notas de versión con correcciones, limitaciones y cambios de formato.
8. Confirmar que los artefactos correctos se adjuntan a la Release: instalador Windows, ZIP del sitio web y ejecutables adicionales si se decide publicarlos.
9. Crear etiqueta nueva solo con aprobación expresa de Antonio. No usar una etiqueta existente para reemplazar binarios publicados salvo decisión explícita y justificada.

**Listo cuando:** Antonio instala la versión candidata, abre un proyecto antiguo, crea uno nuevo, calcula, revisa el unifilar y genera PDF/Excel/DXF; el navegador refleja la misma versión funcional.

### Etapa 7 — Definición y aprobación del rediseño de interfaz (NO IMPLEMENTAR TODAVÍA)

Esta etapa comienza únicamente después de que Antonio acepte el cierre técnico de las etapas 1–6 o autorice explícitamente adelantar la propuesta visual sin tocar el código.

#### Principio de producto

**Una interfaz adaptativa, un solo modelo y una sola experiencia.**
- No mantener dos modos completos Guiado/Avanzado.
- El flujo Proyecto → Tableros → Circuitos → Revisar → Memoria funciona como orientación no lineal, indicadores de avance y accesos contextuales.
- El usuario experto puede saltar directamente al tablero, circuito, revisión, unifilar o memoria.
- Revelación progresiva: los datos esenciales visibles; opciones avanzadas y procedimientos disponibles sin saturar la pantalla.
- La interfaz debe ser una herramienta profesional de ingeniería, no un formulario de cinco pantallas independientes.

#### Composición de escritorio

- Barra superior: proyecto, guardar/estado, deshacer/rehacer, búsqueda/comandos, navegación contextual.
- Navegador izquierdo: jerarquía de instalación, tableros, alimentadores, circuitos y documentos.
- Área central: vista de trabajo activa (cuadro de cargas, tablero, unifilar, procedimiento o memoria).
- Inspector derecho: Datos, Resultados y Normativa del elemento seleccionado.
- Panel inferior plegable: Revisar, trazabilidad, mensajes y observaciones.
- Paneles acoplables pueden seguir usando egui_dock donde aporte valor; no duplicar pantallas ni estado de proyecto.

#### Tablet

- Navegador plegable.
- Inspector como panel lateral/deslizante.
- Canvas central prioritario.
- Acciones táctiles con tamaños adecuados; conservar tabla donde quepa y adaptar a tarjetas cuando no.

#### Teléfono

- Una columna, sin escritorio encogido.
- Navegación contextual compacta, no una segunda aplicación guiada.
- Listas/tarjetas para circuitos; ficha de elemento a pantalla completa o panel modal; acciones primarias visibles.
- Unifilar con zoom/pan táctil y selección de elementos.
- Cálculo rápido con datos arriba y resultado abajo.
- Campos numéricos con teclado numérico; controles táctiles objetivo de 44 px; ninguna acción depende de hover.
- El teclado virtual no tapa el campo activo.
- Archivos .iees, PDF y exportaciones usan las capacidades del navegador y muestran límites reales.
- PWA/offline mantiene datos del proyecto localmente y no los elimina al actualizar.

#### Pantallas y estados a diseñar

1. Inicio/proyecto y datos de obra.
2. Vista general de instalación y fuente.
3. Navegador de tableros y topología.
4. Tablero y alimentador.
5. Cuadro de cargas/circuitos.
6. Inspector de circuito.
7. Revisión de incidencias y dictamen.
8. Unifilar en modo Revisión y modo Plano.
9. Procedimiento y trazabilidad («Cómo se obtuvo»).
10. Memoria, vista previa y exportación.

Estados que deben aparecer en los diseños: vacío, incompleto, Válido, No concluyente, Sin evaluar, Dato inválido, Cumple, No cumple, fuera de alcance, aprobación profesional pendiente. No confundir estado de evaluación matemática con cumplimiento normativo.

#### Entregables antes de programar

- mapa de navegación y flujos;
- wireframes y mockups de escritorio/tablet/teléfono;
- sistema de diseño: tipografía, color, espaciado, tamaños, densidad, iconos, estados, accesibilidad;
- matriz de componentes egui reutilizables y correspondencia con módulos existentes;
- estados vacíos, errores y casos de contenido largo;
- criterios de aceptación y capturas de referencia en 1440×900, 1024×768, 768×1024 y 390×844;
- plan de migración desde `guided.rs` y `workspace.rs` evitando dos implementaciones.

**Antonio debe aprobar los mockups y la estructura antes de iniciar la implementación.**

### Etapa 8 — Implementación del rediseño adaptativo

1. Crear la base de tokens y componentes compartidos.
2. Crear un único shell adaptable y un solo modelo de selección.
3. Migrar el navegador, inspector y panel de revisión.
4. Integrar cuadro de cargas, tablero y formularios.
5. Integrar el unifilar actualizado, procedimientos y memoria.
6. Implementar tablet/teléfono con la misma fuente de datos.
7. Migrar las funciones exclusivas que sigan siendo necesarias (procedimiento completo, selección de columnas, exportaciones).
8. Retirar la antigua separación Guiado/Avanzado solo cuando todas las funciones estén disponibles y verificadas.
9. Actualizar la guía, capturas y video de sitio.
10. Pruebas de regresión en Windows, Linux y navegador.

**Listo cuando:** el mismo proyecto se puede crear, editar, revisar y exportar en escritorio, tablet y teléfono; no existen dos experiencias paralelas ni funciones esenciales accesibles solo desde una vista antigua.

### Etapa 9 — Uso diario, documentos y unifilar fase 3

Se conserva el alcance funcional original, pero cada función nueva nace ya integrada en la interfaz unificada y en teléfono/tablet.

#### Uso diario
- Revisar avisos agrupados sin duplicados.
- Tabla de circuitos cómoda a 1280 px y adaptable; elegir columnas; edición directa de nombre, carga, longitud y fases; navegación con Tab/Enter.
- Duplicar circuito y plantillas de cargas editables.
- Guardar en el proyecto las decisiones «Es otra carga» para que no regresen avisos resueltos.
- Proyectos recientes en escritorio y almacenamiento local en navegador.
- Un solo flujo claro para generar PDF; evitar botones duplicados.
- Importación/exportación Excel de ida y vuelta, conservando tipo de carga, datos de motor/placa, catálogo y posiciones. Prueba de ida/vuelta idéntica.

#### Cuadro de cargas
Usar `CUADRO_DE_CARGAS_CMAPAS.xlsx` como referencia visual:
- una hoja por tablero, horizontal;
- encabezado con nombre, ubicación, tipo de carga, tensión, marca/modelo;
- circuitos por renglón;
- columnas de tipos de carga con símbolo, watts/unidad y cantidad;
- resultados eléctricos, fases, ITM, conductores, longitud, caída y tubería;
- pie con carga instalada, datos del alimentador y desbalanceo;
- fórmulas vivas solo para los cálculos que el usuario pueda verificar; resultados normativos de IEEStudio identificados como resultados del programa;
- personalización de columnas, orden, unidades y textos.

Antes de implementar, Antonio debe decidir convivencia del catálogo con captura directa por watts, unidad predeterminada de conductores y regla de 180 VA por salida. No sustituir el método de caída de IEEStudio por fórmulas de Excel sin aprobación.

#### Membrete y plantilla
- Archivo independiente `.ieesplantilla`, importable/exportable.
- Empresa, contacto, responsable/cédula, logotipos, colores y tipografía.
- Portada, membrete, encabezado/pie, numeración, secciones y textos configurables.
- Vista previa de portada y página interior.
- Reutilizar la plantilla Typst; no duplicar el motor de reportes.
- La leyenda «No constituye aprobación profesional» no se puede quitar.
- El proyecto recuerda plantilla; si falta, usa genérica y avisa.
- Reducir y limitar imágenes para no inflar proyectos ni PDF.

#### Unifilar fase 3
- DXF con bloques nombrados y atributos editables (identificador, amperes, calibre, etc.).
- Capas con nombres claros.
- Hojas carta, doble carta, 60×90 y 90×120 cm horizontal, con marco, escala, simbología y cajetín.
- Diagramas grandes en varias hojas con referencias de continuidad.
- PDF del plano independiente.
- Pruebas abriendo el DXF en AutoCAD, LibreCAD u ODA; declarar qué programa se usó.
- Probar varios niveles, nombres largos y textos encimados.

### Etapa 10 — Cierre de 1.0

- Revisión profesional de los fixtures y sus referencias; nunca inventar `reviewed_by`.
- Cotejo de tablas y símbolos con los documentos normativos originales.
- Prueba con un proyecto real de Antonio contra su memoria original.
- Pruebas completas de escritorio, navegador y teléfono.
- Auditoría final independiente de cálculo y de interfaz.
- Actualizar `README.md`, `docs/roadmap.md`, guía, política de versiones y notas de Release.
- Publicar solo cuando Antonio autorice.

## 5. Preguntas que Claude debe presentar a Antonio (no todas de golpe)

Presentar únicamente la decisión que sea necesaria para continuar la etapa activa.

1. **D-1:** aprobar la metodología provisional/conservadora de falla fase-neutro o mantener No concluyente hasta validar la fuente.
2. **D-2:** cómo captura CFE la falla (tensión primaria/secundaria y datos disponibles).
3. **D-3:** texto y persistencia de confirmación «Es otra carga».
4. **D-4:** conservar bus infinito en transformadores intermedios o adoptar propagación aguas arriba, después de cotejo numérico.
5. **D-5:** distinguir equipo de CFE/propio y criterio de capacidad del transformador de acometida.
6. **D-7:** aceptar migración de formato .iees a versión 4.
7. **D-8:** facilitar/revisar fuente normativa de NMX-J-116.
8. Cuadro de cargas: watts directos + catálogo, mm²/AWG y regla de contactos de 180 VA.
9. Plantilla: portada/membrete de ejemplo.
10. DXF: programa CAD de referencia para prueba.

## 6. Cómo reportar cada etapa

Claude debe entregar un resumen con:
- qué cambió para el usuario final;
- qué hallazgos cerró (IDs H-xx y 17–20);
- qué decisiones fueron aprobadas y dónde están documentadas;
- pruebas ejecutadas y resultado real (no solo «todo pasa»);
- capturas de escritorio/tablet/teléfono cuando aplique;
- comparación de resultados numéricos y trazabilidad;
- compatibilidad de archivos y pruebas de migración;
- estado de CI y enlaces de Actions/PR;
- pendientes y riesgos conocidos.

**No usar «corregido» si solo se añadió una etiqueta o se ocultó un aviso.** La causa técnica debe estar resuelta o el resultado debe quedar explícitamente no concluyente.

## 7. Referencias del repositorio

- `docs/auditoria-modelo-y-falla.md`
- `docs/auditoria-fable-pr11.md`
- `docs/architecture.md`
- `docs/engineering-methodology.md`
- `docs/roadmap.md`
- `docs/decisions/0017-version-web.md`
- `docs/decisions/0020-flujo-guiado.md`
- `docs/decisions/0022-unifilar-como-plano.md`
- `docs/decisions/0023-falla-por-bus-infinito.md`
- `docs/decisions/0024-sistema-y-conexion.md`
- `docs/decisions/0025-estado-propio-y-capacidad-interruptiva-por-tablero.md`
- `docs/decisions/0026-fuente-unica-y-topologia.md`
- `docs/decisions/0027-estados-y-trazabilidad.md`
- `docs/decisions/0028-motores-y-propagacion-de-la-falla.md`
- `.github/workflows/ci.yml`
- `tools/sitio-web.sh`
- `website/README.md`

# Auditoría del modelo eléctrico y del estudio de falla: informe técnico

Noviembre de 2026. Revisé el modelo de conexiones, los transformadores, el
estudio de falla por bus infinito (decisión 0023), la aportación de motores y
la forma en que se informan los resultados incompletos. Los criterios
adoptados están en las decisiones 0026, 0027 y 0028. El trabajo está en el
PR #11.

> **Validación numérica y validación normativa son cosas distintas.** Que
> las pruebas pasen demuestra que el programa calcula lo que el método dice,
> y el cotejo independiente demuestra que lo calcula bien. **No** demuestra
> que el método ni sus factores (`k = 4`, tolerancia del 10 %) cumplan
> íntegramente la NOM-001-SEDE-2012 o la IEC 60909. Esa revisión la tiene
> que hacer un ingeniero con el texto vigente de las normas.

## 1. Cómo está armado el modelo

- **`Source.system`** es la tensión de servicio.
- **`Source.substation`** es el transformador de la acometida, de CFE o
  propio. Su secundario es `Source.system`, así que no se vuelve a capturar.
- **`Panel.transformer`** es un transformador **intermedio**:
  - su primario es el alimentador del tablero, a la tensión del tablero de
    donde sale;
  - su secundario fija la tensión de ese tablero y de todo lo que cuelga de
    él.
- **`connection::bus_system`** es la única función que deduce el sistema de
  unas barras. La usan el cálculo, la interfaz y el unifilar. Los circuitos
  no preguntan su sistema.
- **Estudio de falla.** Divide la instalación en zonas de tensión: la de la
  fuente y la del secundario de cada transformador intermedio.

## 2. Hallazgos

Gravedad:
- **Crítica:** el resultado puede quedar del lado inseguro o mostrar un
  número que no corresponde.
- **Alta:** se ocultan datos faltantes o se inventan.
- **Media:** inconsistencias que pueden volverse errores.
- **Baja:** limitaciones que solo se documentan.

Origen:
- **O:** hallazgo del plan original.
- **A:** hallazgo adicional de esta auditoría.

| # | Origen | Gravedad | Hallazgo | Dónde | Estado |
|---|---|---|---|---|---|
| 1 | A | Crítica | La falla de un transformador intermedio que alimenta una o dos fases usaba `S / (√3 · V)`: salía √3 veces menor (−42 %), del lado inseguro, y contradecía su propio dimensionamiento. | `fault.rs` | Corregido |
| 2 | O | Crítica | Todos los motores se sumaban en todas las barras, aunque estuvieran a otra tensión o detrás de un transformador. | `fault.rs`, `with_motors` | Corregido (0028) |
| 3 | O | Crítica | Un transformador con datos que no servían dejaba a su secundario con la falla del primario, que está a otra tensión. | `fault.rs` | Corregido |
| 4 | O | Alta | Valores inventados al activar el transformador (45 kVA, 3 %, 220Y/127 V, 3 m) o la acometida (112.5 kVA, 3 %, 13.2 kV). | `properties.rs` | Corregido (0026) |
| 5 | A | Alta | «Capturar» proponía 10 kA, 10 A, 225 A o 100 A en los datos de catálogo. Un motor nuevo nacía con 1 hp, así que nunca aparecía incompleto. | `widgets.rs`, `state.rs` | Corregido |
| 6 | O | Alta | Sin datos de la fuente se cancelaba todo el estudio (`source_fault(...)?`). | `fault.rs` | Corregido |
| 7 | O | Alta | Los errores de la acometida se descartaban (`.ok()?`). | `fault.rs` | Corregido |
| 8 | O | Alta | Los motores sin datos se omitían sin avisar. | `fault.rs` | Corregido |
| 9 | A | Alta | «Agregar tablero» lo creaba trifásico aunque el sistema fuera 240/120 V. | `state.rs` | Corregido |
| 10 | A | Media | Si fallaban la regla del 125 % del transformador o la 430-24, el tramo se dimensionaba sin ellas y sin avisar. El 125 % estaba escrito en el código como respaldo de la tabla. | `calc.rs`, `transformer.rs` | Corregido |
| 11 | A | Media | «Bus infinito» y «cota superior» aparecían también cuando la falla era el dato de la suministradora. | textos | Corregido |
| 12 | A | Media | El sistema de las barras estaba en tres copias de código. | `calc.rs`, `diagrams` | Corregido |
| 13 | O | Media | Ningún aviso de transformador capturado dos veces. | — | Corregido (0026) |
| 14 | O | Media | Las cargas que parecen motor solo las detectaba la vista guiada; no llegaban al resultado, la memoria ni la línea de comandos. | `guided.rs` | Corregido |
| 15 | A | Media | «Sin evaluar» mezclaba dato faltante, dato inconsistente y cota no concluyente. La etiqueta «No concluyente» se armaba en cada pantalla. | interfaz, memoria | Corregido en falla (0027) |
| 16 | A | Media | A un motor sin potencia, la verificación «Datos del equipo» le decía «dato inválido». | `calc.rs` | Corregido |
| 17 | A | Baja | El secundario de un transformador usa la conexión de su primario. | `connection.rs` | Pendiente |
| 18 | A | Baja | En 120/240 V la falla de fase a neutro puede superar a la de fase a fase. | 0023 | Pendiente |
| 19 | A | Baja | Se aceptan longitudes de 0 m, y los circuitos nuevos nacen con 1000 W y 10 m. | `state.rs` | Pendiente (rediseño de la captura) |
| 20 | A | Baja | Las verificaciones de conductor, protección, caída y tierra aún no usan los cuatro estados. | `calc.rs` | Pendiente (0027) |

## 3. Criterios implementados

- **0026. Fuente única y topología.**
  - Cada dato se captura una vez, en su nivel; sin valores supuestos.
  - Los dos transformadores tienen nombres distintos.
  - Una sola deducción de la tensión de cada barra.
  - Avisos de posible duplicado por topología y datos, con confirmación del
    usuario.
- **0027. Estados y trazabilidad.**
  - Cuatro estados: Válido, No concluyente, Sin evaluar y Dato inválido.
  - Cada falla de barras y cada lugar lleva su «Cómo se obtuvo»: qué se
    calculó, dónde, los datos, las hipótesis, lo que contribuye, lo que falta
    y los problemas.
  - Lo usan la interfaz, Revisar, la memoria y la línea de comandos sin
    repetir la lógica.
- **0028. Motores y propagación.**
  - Zonas de tensión.
  - Los motores de la misma zona aportan completos.
  - Los de un secundario llegan a través de la impedancia de su
    transformador: `Im = Ib₁ / (Zt + Ib₂ / Im₂)`.
  - Los de aguas arriba no se suman a un secundario por bus infinito.
  - Estudio por partes.

## 4. Decisiones que requirieron aprobación

- **Método de motores.** Se presentaron tres alternativas (0028). El dueño
  del proyecto eligió la A el 11 de noviembre de 2026.
  - La A se implementó antes de esa confirmación, en el mismo mensaje en que
    se presentaron las alternativas. El plan pedía esperar; no se fusionó
    nada antes de la aprobación.
- **Principio de captura única y cuatro estados.** Los fijó el dueño del
  proyecto en su plan de auditoría.

## 5. Archivos modificados

**Cálculo (`crates/ieestudio-project`)**
- `fault.rs`: zonas, motores por zona, estudio parcial, estados y
  trazabilidad.
- `evaluation.rs`: nuevo; estados y `Trace`.
- `review.rs`: nuevo; cargas que parecen motor y duplicados.
- `calc.rs`: errores con su causa, resultados nuevos, una sola deducción del
  sistema.
- `transformer.rs`: potencia opcional y 125 % solo de la tabla.
- `model.rs`: datos opcionales y `confirmed_distinct`.
- `motors.rs`: motor sin potencia.
- `verdict.rs`: fuera de alcance por lugar.

**Interfaz (`crates/ieestudio-app`)**
- `widgets.rs`: campos que empiezan vacíos y «Cómo se obtuvo».
- `properties.rs`: transformadores, acometida y falla.
- `guided.rs`: Revisar con duplicados, trazabilidad y falla incompleta.
- `state.rs`: tablero y motor nuevos.
- `browser.rs` y `schedule.rs`: textos.

**Salidas**
- `ieestudio-diagrams`: sistema de barras, «SIN DATO» e Icc incompleta.
- `ieestudio-reports`: memoria con trazabilidad, motores sin datos y avisos.
- `ieestudio-cli`: lo que falta, los problemas y los avisos.

**Documentación**
- Decisiones 0026, 0027 y 0028; 0023 y el índice actualizados.
- `roadmap.md`.

## 6. Pruebas y resultados

**Ejecutadas el 11 de noviembre de 2026:**
- `cargo fmt --all`, sin cambios pendientes.
- `cargo clippy --all-targets -- -D warnings`, sin avisos.
- `cargo clippy --target wasm32-unknown-unknown -p ieestudio-app -- -D warnings`,
  sin avisos.
- `cargo test --workspace`: **175 pruebas, 0 fallas**. De ellas, 38 son de
  `project_cases` y 25 de la app.

Escenarios del plan de auditoría (`tests/project_cases.rs`):

| Escenario | Prueba | Qué verifica |
|---|---|---|
| Transformador de CFE con datos suficientes | `a_cfe_transformer_with_enough_data_is_valid` | Válido, «Cumple», datos, hipótesis y lo que contribuye |
| Baja tensión con dato de falla | `low_voltage_service_with_the_supplier_fault` | 10.0836 kA; no concluyente con 10 kA y cumple con 22 kA; hipótesis «Dato de la suministradora» |
| Acometida y transformador propio aguas abajo | `service_transformer_and_an_own_transformer_downstream` | 11.6214 kA, 11.7274 kA y 4.4859 kA; sin aviso de duplicado; tensiones en la trazabilidad |
| Dos niveles de tensión con motores en ambos | `motors_contribute_by_voltage_zone` | 4.3739, 4.4859, 50.05 A y 25.10605 kA; sin motores de arriba en el secundario |
| Motor aguas abajo de un transformador intermedio | `motors_contribute_by_voltage_zone`, `voltage_and_fault_propagate_through_several_panels` | Llega a través del transformador, no por suma directa |
| Transformador intermedio con datos inválidos | `an_intermediate_transformer_with_invalid_data` | Impedancia 0 y potencia negativa: «Dato inválido», con el problema y sin la falla del primario |
| Transformador intermedio sin potencia | `a_transformer_without_power_is_not_evaluated` | Sin dimensionar y sin falla, con la causa |
| Sin dato de la suministradora y transformador propio abajo | `a_partial_study_evaluates_what_has_data` | TF-1 evaluado; fuera de alcance solo en la acometida y TGN |
| Sin ningún dato de falla | `no_fault_data_at_all`, `interrupting_rating_is_out_of_scope` | Sin estudio ni trazabilidad inventada; veredicto fuera de alcance |
| Motor sin potencia | `a_motor_without_power_makes_the_fault_incomplete` | Listado con lo que falta; no concluyente arriba y abajo |
| Varios motores, alguno incompleto | igual que el anterior | El motor con datos sigue aportando 0.112 kA |
| Posible duplicado | `a_transformer_with_equal_voltages_is_flagged`, `equal_ratings_are_flagged_only_beside_the_service` | Avisa y la confirmación lo quita |
| Misma potencia en otro lugar | `equal_ratings_are_flagged_only_beside_the_service` | No se marca |
| Relación 1:1 | `a_one_to_one_transformer_is_not_rejected` | Se avisa, se calcula (2.0047 kA) y confirmarlo no cambia los resultados |
| Propagación por varios tableros | `voltage_and_fault_propagate_through_several_panels` | T-3 a 220Y/127 V; circuito de 127 V con 10 A; motor de 15.2 A; 4.4347 kA |
| Transformador de dos fases | `a_two_phase_transformer_uses_its_single_phase_current` | 204.55 A y 7.5758 kA |

Otras pruebas: de la fórmula de transferencia en sus dos límites
(`fault.rs`), del módulo `review`, de `evaluation`, y de la app (tablero
nuevo y motor sin potencia).

### Cotejo independiente

Recalculé los valores de referencia con otro método: en ohms y no en por
unidad, sin el código del programa. Cada impedancia se refirió con el
cuadrado de la relación de transformación, y el grupo de motores se modeló
como `Zm = V / (√3 · k · ΣIpc)`.

| Caso | Prueba | Cotejo |
|---|---|---|
| Secundario de 45 kVA, 3 %, 220 V | 4.3739 kA | 4.3739 kA |
| Motores de 220 V (28 A) vistos en 480 V | 50.05 A | 50.052 A |
| Punto de conexión con 25 kA | 25.10605 kA | 25.10605 kA |
| Acometida de 500 kVA, 5.75 %, 480 V, con motores | 11.6214 / 11.7274 kA | 11.6214 / 11.7274 kA |
| TF-1 con un motor de 5 hp tres tableros abajo | 4.4347 kA | 4.4347 kA |
| 45 kVA a 220 V en dos fases | 7.5758 kA | 7.5758 kA |

Las corrientes de los motores (14 A, 28 A y 15.2 A) son las de las Tablas
430-250 en las columnas de 460 V y 230 V.

## 7. Limitaciones que permanecen

- **Impedancias no descontadas.** No se descuenta la de los alimentadores ni
  la de la red aguas arriba de un transformador intermedio. Todo es cota
  superior, y un interruptor que no la alcanza queda no concluyente, nunca
  «no cumple».
- **Motores lejanos.** Dentro de una zona se suman todos los motores, aunque
  uno lejano aporte menos.
- **Factores del perfil.** `k = 4` y la tolerancia del 10 % son criterios de
  ingeniería del perfil, no de la norma.
- **Hallazgos 17 a 20.** Siguen pendientes: conexión del secundario, falla de
  fase a neutro en 120/240 V, datos iniciales de la captura y los cuatro
  estados en el resto de los tramos.
- **Tablas por cotejar.** Falta cotejar la tabla de impedancias mínimas con
  el texto de la NMX-J-116-ANCE (decisión 0023).

## 8. Posibles regresiones y lo que conviene revisar

- **Archivos guardados.** Un proyecto que tenía los valores inventados (por
  ejemplo, 45 kVA) los conserva, porque ya están en el archivo. Conviene
  revisar esos transformadores.
- **Un dato vacío cambia resultados.** Un dato que antes se suponía y ahora
  queda vacío deja ese tablero sin evaluar donde antes había un número. Es
  intencional.
- **Estado del proyecto.** El veredicto puede pasar de «conforme» a
  «pendiente» en proyectos con motores sin potencia o con un transformador
  intermedio sin impedancia.
- **Fallas que cambian.** Con transformadores intermedios la falla cambia:
  - sube en los transformadores de una o dos fases (hallazgo 1);
  - baja en las barras donde antes se sumaban motores de otra tensión.
- **Revisión visual pendiente.** Revisé con capturas la ficha del tablero y
  Revisar. Falta revisar la memoria completa con un proyecto real del
  usuario.

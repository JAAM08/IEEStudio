# Registro de decisiones

Cada archivo registra una decisión: el contexto, lo que se decidió y sus
consecuencias. Una decisión no se cambia editando su archivo: se escribe una
nueva que la sustituye y se marca la anterior como «Sustituida por».

| Núm. | Decisión | Estado |
| --- | --- | --- |
| 0001 | [Rust y motor independiente de la interfaz](0001-rust-y-arquitectura.md) | Aceptada |
| 0002 | [Interfaz con egui](0002-interfaz-egui.md) | Aceptada |
| 0003 | [Proyecto en un archivo JSON](0003-archivo-json.md) | Aceptada |
| 0004 | [Licencia doble MIT o Apache-2.0](0004-licencia.md) | Aceptada |
| 0005 | [Criterios de cálculo](0005-criterios-de-calculo.md) | Aceptada |
| 0006 | [Memoria en PDF con Typst](0006-memoria-typst.md) | Aceptada |
| 0007 | [Unifilar como trazos independientes del dibujo](0007-unifilar.md) | Aceptada; ampliada por 0022 |
| 0008 | [Reglas y referencias normativas separadas](0008-referencias-normativas.md) | Aceptada |
| 0009 | [Veredicto en tres niveles](0009-veredicto.md) | Aceptada |
| 0010 | [Interfaz por paneles acoplables y espacios de trabajo](0010-interfaz-profesional.md) | Aceptada |
| 0011 | [Motores y motocompresores](0011-motores.md) | Aceptada |
| 0012 | [Carga de los alimentadores según el artículo 220](0012-carga-articulo-220.md) | Aceptada |
| 0013 | [Cortocircuito y capacidad interruptiva](0013-cortocircuito.md) | Sustituida por 0018 |
| 0014 | [Cuadro de cargas de ida y vuelta con Excel](0014-excel.md) | Aceptada |
| 0015 | [Conductores en paralelo propuestos por el programa](0015-paralelos.md) | Aceptada |
| 0016 | [Transformadores dentro del proyecto](0016-transformador.md) | Aceptada; cortocircuito retirado por 0018 |
| 0017 | [La aplicación en el navegador](0017-version-web.md) | Aceptada |
| 0018 | [El cortocircuito sale de IEEStudio y pasa a la suite](0018-cortocircuito-fuera-de-alcance.md) | Aceptada; ampliada por 0023 |
| 0019 | [Rediseño de la interfaz, fase 1: escritorio](0019-rediseno-fase-1.md) | Aceptada; vale dentro de la vista avanzada |
| 0020 | [Vista guiada en cinco pasos](0020-flujo-guiado.md) | Aceptada |
| 0021 | [Agregar un circuito eligiendo primero qué se conecta](0021-agregar-por-tipo-de-carga.md) | Aceptada |
| 0022 | [El unifilar con el formato de un plano eléctrico](0022-unifilar-como-plano.md) | Aceptada |
| 0023 | [Capacidad interruptiva con la falla estimada por bus infinito](0023-falla-por-bus-infinito.md) | Aceptada; motores y datos incompletos corregidos por 0028 |
| 0024 | [El sistema decide qué conexiones existen](0024-sistema-y-conexion.md) | Aceptada |
| 0025 | [El estado de un circuito es el de su cálculo; la capacidad interruptiva se revisa por tablero](0025-estado-propio-y-capacidad-interruptiva-por-tablero.md) | Aceptada |
| 0026 | [Fuente única de datos eléctricos y propagación por la topología](0026-fuente-unica-y-topologia.md) | Aceptada |
| 0027 | [Estados de evaluación, resultados parciales y trazabilidad](0027-estados-y-trazabilidad.md) | Aceptada; por ahora en el estudio de falla |
| 0028 | [Contribución de motores y propagación de la corriente de falla](0028-motores-y-propagacion-de-la-falla.md) | Aceptada |

El estado del proyecto y lo pendiente están en `../roadmap.md`. Las auditorías del
PR #11 están en `../auditoria-modelo-y-falla.md` (Opus) y
`../auditoria-fable-pr11.md` (Fable); las decisiones que esperan aprobación,
en `../decisiones-pendientes-d1-d8.md`.

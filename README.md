# VoltForge

Cálculo de instalaciones eléctricas de baja tensión, de código abierto.

VoltForge modela una instalación una sola vez y de ese modelo deriva los
cálculos, el cuadro de cargas, el diagrama unifilar y la memoria de cálculo.
El perfil normativo inicial es la NOM-001-SEDE-2012.

> **Estado: desarrollo temprano.** Hoy existe el motor de cálculo con sus
> tablas y pruebas. Todavía no hay interfaz gráfica ni instalador.

## Qué funciona hoy

| Cálculo | Detalle |
| --- | --- |
| Corriente nominal | Una, dos y tres fases, a partir de potencia y factor de potencia |
| Corriente de diseño | 125 % de la carga continua más 100 % de la no continua |
| Caída de tensión | Por impedancia con la Tabla 9, conductores en paralelo, ajuste opcional de resistencia por temperatura y comprobación con la fórmula exacta |
| Longitud máxima | Longitud con la que un tramo cumple un límite de caída |
| Conductor | Menor calibre que cumple terminales, temperatura ambiente, agrupamiento, protección y, si se pide, un límite de caída de tensión; indica qué criterio lo determinó |
| Interruptor | Capacidad normalizada inmediata superior a la corriente de diseño |
| Puesta a tierra | Tabla 250-122, con el incremento proporcional cuando el conductor de fase se sobredimensiona |
| Tubería conduit | EMT, IMC, RMC y PVC cédula 40 y 80; ocupación de 53, 31 y 40 % según el número de conductores, niples al 60 % y verificación de atascamiento |

Lo siguiente en el plan: modelo del proyecto, cuadro de cargas, línea de
comandos e interfaz gráfica.

## Probarlo

Requiere [Rust](https://rustup.rs) 1.80 o posterior.

```
cargo test                          # ejecuta los casos de referencia
cargo run --example procedimiento   # imprime dos cálculos paso a paso
```

## Organización

```
crates/voltforge-core/   motor de cálculo, sin dependencias externas
data/nom-001-sede-2012/  tablas de la norma en CSV, revisables en una hoja de cálculo
tools/                   extracción y validación de las tablas desde el texto de la norma
```

El motor no depende de ninguna interfaz, calcula con precisión completa y no
redondea pasos intermedios. Cuando una tabla no cubre un caso, devuelve un
error que explica qué falta en lugar de suponer un valor.

## Verificación

- Las tablas se validan por consistencia interna y se cotejaron contra una
  captura independiente. Ver [`data/nom-001-sede-2012/README.md`](data/nom-001-sede-2012/README.md).
- Cada cálculo tiene casos de prueba con resultado conocido en
  [`crates/voltforge-core/tests/`](crates/voltforge-core/tests/).

## Responsabilidad profesional

VoltForge asiste el cálculo; no sustituye el criterio ni la firma de un
ingeniero. El programa distingue entre un cálculo matemático, una comprobación
normativa y una aprobación profesional, y esta última siempre es de quien
firma el proyecto. Las tablas incluidas son una transcripción de trabajo: la
fuente con validez oficial es la norma publicada en el Diario Oficial de la
Federación.

## Agradecimientos

Al Ing. Juan Carlos Vega Mendoza, cuyas hojas de cálculo de caída de tensión y
de tubería sirvieron como referencia de lógica y como casos de prueba.

## Licencia

Doble licencia, a tu elección: [MIT](LICENSE-MIT) o [Apache-2.0](LICENSE-APACHE).

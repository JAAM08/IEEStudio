# Decisiones de diseño y estado del proyecto

Registro de lo que ya se decidió y por qué, para que cualquier persona o
sesión de trabajo pueda continuar sin volver a discutirlo.

## Propósito

VoltForge modela una instalación eléctrica de baja tensión una sola vez y de
ese modelo deriva cálculos, cuadro de cargas, diagrama unifilar y memoria de
cálculo. Es de código abierto, de escritorio (Windows primero) y manejable
también por línea de comandos y por agentes de IA.

## Decisiones técnicas

| Tema | Decisión |
| --- | --- |
| Lenguaje | Rust en todo el programa |
| Arquitectura | Motor de cálculo independiente de la interfaz; todo es un comando |
| Interfaz gráfica | egui / eframe; el mismo código debe servir para escritorio y navegador |
| Licencia | Doble, MIT o Apache-2.0 |
| Formato del proyecto | JSON legible, con `format_version` |
| Tablas normativas | CSV por perfil en `data/`, incrustadas al compilar |
| Memoria de cálculo | PDF con Typst; el motor entrega datos y una plantilla los compone |
| Idioma | Español en la interfaz y la documentación; identificadores de código en inglés |
| Perfil normativo inicial | NOM-001-SEDE-2012 |

## Criterios de ingeniería

- **Caída de tensión:** fórmula de la Tabla 9, `k·I·L·(R·cos θ + X·sen θ)/n`.
  La fórmula exacta se calcula como comprobación.
- **Ajuste de resistencia por temperatura:** opcional y apagado por defecto.
  Usa la fórmula de la Tabla 8, nota 2.
- **Cargas continuas:** solo la parte continua lleva el 125 %.
- **Selección de conductor:** el programa propone el menor calibre que cumple
  y dice qué criterio lo determinó. Un calibre fijado por el usuario se
  verifica, nunca se cambia.
- **Terminales:** 60 °C hasta 100 A y 75 °C arriba de 100 A.
- **Interruptores:** se omiten 16, 32 y 63 A por defecto (serie IEC).
- **Puesta a tierra:** Tabla 250-122, con el incremento proporcional de
  250-122(b) cuando el conductor de fase se sobredimensiona.
- **Neutro como portador de corriente:** cuenta en una fase y neutro, y en
  dos fases con neutro de un sistema en estrella; en trifásico de cuatro
  hilos solo con carga mayormente no lineal.
- **Tubería:** 53, 31 y 40 % según el número de conductores; niple al 60 %
  como opción por tramo; atascamiento entre 2.8 y 3.2 con el diámetro interior
  real, subiendo un tamaño.
- **Límites de caída:** 3 % en derivado, 2 % en alimentador y 5 % acumulado,
  todos configurables. El de alimentador es criterio propio, no de la norma.
- **Redondeo:** el motor no redondea. Las comparaciones usan el valor
  completo y toda selección de tabla toma el valor inmediato superior.

## Estados

Cada elemento queda como **cumple**, **no cumple** o **no evaluado**. Nunca
se marca algo como conforme si falta un dato o una regla. El programa no
otorga aprobación profesional.

## Estado actual

Hecho:

- Tablas de la NOM en CSV, validadas y cotejadas contra una captura independiente.
- Motor: corriente, caída de tensión, conductor, interruptor, puesta a tierra y tubería.
- Modelo del proyecto con subtableros, cuadro de cargas y caída acumulada.
- Línea de comandos: `ejemplo` y `calcular`, con salida en JSON.
- Integración continua en Windows y Linux, que publica el ejecutable.

Pendiente, en orden:

1. Interfaz gráfica: pantalla principal y cálculo rápido.
2. Vista de procedimiento paso a paso y recomendaciones aplicables.
3. Memoria de cálculo en PDF.
4. Diagrama unifilar automático para instalaciones radiales.
5. Instalador de Windows.
6. Servidor MCP para agentes.

Fuera del alcance de la versión 0.1: reglas de motores y de climatización,
cortocircuito y capacidad interruptiva, coordinación, transformador y media
tensión, charola, sistema fotovoltaico, tierras, y otros perfiles normativos.

## Fuentes

- NOM-001-SEDE-2012, publicación del Diario Oficial de la Federación.
- Hojas de cálculo del Ing. Juan Carlos Vega Mendoza, como referencia de
  lógica y casos de prueba. No se incluyen en el repositorio.
- Manual Técnico de Cables de Energía, Condumex, 2006, como referencia teórica.

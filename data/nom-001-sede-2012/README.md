# Tablas de la NOM-001-SEDE-2012

Datos normativos del perfil `nom-001-sede-2012`, capturados de la publicación
de la norma en el Diario Oficial de la Federación. Cada archivo CSV
corresponde a una tabla o sección y se puede abrir y revisar en cualquier hoja
de cálculo.

| Archivo | Origen en la norma |
| --- | --- |
| `table_1_conduit_fill.csv` | Capítulo 10, Tabla 1 |
| `table_4_conduit.csv` | Capítulo 10, Tabla 4 (EMT, IMC, RMC, PVC cédula 40 y 80) |
| `table_5_insulated_conductors.csv` | Capítulo 10, Tabla 5 |
| `table_8_conductor_properties.csv` | Capítulo 10, Tabla 8 (cableado clase B) |
| `table_9_impedance.csv` | Capítulo 10, Tabla 9 (ohms al neutro por kilómetro, 75 °C) |
| `table_310_15_b_16_ampacity.csv` | Tabla 310-15(b)(16) |
| `table_310_15_b_2_a_temperature_correction.csv` | Tabla 310-15(b)(2)(a) |
| `table_310_15_b_3_a_bundling_adjustment.csv` | Tabla 310-15(b)(3)(a) |
| `table_250_122_equipment_grounding.csv` | Tabla 250-122 |
| `section_240_6_standard_ratings.csv` | Sección 240-6(a). La columna `series` separa los valores de uso común en México de los de la serie IEC (16, 32 y 63 A) |
| `section_240_4_d_small_conductors.csv` | Sección 240-4(d), calibres 14 a 10 AWG |
| `table_430_248_single_phase_motors.csv` | Tabla 430-248, corriente a plena carga de motores monofásicos (115, 127, 208 y 230 V) |
| `table_430_250_three_phase_motors.csv` | Tabla 430-250, motores trifásicos de inducción de jaula de ardilla y de rotor devanado (115 a 575 V). No incluye la columna de 2300 V (media tensión) ni la de motores síncronos de factor de potencia unitario |
| `motor_voltage_columns.csv` | Qué columna de las tablas 430-248 y 430-250 corresponde a cada tensión del sistema, según el texto de cada tabla («de 110 a 120 volts, 220 a 240 volts…»). La columna de 127 V es propia de la NOM |
| `table_430_52_motor_protection.csv` | Tabla 430-52, ajuste máximo de la protección contra cortocircuito del circuito del motor, en porciento de su corriente a plena carga |
| `table_220_12_lighting_load.csv` | Tabla 220-12, carga mínima de alumbrado general por tipo de inmueble, en VA/m². `dwelling` marca la vivienda (220-14(j)); `area_only`, las áreas que solo existen dentro de otro inmueble (pasillos, lugares de reunión, bodegas) |
| `table_220_42_lighting_demand.csv` | Tabla 220-42, factores de demanda del alumbrado por escalones. Un tipo de inmueble sin filas va al 100 % («todos los demás») |
| `table_220_44_receptacle_demand.csv` | Tabla 220-44, factores de demanda de contactos fuera de vivienda |
| `table_220_56_kitchen_demand.csv` | Tabla 220-56, factores de demanda de equipos de cocina fuera de vivienda |
| `section_220_14_k_office_receptacles.csv` | Sección 220-14(k), carga mínima de contactos en bancos y oficinas |
| `profile.csv` | Datos de la norma (código, título, edición, URL) y parámetros: 125 % de carga continua (210-19(a)(1)), terminales de 60 °C hasta 100 A (110-14(c)), valor inmediato superior hasta 800 A (240-4(b)), calibres mínimos, atascamiento (Tabla 1, nota 2) y niples (notas de las tablas, nota 4), y los factores de motores (430-22, 430-24, 430-32, 430-52), de motocompresores (440-22, 440-32), la carga por salida de contacto (220-14(i)) y el factor de aparatos fijos de vivienda (220-53) |
| `references.csv` | Cláusula de cada regla del motor, con una descripción propia (no es texto de la norma) |

## Cómo se capturaron

Las tablas grandes (1 a 9 del capítulo 10 y la 310-15(b)(16)) se extraen del
texto del PDF con `tools/extract_nom_tables.py`, que además valida la
consistencia interna: el área de cada conductor y de cada tubo contra
π/4 · d², y las columnas de 60, 53, 31 y 40 % contra el área total.
Las tablas cortas se capturaron a mano.

Las tablas 5, 8, 9 y 310-15(b)(16) se cotejaron además, celda por celda,
contra una captura independiente (las hojas de cálculo del Ing. Juan Carlos
Vega Mendoza), sin diferencias en los valores normativos.

## Erratas del texto fuente

El texto publicado tiene errores de imprenta. Los que afectan a estos datos
están corregidos y anotados:

- **Tabla 5, TW/THW/THHW/THW-2 calibre 10 AWG:** el DOF imprime un área de
  55.68 mm². Con su diámetro de 4.470 mm el área es 15.69 mm², y la tabla de
  origen (NEC) indica 15.68 mm². Se usa 15.68.
- **Tabla 310-15(b)(2)(a):** el intervalo de temperatura «91-75» es 71-75 °C.
- **Tabla 430-250, 10 hp a 575 V:** el DOF imprime 44 A. La secuencia de la
  columna (6.1, 9, ?, 17) y la tabla de origen (NEC) indican 11 A. Se usa 11.
- **Tabla 430-250, 10 hp a 200 V:** el DOF imprime 32.3 A y la tabla de
  origen (NEC 2011) 32.2 A. No es una errata evidente; se respeta el 32.3 de
  la NOM.
- **Tabla 4:** el encabezado del artículo 358 dice «no metálico (EMT)»; el
  tubo EMT es metálico de pared delgada. No afecta a los valores.

## Aviso

Estos archivos son una transcripción de trabajo. La fuente con validez
oficial es la norma publicada en el DOF. Si encuentras una diferencia,
repórtala.

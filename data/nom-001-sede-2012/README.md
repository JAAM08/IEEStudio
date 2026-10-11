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
| `section_240_6_standard_ratings.csv` | Sección 240-6(a) |

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
- **Tabla 4:** el encabezado del artículo 358 dice «no metálico (EMT)»; el
  tubo EMT es metálico de pared delgada. No afecta a los valores.

## Aviso

Estos archivos son una transcripción de trabajo. La fuente con validez
oficial es la norma publicada en el DOF. Si encuentras una diferencia,
repórtala.

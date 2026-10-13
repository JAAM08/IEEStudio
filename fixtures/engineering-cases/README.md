# Casos de ingeniería

Casos de referencia en archivos CSV, uno por cálculo y por perfil normativo.
Un ingeniero puede revisarlos y ampliarlos en una hoja de cálculo sin leer
Rust; las pruebas de `voltforge-electrical` (`tests/engineering_cases.rs`)
los ejecutan todos.

| Archivo | Qué verifica |
| --- | --- |
| `voltage_drop.csv` | Caída de tensión en volts y en porciento |
| `conductor_sizing.csv` | Corriente de diseño, interruptor, calibre, ampacidad permitida y puesta a tierra |
| `conduit.csv` | Área de conductores, ocupación permitida y tamaño del tubo |
| `motors.csv` | Corriente de motores y motocompresores, conductor, interruptor y calibre |
| `feeder_load.csv` | Carga del alimentador según el artículo 220: mínimas por área y factores de demanda. Las cargas de varios circuitos se separan con `;`. Los ejecuta `voltforge-project` (`tests/engineering_cases.rs`) |

Columnas comunes:

- `source`: de dónde sale el valor esperado (hoja de referencia, cálculo a mano).
- `reviewed_by`: nombre y cédula de quien revisó el caso. **Vacío significa
  que el caso aún no tiene revisión de un ingeniero**; la prueba lo ejecuta,
  pero no cuenta como validado.

Un valor esperado vacío no se compara. Las tolerancias están en la unidad del
valor mostrado.

Para agregar un caso: una fila nueva con un `id` único, el cálculo hecho a mano
con su fuente y, cuando se revise, el nombre de quien lo revisó.

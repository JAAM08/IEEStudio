# 0035. Tubería compartida con conductores de otros circuitos

**Estado:** aceptada (25 de noviembre de 2026). El dueño del proyecto aprobó
el criterio (D-1 y D-5 de la fase 0.1) después de la auditoría técnica de la
fase 0, hallazgo C-1. Implementada en la fase 0.1.

## Contexto

En cada tramo se puede capturar «Otros portadores»: cuántos conductores
portadores de corriente de otros circuitos van en la misma tubería. Ese dato
sirve para el factor de agrupamiento (Tabla 310-15(b)(3)(a)), que cuenta
solo portadores, y se aplicaba bien.

La ocupación de la tubería (Capítulo 10, Tabla 1 y sus notas) es otra
verificación: suma el área de **todos** los conductores que lleva la
tubería, incluidos neutros que no portan corriente y conductores de puesta a
tierra, y el área depende del calibre, el aislamiento y la cubierta de cada
uno. El programa no tiene esos datos de los otros circuitos: solo un conteo
de portadores. Hasta ahora calculaba la ocupación como si la tubería fuera
solo del circuito y la daba por buena. Con seis portadores ajenos de 12 AWG,
el tramo decía «EMT de 16 mm, cumple» cuando hacía falta al menos 21 mm, y el
proyecto salía «Conforme en lo evaluado».

## Decisión

- Si un tramo tiene otros portadores en su tubería, **la ocupación queda sin
  evaluar**: la verificación «Tubería» dice que la tubería es compartida y
  qué falta para verificarla. El tramo y el dictamen no pueden ser
  conformes.
- **No se publica ningún diámetro**: la memoria, el unifilar, la línea de
  comandos, Excel y la aplicación muestran «—» en la tubería.
- No se suponen los calibres de los otros conductores: un dato de
  agrupamiento no sirve como dato de ocupación.
- La ampacidad, el calibre, el interruptor, la tierra y la caída de tensión
  no cambian.
- El paso de tubería del procedimiento se conserva como referencia y dice
  que es el mínimo para los conductores del circuito, no la tubería a
  instalar.
- Si los conductores del propio circuito ya no caben en ningún tubo, el
  resultado sigue siendo «No cumple»: con más conductores tampoco caben.
- La tubería compartida no hace proponer conductores en paralelo: el
  criterio se aplica después de elegir los conductores por fase.

## Consecuencias

- Los proyectos que capturaron otros portadores pasan de «Conforme» a
  «Evaluación incompleta». El archivo `.iees` no cambia.
- Verificar una tubería compartida necesitará capturar la canalización con
  todos sus conductores (una entidad de canalización); queda para una etapa
  posterior y no forma parte de esta decisión.
- Caso de referencia: `fixtures/fase-0/c1-tuberia-compartida.iees`; pruebas
  en `crates/ieestudio-project/tests/fase0.rs` y
  `crates/ieestudio-cli/tests/fase0_salidas.rs`.

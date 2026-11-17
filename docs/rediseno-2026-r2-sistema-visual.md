# Rediseño 2026 — R2: sistema visual

**Fecha:** 17 de noviembre de 2026.
**Base:** `main` con R1 integrado (PR #24).
**Referencias:** dirección visual aprobada en Claude Design (cuatro tableros:
sistema visual, piezas comunes, escritorio y teléfono) y jerarquía cromática
definida por el dueño del proyecto.

R2 cambia solo el aspecto. No toca el motor, las tablas, las decisiones
técnicas, el formato `.iees`, la memoria en PDF ni el dibujo del unifilar.

## 1. Qué cambia para quien usa el programa

- Colores nuevos con una jerarquía fija, igual en tema oscuro y claro:
  - **Amarillo eléctrico**: acento principal; acciones primarias, selección
    activa e indicadores importantes. Destaca sin dominar.
  - **Azul técnico**: información eléctrica, datos técnicos, referencias a la
    norma y elementos interactivos secundarios.
  - **Morado lavanda**: acento secundario de identidad; categorías (tipo de
    carga), calibre fijado y origen «fijado».
  - **Verde, rojo y naranja**: solo para estados (cumple, no cumple, sin evaluar).
  - **Obsidiana, grafito y pizarra** dominan las superficies; el tema claro
    usa sus equivalentes.
- Las fases A, B y C usan azul, lavanda y cian en las barras y letras del
  cuadro de cargas.
- Los estados del cuadro de cargas, del inspector y de la vista guiada son
  etiquetas pequeñas de la misma forma y altura, con ícono y texto (el color
  nunca es lo único que distingue).
- Se quitó el menú «Color de acento»: el acento ahora es parte de la
  identidad. Las preferencias guardadas con un acento anterior se leen sin
  error y se ignoran.
- Esquinas con tres medidas: 6 px controles, 8 px tarjetas y menús, 10 px
  ventanas.
- Controles un poco más compactos (alto mínimo 26 px).

## 2. Valores

| Papel | Oscuro | Claro |
|---|---|---|
| Fondo | `#17191E` | `#E9ECF0` |
| Panel | `#20242B` | `#F4F5F7` |
| Elevado | `#292E37` | `#FFFFFF` |
| Texto | `#E8EBF0` | `#1B1F25` |
| Acento (amarillo) | `#E8BD35` | texto `#7A5C00` |
| Info (azul) | `#6D9FE8` | `#2B66C4` |
| Lavanda | `#9B91CC` | `#6A5FA8` |
| Cumple | `#75B99B` | `#1F7A45` |
| No cumple | `#F08A8A` | `#C0303A` |
| Sin evaluar | `#E8924A` | `#B4560B` |

Los valores viven en `crates/ieestudio-app/src/theme.rs`. No hay valores
normativos involucrados.

## 3. Pruebas

- Tres pruebas nuevas en `theme.rs`: contraste mínimo 4.5:1 del texto sobre
  panel y superficie elevada en ambos temas, contraste ≥ 7:1 del texto sobre
  el acento, y separación de tono entre el acento amarillo y el naranja de
  «sin evaluar» para que no se confundan.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, clippy para
  `wasm32` y `cargo test` pasan (279 pruebas, 0 fallas).

## 4. Capturas

Matriz completa de `tools/capturas.sh` ejecutada una vez al final: **70 de 70
capturas sin fallas** (vistas × cinco tamaños × dos temas). Durante la
iteración se usó muestreo (dos vistas, dos tamaños, un tema). Capturas
representativas en `docs/capturas-r2/`.

## 5. Rendimiento

Compilación optimizada, 1440 × 900, misma máquina que R1 (2 núcleos, dibujo
por software). «Antes» es el binario de R1.

| Proyecto | Vista | Mediana antes → después (ms) | p95 antes → después (ms) | Memoria pico antes → después (MiB) |
|---|---|---|---|---|
| casa | proyecto | 0.21 → 0.24 | 0.27 → 0.36 | 164 → 166 |
| casa | tableros | 0.29 → 0.32 | 0.41 → 0.43 | 164 → 167 |
| casa | circuitos | 0.42 → 0.45 | 0.46 → 0.63 | 163 → 166 |
| casa | revisar | 0.38 → 0.42 | 0.49 → 0.71 | 163 → 165 |
| casa | avanzada | 0.94 → 1.11 | 1.14 → 1.73 | 168 → 170 |
| taller | proyecto | 0.23 → 0.29 | 0.32 → 0.47 | 164 → 166 |
| taller | tableros | 0.36 → 0.46 | 0.74 → 0.77 | 165 → 167 |
| taller | circuitos | 0.47 → 0.54 | 0.68 → 0.88 | 164 → 161 |
| taller | revisar | 0.51 → 0.56 | 0.85 → 0.80 | 169 → 170 |
| taller | avanzada | 1.11 → 1.23 | 1.64 → 2.12 | 167 → 170 |
| grande | proyecto | 0.48 → 0.50 | 0.59 → 0.62 | 172 → 174 |
| grande | tableros | 0.73 → 0.83 | 0.86 → 1.14 | 173 → 175 |
| grande | circuitos | 0.87 → 0.90 | 1.09 → 1.18 | 173 → 176 |
| grande | revisar | 9.28 → 9.47 | 12.37 → 13.63 | 190 → 189 |
| grande | avanzada | 3.00 → 3.11 | 3.89 → 4.58 | 176 → 179 |

Las medianas suben entre 0.0 y 0.15 ms, dentro del ruido de 15–25 % medido en
R1. El caso más pesado (Grande, Revisar) pasa de 9.28 a 9.47 ms de mediana,
bajo los 16 ms de un cuadro a 60 Hz. La memoria sube de 1 a 3 MiB.

## 6. Versión web

`tools/sitio-web.sh` compila y arma el sitio sin errores (compilación
optimizada para `wasm32`, 6 min 35 s). El módulo pesa 50,778,609 bytes
(≈ 48.4 MiB) frente a 50,777,970 bytes del sitio armado antes de R2: la
diferencia es de 639 bytes. Solo se verificó que compila y se arma; no se
abrió en un navegador.

## 7. Piezas de la maqueta que quedan para después

La maqueta incluye control segmentado, interruptor, hoja inferior y filas
táctiles de 44 px. No se implementaron porque ninguna pantalla actual las
usa; llegan con R3 y R6, cuando haya dónde ponerlas.

## 8. Límites

- Las mediciones son de una máquina con dibujo por software; sirven para
  comparar antes y después, no como promesa para otra computadora.
- La revisión visual detallada se hizo sobre capturas muestreadas; el resto
  de la matriz se verificó automáticamente (que ninguna vista se cierre).

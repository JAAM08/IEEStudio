# Rediseño 2026 — R4: cuadro, navegador e inspector unificados

**Fecha:** 18 de noviembre de 2026.
**Base:** `main` con R3 integrado (PR #26).
**Referencias:** maqueta R0 aprobada (secciones 4 a 6), diagnóstico del
rediseño (texto de la verificación de capacidad interruptiva).

R4 completa la captura dentro del marco único. No toca el motor, las tablas,
las decisiones técnicas, el formato `.iees`, la memoria en PDF ni el dibujo del
unifilar. Ningún número cambia: lo que cambia es dónde se captura y cómo se ve.

## 1. Qué cambia para quien usa el programa

**Vista Proyecto**
- Un resumen con cuatro cifras: la verificación (conforme o no, en lo
  evaluado), los tramos calculados, la carga de la instalación y la falla en
  el punto de conexión.
- Una tabla de tableros con lo esencial de cada uno (de dónde se alimenta,
  circuitos, carga calculada, alimentador y estado) y un botón para abrirlo.
- Cuatro bloques de captura: datos de la obra, tipo de inmueble, suministro y
  acometida, y criterios de diseño.
- La **verificación de capacidad interruptiva** es un interruptor con la nota
  fija «Estimación conservadora y limitada; no sustituye un estudio integral de
  cortocircuito». Apagarla pide confirmar: «La capacidad interruptiva quedará
  fuera de alcance y así se anotará en la memoria. No se borra ningún dato de
  la fuente, los transformadores ni los interruptores». Encenderla no pregunta.
  El comportamiento es el de la decisión 0032, sin cambios.
- La carga de la instalación dice «Sin conocer» si a algún circuito le falta
  la potencia: una suma parcial no se presenta como total (decisión 0033).

**Tableros y circuitos**
- Un solo cuadro con dos juegos de columnas:
  - **Esencial** (el de omisión): lo que capturas (tipo como símbolo, carga,
    fases y longitud) y lo que resuelve IEEStudio (calibre, interruptor, caída
    acumulada y estado), con los dos encabezados de grupo que tenía la tabla
    guiada.
  - **Completo**: las columnas que elijas en ≡ › Ver › Personalizar columnas.
- Filtro **Todos / Pendientes** (lo que no cumple o quedó sin evaluar), además
  del filtro por texto.
- «Agregar circuito» es la acción amarilla del cuadro y abre «¿Qué vas a
  conectar?».
- Menú **Tablero**: balancear fases (no reparte si falta una potencia,
  decisión 0033), agregar subtablero y eliminar el tablero.
- La franja del tablero agrega el factor de demanda (F.D.) y pasa sus cifras
  al renglón de abajo cuando no caben.

**Navegador**
- Un solo botón «Agregar» con tres opciones sin ambigüedad: tablero desde la
  acometida, subtablero del tablero elegido y circuito en él. Antes «+
  Tablero» creaba un subtablero o un tablero según lo que estuviera elegido.

**Ficha (inspector)**
- Arriba, el estado y el tipo, con tres botones: procedimiento, duplicar y
  eliminar.
- Si algo no cumple o quedó sin evaluar, un recuadro lo dice en palabras, con
  su cláusula, «Usar … (propuesto)» cuando fijaste otro calibre y «Ver cómo se
  calculó», que abre el procedimiento abajo.
- «Decisiones de diseño» se llama ahora **Ajustes manuales** e incluye fijar el
  calibre que resultó en un clic (candado), como hacía la barra contextual.
- La barra contextual del cuadro se retira: sus acciones viven en la ficha.

**Importar Excel en tres pasos**
1. Elegir el archivo.
2. Vista previa: cuántos circuitos y tableros se agregan o se actualizan y qué
   filas no se pueden importar. Nada cambia hasta pulsar «Importar».
3. Informe, con «Deshacer la importación».

## 2. Ventanas angostas

Igual que en R3: con menos de 900 px «Tableros y circuitos» conserva sus dos
contenidos de antes (Tableros y Circuitos) y la tabla guiada. La vista
Proyecto nueva ya se acomoda a cualquier ancho (las cifras y las tarjetas
pasan una debajo de otra). Cajones, hojas y barra inferior son de R6.

## 3. Funciones de la sección 6 de la maqueta que cubre esta etapa

| Fila | Función | Dónde queda |
|---|---|---|
| 2 | Datos de obra | Proyecto › Datos de la obra |
| 3 | Tipo de inmueble (tarjetas y «Otro tipo») | Proyecto › Tipo de inmueble |
| 4 | Suministro, topología, «Ajustarlos» | Proyecto › Suministro y acometida |
| 5 | Transformador de la acometida, banco, dato de CFE | Proyecto › Suministro y acometida (la captura simplificada es de R8) |
| 6 | Capacidad interruptiva encendida o apagada | Interruptor con nota fija y confirmación |
| 7 | Criterios de diseño | Proyecto › Criterios de diseño |
| 8 | Árbol de tableros | Un solo navegador (el árbol guiado queda solo en ventanas angostas) |
| 9 | Ficha de tablero | Inspector › Datos |
| 10 | Agregar tablero o subtablero | Navegador › Agregar, menú Tablero y «¿Qué vas a conectar?» |
| 11 | «¿Qué vas a conectar?» | Ventana, sin cambios |
| 12 | Tabla guiada | Juego «Esencial» del cuadro |
| 13 | Columnas y filtros | «Esencial» y «Completo», «Todos» y «Pendientes»; columnas en ≡ › Ver |
| 14 | Barra contextual | Ficha: duplicar, eliminar, fases en Datos, fijar calibre en Ajustes manuales |
| 15 | Renglones de alimentador y secundario | Primeros renglones del cuadro, sin cambios |
| 16 | Ficha de circuito | Inspector › Datos, con lo que no cumple y su arreglo arriba |
| 17 | Resultados y Normativa | Pestañas del inspector, sin cambios |
| 19 | Balancear fases | Menú Tablero y aviso con botón en Revisar (ya estaba); no reparte si falta una potencia |
| 26 | Importar Excel | Tres pasos: archivo, vista previa, informe con deshacer |

## 4. Pruebas

- **Mismos resultados en interfaz, memoria y línea de comandos.** Con cada
  proyecto de `fixtures/` (22 archivos; los que a propósito no se pueden
  calcular deben fallar igual en los dos lados): el resultado que usa la interfaz es idéntico, campo
  por campo, al de `ieestudio calcular --json`; el calibre, el interruptor, la
  caída del tramo y la acumulada del cuadro son las mismas cadenas que escribe
  la memoria para cada circuito.
- Cuadro: el filtro «Pendientes» deja lo que no cumple y lo sin evaluar; el
  juego «Esencial» pone lo capturado antes de lo resuelto y el estado al final;
  «Completo» son las columnas elegidas.
- Proyecto: el resumen tiene sus cuatro cifras; la carga es la de los tableros
  que salen de la acometida; sin una potencia dice «Sin conocer»; con la
  verificación apagada, la falla queda «Fuera de alcance».
- Preferencias: el juego de columnas se recuerda; las de versiones anteriores
  se siguen leyendo.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, clippy para
  `wasm32` y `cargo test` pasan (290 pruebas, 0 fallas).

## 5. Capturas

Matriz completa de `tools/capturas.sh`, ejecutada una vez al final: **70 de
70 capturas sin fallas** (compilación optimizada). Además, **28 de 28** en
compilación de depuración en los cuatro tamaños angostos, la que se cierra con
un ancho negativo. Después de la matriz solo cambió que las cifras del resumen
de Proyecto se parten en dos renglones en vez de recortarse; esa vista se
volvió a capturar. Capturas representativas en `docs/capturas-r4/`.

## 6. Rendimiento

Compilación optimizada, 1440 × 900, misma máquina y mismo procedimiento que R3.

| Proyecto | Vista | Mediana R3 → R4 (ms) | p95 R3 → R4 (ms) | Máx. R4 (ms) | Memoria pico R3 → R4 (MiB) |
|---|---|---|---|---|---|
| casa | proyecto | 0.29 → 0.37 | 0.42 → 0.51 | 0.75 | 163 → 164 |
| casa | cuadro | 1.06 → 0.96 | 1.53 → 1.33 | 1.67 | 166 → 162 |
| casa | unifilar | 0.62 → 0.63 | 0.83 → 0.78 | 0.88 | 168 → 168 |
| casa | revisar | 0.53 → 0.51 | 0.73 → 0.67 | 0.77 | 166 → 162 |
| casa | procedimiento | 1.18 → 1.11 | 1.49 → 1.47 | 1.61 | 166 → 167 |
| casa | dividida | 1.10 → 1.05 | 1.61 → 1.59 | 1.76 | 167 → 169 |
| taller | proyecto | 0.30 → 0.43 | 0.40 → 0.61 | 0.83 | 163 → 164 |
| taller | cuadro | 1.10 → 1.10 | 1.37 → 1.50 | 1.84 | 166 → 167 |
| taller | unifilar | 0.68 → 0.73 | 1.20 → 1.04 | 2.74 | 166 → 166 |
| taller | revisar | 0.73 → 0.68 | 0.93 → 0.96 | 1.06 | 168 → 168 |
| taller | procedimiento | 1.21 → 1.22 | 1.80 → 1.82 | 2.13 | 163 → 167 |
| taller | dividida | 1.10 → 1.13 | 1.31 → 1.56 | 1.78 | 168 → 170 |
| grande | proyecto | 0.55 → 0.78 | 0.75 → 1.01 | 1.15 | 172 → 174 |
| grande | cuadro | 3.86 → 3.80 | 5.31 → 5.01 | 7.78 | 178 → 178 |
| grande | unifilar | 4.32 → 4.43 | 5.86 → 6.35 | 7.02 | 188 → 188 |
| grande | revisar | 10.20 → 10.46 | 12.61 → 12.88 | 14.21 | 187 → 187 |
| grande | procedimiento | 3.63 → 3.51 | 5.42 → 4.76 | 5.14 | 177 → 177 |
| grande | dividida | 3.53 → 3.56 | 4.05 → 6.03 | 10.34 | 180 → 179 |

Las vistas de trabajo quedan dentro del ruido de 15 a 25 % medido en R1. La
vista Proyecto sube (de 0.29 a 0.37 ms en la casa y de 0.55 a 0.78 ms en el
proyecto grande) porque ahora dibuja el resumen y la tabla de tableros; sigue
muy por debajo de los 16 ms de un cuadro. El caso más pesado sigue siendo
Revisar con 386 circuitos (mediana 10.5 ms, p95 12.9 ms).

## 7. Versión web

`tools/sitio-web.sh` compila y arma el sitio sin errores (4 min 18 s). El
módulo pesa 50,552,735 bytes contra 50,472,099 de R3: 80,636 bytes más, por la
vista Proyecto y la vista previa de la importación. Solo se verificó que
compila y se arma; no se abrió en un navegador.

## 8. Límites y lo que queda

- **Pantalla de inicio con dos experiencias** («Proyecto completo» y «Cálculo
  rápido», requisito del 18 de noviembre de 2026): no es de R4. Queda anotada
  en `roadmap.md` como parte de R6, donde el cálculo rápido pasa a ser su
  propia pantalla en teléfono y se arma el marco de cada tamaño.
- En ventanas angostas sigue la tabla guiada de antes; el cuadro único con
  tarjetas para teléfono es de R6.
- La ficha del proyecto en el inspector (al elegir el proyecto en el
  navegador) sigue disponible con los mismos datos que la vista Proyecto.
- Ningún clic se probó con pruebas automáticas de la interfaz (no existen en
  este proyecto); el comportamiento se revisó con capturas.
- Las mediciones son de una máquina con dibujo por software; sirven para
  comparar, no como promesa para otra computadora.

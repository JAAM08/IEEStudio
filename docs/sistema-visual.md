# Sistema visual de IEEStudio

**Vigente desde:** R4.5 (19 de noviembre de 2026).
**Obligatorio** para toda pantalla nueva o modificada, incluidas las etapas R5,
R6 y R7 del rediseño. Lo aprobó el dueño del proyecto en el lienzo de Claude
Design «IEEStudio — Dirección visual R2 y lenguaje R4.5» (mesas 1 a 7).

Referencias de lenguaje: macOS y HarmonyOS, **sin copiar** controles ni
identidad de ninguno de los dos. Orden, no efectos: sin desenfoque, sin
transparencias grandes, sin animaciones fuera de cajones y hojas.

Los valores viven en `crates/ieestudio-app/src/theme.rs`. La prueba
`the_interface_uses_only_the_visual_system` falla si el código de la interfaz
usa un tamaño de letra fuera de la escala, un margen o espacio fuera de la
retícula, o convierte un rótulo a mayúsculas.

## 1. Color (aprobado en R2, sin cambios)

| Papel | Uso | Oscuro | Claro |
|---|---|---|---|
| Amarillo eléctrico | Acción principal de la vista, vista activa, foco, indicadores importantes. **Una sola acción amarilla por vista.** | `#E8BD35` | `#E8BD35` (texto `#7A5C00`) |
| Azul técnico | Información eléctrica, referencias a la norma, enlaces | `#6D9FE8` | `#2B66C4` |
| Morado lavanda | Categorías (tipo de carga), calibre fijado | `#9B91CC` | `#6A5FA8` (texto) |
| Verde, rojo, naranja | Solo estados: cumple, no cumple, sin evaluar | `#75B99B` · `#F08A8A` · `#E8924A` | `#1F7A45` · `#C0303A` · `#B4560B` |
| Obsidiana, grafito, pizarra | Superficies | ver §3 | ver §3 |

Los estados llevan siempre **ícono, texto y color**; el color nunca es lo único
que distingue.

## 2. Tipografía

IBM Plex Sans y Plex Mono (ya incluidas). **Cinco tamaños**:

| Nombre | Tamaño | Peso | Uso |
|---|---|---|---|
| `TITLE` | 22 | 600 | Título de sección; cifras grandes en mono |
| `HEAD` | 15 | 600 | Lo que se está viendo: tablero, circuito, bloque |
| `BODY` | 13 | 400 | Texto, campos, tablas, botones |
| `SECONDARY` | 12 | 400 | Aclaraciones, subtítulos, títulos de grupo (600) |
| `FOOT` | 11 | 500 | Rótulos de cifras y columnas |

**Excepciones permitidas** (no se agregan otras):

| Nombre | Tamaño | Cuándo |
|---|---|---|
| `VIEW_TITLE` | 28 | El título principal de una vista: uno por pantalla (Proyecto, Memoria, inicio) |
| `TOUCH` | 15 | Teléfono: texto de renglones y botones de 44 px |
| `TOUCH_FIGURE` | 17 | Teléfono: cifras del resumen |

Rótulos **en tipo oración** («Carga conectada», «Lo que capturas»), nunca en
mayúsculas. Las cifras van en Plex Mono.

Íconos: `ICON` 14 px en texto, tablas y fichas; `ICON_BAR` 16 px en barras y
botones de solo ícono. Grises (`ink2`/`ink3`); el acento solo en lo activo.

## 3. Superficies y profundidad

Tres tonos y una sola sombra:

| Nivel | Oscuro | Claro | Dónde |
|---|---|---|---|
| Cromo (`bg`) | `#17191E` | `#E9ECF0` | Barra superior, navegador, barra de estado |
| Contenido (`panel`) | `#20242B` | `#F4F5F7` | Vistas, cuadro, inspector |
| Elevado (`raise`) | `#292E37` | `#FFFFFF` | Grupos de campos, controles, recuadros |
| Flotante | `raise` + sombra | `raise` + sombra | Menús, ventanas, avisos, hojas: **la única sombra** |

Divisores: `hair` (`#2C313A` / `#E3E7EC`) entre renglones de un grupo y entre
regiones; `line` (`#363C46` / `#D3D8DF`) para el borde de los controles y de
los grupos.

**No multiplicar contenedores.** Un recuadro no va dentro de otro recuadro: los
bloques de una vista se separan con un divisor y su título; los campos van en
grupos compactos; el cuadro de cargas va directo sobre el contenido.

## 4. Retícula y medidas

- Espacios: **4, 8, 12, 16, 24 y 32 px** (`SP1`, `SP2`, `SP3`, `SP4`, `SP6`,
  `SP8`). Solo se permite 2 px como ajuste interno de cápsulas y divisores.
- Radios: 6 px controles, 8 px tarjetas y grupos, 10 px ventanas y hojas;
  cápsula (extremos redondos) para la barra de vistas, los controles
  segmentados y las etiquetas de estado.
- Alturas: 24 px controles en fichas y tablas (`CONTROL_H`); 28 px en barras
  (`BAR_CONTROL_H`) y renglones con ratón (`ROW_H`); 44 px táctil
  (`TOUCH_H`).

## 5. Piezas comunes (`widgets.rs`)

| Pieza | Función | Regla |
|---|---|---|
| Grupo de campos | `section` + `property` | Título en tipo oración; renglones de 28 px sobre `raise` con divisores `hair`; `end_group` cierra el grupo antes de algo que no es campo |
| Botón principal | `primary_button`, `primary_bar_button` | Amarillo, texto oscuro, seminegrita; uno por vista |
| Botón secundario | botón de egui | `raise` con borde `line` |
| Control segmentado | `segmented` | Cápsula; la opción elegida elevada |
| Interruptor | `switch` | Opciones de alcance; apagarlo puede pedir confirmación |
| Etiqueta de estado | `status_chip` | Cápsula con ícono, texto y color |
| Etiqueta de categoría | `tag` | Lavanda |
| Insignia de conteo | `count_badge` | Número en píldora tenue del color del estado |

## 6. Navegación

- **Barra de vistas:** una cápsula centrada en la barra superior; la vista
  activa va elevada con su ícono en amarillo. Sin subrayado. Revisar lleva la
  insignia de pendientes.
- **Navegador:** en el tono del cromo; la selección es una píldora `selected`,
  **sin barra lateral**; el ícono del elegido en amarillo.
- **Cuadro:** la fila elegida se marca solo con el fondo `selected`.
- **Barra de estado:** discreta, en `SECONDARY`, pero conserva los tres niveles
  con su marca: cálculo (punto del color de su estado), cumplimiento (punto del
  color de su estado) y aprobación profesional (anillo vacío: la da el
  ingeniero, nunca el programa). «Siguiente» es un botón de texto, sin
  amarillo.
- **Teléfono (R6):** barra inferior de cinco destinos con una cápsula detrás
  del ícono activo; hojas inferiores; renglones de 44 px. La navegación y el
  desplazamiento se validan funcionalmente en R6.

## 7. Densidad

El sistema no sacrifica información de ingeniería: las tablas conservan sus
columnas, las cifras su precisión y los renglones 28 px con ratón. En teléfono
no se comprimen controles ni textos: lo que no cabe pasa abajo o a una hoja.

## 8. Cómo se usa en cada etapa

| Etapa | Qué debe cumplir |
|---|---|
| R5 (Unifilar y Memoria) | Controles flotantes del unifilar en cápsula con la sombra de lo flotante; ficha de Memoria (firma, «Antes de entregar») en grupos de campos |
| R6 (Tablet y teléfono) | Excepciones táctiles, hojas, barra inferior, pantalla de inicio con «Proyecto completo» y «Cálculo rápido» |
| R7 (Retiro y cierre) | Lo que quede fuera de este documento se corrige o se retira |

Cualquier excepción nueva se propone y se aprueba antes de programarla, y se
agrega aquí.

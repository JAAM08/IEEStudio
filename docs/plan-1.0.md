# IEEStudio — Plan de trabajo hacia la 1.0

Escrito el 10 de noviembre de 2026. Este documento es la instrucción completa para cualquier modelo de Claude que continúe el trabajo. Si eres ese modelo: léelo entero antes de tocar nada, trabaja **una etapa a la vez** y no pases a la siguiente sin cumplir su «Listo cuando».

## 1. Qué pidió el dueño del proyecto (Antonio)

1. **Teléfono ya, no después.** Un ingeniero en obra lleva su celular, no una laptop. La app debe poder usarse completa en un teléfono antes de la 1.0.
2. **Mejoras de uso diario** (el «punto 2» de la revisión): avisos agrupados, tabla de circuitos cómoda y editable, avisos que no regresan, Excel completo.
3. **Unifilar fase 3:** DXF con bloques y atributos, y tamaños de hoja.
4. **Rendimiento:** la app debe estar bien optimizada, **incluida la generación del PDF**.
5. **PDF personalizable (membrete):** que no sea un reporte genérico. Cada usuario debe poder poner portada, logotipo, colores y membrete a su gusto.
6. **Cuadro de cargas con su formato:** en la app y en la exportación a Excel, con el archivo `CUADRO_DE_CARGAS_CMAPAS.xlsx` como base, y personalizable.
7. **Que el programa no pregunte lo que ya sabe.** En el cálculo rápido eligió el sistema `1F-3H 240/120 V` y aun así pudo elegir «3 fases» en Conexión. Si el sistema ya dice cuántas fases hay, no debe volver a pedirlas ni permitir una combinación imposible.

8. **Estados que se entiendan** (evaluación de la interfaz del 10-nov-2026, aceptada por Antonio): hoy casi todo sale «Sin evaluar» por un solo motivo, los conteos de pendientes no coinciden entre pantallas, hay textos encimados o cortados y el unifilar no tiene un lugar propio en la vista guiada. Ver la etapa 1B.
9. **La vista avanzada queda congelada** (decisión de Antonio, 10-nov-2026): sigue funcionando en escritorio, pero no se adapta a teléfono ni recibe funciones nuevas. Lo que solo ella tiene (procedimiento completo, elegir columnas) se pasa a la vista guiada.
10. **El aviso «pensado para pantallas grandes» debe desaparecer** en cuanto el teléfono sea de verdad utilizable (etapa 3).
11. **Errores que aparezcan en el camino se arreglan**, sin pedir permiso, y se reportan. Para dejar la app a nivel profesional, quien ejecute toma la mejor decisión de diseño posible y la explica.

## 2. Estado actual (verifícalo antes de empezar)

- Repositorio: `github.com/JAAM08/IEEStudio`, rama `main`. Rust, interfaz egui (escritorio y navegador por WebAssembly), memoria en PDF con Typst, Excel con `rust_xlsxwriter` y `calamine`.
- El PR #6 ya está en `main` (10-nov-2026): unifilar como plano (decisión 0022), cortocircuito por bus infinito con datos de catálogo (0023) y símbolos de norma. Compruébalo: debe existir `crates/ieestudio-project/src/fault.rs`.
- Lee primero: `CLAUDE.md`, `docs/architecture.md`, `docs/engineering-methodology.md`, `docs/roadmap.md` y las decisiones `0017` (versión web), `0020` (vista guiada), `0021`, `0022`, `0023`, `0014` (Excel) y `0006` (memoria Typst).
- Hechos comprobados en el código:
  - El unifilar se reconstruye completo **en cada cuadro de pantalla** (`panels/mod.rs` y `guided.rs` llaman a `single_line…` dentro del dibujo).
  - Cada edición recalcula **todo el proyecto** (`Doc::edited` → `calculate`).
  - Tres archivos de la interfaz pasan de 1 900 líneas (`guided.rs`, `properties.rs`, `app.rs`) y `ieestudio-diagrams/src/lib.rs` también.
  - **Sistema y conexión no se validan entre sí.** `CircuitKind::reference_voltage` solo revisa que el sistema tenga la tensión, así que un circuito de «3 fases» en un sistema monofásico de tres hilos (240/120 V) se calcula con √3 y 240 V y da un resultado que parece válido. El cálculo rápido (`panels/quick.rs`) y la «Conexión» del alimentador de un tablero (`properties.rs`, `feeder_inputs`) ofrecen siempre las tres conexiones. La ficha del circuito ya limita las opciones a las del tablero, pero no al sistema.

## 3. Reglas para quien ejecute (obligatorias)

1. **Antonio revisa el funcionamiento, no el código.** Explícale cada cambio en términos de lo que el programa hace, en español, con capturas.
2. **No uses control de su computadora** (computer use). Si necesita probar algo, mándale el archivo o el enlace y él lo corre.
3. **Antes de cada etapa:** preséntale maquetas o capturas de lo que vas a construir cuando la etapa cambie pantallas o formatos, y espera su visto bueno. Para decisiones de diseño menores, decide tú y explica por qué.
4. **Decisiones de ingeniería o de norma son de Antonio.** No inventes valores normativos ni supongas datos: lo que no se puede evaluar se muestra «no evaluado». Los valores van en `data/<perfil>/*.csv`, nunca en el código.
5. **Un pull request por etapa (o subetapa), siempre contra `main`.** No encadenes PR sobre otras ramas: ya pasó que se fusionaron en la rama equivocada. Si una etapa depende de otra sin fusionar, espera o pide que la acepte.
6. **Antes de cada commit** deben pasar: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test` y `cargo clippy --target wasm32-unknown-unknown -p ieestudio-app -- -D warnings`.
7. **Revisa la interfaz con capturas** (`xvfb-run -a cargo run -p ieestudio-app -- … --captura x.png`) en tema oscuro y claro, y desde la etapa 2 también a tamaño de teléfono. Mira las capturas tú mismo antes de mandarlas.
8. **Cada regla de cálculo nueva** lleva su `Rule`, su referencia y sus tres pruebas (matemática, de referencia en `fixtures/`, normativa). Cada cambio de comportamiento lleva su decisión en `docs/decisions/` y su renglón en `docs/roadmap.md`.
9. **El formato del archivo `.iees` no se rompe:** campos nuevos con valor por omisión; los proyectos viejos deben abrir igual. Agrega una prueba de lectura de un archivo anterior.
10. **Todo debe funcionar también en el navegador:** archivos solo por el módulo `files` (decisión 0017), nada de `std::fs` directo, nada de hilos que WebAssembly no tenga sin su alternativa.
11. **No preguntes lo que el programa ya sabe.** Antes de agregar un campo, revisa si su valor se deduce de otro dato ya capturado; si se deduce, muéstralo, no lo pidas. Si solo hay una opción válida, no pongas una lista.
12. **Mejora continua.** No des una pantalla por terminada porque ya existe: al tocarla, mírala como usuario nuevo. Si encuentras un error (de cálculo, de texto, de acomodo) mientras trabajas una etapa, arréglalo en el mismo PR si es chico o en uno aparte si no, y repórtalo. Solo las decisiones de ingeniería o de norma esperan a Antonio.
13. **La vista avanzada no se toca** salvo para que siga compilando y no se rompa. Nada nuevo se construye en ella.
14. Si GitHub rechaza tus subidas, no insistas: empaqueta con `git bundle` y mándalo en el chat.

## 4. Etapas

El orden importa. La corrección de sistema y conexión va primero porque hoy produce resultados equivocados. El rendimiento sigue porque el teléfono lo necesita; la base de teléfono va antes que las pantallas nuevas para que cada una nazca ya adaptada.

### Etapa 0 — Preparación (corta)

**Qué:** dejar el terreno listo y poder medir.
- Confirmar que este plan está en `docs/plan-1.0.md`.
- Crear un proyecto de prueba grande generado por código: 12 tableros en 3 niveles, 400 circuitos, motores y un transformador. Guardarlo como generador en las pruebas, no como archivo enorme.
- Medir y anotar en el PR, con ese proyecto: tiempo de recálculo, tiempo de un cuadro de pantalla en cada paso, tiempo de generar el PDF, tamaño del `.wasm`.
- Agregar una opción `--medir` (o una prueba) que repita esas mediciones, para comparar después.
- Pruebas de interfaz por captura: un script que tome las capturas clave y las compare contra las guardadas, con tolerancia.

**Listo cuando:** hay una tabla de tiempos «antes» y un comando para repetirla.

### Etapa 1 — Sistema y conexión coherentes (corrección)

**El problema:** el sistema ya define qué conexiones existen, y el programa las vuelve a preguntar y acepta las imposibles.

| Sistema | Conexiones posibles |
|---|---|
| `1F-2H 127 V` | Solo 1 fase y neutro |
| `1F-3H 240/120 V` | 1 fase y neutro (120 V) o 2 fases (240 V). **Nunca 3 fases** |
| `3F-4H 220Y/127 V`, `480Y/277 V` | 1 fase y neutro, 2 fases o 3 fases |
| `3F-3H` sin neutro | 2 fases o 3 fases; nunca fase y neutro |

**Qué y cómo:**
1. **En el motor de cálculo** (`ieestudio-core`): una sola función que diga qué conexiones admite un sistema (por ejemplo `System::allows(kind)` y `System::kinds()`), deducida de sus tensiones (relación entre fases y neutro de 2 = monofásico de tres hilos; de √3 = trifásico). `reference_voltage`, el cálculo del proyecto y el cálculo rápido deben **rechazar con un error claro** la combinación imposible: se muestra «no evaluado: un sistema monofásico de tres hilos no tiene tres fases», nunca un número.
2. **En el cálculo rápido:** la lista «Conexión» ofrece solo las del sistema elegido, con su tensión (`1F · 120 V`, `2F · 240 V`), igual que la ficha del circuito. Si solo hay una, se muestra como texto, sin lista. Al cambiar el sistema, si la conexión ya no existe se pasa a la más cercana y se avisa. Para motores, la lista de hp sigue a la conexión (ya lo hace).
3. **En el alimentador de un tablero:** la «Conexión» se limita a las del sistema de donde se alimenta (la fuente, el tablero padre o el secundario de su transformador) y a las del tablero padre.
4. **En la ficha del circuito:** además del tablero, considerar el sistema (hoy se usa la tensión pero no la regla de 3 fases en monofásico).
5. **Fases A, B, C:** se eligen con botones excluyentes (A | B | C, o AB | BC | CA), no con tres casillas que dejan marcar una combinación inválida. Si la conexión ocupa todas las fases del tablero (3 fases en un tablero trifásico, 2 en uno de dos fases, 1 en uno de una), no se preguntan: se muestran. Solo se eligen cuando hay opciones reales (por ejemplo, un circuito de 1 fase en un tablero de 3). «Lleva neutro» solo aparece si el sistema tiene neutro.
6. **Proyectos existentes con una combinación imposible:** deben abrir; el tramo queda «no evaluado» con el motivo y un botón para corregirlo. Agregar el aviso a Revisar.
7. **Barrido del mismo defecto:** revisar todos los formularios (proyecto, tablero, circuito, cálculo rápido, transformador, acometida) y listar cada campo que pide algo que se deduce de otro. Corregir los claros; los dudosos, presentárselos a Antonio en una lista corta.

**Pruebas:** una por cada renglón de la tabla de arriba, en el motor; una de que el cálculo rápido con `1F-3H` y 3 fases devuelve error; una de lectura de un proyecto viejo con la combinación imposible.

**Listo cuando:** en ninguna pantalla se puede elegir una conexión que el sistema no tiene, y el caso de la captura de Antonio (`1F-3H 240/120 V`, 61 700 W) ya no ofrece «3 fases».

### Etapa 1B — Claridad de estados

**El problema** (visto en las capturas del proyecto de ejemplo con 10 kA en el tablero): 6 de 7 circuitos dicen «Sin evaluar» solo porque la falla por bus infinito no es concluyente; el pie dice «0 cumplen» aunque cable, interruptor y caída estén bien; en Revisar, 11 de 13 pendientes son el mismo aviso; el paso Revisar marca «2», la lista 13 y la Memoria 2 + 11; en la vista avanzada el alimentador dice «Cumple» y su tablero «No cumple».

**Qué y cómo:**
1. **Dos preguntas separadas.** El estado de un circuito o alimentador responde solo a su cálculo propio (carga, conductor, protección, caída, tierra, tubería). La **capacidad interruptiva** es una verificación del tablero: un solo aviso por tablero con la corriente de falla, los interruptores que no la alcanzan o no tienen dato, y el campo para capturar los kA ahí mismo. El dictamen global no cambia de criterio: mientras ese aviso exista, el proyecto no se marca conforme (nada se marca conforme si falta una verificación). En la ficha del circuito queda como un renglón, no como el recuadro principal.
2. **Un solo conteo de pendientes** calculado en un lugar y usado por la barra de pasos, el pie, Revisar, la Memoria y el PDF. Se cuentan asuntos por resolver, no verificaciones internas.
3. **Dos palabras para lo que no es cumple / no cumple:** «Sin evaluar» (falta un dato o la regla no cubre el caso) y «No concluyente» (se calculó con un método conservador y no alcanza para afirmar). Quitar «no evaluado» y «pendiente» como sinónimos en la interfaz; «Pendiente» queda solo para la aprobación profesional.
4. **Nada encimado ni cortado a 1280 px ni a 1024 px:** la barra de pasos reduce a número los pasos que no caben; la columna Estado es un icono con color, fija a la derecha, con el texto en la ayuda y en la ficha; revisar el encabezado de la vista avanzada solo para quitar los textos encimados.
5. **Textos engañosos:** «Responsable y cédula capturados» cuando faltan → «Falta responsable y cédula»; «0 contactos» → «sin contar»; el pie del cálculo rápido debe decir que su dictamen es el de ese cálculo (hoy se confunde con el del proyecto, que aparece justo debajo); lo que no cumple va en rojo con su icono también en la lista de la Memoria.
6. **Desbalance visible:** cuando pasa del límite del perfil, es un aviso con el botón «Balancear», no letra gris.
7. **Contraste en tema claro:** el amarillo de acento no se usa como color de texto sobre fondo claro.

**Pruebas:** el conteo único tiene pruebas con el proyecto de ejemplo; una prueba de que un circuito con todo bien y sin kA concluyente aparece como «Cumple» y el tablero lleva el aviso; capturas a 1280 y 1024 px sin recortes.

**Listo cuando:** en el proyecto de ejemplo los circuitos correctos se ven en verde, el único problema real (C-04) destaca, y el mismo número de pendientes aparece en todas las pantallas.

### Etapa 2 — Rendimiento, incluido el PDF

**Qué y cómo:**
1. **Unifilar en memoria.** Guardar el diagrama junto con el número de revisión del documento (`Doc::revision`), el modo y las opciones; reconstruirlo solo cuando cambien. Aplica al panel Unifilar, a la miniatura de Revisar y a la exportación.
2. **Dibujo solo de lo visible.** En el unifilar y en las tablas, no enviar a pantalla lo que queda fuera del área visible.
3. **Recalcular sin trabar la escritura.** Mientras el usuario arrastra un número o escribe, recalcular como máximo una vez por cuadro; si con el proyecto grande pasa de 30 ms, recalcular al soltar. Evaluar recálculo parcial (solo el tablero editado y sus ancestros) únicamente si la medición lo justifica.
4. **Sin copias por cuadro.** Buscar los `clone()` de resultados dentro de funciones de dibujo y cambiarlos por referencias o por valores guardados por revisión.
5. **PDF:** generarlo fuera del hilo de la interfaz en escritorio (con indicador de avance y botón cancelar) y en el navegador sin congelar la pantalla (por partes o en un Web Worker). Reutilizar entre generaciones lo que no cambia (fuentes y plantilla ya cargadas). Guardar en memoria el último PDF por revisión para que «vista previa» y «guardar» no lo generen dos veces.
6. **Tamaño y arranque en el navegador:** medir el `.wasm`, activar optimización por tamaño si no rompe tiempos, y mostrar una pantalla de carga con avance.
7. **Dividir los archivos grandes** de la interfaz por pantalla o por sección, sin cambiar comportamiento (un commit aparte, verificable con las capturas).

**Metas con el proyecto grande, en una laptop común:** cuadro de pantalla < 16 ms en todos los pasos; edición de un número sin retraso visible; PDF < 5 s en escritorio y sin congelar la pantalla en el navegador. Si una meta no se alcanza, repórtalo con números; no la declares cumplida.

**Listo cuando:** la tabla «antes / después» está en el PR y las capturas no cambiaron.

### Etapa 3 — Base para teléfono

**Punto de partida:** Antonio ya la abrió en su teléfono: abre, pero es la pantalla de escritorio encogida y con el aviso de «pensado para pantallas grandes». No esperes otra prueba suya para empezar; revisa tú con capturas a tamaño de teléfono y mándale el enlace cuando haya algo que probar.

**Qué y cómo:**
1. **Tres tamaños** según el ancho disponible: teléfono (< 700 px), tablet (700–1100 px) y escritorio. Un solo lugar del código decide el tamaño; las pantallas lo consultan.
2. **En teléfono, vista guiada por paso a pantalla completa:** barra inferior con los cinco pasos (Proyecto, Tableros, Circuitos, Revisar, Memoria); la ficha del tablero o del circuito se abre como hoja a pantalla completa con botón «Listo»; las tablas se vuelven listas de tarjetas (una por circuito con lo esencial y su estado).
3. **El cálculo rápido en teléfono** es la herramienta de obra más probable: datos arriba, resultado abajo, sin paneles lado a lado.
4. **Tacto:** controles de al menos 44 px, campos numéricos con teclado numérico, nada que dependa de pasar el cursor (cada ayuda emergente debe tener un botón «i» equivalente), desplazamiento por arrastre.
5. **Teclado en pantalla:** el campo activo no debe quedar tapado.
6. **Unifilar en teléfono:** pellizco para acercar, arrastre para mover, tocar un circuito abre su ficha.
7. **Archivos en teléfono:** abrir, guardar y compartir el `.iees` y el PDF con lo que el navegador permita; guardado automático local para no perder trabajo en obra sin señal.
8. **Instalable y sin conexión** (PWA): revisar que funcione sin red después de la primera carga.
9. Opción de captura `--tamano telefono|tablet` para revisar sin teléfono.
10. **Unifilar con lugar propio en la vista guiada:** botón «Abrir unifilar» a pantalla completa desde Revisar y Memoria, con Revisión / Plano y exportar. La miniatura ilegible de Revisar se cambia por un esquema simple de tableros con su estado.
11. **Quitar el aviso de pantallas grandes** (`crates/ieestudio-app/web/index.html`), corregir la pregunta «¿Funciona en el celular?» de `website/guia/index.html` y anotar el cambio en las decisiones 0017 y 0020. Solo cuando lo anterior funcione: no se quita el aviso antes de merecerlo.
12. La vista avanzada no se ofrece en teléfono.

**Regla desde aquí:** toda pantalla nueva de las etapas siguientes se entrega también en tamaño teléfono, con su captura.

**Listo cuando:** Antonio captura en su teléfono un proyecto chico de principio a fin (tablero, 5 circuitos con un motor, revisar, PDF) y un cálculo rápido, y lo confirma.

### Etapa 4 — Uso diario

1. **Avisos agrupados en Revisar.** El de capacidad interruptiva ya quedó por tablero en la etapa 1B; revisar si otros avisos repetidos merecen lo mismo.
2. **Tabla de circuitos:** que quepa completa a 1280 px (hoy se corta la columna Estado); elegir qué columnas ver; **editar en la celda** nombre, carga, longitud y fases; teclas Tab y Enter para moverse.
3. **Duplicar circuito y plantillas** («contactos de 180 VA × 6», «minisplit 1 t»), editables por el usuario.
4. **«Es otra carga» se guarda** en el proyecto para que el aviso no regrese.
5. **Excel de ida y vuelta completo:** que exportar e importar conserve tipo de carga, datos de motor y de placa, datos de catálogo y posiciones. Prueba: exportar → importar → el proyecto queda idéntico.

6. **Proyectos recientes** en la pantalla de inicio (en el navegador, los guardados localmente).
7. **Un solo botón para exportar el PDF** en Memoria (hoy hay tres).
8. **Paso Tableros:** aprovechar el espacio vacío del lienzo y que la ficha no obligue a desplazarse para lo común.
9. **Pasar a la vista guiada lo que solo tiene la avanzada:** el procedimiento completo del alimentador y elegir columnas.

**Listo cuando:** capturar 40 circuitos se puede hacer sin abrir la ficha de cada uno, y la prueba de ida y vuelta pasa.

### Etapa 5 — Cuadro de cargas con el formato de Antonio

**El formato base** (archivo `CUADRO_DE_CARGAS_CMAPAS.xlsx`, una hoja por tablero, horizontal):
- **Encabezado:** Nombre del tablero, Ubicación, Tipo de carga eléctrica (Normal, Emergencia), Voltaje de operación (`220 3F, 4H`), Marca y modelo.
- **Renglones:** uno por circuito, con «No. / Etapa» y «Circuito / Áreas».
- **Columnas de tipos de carga:** una por cada tipo de equipo del tablero («Luminaria STL 6004», «Contacto normal», «Contacto regulado»…), con su **símbolo**, sus **watts por unidad** y, en cada renglón, la **cantidad** de ese equipo en el circuito. En tableros generales las columnas son los subtableros.
- **Columnas de resultado:** Carga total (W), Fases A / B / C, Volts, Amp., I.T.M. (`1X15`, `3X100`), factor de corrección o de agrupamiento, I corregida, conductor de fase, neutro y tierra **en mm²**, longitud (m), % de caída, diámetro del tubo (mm).
- **Pie:** renglón «CARGA INSTALADA» con las sumas y los datos del alimentador; «Desbalanceo máximo entre fases» A-B, A-C y B-C.

**Lo que hay que construir:**
1. **Catálogo de cargas del proyecto** (dato nuevo): tipos de equipo con nombre, watts por unidad, factor de potencia opcional, tipo de carga (alumbrado, contactos…) y símbolo. Cada circuito puede llevar **cantidades por tipo**; su potencia sale de la suma. Capturar la potencia directa, como hoy, debe seguir funcionando. Para contactos, respetar la regla de 180 VA por salida ya existente: si hay conflicto entre los watts del catálogo y el mínimo de norma, decide Antonio.
2. **Datos nuevos del tablero:** ubicación y tipo de carga eléctrica (normal, emergencia, regulada).
3. **Vista «Cuadro de cargas» en la app** con ese mismo acomodo (matriz de cantidades editable), además de la tabla actual.
4. **Exportación a Excel con ese formato:** una hoja por tablero, encabezado, símbolos como imagen, **fórmulas vivas** (carga total = Σ cantidad × watts; amperes; sumas; desbalanceo) para que quien lo abra pueda seguir el cálculo; los valores que IEEStudio calcula con tablas de norma (conductor, interruptor, caída) van como valor con una nota de que vienen del programa. Tamaño de impresión ajustado a una hoja horizontal.
5. **Cuadro de cargas en el PDF** de la memoria con el mismo formato.
6. **Personalizable:** qué columnas aparecen y en qué orden, calibres en mm², AWG o ambos, textos del encabezado, y si lleva símbolos. Se guarda como parte de la plantilla de la etapa 6.
7. **Importar** un Excel con este formato para crear los circuitos (al menos el que exporta la app).

**Diferencias que debes señalarle a Antonio, no resolver solo:** su archivo calcula la caída con fórmulas y factores propios por renglón; IEEStudio usa su método con las tablas del perfil. El cuadro debe mostrar los resultados de IEEStudio; si quiere otra cosa, que lo decida.

**Listo cuando:** el proyecto de ejemplo exportado se ve como su archivo (compara captura contra captura) y Antonio lo aprueba.

### Etapa 6 — Membrete y plantilla de documentos

**Qué:** que cada usuario le dé su imagen a todo lo que sale del programa: memoria en PDF, hoja del cálculo rápido, cuadro de cargas (PDF y Excel) y cajetín del unifilar.

**La plantilla guarda:**
- Datos de la empresa: nombre, dirección, teléfono, correo, sitio, RFC opcional, responsable y cédula.
- Logotipo (PNG, JPG o SVG) y, opcional, un segundo logotipo (del cliente).
- Colores: principal, secundario y de tablas. Tipografía de entre las incluidas.
- **Portada:** generada (logotipo, título, obra, cliente, fecha, revisión) con dos o tres acomodos a elegir, **o una imagen o PDF de portada propio** que se inserta tal cual.
- **Membrete:** encabezado y pie de cada página (logotipo, datos, número de página, leyenda), o una hoja membretada propia como fondo.
- Qué secciones lleva la memoria y en qué orden; texto libre de introducción y de notas finales.
- Las preferencias del cuadro de cargas de la etapa 5.

**Cómo:**
- La plantilla es un archivo aparte (`.ieesplantilla`) que se guarda en la configuración de la app y se puede exportar e importar para compartirla en un despacho. El proyecto recuerda qué plantilla usa; si no está en esa máquina, usa la genérica y avisa.
- Editor de plantilla con **vista previa en vivo** de la portada y de una página interior.
- En Typst: convertir el preámbulo actual en una plantilla con parámetros; no duplicar el reporte.
- La plantilla genérica actual queda como predeterminada. La leyenda «No constituye aprobación profesional» **no se puede quitar**.
- Imágenes: limitar tamaño y reducirlas al guardarlas, para no inflar archivos ni alentar el PDF (liga con la etapa 2).

**Listo cuando:** con una plantilla de prueba (logotipo, colores y portada propios) la memoria, el cuadro de cargas y el unifilar salen con esa imagen, en escritorio y en el navegador.

### Etapa 7 — Unifilar fase 3

1. **DXF con bloques:** cada símbolo (interruptor, medidor, transformador, tablero, cada tipo de carga) como bloque con nombre, insertado con **atributos** editables (identificador, amperes, calibre, etc.). Capas con nombres claros por tipo de elemento. Probar abriéndolo en AutoCAD o, si no hay, en LibreCAD u ODA; decirle a Antonio cuál se usó.
2. **Tamaños de hoja:** carta, doble carta, 60×90 y 90×120 cm, horizontal; marco, cajetín y escala de texto ajustados a la hoja; cajetín con el logotipo de la plantilla.
3. **Diagramas grandes:** si no cabe legible en la hoja elegida, repartir en varias hojas (una por tablero principal o por subárbol) con referencias «continúa en hoja N».
4. **PDF del plano solo**, además del que va en la memoria.
5. Probar con varios niveles de subtableros y nombres largos; corregir textos encimados.

**Listo cuando:** el DXF del proyecto grande abre con bloques y atributos editables, y el PDF en doble carta es legible.

### Etapa 8 — Cierre de la 1.0

- Actualizar las capturas y la guía del sitio; revisar el instalador y su firma.
- Pendientes de norma que dependen de Antonio: firmar los casos de `fixtures/` (`reviewed_by`), cotejar la tabla de la NMX-J-116 y los símbolos con la NMX-J-136-ANCE-2019.
- Probar un proyecto real de Antonio de punta a punta contra su memoria original.
- Subir la versión, notas de la versión y lista de lo que queda para después.

## 5. Decisiones que Antonio debe tomar (pregúntale al llegar a cada una)

| Etapa | Decisión |
|---|---|
| 1 | La lista de campos dudosos que salga del barrido (qué más sobra en los formularios). |
| 5 | Si el catálogo de cargas reemplaza o convive con la captura por watts (recomendado: convive). |
| 5 | Calibres en mm², AWG o ambos por omisión (su formato usa mm²; la app usa AWG). |
| 5 | Qué hacer cuando los watts de un contacto del catálogo quedan por debajo de los 180 VA de la norma. |
| 5 | Que el cuadro muestre la caída de tensión de IEEStudio y no la de las fórmulas de su Excel. |
| 6 | Qué acomodos de portada quiere y si tiene un membrete propio para usar de ejemplo. |
| 7 | Con qué programa abre los DXF, para probar ahí. |

## 6. Cómo reportar al terminar cada etapa

Un mensaje corto con: qué cambió para quien usa el programa, capturas (escritorio y teléfono), la tabla de tiempos si aplica, qué quedó pendiente y por qué, y el enlace del pull request contra `main`.

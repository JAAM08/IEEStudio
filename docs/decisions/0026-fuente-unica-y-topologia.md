# 0026. Fuente única de datos eléctricos y propagación por la topología

**Estado:** aceptada (noviembre de 2026). El principio lo fijó el dueño del
proyecto. Las decisiones 0027 (estados y trazabilidad) y 0028 (motores y
cortocircuito) lo completan. El diagnóstico está en
`../auditoria-modelo-y-falla.md`.

## Problema

- **Transformador inventado.** Al activar un transformador, la interfaz
  creaba 45 kVA, 3 %, secundario 220Y/127 V y 3 m de conductores. En la
  acometida creaba 112.5 kVA, 3 % y 13.2 kV. Un motor nuevo nacía con 1 hp, y
  «Capturar» proponía 10 kA de capacidad interruptiva. Esos valores daban
  resultados con apariencia válida sin que nadie los hubiera capturado.
- **Nombres parecidos.** El transformador que alimenta toda la instalación y
  uno intermedio dentro de ella se describían casi igual. Eso invitaba a
  capturar dos veces el mismo equipo y a crear un transformador ficticio
  entre la acometida y el primer tablero. Ese equipo ficticio altera la
  tensión y la capacidad interruptiva.
- **Tres copias de la misma lógica.** El sistema de unas barras se deducía en
  tres lugares con código copiado. Un tablero nuevo nacía trifásico aunque el
  sistema no lo permitiera.

## Criterio

Un dato eléctrico se captura **una sola vez**, en el elemento y el nivel de
la instalación al que pertenece. Lo que depende de él lo obtiene del modelo
de conexiones. Un dato desconocido queda vacío y el resultado dice cuál
falta. Nunca se reemplaza por un valor que parezca válido.

**Transformador de la acometida** (`Source.substation`)
- Es el que alimenta toda la instalación desde media tensión, sea de CFE o
  propio.
- Su secundario es el sistema de suministro.
- La interfaz lo llama «La acometida tiene transformador (CFE o propio)» y
  aclara que no se repite en ningún tablero.

**Transformador intermedio** (`Panel.transformer`)
- Es uno adicional dentro de la instalación.
- Su primario es el alimentador del tablero, a la tensión del tablero de
  donde sale.
- Su secundario fija la tensión del tablero y de todo lo que cuelga de él.
- La interfaz lo llama «Un transformador intermedio alimenta este tablero» y
  aclara que el de CFE no va ahí.

**Tensión de cada barra**
- La deduce una sola función, `connection::bus_system`, que usan el cálculo,
  la interfaz y el unifilar.
- Los circuitos heredan la tensión de su tablero.
- Un tablero nuevo toma la conexión de más fases que existe donde se
  alimenta (decisión 0024).

**Datos que empiezan vacíos**
- Del transformador: potencia, impedancia, longitud del secundario y
  tensión del primario.
- La falla de la suministradora.
- Las capacidades interruptivas.
- La potencia de un motor.
- Al activar un transformador intermedio, el secundario empieza con la
  tensión de donde sale (relación 1:1), para que el usuario elija la suya.
- Sin potencia, el transformador no se dimensiona.
- Sin potencia o sin impedancia, su falla no se estima.
- Los campos aceptan coma decimal y solo valores mayores que cero.

### Posibles transformadores duplicados

La revisión (`review::duplicate_flags`) se basa en la topología y en los
datos, y nunca bloquea nada.

- **Relación 1:1.** El secundario tiene la tensión del primario. Puede ser un
  transformador de aislamiento, que es legítimo, así que se avisa y el texto
  ofrece las dos salidas.
- **Relación 1:1 junto a la acometida.** Si el transformador sale directo del
  punto de conexión y la acometida ya tiene transformador, el aviso dice que
  parece el mismo equipo.
- **Misma potencia junto a la acometida.** Se avisa solo si sale directo del
  punto de conexión, la acometida tiene transformador y los dos tienen la
  misma potencia. Dos transformadores de igual potencia en otros lugares no se
  marcan.
- **Confirmación.** «Son equipos distintos» (en Revisar o en la ficha del
  tablero) guarda `Transformer::confirmed_distinct` con el proyecto, y el
  aviso ya no sale. El cálculo no cambia con la confirmación.

## Alternativas consideradas

- **Valores típicos marcados como «supuestos».** Se descartó: igual producen
  números que parecen resultados.
- **Rechazar la relación 1:1 o la potencia repetida.** Se descartó porque son
  configuraciones legítimas.
- **Hacer `secondary` opcional en el modelo.** Era más estricto, pero obliga
  a cambiar cada consumidor del sistema de barras. Empezar en 1:1 con el
  aviso logra lo mismo: no se calcula con una tensión que el usuario no
  eligió sin que se avise.

## Limitaciones

- **Captura todavía incompleta.** Las longitudes de 0 m se aceptan, y los
  circuitos y alimentadores nuevos nacen con 1000 W y 10 m. Se corrigen con
  el rediseño de la captura.
- **Conexión del secundario.** El secundario de un transformador intermedio
  usa la conexión de su primario. Falta un dato de conexión propio del
  secundario.

## Consecuencias para otros módulos

- Los archivos guardados con potencia o impedancia numérica se siguen
  leyendo igual. Los nuevos omiten los datos vacíos.
- El unifilar escribe «POTENCIA SIN DATO» o «Z = SIN DATO» en vez de un
  número.
- La memoria dice que el transformador no tiene potencia capturada.
- La línea de comandos y la memoria muestran los avisos de duplicado.

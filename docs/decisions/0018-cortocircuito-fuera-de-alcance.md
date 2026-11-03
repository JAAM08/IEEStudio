# 0018. El cortocircuito sale de IEEStudio y pasa a la suite

**Estado:** aceptada (noviembre de 2026). Sustituye a la 0013 y al punto de
cortocircuito de la 0016.

## Contexto

La 0013 metió el cálculo de cortocircuito en el proyecto: corriente de falla
en cada tablero por el método de impedancias, aportación de motores y
comparación con la capacidad interruptiva de cada interruptor. Con eso la
interfaz pedía datos en cuatro lugares (fuente, criterios, tablero,
alimentador y secundario del transformador) y mostraba resultados en otros
cinco, mezclados con el diseño de los circuitos. Para quien diseña la
instalación no quedaba claro qué parte era cortocircuito.

Además, un estudio de cortocircuito serio (fallas de fase a tierra,
coordinación de protecciones, arco eléctrico) y una malla de tierras son
programas completos por sí mismos. Hacerlos bien dentro de IEEStudio lo
volvería un programa interminable.

## Decisión

- **IEEStudio diseña la instalación de baja tensión**: carga, conductores,
  protección por sobrecorriente, tierra de equipos, tubería, caída de tensión,
  motores, transformadores, unifilar y memoria. El cortocircuito y la malla de
  tierras serán programas aparte de una suite eléctrica, reimplementados desde
  cero a partir de las normas públicas.
- **Se retira** el cálculo de falla (`short_circuit`, `fault`), sus datos en
  el proyecto, en la interfaz, en la memoria, en Excel, en el unifilar y en
  el servidor MCP, con sus pruebas y su caso de referencia.
- **Se queda** la protección de motores contra cortocircuito (430-52): es el
  tamaño del interruptor del motor, no un estudio de falla. También la puesta
  a tierra de equipos (250-122) y el conductor al electrodo (250-66): son
  calibres, no una malla.
- **Conforme en lo evaluado.** La norma exige verificar la capacidad
  interruptiva (110-9, 110-10) y el programa ya no lo hace. Para no declarar
  revisado lo que no se revisó, el veredicto lleva la lista de lo que queda
  fuera de su alcance, con su referencia. Si todo lo demás cumple, la
  verificación normativa dice «Conforme en lo evaluado» y nombra lo que falta:
  «Fuera de alcance: capacidad interruptiva, requiere estudio de
  cortocircuito». La memoria lo muestra en Datos y criterios de diseño, y las
  propiedades del proyecto en su propia sección.
- **Archivos anteriores.** Los `.iees` con datos de falla y los libros de
  Excel con la columna «Cap. interruptiva kA» se abren sin error: esos datos
  se ignoran y ya no se escriben. El formato sigue siendo el 3, así que un
  programa anterior abre lo que escribe este. Las preferencias guardadas con
  la columna «Icc kA» visible se leen y solo se descarta esa columna.

## Punto de conexión con la suite

Cuando exista el programa de cortocircuito, entregará la corriente de falla
disponible en cada tablero. IEEStudio la importará como dato, sin calcularla,
y volverá a comparar cada interruptor con ella: la capacidad interruptiva
saldrá de la lista de fuera de alcance para los tableros que tengan el dato.
El intercambio (archivo o formato) se decide cuando el otro programa exista.

## Consecuencias

- El proyecto, la interfaz y la memoria son más simples, y el veredicto dice
  con exactitud qué se verificó.
- Un proyecto ya no se puede declarar conforme sin matices: siempre llevará
  la nota de fuera de alcance mientras no se importe la corriente de falla.
- La aprobación profesional sigue siendo de quien firma (0009); con la nota,
  sabe que debe resolver la capacidad interruptiva con otro estudio.

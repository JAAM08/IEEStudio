# 0015. Conductores en paralelo propuestos por el programa

**Estado:** aceptada (octubre de 2026).

## Contexto

En un alimentador grande, el menor calibre que cumple con un conductor por
fase puede ser de 1000 kcmil o más, o no caber en ningún tubo. En la práctica
se instalan dos o más conductores por fase. Hasta ahora el programa solo los
usaba si el usuario los fijaba, y todos los juegos iban en un mismo tubo, con
el ajuste por agrupamiento de seis o más portadores.

## Decisión

- **Un juego por tubo.** Con conductores en paralelo, cada juego (fases y
  neutro) va en su propio tubo con su conductor de puesta a tierra
  (250-122(f)). El agrupamiento se cuenta por tubo y la tubería se calcula
  para un juego; el resultado dice cuántos tubos iguales lleva. Una casilla
  permite poner todos los juegos en un solo tubo, con su ajuste.
- **El programa propone paralelos** cuando el usuario no fijó el calibre ni
  los conductores por fase y un solo conductor no sirve: ninguno cumple, no
  cabe en ningún tubo o es mayor que el calibre máximo de los criterios del
  proyecto (500 kcmil por omisión). Prueba de dos a ocho por fase y toma el
  menor número que cumple; los paralelos empiezan en 1/0 AWG (310-10(h)). El
  procedimiento dice por qué.
- El cálculo rápido usa el mismo criterio de un juego por tubo.

## Consecuencias

Un alimentador que antes quedaba sin tubería o con calibres poco prácticos
ahora se resuelve solo. El calibre máximo es una preferencia de diseño, no un
valor normativo, y vive en los criterios del proyecto.

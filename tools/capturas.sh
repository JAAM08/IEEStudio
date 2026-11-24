#!/usr/bin/env bash
# Captura la aplicación en las vistas y los tamaños de referencia del
# rediseño (etapas R1 a R3) y sale con error si alguna se cierra o no deja imagen.
#
# Uso: tools/capturas.sh [carpeta] [programa]
#   carpeta   donde quedan las imágenes y resumen.txt (target/capturas)
#   programa  ieestudio-app a probar (target/debug/ieestudio-app). La
#             compilación de depuración revisa los anchos negativos de egui y
#             se cierra con ellos; la optimizada solo los dibuja mal.
#
# Variables: TAMANOS, VISTAS y TEMAS cambian la matriz; PROYECTO usa otro
# archivo en lugar del ejemplo del programa.
#
# Requiere xvfb-run (en Linux) para dibujar sin pantalla.
set -uo pipefail
cd "$(dirname "$0")/.."

out=${1:-target/capturas}
app=${2:-target/debug/ieestudio-app}
mkdir -p "$out"

if [ "$app" = target/debug/ieestudio-app ]; then
  cargo build -q -p ieestudio-app || exit 1
fi
proyecto=${PROYECTO:-}
if [ -z "$proyecto" ]; then
  proyecto="$out/ejemplo.iees"
  cargo run -q -p ieestudio-cli -- ejemplo "$proyecto" > /dev/null || exit 1
fi

tamanos=${TAMANOS:-"1440x900 1024x768 768x1024 600x844 390x844"}
vistas=${VISTAS:-"inicio rapido proyecto tableros circuitos ficha unifilar revisar memoria procedimiento"}
temas=${TEMAS:-"oscuro claro"}

# Opciones de la aplicación para cada vista. «procedimiento» es el cuadro con
# el panel de procedimiento abierto abajo; «dividida», dos vistas a la vez;
# «ficha», la del circuito abierta (pantalla completa en teléfono, cajón en
# tableta); «inicio» y «rapido», las dos experiencias de la pantalla de inicio.
opciones() {
  case "$1" in
    tableros) echo "--vista tableros --seleccion TG-1" ;;
    circuitos) echo "--vista tableros --seleccion C-03" ;;
    unifilar) echo "--vista unifilar --seleccion C-03" ;;
    procedimiento) echo "--vista tableros --seleccion C-03 --procedimiento" ;;
    dividida) echo "--vista tableros --seleccion C-03 --dividir" ;;
    ficha) echo "--vista tableros --seleccion C-03 --ficha" ;;
    inicio) echo "--inicio" ;;
    rapido) echo "--rapido" ;;
    *) echo "--vista $1" ;;
  esac
}

resumen="$out/resumen.txt"
printf '%-10s %-10s %-7s %s\n' vista tamaño tema resultado > "$resumen"
fallas=0
for tam in $tamanos; do
  for vista in $vistas; do
    for tema in $temas; do
      nombre="$vista-$tam-$tema"
      imagen="$out/$nombre.png"
      rm -f "$imagen"
      extra=""
      [ "$tema" = claro ] && extra="--claro"
      # shellcheck disable=SC2046
      timeout 180 xvfb-run -a -s "-screen 0 ${tam}x24" "$app" "$proyecto" \
        $(opciones "$vista") $extra --tamano "$tam" --captura "$imagen" \
        > "$out/$nombre.log" 2>&1
      salida=$?
      if grep -q "panicked" "$out/$nombre.log"; then
        resultado="SE CERRÓ: $(grep -A1 panicked "$out/$nombre.log" | tail -1)"
        fallas=$((fallas + 1))
      elif [ $salida -ne 0 ] || [ ! -s "$imagen" ]; then
        resultado="SIN IMAGEN (salida $salida)"
        fallas=$((fallas + 1))
      else
        resultado="ok"
        rm -f "$out/$nombre.log"
      fi
      printf '%-10s %-10s %-7s %s\n' "$vista" "$tam" "$tema" "$resultado" | tee -a "$resumen"
    done
  done
done

total=$(($(wc -l < "$resumen") - 1))
echo "$((total - fallas)) de $total capturas sin fallas. Resumen en $resumen"
[ "$fallas" -eq 0 ]

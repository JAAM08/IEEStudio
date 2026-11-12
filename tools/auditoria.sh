#!/usr/bin/env bash
# Repite la batería de la etapa 0 del plan 1.0: los casos de las auditorías
# del PR #11 (Fable H-01 a H-18, Opus 17 a 20) y la compatibilidad de archivos
# con la versión anterior al PR #11.
#
#   tools/auditoria.sh             casos abiertos y su estado actual
#   tools/auditoria.sh anterior    además, lee los casos con la versión ef59295
#
# Un caso «FALLA» es un hallazgo que sigue abierto: el mensaje dice lo que hoy
# muestra el programa. Al corregirlo, su prueba pasa y se le quita el
# #[ignore] en el mismo PR.
set -euo pipefail
cd "$(dirname "$0")/.."
mode="${1:-}"

echo "== Base"
git log -1 --format='%h %s'
echo

echo "== Pruebas que no deben cambiar"
cargo test -q -p ieestudio-project --test auditoria_pr11 2>&1 | grep -E '^test result'
cargo test -q -p ieestudio-cli --test auditoria_salidas 2>&1 | grep -E '^test result'
echo

echo "== Hallazgos abiertos (FALLA = sigue abierto)"
for t in "ieestudio-project auditoria_pr11" "ieestudio-cli auditoria_salidas"; do
  read -r crate file <<<"$t"
  # Las pruebas abiertas fallan a propósito: su salida no detiene el script.
  { cargo test -p "$crate" --test "$file" -- --ignored --skip informe 2>&1 || true; } \
    | sed -n -e 's/^test \(.*\) \.\.\. FAILED$/  FALLA  \1/p' -e 's/^test \(.*\) \.\.\. ok$/  pasa   \1/p' \
    | sort -k2
done
echo

echo "== Estado actual de cada caso"
cargo test -q -p ieestudio-project --test auditoria_pr11 informe -- --ignored --nocapture 2>&1 | grep '^|'

if [ "$mode" = "anterior" ]; then
  echo
  echo "== Archivos actuales leídos con la versión anterior al PR #11 (ef59295)"
  dir=target/anterior-src
  [ -d "$dir" ] || git worktree add -f "$dir" ef59295 >/dev/null
  (cd "$dir" && CARGO_TARGET_DIR=../anterior cargo build -q --release -p ieestudio-cli)
  for f in fixtures/auditoria-pr11/*.iees; do
    if err=$(target/anterior/release/ieestudio calcular "$f" 2>&1 >/dev/null); then
      printf '  abre        %s\n' "$(basename "$f")"
    else
      printf '  NO ABRE     %s: %s\n' "$(basename "$f")" "$(echo "$err" | head -1)"
    fi
  done
fi

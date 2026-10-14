#!/usr/bin/env bash
# Arma ieestudio.com en target/sitio: la página de inicio (website/) y la
# aplicación en el navegador en /app, compilada a WebAssembly.
#
# Requiere el destino wasm32-unknown-unknown y wasm-bindgen-cli de la misma
# versión que el crate wasm-bindgen de Cargo.lock. No se usa wasm-opt: tarda más
# de diez minutos y, comprimido, el archivo queda casi igual.
set -euo pipefail
cd "$(dirname "$0")/.."

out=target/sitio
cargo build --release --target wasm32-unknown-unknown -p ieestudio-app

rm -rf "$out"
mkdir -p "$out/app"
cp -r website/. "$out/"
rm -f "$out/README.md"

wasm-bindgen --target web --no-typescript --out-dir "$out/app" --out-name ieestudio-app \
  target/wasm32-unknown-unknown/release/ieestudio-app.wasm
cp crates/ieestudio-app/web/* "$out/app/"
# Cada compilación tiene su propia versión de caché.
version=$(sha256sum "$out/app/ieestudio-app_bg.wasm" | cut -c1-12)
sed -i "s/__VERSION__/$version/" "$out/app/sw.js"

echo "Sitio listo en $out (aplicación: $(du -h "$out/app/ieestudio-app_bg.wasm" | cut -f1))"

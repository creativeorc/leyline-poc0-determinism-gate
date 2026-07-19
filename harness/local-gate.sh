#!/usr/bin/env bash
# Local gate (§6.5): build native + both wasm targets, run the locally-runnable
# cells (native + Node, plus wasmtime if installed), and cross-check their
# digests with compare.py. CI is the full 7-cell confirmation; this is the fast
# local loop. Goldens are compared too, once minted (T8).
set -euo pipefail
cd "$(dirname "$0")/.."

out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT

echo "building native + wasm targets..."
cargo build --release -p gate-cli >/dev/null
cargo build --release -p gate-wasm --target wasm32-unknown-unknown >/dev/null

annotate() { CELL="$1" python3 harness/ci/annotate.py; }

echo "cell A (native)..."
./target/release/gate-cli --json --runtime-version "$(uname -sr)" | annotate A > "$out/A.json"

echo "cell F (node/v8)..."
node harness/node/run.mjs --toolchain "$(rustc -V)" | annotate F > "$out/F.json"

if command -v wasmtime >/dev/null 2>&1; then
  echo "cell D (wasmtime)..."
  cargo build --release -p gate-cli --target wasm32-wasip1 >/dev/null
  wasmtime run target/wasm32-wasip1/release/gate-cli.wasm \
    --json --runtime-version "$(wasmtime --version)" | annotate D > "$out/D.json"
else
  echo "cell D (wasmtime): SKIPPED (wasmtime not on PATH)"
fi

golden=""
[ -f goldens/v0.json ] && golden="--golden goldens/v0.json"

echo
python3 harness/compare/compare.py hashes $golden "$out"/*.json

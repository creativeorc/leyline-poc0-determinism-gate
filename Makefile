# Local loop (§6.5). CI is confirmation, not the debugger.
# `make gate` builds native + both wasm targets, runs the native + Node (+
# wasmtime if present) harnesses, and compares against goldens.
#
# Wired so far: build, run, lint, test, wasm, f5. Full `gate` (compare vs
# goldens across all cells) lands with T5–T6.

.PHONY: build run lint test wasm f5 gate clean

build:
	cargo build --release

# Native digests as JSON (the informal dev reference; formal goldens are minted
# via the ceremony, not here).
run:
	cargo run --release -p gate-cli -- --json

lint:
	cargo clippy --workspace --all-targets -- -D warnings
	cargo fmt --all --check

test:
	cargo test --workspace

wasm:
	cargo build --release -p gate-cli --target wasm32-wasip1
	cargo build --release -p gate-wasm --target wasm32-unknown-unknown

# Run the wasip1 build under wasmtime (cell D). Needs wasmtime on PATH.
run-wasmtime: wasm
	wasmtime run target/wasm32-wasip1/release/gate-cli.wasm --json

# Run the unknown-unknown build under Node/V8 (cell F). Needs node on PATH.
run-node: wasm
	node harness/node/run.mjs --toolchain "$$(rustc -V)"

# F5 inverted check: covenant lints MUST reject the fixture (§8, AC4).
f5:
	bash fixtures/f5-lint/expect-red.sh

# Local gate (§6.5): build + run local cells + compare.py cross-check.
gate:
	bash harness/local-gate.sh

clean:
	cargo clean
	rm -rf gate-out

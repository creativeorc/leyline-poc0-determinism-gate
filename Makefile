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

# F5 inverted check: covenant lints MUST reject the fixture (§8, AC4).
f5:
	bash fixtures/f5-lint/expect-red.sh

# TODO(T5–T6): gate = build + wasm + run native/Node/wasmtime + compare.py vs goldens
gate:
	@echo "make gate: full local gate lands in T3–T6 (see docs/POC0-SPEC.md §6.5)."

clean:
	cargo clean
	rm -rf gate-out

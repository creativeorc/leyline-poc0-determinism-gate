// Node/V8 harness (cell F/G) — §6.3. Loads the wasm32-unknown-unknown cdylib
// with an EMPTY import object, runs each seed through run_gate(), reads the
// 32-byte digest from exported memory, and emits the same JSON shape as
// `gate-cli --json` with "runtime": "node". Plain Node, no bundler, no npm.
//
// T1 STUB. Real implementation lands in T5 (needs gate-wasm's run_gate export).

// TODO(T5):
//   const bytes = await readFile(wasmPath);
//   const { instance } = await WebAssembly.instantiate(bytes, {}); // zero imports (Q6)
//   for (const seed of SEEDS) { run_gate(seed, ptr); read 32 bytes; hex; }
//   print JSON: { generator_version, target, runtime: "node",
//                 runtime_version: process.version + " v8 " + process.versions.v8,
//                 host_arch: process.arch, repeats: 3, seeds: {...} }

console.error('run.mjs is a T1 stub; implement in T5 (see docs/POC0-SPEC.md §6.3).');
process.exit(2);

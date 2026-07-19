// Node/V8 harness (cells F/G) — §6.3.
//
// Loads the `wasm32-unknown-unknown` cdylib with an EMPTY import object (Q6:
// verified zero imports), runs each seed through `run_gate` (which does the same
// 3-repeat in-process self-check as gate-cli), reads the 32-byte digest from
// exported memory, and prints the same JSON shape as `gate-cli --json` with
// "runtime": "node". Plain Node, no bundler, no npm.
//
// Usage: node run.mjs [--wasm <path>] [--toolchain <rustc -V>]

import { readFileSync } from "node:fs";
import { argv, arch, exit, version, versions } from "node:process";

// Canonical seeds (§5.1), SAME ORDER as gate-runner::SEEDS. The count is
// cross-checked against the wasm module below; a value drift would surface as a
// digest mismatch in the fan-in.
const SEEDS = [
  0x0000000000000000n,
  0x0000000000000001n,
  0x00000000ffffffffn,
  0xffffffffffffffffn,
  0x8000000000000000n,
  0x9e3779b97f4a7c15n,
  0x0123456789abcdefn,
  0xdeadbeefcafebaben,
  0x5eed5eed5eed5eedn,
  0x0f1e2d3c4b5a6978n,
];

function parseArgs() {
  let wasm = "target/wasm32-unknown-unknown/release/gate_wasm.wasm";
  let toolchain = "unknown";
  for (let i = 2; i < argv.length; i++) {
    if (argv[i] === "--wasm") wasm = argv[++i];
    else if (argv[i] === "--toolchain") toolchain = argv[++i];
    else {
      console.error(`run.mjs: unknown argument: ${argv[i]}`);
      exit(2);
    }
  }
  return { wasm, toolchain };
}

function fail(msg) {
  console.error(`run.mjs: ${msg}`);
  exit(1);
}

const { wasm, toolchain } = parseArgs();

const bytes = readFileSync(wasm);
// Zero imports (Q6). If this ever throws about a missing import, stub it here
// and prove it is unreachable on green runs — do not paper over numerics.
const { instance } = await WebAssembly.instantiate(bytes, {});
const { memory, alloc, run_gate, generator_version, seed_count } =
  instance.exports;

if (Number(seed_count()) !== SEEDS.length) {
  fail(`seed count mismatch: wasm ${seed_count()} vs harness ${SEEDS.length}`);
}

const ptr = alloc(32);
const seeds = {};
for (const seed of SEEDS) {
  const rc = run_gate(seed, ptr);
  if (rc !== 0) {
    fail(`SELF-CHECK FAILED: in-process nondeterminism for seed 0x${seed
      .toString(16)
      .padStart(16, "0")} (run_gate rc=${rc})`);
  }
  const view = new Uint8Array(memory.buffer, ptr, 32);
  const hex = Array.from(view, (b) => b.toString(16).padStart(2, "0")).join("");
  seeds[`0x${seed.toString(16).padStart(16, "0")}`] = hex;
}

// Same harness, two engines: Node runs V8; Bun runs JavaScriptCore (WebKit) —
// the engine the iPad's WKWebView ships, so the Bun cell is our JSC proxy.
const isBun = Boolean(versions.bun);
const out = {
  generator_version: Number(generator_version()),
  target: "wasm32-unknown-unknown",
  runtime: isBun ? "bun" : "node",
  toolchain,
  runtime_version: isBun
    ? `bun ${versions.bun} jsc ${versions.webkit ?? "?"}`
    : `node ${version} v8 ${versions.v8}`,
  host_arch: arch,
  repeats: 3,
  seeds,
};
console.log(JSON.stringify(out));

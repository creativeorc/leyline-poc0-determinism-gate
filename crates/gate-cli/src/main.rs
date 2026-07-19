//! gate-cli — native + `wasm32-wasip1` harness (§6.1).
//!
//! Runs all canonical seeds 3× in-process (fails fast on any per-process
//! disagreement) and reports per-seed SHA-256 digests.
//!
//! Flags:
//! - `--json`: emit the canonical JSON (else a human summary)
//! - `--repeats <n>`: in-process repeats for the self-check (default 3)
//! - `--transcript-out <dir>`: also dump raw transcripts (`seed_<hex>.bin`)
//! - `--runtime-version <s>`: runtime version for metadata (CI supplies e.g. the wasmtime version; default "unknown")
//!
//! Target triple and toolchain come from `build.rs` (compile-time), so no
//! `std::env`/`std::process` on the runtime path — keeps the wasm builds clean.

use std::process::ExitCode;

use gate_runner::{Digest32, SEEDS, run_all, to_hex, transcript};

const GATE_TARGET: &str = env!("GATE_TARGET");
const GATE_RUSTC: &str = env!("GATE_RUSTC");

/// "native" for host builds; "wasmtime" for the `wasm32-wasip1` build run under
/// wasmtime. (The `wasm32-unknown-unknown` / Node cell uses `gate-wasm`, not this.)
fn runtime() -> &'static str {
    if cfg!(target_family = "wasm") {
        "wasmtime"
    } else {
        "native"
    }
}

struct Args {
    json: bool,
    repeats: usize,
    transcript_out: Option<String>,
    runtime_version: String,
}

fn parse_args() -> Result<Args, String> {
    let mut json = false;
    let mut repeats = 3usize;
    let mut transcript_out = None;
    let mut runtime_version = "unknown".to_owned();

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--repeats" => {
                let v = it.next().ok_or("--repeats needs a value")?;
                repeats = v.parse().map_err(|_| format!("bad --repeats: {v}"))?;
            }
            "--transcript-out" => {
                transcript_out = Some(it.next().ok_or("--transcript-out needs a directory")?);
            }
            "--runtime-version" => {
                runtime_version = it.next().ok_or("--runtime-version needs a value")?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Args {
        json,
        repeats,
        transcript_out,
        runtime_version,
    })
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out
}

fn print_json(args: &Args, digests: &[(u64, Digest32)]) {
    let seeds = digests
        .iter()
        .map(|(seed, d)| format!("\"{seed:#018x}\":\"{}\"", to_hex(d)))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"generator_version\":{gv},\"target\":\"{target}\",\"runtime\":\"{runtime}\",\
         \"toolchain\":\"{toolchain}\",\"runtime_version\":\"{rtv}\",\"host_arch\":\"{arch}\",\
         \"repeats\":{repeats},\"seeds\":{{{seeds}}}}}",
        gv = gate_runner::GENERATOR_VERSION,
        target = json_escape(GATE_TARGET),
        runtime = runtime(),
        toolchain = json_escape(GATE_RUSTC),
        rtv = json_escape(&args.runtime_version),
        arch = std::env::consts::ARCH,
        repeats = args.repeats,
    );
}

fn print_human(digests: &[(u64, Digest32)]) {
    println!(
        "gate-cli {} / {} / {}  (gen v{})",
        GATE_TARGET,
        runtime(),
        GATE_RUSTC,
        gate_runner::GENERATOR_VERSION,
    );
    for (seed, d) in digests {
        println!("  {seed:#018x}  {}", to_hex(d));
    }
}

fn dump_transcripts(dir: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for &seed in &SEEDS {
        let path = format!("{dir}/seed_{seed:016x}.bin");
        std::fs::write(path, transcript(seed))?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("gate-cli: {e}");
            return ExitCode::from(2);
        }
    };

    if let Some(dir) = &args.transcript_out
        && let Err(e) = dump_transcripts(dir)
    {
        eprintln!("gate-cli: writing transcripts to {dir}: {e}");
        return ExitCode::FAILURE;
    }

    match run_all(args.repeats) {
        Ok(digests) => {
            if args.json {
                print_json(&args, &digests);
            } else {
                print_human(&digests);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            // Per-process nondeterminism — the fast fail (§6.1). Red cell.
            eprintln!("gate-cli: SELF-CHECK FAILED: {e}");
            ExitCode::FAILURE
        }
    }
}

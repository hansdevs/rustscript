# ═══════════════════════════════════════════════════════════════
#  RustScript Turbo — Factorial Benchmark (Optimized)
#
#  Uses native binary-splitting factorial() — same strategy as
#  Python's math.factorial (implemented in C), but in Rust.
#
#  Run:   rustscript run examples/benchmark.rsx
#  Stop:  Ctrl+C  or  rustscript stop
# ═══════════════════════════════════════════════════════════════

import turbo

print("╔═══════════════════════════════════════════════════════╗")
print("║  RustScript Turbo — Factorial Benchmark               ║")
print("║  Binary-splitting (native Rust) vs sequential         ║")
print("╚═══════════════════════════════════════════════════════╝")
print("")

# ── Test targets ──────────────────────────────────────────────
let targets = [100, 1000, 5000, 10000, 25000, 50000, 100000, 250000, 500000, 1000000]

let results = []

print("  Method          n          digits       time(ms)")
print("  ─────────────── ────────── ──────────── ──────────")

for n in targets {
    # ── Native factorial (binary splitting) ───────────────
    let t0 = timestamp()
    let f = factorial(n)
    let t1 = timestamp()
    let native_ms = t1 - t0
    let digits = digit_count(f)
    let mem = mem_usage(f)

    print("  native          {n}       {digits}      {native_ms}")

    let entry = {
        "n": n,
        "digits": digits,
        "native_ms": native_ms,
        "mem_bytes": mem
    }
    results = push(results, entry)
}

print("")
print("  ─────────────────────────────────────────────────────")

# ── Also run sequential loop for comparison on smaller values ─
print("")
print("  Sequential loop comparison (small n):")
print("  n          loop(ms)     native(ms)   speedup")
print("  ────────── ──────────── ──────────── ────────")

let seq_targets = [1000, 5000, 10000, 25000, 50000]
for n in seq_targets {
    # Sequential: 1*2*3*...*n
    let t0 = timestamp()
    let fact = bigint(1)
    let i = 1
    while i <= n {
        fact = fact * i
        i += 1
    }
    let t1 = timestamp()
    let loop_ms = t1 - t0

    # Native
    let t2 = timestamp()
    let f2 = factorial(n)
    let t3 = timestamp()
    let nat_ms = t3 - t2

    if nat_ms > 0 {
        let speedup = loop_ms / nat_ms
        print("  {n}       {loop_ms}          {nat_ms}          {speedup}x")
    } else {
        print("  {n}       {loop_ms}          {nat_ms}          ∞x")
    }
}

# ── Save results ──────────────────────────────────────────────
let total_end = timestamp()
write_json("benchmark_results.json", {
    "benchmark": "factorial",
    "language": "RustScript + Turbo",
    "engine": "binary-splitting factorial, base-10^9 limbs, Rust u128 mul",
    "results": results
})
print("")
print("  ── results saved to benchmark_results.json ──")

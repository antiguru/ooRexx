//! Generates random `digits|a|op|b` arithmetic cases for differential testing.
//!
//! The curated value lists that drove Tasks 2.3 and 2.4 tested ~35,000 cases
//! and still missed three defects, all of them at the exponent extremes --
//! including one that panicked. Hand-picked inputs test what the author
//! thought of; this tests what they did not.
//!
//! Output goes to stdout for the interpreter and the Rust harness to consume,
//! so the oracle is invoked once per batch rather than once per case.
//! Deterministic given a seed.

use std::fmt::Write as _;

/// xorshift64*, so the generator has no dependencies and a run is
/// reproducible from its seed alone.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

/// Builds one numeric literal, biased toward the shapes that break things:
/// exponents at the representable edge, long digit strings, trailing zeros.
fn literal(rng: &mut Rng) -> String {
    let mut out = String::new();
    if rng.below(2) == 0 {
        out.push('-');
    }
    let len = 1 + rng.below(12) as usize;
    for i in 0..len {
        // Bias the leading digit away from zero sometimes, but not always --
        // leading zeros are their own edge case.
        let d = if i == 0 && rng.below(4) > 0 { 1 + rng.below(9) } else { rng.below(10) };
        let _ = write!(out, "{d}");
    }
    if rng.below(3) == 0 {
        let point = rng.below(out.len() as u64) as usize;
        out.insert(point.max(1), '.');
    }
    match rng.below(6) {
        0 => {} // no exponent
        1 => {
            // Near the representable edge, where the interpreter switches
            // between a value, an overflow, and an unconvertible literal.
            let edge = 999_999_999i64 - rng.below(15) as i64;
            let sign = if rng.below(2) == 0 { "" } else { "-" };
            let _ = write!(out, "e{sign}{edge}");
        }
        2 => {
            // Deliberately past the edge: these must be rejected.
            let over = 999_999_999i64 + 1 + rng.below(1_000) as i64;
            let sign = if rng.below(2) == 0 { "" } else { "-" };
            let _ = write!(out, "e{sign}{over}");
        }
        _ => {
            let e = rng.below(40) as i64 - 20;
            let _ = write!(out, "e{e}");
        }
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let count: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(20_000);
    let mut rng = Rng(seed.max(1));

    const OPS: [&str; 6] = ["+", "-", "*", "/", "%", "//"];
    const DIGITS: [u32; 9] = [1, 2, 3, 5, 7, 9, 12, 18, 30];

    for _ in 0..count {
        let d = rng.pick(&DIGITS);
        let a = literal(&mut rng);
        let op = rng.pick(&OPS);
        let b = literal(&mut rng);
        println!("{d}|{a}|{op}|{b}");
    }
}

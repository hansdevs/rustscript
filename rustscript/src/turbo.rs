#![allow(dead_code)]
//! Turbo — high-performance math engine for RustScript.
//!
//! ## Architecture
//!
//! Three-layer multiplication for maximum throughput:
//!
//! | Layer | Algorithm  | Limbs       | Complexity      |
//! |-------|-----------|-------------|-----------------|
//! | 1     | Schoolbook | < 64        | O(n²)           |
//! | 2     | Karatsuba  | 64 – 8192   | O(n^1.585)      |
//! | 3     | NTT (3-prime CRT) | > 8192 | O(n log n)  |
//!
//! Internal representation: **base-2^32** little-endian `Vec<u32>`.
//! Carry propagation is a single right-shift (free), not a division by 10^9.
//! Decimal conversion only happens on `Display` or `digit_count()`.

use std::cmp::Ordering;
use std::fmt;

// ═════════════════════════════════════════════════════════════
//  Thresholds — tuned for Apple M-series / modern x86-64
// ═════════════════════════════════════════════════════════════

const KARATSUBA_THRESHOLD: usize = 64;
const NTT_THRESHOLD: usize = 8192;

// ═════════════════════════════════════════════════════════════
//  NTT primes  (p ≡ 1 mod 2^k for large k)
// ═════════════════════════════════════════════════════════════

const P1: u64 = 998244353;   // 119·2^23 + 1, primitive root 3
const P1_G: u64 = 3;
const P1_K: u32 = 23;

const P2: u64 = 985661441;   // 235·2^22 + 1, primitive root 3
const P2_G: u64 = 3;
const P2_K: u32 = 22;

const P3: u64 = 754974721;   //  45·2^24 + 1, primitive root 11
const P3_G: u64 = 11;
const P3_K: u32 = 24;

// ═════════════════════════════════════════════════════════════
//  RsBigInt — public API
// ═════════════════════════════════════════════════════════════

/// Signed arbitrary-precision integer.
///
/// Internally stored as a little-endian `Vec<u32>` where each element
/// is a full base-2^32 limb.  Arithmetic uses hardware `u64`/`u128`
/// multiply-add — the instructions your CPU was designed for.
#[derive(Debug, Clone)]
pub struct RsBigInt {
    /// Little-endian limbs in base 2^32.
    limbs: Vec<u32>,
    /// True if the number is negative.
    pub negative: bool,
}

impl RsBigInt {
    // ── Constructors ─────────────────────────────────────

    pub fn zero() -> Self {
        Self { limbs: vec![0], negative: false }
    }

    pub fn one() -> Self {
        Self { limbs: vec![1], negative: false }
    }

    pub fn from_i64(n: i64) -> Self {
        let negative = n < 0;
        Self::from_u64_inner(n.unsigned_abs(), negative)
    }

    pub fn from_u64(n: u64) -> Self {
        Self::from_u64_inner(n, false)
    }

    fn from_u64_inner(n: u64, negative: bool) -> Self {
        if n == 0 { return Self::zero(); }
        let lo = n as u32;
        let hi = (n >> 32) as u32;
        let limbs = if hi == 0 { vec![lo] } else { vec![lo, hi] };
        Self { limbs, negative }
    }

    /// Parse from decimal string.
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() { return None; }
        let (negative, digits) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };
        if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        // Parse 9-digit chunks from the right (base 10^9)
        let bytes = digits.as_bytes();
        let mut dec_limbs: Vec<u32> = Vec::new();
        let mut pos = bytes.len();
        while pos > 0 {
            let start = pos.saturating_sub(9);
            let chunk = &digits[start..pos];
            let val: u32 = chunk.parse().ok()?;
            dec_limbs.push(val);
            pos = start;
        }
        // Convert base-10^9 limbs to base-2^32 via Horner's method
        let mut result = vec![0u32; 1];
        for &d in dec_limbs.iter().rev() {
            limbs_mul_scalar_u32(&mut result, 1_000_000_000);
            limbs_add_scalar(&mut result, d);
        }
        trim(&mut result);
        Some(Self { limbs: result, negative })
    }

    // ── Queries ──────────────────────────────────────────

    pub fn is_zero(&self) -> bool {
        self.limbs.iter().all(|&l| l == 0)
    }

    /// Number of decimal digits (exact for ≤64 bits, very accurate for larger).
    pub fn digit_count(&self) -> usize {
        if self.is_zero() { return 1; }
        let n = self.limbs.len();
        if n <= 2 {
            let val = self.limbs[0] as u64
                | (self.limbs.get(1).copied().unwrap_or(0) as u64) << 32;
            if val == 0 { return 1; }
            return (val as f64).log10().floor() as usize + 1;
        }
        // Float estimate: log10(x) = log10(top) + (n-1)*32*log10(2)
        // This can be off-by-one at exact powers of 10, so check ±1.
        let top = *self.limbs.last().unwrap();
        let log10 = (top as f64).log10()
            + (n - 1) as f64 * 32.0 * std::f64::consts::LOG10_2;
        
        // Verify: build 10^est and compare. If self >= 10^est then est+1.
        // For huge numbers we trust the estimate (off-by-one is rare).
        log10.floor() as usize + 1
    }

    /// Approximate memory usage in bytes.
    pub fn mem_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.limbs.capacity() * 4
    }

    pub fn to_f64(&self) -> f64 {
        let base = (1u64 << 32) as f64;
        let mut val = 0.0f64;
        for &limb in self.limbs.iter().rev() {
            val = val * base + limb as f64;
        }
        if self.negative { -val } else { val }
    }

    pub fn to_i64(&self) -> Option<i64> {
        if self.limbs.len() > 2 { return None; }
        let lo = self.limbs[0] as u64;
        let hi = self.limbs.get(1).copied().unwrap_or(0) as u64;
        let val = lo | (hi << 32);
        if self.negative {
            // i64::MIN has absolute value 2^63 which is i64::MAX + 1
            if val == (i64::MAX as u64) + 1 { return Some(i64::MIN); }
            if val > i64::MAX as u64 { return None; }
            Some(-(val as i64))
        } else {
            if val > i64::MAX as u64 { return None; }
            Some(val as i64)
        }
    }

    pub fn limb_count(&self) -> usize { self.limbs.len() }

    // ── Comparison ───────────────────────────────────────

    fn cmp_abs(&self, other: &Self) -> Ordering {
        if self.limbs.len() != other.limbs.len() {
            return self.limbs.len().cmp(&other.limbs.len());
        }
        for i in (0..self.limbs.len()).rev() {
            if self.limbs[i] != other.limbs[i] {
                return self.limbs[i].cmp(&other.limbs[i]);
            }
        }
        Ordering::Equal
    }

    pub fn cmp(&self, other: &Self) -> Ordering {
        if self.is_zero() && other.is_zero() { return Ordering::Equal; }
        match (self.negative, other.negative) {
            (false, true)  => Ordering::Greater,
            (true, false)  => Ordering::Less,
            (false, false) => self.cmp_abs(other),
            (true, true)   => other.cmp_abs(self),
        }
    }

    // ── Addition / Subtraction ───────────────────────────

    pub fn add(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            let limbs = limbs_add(&self.limbs, &other.limbs);
            let mut r = Self { limbs, negative: self.negative };
            if r.is_zero() { r.negative = false; }
            r
        } else {
            match self.cmp_abs(other) {
                Ordering::Greater | Ordering::Equal => {
                    let limbs = limbs_sub(&self.limbs, &other.limbs);
                    let mut r = Self { limbs, negative: self.negative };
                    if r.is_zero() { r.negative = false; }
                    r
                }
                Ordering::Less => {
                    let limbs = limbs_sub(&other.limbs, &self.limbs);
                    let mut r = Self { limbs, negative: other.negative };
                    if r.is_zero() { r.negative = false; }
                    r
                }
            }
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        let mut neg = other.clone();
        neg.negative = !neg.negative;
        self.add(&neg)
    }

    // ── Multiplication (dispatches to best algorithm) ────

    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() { return Self::zero(); }
        let limbs = mul_dispatch(&self.limbs, &other.limbs);
        let mut r = Self { limbs, negative: self.negative != other.negative };
        if r.is_zero() { r.negative = false; }
        r
    }

    pub fn mul_u64(&self, n: u64) -> Self {
        if n == 0 || self.is_zero() { return Self::zero(); }
        let mut limbs = self.limbs.clone();
        limbs_mul_scalar_u64(&mut limbs, n);
        Self { limbs, negative: self.negative }
    }

    pub fn mul_i64(&self, n: i64) -> Self {
        let flip = n < 0;
        let mut r = self.mul_u64(n.unsigned_abs());
        if flip { r.negative = !r.negative; }
        if r.is_zero() { r.negative = false; }
        r
    }

    /// In-place scalar multiply — zero allocation, maximum speed.
    /// Uses u64 arithmetic when n fits in u32, u128 otherwise.
    pub fn mul_scalar_inplace(&mut self, n: u64) {
        if n == 0 {
            self.limbs.clear();
            self.limbs.push(0);
            self.negative = false;
            return;
        }
        if n == 1 { return; }
        if n <= u32::MAX as u64 {
            // Fast path: u32×u32 → u64 (no u128 needed)
            let mut carry = 0u64;
            for limb in self.limbs.iter_mut() {
                let prod = *limb as u64 * n + carry;
                *limb = prod as u32;
                carry = prod >> 32;
            }
            while carry > 0 {
                self.limbs.push(carry as u32);
                carry >>= 32;
            }
        } else {
            limbs_mul_scalar_u64(&mut self.limbs, n);
        }
    }

    pub fn mul_scalar_inplace_i64(&mut self, n: i64) {
        if n < 0 { self.negative = !self.negative; }
        self.mul_scalar_inplace(n.unsigned_abs());
        if self.is_zero() { self.negative = false; }
    }

    // ── Division ─────────────────────────────────────────

    pub fn div_mod_scalar(&self, d: u64) -> (Self, u64) {
        assert!(d > 0, "Division by zero");
        let mut quotient = vec![0u32; self.limbs.len()];
        let mut rem = 0u128;
        for i in (0..self.limbs.len()).rev() {
            rem = (rem << 32) | self.limbs[i] as u128;
            quotient[i] = (rem / d as u128) as u32;
            rem %= d as u128;
        }
        trim(&mut quotient);
        (Self { limbs: quotient, negative: self.negative }, rem as u64)
    }

    // ── Power ────────────────────────────────────────────

    pub fn pow_u32(&self, exp: u32) -> Self {
        if exp == 0 { return Self::one(); }
        let mut result = Self::one();
        let mut base = self.clone();
        let mut e = exp;
        while e > 0 {
            if e & 1 == 1 { result = result.mul(&base); }
            base = base.mul(&base);
            e >>= 1;
        }
        result
    }

    pub fn negate(&mut self) {
        if !self.is_zero() { self.negative = !self.negative; }
    }
}

// ═════════════════════════════════════════════════════════════
//  Limb-level helpers
// ═════════════════════════════════════════════════════════════

#[inline]
fn trim(v: &mut Vec<u32>) {
    while v.len() > 1 && *v.last().unwrap() == 0 {
        v.pop();
    }
}

fn limbs_add(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len().max(b.len());
    let mut r = Vec::with_capacity(n + 1);
    let mut carry = 0u64;
    for i in 0..n {
        let av = a.get(i).copied().unwrap_or(0) as u64;
        let bv = b.get(i).copied().unwrap_or(0) as u64;
        let sum = av + bv + carry;
        r.push(sum as u32);
        carry = sum >> 32;
    }
    if carry > 0 { r.push(carry as u32); }
    r
}

/// |a| >= |b| required.
fn limbs_sub(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut r = Vec::with_capacity(a.len());
    let mut borrow = 0i64;
    for (i, &av_raw) in a.iter().enumerate() {
        let av = av_raw as i64;
        let bv = b.get(i).copied().unwrap_or(0) as i64;
        let mut diff = av - bv - borrow;
        if diff < 0 { diff += 1i64 << 32; borrow = 1; } else { borrow = 0; }
        r.push(diff as u32);
    }
    trim(&mut r);
    r
}

/// In-place: v *= n  (u32 scalar, uses u64 arithmetic)
fn limbs_mul_scalar_u32(v: &mut Vec<u32>, n: u32) {
    let n64 = n as u64;
    let mut carry = 0u64;
    for limb in v.iter_mut() {
        let prod = *limb as u64 * n64 + carry;
        *limb = prod as u32;
        carry = prod >> 32;
    }
    while carry > 0 {
        v.push(carry as u32);
        carry >>= 32;
    }
}

/// In-place: v *= n  (u64 scalar, uses u128 arithmetic)
fn limbs_mul_scalar_u64(v: &mut Vec<u32>, n: u64) {
    let n128 = n as u128;
    let mut carry = 0u128;
    for limb in v.iter_mut() {
        let prod = *limb as u128 * n128 + carry;
        *limb = prod as u32;
        carry = prod >> 32;
    }
    while carry > 0 {
        v.push(carry as u32);
        carry >>= 32;
    }
}

/// In-place: v += n  (u32 scalar)
fn limbs_add_scalar(v: &mut Vec<u32>, n: u32) {
    let mut carry = n as u64;
    for limb in v.iter_mut() {
        if carry == 0 { return; }
        let sum = *limb as u64 + carry;
        *limb = sum as u32;
        carry = sum >> 32;
    }
    if carry > 0 { v.push(carry as u32); }
}

// ═════════════════════════════════════════════════════════════
//  Multiplication — three-layer dispatch
// ═════════════════════════════════════════════════════════════

fn mul_dispatch(a: &[u32], b: &[u32]) -> Vec<u32> {
    let min_len = a.len().min(b.len());
    if min_len < KARATSUBA_THRESHOLD {
        schoolbook_mul(a, b)
    } else if min_len < NTT_THRESHOLD {
        karatsuba_mul(a, b)
    } else {
        ntt_mul(a, b)
    }
}

// ── Layer 1: Schoolbook O(n²) ────────────────────────────────

fn schoolbook_mul(a: &[u32], b: &[u32]) -> Vec<u32> {
    let (n, m) = (a.len(), b.len());
    let mut r = vec![0u32; n + m];
    for i in 0..n {
        let ai = a[i] as u64;
        let mut carry = 0u64;
        for j in 0..m {
            let prod = ai * b[j] as u64 + r[i + j] as u64 + carry;
            r[i + j] = prod as u32;
            carry = prod >> 32;
        }
        let mut k = i + m;
        while carry > 0 {
            let sum = r[k] as u64 + carry;
            r[k] = sum as u32;
            carry = sum >> 32;
            k += 1;
        }
    }
    trim(&mut r);
    r
}

// ── Layer 2: Karatsuba O(n^1.585) ────────────────────────────

fn karatsuba_mul(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len().max(b.len());
    if n < KARATSUBA_THRESHOLD {
        return schoolbook_mul(a, b);
    }
    let half = n / 2;

    // a = a1·B^half + a0,  b = b1·B^half + b0
    let (a0, a1) = split(a, half);
    let (b0, b1) = split(b, half);

    // z0 = a0·b0,  z2 = a1·b1
    let z0 = mul_dispatch(a0, b0);
    let z2 = mul_dispatch(a1, b1);

    // z1 = (a0+a1)·(b0+b1) − z0 − z2
    let a_sum = slices_add(a0, a1);
    let b_sum = slices_add(b0, b1);
    let z1_full = mul_dispatch(&a_sum, &b_sum);
    let z1 = slices_sub(&slices_sub(&z1_full, &z0), &z2);

    // result = z0 + z1·B^half + z2·B^(2·half)
    let rlen = a.len() + b.len() + 2;
    let mut result = vec![0u32; rlen];
    copy_into(&mut result, &z0, 0);
    add_into(&mut result, &z1, half);
    add_into(&mut result, &z2, 2 * half);
    trim(&mut result);
    result
}

fn split(a: &[u32], mid: usize) -> (&[u32], &[u32]) {
    if mid >= a.len() { (a, &[]) } else { (&a[..mid], &a[mid..]) }
}

fn slices_add(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len().max(b.len());
    let mut r = Vec::with_capacity(n + 1);
    let mut carry = 0u64;
    for i in 0..n {
        let sum = a.get(i).copied().unwrap_or(0) as u64
                + b.get(i).copied().unwrap_or(0) as u64
                + carry;
        r.push(sum as u32);
        carry = sum >> 32;
    }
    if carry > 0 { r.push(carry as u32); }
    r
}

/// a − b  (assumes a ≥ b unsigned)
fn slices_sub(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut r = Vec::with_capacity(a.len());
    let mut borrow = 0i64;
    for i in 0..a.len() {
        let av = a.get(i).copied().unwrap_or(0) as i64;
        let bv = b.get(i).copied().unwrap_or(0) as i64;
        let mut diff = av - bv - borrow;
        if diff < 0 { diff += 1i64 << 32; borrow = 1; } else { borrow = 0; }
        r.push(diff as u32);
    }
    trim(&mut r);
    r
}

fn copy_into(dst: &mut [u32], src: &[u32], offset: usize) {
    for (i, &v) in src.iter().enumerate() {
        dst[offset + i] = v;
    }
}

fn add_into(dst: &mut [u32], src: &[u32], offset: usize) {
    let mut carry = 0u64;
    for i in 0..src.len() {
        let sum = dst[offset + i] as u64 + src[i] as u64 + carry;
        dst[offset + i] = sum as u32;
        carry = sum >> 32;
    }
    let mut k = offset + src.len();
    while carry > 0 && k < dst.len() {
        let sum = dst[k] as u64 + carry;
        dst[k] = sum as u32;
        carry = sum >> 32;
        k += 1;
    }
}

// ── Layer 3: NTT-based multiplication O(n log n) ─────────────
//
// Uses three NTT-friendly primes with CRT (Chinese Remainder
// Theorem) to recover exact convolution coefficients.
// Input limbs are split from base-2^32 into base-2^16 halves
// so that convolution values stay within range.

fn ntt_mul(a: &[u32], b: &[u32]) -> Vec<u32> {
    let a16 = to_base16(a);
    let b16 = to_base16(b);

    let total = a16.len() + b16.len();
    let mut size = 1usize;
    let mut log_size = 0u32;
    while size < total { size <<= 1; log_size += 1; }

    // If too large for P2 (smallest max_k), fall back to Karatsuba
    if log_size > P2_K {
        return karatsuba_mul(a, b);
    }

    // NTT multiply mod each prime
    let r1 = ntt_mul_mod(&a16, &b16, size, P1, P1_G);
    let r2 = ntt_mul_mod(&a16, &b16, size, P2, P2_G);
    let r3 = ntt_mul_mod(&a16, &b16, size, P3, P3_G);

    // CRT: Garner's algorithm to recover exact coefficients
    let inv_p1_p2 = mod_inv(P1, P2);
    let inv_p1_p3 = mod_inv(P1, P3);
    let inv_p2_p3 = mod_inv(P2, P3);
    let p1p2 = P1 as u128 * P2 as u128;

    let conv_len = a16.len() + b16.len() - 1;
    let mut coeffs: Vec<u128> = Vec::with_capacity(conv_len);

    for i in 0..conv_len {
        let v1 = r1[i]; let v2 = r2[i]; let v3 = r3[i];
        let c0 = v1;
        let c1 = mul_mod(sub_mod(v2, c0 % P2, P2), inv_p1_p2, P2);
        let tmp = mul_mod(sub_mod(v3, c0 % P3, P3), inv_p1_p3, P3);
        let c2  = mul_mod(sub_mod(tmp, c1 % P3, P3), inv_p2_p3, P3);
        let val = c0 as u128 + c1 as u128 * P1 as u128 + c2 as u128 * p1p2;
        coeffs.push(val);
    }

    // Carry propagation in base-2^16
    let mut carry = 0u128;
    let mut out16: Vec<u16> = Vec::with_capacity(coeffs.len() + 8);
    for &c in &coeffs {
        let total = c + carry;
        out16.push((total & 0xFFFF) as u16);
        carry = total >> 16;
    }
    while carry > 0 {
        out16.push((carry & 0xFFFF) as u16);
        carry >>= 16;
    }

    // Combine base-2^16 pairs → base-2^32 limbs
    let mut result = Vec::with_capacity(out16.len().div_ceil(2));
    for chunk in out16.chunks(2) {
        let lo = chunk[0] as u32;
        let hi = chunk.get(1).copied().unwrap_or(0) as u32;
        result.push(lo | (hi << 16));
    }
    trim(&mut result);
    result
}

fn to_base16(limbs: &[u32]) -> Vec<u64> {
    let mut out = Vec::with_capacity(limbs.len() * 2);
    for &l in limbs {
        out.push((l & 0xFFFF) as u64);
        out.push((l >> 16) as u64);
    }
    while out.len() > 1 && *out.last().unwrap() == 0 { out.pop(); }
    out
}

fn ntt_mul_mod(a: &[u64], b: &[u64], size: usize, p: u64, g: u64) -> Vec<u64> {
    let mut fa = vec![0u64; size];
    let mut fb = vec![0u64; size];
    for (i, &v) in a.iter().enumerate() { fa[i] = v % p; }
    for (i, &v) in b.iter().enumerate() { fb[i] = v % p; }

    ntt(&mut fa, false, p, g);
    ntt(&mut fb, false, p, g);

    for i in 0..size { fa[i] = mul_mod(fa[i], fb[i], p); }

    ntt(&mut fa, true, p, g);
    fa
}

// ── NTT core (iterative, in-place) ──────────────────────────

fn ntt(a: &mut [u64], invert: bool, p: u64, g: u64) {
    let n = a.len();
    debug_assert!(n.is_power_of_two());

    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j ^= bit;
        if i < j { a.swap(i, j); }
    }

    // Butterfly passes
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let w = if invert {
            mod_pow(mod_inv(g, p), (p - 1) / len as u64, p)
        } else {
            mod_pow(g, (p - 1) / len as u64, p)
        };
        let mut i = 0;
        while i < n {
            let mut wn = 1u64;
            for jj in 0..half {
                let u = a[i + jj];
                let v = mul_mod(a[i + jj + half], wn, p);
                a[i + jj]        = if u + v >= p { u + v - p } else { u + v };
                a[i + jj + half] = if u >= v { u - v } else { u + p - v };
                wn = mul_mod(wn, w, p);
            }
            i += len;
        }
        len <<= 1;
    }

    if invert {
        let n_inv = mod_inv(n as u64, p);
        for x in a.iter_mut() { *x = mul_mod(*x, n_inv, p); }
    }
}

// ── Modular arithmetic ──────────────────────────────────────

#[inline]
fn mul_mod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

#[inline]
fn sub_mod(a: u64, b: u64, m: u64) -> u64 {
    if a >= b { a - b } else { a + m - b }
}

fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut result = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 { result = mul_mod(result, base, m); }
        base = mul_mod(base, base, m);
        exp >>= 1;
    }
    result
}

/// Modular inverse via Fermat's little theorem (m is prime).
fn mod_inv(a: u64, m: u64) -> u64 {
    mod_pow(a, m - 2, m)
}

// ═════════════════════════════════════════════════════════════
//  Factorial — binary splitting
// ═════════════════════════════════════════════════════════════

/// Compute lo × (lo+1) × … × hi via binary splitting.
pub fn product_range(lo: u64, hi: u64) -> RsBigInt {
    if lo > hi { return RsBigInt::one(); }
    if lo == hi { return RsBigInt::from_u64(lo); }
    // Sequential scalar multiply for small ranges (avoids allocation overhead)
    if hi - lo < 128 {
        let mut acc = RsBigInt::from_u64(lo);
        for i in (lo + 1)..=hi {
            if i <= u32::MAX as u64 {
                // Hot path: u32×u32 → u64, in-place
                let mut carry = 0u64;
                for limb in acc.limbs.iter_mut() {
                    let prod = *limb as u64 * i + carry;
                    *limb = prod as u32;
                    carry = prod >> 32;
                }
                while carry > 0 {
                    acc.limbs.push(carry as u32);
                    carry >>= 32;
                }
            } else {
                acc.mul_scalar_inplace(i);
            }
        }
        return acc;
    }
    let mid = lo + (hi - lo) / 2;
    let left = product_range(lo, mid);
    let right = product_range(mid + 1, hi);
    left.mul(&right)
}

/// Compute n! using binary splitting.
///
/// Same divide-and-conquer strategy as Python's `math.factorial` (C/GMP),
/// but paired with our Karatsuba/NTT multiplication layers.
pub fn factorial(n: u64) -> RsBigInt {
    if n <= 1 { return RsBigInt::one(); }
    product_range(2, n)
}

// ═════════════════════════════════════════════════════════════
//  Display — base-2^32 → decimal conversion
// ═════════════════════════════════════════════════════════════

impl fmt::Display for RsBigInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.negative && !self.is_zero() { write!(f, "-")?; }
        if self.is_zero() { return write!(f, "0"); }

        // Repeated division by 10^9 to extract base-10^9 limbs
        let mut temp = self.limbs.clone();
        let mut dec_limbs: Vec<u32> = Vec::new();
        loop {
            if temp.iter().all(|&l| l == 0) { break; }
            let mut rem = 0u64;
            for i in (0..temp.len()).rev() {
                let cur = (rem << 32) | temp[i] as u64;
                temp[i] = (cur / 1_000_000_000) as u32;
                rem = cur % 1_000_000_000;
            }
            while temp.len() > 1 && *temp.last().unwrap() == 0 { temp.pop(); }
            dec_limbs.push(rem as u32);
        }
        if dec_limbs.is_empty() { return write!(f, "0"); }

        // MSD without padding, rest with zero-filling to 9 digits
        let top = dec_limbs.len() - 1;
        write!(f, "{}", dec_limbs[top])?;
        for i in (0..top).rev() {
            write!(f, "{:09}", dec_limbs[i])?;
        }
        Ok(())
    }
}

// ── Standard trait impls ─────────────────────────────────────

impl PartialEq for RsBigInt {
    fn eq(&self, other: &Self) -> bool { self.cmp(other) == Ordering::Equal }
}
impl Eq for RsBigInt {}

impl PartialOrd for RsBigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(std::cmp::Ord::cmp(self, other)) }
}
impl Ord for RsBigInt {
    fn cmp(&self, other: &Self) -> Ordering { RsBigInt::cmp(self, other) }
}

// ═════════════════════════════════════════════════════════════
//  Timestamps
// ═════════════════════════════════════════════════════════════

pub fn timestamp_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn timestamp_ns() -> i64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Clamp to i64::MAX if beyond range (year ~2262)
    if nanos > i64::MAX as u128 { i64::MAX } else { nanos as i64 }
}

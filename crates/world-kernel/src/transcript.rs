//! Canonical transcript serialization (§5.6).
//!
//! ```text
//! header:  magic "WGT0" · GENERATOR_VERSION u32 LE · seed u64 LE
//! section: tag 4 bytes · payload_len u64 LE · payload
//! ```
//! All integers little-endian fixed-width; every f64 as `to_bits()` u64 LE;
//! collections length-prefixed u64 LE; no padding, no text. The writer asserts
//! `is_finite()` before every float write (R6) — a violation panics, the cell
//! fails, the gate is red. That assert is the defense, not a bug.

use crate::GENERATOR_VERSION;

const MAGIC: &[u8; 4] = b"WGT0";

/// Section tags (§5.6).
pub mod tag {
    pub const W1: &[u8; 4] = b"W1AR";
    pub const W2: &[u8; 4] = b"W2TR";
    pub const W3: &[u8; 4] = b"W3HF";
    pub const W4: &[u8; 4] = b"W4SO";
    pub const W5: &[u8; 4] = b"W5RD";
    pub const W6: &[u8; 4] = b"W6EP";
}

/// A growable byte buffer with canonical little-endian writers. Used both for
/// the whole transcript and for building each section's payload before it is
/// length-prefixed.
#[derive(Default)]
pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    #[must_use]
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// f64 as its IEEE bits, LE. R6: no NaN/inf ever crosses this boundary.
    pub fn f64_bits(&mut self, v: f64) {
        assert!(
            v.is_finite(),
            "R6 violation: non-finite f64 reached serialization ({v:?})"
        );
        self.u64(v.to_bits());
    }

    fn tag(&mut self, t: &[u8; 4]) {
        self.buf.extend_from_slice(t);
    }

    fn raw(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }
}

/// Builds a full transcript: header, then framed sections in fixed order.
pub struct Transcript {
    out: Writer,
}

impl Transcript {
    /// Write the header and start the transcript for `seed`.
    #[must_use]
    pub fn begin(seed: u64) -> Self {
        let mut out = Writer::new();
        out.tag(MAGIC);
        out.u32(GENERATOR_VERSION);
        out.u64(seed);
        Self { out }
    }

    /// Append one framed section: tag · payload_len u64 LE · payload.
    pub fn section(&mut self, tag: &[u8; 4], payload: Writer) {
        let bytes = payload.into_bytes();
        self.out.tag(tag);
        self.out.u64(bytes.len() as u64);
        self.out.raw(&bytes);
    }

    /// Finish and return the transcript bytes.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.out.into_bytes()
    }
}

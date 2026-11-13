//! The whitespace sift must give the same bytes as the sift it replaced.
//!
//! `support/legacy_sifter.rs` is the sifter as it stood before the single pass
//! rewrite, copied without edits. Every test here feeds both the same input and
//! compares the output byte for byte.
#![cfg(all(feature = "rewriter", feature = "stream"))]

use html2md::extended::sifter::{WhitespaceSifter, WhitespaceSifterBytes};

// The frozen file imports these three from its crate root.
pub(crate) const LINK_LABEL_OPEN: u8 = 0x02;
pub(crate) const LINK_LABEL_CLOSE: u8 = 0x03;
pub(crate) const fn is_link_label_marker(b: u8) -> bool {
    b == LINK_LABEL_OPEN || b == LINK_LABEL_CLOSE
}

#[allow(dead_code)]
#[path = "support/legacy_sifter.rs"]
mod legacy;

use legacy::WhitespaceSifterBytes as LegacyBytes;

fn legacy_lines(input: &[u8]) -> String {
    LegacyBytes::sift_bytes_preserve_newlines(&input)
}

fn legacy_flat(input: &[u8]) -> String {
    LegacyBytes::sift_bytes(&input)
}

fn new_lines(input: &[u8]) -> String {
    WhitespaceSifterBytes::sift_bytes_preserve_newlines(&input)
}

fn new_flat(input: &[u8]) -> String {
    WhitespaceSifterBytes::sift_bytes(&input)
}

/// xorshift64*, seeded, so a failure reproduces.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % (n as u64)) as usize
    }
}

fn assert_same(input: &[u8], what: &str) {
    let want = legacy_lines(input);
    let got = new_lines(input);
    assert!(
        want == got,
        "{what}: line sift differs for {input:02x?}\nlegacy {want:?}\nnew    {got:?}"
    );
    let want = legacy_flat(input);
    let got = new_flat(input);
    assert!(
        want == got,
        "{what}: flat sift differs for {input:02x?}\nlegacy {want:?}\nnew    {got:?}"
    );
}

/// Every scalar value, in the positions where the sift treats it differently:
/// alone, after text, after a space, at a line start and inside a link label.
#[test]
fn every_scalar_matches_legacy() {
    let mut failures = Vec::new();
    let mut buf = [0u8; 4];
    let mut input = Vec::with_capacity(32);

    for code in 0u32..=0x10_FFFF {
        let Some(ch) = char::from_u32(code) else {
            continue;
        };
        let encoded = ch.encode_utf8(&mut buf).as_bytes().to_vec();

        let frames: [(&[u8], &[u8]); 5] = [
            (b"", b""),
            (b"ab", b"cd"),
            (b"a ", b" b"),
            (b"a\n", b"\nb"),
            (b"[\x02 a ", b" b \x03](u)"),
        ];

        for (before, after) in frames {
            input.clear();
            input.extend_from_slice(before);
            input.extend_from_slice(&encoded);
            input.extend_from_slice(after);

            if legacy_lines(&input) != new_lines(&input) || legacy_flat(&input) != new_flat(&input)
            {
                failures.push(code);
                break;
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} scalars differ from the legacy sift, first ones: {:x?}",
        failures.len(),
        &failures[..failures.len().min(32)]
    );
}

/// The bytes the sift branches on, mixed with lead and continuation bytes, so
/// random strings hit invalid, overlong and truncated sequences often.
const ALPHABET: &[u8] = &[
    b'a', b'b', b'Z', b'1', b'.', b'[', b']', b'(', b')', b'*', b'|', b' ', b' ', b' ', b'\t',
    b'\n', b'\n', b'\r', 0x0C, 0x00, 0x01, 0x02, 0x03, 0x7F, 0x80, 0x89, 0xA0, 0xAF, 0xBB, 0xBF,
    0xC2, 0xC3, 0xE2, 0xE3, 0xEF, 0xF0, 0xF4, 0xFF,
];

const SNIPPETS: &[&str] = &[
    "\u{a0}",
    "\u{202f}",
    "\u{2009}",
    "\u{feff}",
    "é",
    "日本語",
    "😀",
    "\r\n",
    "[\u{2}",
    "\u{3}](https://example.com)",
    "word word",
    "  ",
    "\n\n",
];

fn random_input(rng: &mut Rng, max_len: usize) -> Vec<u8> {
    let target = rng.below(max_len + 1);
    let mut out = Vec::with_capacity(target + 16);
    while out.len() < target {
        if rng.below(4) == 0 {
            out.extend_from_slice(SNIPPETS[rng.below(SNIPPETS.len())].as_bytes());
        } else {
            out.push(ALPHABET[rng.below(ALPHABET.len())]);
        }
    }
    out
}

#[test]
fn random_bytes_match_legacy() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for round in 0..40_000 {
        let input = random_input(&mut rng, 48);
        assert_same(&input, &format!("round {round}"));
    }
}

/// A label longer than the fold limit goes back to keeping newlines.
#[test]
fn runaway_label_matches_legacy() {
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    for round in 0..40 {
        let mut input = b"intro\n[\x02".to_vec();
        let words = 600 + rng.below(600);
        for _ in 0..words {
            input.extend_from_slice(b"label ");
            if rng.below(7) == 0 {
                input.push(b'\n');
            }
            if rng.below(11) == 0 {
                input.extend_from_slice("\u{a0}é".as_bytes());
            }
        }
        if rng.below(2) == 0 {
            input.extend_from_slice(b"\x03](u)\nafter  text\n");
        }
        assert_same(&input, &format!("runaway {round}"));
    }
}

/// The `str` trait goes through the same code as the bytes trait.
#[test]
fn str_trait_matches_bytes_trait() {
    let mut rng = Rng(0x5555_aaaa_5555_aaaa);
    for _ in 0..2_000 {
        let bytes = random_input(&mut rng, 64);
        let text = String::from_utf8_lossy(&bytes).into_owned();
        assert_eq!(
            text.sift_preserve_newlines(),
            legacy_lines(text.as_bytes()),
            "input {text:?}"
        );
        assert_eq!(text.sift(), legacy_flat(text.as_bytes()), "input {text:?}");
    }
}

/// Raw rewriter output for a fixture: the bytes lol_html hands the sift, before
/// any cleanup, split the way the rewriter split them.
fn raw_rewriter_chunks(html: &str, commonmark: bool, chunk: usize) -> Vec<Vec<u8>> {
    let settings = html2md::rewriter::writer::get_rewriter_settings_send(commonmark, &None, None);
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    {
        let mut rewriter = lol_html::send::HtmlRewriter::new(settings.into(), |c: &[u8]| {
            pieces.push(c.to_vec());
        });
        for part in html.as_bytes().chunks(chunk) {
            rewriter.write(part).expect("rewrite");
        }
        rewriter.end().expect("rewrite end");
    }
    pieces
}

fn fixtures() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-samples");
    let mut found = Vec::new();
    for dir in [root.clone(), root.join("wiki")] {
        for entry in std::fs::read_dir(&dir)
            .expect("fixture directory")
            .flatten()
        {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "html") {
                let bytes = std::fs::read(&path).expect("fixture file");
                found.push((
                    path.display().to_string(),
                    String::from_utf8_lossy(&bytes).into_owned(),
                ));
            }
        }
    }
    found.sort();
    found
}

/// The real input of the sift: what the rewriter emits for each fixture.
#[test]
fn rewriter_output_matches_legacy() {
    for (name, html) in fixtures() {
        for commonmark in [false, true] {
            let raw: Vec<u8> = raw_rewriter_chunks(&html, commonmark, 8192).concat();
            let want = legacy_lines(&raw);
            let got = new_lines(&raw);
            assert!(want == got, "{name} commonmark={commonmark}: sift differs");
        }
    }
}

/// navi cuts the raw output at newlines that sit outside a link label and
/// sifts each piece by itself. Cut the same way, both sifts must agree piece
/// by piece.
#[test]
fn newline_cut_segments_match_legacy() {
    for (name, html) in fixtures() {
        let raw: Vec<u8> = raw_rewriter_chunks(&html, true, 8192).concat();
        for (index, segment) in raw.split_inclusive(|b| *b == b'\n').enumerate() {
            let want = legacy_lines(segment);
            let got = new_lines(segment);
            assert!(want == got, "{name} segment {index}: sift differs");
        }
    }
}

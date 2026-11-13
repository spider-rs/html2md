use crate::{is_link_label_marker, LINK_LABEL_CLOSE, LINK_LABEL_OPEN};
use auto_encoder::auto_encode_bytes;

/// Character handling bytes.
enum Character {
    SingleByte { data: u8 },
    MultiByte { len: usize },
}

/// A trait containing all `string` whitespace-sifting functions.
pub trait WhitespaceSifter: AsRef<str> {
    /// Removes duplicate ASCII whitespaces (collapses runs).
    /// NOTE: This mode does NOT preserve newlines.
    #[must_use]
    fn sift(&self) -> String {
        let input: &str = self.as_ref();
        let mut out: String = String::with_capacity(input.len());
        sift_preallocated(input.as_bytes(), &mut out);
        out
    }

    /// Removes duplicate ASCII whitespaces but preserves deduplicated newlines,
    /// trims spaces before newlines, and applies markdown post-fixes:
    /// - join bare list markers: "*\nText" -> "* Text", "1.\nText" -> "1. Text"
    /// - join table cell fragments: "|A|\nB|\nC|" -> "|A|B|C|"
    /// - ensure table rows start with '|': "a|b|c|" -> "|a|b|c|"
    #[must_use]
    fn sift_preserve_newlines(&self) -> String {
        let bytes = self.as_ref().as_bytes();
        let mut sink = SiftSink::with_capacity(bytes.len());
        sink.push(bytes);
        sink.finish()
    }
}

/// A trait containing all `Vec<u8>` whitespace-sifting functions.
pub trait WhitespaceSifterBytes: AsRef<[u8]> {
    /// Removes duplicate ASCII whitespaces (collapses runs).
    #[must_use]
    fn sift_bytes(&self) -> String {
        let input = self.as_ref();
        let mut out: String = String::with_capacity(input.len());
        sift_preallocated(input, &mut out);
        out
    }

    /// Removes duplicate ASCII whitespaces but preserves deduplicated newlines,
    /// trims spaces before newlines, and applies markdown post-fixes:
    /// - join bare list markers: "*\nText" -> "* Text", "1.\nText" -> "1. Text"
    /// - join table cell fragments: "|A|\nB|\nC|" -> "|A|B|C|"
    /// - ensure table rows start with '|': "a|b|c|" -> "|a|b|c|"
    #[must_use]
    fn sift_bytes_preserve_newlines(&self) -> String {
        let bytes = self.as_ref();
        let mut sink = SiftSink::with_capacity(bytes.len());
        sink.push(bytes);
        sink.finish()
    }
}

impl<T: AsRef<str>> WhitespaceSifter for T {}
impl<T: AsRef<[u8]>> WhitespaceSifterBytes for T {}

/// A custom implementation of `str::trim_start` (ASCII whitespace only).
fn sift_trim_start(bytes: &[u8], ind: &mut usize, out: &mut String) {
    while *ind < bytes.len() {
        match get_char_metadata(bytes[*ind]) {
            Character::SingleByte { data } => {
                *ind += 1;
                if !is_ascii_whitespace(data) {
                    out.push(data as char);
                    break;
                }
            }
            Character::MultiByte { len } => {
                // Multi-byte char is not ASCII whitespace; emit and stop trimming.
                let _ = extend_from_bytes_with_len(bytes, ind, out, len);
                break;
            }
        }
    }
}

/// A custom implementation for `str::trim_end` (removes one trailing ASCII space if pending).
fn sift_trim_end(out: &mut String, is_last_whitespace: bool) {
    if is_last_whitespace {
        out.pop();
    }
}

/// Write one multibyte sequence, `slice`, whose length came from its first byte.
/// Returns `true` if the sequence was normalized as whitespace (e.g., NBSP -> ' ').
///
/// A well formed character is copied as it is. The byte order mark and any
/// malformed sequence go through the encoding detector, which drops the mark
/// and repairs the rest, as this sift has always done.
#[inline]
fn push_sequence(slice: &[u8], out: &mut String) -> bool {
    match slice {
        // NBSP U+00A0, narrow NBSP U+202F, thin space U+2009.
        [0xC2, 0xA0] | [0xE2, 0x80, 0xAF] | [0xE2, 0x80, 0x89] => {
            out.push(' ');
            true
        }
        BYTE_ORDER_MARK => {
            out.push_str(&auto_encode_bytes(slice));
            false
        }
        _ => {
            match core::str::from_utf8(slice) {
                Ok(text) => out.push_str(text),
                Err(_) => out.push_str(&auto_encode_bytes(slice)),
            }
            false
        }
    }
}

/// U+FEFF in UTF-8.
const BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Extend bytes for a multibyte UTF-8 sequence.
/// Returns `true` if the sequence was normalized as whitespace (e.g., NBSP -> ' ').
#[inline]
fn extend_from_bytes_with_len(bytes: &[u8], ind: &mut usize, out: &mut String, len: usize) -> bool {
    let end = ind.saturating_add(len);

    // A sequence cut off by the end of the input writes nothing.
    let was_whitespace = match bytes.get(*ind..end) {
        Some(slice) => push_sequence(slice, out),
        None => false,
    };

    *ind = end;
    was_whitespace
}

#[inline]
const fn is_newline(codepoint: u8) -> bool {
    matches!(codepoint, LINE_FEED | CARRIAGE_RETURN)
}

/// Sift preallocated safe strings (collapses ASCII whitespace runs).
fn sift_preallocated(bytes: &[u8], out: &mut String) {
    if bytes.is_empty() {
        return;
    }

    let mut ind: usize = 0;
    sift_trim_start(bytes, &mut ind, out);

    let mut is_last_whitespace: bool = false;
    let mut is_last_carriage_return: bool = false;

    while ind < bytes.len() {
        match get_char_metadata(bytes[ind]) {
            Character::SingleByte { data } => {
                ind += 1;

                if is_ascii_whitespace(data) {
                    // CRLF special-case (legacy)
                    if data == LINE_FEED && is_last_carriage_return {
                        out.push('\n');
                        is_last_carriage_return = false;
                        is_last_whitespace = false;
                        continue;
                    }

                    if is_last_whitespace {
                        is_last_carriage_return = data == CARRIAGE_RETURN;
                        continue;
                    }

                    is_last_whitespace = true;
                    is_last_carriage_return = data == CARRIAGE_RETURN;
                    out.push(data as char);
                } else {
                    is_last_whitespace = false;
                    is_last_carriage_return = false;
                    out.push(data as char);
                }
            }
            Character::MultiByte { len } => {
                let was_ws = extend_from_bytes_with_len(bytes, &mut ind, out, len);
                is_last_whitespace = was_ws;
                is_last_carriage_return = false;
            }
        }
    }

    sift_trim_end(out, is_last_whitespace);
}

/// Check if byte is a "regular" ASCII char (not whitespace, not high bit set).
#[inline]
const fn is_regular_ascii(b: u8) -> bool {
    b > 0x20 && b < 0x7F // printable ASCII excluding space
}

/// Longest link label, in output bytes, that the sift folds onto one line.
/// Past this the label is treated as unclosed (an `<a>` the page never ends)
/// and newlines are kept again, so a stray marker cannot flatten a page.
const LINK_LABEL_MAX: usize = 4096;

/// First bytes whose sequences need a look at the whole character: U+00A0
/// starts with `C2`, U+2009 and U+202F with `E2`, and U+FEFF with `EF`. A run
/// of plain multibyte text stops at any of them.
#[inline]
const fn is_checked_lead(b: u8) -> bool {
    matches!(b, 0xC2 | 0xE2 | 0xEF)
}

/// The newline-preserving sift as a sink: bytes go in through [`SiftSink::push`]
/// in pieces of any size, cut at any byte, and the sifted markdown comes out of
/// [`SiftSink::finish`]. The rewriter writes straight into it, so the markdown
/// is cleaned as it is produced and never scanned a second time.
///
/// What it does to the bytes:
/// - drops whitespace and link label markers at the start of each line, which
///   also folds a run of newlines into one;
/// - folds each run of spaces, tabs and form feeds into one space, and drops
///   that space before a newline and at the end of the input;
/// - turns U+00A0, U+202F and U+2009 into a space;
/// - drops one trailing newline.
///
/// Link labels: the rewriter writes `[` plus `LINK_LABEL_OPEN` before an
/// anchor and `LINK_LABEL_CLOSE` plus `](url)` after it. Between the two, every
/// whitespace run, newlines included, becomes one space, and the label is
/// trimmed at both ends, the way a browser renders inline anchor text. Both
/// markers are dropped.
///
/// Outside a link label no state survives a newline, so input cut at such a
/// newline and sifted piece by piece gives the same lines.
pub(crate) struct SiftSink {
    out: String,
    /// True until the first kept byte of the current line.
    line_start: bool,
    /// True when the last byte written is a folded space that a newline, a
    /// label close or the end of the input would remove.
    last_whitespace: bool,
    last_carriage_return: bool,
    /// Open link labels; 0 outside any label.
    label_depth: u32,
    /// `out.len()` where the outermost open label started.
    label_start: usize,
    /// True right after a label opens, until its first non-whitespace byte.
    label_trim: bool,
    /// The first bytes of a multibyte sequence cut by the end of a piece.
    carry: [u8; 4],
    carry_len: usize,
}

impl SiftSink {
    /// A sink whose output buffer holds `capacity` bytes before it grows.
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            out: String::with_capacity(capacity),
            line_start: true,
            last_whitespace: false,
            last_carriage_return: false,
            label_depth: 0,
            label_start: 0,
            label_trim: false,
            carry: [0; 4],
            carry_len: 0,
        }
    }

    /// Sift the next piece of input.
    pub(crate) fn push(&mut self, piece: &[u8]) {
        let bytes = self.finish_carry(piece);
        let len = bytes.len();
        let mut ind = 0usize;

        while ind < len {
            let b = bytes[ind];

            if b >= 0x80 {
                let end = ind + sequence_len(b);
                if end > len {
                    // The piece ends inside this sequence. Keep its first
                    // bytes for the next piece.
                    self.start_carry(&bytes[ind..]);
                    return;
                }

                if is_checked_lead(b) {
                    self.sequence(&bytes[ind..end]);
                    ind = end;
                } else {
                    ind = self.multibyte_run(bytes, ind, end);
                }
                continue;
            }

            if self.line_start {
                if is_ascii_whitespace(b) || is_link_label_marker(b) {
                    ind += 1;
                    continue;
                }
                // The first kept byte of the line. Every flag is clear here,
                // so the cases below write it as it is.
                self.line_start = false;
            }

            if is_regular_ascii(b) {
                ind = self.ascii_run(bytes, ind);
                continue;
            }

            ind += 1;

            if is_ascii_whitespace(b) {
                self.whitespace(b);
            } else if b == LINK_LABEL_OPEN {
                if self.label_depth == 0 {
                    self.label_start = self.out.len();
                }
                self.label_depth = self.label_depth.saturating_add(1);
                self.label_trim = true;
                // The `[` before the marker is never a space, so this is
                // already false. Keep the close-side pop label-local.
                self.last_whitespace = false;
                self.last_carriage_return = false;
            } else if b == LINK_LABEL_CLOSE {
                if self.label_depth > 0 {
                    // Trim the label's trailing space.
                    if self.last_whitespace {
                        self.out.pop();
                        self.last_whitespace = false;
                    }
                    self.label_depth -= 1;
                }
                self.label_trim = false;
                self.last_carriage_return = false;
            } else {
                self.last_whitespace = false;
                self.last_carriage_return = false;
                self.label_trim = false;
                self.out.push(b as char);
            }
        }
    }

    /// End the input and return the sifted text.
    pub(crate) fn finish(mut self) -> String {
        if self.carry_len != 0 && !self.line_start {
            // The input ends inside a multibyte sequence. Nothing is written
            // for it, and it still counts as the last thing on the line, so a
            // space before it stays.
            self.last_whitespace = false;
        }

        sift_trim_end(&mut self.out, self.last_whitespace);

        // Drop trailing newline(s)
        if self.out.ends_with("\r\n") {
            let _ = self.out.pop();
            let _ = self.out.pop();
        } else if self.out.ends_with('\n') {
            let _ = self.out.pop();
        }

        self.out
    }

    /// Complete a sequence the last piece left unfinished. Returns what is
    /// left of `piece`.
    #[inline]
    fn finish_carry<'a>(&mut self, piece: &'a [u8]) -> &'a [u8] {
        if self.carry_len == 0 {
            return piece;
        }

        let have = self.carry_len;
        let need = sequence_len(self.carry[0]);
        let take = need.saturating_sub(have).min(piece.len());
        let (head, rest) = piece.split_at(take);

        if let Some(slot) = self.carry.get_mut(have..have + take) {
            slot.copy_from_slice(head);
        }

        if have + take < need {
            self.carry_len = have + take;
            return rest;
        }

        self.carry_len = 0;
        let carry = self.carry;
        if let Some(sequence) = carry.get(..need) {
            self.sequence(sequence);
        }
        rest
    }

    /// Keep the first bytes of a sequence that the piece cut short.
    #[inline]
    fn start_carry(&mut self, head: &[u8]) {
        if let Some(slot) = self.carry.get_mut(..head.len()) {
            slot.copy_from_slice(head);
            self.carry_len = head.len();
        }
    }

    /// Write one whole multibyte sequence and update the line state.
    #[inline]
    fn sequence(&mut self, slice: &[u8]) {
        let was_last_whitespace = self.last_whitespace;
        let was_whitespace = push_sequence(slice, &mut self.out);

        if self.line_start {
            // A multibyte character always ends the line start trim, and a
            // space written for one there is kept.
            self.line_start = false;
            return;
        }

        if was_whitespace && self.label_depth > 0 && (self.label_trim || was_last_whitespace) {
            // Unicode space inside a label folds like ASCII space.
            self.out.pop();
            self.last_whitespace = was_last_whitespace;
        } else {
            self.last_whitespace = was_whitespace;
            self.label_trim = false;
        }
        self.last_carriage_return = false;
    }

    /// Copy a run of multibyte characters in one write. `ind` is the start of
    /// a whole sequence that ends at `end`, and its first byte is not a
    /// checked lead. Returns the index after the run.
    #[inline]
    fn multibyte_run(&mut self, bytes: &[u8], ind: usize, end: usize) -> usize {
        let len = bytes.len();
        let mut stop = end;

        while stop < len {
            let b = bytes[stop];
            if b < 0x80 || is_checked_lead(b) {
                break;
            }
            let next = stop + sequence_len(b);
            if next > len {
                break;
            }
            stop = next;
        }

        let run = &bytes[ind..stop];

        match core::str::from_utf8(run) {
            Ok(text) => {
                self.out.push_str(text);
                self.line_start = false;
                self.last_whitespace = false;
                self.last_carriage_return = false;
                self.label_trim = false;
            }
            Err(_) => {
                // Malformed bytes in the run. Take it one sequence at a time,
                // the lengths read from each first byte.
                let mut at = 0usize;
                while at < run.len() {
                    let next = (at + sequence_len(run[at])).min(run.len());
                    self.sequence(&run[at..next]);
                    at = next;
                }
            }
        }

        stop
    }

    /// Copy a run of printable ASCII in one write. Outside a link label the run
    /// also takes each single space that sits between two printable bytes,
    /// which is what a line of prose is made of. Returns the index after it.
    #[inline]
    fn ascii_run(&mut self, bytes: &[u8], ind: usize) -> usize {
        let len = bytes.len();
        // Inside a label a space also checks the label length, so it takes
        // the slow path there.
        let take_spaces = self.label_depth == 0;
        let mut stop = ind + 1;

        loop {
            while stop < len && is_regular_ascii(bytes[stop]) {
                stop += 1;
            }
            if take_spaces
                && stop + 1 < len
                && bytes[stop] == SPACE
                && is_regular_ascii(bytes[stop + 1])
            {
                stop += 2;
            } else {
                break;
            }
        }

        // Safety: every byte in the run is ASCII (0x20 to 0x7E), checked above.
        let run = unsafe { std::str::from_utf8_unchecked(&bytes[ind..stop]) };
        self.out.push_str(run);
        self.last_whitespace = false;
        self.last_carriage_return = false;
        self.label_trim = false;

        stop
    }

    /// One ASCII whitespace byte in the body of a line.
    #[inline]
    fn whitespace(&mut self, b: u8) {
        if self.label_depth > 0 {
            if self.out.len().saturating_sub(self.label_start) <= LINK_LABEL_MAX {
                self.last_carriage_return = false;
                if self.label_trim || self.last_whitespace {
                    return;
                }
                self.last_whitespace = true;
                self.out.push(' ');
                return;
            }
            // Runaway label: stop folding, keep newlines from here.
            self.label_depth = 0;
            self.label_trim = false;
        }

        if is_newline(b) {
            // Drop trailing ASCII space before newline (fixes " \n").
            if self.last_whitespace {
                self.out.pop();
            }
            if self.last_carriage_return {
                self.out.push('\r');
            }
            self.out.push('\n');

            // A new line starts with every flag clear.
            self.line_start = true;
            self.last_whitespace = false;
            self.last_carriage_return = false;
            return;
        }

        self.last_carriage_return = b == CARRIAGE_RETURN;

        if self.last_whitespace {
            return;
        }
        self.last_whitespace = true;
        self.out.push(' ');
    }
}

/// Length of the UTF-8 sequence a first byte announces. A continuation byte in
/// first position counts as the start of a two byte sequence.
#[inline]
const fn sequence_len(first_byte: u8) -> usize {
    match get_char_metadata(first_byte) {
        Character::SingleByte { .. } => 1,
        Character::MultiByte { len } => len,
    }
}

/// Binary extracted from `std`.
#[inline]
const fn get_char_metadata(first_byte: u8) -> Character {
    match first_byte {
        0b0000_0000..=0b0111_1111 => Character::SingleByte { data: first_byte },
        0b1000_0000..=0b1101_1111 => Character::MultiByte { len: 2 },
        0b1110_0000..=0b1110_1111 => Character::MultiByte { len: 3 },
        0b1111_0000..=0b1111_1111 => Character::MultiByte { len: 4 },
    }
}

#[allow(clippy::cast_possible_truncation)]
const SPACE: u8 = ' ' as u32 as u8;
#[allow(clippy::cast_possible_truncation)]
const HORIZONTAL_TAB: u8 = '\t' as u32 as u8;
#[allow(clippy::cast_possible_truncation)]
const LINE_FEED: u8 = '\n' as u32 as u8;
#[allow(clippy::cast_possible_truncation)]
const FORM_FEED: u8 = '\x0C' as u32 as u8;
#[allow(clippy::cast_possible_truncation)]
const CARRIAGE_RETURN: u8 = '\r' as u32 as u8;

/// ASCII whitespace definition (matches `char::is_ascii_whitespace`).
#[inline]
const fn is_ascii_whitespace(codepoint: u8) -> bool {
    matches!(
        codepoint,
        SPACE | HORIZONTAL_TAB | LINE_FEED | FORM_FEED | CARRIAGE_RETURN
    )
}

#[cfg(test)]
mod tests {
    use super::SiftSink;

    fn whole(input: &[u8]) -> String {
        let mut sink = SiftSink::with_capacity(input.len());
        sink.push(input);
        sink.finish()
    }

    fn pieces(input: &[u8], cuts: &[usize]) -> String {
        let mut sink = SiftSink::with_capacity(0);
        let mut from = 0usize;
        for cut in cuts {
            sink.push(&input[from..*cut]);
            from = *cut;
        }
        sink.push(&input[from..]);
        sink.finish()
    }

    const CASES: &[&[u8]] = &[
        b"",
        b"\n",
        b"  a  b  \n\n  c \t d  ",
        b"a \xE2",
        b"\xE2",
        b" \xF0\x9F",
        "caf\u{e9}  \u{65e5}\u{672c}\u{8a9e} \u{1f600}\n\n\u{a0}x\u{2009}\u{202f}y".as_bytes(),
        "\u{feff}a \u{feff} b\n\u{feff}".as_bytes(),
        b"[\x02  one \n two\t\x03](u) tail  \r\n next",
        "see [\x02 \u{a0} caf\u{e9} \u{a0}\n \x03](https://example.com/a b)\n".as_bytes(),
        b"\x80a\xC3\n\xFFzz\xE3\x81 x\xBF\xBF\xBF\xBF y",
        b"word word  word \n word\tword\x0Cword",
        b"[\x02 a \xE2",
        b"[\x02 a \xE3\x81\x82\xE3AB \x03](u)",
        "a \u{feff}".as_bytes(),
    ];

    /// Cutting the input anywhere, into two or three pieces, never changes
    /// the result.
    #[test]
    fn any_split_gives_the_same_output() {
        for case in CASES {
            let want = whole(case);
            for first in 0..=case.len() {
                assert_eq!(pieces(case, &[first]), want, "{case:02x?} cut at {first}");
                for second in first..=case.len() {
                    assert_eq!(
                        pieces(case, &[first, second]),
                        want,
                        "{case:02x?} cut at {first} and {second}"
                    );
                }
            }
        }
    }

    /// One byte at a time is the hardest split for the carried sequence.
    #[test]
    fn single_byte_pieces_give_the_same_output() {
        for case in CASES {
            let mut sink = SiftSink::with_capacity(0);
            for b in case.iter() {
                sink.push(std::slice::from_ref(b));
            }
            assert_eq!(sink.finish(), whole(case), "{case:02x?}");
        }
    }

    /// Seeded random bytes, malformed sequences included, cut into random
    /// pieces. Long enough to pass the label fold limit.
    #[test]
    fn random_bytes_in_random_pieces_give_the_same_output() {
        const ALPHABET: &[u8] = &[
            b'a', b'b', b'.', b'[', b']', b' ', b' ', b'\t', b'\n', b'\r', 0x0C, 0x02, 0x02, 0x03,
            0x80, 0xA0, 0xAF, 0xBB, 0xBF, 0xC2, 0xC3, 0xE2, 0xE3, 0xEF, 0xF0, 0xFF,
        ];
        let mut state: u64 = 0x2545_f491_4f6c_dd1d;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };

        for round in 0..300 {
            let len = (next() % 9000) as usize;
            let mut input = Vec::with_capacity(len);
            for _ in 0..len {
                input.push(ALPHABET[(next() % ALPHABET.len() as u64) as usize]);
            }

            let want = whole(&input);
            let mut sink = SiftSink::with_capacity(0);
            let mut from = 0usize;
            while from < input.len() {
                let size = 1 + (next() % 97) as usize;
                let to = (from + size).min(input.len());
                sink.push(&input[from..to]);
                from = to;
            }
            assert!(sink.finish() == want, "round {round}");
        }
    }

    /// A label longer than the fold limit, fed in uneven pieces.
    #[test]
    fn long_label_survives_any_piece_size() {
        let mut input = b"intro\n[\x02".to_vec();
        for n in 0..1500 {
            input.extend_from_slice(b"label ");
            if n % 7 == 0 {
                input.push(b'\n');
            }
            if n % 11 == 0 {
                input.extend_from_slice("\u{a0}\u{e9}\u{65e5}".as_bytes());
            }
        }
        input.extend_from_slice(b"\x03](u)\nafter  text\n");

        let want = whole(&input);
        for size in [1usize, 2, 3, 5, 7, 64, 251, 4096] {
            let mut sink = SiftSink::with_capacity(0);
            for piece in input.chunks(size) {
                sink.push(piece);
            }
            assert_eq!(sink.finish(), want, "piece size {size}");
        }
    }
}

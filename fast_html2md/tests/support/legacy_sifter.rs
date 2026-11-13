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
        let input = self.as_ref();
        let mut out = String::with_capacity(input.len());
        let bytes = input.as_bytes();
        let mut ind: usize = 0;

        while ind < bytes.len() {
            sift_preallocated_until_newline(bytes, &mut ind, &mut out);
        }

        // Drop trailing newline(s)
        if out.ends_with("\r\n") {
            let _ = out.pop();
            let _ = out.pop();
        } else if out.ends_with('\n') {
            let _ = out.pop();
        }

        out
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
        let mut out = String::with_capacity(bytes.len());
        let mut ind: usize = 0;

        while ind < bytes.len() {
            sift_preallocated_until_newline(bytes, &mut ind, &mut out);
        }

        // Drop trailing newline(s)
        if out.ends_with("\r\n") {
            let _ = out.pop();
            let _ = out.pop();
        } else if out.ends_with('\n') {
            let _ = out.pop();
        }

        out
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

/// Extend bytes for a multibyte UTF-8 sequence.
/// Returns `true` if the sequence was normalized as whitespace (e.g., NBSP -> ' ').
#[inline]
fn extend_from_bytes_with_len(bytes: &[u8], ind: &mut usize, out: &mut String, len: usize) -> bool {
    let end = ind.saturating_add(len);

    if *ind <= end && end <= bytes.len() {
        let slice = &bytes[*ind..end];

        // Normalize common Unicode "space-like" sequences to ASCII space.
        // NBSP U+00A0: C2 A0
        if slice == [0xC2, 0xA0] {
            out.push(' ');
            *ind = end;
            return true;
        }

        // Narrow NBSP U+202F: E2 80 AF
        if slice == [0xE2, 0x80, 0xAF] {
            out.push(' ');
            *ind = end;
            return true;
        }

        // Thin space U+2009: E2 80 89
        if slice == [0xE2, 0x80, 0x89] {
            out.push(' ');
            *ind = end;
            return true;
        }

        let output = auto_encode_bytes(slice);
        out.push_str(&output);
    }

    *ind = end;
    false
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

/// `sift_trim_start` for the newline-preserving sift. It also drops link
/// label markers met before the first real byte of a line.
fn sift_trim_start_line(bytes: &[u8], ind: &mut usize, out: &mut String) {
    while *ind < bytes.len() {
        match get_char_metadata(bytes[*ind]) {
            Character::SingleByte { data } => {
                *ind += 1;
                if !is_ascii_whitespace(data) && !is_link_label_marker(data) {
                    out.push(data as char);
                    break;
                }
            }
            Character::MultiByte { len } => {
                let _ = extend_from_bytes_with_len(bytes, ind, out, len);
                break;
            }
        }
    }
}

/// Sift preallocated until newline (preserves deduped newlines and trims spaces before them).
///
/// Link labels: the rewriter writes `[` plus `LINK_LABEL_OPEN` before an
/// anchor and `LINK_LABEL_CLOSE` plus `](url)` after it. Between the two, every
/// whitespace run, newlines included, becomes one space, and the label is
/// trimmed at both ends, the way a browser renders inline anchor text. Both
/// markers are dropped. This happens inside the same single pass.
fn sift_preallocated_until_newline(bytes: &[u8], ind: &mut usize, out: &mut String) {
    sift_trim_start_line(bytes, ind, out);

    let mut is_last_whitespace = false;
    let mut is_last_carriage_return = false;
    // Open link labels; 0 outside any label.
    let mut label_depth: u32 = 0;
    // `out.len()` where the outermost open label started.
    let mut label_start: usize = 0;
    // True right after a label opens, until its first non-whitespace byte.
    let mut label_trim = false;

    while *ind < bytes.len() {
        let b = bytes[*ind];

        // Fast path: batch process runs of regular ASCII characters
        if is_regular_ascii(b) {
            let start = *ind;
            *ind += 1;

            // Scan ahead for more regular ASCII
            while *ind < bytes.len() && is_regular_ascii(bytes[*ind]) {
                *ind += 1;
            }

            // Safety: we verified all bytes are ASCII (< 0x80)
            let slice = unsafe { std::str::from_utf8_unchecked(&bytes[start..*ind]) };
            out.push_str(slice);
            is_last_whitespace = false;
            is_last_carriage_return = false;
            label_trim = false;
            continue;
        }

        match get_char_metadata(b) {
            Character::SingleByte { data } => {
                *ind += 1;

                if is_ascii_whitespace(data) {
                    if label_depth > 0 {
                        if out.len().saturating_sub(label_start) <= LINK_LABEL_MAX {
                            is_last_carriage_return = false;
                            if label_trim || is_last_whitespace {
                                continue;
                            }
                            is_last_whitespace = true;
                            out.push(' ');
                            continue;
                        }
                        // Runaway label: stop folding, keep newlines from here.
                        label_depth = 0;
                        label_trim = false;
                    }

                    if is_newline(data) {
                        // Drop trailing ASCII space before newline (fixes " \n").
                        if is_last_whitespace {
                            out.pop();
                            is_last_whitespace = false;
                        }

                        if is_last_carriage_return {
                            out.push('\r');
                        }
                        out.push('\n');
                        return;
                    }

                    is_last_carriage_return = data == CARRIAGE_RETURN;

                    if is_last_whitespace {
                        continue;
                    }
                    is_last_whitespace = true;
                    out.push(' ');
                } else if data == LINK_LABEL_OPEN {
                    if label_depth == 0 {
                        label_start = out.len();
                    }
                    label_depth = label_depth.saturating_add(1);
                    label_trim = true;
                    // The `[` before the marker is never a space, so this is
                    // already false. Keep the close-side pop label-local.
                    is_last_whitespace = false;
                    is_last_carriage_return = false;
                } else if data == LINK_LABEL_CLOSE {
                    if label_depth > 0 {
                        // Trim the label's trailing space.
                        if is_last_whitespace {
                            out.pop();
                            is_last_whitespace = false;
                        }
                        label_depth -= 1;
                    }
                    label_trim = false;
                    is_last_carriage_return = false;
                } else {
                    is_last_whitespace = false;
                    is_last_carriage_return = false;
                    label_trim = false;
                    out.push(data as char);
                }
            }
            Character::MultiByte { len } => {
                let was_last_whitespace = is_last_whitespace;
                let was_ws = extend_from_bytes_with_len(bytes, ind, out, len);
                if was_ws && label_depth > 0 && (label_trim || was_last_whitespace) {
                    // Unicode space inside a label folds like ASCII space.
                    out.pop();
                    is_last_whitespace = was_last_whitespace;
                } else {
                    is_last_whitespace = was_ws;
                    label_trim = false;
                }
                is_last_carriage_return = false;
            }
        }
    }

    sift_trim_end(out, is_last_whitespace);
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

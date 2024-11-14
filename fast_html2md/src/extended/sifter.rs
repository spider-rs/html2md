use auto_encoder::auto_encode_bytes;
use std::str;

/// Charector handling bytes.
enum Character {
    SingleByte { data: u8 },
    MultiByte { len: usize },
}

/// A trait containing all `string` whitespace-sifting functions.
pub trait WhitespaceSifter: AsRef<str> {
    /// This removes duplicate [whitespaces](https://doc.rust-lang.org/reference/whitespace.html) from a `string` implementing `AsRef<str>`.
    /// This follows the [is_ascii_whitespace](https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_whitespace) implementation.
    /// This treats carriage-returns as just one `char` in the `string`.
    #[must_use]
    fn sift(&self) -> String {
        let input: &str = self.as_ref();
        let mut out: String = String::with_capacity(input.len());
        sift_preallocated(input.as_bytes(), &mut out);
        out
    }

    /// This removes duplicate [whitespaces](https://doc.rust-lang.org/reference/whitespace.html) from a `string` implementing `AsRef<str>`.
    /// This follows the [is_ascii_whitespace](https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_whitespace) implementation.
    /// This preserves deduplicated newlines.
    /// This treats carriage-returns as just one `char` in the `string`.
    #[must_use]
    fn sift_preserve_newlines(&self) -> String {
        let input = self.as_ref();
        let mut out = String::with_capacity(input.len());
        let bytes = input.as_bytes();
        let mut ind: usize = 0;

        while ind < bytes.len() {
            sift_preallocated_until_newline(bytes, &mut ind, &mut out);
        }

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

/// A custom implementation of `str::trim_start`.
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
                extend_from_bytes_with_len(bytes, ind, out, len);
                break;
            }
        }
    }
}

/// A custom implementation for `str::trim_end`.
fn sift_trim_end(out: &mut String, is_last_whitespace: bool) {
    if is_last_whitespace {
        out.pop();
    }
}

/// Extend the bytes from a slice.
fn extend_from_bytes_with_len(bytes: &[u8], ind: &mut usize, out: &mut String, len: usize) {
    let end = ind.saturating_add(len);
    // Check bounds to ensure we don't run into an out-of-bounds error.
    if *ind <= end && end <= bytes.len() {
        // Todo: we want to pass in the bytes encoded to string.
        out.push_str(&auto_encode_bytes(&bytes[*ind..end]));
    }
    *ind = end;
}

#[inline]
const fn is_newline(codepoint: u8) -> bool {
    matches!(codepoint, LINE_FEED | CARRIAGE_RETURN)
}

/// Sift preallocate safe strings.
fn sift_preallocated(bytes: &[u8], out: &mut String) {
    if !bytes.is_empty() {
        let mut ind: usize = 0;
        sift_trim_start(bytes, &mut ind, out);
        let mut is_last_whitespace: bool = false;
        let mut is_last_carriage_return: bool = false;

        while ind < bytes.len() {
            match get_char_metadata(bytes[ind]) {
                Character::SingleByte { data } => {
                    ind += 1;
                    if is_ascii_whitespace(data) {
                        if data == LINE_FEED && is_last_carriage_return {
                            out.push('\n');
                            is_last_carriage_return = false;
                            continue;
                        }
                        if is_last_whitespace {
                            continue;
                        }
                        is_last_whitespace = true;
                    } else {
                        is_last_whitespace = false;
                    }
                    out.push(data as char);
                    is_last_carriage_return = data == CARRIAGE_RETURN;
                }
                Character::MultiByte { len } => {
                    extend_from_bytes_with_len(bytes, &mut ind, out, len);
                }
            }
            is_last_carriage_return = false;
        }
        sift_trim_end(out, is_last_whitespace);
    }
}

/// Sift preallocate until complete.
fn sift_preallocated_until_newline(bytes: &[u8], ind: &mut usize, out: &mut String) {
    sift_trim_start(bytes, ind, out);

    let mut is_last_whitespace = false;
    let mut is_last_carriage_return = false;

    while *ind < bytes.len() {
        match get_char_metadata(bytes[*ind]) {
            Character::SingleByte { data } => {
                *ind += 1;
                if is_ascii_whitespace(data) {
                    if is_newline(data) {
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
                } else {
                    is_last_whitespace = false;
                }
                out.push(data as char);
            }
            Character::MultiByte { len } => {
                extend_from_bytes_with_len(bytes, ind, out, len);
            }
        }
        is_last_carriage_return = false;
    }
    sift_trim_end(out, is_last_whitespace);
}

/// Binary extracted from [std](https://doc.rust-lang.org/src/core/str/validations.rs.html#36).
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

/// Values extracted from [std](https://doc.rust-lang.org/src/core/char/methods.rs.html#1680).
#[inline]
const fn is_ascii_whitespace(codepoint: u8) -> bool {
    matches!(
        codepoint,
        SPACE | HORIZONTAL_TAB | LINE_FEED | FORM_FEED | CARRIAGE_RETURN
    )
}

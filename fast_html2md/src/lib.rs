use extended::sifter::{WhitespaceSifter, WhitespaceSifterBytes};

// we want to just use the rewriter instead for v0.1.
pub mod extended;

#[cfg(feature = "scraper")]
pub use markup5ever_rcdom::{Handle, NodeData, RcDom};

#[cfg(feature = "rewriter")]
pub mod rewriter;
#[cfg(feature = "scraper")]
pub mod scraper;
#[cfg(feature = "scraper")]
pub use scraper::{
    ignore, parse_html, parse_html_custom, parse_html_custom_base, parse_html_custom_with_url,
    parse_html_extended,
};

// Regex patterns only needed for the scraper feature
#[cfg(feature = "scraper")]
lazy_static::lazy_static! {
    pub(crate) static ref MARKDOWN_MIDDLE_KEYCHARS: regex::Regex =
        regex::Regex::new(r"[<>*\\_~]").expect("valid regex pattern");
}

/// Main function of this library to come. Rewrites incoming HTML, converts it into Markdown
/// and returns converted string. Incomplete work in progress for major performance increases.
/// # Arguments
/// `html` is source HTML as `String`
#[cfg(feature = "rewriter")]
pub fn rewrite_html(html: &str, commonmark: bool) -> String {
    rewriter::writer::convert_html_to_markdown(html, &None, commonmark, &None).unwrap_or_default()
}

/// Main function of this library async streaming. Rewrites incoming HTML, converts it into Markdown
/// and returns converted string. Incomplete work in progress for major performance increases.
/// # Arguments
/// `html` is source HTML as `String`
#[cfg(all(feature = "stream", feature = "rewriter"))]
pub async fn rewrite_html_streaming(html: &str, commonmark: bool) -> String {
    rewriter::writer::convert_html_to_markdown_send(html, &None, commonmark, &None)
        .await
        .unwrap_or_default()
}

/// Custom variant of rewrite function.
///
/// You can also override standard tag handlers this way
/// # Arguments
/// `html` is source HTML as `String`
/// `custom` is custom tag hadler producers for tags you want, can be empty
/// `commonmark` is for adjusting markdown output to commonmark
/// `url` is used to provide absolute url handling
#[cfg(all(feature = "stream", feature = "rewriter"))]
pub fn rewrite_html_custom_with_url(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<url::Url>,
) -> String {
    rewriter::writer::convert_html_to_markdown(html, &custom, commonmark, url).unwrap_or_default()
}

/// Custom variant of rewrite function.
///
/// You can also override standard tag handlers this way
/// # Arguments
/// `html` is source HTML as `String`
/// `custom` is custom tag hadler producers for tags you want, can be empty
/// `commonmark` is for adjusting markdown output to commonmark
/// `url` is used to provide absolute url handling
/// `chunk_size` the chunk size to use.
#[cfg(all(feature = "stream", feature = "rewriter"))]
pub async fn rewrite_html_custom_with_url_and_chunk(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<url::Url>,
    chunk_size: usize,
) -> String {
    rewriter::writer::convert_html_to_markdown_send_with_size(
        html, &custom, commonmark, url, chunk_size,
    )
    .await
    .unwrap_or_default()
}

/// Custom variant of rewrite function streaming async.
///
/// You can also override standard tag handlers this way
/// # Arguments
/// `html` is source HTML as `String`
/// `custom` is custom tag hadler producers for tags you want, can be empty
/// `commonmark` is for adjusting markdown output to commonmark
/// `url` is used to provide absolute url handling
#[cfg(all(feature = "stream", feature = "rewriter"))]
pub async fn rewrite_html_custom_with_url_streaming(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<url::Url>,
) -> String {
    rewriter::writer::convert_html_to_markdown_send(html, &custom, commonmark, url)
        .await
        .unwrap_or_default()
}

/// Re-export the stream error type.
#[cfg(all(feature = "stream", feature = "rewriter"))]
pub use rewriter::writer::StreamConvertError;

/// Convert an async stream of HTML byte chunks into markdown.
///
/// Genuinely async — yields to the executor while waiting for each input chunk.
/// Accepts any `Stream<Item = Result<B, E>>` where `B: AsRef<[u8]>`, so it works
/// with `Vec<u8>`, `bytes::Bytes`, `&[u8]`, etc.
///
/// # Arguments
/// * `stream` - an async stream of byte chunks (e.g. from an HTTP response body)
/// * `commonmark` - adjust output to CommonMark spec
#[cfg(all(feature = "stream", feature = "rewriter"))]
pub async fn rewrite_html_stream<S, B, E>(
    stream: S,
    commonmark: bool,
) -> Result<String, StreamConvertError<E>>
where
    S: futures_util::Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
{
    rewriter::writer::convert_html_stream_to_markdown(stream, &None, commonmark, &None).await
}

/// Convert an async stream of HTML byte chunks into markdown with custom options.
///
/// # Arguments
/// * `stream` - an async stream of byte chunks
/// * `custom` - custom tag handler producers for tags to ignore
/// * `commonmark` - adjust output to CommonMark spec
/// * `url` - base URL for resolving relative links
#[cfg(all(feature = "stream", feature = "rewriter"))]
pub async fn rewrite_html_stream_custom_with_url<S, B, E>(
    stream: S,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<url::Url>,
) -> Result<String, StreamConvertError<E>>
where
    S: futures_util::Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
{
    rewriter::writer::convert_html_stream_to_markdown(stream, custom, commonmark, url).await
}

/// Called after all processing has been finished
///
/// Clears excessive punctuation that would be trimmed by renderer anyway
pub fn clean_markdown(input: &str) -> String {
    input.sift_preserve_newlines()
}

/// Called after all processing has been finished
///
/// Clears excessive punctuation that would be trimmed by renderer anyway
pub fn clean_markdown_bytes(input: &Vec<u8>) -> String {
    input.sift_bytes_preserve_newlines()
}

/// Check if a byte needs markdown escaping.
#[inline]
const fn needs_escape(b: u8) -> bool {
    matches!(b, b'<' | b'>' | b'*' | b'\\' | b'_' | b'~')
}

/// Marks the start of a link label in the raw rewriter output. The rewriter
/// emits `[` followed by this byte, and the final whitespace sift consumes it.
/// It never reaches the returned markdown.
pub(crate) const LINK_LABEL_OPEN: u8 = 0x02;

/// Marks the end of a link label in the raw rewriter output. The rewriter
/// emits this byte followed by `](url)`, and the final whitespace sift
/// consumes it. It never reaches the returned markdown.
pub(crate) const LINK_LABEL_CLOSE: u8 = 0x03;

/// `LINK_LABEL_OPEN` as a str, for the rewriter's `before` insert.
pub(crate) const LINK_LABEL_OPEN_STR: &str = "[\u{2}";

/// True when the byte is one of the two link label markers.
#[inline]
pub(crate) const fn is_link_label_marker(b: u8) -> bool {
    b == LINK_LABEL_OPEN || b == LINK_LABEL_CLOSE
}

/// Marks the start of a `<pre>` block in the raw rewriter output. The final
/// whitespace sift writes the opening fence for it and copies every byte up
/// to `CODE_BLOCK_CLOSE` as it is: indentation, blank lines and all. It never
/// reaches the returned markdown.
pub(crate) const CODE_BLOCK_OPEN: u8 = 0x0E;

/// Marks the end of a `<pre>` block in the raw rewriter output. The sift
/// writes the closing fence for it. It never reaches the returned markdown.
pub(crate) const CODE_BLOCK_CLOSE: u8 = 0x0F;

/// `CODE_BLOCK_OPEN` as a str, for the rewriter's `before` insert.
pub(crate) const CODE_BLOCK_OPEN_STR: &str = "\u{e}";

/// `CODE_BLOCK_CLOSE` as a str, for the rewriter's `after` insert.
pub(crate) const CODE_BLOCK_CLOSE_STR: &str = "\u{f}";

/// True when the byte is any marker the sift consumes: the two link label
/// markers and the two code block markers.
#[inline]
pub(crate) const fn is_sift_marker(b: u8) -> bool {
    is_link_label_marker(b) || b == CODE_BLOCK_OPEN || b == CODE_BLOCK_CLOSE
}

/// Drop literal marker bytes from page supplied text, so only the rewriter's
/// own markers reach the sift. Borrows when there are none, which is every
/// real page.
#[inline]
pub(crate) fn strip_sift_markers(input: &str) -> std::borrow::Cow<'_, str> {
    if input.bytes().any(is_sift_marker) {
        std::borrow::Cow::Owned(
            input
                .chars()
                .filter(|c| !matches!(c, '\u{2}' | '\u{3}' | '\u{e}' | '\u{f}'))
                .collect(),
        )
    } else {
        std::borrow::Cow::Borrowed(input)
    }
}

/// Check if byte could start a special sequence (escape char or &nbsp;).
#[inline]
const fn is_special_byte(b: u8) -> bool {
    needs_escape(b) || b == b'&'
}

/// Scan classes, one table load per byte instead of a chain of compares.
/// `SCAN_TEXT`: the text escape scan stops here. `SCAN_CODE`: the code text
/// scan stops here.
const SCAN_TEXT: u8 = 1;
const SCAN_CODE: u8 = 2;

const SCAN_TABLE: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut b = 0usize;
    while b < 256 {
        let byte = b as u8;
        let mut class = 0;
        if is_special_byte(byte) || is_sift_marker(byte) {
            class |= SCAN_TEXT;
        }
        if byte == b'&' || is_sift_marker(byte) {
            class |= SCAN_CODE;
        }
        table[b] = class;
        b += 1;
    }
    table
};

/// Bytes the text escape scan stops on: the special bytes plus the sift
/// markers, which are dropped from page text.
#[inline(always)]
const fn is_text_scan_byte(b: u8) -> bool {
    SCAN_TABLE[b as usize] & SCAN_TEXT != 0
}

/// Bytes the code text scan stops on: `&` and the sift markers.
#[inline(always)]
const fn is_code_scan_byte(b: u8) -> bool {
    SCAN_TABLE[b as usize] & SCAN_CODE != 0
}

/// Check if a string contains any characters that need markdown escaping.
/// Use this to avoid allocation when no escaping is needed.
#[inline]
pub fn contains_markdown_chars(input: &str) -> bool {
    input.as_bytes().iter().any(|&b| is_special_byte(b))
}

/// Decode a single HTML entity at the start of a byte slice.
/// Returns the decoded string and the number of bytes consumed, or None if not a recognized entity.
/// Handles named entities (&amp; &lt; &gt; &quot; &nbsp; &apos;) and numeric (&#N; &#xH;).
#[inline]
fn decode_html_entity(bytes: &[u8]) -> Option<(&'static str, usize)> {
    debug_assert_eq!(bytes[0], b'&');

    // Find the semicolon (cap search at 10 bytes for perf — longest named entity we care about is &nbsp; = 6)
    let limit = bytes.len().min(12);
    let semi = bytes[1..limit].iter().position(|&b| b == b';')?;
    let entity = &bytes[1..semi + 1]; // between & and ;
    let consumed = semi + 2; // includes & and ;

    match entity {
        b"amp" => Some(("&", consumed)),
        b"lt" => Some(("\\<", consumed)),
        b"gt" => Some(("\\>", consumed)),
        b"quot" => Some(("\"", consumed)),
        b"apos" => Some(("'", consumed)),
        // A non-breaking space is still a space: dropping it joins two words.
        b"nbsp" => Some((" ", consumed)),
        _ if entity.first() == Some(&b'#') => decode_numeric_entity(entity, consumed),
        _ => None,
    }
}

/// Decode numeric HTML entities: &#39; &#x27; etc.
#[inline]
fn decode_numeric_entity(entity: &[u8], consumed: usize) -> Option<(&'static str, usize)> {
    let (digits, radix) = if entity.get(1) == Some(&b'x') || entity.get(1) == Some(&b'X') {
        (&entity[2..], 16)
    } else {
        (&entity[1..], 10)
    };

    if digits.is_empty() {
        return None;
    }

    // Parse into a u32 without allocation
    let mut val: u32 = 0;
    for &b in digits {
        let d = match b {
            b'0'..=b'9' => (b - b'0') as u32,
            b'a'..=b'f' if radix == 16 => (b - b'a' + 10) as u32,
            b'A'..=b'F' if radix == 16 => (b - b'A' + 10) as u32,
            _ => return None,
        };
        val = val.checked_mul(radix)?.checked_add(d)?;
    }

    // Map common code points to static strings to avoid allocation
    match val {
        0x26 => Some(("&", consumed)),          // &
        0x3C => Some(("\\<", consumed)),        // <
        0x3E => Some(("\\>", consumed)),        // >
        0x22 => Some(("\"", consumed)),         // "
        0x27 => Some(("'", consumed)),          // '
        0xA0 => Some((" ", consumed)),          // nbsp
        0x2014 => Some(("\u{2014}", consumed)), // em dash
        0x2013 => Some(("\u{2013}", consumed)), // en dash
        0x2018 => Some(("\u{2018}", consumed)), // left single quote
        0x2019 => Some(("\u{2019}", consumed)), // right single quote
        0x201C => Some(("\u{201c}", consumed)), // left double quote
        0x201D => Some(("\u{201d}", consumed)), // right double quote
        _ => None,                              // unrecognized numeric entity — pass through as-is
    }
}

/// Replace the markdown chars cleanly.
/// Optimized to scan bytes and process in bulk segments.
/// Returns None if no changes needed (avoids allocation).
#[inline]
pub fn replace_markdown_chars_opt(input: &str) -> Option<String> {
    let bytes = input.as_bytes();

    // Fast path: scan for any special character
    let first_special = bytes.iter().position(|&b| is_text_scan_byte(b));

    match first_special {
        None => None,
        Some(first_pos) => {
            // Pre-allocate with some headroom for escapes
            let mut output = String::with_capacity(input.len() + input.len() / 8);

            // Copy everything before first special char
            output.push_str(&input[..first_pos]);

            // Process the rest byte-by-byte from first_pos
            let mut i = first_pos;
            while i < bytes.len() {
                let b = bytes[i];

                if needs_escape(b) {
                    output.push('\\');
                    output.push(b as char);
                    i += 1;
                } else if is_sift_marker(b) {
                    // Literal marker bytes in page text would open or close
                    // a label or a code block in the sift. Drop them.
                    i += 1;
                } else if b == b'&' {
                    // Decode HTML entities in a single pass
                    if let Some((decoded, len)) = decode_html_entity(&bytes[i..]) {
                        output.push_str(decoded);
                        i += len;
                    } else {
                        output.push('&');
                        i += 1;
                    }
                } else {
                    // Find the next special character and copy the segment
                    let segment_start = i;
                    i += 1;
                    while i < bytes.len() && !is_text_scan_byte(bytes[i]) {
                        i += 1;
                    }
                    output.push_str(&input[segment_start..i]);
                }
            }

            Some(output)
        }
    }
}

/// Longest entity, `&` and `;` included, that [`code_text_opt`] decodes. A
/// decimal reference to the last code point, `&#1114111;`, is ten bytes.
const CODE_ENTITY_MAX: usize = 10;

/// Decode one entity at the start of `bytes` for code text. Returns the
/// character and the bytes consumed. Unlike `decode_html_entity` nothing is
/// escaped, a non-breaking space becomes a space, and every numeric
/// reference decodes.
#[inline]
fn decode_code_entity(bytes: &[u8]) -> Option<(char, usize)> {
    let limit = bytes.len().min(CODE_ENTITY_MAX);
    let semi = bytes.get(1..limit)?.iter().position(|&b| b == b';')?;
    let entity = bytes.get(1..semi + 1)?;
    let consumed = semi + 2;

    let decoded = match entity {
        b"amp" => '&',
        b"lt" => '<',
        b"gt" => '>',
        b"quot" => '"',
        b"apos" => '\'',
        b"nbsp" => ' ',
        [b'#', digits @ ..] => {
            let (digits, radix) = match digits {
                [b'x' | b'X', hex @ ..] => (hex, 16),
                _ => (digits, 10),
            };
            if digits.is_empty() {
                return None;
            }
            let mut value: u32 = 0;
            for &b in digits {
                let digit = (b as char).to_digit(radix)?;
                value = value.checked_mul(radix)?.checked_add(digit)?;
            }
            match char::from_u32(value)? {
                '\u{a0}' => ' ',
                // A reference to NUL or to a marker byte never becomes one.
                '\0' | '\u{2}' | '\u{3}' | '\u{e}' | '\u{f}' => return None,
                c => c,
            }
        }
        _ => return None,
    };

    Some((decoded, consumed))
}

/// Text inside `<pre>`, `<code>` and `<samp>`: the five named entities,
/// `&nbsp;` and numeric references are decoded, sift markers are dropped and
/// nothing is escaped, because a backslash is literal inside a code span or
/// a fence. Returns `None` when the text is already what should be written.
#[inline]
pub(crate) fn code_text_opt(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let first = bytes.iter().position(|&b| is_code_scan_byte(b))?;

    let mut output = String::with_capacity(input.len());
    output.push_str(input.get(..first)?);

    let mut i = first;
    while let Some(&b) = bytes.get(i) {
        if is_sift_marker(b) {
            i += 1;
        } else if b == b'&' {
            match bytes.get(i..).and_then(decode_code_entity) {
                Some((decoded, len)) => {
                    output.push(decoded);
                    i += len;
                }
                None => {
                    output.push('&');
                    i += 1;
                }
            }
        } else {
            let start = i;
            i += 1;
            while bytes.get(i).is_some_and(|&b| !is_code_scan_byte(b)) {
                i += 1;
            }
            // Both ends sit next to ASCII bytes or the ends of the input, so
            // they are char boundaries.
            output.push_str(input.get(start..i)?);
        }
    }

    Some(output)
}

/// The length of an entity that the end of `input` cuts short: a `&` in the
/// last few bytes with no `;` after it and only name or number characters in
/// between. The rewriter holds those bytes back until the next piece of the
/// same text node, so a reference split across two writes still decodes.
#[inline]
pub(crate) fn cut_entity_len(input: &str) -> usize {
    let bytes = input.as_bytes();
    let from = bytes.len().saturating_sub(CODE_ENTITY_MAX - 1);
    for (at, b) in bytes.iter().enumerate().skip(from).rev() {
        match b {
            b'&' => return bytes.len() - at,
            b'#' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => {}
            _ => return 0,
        }
    }
    0
}

/// Replace the markdown chars cleanly.
/// Optimized to scan bytes and process in bulk segments.
#[inline]
pub fn replace_markdown_chars(input: &str) -> String {
    replace_markdown_chars_opt(input).unwrap_or_else(|| input.to_string())
}

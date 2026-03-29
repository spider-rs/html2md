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

/// Check if byte could start a special sequence (escape char or &nbsp;).
#[inline]
const fn is_special_byte(b: u8) -> bool {
    needs_escape(b) || b == b'&'
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
        b"nbsp" => Some(("", consumed)), // strip non-breaking spaces like before
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
        0x3C => Some(("\\<", consumed)),         // <
        0x3E => Some(("\\>", consumed)),         // >
        0x22 => Some(("\"", consumed)),          // "
        0x27 => Some(("'", consumed)),           // '
        0xA0 => Some(("", consumed)),            // nbsp
        0x2014 => Some(("\u{2014}", consumed)),  // em dash
        0x2013 => Some(("\u{2013}", consumed)),  // en dash
        0x2018 => Some(("\u{2018}", consumed)),  // left single quote
        0x2019 => Some(("\u{2019}", consumed)),  // right single quote
        0x201C => Some(("\u{201c}", consumed)),  // left double quote
        0x201D => Some(("\u{201d}", consumed)),  // right double quote
        _ => None, // unrecognized numeric entity — pass through as-is
    }
}

/// Replace the markdown chars cleanly.
/// Optimized to scan bytes and process in bulk segments.
/// Returns None if no changes needed (avoids allocation).
#[inline]
pub fn replace_markdown_chars_opt(input: &str) -> Option<String> {
    let bytes = input.as_bytes();

    // Fast path: scan for any special character
    let first_special = bytes.iter().position(|&b| is_special_byte(b));

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
                    while i < bytes.len() && !is_special_byte(bytes[i]) {
                        i += 1;
                    }
                    output.push_str(&input[segment_start..i]);
                }
            }

            Some(output)
        }
    }
}

/// Replace the markdown chars cleanly.
/// Optimized to scan bytes and process in bulk segments.
#[inline]
pub fn replace_markdown_chars(input: &str) -> String {
    replace_markdown_chars_opt(input).unwrap_or_else(|| input.to_string())
}

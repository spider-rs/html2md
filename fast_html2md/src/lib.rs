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
                    // Check for &nbsp; (6 bytes)
                    if i + 5 < bytes.len() && &bytes[i..i + 6] == b"&nbsp;" {
                        // Skip &nbsp; entirely
                        i += 6;
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

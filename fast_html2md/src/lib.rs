use extended::sifter::WhitespaceSifterBytes;
use lazy_static::lazy_static;
pub use markup5ever_rcdom::{Handle, NodeData, RcDom};
use regex::Regex;
use std::collections::HashSet;
use url::Url;
// we want to just use the rewriter instead for v0.1.
pub mod extended;
pub mod rewriter;
pub mod scraper;
use extended::sifter::WhitespaceSifter;

pub use scraper::ignore;
pub use scraper::{
    parse_html, parse_html_custom, parse_html_custom_base, parse_html_custom_with_url,
    parse_html_extended,
};

lazy_static! {
    static ref MARKDOWN_MIDDLE_KEYCHARS: Regex = Regex::new(r"[<>*\\_~]").expect("valid regex pattern");               // for Markdown escaping
    static ref MARKDOWN_MIDDLE_KEYCHARS_SET: regex::RegexSet = regex::RegexSet::new(&[
        r"[<>*\\_~]",  // Matches any single markdown character
        r"&nbsp;"      // Matches the entire "&nbsp;" string
    ]).expect("valid regex set");
}

/// Main function of this library to come. Rewrites incoming HTML, converts it into Markdown
/// and returns converted string. Incomplete work in progress for major performance increases.
/// # Arguments
/// `html` is source HTML as `String`
pub fn rewrite_html(html: &str, commonmark: bool) -> String {
    rewriter::writer::convert_html_to_markdown(html, &None, commonmark, &None).unwrap_or_default()
}

/// Main function of this library async streaming. Rewrites incoming HTML, converts it into Markdown
/// and returns converted string. Incomplete work in progress for major performance increases.
/// # Arguments
/// `html` is source HTML as `String`
#[cfg(feature = "tokio")]
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
#[cfg(feature = "tokio")]
pub fn rewrite_html_custom_with_url(
    html: &str,
    custom: &Option<HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
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
#[cfg(feature = "tokio")]
pub async fn rewrite_html_custom_with_url_and_chunk(
    html: &str,
    custom: &Option<HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
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
#[cfg(feature = "tokio")]
pub async fn rewrite_html_custom_with_url_streaming(
    html: &str,
    custom: &Option<HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
) -> String {
    rewriter::writer::convert_html_to_markdown_send(html, &custom, commonmark, url)
        .await
        .unwrap_or_default()
}

/// Called after all processing has been finished
///
/// Clears excessive punctuation that would be trimmed by renderer anyway
pub fn clean_markdown(input: &str) -> String {
    input.sift()
}

/// Called after all processing has been finished
///
/// Clears excessive punctuation that would be trimmed by renderer anyway
pub fn clean_markdown_bytes(input: &Vec<u8>) -> String {
    input.sift_bytes()
}

/// Replace the markdown chars cleanly.
pub fn replace_markdown_chars(input: &str) -> String {
    use crate::MARKDOWN_MIDDLE_KEYCHARS_SET;

    if !MARKDOWN_MIDDLE_KEYCHARS_SET.is_match(input) {
        return input.to_string();
    }

    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            entity.push(ch);
            while let Some(&next_ch) = chars.peek() {
                entity.push(next_ch);
                chars.next();
                if entity == "&nbsp;" {
                    entity.clear(); // discard &nbsp;
                    break;
                } else if next_ch == ';' || entity.len() > 6 {
                    output.push_str(&entity);
                    break;
                }
            }
            if !entity.is_empty() {
                output.push_str(&entity);
            }
        } else if "<>*\\_~".contains(ch) {
            output.push('\\');
            output.push(ch);
        } else {
            output.push(ch);
        }
    }

    output
}

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

/// Custom variant of rewrite function.
///
/// You can also override standard tag handlers this way
/// # Arguments
/// `html` is source HTML as `String`
/// `custom` is custom tag hadler producers for tags you want, can be empty
/// `commonmark` is for adjusting markdown output to commonmark
/// `url` is used to provide absolute url handling
pub fn rewrite_html_custom_with_url(
    html: &str,
    custom: &Option<HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
) -> String {
    rewriter::writer::convert_html_to_markdown(html, &custom, commonmark, url).unwrap_or_default()
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

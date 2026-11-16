use crate::{strip_sift_markers, LINK_LABEL_CLOSE, LINK_LABEL_OPEN_STR};
use lol_html::html_content::{ContentType::Html, Element};
use percent_encoding::percent_decode_str;
use std::borrow::Cow;
use url::Url;

/// Build the markdown link suffix: the label close marker, then `](url)`.
///
/// The marker tells the final whitespace sift where the label ends, so the
/// sift can fold the label's inline whitespace onto one line in the same
/// pass. The sift drops the marker.
#[inline]
fn build_link_suffix(url: &str, needs_angle_brackets: bool) -> String {
    if needs_angle_brackets {
        // A URL with control bytes could carry a literal marker. Drop it so
        // the sift sees only ours.
        let url = strip_sift_markers(url);
        let mut s = String::with_capacity(url.len() + 5); // \x03](<url>)
        s.push(LINK_LABEL_CLOSE as char);
        s.push_str("](<");
        s.push_str(&url);
        s.push_str(">)");
        s
    } else {
        let mut s = String::with_capacity(url.len() + 4); // \x03](url)
        s.push(LINK_LABEL_CLOSE as char);
        s.push_str("](");
        s.push_str(url);
        s.push(')');
        s
    }
}

/// Resolve the href against the page url and decide on angle brackets.
#[inline]
fn resolve_href<'a>(href: &'a str, url: &Option<Url>) -> (Cow<'a, str>, bool) {
    let decoded_url: Cow<'_, str> = percent_decode_str(href).decode_utf8_lossy();

    let resolved_url = if decoded_url.starts_with('/') {
        match &url {
            Some(url) => {
                if let Ok(u) = url.join(&decoded_url) {
                    Cow::Owned(u.to_string())
                } else {
                    decoded_url
                }
            }
            None => decoded_url,
        }
    } else {
        decoded_url
    };

    let needs_brackets = resolved_url
        .bytes()
        .any(|b| b.is_ascii_control() || b == b' ');

    (resolved_url, needs_brackets)
}

/// True for a heading permalink: `class="headerlink"` (mkdocs, Sphinx) or
/// `class="hash-link"` (Docusaurus). Its only text is a symbol such as `§`,
/// `¶` or a zero width space, so the markdown drops the whole anchor. The
/// class decides it, never the text, so a heading whose text is a link keeps
/// it.
#[inline]
fn is_permalink_class(class: Option<String>) -> bool {
    class.is_some_and(|c| {
        c.split_ascii_whitespace()
            .any(|t| t == "headerlink" || t == "hash-link")
    })
}

/// True when the anchor could be a heading permalink, so its class has to be
/// read. A permalink links to its own heading: its `href` holds a fragment
/// or is absent, and it has an attribute besides `href`. Every other anchor
/// is ruled out here without the class lookup, which allocates once for the
/// name and once per attribute.
#[inline]
fn may_be_permalink(href: Option<&str>, attributes: usize) -> bool {
    href.map_or(true, |h| h.contains('#')) && attributes > usize::from(href.is_some())
}

/// Rewrite the anchor.
pub(crate) fn rewrite_anchor_element(
    el: &mut Element,
    _commonmark: bool,
    url: &Option<Url>,
) -> Result<(), std::io::Error> {
    let href = el.get_attribute("href");
    if may_be_permalink(href.as_deref(), el.attributes().len())
        && is_permalink_class(el.get_attribute("class"))
    {
        el.remove();
        return Ok(());
    }
    if let Some(href) = href {
        let (resolved_url, needs_brackets) = resolve_href(&href, url);
        el.before(LINK_LABEL_OPEN_STR, Html);
        el.after(&build_link_suffix(&resolved_url, needs_brackets), Html);
    }
    Ok(())
}

/// Rewrite the anchor.
pub(crate) fn rewrite_anchor_element_send(
    el: &mut lol_html::send::Element,
    _commonmark: bool,
    url: &Option<Url>,
) -> Result<(), std::io::Error> {
    let href = el.get_attribute("href");
    if may_be_permalink(href.as_deref(), el.attributes().len())
        && is_permalink_class(el.get_attribute("class"))
    {
        el.remove();
        return Ok(());
    }
    if let Some(href) = href {
        let (resolved_url, needs_brackets) = resolve_href(&href, url);
        el.before(LINK_LABEL_OPEN_STR, Html);
        el.after(&build_link_suffix(&resolved_url, needs_brackets), Html);
    }
    Ok(())
}

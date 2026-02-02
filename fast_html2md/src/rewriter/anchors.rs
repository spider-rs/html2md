use lol_html::html_content::{ContentType::Html, Element};
use percent_encoding::percent_decode_str;
use std::borrow::Cow;
use url::Url;

/// Build markdown link suffix efficiently.
#[inline]
fn build_link_suffix(url: &str, needs_angle_brackets: bool) -> String {
    if needs_angle_brackets {
        let mut s = String::with_capacity(url.len() + 4); // ](<url>)
        s.push_str("](<");
        s.push_str(url);
        s.push_str(">)");
        s
    } else {
        let mut s = String::with_capacity(url.len() + 3); // ](url)
        s.push_str("](");
        s.push_str(url);
        s.push(')');
        s
    }
}

/// Rewrite the anchor.
pub(crate) fn rewrite_anchor_element(
    el: &mut Element,
    _commonmark: bool,
    url: &Option<Url>,
) -> Result<(), std::io::Error> {
    if let Some(href) = el.get_attribute("href") {
        let decoded_url: Cow<'_, str> = percent_decode_str(&href).decode_utf8_lossy();

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

        el.before("[", Html);
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
    if let Some(href) = el.get_attribute("href") {
        let decoded_url: Cow<'_, str> = percent_decode_str(&href).decode_utf8_lossy();

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

        el.before("[", Html);
        el.after(&build_link_suffix(&resolved_url, needs_brackets), Html);
    }
    Ok(())
}

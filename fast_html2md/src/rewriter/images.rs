use lol_html::html_content::Element;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::borrow::Cow;
use url::Url;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/// Build markdown image syntax efficiently.
#[inline]
fn build_image_markdown(alt: &str, url: &str, title: &str) -> String {
    if title.is_empty() {
        // ![alt](url)
        let mut s = String::with_capacity(alt.len() + url.len() + 5);
        s.push_str("![");
        s.push_str(alt);
        s.push_str("](");
        s.push_str(url);
        s.push(')');
        s
    } else {
        // ![alt](url "title")
        let mut s = String::with_capacity(alt.len() + url.len() + title.len() + 8);
        s.push_str("![");
        s.push_str(alt);
        s.push_str("](");
        s.push_str(url);
        s.push_str(" \"");
        s.push_str(title);
        s.push_str("\")");
        s
    }
}

/// Push attribute to HTML string efficiently.
#[inline]
fn push_attr(s: &mut String, name: &str, value: &str) {
    s.push(' ');
    s.push_str(name);
    s.push_str("=\"");
    s.push_str(value);
    s.push('"');
}

/// Rewrite the image.
pub(crate) fn rewrite_image_element(
    el: &mut Element,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<(), std::io::Error> {
    let src = el.get_attribute("src").unwrap_or_default();
    let alt = el.get_attribute("alt").unwrap_or_default();
    let title = el.get_attribute("title").unwrap_or_default();

    let height = el.get_attribute("height");
    let width = el.get_attribute("width");
    let align = el.get_attribute("align");

    if commonmark && (height.is_some() || width.is_some() || align.is_some()) {
        let mut img_tag = String::with_capacity(src.len() + 64);
        img_tag.push_str("<img src=\"");
        img_tag.push_str(&src);
        img_tag.push('"');

        if !alt.is_empty() {
            push_attr(&mut img_tag, "alt", &alt);
        }
        if !title.is_empty() {
            push_attr(&mut img_tag, "title", &title);
        }
        if let Some(ref h) = height {
            push_attr(&mut img_tag, "height", h);
        }
        if let Some(ref w) = width {
            push_attr(&mut img_tag, "width", w);
        }
        if let Some(ref a) = align {
            push_attr(&mut img_tag, "align", a);
        }

        img_tag.push_str(" />");
        el.set_inner_content(&img_tag, lol_html::html_content::ContentType::Html);
    } else {
        let img_url: Cow<str> = if src.contains(' ') {
            Cow::Owned(utf8_percent_encode(&src, FRAGMENT).to_string())
        } else if src.starts_with('/') {
            if let Some(ref u) = url {
                if let Ok(n) = u.join(&src) {
                    Cow::Owned(n.to_string())
                } else {
                    Cow::Borrowed(&src)
                }
            } else {
                Cow::Borrowed(&src)
            }
        } else {
            Cow::Borrowed(&src)
        };

        el.replace(
            &build_image_markdown(&alt, &img_url, &title),
            lol_html::html_content::ContentType::Html,
        );
    }

    Ok(())
}

/// Rewrite the image.
pub(crate) fn rewrite_image_element_send(
    el: &mut lol_html::send::Element,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<(), std::io::Error> {
    let src = el.get_attribute("src").unwrap_or_default();
    let alt = el.get_attribute("alt").unwrap_or_default();
    let title = el.get_attribute("title").unwrap_or_default();

    let height = el.get_attribute("height");
    let width = el.get_attribute("width");
    let align = el.get_attribute("align");

    if commonmark && (height.is_some() || width.is_some() || align.is_some()) {
        let mut img_tag = String::with_capacity(src.len() + 64);
        img_tag.push_str("<img src=\"");
        img_tag.push_str(&src);
        img_tag.push('"');

        if !alt.is_empty() {
            push_attr(&mut img_tag, "alt", &alt);
        }
        if !title.is_empty() {
            push_attr(&mut img_tag, "title", &title);
        }
        if let Some(ref h) = height {
            push_attr(&mut img_tag, "height", h);
        }
        if let Some(ref w) = width {
            push_attr(&mut img_tag, "width", w);
        }
        if let Some(ref a) = align {
            push_attr(&mut img_tag, "align", a);
        }

        img_tag.push_str(" />");
        el.set_inner_content(&img_tag, lol_html::html_content::ContentType::Html);
    } else {
        let img_url: Cow<str> = if src.contains(' ') {
            Cow::Owned(utf8_percent_encode(&src, FRAGMENT).to_string())
        } else if src.starts_with('/') {
            if let Some(ref u) = url {
                if let Ok(n) = u.join(&src) {
                    Cow::Owned(n.to_string())
                } else {
                    Cow::Borrowed(&src)
                }
            } else {
                Cow::Borrowed(&src)
            }
        } else {
            Cow::Borrowed(&src)
        };

        el.replace(
            &build_image_markdown(&alt, &img_url, &title),
            lol_html::html_content::ContentType::Html,
        );
    }

    Ok(())
}

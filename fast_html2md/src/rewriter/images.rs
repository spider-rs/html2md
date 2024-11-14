use lol_html::html_content::Element;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use url::Url;

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/// Rewrite the image.
pub(crate) fn rewrite_image_element(
    el: &mut Element,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<(), std::io::Error> {
    let src = el.get_attribute("src").unwrap_or_default();
    let alt = el.get_attribute("alt").unwrap_or_default();
    let title = el.get_attribute("title").unwrap_or_else(|| "".to_string());

    let height = el.get_attribute("height");
    let width = el.get_attribute("width");
    let align = el.get_attribute("align");

    if commonmark && (height.is_some() || width.is_some() || align.is_some()) {
        let mut img_tag = format!("<img src=\"{}\"", src);

        if let Some(alt) = el.get_attribute("alt") {
            img_tag.push_str(&format!(" alt=\"{}\"", alt));
        }
        if let Some(title) = el.get_attribute("title") {
            img_tag.push_str(&format!(" title=\"{}\"", title));
        }
        if let Some(height) = height {
            img_tag.push_str(&format!(" height=\"{}\"", height));
        }
        if let Some(width) = width {
            img_tag.push_str(&format!(" width=\"{}\"", width));
        }
        if let Some(align) = align {
            img_tag.push_str(&format!(" align=\"{}\"", align));
        }

        img_tag.push_str(" />");
        el.set_inner_content(&img_tag, lol_html::html_content::ContentType::Html);
    } else {
        let mut img_url = if src.contains(' ') {
            utf8_percent_encode(&src, FRAGMENT).to_string()
        } else {
            src.clone()
        };

        if img_url.starts_with('/') {
            if let Some(ref u) = url {
                if let Ok(n) = u.join(&img_url) {
                    img_url = n.to_string();
                }
            }
        }

        el.replace(
            &format!(
                "![{}]({}{})",
                alt,
                img_url,
                if !title.is_empty() {
                    format!(" \"{}\"", title)
                } else {
                    "".to_string()
                }
            ),
            lol_html::html_content::ContentType::Html,
        );
    }

    Ok(())
}

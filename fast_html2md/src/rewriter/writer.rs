use super::iframes::handle_iframe;
use super::images::rewrite_image_element;
use crate::clean_markdown;
use lol_html::doc_comments;
use lol_html::html_content::ContentType::Text;
use lol_html::html_content::Element;
use lol_html::{element, rewrite_str, RewriteStrSettings};
use std::sync::Arc;
use url::Url;

/// Insert a new line
#[inline]
pub fn insert_newline(element: &mut Element) {
    element.after("\n", Text);
}

/// Handle the lol_html tag.
#[inline]
fn handle_tag(
    element: &mut Element,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    element.remove_and_keep_content();

    // Add the markdown equivalents before the element.
    match element.tag_name().as_str() {
        "h1" => {
            element.before("# ", Text);
            insert_newline(element);
        }
        "h2" => {
            element.before("## ", Text);
            insert_newline(element);
        }
        "h3" => {
            element.before("### ", Text);
            insert_newline(element);
        }
        "h4" => {
            element.before("#### ", Text);
            insert_newline(element);
        }
        "h5" => {
            element.before("##### ", Text);
            insert_newline(element);
        }
        "h6" => {
            element.before("###### ", Text);
            insert_newline(element);
        }
        "p" => element.before("\n", Text),
        "hr" => {
            insert_newline(element);
            element.append("---", Text);
            insert_newline(element);
        }
        "br" => insert_newline(element),
        "li" => element.before("* ", Text),
        "a" => {
            if let Some(href) = element.get_attribute("href") {
                element.before("[", lol_html::html_content::ContentType::Text);
                element.after(
                    &format!("]({})", href),
                    lol_html::html_content::ContentType::Text,
                );
                element.set_inner_content("", lol_html::html_content::ContentType::Text);
                // Remove content tags.
            }
        }
        "img" => {
            let _ = rewrite_image_element(element, commonmark, &url);
        }
        "tr" => {
            element.before("| ", Text);
            element.after(" |\n", Text);
        }
        "th" => {
            element.before("**", Text);
            element.after("** | ", Text);
        }
        "td" => {
            element.after(" | ", Text);
        }
        "iframe" => {
            let _ = handle_iframe(element);
        }
        _ => (),
    }

    Ok(())
}

/// Get the HTML rewriter settings to convert ot markdown.
pub fn get_rewriter_settings(
    commonmark: bool,
    url: Option<Url>,
) -> RewriteStrSettings<'static, 'static> {
    RewriteStrSettings {
        document_content_handlers: vec![doc_comments!(|c| {
            c.remove();
            Ok(())
        })],
        element_content_handlers: vec![
            element!("head, nav, svg", |el| {
                el.remove();
                Ok(())
            }),
            element!("*:not(script):not(head):not(style):not(svg)", move |el| {
                let _ = handle_tag(el, commonmark, &url);
                Ok(())
            }),
        ],
        ..RewriteStrSettings::default()
    }
}

/// Convert to markdown streaming re-writer
pub(crate) fn convert_html_to_markdown(
    html: &str,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<String, Box<dyn std::error::Error>> {
    let settings = get_rewriter_settings(commonmark, url.clone());

    match rewrite_str(&Box::new(html), settings) {
        Ok(markdown) => Ok(clean_markdown(&markdown)),
        Err(e) => Err(e.into()),
    }
}

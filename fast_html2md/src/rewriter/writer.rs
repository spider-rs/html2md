use super::iframes::handle_iframe;
use super::images::rewrite_image_element;
use super::lists::handle_list_or_item;
use super::quotes::{rewrite_blockquote_element, rewrite_blockquote_text};
use super::styles::rewrite_style_element;
use crate::clean_markdown;
use lol_html::html_content::ContentType::Text;
use lol_html::html_content::Element;
use lol_html::{doc_comments, text};
use lol_html::{element, rewrite_str, RewriteStrSettings};
use std::cell::RefCell;
use std::rc::Rc;
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
    list_type: Rc<RefCell<Option<String>>>,
    order_counter: Rc<RefCell<usize>>,
    quote_depth: Rc<RefCell<usize>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let element_name = element.tag_name();

    let remove_attrs =
        commonmark && (element_name.as_str() == "sub" || element_name.as_str() == "sup");

    // check common mark includes.
    if remove_attrs {
        let attrs = element
            .attributes()
            .iter()
            .map(|f| f.name())
            .collect::<Vec<String>>();

        for attr in attrs.iter() {
            element.remove_attribute(&attr);
        }
    } else {
        element.remove_and_keep_content();
    }

    // Add the markdown equivalents before the element.
    match element_name.as_str() {
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
        "a" => {
            if let Some(href) = element.get_attribute("href") {
                element.before("[", lol_html::html_content::ContentType::Text);
                element.after(
                    &format!("]({})", href),
                    lol_html::html_content::ContentType::Text,
                );
                element.set_inner_content("", lol_html::html_content::ContentType::Text);
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
        "b" | "i" | "s" | "strong" | "em" | "del" => {
            let _ = rewrite_style_element(element);
        }
        "ol" | "ul" | "menu" | "li" => {
            let _ = handle_list_or_item(element, list_type.clone(), order_counter.clone());
        }
        "q" | "cite" | "blockquote" => {
            let _ = rewrite_blockquote_element(element, quote_depth);
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
    let list_type = Rc::new(RefCell::new(None));
    let order_counter = Rc::new(RefCell::new(0));
    let quote_depth = Rc::new(RefCell::new(0));

    let quote_depth1 = quote_depth.clone();

    RewriteStrSettings {
        document_content_handlers: vec![doc_comments!(|c| {
            c.remove();
            Ok(())
        })],
        element_content_handlers: vec![
            text!("blockquote, q, cite", move |el| {
                let _ = rewrite_blockquote_text(el, quote_depth1.clone());
                Ok(())
            }),
            text!("summary, details", move |el| {
                *el.as_mut_str() = el.as_str().trim().into();
                Ok(())
            }),
            element!("head, nav", |el| {
                el.remove();
                Ok(())
            }),
            element!("*:not(script):not(head):not(style):not(svg)", move |el| {
                let _ = handle_tag(
                    el,
                    commonmark,
                    &url,
                    list_type.clone(),
                    order_counter.clone(),
                    quote_depth.clone(),
                );
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

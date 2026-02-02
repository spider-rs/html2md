use super::anchors::{rewrite_anchor_element, rewrite_anchor_element_send};
use super::iframes::{handle_iframe, handle_iframe_send};
use super::images::{rewrite_image_element, rewrite_image_element_send};
use super::lists::{handle_list_or_item, handle_list_or_item_send};
use super::quotes::{rewrite_blockquote_element, rewrite_blockquote_element_send};
use super::styles::{rewrite_style_element, rewrite_style_element_send};
use super::{
    insert_newline_after, insert_newline_after_send, insert_newline_before,
    insert_newline_before_send,
};
use lol_html::html_content::{
    ContentType::{Html, Text},
    Element,
};
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicUsize},
    Arc,
};
use url::Url;

/// Handle the lol_html tag (sync).
///
/// NOTE:
/// - `in_table` tracks whether we're inside a <table>.
/// - `table_row_start` tracks start-of-row so we can emit a leading '|' once per row.
/// - `list_item_start` is set by list.rs when "* " / "N. " is emitted; used to avoid "*\nText".
#[inline]
pub fn handle_tag(
    element: &mut Element,
    commonmark: bool,
    url: &Option<Url>,
    list_type: &mut Option<&'static str>,
    order_counter: &mut usize,
    quote_depth: Rc<AtomicUsize>,
    in_table: &mut bool,
    table_row_start: &mut bool,
    list_item_start: &mut bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let element_name = element.tag_name();
    let element_name = element_name.as_str();

    let remove_attrs = commonmark && (element_name == "sub" || element_name == "sup");

    // check common mark includes.
    if remove_attrs {
        let attrs = element
            .attributes()
            .iter()
            .map(|f| f.name())
            .collect::<Vec<String>>();

        for attr in attrs.iter() {
            element.remove_attribute(attr);
        }
    } else {
        element.remove_and_keep_content();
    }

    // Add the markdown equivalents before/after the element.
    match element_name {
        "h1" => {
            element.before("# ", Text);
            insert_newline_after(element);
        }
        "h2" => {
            element.before("## ", Text);
            insert_newline_after(element);
        }
        "h3" => {
            element.before("### ", Text);
            insert_newline_after(element);
        }
        "h4" => {
            element.before("#### ", Text);
            insert_newline_after(element);
        }
        "h5" => {
            element.before("##### ", Text);
            insert_newline_after(element);
        }
        "h6" => {
            element.before("###### ", Text);
            insert_newline_after(element);
        }

        // KEY FIX FOR LISTS:
        // If list.rs just emitted "* " or "N. ", do NOT insert a newline before the first <p>.
        "p" => {
            if !*list_item_start {
                insert_newline_before(element);
            }
            insert_newline_after(element);
        }

        "hr" => {
            insert_newline_before(element);
            element.append("---", Text);
            insert_newline_after(element);
        }
        "br" => insert_newline_after(element),

        "a" => {
            let _ = rewrite_anchor_element(element, commonmark, url);
        }
        "img" => {
            let _ = rewrite_image_element(element, commonmark, url);
        }

        // TABLES (minimal pipe rendering support)
        "table" => {
            *in_table = true;
            *table_row_start = false;
        }
        "tr" => {
            if *in_table {
                *table_row_start = true;
            }
            insert_newline_after(element);
        }
        "th" | "td" => {
            if *in_table && *table_row_start {
                element.before("|", Html);
                *table_row_start = false;
            }

            if element_name == "th" && commonmark {
                element.before("** ", Html);
                element.after("**|", Html);
            } else {
                element.after("|", Html);
            }
        }

        "iframe" => {
            let _ = handle_iframe(element);
        }
        "b" | "i" | "s" | "strong" | "em" | "del" => {
            let _ = rewrite_style_element(element);
        }

        // LISTS: list.rs sets list_item_start=true when it emits a marker.
        "ol" | "ul" | "menu" | "li" => {
            let _ = handle_list_or_item(element, list_type, order_counter, list_item_start);
        }

        "q" | "cite" | "blockquote" => {
            let _ = rewrite_blockquote_element(element, quote_depth);
        }

        "div" | "section" | "header" | "footer" => {
            insert_newline_before(element);
            insert_newline_after(element);
        }
        "pre" => {
            element.before("\n```\n", Html);
            element.after("\n```\n", Html);
        }
        "code" | "samp" => {
            element.before("`", Html);
            element.after("`", Html);
        }
        _ => (),
    }

    Ok(())
}

/// Handle the lol_html tag (send).
#[inline]
pub fn handle_tag_send(
    element: &mut lol_html::send::Element,
    commonmark: bool,
    url: &Option<Url>,
    list_type: &mut Option<&'static str>,
    order_counter: &mut usize,
    quote_depth: Arc<AtomicUsize>,
    in_table: &mut bool,
    table_row_start: &mut bool,
    list_item_start: &mut bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let element_name = element.tag_name();
    let element_name = element_name.as_str();

    let remove_attrs = commonmark && (element_name == "sub" || element_name == "sup");

    // check common mark includes.
    if remove_attrs {
        let attrs = element
            .attributes()
            .iter()
            .map(|f| f.name())
            .collect::<Vec<String>>();

        for attr in attrs.iter() {
            element.remove_attribute(attr);
        }
    } else {
        element.remove_and_keep_content();
    }

    match element_name {
        "h1" => {
            element.before("# ", Html);
            insert_newline_after_send(element);
        }
        "h2" => {
            element.before("## ", Text);
            insert_newline_after_send(element);
        }
        "h3" => {
            element.before("### ", Text);
            insert_newline_after_send(element);
        }
        "h4" => {
            element.before("#### ", Text);
            insert_newline_after_send(element);
        }
        "h5" => {
            element.before("##### ", Text);
            insert_newline_after_send(element);
        }
        "h6" => {
            element.before("###### ", Text);
            insert_newline_after_send(element);
        }

        // KEY FIX FOR LISTS
        "p" => {
            if !*list_item_start {
                insert_newline_before_send(element);
            }
            insert_newline_after_send(element);
        }

        "hr" => {
            insert_newline_before_send(element);
            element.append("---", Text);
            insert_newline_after_send(element);
        }
        "br" => insert_newline_after_send(element),

        "a" => {
            let _ = rewrite_anchor_element_send(element, commonmark, url);
        }
        "img" => {
            let _ = rewrite_image_element_send(element, commonmark, url);
        }

        // TABLES
        "table" => {
            *in_table = true;
            *table_row_start = false;
        }
        "tr" => {
            if *in_table {
                *table_row_start = true;
            }
            insert_newline_after_send(element);
        }
        "th" | "td" => {
            if *in_table && *table_row_start {
                element.before("|", Html);
                *table_row_start = false;
            }

            if element_name == "th" && commonmark {
                element.before("** ", Html);
                element.after("**|", Html);
            } else {
                element.after("|", Html);
            }
        }

        "iframe" => {
            let _ = handle_iframe_send(element);
        }
        "b" | "i" | "s" | "strong" | "em" | "del" => {
            let _ = rewrite_style_element_send(element);
        }

        // LISTS
        "ol" | "ul" | "menu" | "li" => {
            let _ = handle_list_or_item_send(element, list_type, order_counter, list_item_start);
        }

        "q" | "cite" | "blockquote" => {
            let _ = rewrite_blockquote_element_send(element, quote_depth.clone());
        }

        "div" | "section" | "header" | "footer" => {
            insert_newline_before_send(element);
            insert_newline_after_send(element);
        }
        "pre" => {
            element.before("\n```\n", Html);
            element.after("\n```\n", Html);
        }
        "code" | "samp" => {
            element.before("`", Html);
            element.after("`", Html);
        }
        _ => (),
    }

    Ok(())
}

use super::handle::handle_tag;
use super::quotes::rewrite_blockquote_text;
use crate::clean_markdown_bytes;
use lol_html::html_content::ContentType::Text;
use lol_html::html_content::Element;
use lol_html::{doc_comments, doctype, text};
use lol_html::{element, RewriteStrSettings};
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

/// Insert a new line after
#[inline]
pub fn insert_newline_after(element: &mut Element) {
    element.after("\n", Text);
}

/// Insert a new line before
#[inline]
pub fn insert_newline_before(element: &mut Element) {
    element.before("\n", Text);
}

/// Replace the markdown chars cleanly.
fn replace_markdown_chars(input: &str) -> String {
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

/// Get the HTML rewriter settings to convert to markdown.
pub fn get_rewriter_settings(
    commonmark: bool,
    custom: &Option<std::collections::HashSet<String>>,
    url: Option<Url>,
) -> RewriteStrSettings<'static, 'static> {
    let list_type = Rc::new(RefCell::new(None));
    let order_counter = Rc::new(RefCell::new(0));
    let quote_depth = Rc::new(RefCell::new(0));
    let quote_depth1 = quote_depth.clone();
    let inside_table = Rc::new(RefCell::new(false));

    let mut element_content_handlers = Vec::with_capacity(
        4 + custom
            .as_ref()
            .map_or(0, |c| if c.is_empty() { 0 } else { 1 }),
    );

    element_content_handlers.push(text!("blockquote, q, cite", move |el| {
        let _ = rewrite_blockquote_text(el, quote_depth1.clone());
        Ok(())
    }));

    element_content_handlers.push(text!(
        "*:not(script):not(head):not(style):not(svg)",
        move |el| {
            *el.as_mut_str() = replace_markdown_chars(el.as_str().trim().into());
            Ok(())
        }
    ));

    element_content_handlers.push(element!("head, nav, script, noscript, style", |el| {
        el.remove();
        Ok(())
    }));

    element_content_handlers.push(element!("*", move |el| {
        let _ = handle_tag(
            el,
            commonmark,
            &url,
            list_type.clone(),
            order_counter.clone(),
            quote_depth.clone(),
            inside_table.clone(),
        );
        Ok(())
    }));

    if let Some(ignore) = custom {
        let ignore_handler = element!(
            ignore.iter().cloned().collect::<Vec<String>>().join(","),
            |el| {
                el.remove();
                Ok(())
            }
        );

        element_content_handlers.push(ignore_handler);
    }

    RewriteStrSettings {
        document_content_handlers: vec![
            doc_comments!(|c| {
                c.remove();
                Ok(())
            }),
            doctype!(|c| {
                c.remove();
                Ok(())
            }),
        ],
        element_content_handlers,
        ..RewriteStrSettings::default()
    }
}

/// Convert to markdown streaming re-writer
pub(crate) fn convert_html_to_markdown(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<String, Box<dyn std::error::Error>> {
    let settings = get_rewriter_settings(commonmark, custom, url.clone());

    match rewrite_str(&Box::new(html), settings) {
        Ok(markdown) => Ok(clean_markdown_bytes(&markdown)),
        Err(e) => Err(e.into()),
    }
}

/// Shortcut to rewrite string and encode correctly
pub fn rewrite_str<'h, 's, H: lol_html::HandlerTypes>(
    html: &str,
    settings: impl Into<lol_html::Settings<'h, 's, H>>,
) -> Result<Vec<u8>, lol_html::errors::RewritingError> {
    let mut output = vec![];

    let mut rewriter = lol_html::HtmlRewriter::new(settings.into(), |c: &[u8]| {
        output.extend_from_slice(c);
    });

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    Ok(output)
}

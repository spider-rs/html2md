use super::handle::handle_tag;
use super::quotes::rewrite_blockquote_text;
use crate::clean_markdown_bytes;
use crate::rewriter::{handle::handle_tag_send, quotes::rewrite_blockquote_text_send};
use lol_html::{doc_comments, doctype, element, html_content::EndTag, text, RewriteStrSettings};
use std::rc::Rc;
use std::sync::{atomic::AtomicUsize, Arc};
use url::Url;

/// End tag handler type sync send.
type EndHandler = Box<
    dyn for<'b> FnOnce(
            &mut EndTag<'b>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>
        + Send
        + 'static,
>;

/// End tag local handler type sync send.
type LocalEndHandler = Box<
    dyn for<'b> FnOnce(
            &mut EndTag<'b>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>
        + 'static,
>;

/// Estimate the size of the markdown.
fn estimate_markdown(html: &str) -> usize {
    if html.is_empty() {
        0
    } else {
        (html.len() / 2).max(50)
    }
}

/// Get the HTML rewriter settings to convert to markdown.
pub fn get_rewriter_settings(
    commonmark: bool,
    custom: &Option<std::collections::HashSet<String>>,
    url: Option<Url>,
) -> RewriteStrSettings<'static, 'static> {
    let mut list_type = None;
    let mut order_counter = 0 as usize;
    let quote_depth = Rc::new(AtomicUsize::new(0));
    let quote_depth1 = quote_depth.clone();
    let repaired_head = Rc::new(std::sync::OnceLock::new());
    let mut inside_table = false;

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
            *el.as_mut_str() = crate::replace_markdown_chars(el.as_str().trim().into());
            Ok(())
        }
    ));

    element_content_handlers.push(element!(
        "head, nav, footer, script, noscript, style",
        move |el| {
            let repaired_head_element: bool = repaired_head.get().is_some();
            let head_element = el.tag_name() == "head";
            if head_element && !repaired_head_element {
                if let Some(hvec) = el.end_tag_handlers() {
                    let repaired_head = repaired_head.clone();
                    let h1: LocalEndHandler =
                        Box::new(move |end: &mut lol_html::html_content::EndTag<'_>| {
                            let repaired_element = repaired_head.get().is_some();
                            if end.name() == "html" && !repaired_element {
                                let _ = repaired_head.set(true);
                                end.after("</head>", lol_html::html_content::ContentType::Html);
                            } else {
                                end.remove();
                            }
                            Ok(())
                        });
                    hvec.push(h1);
                }
            } else {
                el.remove();
            }
            Ok(())
        }
    ));

    element_content_handlers.push(element!("*", move |el| {
        let _ = handle_tag(
            el,
            commonmark,
            &url,
            &mut list_type,
            &mut order_counter,
            quote_depth.clone(),
            &mut inside_table,
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

/// Get the HTML rewriter settings to convert to markdown sync send.
pub fn get_rewriter_settings_send(
    commonmark: bool,
    custom: &Option<std::collections::HashSet<String>>,
    url: Option<Url>,
) -> lol_html::send::Settings<'static, 'static> {
    let mut list_type = None;
    let mut order_counter = 0 as usize;
    let quote_depth = Arc::new(AtomicUsize::new(0));
    let quote_depth1 = quote_depth.clone();
    let repaired_head = Arc::new(std::sync::OnceLock::new());
    let mut inside_table = false;

    let mut element_content_handlers = Vec::with_capacity(
        4 + custom
            .as_ref()
            .map_or(0, |c| if c.is_empty() { 0 } else { 1 }),
    );

    element_content_handlers.push(text!("blockquote, q, cite", move |el| {
        let _ = rewrite_blockquote_text_send(el, quote_depth.clone());
        Ok(())
    }));

    element_content_handlers.push(text!(
        "*:not(script):not(head):not(style):not(svg)",
        move |el| {
            *el.as_mut_str() = crate::replace_markdown_chars(el.as_str().trim().into());
            Ok(())
        }
    ));

    element_content_handlers.push(element!(
        "head, nav, footer, script, noscript, style",
        move |el| {
            let repaired_head_element: bool = repaired_head.get().is_some();
            let head_element = el.tag_name() == "head";
            if head_element && !repaired_head_element {
                if let Some(hvec) = el.end_tag_handlers() {
                    let repaired_head = repaired_head.clone();
                    let h1: EndHandler =
                        Box::new(move |end: &mut lol_html::html_content::EndTag<'_>| {
                            let repaired_element = repaired_head.get().is_some();
                            if end.name() == "html" && !repaired_element {
                                let _ = repaired_head.set(true);
                                end.after("</head>", lol_html::html_content::ContentType::Html);
                            } else {
                                end.remove();
                            }
                            Ok(())
                        });
                    hvec.push(h1);
                }
            } else {
                el.remove();
            }
            Ok(())
        }
    ));

    element_content_handlers.push(element!("*", move |el| {
        let _ = handle_tag_send(
            el,
            commonmark,
            &url,
            &mut list_type,
            &mut order_counter,
            quote_depth1.clone(),
            &mut inside_table,
        );
        Ok(())
    }));

    if let Some(ignore) = custom {
        if !ignore.is_empty() {
            let ignore_handler = element!(
                ignore.iter().cloned().collect::<Vec<String>>().join(","),
                |el| {
                    el.remove();
                    Ok(())
                }
            );

            element_content_handlers.push(ignore_handler);
        }
    }

    lol_html::send::Settings {
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
        ..lol_html::send::Settings::new_send()
    }
}

/// Shortcut to rewrite string and encode correctly
pub(crate) fn rewrite_str<'h, 's, H: lol_html::HandlerTypes>(
    html: &str,
    settings: impl Into<lol_html::Settings<'h, 's, H>>,
) -> Result<Vec<u8>, lol_html::errors::RewritingError> {
    let mut output = Vec::with_capacity(estimate_markdown(html));

    let mut rewriter = lol_html::HtmlRewriter::new(settings.into(), |c: &[u8]| {
        output.extend_from_slice(c);
    });

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    Ok(output)
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

/// Convert to markdown streaming re-writer with chunk size.
#[cfg(feature = "stream")]
pub async fn convert_html_to_markdown_send_with_size(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
    chunk_size: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    use futures_util::stream::{self, StreamExt};
    let settings = get_rewriter_settings_send(commonmark, custom, url.clone());
    let mut rewrited_bytes: Vec<u8> = Vec::with_capacity(estimate_markdown(html));

    let mut rewriter = lol_html::send::HtmlRewriter::new(settings.into(), |c: &[u8]| {
        rewrited_bytes.extend_from_slice(&c);
    });

    let mut wrote_error = false;

    let mut stream = stream::iter(html.as_bytes().chunks(chunk_size));

    while let Some(chunk) = stream.next().await {
        if rewriter.write(chunk).is_err() {
            wrote_error = true;
            break;
        }
    }

    if !wrote_error {
        let _ = rewriter.end();
    }

    Ok(clean_markdown_bytes(&rewrited_bytes))
}

/// Convert to markdown streaming re-writer
#[cfg(feature = "stream")]
pub async fn convert_html_to_markdown_send(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<String, Box<dyn std::error::Error>> {
    convert_html_to_markdown_send_with_size(html, custom, commonmark, url, 8192).await
}

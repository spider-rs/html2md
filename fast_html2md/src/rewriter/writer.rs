use super::handle::handle_tag;
use super::quotes::rewrite_blockquote_text;
use crate::clean_markdown_bytes;
use crate::rewriter::{handle::handle_tag_send, quotes::rewrite_blockquote_text_send};
use lol_html::{doc_comments, doctype, element, html_content::EndTag, text, RewriteStrSettings};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicU8, AtomicUsize, Ordering},
    Arc,
};
use url::Url;

lazy_static::lazy_static! {
    #[cfg(feature = "ignore_cookies")]
    /// Cookie banner patterns.
    static ref COOKIE_BANNER_SELECTOR: &'static str =
        "body > #onetrust-banner-sdk,#didomi-host,#qc-cmp2-container,#cookie-banner,#__rptl-cookiebanner";
}

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

// ===== perf helpers =====

#[inline]
fn is_ascii_ws_only(s: &str) -> bool {
    // Equivalent to "trim().is_empty()" for the whitespace you actually see in HTML formatting.
    // Avoids Unicode trim work.
    s.as_bytes()
        .iter()
        .all(|&b| matches!(b, b' ' | b'\n' | b'\r' | b'\t' | 0x0C))
}

/// Estimate the size of the markdown.
fn estimate_markdown(html: &str) -> usize {
    if html.is_empty() {
        0
    } else {
        (html.len() / 2).max(50)
    }
}

// ===== send flags packed into one atomic =====
const F_IN_TABLE: u8 = 1 << 0;
const F_LI_START: u8 = 1 << 1;

#[inline]
fn flag_set(flags: &AtomicU8, mask: u8) {
    let _ = flags.fetch_or(mask, Ordering::Relaxed);
}

#[inline]
fn flag_clear(flags: &AtomicU8, mask: u8) {
    let _ = flags.fetch_and(!mask, Ordering::Relaxed);
}

/// Get the HTML rewriter settings to convert to markdown.
pub fn get_rewriter_settings(
    commonmark: bool,
    custom: &Option<std::collections::HashSet<String>>,
    url: Option<Url>,
) -> RewriteStrSettings<'static, 'static> {
    let mut list_type = None;
    let mut order_counter = 0usize;

    let quote_depth = Rc::new(AtomicUsize::new(0));
    let quote_depth1 = quote_depth.clone();

    let repaired_head = Rc::new(std::sync::OnceLock::new());

    // flags (non-send) are already fast
    let list_item_start_flag = Rc::new(Cell::new(false));
    let in_table_flag = Rc::new(Cell::new(false));

    // state passed into handle_tag
    let mut in_table = false;
    let mut table_row_start = false;
    let mut list_item_start = false;

    let mut element_content_handlers = Vec::with_capacity(
        4 + custom
            .as_ref()
            .map_or(0, |c| if c.is_empty() { 0 } else { 1 })
            + {
                #[cfg(feature = "ignore_cookies")]
                {
                    1
                }
                #[cfg(not(feature = "ignore_cookies"))]
                {
                    0
                }
            },
    );

    #[cfg(feature = "ignore_cookies")]
    {
        element_content_handlers.push(lol_html::element!(COOKIE_BANNER_SELECTOR, |el| {
            el.remove();
            Ok(())
        }));
    }

    element_content_handlers.push(text!("blockquote, q, cite", move |el| {
        let _ = rewrite_blockquote_text(el, quote_depth1.clone());
        Ok(())
    }));

    // TEXT HANDLER: drop whitespace-only nodes inside tables + at list item start
    let list_item_start_flag_text = list_item_start_flag.clone();
    let in_table_flag_text = in_table_flag.clone();
    element_content_handlers.push(text!(
        "*:not(script):not(head):not(style):not(svg)",
        move |el| {
            let s = el.as_str();

            // inside table: ignore formatting whitespace between cells
            if in_table_flag_text.get() && is_ascii_ws_only(s) {
                *el.as_mut_str() = String::new();
                return Ok(());
            }

            // list marker fix: swallow whitespace-only nodes until first real text
            if list_item_start_flag_text.get() {
                if is_ascii_ws_only(s) {
                    *el.as_mut_str() = String::new();
                    return Ok(());
                }
                list_item_start_flag_text.set(false);
            }

            // Only allocate if escaping is actually needed
            if let Some(escaped) = crate::replace_markdown_chars_opt(s) {
                *el.as_mut_str() = escaped;
            }
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

    // ELEMENT HANDLER: manage flags + call handle_tag
    let list_item_start_flag_el = list_item_start_flag.clone();
    let in_table_flag_el = in_table_flag.clone();

    element_content_handlers.push(element!("*", move |el| {
        // Table start: enable flag and add end-tag handler to disable.
        if el.tag_name().as_str() == "table" {
            in_table_flag_el.set(true);
            if let Some(hvec) = el.end_tag_handlers() {
                let in_table_flag_end = in_table_flag_el.clone();
                let h: LocalEndHandler =
                    Box::new(move |_end: &mut lol_html::html_content::EndTag<'_>| {
                        in_table_flag_end.set(false);
                        Ok(())
                    });
                hvec.push(h);
            }
        }

        // sync state from flags
        in_table = in_table_flag_el.get();
        list_item_start = list_item_start_flag_el.get();

        let _ = handle_tag(
            el,
            commonmark,
            &url,
            &mut list_type,
            &mut order_counter,
            quote_depth.clone(),
            &mut in_table,
            &mut table_row_start,
            &mut list_item_start,
        );

        // mirror list flag for text handler
        list_item_start_flag_el.set(list_item_start);

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
    let mut order_counter = 0usize;

    let quote_depth = Arc::new(AtomicUsize::new(0));
    let quote_depth1 = quote_depth.clone();

    let repaired_head = Arc::new(std::sync::OnceLock::new());

    // packed flags (single atomic load per handler call)
    let flags = Arc::new(AtomicU8::new(0));

    // state passed into handle_tag_send
    let mut in_table = false;
    let mut table_row_start = false;
    let mut list_item_start = false;

    let mut element_content_handlers = Vec::with_capacity(
        4 + custom
            .as_ref()
            .map_or(0, |c| if c.is_empty() { 0 } else { 1 })
            + {
                #[cfg(feature = "ignore_cookies")]
                {
                    1
                }
                #[cfg(not(feature = "ignore_cookies"))]
                {
                    0
                }
            },
    );

    #[cfg(feature = "ignore_cookies")]
    {
        element_content_handlers.push(lol_html::element!(COOKIE_BANNER_SELECTOR, |el| {
            el.remove();
            Ok(())
        }));
    }

    element_content_handlers.push(text!("blockquote, q, cite", move |el| {
        let _ = rewrite_blockquote_text_send(el, quote_depth.clone());
        Ok(())
    }));

    // TEXT HANDLER (send): single atomic load + ASCII whitespace scan
    let flags_text = flags.clone();
    element_content_handlers.push(text!(
        "*:not(script):not(head):not(style):not(svg)",
        move |el| {
            let f = flags_text.load(Ordering::Relaxed);
            let in_table_now = (f & F_IN_TABLE) != 0;
            let li_start_now = (f & F_LI_START) != 0;

            let s = el.as_str();

            if in_table_now && is_ascii_ws_only(s) {
                *el.as_mut_str() = String::new();
                return Ok(());
            }

            if li_start_now {
                if is_ascii_ws_only(s) {
                    *el.as_mut_str() = String::new();
                    return Ok(());
                }
                // clear li-start
                flag_clear(&*flags_text, F_LI_START);
            }

            // Only allocate if escaping is actually needed
            if let Some(escaped) = crate::replace_markdown_chars_opt(s) {
                *el.as_mut_str() = escaped;
            }
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

    // ELEMENT HANDLER (send): set/clear packed flags + call handle_tag_send
    let flags_el = flags.clone();
    element_content_handlers.push(element!("*", move |el| {
        // table start
        if el.tag_name().as_str() == "table" {
            flag_set(&*flags_el, F_IN_TABLE);

            if let Some(hvec) = el.end_tag_handlers() {
                let flags_end = flags_el.clone();
                let h: EndHandler =
                    Box::new(move |_end: &mut lol_html::html_content::EndTag<'_>| {
                        flag_clear(&*flags_end, F_IN_TABLE);
                        Ok(())
                    });
                hvec.push(h);
            }
        }

        // local bools for handle_tag_send
        let f = flags_el.load(Ordering::Relaxed);
        in_table = (f & F_IN_TABLE) != 0;
        list_item_start = (f & F_LI_START) != 0;

        let _ = handle_tag_send(
            el,
            commonmark,
            &url,
            &mut list_type,
            &mut order_counter,
            quote_depth1.clone(),
            &mut in_table,
            &mut table_row_start,
            &mut list_item_start,
        );

        // mirror li-start back into packed flags
        if list_item_start {
            flag_set(&*flags_el, F_LI_START);
        } else {
            flag_clear(&*flags_el, F_LI_START);
        }

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

    match rewrite_str(html, settings) {
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
    let settings = get_rewriter_settings_send(commonmark, custom, url.clone());
    let mut rewrited_bytes: Vec<u8> = Vec::with_capacity(estimate_markdown(html));

    let mut rewriter = lol_html::send::HtmlRewriter::new(settings.into(), |c: &[u8]| {
        rewrited_bytes.extend_from_slice(c);
    });

    let bytes = html.as_bytes();

    // Process in chunks without async overhead for in-memory data
    let mut wrote_error = false;
    for chunk in bytes.chunks(chunk_size) {
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

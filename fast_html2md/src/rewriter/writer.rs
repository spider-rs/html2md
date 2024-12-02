use super::handle::handle_tag;
use super::quotes::rewrite_blockquote_text;
use crate::clean_markdown_bytes;
use crate::rewriter::handle::handle_tag_send;
use crate::rewriter::quotes::rewrite_blockquote_text_send;
use lol_html::{doc_comments, doctype, text};
use lol_html::{element, RewriteStrSettings};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use url::Url;

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
            *el.as_mut_str() = crate::replace_markdown_chars(el.as_str().trim().into());
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

/// Get the HTML rewriter settings to convert to markdown sync send.
pub fn get_rewriter_settings_send(
    commonmark: bool,
    custom: &Option<std::collections::HashSet<String>>,
    url: Option<Url>,
) -> lol_html::send::Settings<'static, 'static> {
    let list_type = Arc::new(RwLock::new(None::<String>));
    let order_counter = Arc::new(RwLock::new(0));
    let quote_depth = Arc::new(RwLock::new(0));
    let inside_table = Arc::new(RwLock::new(false));

    let quote_depth1 = quote_depth.clone();

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

    element_content_handlers.push(element!("head, nav, script, noscript, style", |el| {
        el.remove();
        Ok(())
    }));

    element_content_handlers.push(element!("*", move |el| {
        let _ = handle_tag_send(
            el,
            commonmark,
            &url,
            list_type.clone(),
            order_counter.clone(),
            quote_depth1.clone(),
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
    let mut output = vec![];

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
#[cfg(feature = "tokio")]
pub async fn convert_html_to_markdown_send_with_size(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
    chunk_size: usize,
) -> Result<String, Box<dyn std::error::Error>> {
    use tokio_stream::StreamExt;
    let settings = get_rewriter_settings_send(commonmark, custom, url.clone());
    let (txx, mut rxx) = tokio::sync::mpsc::unbounded_channel();

    let mut rewriter = lol_html::send::HtmlRewriter::new(settings.into(), |c: &[u8]| {
        let _ = txx.send(c.to_vec());
    });

    let html_bytes = html.as_bytes();
    let chunks = html_bytes.chunks(chunk_size);

    let mut stream = tokio_stream::iter(chunks).map(Ok::<&[u8], ()>);

    let mut wrote_error = false;

    while let Some(chunk) = stream.next().await {
        if let Ok(chunk) = chunk {
            if rewriter.write(chunk).is_err() {
                wrote_error = true;
                break;
            }
        }
    }

    if !wrote_error {
        let _ = rewriter.end();
    }

    drop(txx);

    let mut rewrited_bytes: Vec<u8> = Vec::new();

    while let Some(c) = rxx.recv().await {
        rewrited_bytes.extend_from_slice(&c);
    }

    Ok(clean_markdown_bytes(&rewrited_bytes))
}

/// Convert to markdown streaming re-writer
#[cfg(feature = "tokio")]
pub async fn convert_html_to_markdown_send(
    html: &str,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<String, Box<dyn std::error::Error>> {
    convert_html_to_markdown_send_with_size(html, custom, commonmark, url, 8192).await
}

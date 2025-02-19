use lol_html::html_content::{ContentType, Element, TextChunk};
use std::error::Error;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

// Function to handle <blockquote> elements
pub(crate) fn rewrite_blockquote_element(
    el: &mut Element,
    quote_depth: Rc<AtomicUsize>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    quote_depth.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    if let Some(end_tag_handlers) = el.end_tag_handlers() {
        end_tag_handlers.push(Box::new({
            let quote_depth = quote_depth.clone();
            move |_end| {
                quote_depth.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

                Ok(())
            }
        }));
    }

    Ok(())
}

// Function to handle <blockquote> elements sync
pub(crate) fn rewrite_blockquote_element_send(
    el: &mut lol_html::send::Element,
    quote_depth: Arc<AtomicUsize>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    quote_depth.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    if let Some(end_tag_handlers) = el.end_tag_handlers() {
        end_tag_handlers.push(Box::new({
            let quote_depth = quote_depth.clone();
            move |_end| {
                quote_depth.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }
        }));
    }

    Ok(())
}

// Function to handle text within <blockquote> elements
pub(crate) fn rewrite_blockquote_text(
    text_chunk: &mut TextChunk<'_>,
    quote_depth: Rc<AtomicUsize>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let depth = quote_depth.load(std::sync::atomic::Ordering::Relaxed);
    let quote_prefix = "> ".repeat(depth);
    let lines: Vec<&str> = text_chunk.as_str().lines().collect();
    let total_lines = lines.len();

    let last = text_chunk.last_in_text_node();

    let modified_text = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i >= 1 && i == total_lines - 1 {
                format!("{}", line)
            } else {
                format!("{}{}", quote_prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("");

    text_chunk.replace(&modified_text, ContentType::Html);

    if last {
        text_chunk.after("\n", ContentType::Text);
    }

    Ok(())
}

// Function to handle text within <blockquote> elements sync
pub(crate) fn rewrite_blockquote_text_send(
    text_chunk: &mut TextChunk<'_>,
    quote_depth: Arc<AtomicUsize>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let depth = quote_depth.load(std::sync::atomic::Ordering::Relaxed);
    let quote_prefix = "> ".repeat(depth);
    let lines: Vec<&str> = text_chunk.as_str().lines().collect();
    let total_lines = lines.len();

    let last = text_chunk.last_in_text_node();

    let modified_text = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i >= 1 && i == total_lines - 1 {
                format!("{}", line)
            } else {
                format!("{}{}", quote_prefix, line)
            }
        })
        .collect::<Vec<_>>()
        .join("");

    text_chunk.replace(&modified_text, ContentType::Html);

    if last {
        text_chunk.after("\n", ContentType::Text);
    }

    Ok(())
}

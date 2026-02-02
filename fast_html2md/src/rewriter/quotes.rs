use lol_html::html_content::{ContentType, Element, TextChunk};
use std::error::Error;
use std::rc::Rc;
use std::sync::{atomic::AtomicUsize, Arc};

/// Pre-computed quote prefixes for common depths (avoids allocation).
const QUOTE_PREFIXES: [&str; 6] = ["", "> ", "> > ", "> > > ", "> > > > ", "> > > > > "];

/// Get quote prefix, using pre-computed for common depths.
#[inline]
fn get_quote_prefix(depth: usize) -> std::borrow::Cow<'static, str> {
    if depth < QUOTE_PREFIXES.len() {
        std::borrow::Cow::Borrowed(QUOTE_PREFIXES[depth])
    } else {
        std::borrow::Cow::Owned("> ".repeat(depth))
    }
}

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
            move |_end: &mut lol_html::html_content::EndTag<'_>| {
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

    // Fast path: no quoting needed at depth 0
    if depth == 0 {
        return Ok(());
    }

    let quote_prefix = get_quote_prefix(depth);
    let text = text_chunk.as_str();
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len();
    let last = text_chunk.last_in_text_node();

    // Pre-allocate output buffer
    let estimated_size = text.len() + quote_prefix.len() * total_lines;
    let mut modified_text = String::with_capacity(estimated_size);

    for (i, line) in lines.iter().enumerate() {
        if i >= 1 && i == total_lines - 1 {
            modified_text.push_str(line);
        } else {
            modified_text.push_str(&quote_prefix);
            modified_text.push_str(line);
        }
    }

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

    // Fast path: no quoting needed at depth 0
    if depth == 0 {
        return Ok(());
    }

    let quote_prefix = get_quote_prefix(depth);
    let text = text_chunk.as_str();
    let lines: Vec<&str> = text.lines().collect();
    let total_lines = lines.len();
    let last = text_chunk.last_in_text_node();

    // Pre-allocate output buffer
    let estimated_size = text.len() + quote_prefix.len() * total_lines;
    let mut modified_text = String::with_capacity(estimated_size);

    for (i, line) in lines.iter().enumerate() {
        if i >= 1 && i == total_lines - 1 {
            modified_text.push_str(line);
        } else {
            modified_text.push_str(&quote_prefix);
            modified_text.push_str(line);
        }
    }

    text_chunk.replace(&modified_text, ContentType::Html);

    if last {
        text_chunk.after("\n", ContentType::Text);
    }

    Ok(())
}

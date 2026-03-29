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

/// Apply quote prefixes to text lines without allocating a Vec.
#[inline]
fn apply_quote_prefix(text_chunk: &mut TextChunk<'_>, quote_depth: &AtomicUsize) {
    let depth = quote_depth.load(std::sync::atomic::Ordering::Relaxed);

    // Fast path: no quoting needed at depth 0
    if depth == 0 {
        return;
    }

    let quote_prefix = get_quote_prefix(depth);
    let text = text_chunk.as_str();
    let last = text_chunk.last_in_text_node();

    // Count newlines for capacity estimate (single byte scan)
    let newline_count = text.as_bytes().iter().filter(|&&b| b == b'\n').count();
    let estimated_size = text.len() + quote_prefix.len() * (newline_count + 1);
    let mut modified_text = String::with_capacity(estimated_size);

    let mut lines = text.lines().peekable();
    let mut i = 0usize;
    while let Some(line) = lines.next() {
        if i >= 1 && lines.peek().is_none() {
            modified_text.push_str(line);
        } else {
            modified_text.push_str(&quote_prefix);
            modified_text.push_str(line);
        }
        i += 1;
    }

    text_chunk.replace(&modified_text, ContentType::Html);

    if last {
        text_chunk.after("\n", ContentType::Text);
    }
}

// Function to handle text within <blockquote> elements
pub(crate) fn rewrite_blockquote_text(
    text_chunk: &mut TextChunk<'_>,
    quote_depth: &AtomicUsize,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    apply_quote_prefix(text_chunk, quote_depth);
    Ok(())
}

// Function to handle text within <blockquote> elements sync
pub(crate) fn rewrite_blockquote_text_send(
    text_chunk: &mut TextChunk<'_>,
    quote_depth: &AtomicUsize,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    apply_quote_prefix(text_chunk, quote_depth);
    Ok(())
}

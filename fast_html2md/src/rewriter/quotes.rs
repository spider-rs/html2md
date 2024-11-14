use crate::rewriter::counter::Counter;
use lol_html::html_content::{ContentType, Element, TextChunk};
use std::error::Error;
use std::{cell::RefCell, rc::Rc};

// Function to handle <blockquote> elements
pub(crate) fn rewrite_blockquote_element(
    el: &mut Element,
    quote_depth: Rc<RefCell<usize>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    quote_depth.borrow_mut().increment();

    if let Some(end_tag_handlers) = el.end_tag_handlers() {
        end_tag_handlers.push(Box::new({
            let quote_depth = quote_depth.clone();
            move |_end| {
                quote_depth.borrow_mut().decrement();
                Ok(())
            }
        }));
    }

    Ok(())
}

// Function to handle text within <blockquote> elements
pub(crate) fn rewrite_blockquote_text(
    text_chunk: &mut TextChunk<'_>,
    quote_depth: Rc<RefCell<usize>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let depth = *quote_depth.borrow();
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

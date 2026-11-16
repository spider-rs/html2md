use super::handle::handle_tag;
use super::quotes::rewrite_blockquote_text;
use crate::extended::sifter::SiftSink;
use crate::rewriter::{handle::handle_tag_send, quotes::rewrite_blockquote_text_send};
use lol_html::{
    doc_comments, doc_text, doctype, element, html_content::EndTag, text, RewriteStrSettings,
};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering},
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
/// Inside a `<pre>`: text is code and no tag writes markdown.
const F_IN_PRE: u8 = 1 << 2;
/// Inside a `<code>` or `<samp>` outside a `<pre>`: text is code.
const F_IN_CODE: u8 = 1 << 3;

// ===== text pieces =====

/// Rewrite one piece of a text node with `rewrite`: `code_text_opt` for text
/// in a `<pre>`, `<code>` or `<samp>`, `replace_markdown_chars_opt` for the
/// rest.
///
/// A text node reaches the handler in pieces cut wherever the input was cut,
/// so an entity can straddle two of them. `carry` holds the cut entity, at
/// most nine bytes, until the next piece of the same node. `last` is the
/// node's final piece, which flushes it, so nothing is held past the node.
/// Returns `None` when the piece is already what should be written.
fn text_piece(
    carry: &mut String,
    piece: &str,
    last: bool,
    rewrite: fn(&str) -> Option<String>,
) -> Option<String> {
    if carry.is_empty() {
        let cut = if last {
            0
        } else {
            crate::cut_entity_len(piece)
        };
        if cut == 0 {
            return rewrite(piece);
        }
        let keep = piece.len() - cut;
        let (head, tail) = (piece.get(..keep)?, piece.get(keep..)?);
        carry.push_str(tail);
        return Some(rewrite(head).unwrap_or_else(|| head.to_string()));
    }

    let mut joined = std::mem::take(carry);
    joined.push_str(piece);
    let cut = if last {
        0
    } else {
        crate::cut_entity_len(&joined)
    };
    if cut != 0 {
        let keep = joined.len() - cut;
        carry.push_str(joined.get(keep..).unwrap_or_default());
        joined.truncate(keep);
    }
    Some(rewrite(&joined).unwrap_or(joined))
}

// ===== table header delimiter row =====
//
// A pipe table needs a delimiter row under its header or a markdown renderer
// shows the rows as plain text. The writer keeps the first row's shape in one
// integer and writes `|---|---|` under it when every cell of that row is a
// `<th>` that covers one column and one row. A first row with a `<td>` is
// left alone: layout tables look like that, and nothing says their first row
// is a header.

/// Rows seen in the current table, saturating at 2.
const T_ROWS: u32 = 0b11;
/// The first row has a `<th>`.
const T_HAS_TH: u32 = 1 << 2;
/// The first row has a `<td>`.
const T_HAS_TD: u32 = 1 << 3;
/// The delimiter row is written.
const T_DONE: u32 = 1 << 4;
/// Cells in the first row, in the bits from here up.
const T_CELLS_SHIFT: u32 = 8;
/// Most columns a delimiter row is written for.
const T_CELLS_MAX: u32 = 256;

/// True when a `colspan` or `rowspan` value covers more than one cell. A
/// header row with such a cell has fewer pipe cells than the rows under it,
/// and a renderer drops the cells past the header's count, so that row gets
/// no delimiter row.
#[inline]
fn spans_cells(span: Option<String>) -> bool {
    span.map_or(false, |v| v.trim() != "1")
}

/// The state after a `<th>` or `<td>` start tag. `is_th` is false for a
/// `<td>` and for a `<th>` that spans cells.
#[inline]
fn table_cell(state: u32, is_th: bool) -> u32 {
    if state & T_ROWS != 1 {
        return state;
    }
    let cells = (state >> T_CELLS_SHIFT)
        .saturating_add(1)
        .min(T_CELLS_MAX + 1);
    let kind = if is_th { T_HAS_TH } else { T_HAS_TD };
    (state & ((1 << T_CELLS_SHIFT) - 1)) | kind | (cells << T_CELLS_SHIFT)
}

/// The delimiter row the first row asks for, if it is still owed.
#[inline]
fn table_delimiter(state: u32) -> Option<String> {
    let cells = state >> T_CELLS_SHIFT;
    let header = state & (T_ROWS | T_HAS_TH | T_HAS_TD | T_DONE) == (1 | T_HAS_TH);
    if !header || cells == 0 || cells > T_CELLS_MAX {
        return None;
    }
    let mut row = String::with_capacity(cells as usize * 4 + 2);
    row.push('|');
    for _ in 0..cells {
        row.push_str("---|");
    }
    row.push('\n');
    Some(row)
}

/// The state after a `<tr>` start tag, and the delimiter row to write before
/// it when this row follows a header row.
#[inline]
fn table_row(state: u32) -> (u32, Option<String>) {
    let delimiter = table_delimiter(state);
    let mut next = state;
    if delimiter.is_some() {
        next |= T_DONE;
    }
    let rows = ((state & T_ROWS) + 1).min(2);
    ((next & !T_ROWS) | rows, delimiter)
}

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
    let mut list_type: Option<&'static str> = None;
    let mut order_counter = 0usize;

    let quote_depth = Rc::new(AtomicUsize::new(0));
    let quote_depth1 = quote_depth.clone();

    let repaired_head = Rc::new(std::sync::OnceLock::new());

    // flags (non-send) are already fast
    let list_item_start_flag = Rc::new(Cell::new(false));
    let in_table_flag = Rc::new(Cell::new(false));
    // true inside `<pre>`, `<code>` and `<samp>`: text is code
    let in_code_flag = Rc::new(Cell::new(false));
    // true inside `<pre>`: no tag writes markdown
    let in_pre_flag = Rc::new(Cell::new(false));
    // true once the text handler has seen the current text chunk (see
    // `strip_unseen_markers`)
    let text_seen = Rc::new(Cell::new(false));
    // the first row of the current table (see `table_row`)
    let table_head = Rc::new(Cell::new(0u32));

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

    // Code inside a quote keeps its lines: the quote prefix would join them.
    let in_code_flag_quote = in_code_flag.clone();
    element_content_handlers.push(text!("blockquote, q, cite", move |el| {
        if !in_code_flag_quote.get() {
            let _ = rewrite_blockquote_text(el, &quote_depth1);
        }
        Ok(())
    }));

    // TEXT HANDLER: drop whitespace-only nodes inside tables + at list item start
    let list_item_start_flag_text = list_item_start_flag.clone();
    let in_table_flag_text = in_table_flag.clone();
    let in_code_flag_text = in_code_flag.clone();
    let text_seen_text = text_seen.clone();
    let mut text_carry = String::new();
    element_content_handlers.push(text!(
        "*:not(script):not(head):not(style):not(svg)",
        move |el| {
            text_seen_text.set(true);
            // code: every byte is kept, nothing is escaped
            if in_code_flag_text.get() {
                // code is real text: it ends the list marker fix below
                if list_item_start_flag_text.get() && !is_ascii_ws_only(el.as_str()) {
                    list_item_start_flag_text.set(false);
                }
                let last = el.last_in_text_node();
                if let Some(code) =
                    text_piece(&mut text_carry, el.as_str(), last, crate::code_text_opt)
                {
                    *el.as_mut_str() = code;
                }
                return Ok(());
            }

            let s = el.as_str();

            // inside table: ignore formatting whitespace between cells
            // A held entity is text, so the piece after it is never dropped.
            let held = !text_carry.is_empty();

            if in_table_flag_text.get() && !held && is_ascii_ws_only(s) {
                *el.as_mut_str() = String::new();
                return Ok(());
            }

            // list marker fix: swallow whitespace-only nodes until first real text
            if list_item_start_flag_text.get() {
                if !held && is_ascii_ws_only(s) {
                    *el.as_mut_str() = String::new();
                    return Ok(());
                }
                list_item_start_flag_text.set(false);
            }

            // Only allocate if escaping is actually needed
            let last = el.last_in_text_node();
            if let Some(escaped) =
                text_piece(&mut text_carry, s, last, crate::replace_markdown_chars_opt)
            {
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
    let in_code_flag_el = in_code_flag.clone();
    let in_pre_flag_el = in_pre_flag.clone();
    let table_head_el = table_head.clone();

    element_content_handlers.push(element!("*", move |el| {
        // The tag name is an allocation, so it is read once per element.
        let tag_name = el.tag_name();

        // Inside a `<pre>` a tag is highlighting or a line wrapper. It
        // writes no markdown; only its text is kept, and a `<br>` is a line
        // break.
        if in_pre_flag_el.get() {
            el.remove_and_keep_content();
            if tag_name == "br" {
                el.after("\n", lol_html::html_content::ContentType::Text);
            }
            return Ok(());
        }

        // Code start: text is code until the end tag. A `<pre>` with no end
        // tag runs to the end of the page, as it does in a browser.
        let is_pre = tag_name == "pre";
        if is_pre || ((tag_name == "code" || tag_name == "samp") && !in_code_flag_el.get()) {
            if let Some(hvec) = el.end_tag_handlers() {
                in_code_flag_el.set(true);
                in_pre_flag_el.set(is_pre);
                let in_code_flag_end = in_code_flag_el.clone();
                let in_pre_flag_end = in_pre_flag_el.clone();
                let h: LocalEndHandler =
                    Box::new(move |_end: &mut lol_html::html_content::EndTag<'_>| {
                        in_code_flag_end.set(false);
                        in_pre_flag_end.set(false);
                        Ok(())
                    });
                hvec.push(h);
            }
        }

        // Table start: enable flag and add end-tag handler to disable.
        if tag_name == "table" {
            in_table_flag_el.set(true);
            table_head_el.set(0);
            if let Some(hvec) = el.end_tag_handlers() {
                let in_table_flag_end = in_table_flag_el.clone();
                let table_head_end = table_head_el.clone();
                let h: LocalEndHandler =
                    Box::new(move |end: &mut lol_html::html_content::EndTag<'_>| {
                        in_table_flag_end.set(false);
                        // A table that is one header row still gets its
                        // delimiter row.
                        if let Some(row) = table_delimiter(table_head_end.get()) {
                            end.before(&row, lol_html::html_content::ContentType::Html);
                        }
                        table_head_end.set(0);
                        Ok(())
                    });
                hvec.push(h);
            }
        } else if in_table_flag_el.get() {
            if tag_name == "tr" {
                let (next, delimiter) = table_row(table_head_el.get());
                table_head_el.set(next);
                if let Some(row) = delimiter {
                    el.before(&row, lol_html::html_content::ContentType::Html);
                }
            } else if tag_name == "th" || tag_name == "td" {
                // Only the first row decides the delimiter row, so the
                // cells of every later row cost nothing here.
                let state = table_head_el.get();
                if state & T_ROWS == 1 {
                    let header = tag_name == "th"
                        && !spans_cells(el.get_attribute("colspan"))
                        && !spans_cells(el.get_attribute("rowspan"));
                    table_head_el.set(table_cell(state, header));
                }
            }
        }

        // sync state from flags
        in_table = in_table_flag_el.get();
        list_item_start = list_item_start_flag_el.get();

        let _ = handle_tag(
            el,
            &tag_name,
            commonmark,
            &url,
            &mut list_type,
            &mut order_counter,
            &quote_depth,
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
            doc_text!(move |c| {
                if !text_seen.replace(false) {
                    strip_unseen_markers(c);
                }
                Ok(())
            }),
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
    let mut list_type: Option<&'static str> = None;
    let mut order_counter = 0usize;

    let quote_depth = Arc::new(AtomicUsize::new(0));
    let quote_depth1 = quote_depth.clone();

    let repaired_head = Arc::new(std::sync::OnceLock::new());

    // packed flags (single atomic load per handler call)
    let flags = Arc::new(AtomicU8::new(0));
    // true once the text handler has seen the current text chunk (see
    // `strip_unseen_markers`)
    let text_seen = Arc::new(AtomicBool::new(false));
    // the first row of the current table (see `table_row`)
    let table_head = Arc::new(AtomicU32::new(0));

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

    // Code inside a quote keeps its lines: the quote prefix would join them.
    let flags_quote = flags.clone();
    element_content_handlers.push(text!("blockquote, q, cite", move |el| {
        if flags_quote.load(Ordering::Relaxed) & (F_IN_PRE | F_IN_CODE) == 0 {
            let _ = rewrite_blockquote_text_send(el, &quote_depth);
        }
        Ok(())
    }));

    // TEXT HANDLER (send): single atomic load + ASCII whitespace scan
    let flags_text = flags.clone();
    let text_seen_text = text_seen.clone();
    let mut text_carry = String::new();
    element_content_handlers.push(text!(
        "*:not(script):not(head):not(style):not(svg)",
        move |el| {
            text_seen_text.store(true, Ordering::Relaxed);
            let f = flags_text.load(Ordering::Relaxed);
            let in_table_now = (f & F_IN_TABLE) != 0;
            let li_start_now = (f & F_LI_START) != 0;

            // code: every byte is kept, nothing is escaped
            if (f & (F_IN_PRE | F_IN_CODE)) != 0 {
                // code is real text: it ends the list marker fix below
                if li_start_now && !is_ascii_ws_only(el.as_str()) {
                    flag_clear(&*flags_text, F_LI_START);
                }
                let last = el.last_in_text_node();
                if let Some(code) =
                    text_piece(&mut text_carry, el.as_str(), last, crate::code_text_opt)
                {
                    *el.as_mut_str() = code;
                }
                return Ok(());
            }

            let s = el.as_str();

            // A held entity is text, so the piece after it is never dropped.
            let held = !text_carry.is_empty();

            if in_table_now && !held && is_ascii_ws_only(s) {
                *el.as_mut_str() = String::new();
                return Ok(());
            }

            if li_start_now {
                if !held && is_ascii_ws_only(s) {
                    *el.as_mut_str() = String::new();
                    return Ok(());
                }
                // clear li-start
                flag_clear(&*flags_text, F_LI_START);
            }

            // Only allocate if escaping is actually needed
            let last = el.last_in_text_node();
            if let Some(escaped) =
                text_piece(&mut text_carry, s, last, crate::replace_markdown_chars_opt)
            {
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
    let table_head_el = table_head.clone();
    element_content_handlers.push(element!("*", move |el| {
        // The tag name is an allocation, so it is read once per element.
        let tag_name = el.tag_name();

        let f = flags_el.load(Ordering::Relaxed);

        // Inside a `<pre>` a tag is highlighting or a line wrapper. It
        // writes no markdown; only its text is kept, and a `<br>` is a line
        // break.
        if (f & F_IN_PRE) != 0 {
            el.remove_and_keep_content();
            if tag_name == "br" {
                el.after("\n", lol_html::html_content::ContentType::Text);
            }
            return Ok(());
        }

        // Code start: text is code until the end tag. A `<pre>` with no end
        // tag runs to the end of the page, as it does in a browser.
        let is_pre = tag_name == "pre";
        if is_pre || ((tag_name == "code" || tag_name == "samp") && (f & F_IN_CODE) == 0) {
            if let Some(hvec) = el.end_tag_handlers() {
                flag_set(&*flags_el, if is_pre { F_IN_PRE } else { F_IN_CODE });
                let flags_end = flags_el.clone();
                let h: EndHandler =
                    Box::new(move |_end: &mut lol_html::html_content::EndTag<'_>| {
                        flag_clear(&*flags_end, F_IN_PRE | F_IN_CODE);
                        Ok(())
                    });
                hvec.push(h);
            }
        }

        // table start
        if tag_name == "table" {
            flag_set(&*flags_el, F_IN_TABLE);
            table_head_el.store(0, Ordering::Relaxed);

            if let Some(hvec) = el.end_tag_handlers() {
                let flags_end = flags_el.clone();
                let table_head_end = table_head_el.clone();
                let h: EndHandler =
                    Box::new(move |end: &mut lol_html::html_content::EndTag<'_>| {
                        flag_clear(&*flags_end, F_IN_TABLE);
                        // A table that is one header row still gets its
                        // delimiter row.
                        if let Some(row) =
                            table_delimiter(table_head_end.swap(0, Ordering::Relaxed))
                        {
                            end.before(&row, lol_html::html_content::ContentType::Html);
                        }
                        Ok(())
                    });
                hvec.push(h);
            }
        } else if (f & F_IN_TABLE) != 0 {
            if tag_name == "tr" {
                let (next, delimiter) = table_row(table_head_el.load(Ordering::Relaxed));
                table_head_el.store(next, Ordering::Relaxed);
                if let Some(row) = delimiter {
                    el.before(&row, lol_html::html_content::ContentType::Html);
                }
            } else if tag_name == "th" || tag_name == "td" {
                // Only the first row decides the delimiter row, so the
                // cells of every later row cost nothing here.
                let state = table_head_el.load(Ordering::Relaxed);
                if state & T_ROWS == 1 {
                    let header = tag_name == "th"
                        && !spans_cells(el.get_attribute("colspan"))
                        && !spans_cells(el.get_attribute("rowspan"));
                    table_head_el.store(table_cell(state, header), Ordering::Relaxed);
                }
            }
        }

        // local bools for handle_tag_send
        let f = flags_el.load(Ordering::Relaxed);
        in_table = (f & F_IN_TABLE) != 0;
        list_item_start = (f & F_LI_START) != 0;

        let _ = handle_tag_send(
            el,
            &tag_name,
            commonmark,
            &url,
            &mut list_type,
            &mut order_counter,
            &quote_depth1,
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
            doc_text!(move |c| {
                if !text_seen.swap(false, Ordering::Relaxed) {
                    strip_unseen_markers(c);
                }
                Ok(())
            }),
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

/// Drop marker bytes from a text chunk the text handler never saw: text
/// outside every element, or inside one its selector skips. Text the handler
/// sees has its markers dropped there, so this looks at the other text only,
/// and only text that holds a marker byte is copied.
fn strip_unseen_markers(chunk: &mut lol_html::html_content::TextChunk<'_>) {
    if let std::borrow::Cow::Owned(kept) = crate::strip_sift_markers(chunk.as_str()) {
        *chunk.as_mut_str() = kept;
    }
}

/// Rewrite the HTML and sift the markdown as the rewriter writes it, so the
/// output is produced in one pass.
pub(crate) fn rewrite_str<'h, 's, H: lol_html::HandlerTypes>(
    html: &str,
    settings: impl Into<lol_html::Settings<'h, 's, H>>,
) -> Result<String, lol_html::errors::RewritingError> {
    let mut sink = SiftSink::with_capacity(estimate_markdown(html));

    let mut rewriter = lol_html::HtmlRewriter::new(settings.into(), |c: &[u8]| {
        sink.push(c);
    });

    rewriter.write(html.as_bytes())?;
    rewriter.end()?;

    Ok(sink.finish())
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
        Ok(markdown) => Ok(markdown),
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
    // The rewriter writes into the sift, so the markdown is cleaned as it is
    // produced and the output is written once.
    let mut sink = SiftSink::with_capacity(estimate_markdown(html));

    {
        let mut rewriter = lol_html::send::HtmlRewriter::new(settings.into(), |c: &[u8]| {
            sink.push(c);
        });

        // Process in chunks without async overhead for in-memory data.
        // `chunks` panics on a size of 0, so a 0 is read as 1.
        let mut wrote_error = false;
        for chunk in html.as_bytes().chunks(chunk_size.max(1)) {
            if rewriter.write(chunk).is_err() {
                wrote_error = true;
                break;
            }
        }

        if !wrote_error {
            let _ = rewriter.end();
        }
    }

    Ok(sink.finish())
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

/// Error type for stream-based conversion.
#[cfg(feature = "stream")]
#[derive(Debug)]
pub enum StreamConvertError<E> {
    /// The input stream yielded an error.
    Stream(E),
    /// lol_html rewriting failed.
    Rewrite(lol_html::errors::RewritingError),
}

#[cfg(feature = "stream")]
impl<E: std::fmt::Display> std::fmt::Display for StreamConvertError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stream(e) => write!(f, "stream error: {e}"),
            Self::Rewrite(e) => write!(f, "rewrite error: {e}"),
        }
    }
}

#[cfg(feature = "stream")]
impl<E: std::error::Error + 'static> std::error::Error for StreamConvertError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stream(e) => Some(e),
            Self::Rewrite(e) => Some(e),
        }
    }
}

/// Convert an async byte stream of HTML into markdown.
///
/// Genuinely async — yields to the executor between input chunks via `stream.next().await`.
/// Uses `lol_html::send::HtmlRewriter` which handles chunk-boundary splitting internally.
#[cfg(feature = "stream")]
pub async fn convert_html_stream_to_markdown<S, B, E>(
    stream: S,
    custom: &Option<std::collections::HashSet<String>>,
    commonmark: bool,
    url: &Option<Url>,
) -> Result<String, StreamConvertError<E>>
where
    S: futures_util::Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
{
    use futures_util::StreamExt;

    let settings = get_rewriter_settings_send(commonmark, custom, url.clone());
    let mut sink = SiftSink::with_capacity(4096);

    let mut rewriter = lol_html::send::HtmlRewriter::new(settings.into(), |c: &[u8]| {
        sink.push(c);
    });

    futures_util::pin_mut!(stream);

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(StreamConvertError::Stream)?;
        rewriter
            .write(chunk.as_ref())
            .map_err(StreamConvertError::Rewrite)?;
    }

    rewriter.end().map_err(StreamConvertError::Rewrite)?;

    Ok(sink.finish())
}

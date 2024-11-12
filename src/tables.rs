use super::StructuredPrinter;
use super::TagHandler;
use super::{clean_markdown, walk};
use std::borrow::BorrowMut;
use std::sync::Arc;
use std::{cmp, collections::HashMap};

use html5ever::LocalName;
use markup5ever_rcdom::{Handle, NodeData};
use url::Url;

#[derive(Default)]
pub struct TableHandler {
    commonmark: bool,
    url: Option<Arc<Url>>,
}

const TD: LocalName = html5ever::local_name!("td");
const TH: LocalName = html5ever::local_name!("th");

const TABLE_LIMIT: usize = 1000;

impl TableHandler {
    /// A new table handler.
    pub fn new(commonmark: bool, url: Option<std::sync::Arc<Url>>) -> Self {
        TableHandler { commonmark, url }
    }
}

impl TagHandler for TableHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        let mut table_markup = String::new();

        let any_matcher = |cell: &Handle| match cell.data {
            NodeData::Element { ref name, .. } => name.local == TD || name.local == TH,
            _ => false,
        };

        // detect cell width, counts
        let mut column_widths: Vec<usize> = Vec::new();
        let rows = find_children(tag, "tr");

        // detect row count
        let most_big_row = rows.iter().max_by(|left, right| {
            collect_children(left, any_matcher)
                .len()
                .cmp(&collect_children(right, any_matcher).len())
        });

        if most_big_row.is_some() {
            let column_count = match most_big_row {
                Some(tag) => collect_children(tag, any_matcher).len(),
                _ => 0,
            };

            column_widths = vec![3; column_count];

            // detect max column width
            for (idx, row) in rows.iter().enumerate() {
                if idx >= TABLE_LIMIT {
                    break;
                }

                let cells = collect_children(row, any_matcher);

                for index in 0..column_count {
                    // from regular rows
                    if let Some(cell) = cells.get(index) {
                        let text = to_text(cell, self.commonmark, &self.url);

                        column_widths[index] = cmp::max(column_widths[index], text.chars().count());
                    }
                }
            }

            // header row must always be present
            for (idx, row) in rows.iter().enumerate() {
                if idx >= TABLE_LIMIT {
                    break;
                }

                table_markup.push('|');

                let cells = collect_children(row, any_matcher);

                for index in 0..column_count {
                    // we need to fill all cells in a column, even if some rows don't have enough
                    let padded_cell_text = pad_cell_text(
                        &cells.get(index),
                        column_widths[index],
                        self.commonmark,
                        &self.url,
                    );

                    table_markup.push_str(&padded_cell_text);
                    table_markup.push('|');
                }

                table_markup.push('\n');

                if idx == 0 {
                    // first row is a header row
                    // add header-body divider row
                    table_markup.push('|');
                    for index in 0..column_count {
                        let width = column_widths[index];

                        if width < 3 {
                            // no point in aligning, just post as-is
                            table_markup.push_str(&"-".repeat(width.max(2)));
                            table_markup.push('|');
                            continue;
                        }

                        // // try to detect alignment
                        // let mut alignment = String::new();

                        // if let Some(header_cell) = cells.get(index) {
                        //     // we have a header, try to extract alignment from it
                        //     alignment = match header_cell.data {
                        //         NodeData::Element { ref attrs, .. } => {
                        //             let attrs = attrs.borrow();
                        //             let align_attr = attrs
                        //                 .iter()
                        //                 .find(|attr| attr.name.local.to_string() == "align");
                        //             align_attr
                        //                 .map(|attr| attr.value.to_string())
                        //                 .unwrap_or_default()
                        //         }
                        //         _ => String::new(),
                        //     };
                        // }

                        // push lines according to alignment, fallback to default behaviour
                        // match alignment.as_ref() {
                        //     "left" => {
                        //         table_markup.push(':');
                        //         table_markup.push_str(&"-".repeat(width - 1));
                        //     }
                        //     "center" => {
                        //         table_markup.push(':');
                        //         table_markup.push_str(&"-".repeat(width - 2));
                        //         table_markup.push(':');
                        //     }
                        //     "right" => {
                        //         table_markup.push_str(&"-".repeat(width - 1));
                        //         table_markup.push(':');
                        //     }
                        //     _ => table_markup.push_str(&"-".repeat(width)),
                        // }

                        table_markup.push('|');
                    }

                    table_markup.push('\n');
                }
            }

            // remove the children to prevent infinite loops
            tag.children.take();

            printer.insert_newline();
            printer.insert_newline();
            printer.append_str(&table_markup);
        }
    }

    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {}

    fn skip_descendants(&self) -> bool {
        true
    }
}

/// Pads cell text from right and left so it looks centered inside the table cell
/// ### Arguments
/// `tag` - optional reference to currently processed handle, text is extracted from here
///
/// `column_width` - precomputed column width to compute padding length from
fn pad_cell_text(
    tag: &Option<&Handle>,
    column_width: usize,
    commonmark: bool,
    url: &Option<Arc<Url>>,
) -> String {
    let mut result = String::new();

    if let Some(cell) = tag {
        // have header at specified position
        let text = to_text(cell, commonmark, url);

        // compute difference between width and text length
        let len_diff = column_width
            .checked_sub(text.chars().count())
            .unwrap_or_default();

        if len_diff > 0 {
            // should pad
            if len_diff > 1 {
                result.push(' ');
                result.push_str(&text);
                result.push(' ');
            } else {
                // it's just one space, add at the end
                result.push_str(&text);
                result.push(' ');
            }
        } else {
            // shouldn't pad, text fills whole cell
            result.push_str(&text);
        }
    } else {
        // no text in this cell, fill cell with spaces
        result.push(' ');
    }

    result
}

/// Extracts tag name from passed tag
/// Returns empty string if it's not an html element
fn tag_name(tag: &Handle) -> String {
    match tag.data {
        NodeData::Element { ref name, .. } => name.local.to_string(),
        _ => String::new(),
    }
}

/// Find descendants of this tag with tag name `name`
/// This includes both direct children and descendants
fn find_children(tag: &Handle, name: &str) -> Vec<Handle> {
    let children = tag.children.borrow();
    let mut result: Vec<Handle> = vec![];

    for child in children.iter() {
        if tag_name(child) == name {
            result.push(child.clone());
        }

        let mut descendants = find_children(child, name);
        result.append(&mut descendants);
    }

    result
}

/// Collect direct children that satisfy the predicate
/// This doesn't include descendants
fn collect_children<P>(tag: &Handle, predicate: P) -> Vec<Handle>
where
    P: Fn(&Handle) -> bool,
{
    let children = tag.children.borrow();
    let mut result: Vec<Handle> = Vec::with_capacity(children.len());

    for child in children.iter() {
        if predicate(child) {
            result.push(child.clone());
        }
    }

    result
}

/// Convert html tag to text. This collects all tag children in correct order where they're observed
/// and concatenates their text, recursively.
fn to_text(tag: &Handle, commonmark: bool, url: &Option<Arc<Url>>) -> String {
    let mut printer = StructuredPrinter::default();

    walk(
        &tag,
        &mut printer,
        &HashMap::default(),
        commonmark,
        &url,
        true,
    );

    clean_markdown(&printer.data)
}

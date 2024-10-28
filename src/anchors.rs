use super::StructuredPrinter;
use super::TagHandler;
use percent_encoding::percent_decode_str;
use std::borrow::Cow;

use markup5ever_rcdom::{Handle, NodeData};

#[derive(Default)]
pub struct AnchorHandler {
    start_pos: usize,
    url: String,
}

impl TagHandler for AnchorHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        self.start_pos = printer.data.len();
        self.url = match tag.data {
            NodeData::Element { ref attrs, .. } => {
                let attrs = attrs.borrow();
                let href = attrs
                    .iter()
                    .find(|attr| attr.name.local.as_bytes() == b"href");

                match href {
                    Some(link) => link.value.trim().into(),
                    None => String::new(),
                }
            }
            _ => String::new(),
        };
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
        // Percent decode url.
        let url = percent_decode_str(&self.url).decode_utf8_lossy();
        // [CommonMark Spec](https://spec.commonmark.org/0.31.2/#link-destination)
        let url = if url.contains(|c: char| c.is_ascii_control() || c == ' ') {
            Cow::Owned(format!("<{}>", url))
        } else {
            url
        };

        match printer.data.get(self.start_pos..) {
            Some(d) => {
                let starts_new_line = d.starts_with("\n");
                let ends_new_line = d.ends_with("\n");

                if starts_new_line || ends_new_line {
                    // handle start
                    if starts_new_line {
                        printer.insert_str(self.start_pos + 1, "[");
                    } else {
                        printer.insert_str(self.start_pos, "[");
                    }

                    // handle end
                    if ends_new_line {
                        let next_position = printer.data.len();
                        printer.insert_str(next_position - 1, &format!("]({})", url));
                    } else {
                        printer.append_str(&format!("]({})", url));
                    }
                } else {
                    printer.insert_str(self.start_pos, "[");
                    printer.append_str(&format!("]({})", url));
                }
            }
            _ => {
                printer.insert_str(self.start_pos, "[");
                printer.append_str(&format!("]({})", url));
            }
        }
    }
}

use super::StructuredPrinter;
use super::TagHandler;

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
                    Some(link) => link.value.trim_ascii().into(),
                    None => String::new(),
                }
            }
            _ => String::new(),
        };
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
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
                        printer.insert_str(next_position - 1, &format!("]({})", self.url));
                    } else {
                        printer.append_str(&format!("]({})", self.url));
                    }
                } else {
                    printer.insert_str(self.start_pos, "[");
                    printer.append_str(&format!("]({})", self.url));
                }
            }
            _ => {
                printer.insert_str(self.start_pos, "[");
                printer.append_str(&format!("]({})", self.url));
            }
        }
    }
}

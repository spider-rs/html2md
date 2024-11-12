use super::StructuredPrinter;
use super::TagHandler;

use auto_encoder::encode_bytes_from_language;
use html5ever::serialize;
use html5ever::serialize::{SerializeOpts, TraversalScope};
use markup5ever_rcdom::{Handle, NodeData, SerializableHandle};

#[derive(Default)]
pub struct DummyHandler;

impl TagHandler for DummyHandler {
    fn handle(&mut self, _tag: &Handle, _printer: &mut StructuredPrinter) {}
    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {}
}

/// Handler that completely copies tag to printer as HTML with all descendants
#[derive(Default)]
pub struct IdentityHandler {
    /// Commonmark spec
    pub commonmark: bool,
}

impl IdentityHandler {
    /// A new identity handler.
    pub fn new(commonmark: bool) -> Self {
        Self { commonmark }
    }
}

impl TagHandler for IdentityHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        let mut buffer = vec![];

        let options = SerializeOpts {
            traversal_scope: if self.commonmark {
                TraversalScope::IncludeNode
            } else {
                TraversalScope::ChildrenOnly(None)
            },
            ..Default::default()
        };
        let to_be_serialized = SerializableHandle::from(tag.clone());

        let result = serialize(&mut buffer, &to_be_serialized, options);

        if result.is_err() {
            // couldn't serialize the tag
            return;
        }

        // make sure to have encoding
        let conv = encode_bytes_from_language(&buffer, "");
        printer.append_str(&conv);
    }

    fn skip_descendants(&self) -> bool {
        true
    }

    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {}
}

/// Handler that copies just one tag and doesn't skip descendants
#[derive(Default)]
pub struct HtmlCherryPickHandler {
    tag_name: String,
    /// if commonmark or not
    commonmark: bool,
}

impl HtmlCherryPickHandler {
    /// Create a new cherry pick handler.
    pub fn new(commonmark: bool) -> Self {
        Self {
            tag_name: Default::default(),
            commonmark,
        }
    }
}

impl TagHandler for HtmlCherryPickHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        if let NodeData::Element {
            ref name,
            ref attrs,
            ..
        } = tag.data
        {
            let attrs = attrs.borrow();
            self.tag_name = name.local.to_string();

            if self.commonmark {
                printer.append_str(&format!("<{}", self.tag_name));

                for attr in attrs.iter() {
                    printer.append_str(&format!(" {}=\"{}\"", attr.name.local, attr.value));
                }

                printer.append_str(">");
            }
        }
    }

    fn skip_descendants(&self) -> bool {
        false
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
        if self.commonmark {
            printer.append_str(&format!("</{}>", self.tag_name));
        }
    }
}

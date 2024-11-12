use super::Handle;
use super::StructuredPrinter;
use super::TagHandler;
use super::TagHandlerFactory;

#[derive(Clone)]
/// Ignore the tag complete from the markup.
pub struct IgnoreTagFactory;

impl TagHandlerFactory for IgnoreTagFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(self.clone())
    }
}

impl TagHandler for IgnoreTagFactory {
    fn handle(&mut self, _tag: &Handle, _printer: &mut StructuredPrinter) {}
    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {}
    fn skip_descendants(&self) -> bool {
        true
    }
}

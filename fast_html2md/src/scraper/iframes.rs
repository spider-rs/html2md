use super::StructuredPrinter;
use super::TagHandler;

use super::common::get_tag_attr;
use super::dummy::IdentityHandler;

use crate::extended::base::iframe::{INSTAGRAM_PATTERN, VK_PATTERN, YOUTUBE_PATTERN};
use markup5ever_rcdom::Handle;

#[derive(Default)]
pub struct IframeHandler;

impl TagHandler for IframeHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        printer.insert_newline();

        let src = get_tag_attr(tag, "src");
        //let width = get_tag_attr(tag, "width");
        //let height = get_tag_attr(tag, "height");

        if let Some(src) = src {
            if let Some(capture) = YOUTUBE_PATTERN.captures(&src) {
                let media_id = capture.get(1).map_or("", |m| m.as_str());
                printer.append_str(&format!("[![Embedded YouTube video](https://img.youtube.com/vi/{mid}/0.jpg)](https://www.youtube.com/watch?v={mid})", mid = media_id));
                return;
            }

            if let Some(capture) = INSTAGRAM_PATTERN.captures(&src) {
                let media_id = capture.get(1).map_or("", |m| m.as_str());
                printer.append_str(&format!("[![Embedded Instagram post](https://www.instagram.com/p/{mid}/media/?size=m)](https://www.instagram.com/p/{mid}/embed/)", mid = media_id));
                return;
            }

            if let Some(capture) = VK_PATTERN.captures(&src) {
                let owner_id = capture.get(1).map_or("", |m| m.as_str());
                let video_id = capture.get(2).map_or("", |m| m.as_str());
                let _hash = capture.get(3).map_or("", |m| m.as_str());
                printer.append_str(&format!("[![Embedded VK video](https://st.vk.com/images/icons/video_empty_2x.png)](https://vk.com/video{oid}_{vid})", oid = owner_id, vid = video_id));
                return;
            }

            // not found, use generic implementation
            let mut identity = IdentityHandler::default();
            identity.handle(tag, printer);
        }
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
        printer.insert_newline();
    }

    fn skip_descendants(&self) -> bool {
        true
    }
}

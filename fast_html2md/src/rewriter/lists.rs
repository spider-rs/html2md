use super::counter::Counter;
use lol_html::html_content::ContentType;
use lol_html::html_content::Element;
use std::cell::RefCell;
use std::rc::Rc;

// Function to handle list elements and items
#[inline]
pub(crate) fn handle_list_or_item(
    element: &mut Element,
    list_type: Rc<RefCell<Option<String>>>,
    order_counter: Rc<RefCell<usize>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match element.tag_name().as_str() {
        "ul" | "menu" => {
            *list_type.borrow_mut() = Some("ul".to_string());
            order_counter.borrow_mut().reset(); // Reset the order counter for a new list
        }
        "ol" => {
            *list_type.borrow_mut() = Some("ol".to_string());
            order_counter.borrow_mut().reset();
        }
        "li" => {
            if list_type.borrow().as_deref() == Some("ol") {
                let order = order_counter.borrow_mut().increment();
                element.before(&format!("\n{}. ", order), ContentType::Text);
            } else {
                element.before("\n* ", ContentType::Text);
            }
        }
        _ => (),
    }

    Ok(())
}

use super::counter::Counter;
use lol_html::html_content::ContentType;
use lol_html::html_content::Element;

// Function to handle list elements and items
#[inline]
pub(crate) fn handle_list_or_item(
    element: &mut Element,
    list_type: &mut Option<String>,
    order_counter: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match element.tag_name().as_str() {
        "ul" | "menu" => {
            *list_type = Some("ul".to_string());
            order_counter.reset(); // Reset the order counter for a new list
        }
        "ol" => {
            *list_type = Some("ol".to_string());
            order_counter.reset();
        }
        "li" => {
            if list_type.as_deref() == Some("ol") {
                let order = order_counter.increment();
                element.before(&format!("\n{}. ", order), ContentType::Text);
            } else {
                element.before("\n* ", ContentType::Text);
            }
        }
        _ => (),
    }

    Ok(())
}

// Function to handle list elements and items
#[inline]
pub(crate) fn handle_list_or_item_send(
    element: &mut lol_html::send::Element,
    list_type: &mut Option<String>,
    order_counter: &mut usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match element.tag_name().as_str() {
        "ul" | "menu" => {
            *list_type = Some("ul".to_string());

            order_counter.reset();
        }
        "ol" => {
            *list_type = Some("ol".to_string());

            order_counter.reset();
        }
        "li" => {
            let ordered: bool = list_type.as_deref().eq(&Some("ol"));

            if ordered {
                let order = order_counter.increment();

                element.before(&format!("\n{}. ", order), ContentType::Text);
            } else {
                element.before("\n* ", ContentType::Text);
            }
        }
        _ => (),
    }

    Ok(())
}

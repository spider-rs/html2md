use super::counter::Counter;
use lol_html::html_content::ContentType;
use lol_html::html_content::Element;

/// Function to handle list elements and items
///
/// IMPORTANT: `list_item_start` is set to true when we emit a list marker.
/// The tag handler uses it to avoid inserting a newline before the first <p>
/// and the text handler can drop whitespace-only nodes while this is true.
#[inline]
pub(crate) fn handle_list_or_item(
    element: &mut Element,
    list_type: &mut Option<String>,
    order_counter: &mut usize,
    list_item_start: &mut bool,
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
            // We are emitting a marker; next meaningful content should stay on same line.
            *list_item_start = true;

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

/// Function to handle list elements and items (send)
#[inline]
pub(crate) fn handle_list_or_item_send(
    element: &mut lol_html::send::Element,
    list_type: &mut Option<String>,
    order_counter: &mut usize,
    list_item_start: &mut bool,
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
            *list_item_start = true;

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

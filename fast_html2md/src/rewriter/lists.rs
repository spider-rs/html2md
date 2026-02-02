use super::counter::Counter;
use lol_html::html_content::ContentType;
use lol_html::html_content::Element;

/// Pre-computed ordered list markers for common cases (1-20).
/// Avoids format! allocation for the most common list lengths.
const OL_MARKERS: [&str; 21] = [
    "\n0. ", "\n1. ", "\n2. ", "\n3. ", "\n4. ", "\n5. ", "\n6. ", "\n7. ", "\n8. ", "\n9. ",
    "\n10. ", "\n11. ", "\n12. ", "\n13. ", "\n14. ", "\n15. ", "\n16. ", "\n17. ", "\n18. ",
    "\n19. ", "\n20. ",
];

/// Get ordered list marker, using pre-computed for common cases.
#[inline]
fn get_ol_marker(n: usize) -> std::borrow::Cow<'static, str> {
    if n < OL_MARKERS.len() {
        std::borrow::Cow::Borrowed(OL_MARKERS[n])
    } else {
        std::borrow::Cow::Owned(format!("\n{}. ", n))
    }
}

/// Function to handle list elements and items
///
/// IMPORTANT: `list_item_start` is set to true when we emit a list marker.
/// The tag handler uses it to avoid inserting a newline before the first <p>
/// and the text handler can drop whitespace-only nodes while this is true.
#[inline]
pub(crate) fn handle_list_or_item(
    element: &mut Element,
    list_type: &mut Option<&'static str>,
    order_counter: &mut usize,
    list_item_start: &mut bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match element.tag_name().as_str() {
        "ul" | "menu" => {
            *list_type = Some("ul");
            order_counter.reset();
        }
        "ol" => {
            *list_type = Some("ol");
            order_counter.reset();
        }
        "li" => {
            *list_item_start = true;

            if *list_type == Some("ol") {
                let order = order_counter.increment();
                element.before(&get_ol_marker(order), ContentType::Text);
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
    list_type: &mut Option<&'static str>,
    order_counter: &mut usize,
    list_item_start: &mut bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match element.tag_name().as_str() {
        "ul" | "menu" => {
            *list_type = Some("ul");
            order_counter.reset();
        }
        "ol" => {
            *list_type = Some("ol");
            order_counter.reset();
        }
        "li" => {
            *list_item_start = true;

            if *list_type == Some("ol") {
                let order = order_counter.increment();
                element.before(&get_ol_marker(order), ContentType::Text);
            } else {
                element.before("\n* ", ContentType::Text);
            }
        }
        _ => (),
    }

    Ok(())
}

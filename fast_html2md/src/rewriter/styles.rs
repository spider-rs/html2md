use lol_html::html_content::{ContentType::Text, Element};

/// Rewrite the initial elements that need extra styles.
pub(crate) fn rewrite_style_element(
    el: &mut Element,
    tag_name: &str,
) -> Result<(), std::io::Error> {
    let mark = match tag_name {
        "b" | "strong" => "**",
        "i" | "em" => "*",
        "s" | "del" => "~~",
        "u" | "ins" => "__",
        _ => return Ok(()), // Return early if tag is not one of the specified
    };

    el.before(mark, Text);
    el.after(mark, Text);

    Ok(())
}

/// Rewrite the initial elements that need extra styles.
pub(crate) fn rewrite_style_element_send(
    el: &mut lol_html::send::Element,
    tag_name: &str,
) -> Result<(), std::io::Error> {
    let mark = match tag_name {
        "b" | "strong" => "**",
        "i" | "em" => "*",
        "s" | "del" => "~~",
        "u" | "ins" => "__",
        _ => return Ok(()), // Return early if tag is not one of the specified
    };

    el.before(mark, Text);
    el.after(mark, Text);

    Ok(())
}

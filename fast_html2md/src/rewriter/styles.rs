use lol_html::html_content::{ContentType::Text, Element};

/// Rewrite the initial elements that need extra styles.
pub(crate) fn rewrite_style_element(el: &mut Element) -> Result<(), std::io::Error> {
    let tag_name = el.tag_name();

    let mark = match tag_name.as_str() {
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

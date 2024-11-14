use lol_html::html_content::Element;

/// Rewrite the initial elements that need extra styles.
pub(crate) fn rewrite_style_element(el: &mut Element) -> Result<(), std::io::Error> {
    let tag_name = el.tag_name().to_ascii_lowercase();
    let mark = match tag_name.as_str() {
        "b" | "strong" => "**",
        "i" | "em" => "*",
        "s" | "del" => "~~",
        "u" | "ins" => "__",
        _ => return Ok(()), // Return early if tag is not one of the specified
    };

    // Apply the markup before the element's content
    el.before(mark, lol_html::html_content::ContentType::Text);

    // Apply the markup after the element's content
    el.after(mark, lol_html::html_content::ContentType::Text);

    Ok(())
}

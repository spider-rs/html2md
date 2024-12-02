pub(crate) mod anchors;
pub(crate) mod counter;
pub(crate) mod handle;
pub(crate) mod iframes;
pub(crate) mod images;
pub(crate) mod lists;
pub(crate) mod quotes;
pub(crate) mod styles;
pub mod writer;

/// Insert a new line after
#[inline]
pub(crate) fn insert_newline_after(element: &mut lol_html::html_content::Element) {
    element.after("\n", lol_html::html_content::ContentType::Text);
}

/// Insert a new line before
#[inline]
pub(crate) fn insert_newline_before(element: &mut lol_html::html_content::Element) {
    element.before("\n", lol_html::html_content::ContentType::Text);
}

/// Insert a new line after
#[inline]
pub(crate) fn insert_newline_after_send(element: &mut lol_html::send::Element) {
    element.after("\n", lol_html::html_content::ContentType::Text);
}

/// Insert a new line before
#[inline]
pub(crate) fn insert_newline_before_send(element: &mut lol_html::send::Element) {
    element.before("\n", lol_html::html_content::ContentType::Text);
}

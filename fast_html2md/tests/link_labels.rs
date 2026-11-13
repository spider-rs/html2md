//! Link labels keep their text on one line.
//!
//! HTML collapses the whitespace inside an inline element, so
//! `<a>\n  Documentation\n</a>` reads as "Documentation". The markdown label
//! should read the same: `[Documentation](url)`, not `[`, `Documentation`,
//! `](url)` on three lines.

use html2md::{
    rewrite_html, rewrite_html_custom_with_url, rewrite_html_custom_with_url_and_chunk,
    rewrite_html_custom_with_url_streaming,
};
use url::Url;

fn md(html: &str) -> String {
    rewrite_html(html, false)
}

fn md_url(html: &str, url: &str) -> String {
    rewrite_html_custom_with_url(html, &None, false, &Some(Url::parse(url).unwrap()))
}

fn assert_no_markers(s: &str) {
    assert!(
        !s.bytes().any(|b| b == 0x02 || b == 0x03),
        "marker byte leaked: {s:?}"
    );
}

#[test]
fn go_dev_breadcrumb_list() {
    let html = "<ol><li><a href=\"/doc/\">\n  Documentation\n</a></li>\
                <li><a href=\"/doc/effective_go\">\n  Effective Go\n</a></li></ol>";
    let out = md_url(html, "https://go.dev/doc/effective_go");
    assert!(out.contains("1. [Documentation](https://go.dev/doc/)"), "{out}");
    assert!(
        out.contains("2. [Effective Go](https://go.dev/doc/effective_go)"),
        "{out}"
    );
    assert_no_markers(&out);
}

#[test]
fn single_line_links_unchanged() {
    assert_eq!(
        md("<p>see <a href=\"https://a.example/x\">the docs</a> now</p>"),
        "see [the docs](https://a.example/x) now"
    );
    assert_eq!(
        md("<a href=\"https://a.example/\">a  b</a>"),
        "[a b](https://a.example/)"
    );
}

#[test]
fn inner_whitespace_folds_and_trims() {
    let out = md("<a href=\"https://a.example/\">\n\t one \n\n two\t\n</a>");
    assert_eq!(out, "[one two](https://a.example/)");
}

#[test]
fn nbsp_inside_label_folds() {
    let out = md("<a href=\"https://a.example/\">\u{a0}one\u{a0}\u{a0}two\u{a0}</a>");
    assert_eq!(out, "[one two](https://a.example/)");
}

#[test]
fn empty_label() {
    assert_eq!(md("<a href=\"https://a.example/\">\n\n</a>"), "[](https://a.example/)");
    assert_eq!(md("<a href=\"https://a.example/\"></a>"), "[](https://a.example/)");
}

#[test]
fn image_inside_label() {
    let out = md("<a href=\"https://a.example/\">\n<img src=\"https://a.example/l.png\" alt=\"logo\">\nHome\n</a>");
    assert_eq!(out, "[![logo](https://a.example/l.png) Home](https://a.example/)");
}

#[test]
fn text_around_link_keeps_its_lines() {
    let out = md("<p>one</p><p><a href=\"https://a.example/\">\nlink\n</a></p><p>two</p>");
    let flat = md("<p>one</p><p><a href=\"https://a.example/\">link</a></p><p>two</p>");
    assert_eq!(out, flat);
    assert_eq!(out, "one\n[link](https://a.example/)\ntwo");
}

#[test]
fn many_links_in_a_list() {
    let mut html = String::from("<ul>");
    for i in 0..50 {
        html.push_str(&format!("<li><a href=\"https://a.example/{i}\">\n  item {i}\n</a></li>"));
    }
    html.push_str("</ul>");
    let out = md(&html);
    for i in 0..50 {
        assert!(
            out.contains(&format!("* [item {i}](https://a.example/{i})")),
            "{i}: {out}"
        );
    }
    assert_no_markers(&out);
}

#[test]
fn literal_marker_bytes_in_page_are_dropped() {
    let out = md("<p>a\u{2}b\u{3}c</p><a href=\"https://a.example/\" title=\"t\u{2}\">x\u{3}y</a>\
                  <img src=\"https://a.example/i\u{2}.png\" alt=\"a\u{3}lt\">");
    assert_no_markers(&out);
    assert!(out.contains("abc"), "{out}");
    assert!(out.contains("[xy](https://a.example/)"), "{out}");
}

#[test]
fn unclosed_anchor_at_eof() {
    let out = md("<p><a href=\"https://a.example/\">\nopen\nlabel");
    assert_no_markers(&out);
    assert!(out.contains("open label"), "{out}");
}

#[test]
fn nested_and_adjacent_anchors() {
    let out = md("<a href=\"https://a.example/1\">\none\n</a><a href=\"https://a.example/2\">\ntwo\n</a>");
    assert_no_markers(&out);
    assert!(out.contains("[one](https://a.example/1)"), "{out}");
    assert!(out.contains("[two](https://a.example/2)"), "{out}");
    let nested = md("<a href=\"https://a.example/o\">x <a href=\"https://a.example/i\">\ny\n</a> z</a>");
    assert_no_markers(&nested);
}

#[test]
fn crlf_inside_label() {
    let out = md("<a href=\"https://a.example/\">\r\n one\r\n two\r\n</a>");
    assert_eq!(out, "[one two](https://a.example/)");
}

#[test]
fn code_block_outside_label_untouched() {
    let html = "<pre><code>line1\n  line2\n</code></pre><a href=\"https://a.example/\">\nx\n</a>";
    let base = md("<pre><code>line1\n  line2\n</code></pre>");
    let out = md(html);
    assert!(out.starts_with(base.trim_end()), "{out}\n---\n{base}");
    assert!(out.contains("[x](https://a.example/)"), "{out}");
}

#[test]
fn runaway_label_keeps_newlines_past_cap() {
    // A label past the 4 KB cap stops folding, so a broken page cannot merge
    // the rest of the document onto one line.
    let mut body = String::new();
    for i in 0..2000 {
        body.push_str(&format!("<p>para {i}</p>"));
    }
    let out = md(&format!("<a href=\"https://a.example/\">{body}</a>"));
    assert_no_markers(&out);
    assert!(out.lines().count() > 100, "lines: {}", out.lines().count());
}

#[test]
fn idempotent_on_its_own_output() {
    let html = "<ol><li><a href=\"/doc/\">\n  Documentation\n</a></li></ol>";
    let once = md_url(html, "https://go.dev/");
    let twice = html2md::clean_markdown(&once);
    assert_eq!(once, twice);
}

#[tokio::test]
async fn sync_send_and_chunked_agree() {
    let html = "<ol><li><a href=\"/doc/\">\n  Documentation\n</a></li>\
                <li><a href=\"/x\">\n<img src=\"/i.png\" alt=\"i\">\n  X\n</a></li></ol>\
                <p>tail <a href=\"/y\">\n y \n</a></p>";
    let url = Some(Url::parse("https://go.dev/").unwrap());
    let a = rewrite_html_custom_with_url(html, &None, false, &url);
    let b = rewrite_html_custom_with_url_streaming(html, &None, false, &url).await;
    let c = rewrite_html_custom_with_url_and_chunk(html, &None, false, &url, 7).await;
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_no_markers(&a);
}

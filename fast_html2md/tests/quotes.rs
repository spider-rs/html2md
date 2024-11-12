use html2md::parse_html;
use indoc::indoc;
use pretty_assertions::assert_eq;

#[test]
fn test_quotes() {
    let md = parse_html(
        "<p><blockquote>here's a quote\n next line of it</blockquote>And some text after it</p>",
        false,
    );
    assert_eq!(
        md,
        "\n\n> here's a quote next line of it\nAnd some text after it"
    )
}

#[test]
fn test_quotes2() {
    let md = parse_html("<p><blockquote>here's<blockquote>nested quote!</blockquote> a quote\n next line of it</blockquote></p>", false);
    assert_eq!(
        md,
        "\n\n> here's\n> > nested quote!\n> a quote next line of it\n\n"
    )
}

#[test]
fn test_blockquotes() {
    let md = parse_html(
        "<blockquote>Quote at the start of the message</blockquote>Should not crash the parser",
        false,
    );
    assert_eq!(
        md,
        "> Quote at the start of the message\nShould not crash the parser"
    )
}

#[test]
fn test_details() {
    let html = indoc! {"
    <details>
        <summary>There are more things in heaven and Earth, <b>Horatio</b></summary>
        <p>Than are dreamt of in your philosophy</p>
    </details>
    "};
    let md = parse_html(html, false);
    assert_eq!(md, "There are more things in heaven and Earth,**Horatio**\nThan are dreamt of in your philosophy")
}

#[test]
fn test_subsup() {
    let md = parse_html("X<sub>2</sub>", false);
    assert_eq!(md, r#"X2"#);
    let md = parse_html("X<sub>2</sub>", true);
    assert_eq!(md, r#"X<sub>2</sub>"#)
}

#[cfg(feature = "scraper")]
pub mod test {
    use html2md::{parse_html, rewrite_html};
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_quotes() {
        let s =
            "<p><blockquote>here's a quote next line of it</blockquote>And some text after it</p>";
        let m = "> here's a quote next line of it\nAnd some text after it";
        let md = parse_html(s, false);
        assert_eq!(md, m);
        let md = rewrite_html(s, false);
        assert_eq!(md, m);
    }

    #[test]
    fn test_quotes2() {
        let s = "<p><blockquote>here's<blockquote>nested quote!</blockquote> a quote\n next line of it</blockquote></p>";
        let m = "> here's\n> > nested quote!\n> a quote next line of it";
        let md = parse_html(s, false);
        assert_eq!(md, m);
        let md = rewrite_html(s, false);
        assert_eq!(md, m)
    }

    #[test]
    fn test_blockquotes() {
        let s =
            "<blockquote>Quote at the start of the message</blockquote>Should not crash the parser";
        let m = "> Quote at the start of the message\nShould not crash the parser";

        let md = parse_html(s, false);
        assert_eq!(md, m);

        let md = rewrite_html(s, false);
        assert_eq!(md, m);
    }

    #[test]
    fn test_details() {
        let html = indoc! {"
    <details>
        <summary>There are more things in heaven and Earth, <b>Horatio</b></summary>
        <p>Than are dreamt of in your philosophy</p>
    </details>
    "};
        let m = "There are more things in heaven and Earth,**Horatio**\nThan are dreamt of in your philosophy";
        let md = parse_html(html, false);
        assert_eq!(md, m);
        let md = rewrite_html(html, false);
        assert_eq!(md, m)
    }

    #[test]
    fn test_subsup() {
        let md = parse_html("X<sub>2</sub>", false);
        assert_eq!(md, r#"X2"#);
        let md = parse_html("X<sub>2</sub>", true);
        assert_eq!(md, r#"X<sub>2</sub>"#);

        let md = rewrite_html("X<sub>2</sub>", false);
        assert_eq!(md, r#"X2"#);
        let md = rewrite_html("X<sub>2</sub>", true);
        assert_eq!(md, r#"X<sub>2</sub>"#);
    }
}

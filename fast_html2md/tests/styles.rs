use html2md::{parse_html, rewrite_html};
use pretty_assertions::assert_eq;

#[test]
fn test_styles_with_spaces() {
    let s = r#"It read:<s> Nobody will ever love you</s>"#;

    let md = parse_html(s, false);
    assert_eq!(md, r#"It read:~~Nobody will ever love you~~"#);
    let md = rewrite_html(s, false);
    assert_eq!(md, r#"It read:~~Nobody will ever love you~~"#);
}

#[test]
fn test_styles_with_newlines() {
    let s = r#"
And she said:<br/>
<s>We are all just prisoners here<br/>
<u> Of our own device<br/>  </s>
And in the master's chambers<br/>
They gathered for the feast<br/>
<em>They stab it with their steely knives</em><br/>
<strong>But they just can't kill the beast<br/></strong>"#;

    let m = "And she said:\n~~We are all just prisoners here\nOf our own device~~\nAnd in the master's chambers\nThey gathered for the feast\n*They stab it with their steely knives*\n**But they just can't kill the beast**";

    let md = parse_html(s, false);

    assert_eq!(md, m);

    // let md = rewrite_html(s, false);

    // assert_eq!(md, m);
}

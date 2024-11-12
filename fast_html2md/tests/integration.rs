extern crate spectral;

// use html2md::ignore::IgnoreTagFactory;
// use html2md::{parse_html, parse_html_custom, parse_html_custom_with_url};
use html2md::parse_html;
use indoc::indoc;
use spectral::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use url::Url;

#[test]
#[ignore]
fn test_marcfs() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/marcfs-readme.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert!(!result.is_empty());
}

#[test]
fn test_real_world_wiki() -> Result<(), Box<dyn std::error::Error>> {
    use std::error::Error;
    use std::fs::{self, File};
    use std::io::{self, Read};
    use std::path::Path;

    let paths = fs::read_dir("../test-samples/wiki")?;

    fn run_parse(path: &Path) -> Result<(), Box<dyn Error>> {
        let mut html = String::new();
        let mut html_file = File::open(path)?;
        html_file.read_to_string(&mut html)?;

        let result = parse_html(&html, false);

        if result.is_empty() {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Result is empty",
            )))
        } else {
            Ok(())
        }
    }

    for entry in paths {
        let path = entry?.path();

        if path.is_file() {
            match run_parse(&path) {
                Ok(_) => assert!(true),
                Err(_e) => assert!(false),
            }
        }
    }

    Ok(())
}

#[test]
#[ignore]
fn test_real_world_ja() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/real-world-ja-1.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert!(!result.is_empty());
}

#[test]
#[ignore]
fn test_cheatsheet() {
    let mut html = String::new();
    let mut md = String::new();
    let mut html_file = File::open("../test-samples/markdown-cheatsheet.html").unwrap();
    let mut md_file = File::open("../test-samples/markdown-cheatsheet.md").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    md_file
        .read_to_string(&mut md)
        .expect("File must be readable");
    let md_parsed = parse_html(&html, false);
    assert!(!md_parsed.is_empty());
}

/// newlines after list shouldn't be converted into text of the last list element
#[test]
fn test_list_newlines() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-list-newlines.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert_that(&result).contains(".\n\nxxx xxxx");
    assert_that(&result).contains("xx x.\n\nxxxxx:");
}

#[test]
fn test_lists_from_text() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-lists-from-text.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert_that(&result).contains("\\- x xxxx xxxxx xx xxxxxxxxxx");
    assert_that(&result).contains("\\- x xxxx xxxxxxxx xxxxxxxxx xxxxxx xxx x xxxxxxxx xxxx");
    assert_that(&result).contains("\\- xxxx xxxxxxxx");
}

#[test]
fn test_strong_inside_link() {
    let mut html = String::new();
    let mut html_file =
        File::open("../test-samples/dybr-bug-with-strong-inside-link.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);
    assert_that(&result).contains("[**Just God**](http://fanfics.me/ficXXXXXXX)");
}

#[test]
fn test_tables_with_newlines() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-tables-masked.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let result = parse_html(&html, false);

    assert_that!(result).contains(indoc! {"[![Embedded YouTube video](https://img.youtube.com/vi/ZZZZZZZZZ/0.jpg)](https://www.youtube.com/watch?v=ZZZZZZZZZ)\n\n|Maybe I'm foolish, maybe I'm blind\nThinking I can see through this and see what's behind\nGot no way to prove it so maybe I'm blind\n\nBut I'm only human after all,\nI'm only human after all\nDon't put your blame on me|xxxxx xxxx, x xxxxxx, xxxxx xxxx — xxxxxx\nxxx xxxxx, xxx xxxx xxxxxx xxxxxx xxx, x xxxxxx xxx xxx xx xxx\nxxxx x xxxx xx xxxx xxxxxxx xxxxxxxxxxxxx, xxx xxx xxxxxxxx, x xxxxxx.\n\nxx x xxxxx xxxx xxxxxxx, x xxxxx-xx xxxxxx,\nx xxxxx xxxx xxxxxxx, x xxxxx xxxxxx.\nxx xxxx xxxx|\n|||\n\n[xxxxxx xxxxx xxxxx x xxxxxxx](/)\n\nx xxxx xxxxxxxxx xxxxxxx xxxxxxxxxxx xx xxxx xxxxx. x xxxxx xxxxxxx, xxxx xxxxx xxxxxxx xx xxxxxxxxxx xxxxxx. xxx xxxxxxxx, xxx xxxxxxxxx xxxxxxxxxxxxxx xx xxxxx — xxxxxxxxxx xxxxxxxxxx x xxxxx xxxxxxxxxxxxx xxxxxxxxx. x xxx xxxxxxxxxxxx*xxxx*, xxxxxx xxxx, xxxxxxxxxx xxxxx xxxxxxxx, xxxxxxxxxx x xxxxxxxxx. xx xxxxxx xxxxx xxxxxxxxxxxxxxxxx — x xxxxxx xxx xxxx.\n\nxxxxx xxxxxxxxxx xxxxx x xxxx xxxxxxxxxx xxxxx. xxxxx. x xxxxx: «x xxxxxx xxxxxxx, x xxxxx xxx xxxx, xx xxxxxxxx xxxxxx», — xxx xxxxx xxxxxxxx. xxxxxx xxx x xxxx xxxx xxxxxxxx xxxxxxxx xxxxxxx xxxx xxxxxxxxxxx xxxxxxxxxx, xxxxxxx xxxxxx xxxxxx xxx xxxxx, xxxxxxxxxxx x x xxxxxxx xxxxxxxxx.\n\nxx x xxxxx xxxx xxxxxxx. xxxxxx xxxxx? xxxxxxxxxxx x xxxxxxxxx xxxxxx.\n\nx xxxxx x xxxxxxxxxx x xxxxx... x xxxxxx xxxx xxxxxx xxxxxxx xxxxxxxx. xx xxxx, x xxxxxx xxx-xx xxxxxxxxx xx xxxxxxx, xxx xxxxxx xxxxxx, xxx xxx xxxxx, xxxxx xxxxxxxx xx xxxx... x xxxxxx xxxxxxx xx xxxx xxxxx, xxx, xxxxx xxxx xxxxxxxxxx, x xxxxx xxxxxxxxx xx xxxxx. x xxx-xx xxx xxxxx xxxxxxx xxxxxxxxxxxxx.\n\nxxxxxx xx... xx xxx xx xxxxxxxxxxxxx xxxxxx xxxxxxxxxxxxx x xxxxxxxxxx xxxxx, xxxxx xxx xxxx xxxxxxxxx, x xxxxx xxx xxxxxxxxx, xxx xxxxxxx xxx, xxx xxxx xxxxxxx xxxxxx, x xx xxx, xxx xxxx xxxxxxxx."
});
}

#[test]
fn test_tables_crash2() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/dybr-bug-with-tables-2-masked.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");
    let table_with_vertical_header = parse_html(&html, false);

    assert_that!(table_with_vertical_header).contains(indoc! {"xxxxx xxxxxxxxxx xxxxxxx x xxxxx))~~xxxxxxxx xxxxxxxx~~\n\n## At a Glance\n\n|Current Conditions:|Open all year. No reservations. No services.|\n|||\n| Reservations: | No reservations. |\n| Fees | No fee. |\n| Water: | No water. |"
    });
}

#[test]
fn test_html_from_text() {
    let mut html = String::new();
    let mut html_file = File::open("../test-samples/real-world-1.html").unwrap();
    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");

    let mut tag_factory: HashMap<String, Box<dyn html2md::TagHandlerFactory>> = HashMap::new();
    let tag = Box::new(html2md::scraper::ignore::IgnoreTagFactory {});

    tag_factory.insert(String::from("script"), tag.clone());
    tag_factory.insert(String::from("style"), tag.clone());
    tag_factory.insert(String::from("noscript"), tag.clone());

    tag_factory.insert(String::from("iframe"), tag);

    let result = html2md::parse_html_custom_with_url(
        &html,
        &tag_factory,
        false,
        &Some(Url::parse("https://spider.cloud").unwrap()),
    );
    assert!(!result.is_empty());
}

#[test]
fn test_html_from_text_rewrite() {
    let mut html = Box::new(String::new());
    let mut html_file = File::open("../test-samples/real-world-1.html").unwrap();

    html_file
        .read_to_string(&mut html)
        .expect("File must be readable");

    let result = html2md::rewrite_html(
        &html,
        // &tag_factory,
        // false,
        // &Some(Url::parse("https://spider.cloud").unwrap()),
    );

    assert!(!result.is_empty());
}

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
    assert_that(&result).is_equal_to("xx, xx xxxxx x xxxxxx xxxxxxxx xxxxx xxxxxxxxx xxxx xx xxxx xxxx xxxxxxxx.\nxxxx, xxx xx xxxxx xx xxxxxxxxxxx xxxx.\nxxxxxxxxxxx:\n* xxxxxxx x xxxxxxxxx (xxxxx)\n* xxxxxxx xx xxxxxx xxxxxxx, xxxxxxxxxx xxxxxxxxxx xxxx\n* xxxxxxxxx xx xxxxx, xx xxxxxx xx xxxxxxxxxxx\n* xxxxxxx xxxxxx xxxxxxxxx x xxxxxxxxxx, xxxxxxx xxxxxx x xxxxxxx, x xxxxxx.\n* xx xx, xxxxxx xx xxxxxxxx, xx-xxxx xxx x xxxxxxx xxx xxx, xxxxxxx xx xxxx. xxxxxxxxx xx x.\nxxxxx:\n1. xxxxxxxxx xxxxxxxxxx - xxxxx -\\_- !\n2. xxxxxx Mother of Learning - xxxx, xxxxxxx, xxxxxxxxxxxx\n3. xxxxxx xxxxxxx xxxxxxx, xxxxxxxx \"xxx xxxxx\". xxxxx xxxxx xxxx, xx x xxxxx xxxxxxx.\n4. xxxxxxxx! xxxx xxx xxxxxxxxx xxxx xxx, xx x xxxxxxxxx.\n5. xxxx xxxxxx - xxxxxx xxxxxxxx xxx x 15-17, xxxxxx xxxxxxxxxxxxx xx xxxxxxxx xxx xxxxxxx xxxxxx.\nxxx xxxx, xxxxx x xxxxxxxxx xx xxxxxxxxxx xxxxxx. xxxxxxxxx spelling puns, xxxxxxx, x xxxxxxxxx, xxxxxxxx xxx xxxxxxxx, xxxxxx xxxxxxxxxx xxxxxx.\nxxx xxxxxxx. xxx xxx xxxxxxxx xxxxxx - x x xxxxxxxxxxx xxxxx xxxx xxxxxxxxxx xxx xxxxx, x xxxxxx xxx xxxxxxxx xxxxxxxxxx xxx xxxxx. xx xxxxxx xxxxxxxx:\n* xxx xxxxx x xxx-xxxx xxxxxxxxx. xxxxxx xxx xxxx xxxxxxxx. x xx x xx xxxxxxxx, xx x xxxxxxx xxxxxx xxxxxx xx xxxxxxxxx. xxxxxxxxxx xxxx xxxxx xxxxxx xxxxxxxxx xxxxxxx xx xxxx.\n* xxxxxx xxxx Kotlin, x xxxxxxx. xxxxxxxxxx, xxxxxxxxxx xxx xxxxx xx xxx x xxxxxxxx\n* xxx xxxxx xxxxxxxxxx Rust, xxx xxx x xx xxx xxxx xxxxxxxxx xxxxxxxxxxxxxx xxxx xxx xxxxx, xxxxxxxx xxxxxxxxxxxxxx HTML x Markdown\n* xxx xxxx xxxxxx xxx xxxxxxxx xxxxxx. xx xxxx xxx - xxxxxxxxxxxxx xxxxxxxxxxx xxxxxx x xxxxxxxxx xxxxx x xxxxxxx.\n* xxxxxxxxx xxxx xxxxxxxx xxxxxxx xx FUSE 3.0. xxxxx xxxxxxx xxxxxxx xxx xxxxxxxxxxx.\n* x xxxxxxxx xxxx xxxxxxxx DevOps-xxxxxxx x xxxxx xxxxxxx. xxxxxxxxx, xxx xx xxxxx xxxxxx. x, xx, xxx xxx xxx xxxxxxxxx?\nxxxxx xx xxx:\n\\- xxxxxxxx xxxxxxxx\n\\- xxxxxxx xxxxxxxxx, xxxxxxx xxxxx xxxxx xxxxxxxx\n\\- xxxxxxxxxx xxxx Machine Learning, xxxx xxxxxx xxx xxxxxxxx OpenCL.".to_string());
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

    assert_that!(result).contains(indoc! {"[![Embedded YouTube video](https://img.youtube.com/vi/ZZZZZZZZZ/0.jpg)](https://www.youtube.com/watch?v=ZZZZZZZZZ)\n|Maybe I'm foolish, maybe I'm blind\nThinking I can see through this and see what's behind\nGot no way to prove it so maybe I'm blind\nBut I'm only human after all,\nI'm only human after all\nDon't put your blame on me|xxxxx xxxx, x xxxxxx, xxxxx xxxx —xxxxxx\nxxx xxxxx, xxx xxxx xxxxxx xxxxxx xxx, x xxxxxx xxx xxx xx xxx\nxxxx x xxxx xx xxxx xxxxxxx xxxxxxxxxxxxx, xxx xxx xxxxxxxx, x xxxxxx.\nxx x xxxxx xxxx xxxxxxx, x xxxxx-xx xxxxxx,\nx xxxxx xxxx xxxxxxx, x xxxxx xxxxxx.\nxx xxxx xxxx|\n|||\n[xxxxxx xxxxx xxxxx x xxxxxxx](/)\nx xxxx xxxxxxxxx xxxxxxx xxxxxxxxxxx xx xxxx xxxxx. x xxxxx xxxxxxx, xxxx xxxxx xxxxxxx xx xxxxxxxxxx xxxxxx. xxx xxxxxxxx, xxx xxxxxxxxx xxxxxxxxxxxxxx xx xxxxx —xxxxxxxxxx xxxxxxxxxx x xxxxx xxxxxxxxxxxxx xxxxxxxxx. x xxx xxxxxxxxxxxx*xxxx*, xxxxxx xxxx, xxxxxxxxxx xxxxx xxxxxxxx, xxxxxxxxxx x xxxxxxxxx. xx xxxxxx xxxxx xxxxxxxxxxxxxxxxx —x xxxxxx xxx xxxx.\nxxxxx xxxxxxxxxx xxxxx x xxxx xxxxxxxxxx xxxxx. xxxxx. x xxxxx: «x xxxxxx xxxxxxx, x xxxxx xxx xxxx, xx xxxxxxxx xxxxxx», —xxx xxxxx xxxxxxxx. xxxxxx xxx x xxxx xxxx xxxxxxxx xxxxxxxx xxxxxxx xxxx xxxxxxxxxxx xxxxxxxxxx, xxxxxxx xxxxxx xxxxxx xxx xxxxx, xxxxxxxxxxx x x xxxxxxx xxxxxxxxx.\nxx x xxxxx xxxx xxxxxxx. xxxxxx xxxxx? xxxxxxxxxxx x xxxxxxxxx xxxxxx.\nx xxxxx x xxxxxxxxxx x xxxxx... x xxxxxx xxxx xxxxxx xxxxxxx xxxxxxxx. xx xxxx, x xxxxxx xxx-xx xxxxxxxxx xx xxxxxxx, xxx xxxxxx xxxxxx, xxx xxx xxxxx, xxxxx xxxxxxxx xx xxxx... x xxxxxx xxxxxxx xx xxxx xxxxx, xxx, xxxxx xxxx xxxxxxxxxx, x xxxxx xxxxxxxxx xx xxxxx. x xxx-xx xxx xxxxx xxxxxxx xxxxxxxxxxxxx.\nxxxxxx xx... xx xxx xx xxxxxxxxxxxxx xxxxxx xxxxxxxxxxxxx x xxxxxxxxxx xxxxx, xxxxx xxx xxxx xxxxxxxxx, x xxxxx xxx xxxxxxxxx, xxx xxxxxxx xxx, xxx xxxx xxxxxxx xxxxxx, x xx xxx, xxx xxxx xxxxxxxx."
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

    assert_that!(table_with_vertical_header).contains(indoc! {"xxxxx xxxxxxxxxx xxxxxxx x xxxxx))~~xxxxxxxx xxxxxxxx~~\n## At a Glance\n|Current Conditions:|Open all year. No reservations. No services.|\n|||\n| Reservations: | No reservations. |\n| Fees | No fee. |\n| Water: | No water. |"
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

    let result = html2md::rewrite_html_with_url(
        &html,
        false,
        &Some(Url::parse("https://spider.cloud").unwrap()),
    );

    println!("{:?}", result);
    assert!(!result.is_empty());
}

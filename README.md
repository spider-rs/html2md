# fast_html2md

A Rust html to markdown crate built for performance.

`cargo add fast_html2md`

```rust
use html2md::parse_html;

let md = parse_html("<p>JAMES</p>", false);
assert_eq!(md, "JAMES")
```

## Ignoring Tags

```rust
    let mut tag_factory: HashMap<String, Box<dyn html2md::TagHandlerFactory>> =
        HashMap::new();

    let tag = Box::new(IgnoreTagFactory {});

    tag_factory.insert(String::from("script"), tag.clone());
    tag_factory.insert(String::from("style"), tag.clone());
    tag_factory.insert(String::from("noscript"), tag.clone());
    let html = html2md::parse_html_custom(&html, &tag_factory, false);
```

## Notes

This project is a practical rewrite from the original `html2md` with major bug fixes and performance improvements.

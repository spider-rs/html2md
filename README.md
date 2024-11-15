# fast_html2md

The fastest Rust html to markdown transformer.

`cargo add fast_html2md`

You can use a scraper or rewriter to transform. The rewriter is over 2-3 times faster.

```rust
use html2md::parse_html;

let md = parse_html("<p>JAMES</p>", false);
assert_eq!(md, "JAMES")
```

Using a rewriter.

```rust
use html2md::rewrite_html;

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

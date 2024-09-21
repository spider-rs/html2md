# fast_html2md

A Rust html to markdown crate built for performance.

`cargo add fast_html2md`

```rust
use html2md::parse_html;

let md = parse_html("<p>JAMES</p>", false);
assert_eq!(md, "JAMES")
```

## Notes

This project is a practical rewrite from the original `html2md` with major bug fixes and performance improvements.
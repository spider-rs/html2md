# fast_html2md

The fastest Rust html to markdown transformer.

`cargo add fast_html2md`

You can use a [html5ever](https://docs.rs/html5ever/latest/html5ever/) or [lol_html](https://docs.rs/lol_html/latest/lol_html/) to transform.

Using the rewriter with the default `rewriter` feature flag (recommended and 2x faster baseline).

```rust
let md = html2md::rewrite_html("<p>JAMES</p>", false);
assert_eq!(md, "JAMES")
```

Using the scraper with the `scraper` feature flag.

```rust
let md = html2md::parse_html("<p>JAMES</p>", false);
assert_eq!(md, "JAMES")
```

## License

MIT
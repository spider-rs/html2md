# fast_html2md

The fastest Rust library for transforming HTML into Markdown. Designed for performance and ease-of-use in Rust projects.

## Installation

Add `fast_html2md` to your `Cargo.toml`:

```sh
cargo add fast_html2md
```

## Usage

Below are examples to get started quickly. The library provides several methods depending on your needs.

### Using the Rewriter (Default)

With the default `rewriter` feature, recommended for high performance:

```rust
let md = html2md::rewrite_html("<p>JAMES</p>", false);
assert_eq!(md, "JAMES");
```

### With Async Streaming

For handling large or concurrent workloads, use async streaming. Ensure you have a tokio async runtime:

```rust
let md = html2md::rewrite_html_streaming("<p>JAMES</p>", false).await;
assert_eq!(md, "JAMES");
```

### Using the Scraper

For a different approach, enable the `scraper` feature:

```rust
let md = html2md::parse_html("<p>JAMES</p>", false);
assert_eq!(md, "JAMES");
```

## Features

- **Rewriter:** High performance transformation using the `rewriter` feature (default).
- **Scraper:** Alternative approach for HTML parsing with the `scraper` feature.

### About

The features are split to help you choose the library you need. If your project heavily depends on [`scraper`](https://docs.rs/html5ever/latest/html5ever/) and you need to keep the binary small, you can enable just that feature flag. The same applies to the `rewriter` feature using [`lol_html`](https://docs.rs/lol_html/latest/lol_html/). This project is actively used in production at [Spider](https://spider.cloud).

## License

This project is licensed under the MIT License.
//! The converter returns on every input, never panics, and does work linear
//! in the size of its input.
//!
//! The sift and the rewriter are state machines that carry state from one
//! piece of input to the next: an open code block, an open link label, a cut
//! entity, a cut multibyte character. These tests end the input inside each
//! of those states, feed hostile and random markup a byte at a time, and time
//! inputs of growing size. A hang fails the run by its time limit, a panic
//! fails the test, and work that grows faster than the input fails the ratio
//! check.
#![cfg(all(feature = "rewriter", feature = "stream"))]

use html2md::{
    rewrite_html, rewrite_html_custom_with_url_and_chunk, rewrite_html_custom_with_url_streaming,
    rewrite_html_stream,
};
use std::time::{Duration, Instant};

/// xorshift64, so the random inputs are the same on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn no_markers(out: &str, html: &str, what: &str) {
    assert!(
        !out.bytes().any(|b| matches!(b, 0x02 | 0x03 | 0x0E | 0x0F)),
        "{what}: marker byte in {out:?} for {html:?}"
    );
}

/// Run `html` through the sync, async and streaming entry points, and the
/// streaming one with the input cut into pieces of `piece` bytes.
async fn convert_all(html: &str, piece: usize, what: &str) -> Vec<String> {
    let mut outs = vec![
        rewrite_html(html, false),
        rewrite_html(html, true),
        rewrite_html_custom_with_url_streaming(html, &None, false, &None).await,
        rewrite_html_custom_with_url_and_chunk(html, &None, true, &None, piece).await,
    ];
    let pieces: Vec<Result<Vec<u8>, std::io::Error>> = html
        .as_bytes()
        .chunks(piece.max(1))
        .map(|c| Ok(c.to_vec()))
        .collect();
    outs.push(
        rewrite_html_stream(futures_util::stream::iter(pieces), false)
            .await
            .unwrap_or_default(),
    );
    for (at, out) in outs.iter().enumerate() {
        no_markers(out, html, &format!("{what}, output {at}"));
    }
    outs
}

/// Input that ends inside each state the converter carries between pieces.
#[tokio::test]
async fn input_that_ends_inside_a_state_returns() {
    let cases: &[(&str, &str)] = &[
        ("inside a fence", "<p>x</p><pre><code>  a = 1\n  b"),
        ("inside an entity in code", "<pre>a &am"),
        ("inside a numeric entity in code", "<pre>a &#x3"),
        ("inside an entity in text", "<p>a &nbs"),
        (
            "inside a link label",
            "<p><a href=\"/x\">open label\n  more",
        ),
        ("inside a tag", "<pre>a</pre><p cla"),
        ("inside a table header", "<table><tr><th>a</th><th>b"),
        ("inside a multibyte character", "<pre>\u{e9}\u{65e5}"),
        ("inside inline code", "<p><code>a_b"),
    ];
    for (what, html) in cases {
        for piece in [1usize, 2, 3, 7] {
            let outs = convert_all(html, piece, what).await;
            assert!(
                !outs[0].is_empty() || html.is_empty(),
                "{what}: empty output"
            );
        }
    }
    // The multibyte case loses no byte when it is cut.
    let mut html = "<pre>".to_string();
    html.push_str("\u{e9}\u{65e5}");
    for piece in 1..=6 {
        let out = rewrite_html_custom_with_url_and_chunk(&html, &None, false, &None, piece).await;
        assert_eq!(out, "```\n\u{e9}\u{65e5}\n```", "piece {piece}");
    }
}

/// A megabyte of code in a `<pre>` that never closes.
#[tokio::test]
async fn megabyte_of_unclosed_pre_returns() {
    let mut html = String::from("<p>start</p><pre>");
    let line = "    let a_b = c[i] * 2; // `x`\n";
    while html.len() < 1 << 20 {
        html.push_str(line);
    }
    let started = Instant::now();
    let outs = convert_all(&html, 4096, "unclosed pre").await;
    for out in &outs {
        assert!(
            out.starts_with("start\n```\n    let a_b"),
            "no opening fence"
        );
        assert!(out.ends_with("// `x`\n```"), "no closing fence");
    }
    assert!(
        started.elapsed() < Duration::from_secs(120),
        "took {:?}",
        started.elapsed()
    );
}

/// Marker bytes and tags in every unbalanced order.
#[tokio::test]
async fn unbalanced_markers_and_tags_return() {
    let parts = [
        "\u{e}",
        "\u{f}",
        "\u{2}",
        "\u{3}",
        "<pre>",
        "</pre>",
        "<code>",
        "</code>",
        "<a href=\"/u\">",
        "</a>",
        "<table>",
        "<tr>",
        "<th>",
        "</th>",
        "<td>",
        "</tr>",
        "</table>",
        "&amp;",
        "&",
        "`",
        "```",
        "\n",
        " ",
        "x_y",
    ];
    let mut rng = Rng(0x5eed_0001);
    for round in 0..300 {
        let mut html = String::new();
        for _ in 0..rng.below(60) {
            html.push_str(parts[rng.below(parts.len())]);
        }
        convert_all(&html, 1 + rng.below(9), &format!("round {round}")).await;
    }
}

/// Random tag soup: a fixed seed and a fixed number of rounds.
#[tokio::test]
async fn random_tag_soup_returns_and_never_panics() {
    let tags = [
        "pre",
        "code",
        "samp",
        "a",
        "p",
        "div",
        "span",
        "table",
        "tr",
        "th",
        "td",
        "ul",
        "ol",
        "li",
        "blockquote",
        "br",
        "b",
        "i",
        "img",
        "h1",
        "h2",
        "script",
        "style",
        "title",
        "head",
    ];
    let texts = [
        "x",
        "snake_case",
        "  indented",
        "\n",
        "\r\n",
        "\t",
        "```",
        "`",
        "&lt;",
        "&amp",
        "&#",
        "&#x1F600;",
        "&nbsp;",
        "\u{a0}",
        "\u{e9}",
        "\u{65e5}",
        "*",
        "[",
        "]",
        "|",
        "\u{e}",
        "\u{f}",
    ];
    let mut rng = Rng(0x00c0_ffee_1234_5678);
    for round in 0..1_500 {
        let mut html = String::new();
        for _ in 0..rng.below(80) {
            match rng.below(4) {
                0 => {
                    let tag = tags[rng.below(tags.len())];
                    html.push('<');
                    html.push_str(tag);
                    if rng.below(3) == 0 {
                        html.push_str(" colspan=\"2\" href=\"/h\"");
                    }
                    html.push('>');
                }
                1 => {
                    html.push_str("</");
                    html.push_str(tags[rng.below(tags.len())]);
                    html.push('>');
                }
                _ => html.push_str(texts[rng.below(texts.len())]),
            }
        }
        convert_all(&html, 1 + rng.below(16), &format!("round {round}")).await;
    }
}

/// A page with every construct the patch touches, repeated `times` times.
fn page(times: usize) -> String {
    let unit = "<p>Intro_text with <code>inline_code</code> and <a href=\"/u\">a link</a>.</p>\
<pre><code><span class=\"k\">def</span> <span class=\"n\">f</span>():\n    <span class=\"k\">return</span> `a` ```b```\n\n</code></pre>\
<table><tr><th>k</th><th>v</th></tr><tr><td>a&nbsp;b</td><td>&lt;c&gt;</td></tr></table>\
<p><a href=\"/open\">left open</p><ul><li><pre>x\ty</pre></li></ul>\n";
    unit.repeat(times)
}

/// Best of three, so a scheduler hiccup does not decide the ratio.
async fn time_once(html: &str) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let started = Instant::now();
        let out = rewrite_html_custom_with_url_streaming(html, &None, false, &None).await;
        let took = started.elapsed();
        assert!(!out.is_empty());
        best = best.min(took);
    }
    best
}

/// The 4x to 1x time ratio for `make`, as the best of five tries. Linear work
/// gives a ratio near 4 and quadratic work gives 16. A busy machine can stall
/// one try, so the best try decides: quadratic work never gets a good one.
async fn best_ratio(make: impl Fn(usize) -> String, one_x: usize) -> (f64, String) {
    let (small, large) = (make(one_x), make(one_x * 4));
    let mut best = f64::MAX;
    let mut seen = String::new();
    for _ in 0..5 {
        let one = time_once(&small).await;
        let four = time_once(&large).await;
        let ratio = four.as_secs_f64() / one.as_secs_f64().max(1e-9);
        seen.push_str(&format!(" [1x {one:?}, 4x {four:?}: {ratio:.2}]"));
        best = best.min(ratio);
        if best < 9.0 {
            break;
        }
    }
    (best, seen)
}

/// Work grows with the input, not faster. The bound is generous.
#[tokio::test]
async fn work_is_linear_in_input_size() {
    let (ratio, seen) = best_ratio(page, 400).await;
    assert!(ratio < 9.0, "ratio {ratio:.2}:{seen}");
}

/// The same check on the worst case for fence growth: one block with a long
/// backtick run near its end, at 1x, 2x and 4x the size.
#[tokio::test]
async fn fence_growth_is_linear_in_block_size() {
    fn block(lines: usize) -> String {
        let mut html = String::from("<pre>");
        for _ in 0..lines {
            html.push_str("  a_b = c\n");
        }
        html.push_str("``````</pre>");
        html
    }
    let (ratio, seen) = best_ratio(block, 20_000).await;
    assert!(ratio < 9.0, "ratio {ratio:.2}:{seen}");
}

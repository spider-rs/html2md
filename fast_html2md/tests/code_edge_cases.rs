//! Edge cases for code blocks, inline code and table header rows.
//!
//! Each case runs through every entry point and every small piece size, so a
//! rule that holds only when the whole page arrives in one piece fails here.
#![cfg(all(feature = "rewriter", feature = "stream"))]

use html2md::{
    rewrite_html, rewrite_html_custom_with_url, rewrite_html_custom_with_url_and_chunk,
    rewrite_html_custom_with_url_streaming, rewrite_html_stream, rewrite_html_streaming,
};

/// Every entry point, with the input cut into pieces of every small size.
async fn every_path(html: &str, commonmark: bool) -> Vec<(String, String)> {
    let mut out = vec![
        ("sync".to_string(), rewrite_html(html, commonmark)),
        (
            "sync custom".to_string(),
            rewrite_html_custom_with_url(html, &None, commonmark, &None),
        ),
        (
            "streaming".to_string(),
            rewrite_html_streaming(html, commonmark).await,
        ),
        (
            "streaming custom".to_string(),
            rewrite_html_custom_with_url_streaming(html, &None, commonmark, &None).await,
        ),
    ];
    for size in [1usize, 2, 3, 4, 5, 7, 13, 64] {
        out.push((
            format!("chunk {size}"),
            rewrite_html_custom_with_url_and_chunk(html, &None, commonmark, &None, size).await,
        ));
        let pieces: Vec<Result<Vec<u8>, std::io::Error>> = html
            .as_bytes()
            .chunks(size)
            .map(|c| Ok(c.to_vec()))
            .collect();
        out.push((
            format!("stream {size}"),
            rewrite_html_stream(futures_util::stream::iter(pieces), commonmark)
                .await
                .unwrap_or_default(),
        ));
    }
    out
}

/// Every path, both flavours, gives exactly `want`.
async fn all_equal(html: &str, want: &str) {
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert_eq!(got, want, "{path}, commonmark {commonmark}, html {html:?}");
        }
    }
}

/// Every path, both flavours, holds `code` as a fenced block byte for byte.
async fn all_hold_block(html: &str, code: &str) {
    let block = format!("```\n{code}\n```");
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert!(
                got.contains(&block),
                "{path}, commonmark {commonmark}: no {block:?} in {got:?}"
            );
            assert!(
                !got.bytes().any(|b| matches!(b, 0x02 | 0x03 | 0x0E | 0x0F)),
                "{path}: marker byte in {got:?}"
            );
        }
    }
}

#[tokio::test]
async fn pre_inside_a_list_item() {
    let html = "<ul><li><p>Run:</p><pre><code>def f(a_b):\n    return a_b\n</code></pre></li><li>next_item</li></ul>";
    all_hold_block(html, "def f(a_b):\n    return a_b").await;
    for (path, got) in every_path(html, false).await {
        assert!(got.ends_with("next\\_item"), "{path}: {got:?}");
    }
}

#[tokio::test]
async fn pre_inside_nested_lists() {
    let html = "<ol><li>one<ul><li>two<pre>  x = 1\n\n  y = 2</pre></li></ul></li></ol>";
    all_hold_block(html, "  x = 1\n\n  y = 2").await;
}

#[tokio::test]
async fn pre_inside_a_blockquote() {
    let html = "<blockquote><p>Quote_text</p><pre>if a:\n    b()</pre></blockquote><p>after</p>";
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert!(got.contains("if a:\n"), "{path}: {got:?}");
            assert!(got.contains("    b()\n"), "{path}: {got:?}");
            assert!(got.ends_with("after"), "{path}: {got:?}");
            assert!(!got.contains("b\\(\\)"), "{path}: {got:?}");
        }
    }
}

#[tokio::test]
async fn pre_without_code() {
    all_equal("<pre>a_b *c* [d]</pre>", "```\na_b *c* [d]\n```").await;
}

#[tokio::test]
async fn code_without_pre() {
    all_equal(
        "<p>Call <code>get_item(*args)</code> or <code>[x]</code>.</p>",
        "Call `get_item(*args)` or `[x]`.",
    )
    .await;
}

#[tokio::test]
async fn highlighted_spans_per_token() {
    let html = "<pre><code><span class=\"k\">fn</span><span class=\"w\"> </span><span class=\"n\">snake_case</span><span class=\"p\">()</span><span class=\"w\"> </span><span class=\"p\">{}</span></code></pre>";
    all_equal(html, "```\nfn snake_case() {}\n```").await;
}

#[tokio::test]
async fn highlighted_spans_per_line() {
    let html = "<pre><code><span class=\"line\">if x:</span>\n<span class=\"line\">    y_1()</span>\n<span class=\"line\"></span>\n<span class=\"line\">    y_2()</span>\n</code></pre>";
    all_equal(html, "```\nif x:\n    y_1()\n\n    y_2()\n```").await;
}

#[tokio::test]
async fn line_spans_with_the_newline_inside() {
    let html = "<pre><code><span id=\"l1\">a = 1\n</span><span id=\"l2\">\n</span><span id=\"l3\">    b = 2\n</span></code></pre>";
    all_equal(html, "```\na = 1\n\n    b = 2\n```").await;
}

#[tokio::test]
async fn br_inside_pre() {
    all_equal("<pre>a<br>  b<br/><br>c</pre>", "```\na\n  b\n\nc\n```").await;
}

#[tokio::test]
async fn tabs_in_code() {
    all_equal("<pre>\tif x:\n\t\ty\n</pre>", "```\n\tif x:\n\t\ty\n```").await;
}

#[tokio::test]
async fn crlf_in_code() {
    // CRLF stays as the page wrote it inside the block.
    all_hold_block("<pre>a\r\n  b\r\nc</pre>", "a\r\n  b\r\nc").await;
}

#[tokio::test]
async fn trailing_whitespace_in_code() {
    // Spaces at the end of an inner line are kept; whitespace after the last
    // line is dropped.
    all_equal("<pre>a  \n  b \t\n \n\n</pre>", "```\na  \n  b\n```").await;
}

#[tokio::test]
async fn empty_pre() {
    all_equal("<p>x</p><pre></pre><p>y</p>", "x\n```\n```\ny").await;
    all_equal("<p>x</p><pre><code></code></pre><p>y</p>", "x\n```\n```\ny").await;
    all_equal("<p>x</p><pre>\n</pre><p>y</p>", "x\n```\n```\ny").await;
}

#[tokio::test]
async fn backtick_runs_in_code() {
    all_equal(
        "<pre><code>```rust\nlet a = `b`;\n```</code></pre>",
        "````\n```rust\nlet a = `b`;\n```\n````",
    )
    .await;
    all_equal("<p><code>a`b</code></p>", "`a`b`").await;
}

#[tokio::test]
async fn unclosed_pre_at_end_of_input() {
    all_equal("<p>x</p><pre>  a_b\n    c", "x\n```\n  a_b\n    c\n```").await;
    all_equal("<pre>", "```\n```").await;
}

#[tokio::test]
async fn unclosed_code_at_end_of_input() {
    all_equal("<p>see <code>a_b", "see `a_b").await;
    all_equal("<pre><code>  a_b\n  c", "```\n  a_b\n  c\n```").await;
}

#[tokio::test]
async fn marker_bytes_in_text_and_attributes() {
    let html = "<p>a\u{e}b\u{f}c\u{2}d\u{3}e</p><a href=\"https://a.example/\u{e}x\" title=\"\u{f}\">l\u{e}nk</a><pre>p\u{f}q\u{e}r</pre><img alt=\"\u{e}i\u{f}\" src=\"\u{f}.png\"><p>tail_text</p>";
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert!(
                !got.bytes().any(|b| matches!(b, 0x02 | 0x03 | 0x0E | 0x0F)),
                "{path}: marker byte in {got:?}"
            );
            assert!(got.contains("```\npqr\n```"), "{path}: {got:?}");
            assert!(got.ends_with("tail\\_text"), "{path}: {got:?}");
        }
    }
}

#[tokio::test]
async fn entity_split_across_a_chunk_boundary() {
    // Every piece size from 1 to 13 cuts one of these entities somewhere.
    let html = "<pre>&lt;a&gt; &amp;&amp; &#x3C;b&#62; &quot;c&quot; d&nbsp;e</pre>";
    all_equal(html, "```\n<a> && <b> \"c\" d e\n```").await;
}

/// Every path in one flavour gives exactly `want`.
async fn flavour_equal(html: &str, commonmark: bool, want: &str) {
    for (path, got) in every_path(html, commonmark).await {
        assert_eq!(got, want, "{path}, commonmark {commonmark}, html {html:?}");
    }
}

#[tokio::test]
async fn table_with_colspan_is_unchanged() {
    // A header cell that spans columns gets no delimiter row, as before.
    let html = "<table><tr><th colspan=\"2\">title</th></tr><tr><td>1</td><td>2</td></tr></table>";
    flavour_equal(html, false, "|title|\n|1|2|").await;
    flavour_equal(html, true, "|** title**|\n|1|2|").await;
    // A body cell that spans columns leaves the header row as it is.
    let html = "<table><tr><th>a</th><th>b</th></tr><tr><td colspan=\"2\">wide</td></tr></table>";
    flavour_equal(html, false, "|a|b|\n|---|---|\n|wide|").await;
    flavour_equal(html, true, "|** a**|** b**|\n|---|---|\n|wide|").await;
}

#[tokio::test]
async fn entity_split_across_a_chunk_boundary_in_text() {
    // Every piece size cuts some entity here. Each path gives what the whole
    // input gives in one piece.
    let html = "<p>a&nbsp;b &amp;&amp; c&#160;d &lt;e&gt; AT&T f&quot;</p>";
    for commonmark in [false, true] {
        let whole = rewrite_html(html, commonmark);
        assert!(whole.starts_with("a b && c d "), "{whole:?}");
        assert!(
            !whole.contains("&nbsp;") && !whole.contains("&#"),
            "{whole:?}"
        );
        for (path, got) in every_path(html, commonmark).await {
            assert_eq!(got, whole, "{path}, commonmark {commonmark}");
        }
    }
}

#[tokio::test]
async fn nbsp_is_a_space() {
    all_equal(
        "<p>non&nbsp;breaking and&#160;numeric</p>",
        "non breaking and numeric",
    )
    .await;
}

#[tokio::test]
async fn anchor_left_open_does_not_swallow_the_page() {
    // The first anchor is never closed. The second one restarts the label, so
    // the lines after it keep their breaks instead of folding onto one line.
    let html =
        "<p><a href=\"/a\">badge</p><p><a href=\"/b\">docs</a></p><p>line one</p><p>line two</p>";
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert!(got.contains("\nline one\nline two"), "{path}: {got:?}");
        }
    }
}

#[tokio::test]
async fn line_number_anchors_inside_pre_leave_no_trace() {
    // A highlighter that writes an empty self-link at the start of every
    // line. Inside a `<pre>` the anchors are not links, and they are empty,
    // so the block is the code alone.
    let html = "<div class=\"highlight\"><pre><span></span><code><span id=\"s-1\"><a id=\"ln-1\" name=\"ln-1\" href=\"#ln-1\"></a><span class=\"n\">print</span><span class=\"p\">(</span><span class=\"n\">a_b</span><span class=\"p\">)</span>\n</span><span id=\"s-2\"><a id=\"ln-2\" name=\"ln-2\" href=\"#ln-2\"></a>    <span class=\"k\">return</span> <span class=\"mi\">1</span>\n</span></code></pre></div>";
    all_hold_block(html, "print(a_b)\n    return 1").await;
    for (path, got) in every_path(html, false).await {
        assert!(!got.contains("](#"), "{path}: {got:?}");
        assert!(!got.contains("\\_"), "{path}: {got:?}");
    }
}

#[tokio::test]
async fn line_number_anchors_with_numbers_inside_pre() {
    // The anchor holds the line number as text. It is kept as text, and no
    // link syntax lands in the code.
    let html = "<div class=\"highlight\"><pre><code><a href=\"#l1\" id=\"l1\"><span class=\"linenos\">1</span></a>a_b = 1\n<a href=\"#l2\" id=\"l2\"><span class=\"linenos\">2</span></a>    c = a_b\n</code></pre></div>";
    all_hold_block(html, "1a_b = 1\n2    c = a_b").await;
    for (path, got) in every_path(html, false).await {
        assert!(!got.contains("](#"), "{path}: {got:?}");
    }
}

#[tokio::test]
async fn line_number_table_keeps_the_code_byte_for_byte() {
    // A two cell table: one `<pre>` of line numbers, one `<pre>` of code.
    let html = "<table class=\"highlighttable\"><tr><td class=\"linenos\"><div class=\"linenodiv\"><pre><span class=\"normal\">1</span>\n<span class=\"normal\">2</span></pre></div></td><td class=\"code\"><div><pre><span></span><code>x_y = 1\n    return x_y\n</code></pre></div></td></tr></table>";
    all_hold_block(html, "x_y = 1\n    return x_y").await;
    for (path, got) in every_path(html, false).await {
        assert!(!got.contains("\\_"), "{path}: {got:?}");
        assert_eq!(got.matches("```").count(), 4, "{path}: {got:?}");
    }
}

#[tokio::test]
async fn heading_permalinks_are_dropped() {
    // `headerlink` (mkdocs, Sphinx) and `hash-link` (Docusaurus) anchors are
    // only a symbol linking the heading to itself.
    let html = "<h2 id=\"goals\">Goals<a class=\"headerlink\" href=\"#goals\" title=\"Permanent link\">\u{a7}</a></h2><h3 id=\"x\">Install<a class=\"headerlink\" href=\"#x\">&para;</a></h3><h2 id=\"y\">Use<a href=\"#y\" class=\"anchor hash-link\" aria-label=\"Direct link\">\u{200b}</a></h2><p>Text.</p>";
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert!(got.contains("## Goals\n"), "{path}: {got:?}");
            assert!(got.contains("### Install\n"), "{path}: {got:?}");
            assert!(got.contains("## Use\n"), "{path}: {got:?}");
            assert!(!got.contains('\u{a7}'), "{path}: {got:?}");
            assert!(!got.contains('\u{b6}'), "{path}: {got:?}");
            assert!(!got.contains('\u{200b}'), "{path}: {got:?}");
            assert!(!got.contains("](#"), "{path}: {got:?}");
        }
    }
}

#[tokio::test]
async fn permalink_gate_edges() {
    // The class is read only for an anchor whose href holds a fragment or is
    // absent. An absolute permalink and one with no href are still dropped. A
    // permalink class on a link to another page is not a permalink, so the
    // link stays.
    let html = "<h2 id=\"a\">Alpha<a class=\"headerlink\" href=\"https://example.com/doc#a\">\u{a7}</a></h2><h2 id=\"b\">Beta<a class=\"hash-link\">\u{b6}</a></h2><p><a class=\"headerlink\" href=\"https://example.com/other\">other</a> <a href=\"https://example.com/plain\">plain</a></p>";
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert!(got.contains("## Alpha\n"), "{path}: {got:?}");
            assert!(got.contains("## Beta\n"), "{path}: {got:?}");
            assert!(!got.contains('\u{a7}'), "{path}: {got:?}");
            assert!(!got.contains('\u{b6}'), "{path}: {got:?}");
            assert!(
                got.contains("[other](https://example.com/other)"),
                "{path}: {got:?}"
            );
            assert!(
                got.contains("[plain](https://example.com/plain)"),
                "{path}: {got:?}"
            );
        }
    }
}

#[tokio::test]
async fn heading_whose_text_is_a_self_link_keeps_the_text() {
    // Only the permalink class drops an anchor. A heading wrapped in a plain
    // self-link keeps its text and its link.
    let html = "<h2 id=\"z\"><a href=\"#z\">Setup</a></h2><p>See <a class=\"external\" href=\"#z\">setup</a>.</p>";
    for commonmark in [false, true] {
        for (path, got) in every_path(html, commonmark).await {
            assert!(got.contains("[Setup](#z)"), "{path}: {got:?}");
            assert!(got.contains("[setup](#z)"), "{path}: {got:?}");
        }
    }
}

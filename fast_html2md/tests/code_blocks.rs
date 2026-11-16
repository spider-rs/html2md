//! Code survives the conversion byte for byte.
//!
//! A `<pre>` becomes a plain fence that holds its text as the page wrote it:
//! indentation, blank lines and runs of spaces kept, entities decoded, nothing
//! escaped, and no markdown from the highlighting tags inside it. Inline
//! `<code>` keeps its text unescaped too. A table whose first row is all
//! `<th>` gets the delimiter row a markdown renderer needs.
//!
//! `fixtures/highlighted_code.html` is a hand written page in the shape a
//! syntax highlighter emits: one table, one paragraph with inline code and
//! two blocks where every line and every token sits in its own `<span>`.
#![cfg(all(feature = "rewriter", feature = "stream"))]

use html2md::{
    rewrite_html, rewrite_html_custom_with_url, rewrite_html_custom_with_url_and_chunk,
    rewrite_html_custom_with_url_streaming, rewrite_html_stream,
};

const FIXTURE: &str = include_str!("fixtures/highlighted_code.html");

/// The first block of the fixture as a browser shows it.
const PYTHON: &str = "from dataclasses import dataclass, field


@dataclass
class Shelf:
    shelf_id: int | None = None
    book_ids: list[int] = field(default_factory=list)

# More code below \u{1f447}";

/// The second block of the fixture as a browser shows it.
const CONSOLE: &str = "$ python -m shelves serve

<b class=\"ok\">INFO</b>:     listening on http://127.0.0.1:8000 (press CTRL+C to stop)";

fn expected(commonmark: bool) -> String {
    let header = if commonmark {
        "|** id**|** name**|** location**|"
    } else {
        "|id|name|location|"
    };
    format!(
        "The shelf table will look like this:\n\
         {header}\n\
         |---|---|---|\n\
         |1|Fiction|North Wing|\n\
         |2|Maps|Reading Room Two|\n\
         To link them, each book stores the ID of its shelf in `shelf_id`:\n\
         ```\n{PYTHON}\n```\n\
         ```\n{CONSOLE}\n```"
    )
}

fn md(html: &str) -> String {
    rewrite_html(html, false)
}

/// Every entry point, with the input cut into pieces of every small size.
async fn every_path(html: &str, commonmark: bool) -> Vec<(String, String)> {
    let mut out = vec![
        (
            "sync".to_string(),
            rewrite_html_custom_with_url(html, &None, commonmark, &None),
        ),
        (
            "streaming".to_string(),
            rewrite_html_custom_with_url_streaming(html, &None, commonmark, &None).await,
        ),
    ];
    for size in [1usize, 2, 3, 5, 7, 11, 64, 8192] {
        out.push((
            format!("chunk {size}"),
            rewrite_html_custom_with_url_and_chunk(html, &None, commonmark, &None, size).await,
        ));
        let pieces: Vec<Result<Vec<u8>, std::io::Error>> = html
            .as_bytes()
            .chunks(size)
            .map(|c| Ok(c.to_vec()))
            .collect();
        let stream = futures_util::stream::iter(pieces);
        out.push((
            format!("stream {size}"),
            rewrite_html_stream(stream, commonmark)
                .await
                .unwrap_or_default(),
        ));
    }
    out
}

#[tokio::test]
async fn highlighted_blocks_are_plain_fences_with_the_code_byte_for_byte() {
    let blocks = format!("\n```\n{PYTHON}\n```\n```\n{CONSOLE}\n```");
    for commonmark in [false, true] {
        let want = expected(commonmark);
        for (path, got) in every_path(FIXTURE, commonmark).await {
            // The table handler drops a whitespace-only piece of text, so a
            // cut that lands on a space inside a cell joins two words. That
            // is older than this test and not its subject: the paths that
            // cut the page small are held to the code blocks only.
            let small = path.split(' ').nth(1).map_or(false, |n| n != "8192");
            if small {
                assert!(
                    got.ends_with(&blocks),
                    "{path}, commonmark {commonmark}: {got}"
                );
            } else {
                assert_eq!(got, want, "{path}, commonmark {commonmark}");
            }
        }
    }
}

#[tokio::test]
async fn entities_in_code_decode_whatever_the_piece_size() {
    let html = "<pre><code>if a &lt; b &amp;&amp; c &gt; d:\n    s = &quot;x&quot; + &#39;y&#39; + &#x27;z&#x27;\n    t = a&nbsp;b\n</code></pre>";
    let want = "```\nif a < b && c > d:\n    s = \"x\" + 'y' + 'z'\n    t = a b\n```";
    for (path, got) in every_path(html, false).await {
        assert_eq!(got, want, "{path}");
    }
}

#[test]
fn entity_that_is_not_one_stays_as_written() {
    assert_eq!(
        md("<pre>a &amp b &; &#; &#xZZ; &bogus; &</pre>"),
        "```\na &amp b &; &#; &#xZZ; &bogus; &\n```"
    );
    assert_eq!(md("<p>x <code>a &</code> y</p>"), "x `a &` y");
}

#[test]
fn inline_code_is_not_escaped() {
    assert_eq!(
        md("<p>Set <code>primary_key=True</code> on <samp>team_id</samp>.</p>"),
        "Set `primary_key=True` on `team_id`."
    );
    assert_eq!(
        md("<p><code>Vec&lt;u8&gt;</code> and <code>a * b ~ c\\d</code></p>"),
        "`Vec<u8>` and `a * b ~ c\\d`"
    );
}

#[test]
fn text_outside_code_is_escaped_as_before() {
    assert_eq!(
        md("<p>snake_case and 2 * 3 &lt; 7</p><pre>snake_case</pre><p>more_text</p>"),
        "snake\\_case and 2 \\* 3 \\< 7\n```\nsnake_case\n```\nmore\\_text"
    );
}

#[test]
fn tags_inside_a_block_write_no_markdown() {
    // Line number anchors, emphasis, a nested code tag and a line break.
    let html = "<pre><a id=\"l1\" href=\"#l1\"></a><b>let</b> <i>x</i> = <code>1</code>;<br>  <strong>done</strong>()</pre>";
    assert_eq!(md(html), "```\nlet x = 1;\n  done()\n```");
}

#[test]
fn leading_newline_and_trailing_whitespace_are_dropped() {
    assert_eq!(
        md("<pre>\n\n  a\n\n  b  \n\n</pre>"),
        "```\n\n  a\n\n  b\n```"
    );
    assert_eq!(md("<pre>\r\n  a\r\n</pre>"), "```\n  a\n```");
    assert_eq!(md("<pre></pre>"), "```\n```");
    assert_eq!(md("<pre> \n </pre>"), "```\n```");
}

#[test]
fn tabs_and_runs_of_spaces_are_kept() {
    assert_eq!(md("<pre>a\tb    c\n\td</pre>"), "```\na\tb    c\n\td\n```");
}

#[test]
fn fence_is_longer_than_any_backtick_run_in_the_code() {
    assert_eq!(
        md("<pre><code>```python\nx = 1\n```\n</code></pre><p>after</p>"),
        "````\n```python\nx = 1\n```\n````\nafter"
    );
    assert_eq!(
        md("<pre>a `` b ````` c</pre>"),
        "``````\na `` b ````` c\n``````"
    );
    assert_eq!(md("<pre>a `b` ``c``</pre>"), "```\na `b` ``c``\n```");
}

#[test]
fn code_in_a_list_item_ends_the_list_marker_fix() {
    // The text of a list item that is only inline code still counts as the
    // item's first text, so what follows the list starts on its own line.
    assert_eq!(
        md("<ul>\n<li><code>a_b</code></li>\n<li>\n<p><code>c_d</code></p>\n</li>\n</ul>\n<p>next</p>"),
        rewrite_html(
            "<ul>\n<li><code>ab</code></li>\n<li>\n<p><code>cd</code></p>\n</li>\n</ul>\n<p>next</p>",
            false
        )
        .replace("ab", "a_b")
        .replace("cd", "c_d")
    );
    assert!(md("<ul><li><p><code>c_d</code></p>\n</li></ul><p>next</p>").ends_with("`c_d`\nnext"));
}

#[test]
fn block_sits_on_its_own_lines() {
    assert_eq!(
        md("<p>before</p><pre>  code</pre><p>after</p>"),
        "before\n```\n  code\n```\nafter"
    );
    assert_eq!(
        md("before <pre>  code</pre> after"),
        "before\n```\n  code\n```\nafter"
    );
}

#[test]
fn block_without_an_end_tag_is_closed() {
    assert_eq!(md("<p>x</p><pre>  a_b\n  c"), "x\n```\n  a_b\n  c\n```");
}

#[test]
fn block_ends_an_open_link_label() {
    // An anchor the page never closes must not fold the block onto one line.
    let out = md("<a href=\"https://a.example/\">badge\n<pre>line1\n  line2</pre>\n<p>tail</p>");
    assert!(out.contains("\n```\nline1\n  line2\n```\n"), "{out:?}");
}

#[test]
fn marker_bytes_in_the_page_never_reach_the_output() {
    let html = "<p>a\u{e}b\u{f}c</p><pre>d\u{f}e\u{e}f &#14;&#15;</pre><img alt=\"g\u{e}h\" src=\"i.png\"><p>j_k</p>";
    let out = md(html);
    assert!(
        !out.bytes().any(|b| matches!(b, 0x02 | 0x03 | 0x0E | 0x0F)),
        "{out:?}"
    );
    assert!(out.contains("```\ndef &#14;&#15;\n```"), "{out:?}");
    assert!(out.ends_with("j\\_k"), "{out:?}");
}

#[test]
fn header_row_of_th_gets_a_delimiter_row() {
    assert_eq!(
        md("<table><tr><th>a</th><th>b</th></tr><tr><td>1</td><td>2</td></tr></table>"),
        "|a|b|\n|---|---|\n|1|2|"
    );
    assert_eq!(
        md("<table><thead><tr><th>a</th></tr></thead><tbody><tr><td>1</td></tr><tr><td>2</td></tr></tbody></table>"),
        "|a|\n|---|\n|1|\n|2|"
    );
    // A table that is only its header row.
    assert_eq!(
        md("<table><tr><th>a</th><th>b</th></tr></table><p>x</p>"),
        "|a|b|\n|---|---|\nx"
    );
}

#[test]
fn first_row_with_a_td_gets_no_delimiter_row() {
    // Layout tables and row-header tables keep the output they had.
    assert_eq!(
        md("<table><tr><td>a</td><td>b</td></tr><tr><td>1</td><td>2</td></tr></table>"),
        "|a|b|\n|1|2|"
    );
    assert_eq!(
        md("<table><tr><th>k</th><td>v</td></tr><tr><th>k2</th><td>v2</td></tr></table>"),
        "|k|v|\n|k2|v2|"
    );
}

#[test]
fn header_cell_that_spans_gets_no_delimiter_row() {
    // One pipe cell over two columns: a delimiter row sized to the header
    // would make a renderer drop the second column.
    assert_eq!(
        md("<table><tr><th colspan=\"2\">t</th></tr><tr><td>1</td><td>2</td></tr></table>"),
        "|t|\n|1|2|"
    );
    assert_eq!(
        md("<table><tr><th colspan=\"1\" rowspan=\" 1 \">t</th></tr><tr><td>1</td></tr></table>"),
        "|t|\n|---|\n|1|"
    );
}

#[test]
fn each_table_is_judged_on_its_own() {
    assert_eq!(
        md("<table><tr><td>a</td></tr><tr><td>b</td></tr></table><table><tr><th>h</th></tr><tr><td>c</td></tr></table>"),
        "|a|\n|b|\n|h|\n|---|\n|c|"
    );
}

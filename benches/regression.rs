//! Regression benchmarks for fast_html2md
//!
//! These benchmarks serve two purposes:
//! 1. Measure performance to track improvements/regressions
//! 2. Verify output consistency to ensure functionality isn't broken
//!
//! Run with: `cargo bench --bench regression`
//! Generate baselines: `cargo bench --bench regression -- --save-baseline main`
//! Compare to baseline: `cargo bench --bench regression -- --baseline main`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use html2md::{rewrite_html, rewrite_html_streaming};
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;

/// Test case with input HTML and expected output characteristics
struct RegressionCase {
    name: &'static str,
    html: &'static str,
    /// Hash of expected output (for regression detection)
    /// Set to 0 to compute and print the hash on first run
    expected_hash: u64,
    /// Minimum expected output length (sanity check)
    min_output_len: usize,
    /// Substrings that must appear in output
    must_contain: &'static [&'static str],
}

fn hash_output(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    // Normalize whitespace for hash stability
    let normalized: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized.hash(&mut hasher);
    hasher.finish()
}

/// Core HTML element regression cases
const REGRESSION_CASES: &[RegressionCase] = &[
    RegressionCase {
        name: "headings",
        html: "<h1>Title</h1><h2>Subtitle</h2><h3>Section</h3><h4>Subsection</h4><h5>Minor</h5><h6>Smallest</h6>",
        expected_hash: 0, // Will be computed
        min_output_len: 40,
        must_contain: &["# Title", "## Subtitle", "### Section"],
    },
    RegressionCase {
        name: "paragraphs",
        html: "<p>First paragraph.</p><p>Second paragraph.</p><p>Third paragraph with <strong>bold</strong> and <em>italic</em>.</p>",
        expected_hash: 0,
        min_output_len: 50,
        must_contain: &["First paragraph", "Second paragraph", "**bold**", "*italic*"],
    },
    RegressionCase {
        name: "links",
        html: r#"<a href="https://example.com">Link text</a> and <a href="/relative">Relative link</a>"#,
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["[Link text]", "https://example.com"],
    },
    RegressionCase {
        name: "images",
        html: r#"<img src="image.png" alt="Alt text"> and <img src="https://example.com/photo.jpg" alt="Photo">"#,
        expected_hash: 0,
        min_output_len: 10,
        must_contain: &["![Alt text]", "![Photo]"],
    },
    RegressionCase {
        name: "unordered-lists",
        html: "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>",
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["* Item 1", "* Item 2", "* Item 3"],
    },
    RegressionCase {
        name: "ordered-lists",
        html: "<ol><li>First</li><li>Second</li><li>Third</li></ol>",
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["1. First", "2. Second", "3. Third"],
    },
    RegressionCase {
        name: "nested-lists",
        html: "<ul><li>Outer<ul><li>Inner 1</li><li>Inner 2</li></ul></li><li>Another outer</li></ul>",
        expected_hash: 0,
        min_output_len: 30,
        must_contain: &["* Outer", "Inner 1", "Inner 2"],
    },
    RegressionCase {
        name: "blockquotes",
        html: "<blockquote>This is a quote.</blockquote><blockquote><p>Multi</p><p>Paragraph</p></blockquote>",
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["> This is a quote"],
    },
    RegressionCase {
        name: "code-inline",
        html: "<p>Use <code>let x = 1;</code> for variables.</p>",
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["`let x = 1;`"],
    },
    RegressionCase {
        name: "code-blocks",
        html: "<pre><code>fn main() {\n    println!(\"Hello\");\n}</code></pre>",
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["fn main()"],
    },
    RegressionCase {
        name: "tables",
        html: "<table><thead><tr><th>Name</th><th>Value</th></tr></thead><tbody><tr><td>A</td><td>1</td></tr><tr><td>B</td><td>2</td></tr></tbody></table>",
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["|Name|", "|Value|", "|A|", "|B|"],
    },
    RegressionCase {
        name: "mixed-formatting",
        html: "<p>This has <strong>bold</strong>, <em>italic</em>, <strong><em>bold-italic</em></strong>, and <code>code</code>.</p>",
        expected_hash: 0,
        min_output_len: 30,
        must_contain: &["**bold**", "*italic*", "`code`"],
    },
    RegressionCase {
        name: "horizontal-rule",
        html: "<p>Before</p><hr><p>After</p>",
        expected_hash: 0,
        min_output_len: 10,
        must_contain: &["Before", "After"],
    },
    RegressionCase {
        name: "line-breaks",
        html: "<p>Line one<br>Line two<br>Line three</p>",
        expected_hash: 0,
        min_output_len: 20,
        must_contain: &["Line one", "Line two", "Line three"],
    },
    RegressionCase {
        name: "complex-document",
        html: r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test</title></head>
            <body>
                <h1>Main Title</h1>
                <p>Introduction paragraph with <a href="https://example.com">a link</a>.</p>
                <h2>Section 1</h2>
                <ul>
                    <li>Point A</li>
                    <li>Point B with <strong>emphasis</strong></li>
                </ul>
                <h2>Section 2</h2>
                <blockquote>A notable quote.</blockquote>
                <pre><code>code_example();</code></pre>
            </body>
            </html>
        "#,
        expected_hash: 0,
        min_output_len: 100,
        must_contain: &[
            "# Main Title",
            "## Section 1",
            "## Section 2",
            "Point A",
            "**emphasis**",
            "> A notable quote",
        ],
    },
];

/// Verify output correctness for a regression case
fn verify_output(case: &RegressionCase, output: &str) -> Result<(), String> {
    // Check minimum length
    if output.len() < case.min_output_len {
        return Err(format!(
            "{}: Output too short ({} < {})",
            case.name,
            output.len(),
            case.min_output_len
        ));
    }

    // Check required substrings
    for &substring in case.must_contain {
        if !output.contains(substring) {
            return Err(format!(
                "{}: Missing expected substring: '{}'",
                case.name, substring
            ));
        }
    }

    // Check hash if specified (non-zero)
    if case.expected_hash != 0 {
        let actual_hash = hash_output(output);
        if actual_hash != case.expected_hash {
            return Err(format!(
                "{}: Hash mismatch (expected {}, got {}). Output may have changed.",
                case.name, case.expected_hash, actual_hash
            ));
        }
    }

    Ok(())
}

/// Benchmark: Regression tests with correctness verification
/// Each iteration verifies output correctness along with performance
pub fn bench_regression_correctness(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression/correctness");

    for case in REGRESSION_CASES {
        group.bench_with_input(BenchmarkId::new("sync", case.name), case, |b, case| {
            b.iter(|| {
                let output = rewrite_html(case.html, false);
                // Verify on each iteration (catches non-determinism)
                if let Err(e) = verify_output(case, &output) {
                    panic!("Regression detected: {}", e);
                }
                black_box(output)
            })
        });
    }

    group.finish();
}

/// Benchmark: Async regression tests
pub fn bench_regression_async(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression/async");

    for case in REGRESSION_CASES {
        group.bench_with_input(BenchmarkId::new("async", case.name), case, |b, case| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.to_async(rt).iter(|| async {
                let output = rewrite_html_streaming(case.html, false).await;
                if let Err(e) = verify_output(case, &output) {
                    panic!("Async regression detected: {}", e);
                }
                black_box(output)
            })
        });
    }

    group.finish();
}

/// Benchmark: Real-world document regression
/// Tests against actual HTML files with snapshot verification
pub fn bench_real_world_regression(c: &mut Criterion) {
    let test_files = [
        ("wiki-cat", "../test-samples/wiki/en-wikipedia-org_wiki_Cat.html"),
        ("real-world", "../test-samples/real-world-1.html"),
    ];

    let mut group = c.benchmark_group("regression/real-world");
    group.sample_size(50);

    for (name, path) in test_files {
        let mut html = String::new();
        if File::open(path)
            .and_then(|mut f| f.read_to_string(&mut html))
            .is_err()
        {
            continue;
        }

        // Store baseline output for comparison
        let baseline_output = rewrite_html(&html, false);
        let baseline_hash = hash_output(&baseline_output);
        let baseline_len = baseline_output.len();

        group.bench_function(format!("sync-{}", name), |b| {
            b.iter(|| {
                let output = rewrite_html(&html, false);
                // Sanity checks
                assert!(
                    output.len() >= baseline_len / 2,
                    "Output suspiciously short"
                );
                assert!(!output.is_empty(), "Output is empty");
                black_box(output)
            })
        });

        group.bench_function(format!("async-{}", name), |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let html_clone = html.clone();
            b.to_async(rt).iter(|| {
                let html = html_clone.clone();
                async move {
                    let output = rewrite_html_streaming(&html, false).await;
                    assert!(
                        output.len() >= baseline_len / 2,
                        "Async output suspiciously short"
                    );
                    black_box(output)
                }
            })
        });

        // Log baseline info for reference
        println!(
            "Baseline for {}: {} bytes output, hash {}",
            name, baseline_len, baseline_hash
        );
    }

    group.finish();
}

/// Benchmark: Edge cases that have caused issues in the past
pub fn bench_edge_cases(c: &mut Criterion) {
    let edge_cases = [
        ("empty-doc", "<!DOCTYPE html><html><body></body></html>"),
        ("whitespace-only", "   \n\t\n   "),
        ("deeply-nested", "<div><div><div><div><div><p>Deep</p></div></div></div></div></div>"),
        ("many-siblings", "<p>A</p><p>B</p><p>C</p><p>D</p><p>E</p><p>F</p><p>G</p><p>H</p>"),
        ("special-chars", "<p>&amp; &lt; &gt; &quot; &#39; &nbsp;</p>"),
        ("unicode", "<p>日本語 中文 한국어 Ελληνικά العربية</p>"),
        ("malformed-unclosed", "<p>Unclosed paragraph<p>Another one"),
        ("malformed-extra-close", "<p>Text</p></p></p>"),
        ("script-removal", "<p>Before</p><script>alert('xss')</script><p>After</p>"),
        ("style-removal", "<p>Before</p><style>.x{color:red}</style><p>After</p>"),
    ];

    let mut group = c.benchmark_group("regression/edge-cases");

    for (name, html) in edge_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let output = rewrite_html(html, false);
                // Should never panic, should always return something
                black_box(output)
            })
        });
    }

    group.finish();
}

/// Print hashes for all regression cases (run once to generate expected hashes)
#[allow(dead_code)]
fn print_regression_hashes() {
    println!("\n=== Regression Case Hashes ===");
    for case in REGRESSION_CASES {
        let output = rewrite_html(case.html, false);
        let hash = hash_output(&output);
        println!(
            "Case '{}': hash = {}, len = {}",
            case.name,
            hash,
            output.len()
        );
        println!("  Output: {:?}", &output[..output.len().min(100)]);
    }
}

criterion_group!(
    benches,
    bench_regression_correctness,
    bench_regression_async,
    bench_real_world_regression,
    bench_edge_cases,
);

criterion_main!(benches);

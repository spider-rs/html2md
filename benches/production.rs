//! The conversion the crawler and the browser engine run: the send rewriter fed
//! 8 KiB chunks through `rewrite_html_custom_with_url_streaming`, with a page
//! URL and an ignore set, the way both callers invoke it.
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use html2md::rewrite_html_custom_with_url_streaming;
use std::collections::HashSet;

const SAMPLES: &[(&str, &str)] = &[
    ("ja", "../test-samples/real-world-ja-1.html"),
    ("spider-cloud", "../test-samples/spider-cloud.html"),
    ("real-world-1", "../test-samples/real-world-1.html"),
    (
        "wiki-cat",
        "../test-samples/wiki/en-wikipedia-org_wiki_Cat.html",
    ),
    (
        "wiki-nytimes",
        "../test-samples/wiki/en-wikipedia-org_wiki_The_New_York_Times.html",
    ),
    (
        "wiki-languages",
        "../test-samples/wiki/en-wikipedia-org_wiki_Languages_of_Africa.html",
    ),
];

pub fn bench_production(c: &mut Criterion) {
    let runtime = match tokio::runtime::Builder::new_current_thread().build() {
        Ok(runtime) => runtime,
        Err(_) => return,
    };
    let url = url::Url::parse("https://example.com/docs/page").ok();
    let ignore: Option<HashSet<String>> = Some(HashSet::from(["svg".to_string()]));

    let mut group = c.benchmark_group("production");

    for (name, path) in SAMPLES {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let html = String::from_utf8_lossy(&bytes).into_owned();
        group.throughput(Throughput::Bytes(html.len() as u64));

        for commonmark in [false, true] {
            let id = if commonmark { "commonmark" } else { "markdown" };
            group.bench_with_input(BenchmarkId::new(id, name), &html, |b, html| {
                b.iter(|| {
                    runtime.block_on(async {
                        black_box(
                            rewrite_html_custom_with_url_streaming(
                                black_box(html),
                                &ignore,
                                commonmark,
                                &url,
                            )
                            .await,
                        )
                    })
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, bench_production);
criterion_main!(benches);

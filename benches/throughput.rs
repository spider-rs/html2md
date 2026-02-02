use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use html2md::{rewrite_html, rewrite_html_streaming};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

/// Load all test samples grouped by approximate size category
fn load_test_samples() -> Vec<(&'static str, String)> {
    let samples = [
        ("small", "../test-samples/example.html"),
        ("medium", "../test-samples/marcfs-readme.html"),
        ("large", "../test-samples/real-world-1.html"),
        ("wiki-cat", "../test-samples/wiki/en-wikipedia-org_wiki_Cat.html"),
        (
            "wiki-nytimes",
            "../test-samples/wiki/en-wikipedia-org_wiki_The_New_York_Times.html",
        ),
    ];

    samples
        .iter()
        .filter_map(|(name, path)| {
            let mut html = String::new();
            File::open(path)
                .and_then(|mut f| f.read_to_string(&mut html))
                .ok()?;
            Some((*name, html))
        })
        .collect()
}

/// Load all wiki samples for batch processing benchmarks
fn load_wiki_samples() -> Vec<String> {
    let wiki_dir = Path::new("../test-samples/wiki");
    fs::read_dir(wiki_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "html"))
                .filter_map(|e| {
                    let mut html = String::new();
                    File::open(e.path())
                        .and_then(|mut f| f.read_to_string(&mut html))
                        .ok()?;
                    Some(html)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Benchmark: Throughput across different document sizes
/// Measures bytes/second processing rate for sync rewriter
pub fn bench_throughput_by_size(c: &mut Criterion) {
    let samples = load_test_samples();
    let mut group = c.benchmark_group("throughput/by-size");

    for (name, html) in &samples {
        let bytes = html.len() as u64;
        group.throughput(Throughput::Bytes(bytes));

        group.bench_with_input(BenchmarkId::new("sync", name), html, |b, html| {
            b.iter(|| black_box(rewrite_html(html, false)))
        });

        group.bench_with_input(BenchmarkId::new("async", name), html, |b, html| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.to_async(rt)
                .iter(|| async { black_box(rewrite_html_streaming(html, false).await) });
        });
    }

    group.finish();
}

/// Benchmark: Batch processing (simulates crawler processing multiple pages)
/// Tests sequential throughput of multiple documents
pub fn bench_batch_processing(c: &mut Criterion) {
    let wiki_samples = load_wiki_samples();
    if wiki_samples.is_empty() {
        return;
    }

    let total_bytes: u64 = wiki_samples.iter().map(|s| s.len() as u64).sum();
    let doc_count = wiki_samples.len();

    let mut group = c.benchmark_group("throughput/batch");
    group.throughput(Throughput::Bytes(total_bytes));
    group.sample_size(50);

    group.bench_function(
        format!("sync-sequential-{}docs", doc_count),
        |b| {
            b.iter(|| {
                for html in &wiki_samples {
                    black_box(rewrite_html(html, false));
                }
            })
        },
    );

    group.bench_function(
        format!("async-sequential-{}docs", doc_count),
        |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            b.to_async(rt).iter(|| async {
                for html in &wiki_samples {
                    black_box(rewrite_html_streaming(html, false).await);
                }
            });
        },
    );

    group.finish();
}

/// Benchmark: Concurrent processing (simulates high-throughput crawler)
/// Tests parallel document processing using tokio tasks
pub fn bench_concurrent_processing(c: &mut Criterion) {
    let wiki_samples = load_wiki_samples();
    if wiki_samples.is_empty() {
        return;
    }

    let total_bytes: u64 = wiki_samples.iter().map(|s| s.len() as u64).sum();
    let doc_count = wiki_samples.len();

    let mut group = c.benchmark_group("throughput/concurrent");
    group.throughput(Throughput::Bytes(total_bytes));
    group.sample_size(50);

    // Test with different concurrency levels
    for concurrency in [2, 4, 8] {
        group.bench_function(
            format!("async-concurrent-{}docs-{}workers", doc_count, concurrency),
            |b| {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(concurrency)
                    .enable_all()
                    .build()
                    .unwrap();

                let samples_clone: Vec<_> = wiki_samples.iter().cloned().collect();

                b.to_async(rt).iter(|| {
                    let samples = samples_clone.clone();
                    async move {
                        let tasks: Vec<_> = samples
                            .into_iter()
                            .map(|html| {
                                tokio::spawn(async move {
                                    black_box(rewrite_html_streaming(&html, false).await)
                                })
                            })
                            .collect();

                        for task in tasks {
                            let _ = task.await;
                        }
                    }
                });
            },
        );
    }

    // Sync with rayon-style parallelism (using threads)
    group.bench_function(
        format!("sync-parallel-{}docs", doc_count),
        |b| {
            b.iter(|| {
                std::thread::scope(|s| {
                    let handles: Vec<_> = wiki_samples
                        .iter()
                        .map(|html| s.spawn(|| black_box(rewrite_html(html, false))))
                        .collect();

                    for handle in handles {
                        let _ = handle.join();
                    }
                });
            })
        },
    );

    group.finish();
}

/// Benchmark: Repeated processing of same document (cache effects, JIT warmup)
/// Useful for understanding steady-state performance
pub fn bench_repeated_processing(c: &mut Criterion) {
    let mut html = String::new();
    let path = Path::new("../test-samples/wiki/en-wikipedia-org_wiki_Cat.html");
    if File::open(path)
        .and_then(|mut f| f.read_to_string(&mut html))
        .is_err()
    {
        return;
    }

    let bytes = html.len() as u64;
    let mut group = c.benchmark_group("throughput/repeated");
    group.throughput(Throughput::Bytes(bytes));
    group.sample_size(200);

    group.bench_function("sync-hot-path", |b| {
        b.iter(|| black_box(rewrite_html(&html, false)))
    });

    group.bench_function("async-hot-path", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.to_async(rt)
            .iter(|| async { black_box(rewrite_html_streaming(&html, false).await) });
    });

    group.finish();
}

/// Benchmark: Memory allocation patterns via processing varying chunk counts
/// Helps identify allocation overhead
pub fn bench_allocation_pressure(c: &mut Criterion) {
    // Generate synthetic HTML of varying sizes to test allocation scaling
    let sizes = [1_000, 10_000, 50_000, 100_000];

    let mut group = c.benchmark_group("throughput/allocation");

    for size in sizes {
        // Generate HTML with repeated paragraphs
        let paragraph = "<p>This is a test paragraph with some <strong>bold</strong> and <em>italic</em> text and a <a href=\"https://example.com\">link</a>.</p>\n";
        let repeats = size / paragraph.len();
        let html = format!(
            "<!DOCTYPE html><html><body>{}</body></html>",
            paragraph.repeat(repeats)
        );

        let bytes = html.len() as u64;
        group.throughput(Throughput::Bytes(bytes));

        group.bench_with_input(
            BenchmarkId::new("sync", format!("{}kb", size / 1000)),
            &html,
            |b, html| b.iter(|| black_box(rewrite_html(html, false))),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_throughput_by_size,
    bench_batch_processing,
    bench_concurrent_processing,
    bench_repeated_processing,
    bench_allocation_pressure,
);

criterion_main!(benches);

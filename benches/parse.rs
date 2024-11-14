use criterion::{black_box, criterion_group, criterion_main, Criterion};
use html2md::{parse_html, rewrite_html};
use std::fs::File;
use std::io::Read;

/// bench crawling between different libs
pub fn bench_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("crawl-speed/libraries");
    let sample_count = 100;
    let sample_title = format!("crawl {} samples", sample_count);

    let path = std::path::Path::new("../test-samples/real-world-1.html");
    let mut html = String::new();
    let mut html_file = File::open(path).unwrap();

    html_file.read_to_string(&mut html).unwrap();

    group.bench_function(format!("Scraper real-world-1: {}", sample_title), |b| {
        b.iter(|| black_box(parse_html(&html, false)))
    });

    group.bench_function(format!("Rewriter real-world-1: {}", sample_title), |b| {
        b.iter(|| black_box(rewrite_html(&html, false)))
    });

    let path = std::path::Path::new("../test-samples/wiki/en-wikipedia-org_wiki_Cat.html");

    let mut html = String::new();
    let mut html_file = File::open(path).unwrap();
    html_file.read_to_string(&mut html).unwrap();

    group.bench_function(format!("Scraper wiki-cat: {}", sample_title), |b| {
        b.iter(|| black_box(parse_html(&html, false)))
    });

    group.bench_function(format!("Rewriter wiki-cat: {}", sample_title), |b| {
        b.iter(|| black_box(rewrite_html(&html, false)))
    });

    group.finish();
}

criterion_group!(benches, bench_speed);
criterion_main!(benches);

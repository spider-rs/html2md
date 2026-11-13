//! Exact-output guard for the rewriter entry points.
//!
//! Every fixture in `test-samples/` is converted through each public rewriter
//! entry point under each option set, and the output is compared to the hash
//! recorded in `tests/golden/hashes.tsv`. A performance change must leave every
//! hash as it is. Record a new file only when the output is meant to change:
//! `GOLDEN_RECORD=1 cargo test -p fast_html2md --test golden`.
#![cfg(all(feature = "rewriter", feature = "stream"))]

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// FNV-1a, written out so the recorded values do not depend on the std hasher.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn samples_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-samples")
}

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/hashes.tsv")
}

/// Every `.html` fixture, sorted by name so the record is stable.
fn fixtures() -> Vec<(String, String)> {
    let mut found = Vec::new();
    for dir in [samples_dir(), samples_dir().join("wiki")] {
        let entries = std::fs::read_dir(&dir).expect("fixture directory");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "html") {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let bytes = std::fs::read(&path).expect("fixture file");
                found.push((name, String::from_utf8_lossy(&bytes).into_owned()));
            }
        }
    }
    found.sort();
    found
}

async fn convert_all() -> BTreeMap<String, (u64, usize)> {
    let mut results = BTreeMap::new();
    let ignore: Option<HashSet<String>> = Some(HashSet::from(["aside".to_string()]));
    let base = url::Url::parse("https://example.com/docs/page").ok();

    for (name, html) in fixtures() {
        for commonmark in [false, true] {
            for (url_tag, url) in [("nourl", None), ("url", base.clone())] {
                for (custom_tag, custom) in [("all", None), ("ignore", ignore.clone())] {
                    let key = |entry: &str| {
                        format!("{name}|cm={commonmark}|{url_tag}|{custom_tag}|{entry}")
                    };

                    let sync =
                        html2md::rewrite_html_custom_with_url(&html, &custom, commonmark, &url);
                    results.insert(key("sync"), (fnv1a(sync.as_bytes()), sync.len()));

                    let send = html2md::rewrite_html_custom_with_url_streaming(
                        &html, &custom, commonmark, &url,
                    )
                    .await;
                    results.insert(key("send"), (fnv1a(send.as_bytes()), send.len()));

                    let small = html2md::rewrite_html_custom_with_url_and_chunk(
                        &html, &custom, commonmark, &url, 251,
                    )
                    .await;
                    results.insert(key("send251"), (fnv1a(small.as_bytes()), small.len()));

                    let chunks: Vec<Result<&[u8], std::io::Error>> =
                        html.as_bytes().chunks(37).map(Ok).collect();
                    let stream = futures_util::stream::iter(chunks);
                    let streamed = html2md::rewrite_html_stream_custom_with_url(
                        stream, &custom, commonmark, &url,
                    )
                    .await
                    .expect("stream conversion");
                    results.insert(
                        key("stream37"),
                        (fnv1a(streamed.as_bytes()), streamed.len()),
                    );
                }
            }
        }
    }
    results
}

#[tokio::test]
async fn golden_outputs_do_not_change() {
    let results = convert_all().await;
    assert!(!results.is_empty(), "no fixtures found");

    let mut body = String::new();
    for (key, (hash, len)) in &results {
        let _ = writeln!(body, "{key}\t{hash:016x}\t{len}");
    }

    if std::env::var_os("GOLDEN_RECORD").is_some() {
        let path = golden_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("golden directory");
        }
        std::fs::write(&path, body).expect("write golden file");
        return;
    }

    let recorded = match std::fs::read_to_string(golden_path()) {
        Ok(recorded) => recorded,
        Err(_) => panic!(
            "no golden file at {}. The outputs of this build are:\nGOLDEN-BEGIN\n{body}GOLDEN-END",
            golden_path().display()
        ),
    };
    let mut expected = BTreeMap::new();
    for line in recorded.lines() {
        let mut parts = line.split('\t');
        if let (Some(key), Some(hash), Some(len)) = (parts.next(), parts.next(), parts.next()) {
            expected.insert(key.to_string(), (hash.to_string(), len.to_string()));
        }
    }

    let mut failures = Vec::new();
    for (key, (hash, len)) in &results {
        match expected.get(key) {
            Some((want_hash, want_len)) => {
                if *want_hash != format!("{hash:016x}") || *want_len != len.to_string() {
                    failures.push(format!(
                        "{key}: recorded {want_hash} len {want_len}, got {hash:016x} len {len}"
                    ));
                }
            }
            None => failures.push(format!("{key}: no recorded hash")),
        }
    }
    for key in expected.keys() {
        if !results.contains_key(key) {
            failures.push(format!("{key}: recorded but not produced"));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} outputs changed:\n{}",
        failures.len(),
        results.len(),
        failures.join("\n")
    );
}

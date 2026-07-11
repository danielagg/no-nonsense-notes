use criterion::{criterion_group, criterion_main, Criterion};
use loro::{ExportMode, LoroDoc};
use std::hint::black_box;
use std::time::Duration;

const NUM_EDITS: usize = 10_000;
const LOAD_TIME_MS: u128 = 500;


fn generate_10k_edit_doc() -> (Vec<u8>, String) {
    let doc = LoroDoc::new();
    let text = doc.get_text("content");

    text.insert(0, "# Test Document\n\nThis is a test document with multiple paragraphs.\n\n## Section 1\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit.\n\n## Section 2\n\nSed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n## Section 3\n\nUt enim ad minim veniam, quis nostrud exercitation ullamco laboris.\n").unwrap();

    for i in 0..NUM_EDITS {
        let len = text.to_string().len();
        let pos = ((i * 7 + 13) * 3) % len.max(1);
        if i % 5 == 0 && len > 20 {
            if pos + 1 < len {
                text.delete(pos, 1).unwrap();
            }
        } else {
            let c = match i % 3 {
                0 => 'a',
                1 => '\n',
                _ => ' ',
            };
            let s: String = c.to_string();
            text.insert(pos, &s).unwrap();
        }
    }

    let content_pre = text.to_string();
    let blob = doc.export(ExportMode::Snapshot).unwrap();
    (blob, content_pre)
}

fn get_max_rss() -> u64 {
    unsafe {
        let mut usage: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut usage);
        if cfg!(target_os = "macos") {
            usage.ru_maxrss as u64
        } else {
            usage.ru_maxrss as u64 * 1024
        }
    }
}

fn bench_load_snapshot(c: &mut Criterion) {
    let (snapshot, content_pre) = generate_10k_edit_doc();

    let blob_size = snapshot.len();
    let edit_count = content_pre.len();
    eprintln!("--- Loro 10k-edit benchmark data ---");
    eprintln!("Blob size: {} bytes ({:.2} KB)", blob_size, blob_size as f64 / 1024.0);
    eprintln!("Final content length: {} chars", edit_count);

    let rss_before = get_max_rss();
    let start = std::time::Instant::now();
    let doc = LoroDoc::from_snapshot(&snapshot).unwrap();
    let elapsed = start.elapsed();
    let rss_after = get_max_rss();
    let content_post = doc.get_text("content").to_string();

    assert_eq!(content_pre, content_post, "content mismatch after round-trip");

    eprintln!("--- Timing ---");
    eprintln!("First load: {} ms", elapsed.as_millis());
    let mem_used = rss_after.saturating_sub(rss_before);
    eprintln!("RSS delta: {} bytes ({:.2} MB)", mem_used, mem_used as f64 / (1024.0 * 1024.0));

    assert!(
        elapsed.as_millis() < LOAD_TIME_MS,
        "load time {} ms exceeds {} ms gate",
        elapsed.as_millis(),
        LOAD_TIME_MS
    );

    let mut group = c.benchmark_group("loro_10k_edits");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    group.bench_function("load_snapshot", |b| {
        b.iter(|| {
            let d = LoroDoc::from_snapshot(&snapshot).unwrap();
            black_box(d);
        })
    });

    group.bench_function("import_updates", |b| {
        let updates = doc.export(ExportMode::all_updates()).unwrap();
        b.iter(|| {
            let d = LoroDoc::new();
            d.import(&updates).unwrap();
            black_box(d);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_load_snapshot);
criterion_main!(benches);

use loro::{ExportMode, LoroDoc};

/// Phase 0 gate: confirm Loro handles a 10k-edit markdown document
/// within acceptable time and memory bounds before building on it.
///
/// Thresholds:
///   - load time < 500ms
///   - RSS delta < 50 MB
#[test]
fn loro_10k_edit_gate() {
    let (snapshot, content_pre) = generate_10k_edit_doc();

    let blob_size = snapshot.len();
    eprintln!("Blob size: {blob_size} bytes ({:.2} KB)", blob_size as f64 / 1024.0);
    eprintln!("Final content length: {} chars", content_pre.len());

    let rss_before = get_max_rss();
    let start = std::time::Instant::now();
    let doc = LoroDoc::from_snapshot(&snapshot).unwrap();
    let elapsed = start.elapsed();
    let rss_after = get_max_rss();
    let content_post = doc.get_text("content").to_string();

    assert_eq!(content_pre, content_post, "content mismatch after round-trip");

    let mem_used = rss_after.saturating_sub(rss_before);
    eprintln!("Load time: {} µs", elapsed.as_micros());
    eprintln!("RSS delta: {mem_used} bytes ({:.2} MB)", mem_used as f64 / (1024.0 * 1024.0));

    assert!(
        elapsed.as_millis() < 500,
        "load time {} ms exceeds 500 ms gate",
        elapsed.as_millis()
    );

    assert!(
        mem_used < 50 * 1024 * 1024,
        "memory {mem_used} bytes exceeds 50 MB gate"
    );
}

/// Build a Loro document with ~10k individual edit operations,
/// export as snapshot, return (snapshot_bytes, final_text).
fn generate_10k_edit_doc() -> (Vec<u8>, String) {
    let doc = LoroDoc::new();
    let text = doc.get_text("content");

    text.insert(
        0,
        "# Test Document\n\n\
         This is a test document with multiple paragraphs.\n\n\
         ## Section 1\n\n\
         Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n\n\
         ## Section 2\n\n\
         Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n\
         ## Section 3\n\n\
         Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.\n",
    )
    .unwrap();

    for i in 0..10_000 {
        let len = text.to_string().len();
        let pos = ((i * 7 + 13) * 3) % len.max(1);
        if i % 5 == 0 && len > 20 {
            let end = (pos + 1).min(len - 1);
            text.delete(pos, end - pos).unwrap();
        } else {
            let c = match i % 3 {
                0 => 'a',
                1 => '\n',
                _ => ' ',
            };
            text.insert(pos, &c.to_string()).unwrap();
        }
    }

    let content = text.to_string();
    let blob = doc.export(ExportMode::Snapshot).unwrap();
    (blob, content)
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

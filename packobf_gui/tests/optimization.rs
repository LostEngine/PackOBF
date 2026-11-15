use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use packobf_gui::app::{App, Statistics, Status, default_options};

#[test]
fn optimization_creates_and_reuses_cache_and_invalidates_output_on_failure() {
    let scratch = ScratchDir::new();
    let input = scratch.0.join("pack with spaces.zip");
    let cache = scratch.0.join("cache.bin");
    let output = scratch.0.join("optimized.zip");

    // An empty ZIP consists of a 22-byte end-of-central-directory record.
    let mut zip = [0; 22];
    zip[..4].copy_from_slice(b"PK\x05\x06");
    fs::write(&input, zip).unwrap();

    let mut app = App::default();
    let mut options = default_options();
    options.num_threads = Some(1);
    app.start(input.clone(), Some(cache.clone()), options.clone())
        .unwrap();
    assert!(app.start(input.clone(), None, options.clone()).is_err());
    let stats = wait_for_result(&mut app).unwrap();

    assert!(matches!(app.status(), Status::Done(_)));
    assert_eq!(stats.input_bytes, zip.len());
    assert!(app.copyable_logs().contains("WARNING: Invalid cache file"));
    assert!(cache.is_file());
    assert!(app.poll().is_none());

    app.save_output(&output).unwrap();
    let saved = fs::read(&output).unwrap();
    assert_eq!(saved.len(), stats.output_bytes);
    assert_eq!(&saved[saved.len() - 22..][..4], b"PK\x05\x06");

    app.invalidate_result();
    assert!(matches!(app.status(), Status::Idle));
    assert!(app.save_output(&output).is_err());

    app.start(input.clone(), None, options.clone()).unwrap();
    app.invalidate_result();
    assert!(matches!(app.status(), Status::Running(_)));
    assert!(wait_for_result(&mut app).is_none());
    assert!(matches!(app.status(), Status::Idle));
    assert!(!app.can_save());

    app.start(input.clone(), Some(cache), options.clone())
        .unwrap();
    assert!(app.logs().is_empty());
    assert!(!app.can_save());
    wait_for_result(&mut app).unwrap();
    assert!(app.copyable_logs().contains("INFO: Cache loaded:"));

    fs::write(&input, b"not a ZIP").unwrap();
    app.start(input.clone(), None, options.clone()).unwrap();
    assert!(wait_for_result(&mut app).is_none());
    assert!(matches!(app.status(), Status::Failed("Processing error")));
    assert!(app.copyable_logs().contains("ERROR: "));
    assert!(app.save_output(&output).is_err());
    assert_eq!(fs::read(&output).unwrap(), saved);

    fs::remove_file(&input).unwrap();
    app.start(input, None, options).unwrap();
    assert!(wait_for_result(&mut app).is_none());
    assert!(matches!(app.status(), Status::Failed("Error reading file")));
    assert!(app.copyable_logs().contains("pack with spaces.zip"));
}

fn wait_for_result(app: &mut App) -> Option<Statistics> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let stats = app.poll();
        if !app.is_running() {
            return stats;
        }
        assert!(Instant::now() < deadline, "optimization did not finish");
        thread::sleep(Duration::from_millis(5));
    }
}

struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new() -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let path = std::env::temp_dir().join(format!(
            "packobf-gui-test-{}-{}",
            std::process::id(),
            timestamp.as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

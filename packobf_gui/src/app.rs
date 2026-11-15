//! UI-independent state and background processing for the desktop frontends.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

pub use packobf::options::{Compression, Options, ShaderCompression, SoundQuality};
pub use packobf::version::MinecraftVersion;
pub use packobf::{LogLevel, LogMessage, Progress};
use tokio::sync::{mpsc as tokio_mpsc, watch};

pub fn default_options() -> Options {
    Options {
        rename_files: true,
        corrupt_png_files: true,
        ..Options::normal()
    }
}

#[derive(Default)]
pub struct App {
    job: Option<Job>,
    status: Status,
    logs: Vec<LogMessage>,
    output: Option<Vec<u8>>,
}

struct Job {
    progress: watch::Receiver<Progress>,
    logs: tokio_mpsc::UnboundedReceiver<LogMessage>,
    result: mpsc::Receiver<Result<Output, Failure>>,
    invalidated: bool,
}

struct Output {
    bytes: Vec<u8>,
    stats: Statistics,
}

struct Failure {
    status: &'static str,
    message: String,
}

#[derive(Default)]
pub enum Status {
    #[default]
    Idle,
    Running(Progress),
    Done(Duration),
    Failed(&'static str),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => f.write_str("Idle"),
            Self::Running(Progress::Idle) => f.write_str("Optimizing…"),
            Self::Running(Progress::ReadingZip { current, total }) => {
                write!(f, "Reading ZIP ({current}/{total})")
            }
            Self::Running(Progress::Parsing { current }) => write!(f, "Parsing {current}"),
            Self::Running(Progress::Optimizing {
                current,
                index,
                total,
            }) => write!(f, "Optimizing ({index}/{total}) {current}"),
            Self::Running(Progress::Finishing | Progress::Done) => f.write_str("Finishing…"),
            Self::Done(elapsed) => write!(f, "Done {:.3}s", elapsed.as_secs_f64()),
            Self::Failed(message) => f.write_str(message),
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub elapsed: Duration,
    pub input_bytes: usize,
    pub output_bytes: usize,
}

impl Statistics {
    pub fn input_size(&self) -> String {
        format_bytes(self.input_bytes)
    }

    pub fn output_size(&self) -> String {
        format_bytes(self.output_bytes)
    }

    pub fn saved(&self) -> String {
        let sign = if self.input_bytes < self.output_bytes {
            "-"
        } else {
            ""
        };
        let bytes = self.input_bytes.abs_diff(self.output_bytes);
        let percent = 100.0 - self.output_bytes as f64 / self.input_bytes as f64 * 100.0;
        format!("{sign}{} ({percent:.2}%)", format_bytes(bytes))
    }
}

impl App {
    pub fn start(
        &mut self,
        input: PathBuf,
        cache: Option<PathBuf>,
        options: Options,
    ) -> io::Result<()> {
        if self.is_running() {
            return Err(io::Error::other("Optimization is already running"));
        }

        let (progress_tx, progress) = watch::channel(Progress::Idle);
        let (log_tx, logs) = tokio_mpsc::unbounded_channel();
        let (result_tx, result) = mpsc::channel();

        thread::Builder::new()
            .name("packobf-optimizer".into())
            .spawn(move || {
                let result = optimize(input, cache, &options, progress_tx, &log_tx);
                let _ = result_tx.send(result);
            })?;

        self.job = Some(Job {
            progress,
            logs,
            result,
            invalidated: false,
        });
        self.status = Status::Running(Progress::Idle);
        self.logs.clear();
        self.output = None;
        Ok(())
    }

    /// Called by the frontend's event loop. Delivers statistics once per
    /// successful job, after collecting its remaining log messages.
    pub fn poll(&mut self) -> Option<Statistics> {
        let job = self.job.as_mut()?;
        let result = match job.result.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(Failure {
                status: "Processing error",
                message: "The optimization worker stopped unexpectedly".into(),
            })),
        };

        let limit = if result.is_some() { usize::MAX } else { 512 };
        self.logs
            .extend(std::iter::from_fn(|| job.logs.try_recv().ok()).take(limit));

        let Some(result) = result else {
            self.status = Status::Running(job.progress.borrow().clone());
            return None;
        };
        let invalidated = job.invalidated;
        self.job = None;
        if invalidated {
            self.status = Status::Idle;
        }

        match result {
            Ok(_) if invalidated => None,
            Ok(output) => {
                self.status = Status::Done(output.stats.elapsed);
                self.output = Some(output.bytes);
                Some(output.stats)
            }
            Err(error) => {
                if !invalidated {
                    self.status = Status::Failed(error.status);
                }
                self.logs.push(LogMessage {
                    level: LogLevel::Error,
                    message: error.message,
                });
                None
            }
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Invalidates the result when a frontend changes the optimization inputs.
    /// An active job can finish and report progress, but its result is discarded.
    pub fn invalidate_result(&mut self) {
        self.output = None;
        if let Some(job) = &mut self.job {
            job.invalidated = true;
        } else {
            self.status = Status::Idle;
        }
    }

    pub fn is_running(&self) -> bool {
        self.job.is_some()
    }

    pub fn logs(&self) -> &[LogMessage] {
        &self.logs
    }

    pub fn copyable_logs(&self) -> String {
        self.logs
            .iter()
            .map(log_line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn can_save(&self) -> bool {
        self.output.is_some()
    }

    pub fn save_output(&self, path: &Path) -> io::Result<()> {
        let bytes = self
            .output
            .as_deref()
            .ok_or_else(|| io::Error::other("No optimized resource pack to save"))?;
        fs::write(path, bytes)
    }
}

fn optimize(
    input: PathBuf,
    cache: Option<PathBuf>,
    options: &Options,
    progress: watch::Sender<Progress>,
    logs: &tokio_mpsc::UnboundedSender<LogMessage>,
) -> Result<Output, Failure> {
    let started = Instant::now();
    let bytes = fs::read(&input).map_err(|error| Failure {
        status: "Error reading file",
        message: format!("{}: {error}", input.display()),
    })?;
    let input_bytes = bytes.len();
    let cache = cache
        .map(|path| {
            path.into_os_string().into_string().map_err(|path| Failure {
                status: "Processing error",
                message: format!("Cache path is not valid UTF-8: {}", path.to_string_lossy()),
            })
        })
        .transpose()?;

    // PackOBF uses process-wide identifier mappings and version state.
    static PROCESSING: Mutex<()> = Mutex::new(());
    let _processing = PROCESSING.lock().unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        packobf::process_zip(bytes, options, progress, logs, &cache)
    }));
    let bytes = result
        .map_err(|_| Failure {
            status: "Processing error",
            message: "A Rust panic occurred during optimization".into(),
        })?
        .map_err(|error| Failure {
            status: "Processing error",
            message: error.to_string(),
        })?;

    Ok(Output {
        stats: Statistics {
            elapsed: started.elapsed(),
            input_bytes,
            output_bytes: bytes.len(),
        },
        bytes,
    })
}

pub fn log_line(log: &LogMessage) -> String {
    let level = match log.level {
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARNING",
        LogLevel::Error => "ERROR",
    };
    format!("{level}: {}", log.message)
}

fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }

    match value {
        _ if unit == 0 => format!("{bytes} B"),
        ..10.0 => format!("{value:.2} {}", UNITS[unit]),
        ..100.0 => format!("{value:.1} {}", UNITS[unit]),
        _ => format!("{value:.0} {}", UNITS[unit]),
    }
}

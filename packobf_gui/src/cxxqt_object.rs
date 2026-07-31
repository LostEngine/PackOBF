#![allow(non_snake_case)]

use cxx_qt::Threading;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QByteArray, QString};
use packobf::{LogLevel, LogMessage, Progress};
use packobf::options::{Compression, Options, ShaderCompression};

/// The bridge definition for our QObject
#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qbytearray.h");
        /// An alias to the QString type
        type QString = cxx_qt_lib::QString;
        /// An alias to the QByteArray type
        type QByteArray = cxx_qt_lib::QByteArray;

    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, selected_file)]
        #[qproperty(i32, compression)]
        #[qproperty(i32, shader_compression)]
        #[qproperty(bool, rename_files)]
        #[qproperty(bool, block_unzipping)]
        #[qproperty(bool, corrupt_png_files)]
        #[qproperty(QString, cache_file_path)]

        #[qproperty(QString, progress_text)]
        #[qproperty(QString, logs_text)]

        #[qproperty(bool, show_info)]
        #[qproperty(bool, show_warning)]
        #[qproperty(bool, show_error)]

        #[qproperty(bool, processing)]
        #[qproperty(bool, done)]
        #[qproperty(QByteArray, output)]

        #[qproperty(bool, show_stats_popup)]
        #[qproperty(QString, stats_time)]
        #[qproperty(QString, stats_input)]
        #[qproperty(QString, stats_output)]
        #[qproperty(QString, stats_saved)]

        #[namespace = "app_controller"]
        type AppController = super::AppControllerRust;

        #[qinvokable]
        fn optimize(self: Pin<&mut AppController>);

        #[qinvokable]
        fn save_output(self: Pin<&mut AppController>, path: QString);

        #[qinvokable]
        fn copy_logs(self: Pin<&mut AppController>);

        #[qsignal]
        fn logsBatch(self: Pin<&mut AppController>, batch: QString);

        #[qsignal]
        fn logsCleared(self: Pin<&mut AppController>);
    }

    impl cxx_qt::Threading for AppController {}
}

/// The Rust struct for the QObject
pub struct AppControllerRust {
    selected_file: QString,
    compression: i32,
    shader_compression: i32,
    rename_files: bool,
    block_unzipping: bool,
    corrupt_png_files: bool,
    cache_file_path: QString,

    progress_text: QString,
    logs_text: QString,

    show_info: bool,
    show_warning: bool,
    show_error: bool,

    processing: bool,
    done: bool,
    output: QByteArray,

    show_stats_popup: bool,
    stats_time: QString,
    stats_input: QString,
    stats_output: QString,
    stats_saved: QString,

    pub(crate) internal_logs: Arc<Mutex<Vec<LogMessage>>>,
    pub(crate) tokio_runtime: tokio::runtime::Runtime,
}

impl Default for AppControllerRust {
    fn default() -> Self {
        Self {
            selected_file: QString::default(),
            compression: 1, // Normal
            shader_compression: 0, // None
            rename_files: true,
            block_unzipping: false,
            corrupt_png_files: true,
            cache_file_path: QString::default(),

            progress_text: QString::from("Idle"),
            logs_text: QString::default(),

            show_info: true,
            show_warning: true,
            show_error: true,

            processing: false,
            done: false,
            output: QByteArray::default(),

            show_stats_popup: false,
            stats_time: QString::default(),
            stats_input: QString::default(),
            stats_output: QString::default(),
            stats_saved: QString::default(),

            internal_logs: Arc::new(Mutex::new(Vec::new())),
            tokio_runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
}

pub fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} B", bytes)
    } else if value < 10.0 {
        format!("{:.2} {}", value, UNITS[unit])
    } else if value < 100.0 {
        format!("{:.1} {}", value, UNITS[unit])
    } else {
        format!("{:.0} {}", value, UNITS[unit])
    }
}

impl qobject::AppController {
    pub fn save_output(self: Pin<&mut Self>, path: QString) {
        let output = self.output();
        let bytes = output.as_slice();
        let _ = std::fs::write(path.to_string(), bytes);
    }

    pub fn copy_logs(self: Pin<&mut Self>) {
        let logs = self.rust().internal_logs.lock().unwrap();
        let plain_text = logs.iter().map(|l| format!("{}: {}", match l.level {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        }, l.message)).collect::<Vec<_>>().join("\n");

        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(plain_text);
        }
    }

    pub fn optimize(mut self: Pin<&mut Self>) {
        self.as_mut().set_processing(true);
        self.as_mut().set_done(false);
        self.as_mut().set_progress_text(QString::from("Optimizing..."));

        self.as_ref().rust().internal_logs.lock().unwrap().clear();
        self.as_mut().logsCleared();

        let file_path = self.selected_file().to_string();
        let cache_path = self.cache_file_path().to_string();
        let cache_opt = if cache_path.is_empty() { None } else { Some(cache_path) };

        let options = Options {
            compression: match self.compression() {
                0 => Compression::Fastest,
                1 => Compression::Fast,
                2 => Compression::Normal,
                _ => Compression::Best,
            },
            shader_compression: match self.shader_compression() {
                0 => ShaderCompression::None,
                1 => ShaderCompression::Minify,
                _ => ShaderCompression::MinifyAndObfuscate,
            },
            rename_files: *self.rename_files(),
            block_unzipping: *self.block_unzipping(),
            corrupt_png_files: *self.corrupt_png_files(),
            num_threads: None,
        };

        let qt_thread = self.qt_thread();
        let runtime_handle = self.rust().tokio_runtime.handle().clone();

        runtime_handle.spawn(async move {
            let started = std::time::Instant::now();
            let input_res = std::fs::read(&file_path);

            if let Err(e) = input_res {
                let error_msg = e.to_string();
                qt_thread.queue(move |mut qobject| {
                    qobject.as_mut().set_processing(false);
                    qobject.as_mut().set_progress_text(QString::from("Error reading file"));
                    qobject.as_ref().rust().internal_logs.lock().unwrap().push(LogMessage {
                        level: LogLevel::Error,
                        message: error_msg.clone(),
                    });
                    let batch_str = format!("2{}\x1E", error_msg);
                    qobject.as_mut().logsBatch(QString::from(&batch_str));
                }).unwrap();
                return;
            }

            let input = input_res.unwrap();
            let input_size = input.len();

            let (progress_tx, mut progress_rx) = tokio::sync::watch::channel(Progress::Idle);
            let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel::<LogMessage>();

            let qt_thread_prog = qt_thread.clone();
            tokio::spawn(async move {
                loop {
                    if progress_rx.changed().await.is_err() { break; }
                    let text = match progress_rx.borrow().clone() {
                        Progress::Idle => "Idle".to_string(),
                        Progress::ReadingZip { current, total } => format!("Reading ZIP ({}/{})", current, total),
                        Progress::Parsing { current } => format!("Parsing {}", current),
                        Progress::Optimizing { current, index, total } => format!("Optimizing ({}/{}) {}", index, total, current),
                        Progress::Building { current, index, total } => format!("Building ({}/{}) {}", index, total, current),
                        Progress::Done => "Done".to_string(),
                    };
                    let qt_text = QString::from(&text);
                    qt_thread_prog.queue(move |mut qobject| {
                        qobject.as_mut().set_progress_text(qt_text.clone());
                    }).unwrap();
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
            });

            let qt_thread_log = qt_thread.clone();
            tokio::spawn(async move {
                while let Some(first_log) = log_rx.recv().await {
                    // Collect the first log and immediately pull any others waiting in the queue
                    let mut batch = vec![first_log];
                    while let Ok(log) = log_rx.try_recv() {
                        batch.push(log);
                    }

                    qt_thread_log.queue(move |mut qobject| {
                        let mut joined = String::new();

                        {
                            let q_ref = qobject.as_ref();
                            let mut logs_lock = q_ref.internal_logs.lock().unwrap();

                            for log in batch {
                                let level_char = match log.level {
                                    LogLevel::Info => '0',
                                    LogLevel::Warning => '1',
                                    LogLevel::Error => '2',
                                };

                                // Append formatting elements to construct unified string to parse in QML
                                joined.push(level_char);
                                joined.push_str(&log.message);
                                joined.push('\x1E'); // ASCII Record Separator
                                logs_lock.push(log);
                            }
                        }

                        if !joined.is_empty() {
                            qobject.as_mut().logsBatch(QString::from(&joined));
                        }
                    }).unwrap();

                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            });

            let result = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    packobf::process_zip(input, &options, progress_tx, &log_tx, &cache_opt)
                }))
            }).await;

            let result = match result {
                Ok(inner) => match inner {
                    Ok(Ok(bytes)) => Ok(bytes),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(_) => Err("Rust panic occurred".to_string()),
                },
                Err(e) => Err(e.to_string()),
            };

            let duration = started.elapsed().as_secs_f32();

            qt_thread.queue(move |mut qobject| {
                match result {
                    Ok(bytes) => {
                        let output_size = bytes.len();
                        let saved_bytes = input_size as isize - output_size as isize;
                        let percent = 100.0 - (output_size as f64 / input_size as f64 * 100.0);

                        qobject.as_mut().set_output(QByteArray::from(bytes.as_slice()));
                        qobject.as_mut().set_progress_text(QString::from(&format!("Done {:.3}s", duration)));

                        qobject.as_mut().set_stats_time(QString::from(&format!("{:.3}s", duration)));
                        qobject.as_mut().set_stats_input(QString::from(&format_bytes(input_size)));
                        qobject.as_mut().set_stats_output(QString::from(&format_bytes(output_size)));
                        qobject.as_mut().set_stats_saved(QString::from(&format!("{} ({:.2}%)", format_bytes(saved_bytes.max(0) as usize), percent)));

                        qobject.as_mut().set_show_stats_popup(true);
                    }
                    Err(err) => {
                        qobject.as_ref().rust().internal_logs.lock().unwrap().push(LogMessage {
                            level: LogLevel::Error,
                            message: err.clone(),
                        });
                        let batch_str = format!("2{}\x1E", err);
                        qobject.as_mut().logsBatch(QString::from(&batch_str));
                        qobject.as_mut().set_progress_text(QString::from("Processing error"));
                    }
                }
                qobject.as_mut().set_processing(false);
                qobject.as_mut().set_done(true);
            }).unwrap();
        });
    }

}

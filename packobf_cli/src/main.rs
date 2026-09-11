extern crate core;

use clap::{Parser, ValueEnum};
use console::{Emoji, style};
use indicatif::{DecimalBytes, ProgressBar, ProgressStyle};
use packobf::options::{Options, Preset};
use packobf::{LogLevel, LogMessage, Progress, process_zip};
use std::time::Instant;
use tokio::sync::watch;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    input_file: String,
    #[arg(short, long, value_name = "FILE")]
    output_file: String,

    #[arg(short, long, value_enum, conflicts_with = "options")]
    preset: Option<Preset>,

    #[arg(short, long, default_value = "info")]
    log_level: LogFilter,

    #[arg(long)]
    cache_file: Option<String>,

    #[command(flatten)]
    pub options: Options,
}

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚 ", "");
static CLIP: Emoji<'_, '_> = Emoji("🔗 ", "");
static OPTIMIZING: Emoji<'_, '_> = Emoji("🚀 ", "");
static BUILDING: Emoji<'_, '_> = Emoji("⚒️ ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");
static CHECK: Emoji<'_, '_> = Emoji("✅ ", "OK ");

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let input_bytes = std::fs::read(args.input_file).unwrap();
    let input_size = input_bytes.len();

    let options = match args.preset {
        Some(preset) => Options::from_preset(preset),
        None => args.options,
    };

    let (log_tx, log_rx) = tokio::sync::mpsc::unbounded_channel::<LogMessage>();
    let (tx, rx) = watch::channel(Progress::Idle);
    tokio::spawn(async move { run_progress_loop(rx, log_rx, &args.log_level).await });
    let bytes = process_zip(input_bytes, &options, tx, &log_tx, &args.cache_file).unwrap();
    let ratio = bytes.len() as f64 / input_size as f64;

    println!();
    println!(
        "Zip file size: {} /{}, ({}/{:.1}% {})",
        DecimalBytes(bytes.len() as u64),
        DecimalBytes(input_size as u64),
        DecimalBytes(bytes.len().abs_diff(input_size) as u64),
        (100.0 - ratio * 100.0).abs(),
        if bytes.len() < input_size {
            "smaller"
        } else {
            "larger" // This should never happen
        }
    );
    std::fs::write(args.output_file, bytes).unwrap();
}

pub async fn run_progress_loop(
    mut rx: watch::Receiver<Progress>,
    mut log_rx: tokio::sync::mpsc::UnboundedReceiver<LogMessage>,
    log_filter: &LogFilter,
) {
    let global_started = Instant::now();
    let mut stage_started = Instant::now();
    let mut current_pb: Option<ProgressBar> = None;
    // 1: Idle, 2: Reading, 3: Parsing, 4: Optimizing, 5: Building
    let mut current_stage: u8 = 0;

    let clear_current = |pb: &mut Option<ProgressBar>| {
        if let Some(p) = pb.take() {
            p.finish_and_clear();
        }
    };

    let print_finished_stage = |pb: &mut Option<ProgressBar>, stage_name: &str, start: Instant| {
        if let Some(p) = pb.take() {
            p.finish_and_clear();
        }
        println!(
            "\r{} {} {} {:.3}s",
            style(" ").bold().dim(),
            CHECK,
            style(stage_name).dim(),
            start.elapsed().as_secs_f64()
        );
    };

    let bar_style = ProgressStyle::with_template(
        "{prefix:.bold.dim} {spinner} {wide_msg} [{bar:40.cyan/blue}] {pos}/{len}",
    )
    .unwrap()
    .progress_chars("=> ");

    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    loop {
        tokio::select! {
            Some(log) = log_rx.recv() => {
                if !log_filter.is_valid(&log.level) {
                    continue;
                }
                let msg = match log.level {
                    LogLevel::Info => style(format!("ℹ {}", log.message)).cyan(),
                    LogLevel::Warning => style(format!("⚠ Warning: {}", log.message)).color256(208), // Orange
                    LogLevel::Error => style(format!("✘ Error: {}", log.message)).red().bold(),
                };

                if let Some(ref pb) = current_pb {
                    pb.println(format!("{}", msg));
                } else {
                    eprintln!("{}", msg);
                }
            }

            Ok(_) = rx.changed() => {
                let state = rx.borrow().clone();

        match state {
            Progress::Idle => {
                if current_stage != 1 {
                    println!(
                        "{} {} Initializing...",
                        style("[1/5]").bold().dim(),
                        LOOKING_GLASS
                    );
                    current_stage = 1;
                    stage_started = Instant::now();
                }
            }

            Progress::ReadingZip { current, total } => {
                if current_stage != 2 {
                    print_finished_stage(&mut current_pb, "Initialized", stage_started);
                    clear_current(&mut current_pb);
                    println!(
                        "{} {} Reading archive...",
                        style("[2/5]").bold().dim(),
                        TRUCK
                    );
                    current_stage = 2;
                    stage_started = Instant::now();
                }

                let pb = current_pb.get_or_insert_with(|| {
                    let p = ProgressBar::new(total as u64);
                    p.set_style(bar_style.clone());
                    p.set_prefix("[2/5]");
                    p
                });
                if pb.position() < current as u64 {
                    pb.set_position(current as u64);
                }
                pb.set_message("Unzipping files");
            }

            Progress::Parsing { current } => {
                if current_stage != 3 {
                    print_finished_stage(&mut current_pb, "Archive Read", stage_started);

                    clear_current(&mut current_pb);
                    println!(
                        "{} {} Parsing resource files...",
                        style("[3/5]").bold().dim(),
                        CLIP
                    );
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(spinner_style.clone());
                    pb.set_prefix("[3/5]");
                    current_pb = Some(pb);
                    current_stage = 3;
                    stage_started = Instant::now();
                }

                if let Some(ref pb) = current_pb {
                    pb.set_message(format!("Analyzing {}", current));
                    pb.tick();
                }
            }

            Progress::Optimizing {
                current,
                index,
                total,
            } => {
                if current_stage != 4 {
                    print_finished_stage(&mut current_pb, "Parsing Complete", stage_started);

                    clear_current(&mut current_pb);
                    println!(
                        "{} {} Optimizing resource pack...",
                        style("[4/5]").bold().dim(),
                        OPTIMIZING
                    );
                    current_stage = 4;
                    stage_started = Instant::now();
                }

                let pb = current_pb.get_or_insert_with(|| {
                    let p = ProgressBar::new(total as u64);
                    p.set_style(bar_style.clone());
                    p.set_prefix("[4/5]");
                    p
                });
                if pb.position() < index as u64 {
                    pb.set_position(index as u64);
                }
                pb.set_message(format!("File: {}", current));
            }

            Progress::Building {
                current,
                index,
                total,
            } => {
                if current_stage != 5 {
                    print_finished_stage(&mut current_pb, "Optimizing Complete", stage_started);

                    clear_current(&mut current_pb);
                    println!(
                        "{} {} Building resource pack...",
                        style("[5/5]").bold().dim(),
                        BUILDING
                    );
                    current_stage = 5;
                    stage_started = Instant::now();
                }

                let pb = current_pb.get_or_insert_with(|| {
                    let p = ProgressBar::new(total as u64);
                    p.set_style(bar_style.clone());
                    p.set_prefix("[5/5]");
                    p
                });
                if pb.position() < index as u64 {
                    pb.set_position(index as u64);
                }
                pb.set_message(format!("File: {}", current));
            }

            Progress::Done => {
                print_finished_stage(&mut current_pb, "Resource pack built", stage_started);
                clear_current(&mut current_pb);
                println!(
                    "{} Done in {:.3}s",
                    SPARKLE,
                    global_started.elapsed().as_secs_f64()
                );
                break;
            }
        }
            }
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogFilter {
    Info = 0,
    Warning = 1,
    Error = 2,
    None = 3,
}

impl LogFilter {
    pub fn is_valid(&self, log_level: &LogLevel) -> bool {
        (log_level.clone() as u8) >= (self.clone() as u8)
    }
}

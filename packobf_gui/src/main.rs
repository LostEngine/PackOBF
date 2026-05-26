use std::{
    fs,
    num::NonZeroU32,
    sync::{Arc, Mutex},
    time::Instant,
};

use arboard::Clipboard;
use glow::HasContext;
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use imgui::*;
use imgui_winit_support::{
    winit::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::{Window, WindowAttributes},
    },
    HiDpiMode,
    WinitPlatform,
};
use packobf::{
    options::{Compression, Options, ShaderCompression},
    LogLevel, LogMessage, Progress,
};
use raw_window_handle::HasWindowHandle;
use rfd::FileDialog;
use tokio::sync::{mpsc, watch};

const TITLE: &str = "packobf";

#[derive(Clone)]
struct AppState {
    selected_file: String,

    compression: Compression,
    shader_compression: ShaderCompression,

    rename_files: bool,
    block_unzipping: bool,
    corrupt_png_files: bool,

    cache_file_path: String,

    progress_text: String,
    logs: Vec<LogMessage>,

    show_info: bool,
    show_warning: bool,
    show_error: bool,

    processing: bool,
    done: bool,
    stats: Option<OptimizationStats>,
    show_stats_popup: bool,
    output: Option<Vec<u8>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            selected_file: "".to_string(),

            compression: Compression::Normal,
            shader_compression: ShaderCompression::None,

            rename_files: true,
            block_unzipping: false,
            corrupt_png_files: true,

            cache_file_path: "".to_string(),

            progress_text: "Idle".to_string(),
            logs: vec![],

            show_info: true,
            show_warning: true,
            show_error: true,

            processing: false,
            done: false,
            stats: None,
            show_stats_popup: false,
            output: None,
        }
    }
}

#[derive(Clone, Default)]
struct OptimizationStats {
    duration: f32,
    input_size: usize,
    output_size: usize,
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let state = Arc::new(Mutex::new(AppState::default()));

    let (event_loop, window, surface, context) = create_window();
    let (mut platform, mut imgui) = imgui_init(&window);

    let gl = glow_context(&context);

    let mut renderer =
        imgui_glow_renderer::AutoRenderer::new(gl, &mut imgui).unwrap();

    let mut last_frame = Instant::now();

    #[allow(deprecated)]
    event_loop
        .run(move |event, window_target| {
            platform.handle_event(imgui.io_mut(), &window, &event);

            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();

                    imgui
                        .io_mut()
                        .update_delta_time(now.duration_since(last_frame));

                    last_frame = now;
                }

                Event::AboutToWait => {
                    platform.prepare_frame(imgui.io_mut(), &window).unwrap();

                    window.request_redraw();
                }

                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    unsafe {
                        renderer
                            .gl_context()
                            .clear_color(0.1, 0.1, 0.1, 1.0);

                        renderer
                            .gl_context()
                            .clear(glow::COLOR_BUFFER_BIT);
                    }

                    let ui = imgui.frame();

                    draw_ui(ui, state.clone(), runtime.handle().clone());

                    platform.prepare_render(ui, &window);

                    let draw_data = imgui.render();

                    renderer.render(draw_data).unwrap();

                    surface.swap_buffers(&context).unwrap();
                }

                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    window_target.exit();
                }

                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    if new_size.width > 0 && new_size.height > 0 {
                        surface.resize(
                            &context,
                            NonZeroU32::new(new_size.width).unwrap(),
                            NonZeroU32::new(new_size.height).unwrap(),
                        );
                    }
                }

                _ => {}
            }
        })
        .unwrap();
}

fn draw_ui(
    ui: &Ui,
    state: Arc<Mutex<AppState>>,
    runtime: tokio::runtime::Handle,
) {
    let mut app = state.lock().unwrap();

    let window_size = ui.io().display_size;

    ui.window("packobf")
        .position([0.0, 0.0], Condition::Always)
        .size(window_size, Condition::Always)
        .movable(false)
        .resizable(false)
        .title_bar(false)
        .build(|| {
            ui.text("An open-source Minecraft: Java Edition resource pack minimizer, obfuscator and checker.");

            ui.separator();

            if app.show_stats_popup {
                ui.open_popup("Optimization Complete");
                app.show_stats_popup = false;
            }

            if let Some(stats) = &app.stats {
                ui.modal_popup("Optimization Complete", || {
                    let saved_bytes = stats.input_size as isize - stats.output_size as isize;
                    let percent = 100.0 - (stats.output_size as f64 / stats.input_size as f64 * 100.0);

                    ui.text(format!("Time taken: {:.3}s", stats.duration));

                    ui.text(format!(
                        "Input size: {}",
                        format_bytes(stats.input_size)
                    ));
                    ui.text(format!(
                        "Output size: {}",
                        format_bytes(stats.output_size)
                    ));

                    ui.text(format!(
                        "Saved: {} ({:.2}%)",
                        format_bytes(saved_bytes.max(0) as usize),
                        percent
                    ));

                    ui.separator();

                    if ui.button("OK") {
                        ui.close_current_popup();
                    }
                });
            }

            ui.text("Resource pack file");
            ui.same_line();
            if ui.button("Select file") {
                if let Some(path) = FileDialog::new()
                    .add_filter("Zip", &["zip"])
                    .pick_file()
                {
                    app.selected_file = path.display().to_string();
                }
            }

            ui.input_text("##selected_file", &mut app.selected_file).build();

            ui.separator();

            ui.text("Cache file (Optional)");
            ui.same_line();
            if ui.button("Import cache file") {
                if let Some(path) = FileDialog::new()
                    .add_filter("Bin", &["bin"])
                    .pick_file()
                {
                    app.cache_file_path = path.display().to_string();
                }
            }
            ui.same_line();
            if ui.button("Create cache file") {
                if let Some(path) = FileDialog::new()
                    .add_filter("Bin", &["bin"])
                    .save_file()
                {
                    app.cache_file_path = path.display().to_string();
                }
            }
            ui.input_text("##cache_file", &mut app.cache_file_path).build();

            ui.separator();

            ui.text("Compression");
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("Global compression level for the resource pack. (Applies for textures, sounds, and every file when building the ZIP file).");
                });
            }

            let mut compression_index = match app.compression {
                Compression::Simplest => 0,
                Compression::Normal => 1,
                Compression::Max => 2,
            };

            let compression_items =
                ["Simplest", "Normal", "Max"];

            ui.same_line();
            ui.set_next_item_width(180.0);
            ui.combo_simple_string(
                "##compression",
                &mut compression_index,
                &compression_items,
            );
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("Global compression level for the resource pack. (Applies for textures, sounds, and every file when building the ZIP file).");
                });
            }

            app.compression = match compression_index {
                0 => Compression::Simplest,
                1 => Compression::Normal,
                _ => Compression::Max,
            };

            ui.separator();

            ui.text("Shader compression (Experimental)");
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("Parses GLSL core shaders to minify and obfuscate them. (Experimental)");
                });
            }

            let mut shader_index = match app.shader_compression {
                ShaderCompression::None => 0,
                ShaderCompression::Minify => 1,
                ShaderCompression::MinifyAndObfuscate => 2,
            };

            let shader_items = [
                "None",
                "Minify",
                "Minify and obfuscate",
            ];

            ui.same_line();
            ui.set_next_item_width(180.0);
            ui.combo_simple_string(
                "##shader",
                &mut shader_index,
                &shader_items,
            );
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("Parses GLSL core shaders to minify and obfuscate them. (Experimental)");
                });
            }

            app.shader_compression = match shader_index {
                0 => ShaderCompression::None,
                1 => ShaderCompression::Minify,
                _ => ShaderCompression::MinifyAndObfuscate,
            };

            ui.separator();

            ui.checkbox("Rename files", &mut app.rename_files);

            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("Renames textures, models, and sounds to shorter names while keeping the resource pack working.");
                });
            }

            ui.same_line();
            ui.checkbox(
                "Block resource pack unzipping",
                &mut app.block_unzipping,
            );
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("Adds some bytes to the resource pack to prevent files from being extracted on a file system.");
                });
            }

            ui.same_line();
            ui.checkbox(
                "Corrupt PNG files",
                &mut app.corrupt_png_files,
            );
            if ui.is_item_hovered() {
                ui.tooltip(|| {
                    ui.text("Corrupts PNG files in a way that makes them unreadable for most software except for Minecraft. (Allows removing 4 bytes from each PNG file).");
                });
            }

            ui.separator();

            if !app.processing {
                if ui.button("Optimize") {
                    if !app.selected_file.is_empty() {
                        app.processing = true;
                        app.done = false;
                        app.logs.clear();

                        let file_to_process = app.selected_file.clone();
                        let state_clone = state.clone();
                        let cache = if app.cache_file_path.is_empty() {
                            None
                        } else {
                            Some(app.cache_file_path.clone())
                        };

                        runtime.spawn(async move {
                            run_packobf(file_to_process, cache, state_clone).await;
                        });
                    }
                }
            } else {
                ui.text("Processing...");
            }

            if app.done {
                ui.same_line();

                if ui.button("Save file") {
                    if let Some(bytes) = &app.output {
                        if let Some(path) = FileDialog::new()
                            .set_file_name("resourcepack-optimized.zip")
                            .save_file()
                        {
                            let _ = fs::write(path, bytes);
                        }
                    }
                }
            }

            ui.separator();

            ui.text(format!("Status: {}", app.progress_text));

            ui.separator();

            let width = ui.window_size()[0];

            ui.text("Logs");

            ui.same_line();
            ui.checkbox("Info", &mut app.show_info);
            ui.same_line();
            ui.checkbox("Warning", &mut app.show_warning);
            ui.same_line();
            ui.checkbox("Error", &mut app.show_error);

            ui.same_line_with_pos(width - 45.0);
            if ui.button("Copy") {
                let log_text: String = app
                    .logs
                    .iter()
                    .map(|l| format_log(l).to_string())
                    .collect::<Vec<_>>()
                    .join("\n");

                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(log_text).unwrap();
            }

            ui.child_window("logs")
                .size([0.0, 250.0])
                .build(|| {
                    let at_bottom = ui.scroll_y() >= ui.scroll_max_y() - 5.0;

                    for log in &app.logs {
                        match log.level {
                            LogLevel::Info => {
                                if !app.show_info {
                                    continue;
                                }
                                ui.text_colored(
                                    [0.3, 0.7, 1.0, 1.0],
                                    format_log(log),
                                );
                            }
                            LogLevel::Warning => {
                                if !app.show_warning {
                                    continue;
                                }
                                ui.text_colored(
                                    [1.0, 0.8, 0.2, 1.0],
                                    format_log(log),
                                );
                            }
                            LogLevel::Error => {
                                if !app.show_error {
                                    continue;
                                }
                                ui.text_colored(
                                    [1.0, 0.2, 0.2, 1.0],
                                    format_log(log),
                                );
                            }
                        }
                    }

                    if at_bottom {
                        ui.set_scroll_here_y();
                    }
                });
        });
}

async fn run_packobf(
    path: String,
    cache: Option<String>,
    state: Arc<Mutex<AppState>>,
) {
    let started = Instant::now();
    let input = fs::read(&path).unwrap();
    let input_size = input.len() as usize;

    let options = {
        let app = state.lock().unwrap();

        Options {
            compression: app.compression.clone(),
            shader_compression: app.shader_compression.clone(),
            rename_files: app.rename_files,
            block_unzipping: app.block_unzipping,
            corrupt_png_files: app.corrupt_png_files,
        }
    };

    let (progress_tx, mut progress_rx) =
        watch::channel(Progress::Idle);

    let (log_tx, mut log_rx) =
        mpsc::unbounded_channel::<LogMessage>();

    let state_progress = state.clone();

    tokio::spawn(async move {
        loop {
            if progress_rx.changed().await.is_err() {
                break;
            }

            let text = match progress_rx.borrow().clone() {
                Progress::Idle => "Idle".to_string(),

                Progress::ReadingZip { current, total } => {
                    format!("Reading ZIP ({}/{})", current, total)
                }

                Progress::Parsing { current } => {
                    format!("Parsing {}", current)
                }

                Progress::Building {
                    current,
                    index,
                    total,
                } => {
                    format!(
                        "Building ({}/{}) {}",
                        index,
                        total,
                        current
                    )
                }

                Progress::Done => "Done".to_string(),
            };

            state_progress.lock().unwrap().progress_text = text;
        }
    });

    let state_logs = state.clone();

    tokio::spawn(async move {
        while let Some(log) = log_rx.recv().await {
            state_logs.lock().unwrap().logs.push(log);
        }
    });

    let result = tokio::task::spawn_blocking(move || {
        packobf::process_zip(
            input,
            &options,
            progress_tx,
            &log_tx,
            &cache,
        )
    })
    .await
    .unwrap();

    let mut app = state.lock().unwrap();

    match result {
        Ok(bytes) => {
            let duration = started.elapsed().as_secs_f32();
            let output_size = bytes.len();

            app.output = Some(bytes);
            app.progress_text = format!("Done {:.3}s", started.elapsed().as_secs_f32());

            app.stats = Some(OptimizationStats {
                    duration,
                    input_size,
                    output_size,
                });

            app.show_stats_popup = true;
        }

        Err(err) => {
            app.logs.push(LogMessage {
                level: LogLevel::Error,
                message: err.to_string(),
            });

            app.progress_text = "Error".to_string();
        }
    }

    app.processing = false;
    app.done = true;
}

fn create_window() -> (
    EventLoop<()>,
    Window,
    Surface<WindowSurface>,
    PossiblyCurrentContext,
) {
    let event_loop = EventLoop::new().unwrap();

    let window_attributes = WindowAttributes::default()
        .with_title(TITLE)
        .with_inner_size(LogicalSize::new(1280, 720));

    let (window, cfg) = glutin_winit::DisplayBuilder::new()
        .with_window_attributes(Some(window_attributes))
        .build(
            &event_loop,
            ConfigTemplateBuilder::new(),
            |mut configs| configs.next().unwrap(),
        )
        .unwrap();

    let window = window.unwrap();

    let context_attributes =
        ContextAttributesBuilder::new()
            .build(Some(window.window_handle().unwrap().as_raw()));

    let context = unsafe {
        cfg.display()
            .create_context(&cfg, &context_attributes)
            .unwrap()
    };

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
        .build(
            window.window_handle().unwrap().as_raw(),
            NonZeroU32::new(1280).unwrap(),
            NonZeroU32::new(720).unwrap(),
        );

    let surface = unsafe {
        cfg.display()
            .create_window_surface(&cfg, &attrs)
            .unwrap()
    };

    let context = context.make_current(&surface).unwrap();

    (event_loop, window, surface, context)
}

fn glow_context(
    context: &PossiblyCurrentContext,
) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function_cstr(|s| {
            context.display().get_proc_address(s).cast()
        })
    }
}

fn imgui_init(window: &Window) -> (WinitPlatform, imgui::Context) {
    let mut imgui = imgui::Context::create();

    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::new(&mut imgui);

    platform.attach_window(
        imgui.io_mut(),
        window,
        HiDpiMode::Rounded,
    );

    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: None,
        },
    ]);

    imgui.io_mut().font_global_scale =
        (1.0 / platform.hidpi_factor()) as f32;

    (platform, imgui)
}

fn format_log(log_message: &LogMessage) -> String {
    format!("{}: {}", match log_message.level {
        LogLevel::Info => "INFO",
        LogLevel::Warning => "WARNING",
        LogLevel::Error => "ERROR",
    }, log_message.message)
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

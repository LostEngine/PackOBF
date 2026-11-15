//! C ABI for desktop frontends. Ownership and calling conventions are in
//! `include/packobf_gui.h`.

use std::ffi::{CStr, CString, c_char};
use std::path::PathBuf;
use std::ptr;

use crate::app::{
    App, Compression, LogLevel, MinecraftVersion, Options, ShaderCompression, SoundQuality,
    default_options, log_line,
};

#[repr(C)]
pub struct Settings {
    compression: u8,
    shader_compression: u8,
    sound_quality: u8,
    target_version: u8,
    rename_files: u8,
    block_unzipping: u8,
    corrupt_png_files: u8,
    worker_threads: u32,
}

impl Settings {
    fn options(&self) -> Result<Options, &'static str> {
        use SoundQuality::*;

        let compression = match self.compression {
            0 => Compression::Fast,
            1 => Compression::Normal,
            _ => return Err("Invalid compression"),
        };
        let shader_compression = match self.shader_compression {
            0 => ShaderCompression::None,
            1 => ShaderCompression::Minify,
            2 => ShaderCompression::MinifyAndObfuscate,
            _ => return Err("Invalid shader compression"),
        };
        let sound_quality = [
            Dynamic, MinusOne, Zero, One, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten,
        ]
        .get(usize::from(self.sound_quality))
        .ok_or("Invalid sound quality")?
        .clone();
        let target_version = match self.target_version {
            0 => None,
            version => Some(MinecraftVersion::from_u8(version).ok_or("Invalid Minecraft version")?),
        };
        if self.worker_threads > 256 {
            return Err("Worker threads must be between 0 and 256");
        }

        Ok(Options {
            compression,
            shader_compression,
            sound_quality,
            target_version,
            rename_files: self.rename_files != 0,
            block_unzipping: self.block_unzipping != 0,
            corrupt_png_files: self.corrupt_png_files != 0,
            num_threads: (self.worker_threads > 0).then_some(self.worker_threads as usize),
        })
    }
}

#[repr(C)]
pub struct Snapshot {
    status: *mut c_char,
    summary: *mut c_char,
    log_count: usize,
    running: u8,
    can_save: u8,
}

#[repr(C)]
pub struct LogEntry {
    line: *mut c_char,
    level: u8,
}

#[unsafe(no_mangle)]
pub extern "C" fn packobf_gui_defaults() -> Settings {
    let options = default_options();
    Settings {
        compression: options.compression as u8,
        shader_compression: options.shader_compression as u8,
        sound_quality: options.sound_quality as u8,
        target_version: options.target_version.map_or(0, |version| version as u8),
        rename_files: u8::from(options.rename_files),
        block_unzipping: u8::from(options.block_unzipping),
        corrupt_png_files: u8::from(options.corrupt_png_files),
        worker_threads: options.num_threads.unwrap_or(0) as u32,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn packobf_gui_create() -> *mut App {
    Box::into_raw(Box::new(App::default()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn packobf_gui_destroy(app: *mut App) {
    if !app.is_null() {
        drop(unsafe { Box::from_raw(app) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn packobf_gui_start(
    app: &mut App,
    input: *const c_char,
    cache: *const c_char,
    settings: Settings,
) -> *mut c_char {
    let result = (|| {
        let input = unsafe { path(input) }?;
        let cache = if cache.is_null() {
            None
        } else {
            Some(unsafe { path(cache) }?)
        };
        let options = settings.options()?;
        app.start(input, cache, options)
            .map_err(|error| error.to_string())
    })();
    ffi_error(result)
}

#[unsafe(no_mangle)]
pub extern "C" fn packobf_gui_invalidate(app: &mut App) {
    app.invalidate_result();
}

#[unsafe(no_mangle)]
pub extern "C" fn packobf_gui_poll(app: &mut App) -> Snapshot {
    let summary = app.poll().map_or(ptr::null_mut(), |stats| {
        ffi_string(format!(
            "Time taken: {:.3}s\nInput size: {}\nOutput size: {}\nSaved: {}",
            stats.elapsed.as_secs_f64(),
            stats.input_size(),
            stats.output_size(),
            stats.saved()
        ))
    });
    Snapshot {
        status: ffi_string(app.status().to_string()),
        summary,
        log_count: app.logs().len(),
        running: u8::from(app.is_running()),
        can_save: u8::from(app.can_save()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn packobf_gui_log(app: &App, index: usize) -> LogEntry {
    match app.logs().get(index) {
        Some(log) => LogEntry {
            line: ffi_string(log_line(log)),
            level: match log.level {
                LogLevel::Info => 0,
                LogLevel::Warning => 1,
                LogLevel::Error => 2,
            },
        },
        None => LogEntry {
            line: ptr::null_mut(),
            level: 0,
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn packobf_gui_copyable_logs(app: &App) -> *mut c_char {
    ffi_string(app.copyable_logs())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn packobf_gui_save(app: &App, output: *const c_char) -> *mut c_char {
    ffi_error(
        unsafe { path(output) }
            .and_then(|output| app.save_output(&output).map_err(|error| error.to_string())),
    )
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn packobf_gui_free_string(string: *mut c_char) {
    if !string.is_null() {
        drop(unsafe { CString::from_raw(string) });
    }
}

unsafe fn path(value: *const c_char) -> Result<PathBuf, String> {
    if value.is_null() {
        return Err("A file path is required".into());
    }

    let text = unsafe { CStr::from_ptr(value) }
        .to_str()
        .map_err(|_| "The file path is not valid UTF-8")?;
    if text.is_empty() {
        return Err("A file path is required".into());
    }
    Ok(PathBuf::from(text))
}

fn ffi_string(value: String) -> *mut c_char {
    CString::new(value.replace('\0', "␀")).unwrap().into_raw()
}

fn ffi_error(result: Result<(), String>) -> *mut c_char {
    result.err().map_or(ptr::null_mut(), ffi_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_preserve_gui_defaults_and_decode_frontend_choices() {
        let mut settings = packobf_gui_defaults();
        let options = settings.options().unwrap();
        assert!(matches!(options.compression, Compression::Normal));
        assert_eq!(options.shader_compression, ShaderCompression::None);
        assert_eq!(options.sound_quality, SoundQuality::Dynamic);
        assert_eq!(options.target_version, None);
        assert_eq!(options.num_threads, None);
        assert!(options.rename_files && options.corrupt_png_files && !options.block_unzipping);

        settings.compression = 0;
        settings.shader_compression = 2;
        settings.sound_quality = 12;
        settings.target_version = 97;
        settings.worker_threads = 256;
        let options = settings.options().unwrap();
        assert!(matches!(options.compression, Compression::Fast));
        assert_eq!(
            options.shader_compression,
            ShaderCompression::MinifyAndObfuscate
        );
        assert_eq!(options.sound_quality, SoundQuality::Ten);
        assert_eq!(options.target_version, Some(MinecraftVersion::V26_3));
        assert_eq!(options.num_threads, Some(256));

        settings.worker_threads = 257;
        assert!(settings.options().is_err());
    }

    #[test]
    fn start_reports_invalid_utf8_without_starting_a_job() {
        let mut app = App::default();
        let error = unsafe {
            packobf_gui_start(
                &mut app,
                c"\xff".as_ptr(),
                ptr::null(),
                packobf_gui_defaults(),
            )
        };
        assert!(!error.is_null());
        let message = unsafe { CStr::from_ptr(error) }
            .to_string_lossy()
            .into_owned();
        unsafe { packobf_gui_free_string(error) };
        assert_eq!(message, "The file path is not valid UTF-8");
        assert!(!app.is_running());
    }
}

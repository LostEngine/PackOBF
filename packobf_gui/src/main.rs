#[cfg(target_os = "linux")]
#[path = "../platforms/linux/mod.rs"]
mod linux;

#[cfg(target_os = "linux")]
fn main() -> gtk::glib::ExitCode {
    linux::run()
}

#[cfg(target_os = "windows")]
fn main() -> std::process::ExitCode {
    eprintln!(
        "Build the WinUI 3 frontend in packobf_gui/platforms/windows. See packobf_gui/README.md."
    );
    std::process::ExitCode::FAILURE
}

#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    eprintln!(
        "Build the SwiftUI frontend with bash packobf_gui/platforms/macos/build.sh. See packobf_gui/README.md."
    );
    std::process::ExitCode::FAILURE
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn main() -> std::process::ExitCode {
    eprintln!("PackOBF GUI currently supports Linux, Windows, and macOS.");
    std::process::ExitCode::FAILURE
}

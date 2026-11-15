# PackOBF Desktop

PackOBF Desktop processes Minecraft Java Edition resource packs through the workspace's `packobf` library. The Linux frontend uses GTK 4 and libadwaita. The Windows frontend uses WinUI 3, C#, and XAML. The macOS frontend uses SwiftUI and Swift.

## Build and run on macOS

Install the workspace Rust toolchain and Xcode or the Xcode Command Line Tools with Swift 6 and the macOS 15 SDK or later. The app requires macOS 15 or later. The build uses the architecture of the active Rust toolchain on either Apple Silicon or Intel Macs.

From the repository root:

```sh
bash packobf_gui/platforms/macos/build.sh
open packobf_gui/target/macos/"$(uname -m)"/debug/PackOBF.app
bash packobf_gui/platforms/macos/build.sh test
```

The test command builds the app, runs the shared Rust GUI tests, and exercises the Swift-to-Rust optimization, cache, log, and save workflow. To build an optimized app, run `bash packobf_gui/platforms/macos/build.sh release` and look for the bundle under `packobf_gui/target/macos/<arch>/release/`. The Rust library is embedded in the app bundle. The build signs the bundle ad hoc for local use. Distribution outside the build Mac requires separate distribution signing and notarization.

## Build and run on Linux

Install the workspace Rust toolchain, a C compiler, `pkg-config`, GTK 4.10 or later, and libadwaita 1.5 or later. On distributions that split development packages, install the headers and pkg-config metadata too. For example, on Ubuntu 24.04:

```sh
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev
```

From the repository root:

```sh
cargo build -p packobf_gui --release
cargo run -p packobf_gui --release
cargo test -p packobf_gui
```

The Linux release archive contains the executable and license. The executable is built on Ubuntu 24.04 and links to system GTK and libadwaita libraries. On Ubuntu 24.04, install the runtime packages with `sudo apt install libgtk-4-1 libadwaita-1-0`.

## Build and run on Windows

Install the [.NET 10 SDK](https://dotnet.microsoft.com/download/dotnet/10.0), the Windows 11 SDK (10.0.26100.0), and the Visual Studio C++ build tools with MSVC. Install Rust with the matching MSVC target through rustup. For ARM64, also install the MSVC ARM64 build tools and run `rustup target add aarch64-pc-windows-msvc`.

From the repository root in PowerShell:

```powershell
dotnet build packobf_gui/platforms/windows/PackOBF.Windows.csproj -p:Platform=x64
.\packobf_gui\platforms\windows\bin\x64\Debug\net10.0-windows10.0.26100.0\win-x64\PackOBF.exe
cargo test --locked -p packobf_gui --features ffi --target-dir packobf_gui/target
```

For distribution, publish the unpackaged, self-contained app and ship the whole output folder:

```powershell
dotnet publish packobf_gui/platforms/windows/PackOBF.Windows.csproj -c Release -p:Platform=x64 -o packobf_gui/target/publish/win-x64
```

For ARM64, use `-p:Platform=ARM64` and the `win-arm64` output. MSBuild invokes Cargo with the matching Rust target and copies `packobf_gui.dll` next to `PackOBF.exe`. The published folder includes the .NET and Windows App SDK runtimes. Windows 10 version 1809 or later is required.

## Desktop workflow

Choose a ZIP or enter its path. To reuse an optimization cache, import a BIN file or choose where to create one. The cache is written by the optimizer. **Advanced options** exposes the worker count, where zero lets PackOBF choose.

**Optimize** starts a background job. **Show logs** exposes severity filters and **Copy** copies the full log. After a successful run, dismiss the size and timing summary and use **Save file** to write the optimized ZIP.

## Frontend layout

- `src/app.rs` provides the shared Rust API. A frontend starts a job with paths and typed PackOBF options, calls `App::poll` from its event loop while the job runs, then reads status, logs, and statistics. The latest successful output can be saved until the next run starts or the frontend invalidates it after an input or option changes.
- `platforms/linux/` contains widgets, file choosers, and clipboard integration. GTK dependencies are scoped to Linux in `Cargo.toml`.
- `platforms/windows/` contains the WinUI project, native file pickers, clipboard integration, and C# bindings.
- `platforms/macos/` contains the SwiftUI app, macOS file pickers, clipboard integration, and bundle build script.
- `include/packobf_gui.h` describes the platform-neutral C ABI, including string ownership. The Windows and macOS builds enable Cargo's `ffi` feature and compile the library as a `cdylib`. Swift imports the C header directly.

#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import platform
import queue
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import threading
import time
import urllib.request
import venv
import zipfile
from pathlib import Path

DEFAULT_VERSION = "26.2"
PACK_NAME = "pack.zip"
TIMEOUT = 60

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path, help="resource pack ZIP file")
    parser.add_argument("--version", default=DEFAULT_VERSION)
    parser.add_argument(
        "--keep-open", action="store_true",
        help="leave Minecraft open after the resource pack has loaded",
    )
    args = parser.parse_args()

    args.pack = args.pack.expanduser().resolve()
    if not args.pack.is_file():
        parser.error(f"resource pack does not exist: {args.pack}")
    return args


def ensure_launcher_library(root: Path) -> None:
    try:
        import minecraft_launcher_lib
        return
    except ImportError:
        pass

    environment = root / ".venv"
    python = environment / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
    if not python.exists():
        print("Setting up the private launcher environment…", flush=True)
        venv.EnvBuilder(with_pip=True).create(environment)
        subprocess.run(
            [str(python), "-m", "pip", "install", "--disable-pip-version-check", "minecraft-launcher-lib>=8,<9"],
            check=True,
        )
    os.execv(str(python), [str(python), str(Path(__file__).resolve()), *sys.argv[1:]])


def install_game(root: Path, version: str) -> int:
    import minecraft_launcher_lib as mcl

    print(f"Checking vanilla Minecraft {version} files…", flush=True)
    mcl.install.install_minecraft_version(version, str(root))
    version_json = root / "versions" / version / f"{version}.json"
    metadata = json.loads(version_json.read_text(encoding="utf-8"))
    try:
        return int(metadata["javaVersion"]["majorVersion"])
    except (KeyError, TypeError, ValueError) as error:
        raise RuntimeError(f"Mojang metadata has no valid Java requirement for {version}") from error


def java_platform() -> tuple[str, str, str]:
    system = platform.system()
    os_name = {"Linux": "linux", "Windows": "windows", "Darwin": "mac"}.get(system)
    machine = platform.machine().lower()
    arch = {"x86_64": "x64", "amd64": "x64", "aarch64": "aarch64", "arm64": "aarch64"}.get(machine)
    if not os_name or not arch:
        raise RuntimeError(f"unsupported platform or CPU architecture: {system} {machine}")
    extension = "zip" if system == "Windows" else "tar.gz"
    return os_name, arch, extension


def ensure_java(root: Path, major: int) -> Path:
    system = platform.system()
    home = root / f"jdk-{major}"
    executable = home / "bin" / ("java.exe" if system == "Windows" else "java")
    if executable.exists():
        return executable

    os_name, arch, extension = java_platform()
    url = f"https://api.adoptium.net/v3/binary/latest/{major}/ga/{os_name}/{arch}/jdk/hotspot/normal/eclipse"
    archive = root / f"temurin-{major}.{extension}"
    unpacked = root / f".jdk-{major}-unpack"
    print(f"Downloading Temurin Java {major} for {system}…", flush=True)
    print(f"Downloading from {url}", flush=True)
    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": "mc-test-resource-pack/1.0",
            "Accept": "application/octet-stream",
        },
    )

    with urllib.request.urlopen(request, timeout=60) as response:
        with archive.open("wb") as destination:
            shutil.copyfileobj(response, destination)
    unpacked.mkdir(parents=True, exist_ok=True)
    if extension == "zip":
        with zipfile.ZipFile(archive) as source:
            source.extractall(unpacked)
    else:
        with tarfile.open(archive, "r:gz") as source:
            if hasattr(tarfile, "data_filter"):
                source.extractall(unpacked, filter="data")
            else:  # Python before 3.12
                source.extractall(unpacked)

    contents = list(unpacked.iterdir())
    if len(contents) == 1 and contents[0].is_dir():
        extracted_home = contents[0]
    else:
        extracted_home = unpacked
    if home.exists():
        shutil.rmtree(home)
    extracted_home.rename(home)
    archive.unlink(missing_ok=True)
    if not executable.exists():
        raise RuntimeError(f"Java archive did not contain {executable}")
    return executable


def set_resource_pack(options_path: Path) -> None:
    text = options_path.read_text(encoding="utf-8") if options_path.exists() else ""
    setting = "resourcePacks:" + json.dumps(["vanilla", f"file/{PACK_NAME}"], separators=(",", ":"))
    match = re.search(r"(?m)^resourcePacks:.*$", text)
    if match:
        text = text[:match.start()] + setting + text[match.end():]
    else:
        if text and not text.endswith("\n"):
            text += "\n"
        text += setting + "\n"
    options_path.write_text(text, encoding="utf-8")


def terminate_client(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def run_client(root: Path, java: Path, version: str, keep_open: bool) -> None:
    import minecraft_launcher_lib as mcl

    # Minecraft's bundled logging config normally writes logs/latest.log.
    # Supply a console-only Log4j config so client output stays in this terminal.
    log_config = root / "console-log4j2.xml"
    log_config.write_text(
        """<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<Configuration status=\"WARN\">
  <Appenders>
    <Console name=\"Console\" target=\"SYSTEM_OUT\">
      <PatternLayout pattern=\"[%d{HH:mm:ss}] [%t/%level]: %msg%n\"/>
    </Console>
  </Appenders>
  <Loggers>
    <Root level=\"info\"><AppenderRef ref=\"Console\"/></Root>
  </Loggers>
</Configuration>
""",
        encoding="utf-8",
    )
    options = mcl.utils.generate_test_options()
    options["username"] = "Player123"
    options.update({
        "executablePath": str(java),
        "launcherName": "mc-test-resource-pack",
        "launcherVersion": "1",
        "enableLoggingConfig": False,
        "jvmArguments": [f"-Dlog4j.configurationFile={log_config.as_uri()}"],
    })
    command = mcl.command.get_minecraft_command(version, str(root), options)
    print(f"Starting Minecraft {version}", flush=True)
    process = subprocess.Popen(
        command, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        bufsize=0,
    )
    output: queue.Queue[str | None] = queue.Queue()

    def forward_output() -> None:
        assert process.stdout is not None
        for raw_line in process.stdout:
            line = raw_line.decode(errors="replace")
            output.put(line)
        output.put(None)

    reader = threading.Thread(target=forward_output, name="minecraft-output", daemon=True)
    reader.start()

    def stop_handler(_signum: int, _frame: object) -> None:
        terminate_client(process)
        raise KeyboardInterrupt

    old_sigint = signal.signal(signal.SIGINT, stop_handler)
    old_sigterm = signal.signal(signal.SIGTERM, stop_handler) if hasattr(signal, "SIGTERM") else None
    started = time.monotonic()
    last_output = started
    saw_pack = False
    pack_event = re.compile(r"Reloading ResourceManager:.*file/pack\.zip")

    try:
        while True:
            try:
                line = output.get(timeout=0.2)
            except queue.Empty:
                line = ""
            if line is None:
                if process.poll() is not None:
                    if keep_open and process.returncode == 0:
                        return
                    raise RuntimeError(f"Minecraft exited with status {process.returncode}")
                continue
            if line:
                if pack_event.search(line):
                    saw_pack = True

                if not keep_open and "Caught error loading resourcepacks, removing all selected resourcepacks" in line:
                    terminate_client(process)
                    reader.join(timeout=2)
                    print("\nResource Pack failed to load. Closed Minecraft.")
                    return

                if saw_pack:
                    sys.stdout.write(line)
                    sys.stdout.flush()
                    last_output = time.monotonic()

            if process.poll() is not None and not reader.is_alive() and output.empty():
                raise RuntimeError(f"Minecraft exited with status {process.returncode}")
            if not keep_open and time.monotonic() - started > TIMEOUT and not saw_pack:
                terminate_client(process)
                raise TimeoutError(f"timed out after {TIMEOUT}s waiting for resource-pack loading")
            if saw_pack and not keep_open and time.monotonic() - last_output >= 3:
                terminate_client(process)
                reader.join(timeout=2)
                print("\nResource pack load completed; no client logs for 3 seconds. Closed Minecraft.")
                return
    finally:
        terminate_client(process)
        reader.join(timeout=2)
        signal.signal(signal.SIGINT, old_sigint)
        if old_sigterm is not None:
            signal.signal(signal.SIGTERM, old_sigterm)


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parent / ".minecraft"
    (root / "resourcepacks").mkdir(parents=True, exist_ok=True)
    root.mkdir(parents=True, exist_ok=True)

    try:
        ensure_launcher_library(root)
        java_major = install_game(root, args.version)
        java = ensure_java(root, java_major)
        shutil.copy2(args.pack, root / "resourcepacks" / PACK_NAME)
        set_resource_pack(root / "options.txt")
        run_client(root, java, args.version, args.keep_open)
        return 0
    except KeyboardInterrupt:
        print("\nInterrupted; stopped this launcher session.", file=sys.stderr)
        return 130
    except Exception as error:
        print(f"mc-test-resource-pack: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

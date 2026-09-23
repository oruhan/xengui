use std::env;
use std::ffi::{OsStr, OsString};
use std::process::{Command, ExitCode};

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const ANDROID_TARGETS: &[&str] = &["aarch64-linux-android", "x86_64-linux-android"];

// These are the packages with supported browser entry points. In particular,
// Android-only packages must not be made wasm-compatible with placeholder cfgs.
const WASM_PACKAGES: &[&str] = &[
    "xengui",
    "xengui-wgpu",
    "xenframe",
    "xen-router",
    "xengui_website",
    "xengui-showcase",
    "settings-app",
];

// This group is intentionally separate from the wasm and desktop groups.
const ANDROID_PACKAGES: &[&str] = &["android_demo"];

// Native applications and their platform libraries. android_demo_native is a
// desktop harness for the Android UI, while android_demo itself is Android-only.
const DESKTOP_PACKAGES: &[&str] = &[
    "xen-animation",
    "xen-audio",
    "xen-clipboard",
    "xen-router",
    "xen-router-build",
    "xen-svg",
    "xenframe",
    "xengui",
    "xengui-cli",
    "xengui-icons",
    "xengui-wgpu",
    "android_demo_native",
    "pearl",
    "settings-app",
    "xengui-showcase",
    "xengui_website",
];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "quality".to_owned());

    if args.next().is_some() {
        eprintln!("xtask commands do not accept arguments");
        return ExitCode::FAILURE;
    }

    let result = match command.as_str() {
        "quality" => quality(),
        "host" => host_gates(),
        "wasm" => wasm_check(),
        "android" => android_check(),
        "desktop" => desktop_check(),
        "deny" => deny_check(),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        unknown => Err(format!("unknown xtask command `{unknown}`")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("\nquality gate failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn quality() -> Result<(), String> {
    host_gates()?;
    wasm_check()?;
    android_check()?;
    desktop_check()?;
    deny_check()?;
    println!("\nAll quality gates passed.");
    Ok(())
}

fn host_gates() -> Result<(), String> {
    run("format", "cargo", ["fmt", "--all", "--check"])?;
    run(
        "clippy",
        "cargo",
        [
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run(
        "tests",
        "cargo",
        ["test", "--workspace", "--all-targets", "--locked"],
    )?;

    let mut docs = Command::new("cargo");
    docs.args(["doc", "--workspace", "--no-deps", "--locked"])
        .env("RUSTDOCFLAGS", "-D warnings");
    run_command("rustdoc", &mut docs)
}

fn wasm_check() -> Result<(), String> {
    cargo_check_group("wasm", WASM_TARGET, WASM_PACKAGES)
}

fn android_check() -> Result<(), String> {
    for target in ANDROID_TARGETS {
        cargo_check_group(&format!("android ({target})"), target, ANDROID_PACKAGES)?;
    }
    Ok(())
}

fn desktop_check() -> Result<(), String> {
    let platform = match env::consts::OS {
        "linux" => "Linux",
        "windows" => "Windows",
        "macos" => "macOS",
        other => return Err(format!("unsupported desktop host `{other}`")),
    };

    cargo_check_group(
        &format!("{platform} desktop"),
        host_target(),
        DESKTOP_PACKAGES,
    )
}

fn deny_check() -> Result<(), String> {
    run("dependency policy", "cargo", ["deny", "check"])
}

fn cargo_check_group(name: &str, target: &str, packages: &[&str]) -> Result<(), String> {
    let mut args = vec![
        OsString::from("check"),
        OsString::from("--locked"),
        OsString::from("--all-targets"),
        OsString::from("--target"),
        OsString::from(target),
    ];
    for package in packages {
        args.push(OsString::from("--package"));
        args.push(OsString::from(package));
    }
    run(name, "cargo", args)
}

fn host_target() -> &'static str {
    match (env::consts::ARCH, env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        _ => env::consts::ARCH,
    }
}

fn run<I, S>(name: &str, program: &str, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args);
    run_command(name, &mut command)
}

fn run_command(name: &str, command: &mut Command) -> Result<(), String> {
    println!("\n==> {name}");
    let status = command
        .status()
        .map_err(|error| format!("could not start `{name}`: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{name}` exited with {status}"))
    }
}

fn print_help() {
    println!(
        "cargo xtask [quality|host|wasm|android|desktop|deny]\n\n\
         `quality` is the default and runs every gate for the current host.\n\
         CI runs it on Linux, Windows, and macOS to complete the native matrix."
    );
}

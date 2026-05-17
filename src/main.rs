use clap::Parser;
use std::{
    env,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Parser)]
#[command(about = "Open URLs, domains and files with Windows default apps from WSL", version, long_about = None)]
struct Cli {
    targets: Vec<String>,
}

#[derive(Debug)]
enum LogLevel {
    Info,
    Error,
}

fn log(level: LogLevel, msg: &str) {
    let tag = match level {
        LogLevel::Info => "[INFO]",
        LogLevel::Error => "[ERROR]",
    };
    println!("{tag} {msg}");
}

fn expand_home(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    s.to_string()
}

fn is_url(s: &str) -> bool {
    s.contains("://")
}

fn looks_like_domain(s: &str) -> bool {
    let s = s.split(':').next().unwrap_or(s);
    !s.contains('/')
        && !s.contains('\\')
        && (s.contains('.') || s.eq_ignore_ascii_case("localhost"))
}

fn wslpath_win(path: &str) -> Option<String> {
    let out = Command::new("wslpath")
        .args(["-w", "--", path])
        .output()
        .ok()?;

    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn to_windows_path(p: &str) -> String {
    let p = expand_home(p);
    wslpath_win(&p).unwrap_or(p)
}

fn open_with_windows_shell(target: &str) {
    log(LogLevel::Info, &format!("open: {target}"));

    let _ = Command::new("cmd.exe")
        .args(["/C", "start", "", target])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn dispatch(input: &str) {
    let input = expand_home(input);

    log(LogLevel::Info, &format!("dispatch: {input}"));

    if is_url(&input) {
        return open_with_windows_shell(&input);
    }

    if looks_like_domain(&input) {
        return open_with_windows_shell(&format!("https://{input}"));
    }

    if Path::new(&input).exists() {
        return open_with_windows_shell(&to_windows_path(&input));
    }

    log(LogLevel::Error, &format!("unsupported: {input}"));
}

fn main() {
    let cli = Cli::parse();

    if cli.targets.is_empty() {
        if let Ok(cwd) = env::current_dir() {
            if let Some(p) = cwd.to_str() {
                dispatch(p);
            }
        }
        return;
    }

    for target in cli.targets {
        dispatch(&target);
    }
}

use clap::Parser;
use std::{
    env, fs,
    io::{self, Read},
    path::Path,
    process::{Command, Stdio},
};

#[derive(Parser)]
#[command(about = "Open anything in Windows from WSL", version)]
struct Cli {
    /// Files, URLs, or domains to open. Opens CWD if empty.
    targets: Vec<String>,

    /// Open stdin as a temporary file with this extension (e.g. 'html')
    #[arg(short, long)]
    ext: Option<String>,

    /// Open the "Choose an app" dialog (Windows openas)
    #[arg(long, short = 'O')]
    open_as: bool,

    /// Show the file in Explorer instead of opening it
    #[arg(long, short = 's')]
    select: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Some(ext) = cli.ext {
        if let Err(e) = handle_stdin(&ext) {
            eprintln!("stdin: {e}");
        }
        return;
    }

    let targets = if !cli.targets.is_empty() {
        cli.targets.clone()
    } else {
        vec![
            env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| ".".to_string()),
        ]
    };

    for t in targets {
        dispatch(&t, &cli);
    }
}

fn dispatch(input: &str, opts: &Cli) {
    let target = resolve(input);

    match target {
        Resolved::Url(url) => {
            if opts.select || opts.open_as {
                eprintln!("'--select' or '--open-as' is not supported for URLs: {url}");
            } else {
                run_rundll32("url.dll", "FileProtocolHandler", &url);
            }
        }
        Resolved::File(path) => {
            if opts.select {
                run_explorer_select(&path);
            } else if opts.open_as {
                run_rundll32("shell32.dll", "OpenAs_RunDLL", &path);
            } else {
                run_rundll32("url.dll", "FileProtocolHandler", &path);
            }
        }
    }
}

/// Call explorer.exe /select,<path> (direct binary, not via rundll32)
fn run_explorer_select(win_path: &str) {
    let exe = find_windows_bin("explorer.exe");
    // explorer /select 语法：/select, 后面必须紧跟路径，不能有空格
    let _ = Command::new(exe)
        .arg(format!("/select,{win_path}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Call rundll32.exe dll,func arg
fn run_rundll32(dll: &str, func: &str, arg: &str) {
    let exe = find_windows_bin("rundll32.exe");
    let _ = Command::new(exe)
        .arg(format!("{dll},{func}"))
        .arg(arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

enum Resolved {
    Url(String),
    File(String),
}

fn resolve(input: &str) -> Resolved {
    let s = input.trim();
    if s.contains("://") || s.starts_with("mailto:") {
        return Resolved::Url(s.to_string());
    }
    if looks_like_domain(s) {
        return Resolved::Url(format!("https://{s}"));
    }

    let expanded = expand_home(s);
    let win = wsl_to_win(&expanded);
    Resolved::File(win.unwrap_or(expanded))
}

fn looks_like_domain(s: &str) -> bool {
    !s.contains('/') && !s.contains('\\') && (s.contains('.') || s == "localhost")
}

fn expand_home(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/")
        && let Ok(home) = env::var("HOME")
    {
        return format!("{home}/{rest}");
    }
    s.to_string()
}

fn wsl_to_win(path: &str) -> Option<String> {
    Command::new("wslpath")
        .args(["-w", "--", path])
        .output()
        .ok()
        .and_then(|out| {
            out.status
                .success()
                .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        })
}

/// Find a Windows binary by probing common WSL mount prefixes
fn find_windows_bin(name: &str) -> String {
    for prefix in [
        "/mnt/c/Windows/System32",
        "/mnt/c/Windows",
        "/mnt/c/Windows/Sysnative",
    ] {
        let p = Path::new(prefix).join(name);
        if p.exists() {
            return p.to_string_lossy().into_owned();
        }
    }
    name.to_string()
}

fn handle_stdin(ext: &str) -> io::Result<()> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    if buf.is_empty() {
        return Ok(());
    }

    let dot = if ext.starts_with('.') {
        ext.to_string()
    } else {
        format!(".{ext}")
    };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let tmp = env::temp_dir().join(format!("wsl_open_{ts:x}{dot}"));

    fs::write(&tmp, &buf)?;

    if let Some(path) = wsl_to_win(tmp.to_str().unwrap()) {
        run_rundll32("url.dll", "FileProtocolHandler", &path);
    }
    Ok(())
}

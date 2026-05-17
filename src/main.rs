use std::env;
use std::path::Path;
use std::process::Command;

fn is_url(s: &str) -> bool {
    s.contains("://")
}

fn looks_like_domain(s: &str) -> bool {
    s.contains('.') && !s.contains('/') && !s.contains('\\')
}

fn wslpath_w(path: &str) -> Option<String> {
    let output = Command::new("wslpath")
        .args(["-w", "--", path])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn wslpath_u(path: &str) -> Option<String> {
    let output = Command::new("wslpath").args(["-u", path]).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn linux_to_win(path: &str) -> Option<String> {
    let p = Path::new(path);
    let canonical = p.canonicalize().ok()?;
    let s = canonical.to_str()?;

    if !s.starts_with("/mnt/") {
        return None;
    }

    let parts: Vec<&str> = s.splitn(4, '/').collect();
    if parts.len() < 4 {
        return None;
    }

    let drive = parts[2];
    let rest = parts[3..].join("\\");

    Some(format!("{}:\\{}", drive.to_uppercase(), rest))
}

fn resolve_windows_path(path: &str) -> Option<String> {
    wslpath_w(path)
        .or_else(|| linux_to_win(path))
        .or_else(|| Some(path.to_string()))
}

fn copy_to_windows_temp(path: &str) -> Option<String> {
    let src = Path::new(path);
    let file_name = src.file_name()?.to_str()?;

    let output = Command::new("cmd.exe")
        .args(["/c", "echo", "%TEMP%"])
        .output()
        .ok()?;

    let win_temp = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let linux_temp = wslpath_u(&win_temp).unwrap_or_else(|| "/tmp".to_string());

    let dest_dir = format!("{}/wsl-open", linux_temp.trim_end_matches('/'));
    let dest = format!("{}/{}", dest_dir, file_name);

    std::fs::create_dir_all(&dest_dir).ok()?;
    std::fs::copy(src, &dest).ok()?;

    resolve_windows_path(&dest)
}

fn open_in_windows(target: &str, is_url: bool) {
    if is_url {
        let _ = Command::new("cmd.exe")
            .args(["/c", "start", "", target])
            .status();
    } else {
        let _ = Command::new("cmd.exe")
            .args(["/c", "start", "", target])
            .status();
    }
}

fn is_unc_path(p: &str) -> bool {
    p.starts_with('\\')
}

fn open_path(path: &str) {
    let win = resolve_windows_path(path);

    match win {
        Some(p) if is_unc_path(&p) => match copy_to_windows_temp(path) {
            Some(copy) => open_in_windows(&copy, false),
            None => open_in_windows(&p, false),
        },
        Some(p) => open_in_windows(&p, false),
        None if Path::new(path).is_dir() => {
            eprintln!("wsl-open: cannot open directory outside Windows partition: {path}");
        }
        None => match copy_to_windows_temp(path) {
            Some(p) => open_in_windows(&p, false),
            None => eprintln!("wsl-open: failed to open: {path}"),
        },
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let cwd = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        if let Some(cwd) = cwd.to_str() {
            open_path(cwd);
        }
        return;
    }

    for arg in &args[1..] {
        if is_url(arg) {
            open_in_windows(arg, true);
        } else if Path::new(arg).exists() {
            open_path(arg);
        } else if looks_like_domain(arg) {
            open_in_windows(&format!("https://{arg}"), true);
        } else {
            eprintln!("wsl-open: path does not exist: {arg}");
        }
    }
}

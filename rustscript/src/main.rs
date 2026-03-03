//! RustScript — a compiled language that turns .rsx into self-contained HTML.
//!
//! Usage:
//!   rustscript preview <file.rsx>                      Build + open in browser
//!   rustscript build   <file.rsx>  [-o output.html]    Compile to HTML
//!   rustscript run     <file.rsx>                      Interpret in terminal
//!   rustscript help                                    Show help

mod ast;
mod codegen;
mod interpreter;
mod lexer;
mod parser;
mod server;
mod token;
mod turbo;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub(crate) use base64_engine::encode as base64_encode;

/// Lightweight base64 encoder (no external crate needed).
mod base64_engine {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(data: &[u8]) -> String {
        let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
            out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(CHARS[(triple & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }
}

/// File extensions recognized as images for auto-import.
pub(crate) fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp"
    )
}

/// Map file extension to MIME type.
pub(crate) fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}

/// Derive a clean variable name from a filename (without extension).
/// e.g. "my-logo.png" -> "my_logo", "rustscriptlogo.png" -> "rustscriptlogo"
pub(crate) fn var_name_from_path(path: &Path) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    stem.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "preview" => cmd_preview(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "serve" => cmd_serve(&args[2..]),
        "stop" => cmd_stop(),
        "ps" => cmd_ps(),
        "help" | "--help" | "-h" => print_usage(),
        "--version" | "-V" | "version" => {
            println!("rustscript {}", env!("CARGO_PKG_VERSION"));
        }
        other => {
            // If they just pass a file, auto-detect mode
            if other.ends_with(".rsx") {
                // Peek at source: if it uses `import turbo` or only print(),
                // default to `run` (terminal mode) instead of `preview` (HTML).
                let path = &args[1];
                if should_auto_run(path) {
                    cmd_run(&args[1..]);
                } else {
                    cmd_preview(&args[1..]);
                }
            } else {
                eprintln!("Unknown command: '{}'", other);
                print_usage();
                process::exit(1);
            }
        }
    }
}

/// Peek at a .rsx file's source to decide whether it's a terminal script (run)
/// or a web page (preview). Heuristic: if it imports turbo, or has no HTML tags
/// (no `page { }` or `div { }` etc.), it's a terminal script.
fn should_auto_run(path: &str) -> bool {
    let Ok(src) = fs::read_to_string(path) else {
        return false;
    };
    // `import turbo` → definitely a terminal script
    if src.lines().any(|l| {
        let t = l.trim();
        t == "import turbo" || t.starts_with("import turbo ")
    }) {
        return true;
    }
    // If there's a `page {` block, it's HTML
    if src.contains("page {") || src.contains("page{") {
        return false;
    }
    // If there are HTML tags like div/span/h1/button, it's HTML
    let html_tags = ["div {", "div{", "span {", "span{", "h1 {", "h1{",
                     "button {", "button{", "input {", "input{",
                     "form {", "form{", "section {", "section{",
                     "header {", "header{", "footer {", "footer{",
                     "nav {", "nav{", "main {", "main{"];
    for tag in &html_tags {
        if src.contains(tag) {
            return false;
        }
    }
    // No HTML found → default to run
    true
}

fn cmd_preview(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No input file specified.");
        eprintln!("Usage: rustscript preview <file.rsx>");
        process::exit(1);
    }

    let input = &args[0];
    let html = compile_to_html(input);

    // Write to .rustscript/ directory next to the source file
    let input_path = PathBuf::from(input);
    let parent = input_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let out_dir = parent.join(".rustscript");
    if !out_dir.exists()
        && let Err(e) = fs::create_dir_all(&out_dir) {
            eprintln!("Error creating '{}': {}", out_dir.display(), e);
            process::exit(1);
        }

    // Write a .gitignore so the build artifacts don't get committed
    let gitignore = out_dir.join(".gitignore");
    if !gitignore.exists() {
        let _ = fs::write(&gitignore, "*\n");
    }

    let stem = input_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "rustscript_preview".to_string());
    let out_path = out_dir.join(format!("{}.html", stem));
    let out_str = out_path.to_string_lossy().to_string();

    match fs::write(&out_path, &html) {
        Ok(_) => {
            println!("Built {} ({} bytes)", input, html.len());
            println!("Preview at .rustscript/{}.html", stem);
            println!("Opening in browser...");
            open_in_browser(&out_str);
        }
        Err(e) => {
            eprintln!("Error writing '{}': {}", out_str, e);
            process::exit(1);
        }
    }
}

pub(crate) fn open_in_browser(path: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = process::Command::new("xdg-open").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = process::Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn();
    }
}

fn compile_to_html(input: &str) -> String {
    let program = parse_file(input);
    let base_dir = Path::new(input).parent().unwrap_or_else(|| Path::new("."));
    let canonical = fs::canonicalize(input).unwrap_or_else(|_| PathBuf::from(input));
    let mut seen = HashSet::new();
    seen.insert(canonical);
    let resolved = match server::resolve_imports(program, base_dir, &mut seen) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let mut cgen = codegen::Codegen::new();
    cgen.generate(&resolved)
}

fn cmd_build(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No input file specified.");
        eprintln!("Usage: rustscript build <file.rsx> [-o output.html]");
        process::exit(1);
    }

    let input = &args[0];
    let output = if args.len() >= 3 && args[1] == "-o" {
        args[2].clone()
    } else {
        input.replace(".rsx", ".html")
    };

    let html = compile_to_html(input);

    match fs::write(&output, &html) {
        Ok(_) => {
            println!("Built {} -> {} ({} bytes)", input, output, html.len());
        }
        Err(e) => {
            eprintln!("Error writing '{}': {}", output, e);
            process::exit(1);
        }
    }
}

fn cmd_run(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No input file specified.");
        eprintln!("Usage: rustscript run <file.rsx>");
        process::exit(1);
    }

    let input = &args[0];
    let input_abs = fs::canonicalize(input)
        .unwrap_or_else(|_| PathBuf::from(input))
        .display()
        .to_string();

    // ── Cleanup stale PIDs + kill duplicates ─────────────
    let pid_dir = pid_dir();
    let _ = fs::create_dir_all(&pid_dir);
    cleanup_stale_pids(&pid_dir);
    kill_duplicate_runs(&pid_dir, &input_abs);

    // ── Write our PID file ───────────────────────────────
    let pid = process::id();
    let pid_file = pid_dir.join(format!("{}.pid", pid));
    let pid_content = format!("{}|{}|{}", pid, input_abs, timestamp_now());
    let _ = fs::write(&pid_file, &pid_content);

    // ── Drop guard — cleans up PID even on panic ─────────
    let _guard = PidGuard(pid_file.clone());

    // ── Signal handling (Ctrl+C + SIGTERM) ───────────────
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let pid_file_clone = pid_file.clone();
    ctrlc_register(move || {
        r.store(false, Ordering::SeqCst);
        let _ = fs::remove_file(&pid_file_clone);
    });

    // ── Parse ────────────────────────────────────────────
    let program = parse_file(input);
    let base_dir = Path::new(input.as_str())
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let canonical = fs::canonicalize(input).unwrap_or_else(|_| PathBuf::from(input.as_str()));
    let mut seen = HashSet::new();
    seen.insert(canonical);
    let resolved = match server::resolve_imports(program, base_dir, &mut seen) {
        Ok(r) => r,
        Err(e) => {
            let _ = fs::remove_file(&pid_file);
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // ── Status banner ────────────────────────────────────
    let short_name = Path::new(input)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| input.clone());
    eprintln!("\x1b[1;36m▶ rustscript\x1b[0m running \x1b[1m{}\x1b[0m  (pid {})", short_name, pid);
    eprintln!("\x1b[90m  Stop with: \x1b[0mrustscript stop\x1b[90m  or  \x1b[0mCtrl+C");
    eprintln!("\x1b[90m  ─────────────────────────────────────────\x1b[0m");

    let start_time = Instant::now();

    // ── Spawn status thread ──────────────────────────────
    let r2 = running.clone();
    let short_name2 = short_name.clone();
    let status_thread = std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(5));
            if !r2.load(Ordering::SeqCst) {
                break;
            }
            let elapsed = start_time.elapsed();
            let secs = elapsed.as_secs();
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            if h > 0 {
                eprint!("\r\x1b[90m  ⏱ {} running {:02}:{:02}:{:02}\x1b[0m\x1b[K", short_name2, h, m, s);
            } else {
                eprint!("\r\x1b[90m  ⏱ {} running {:02}:{:02}\x1b[0m\x1b[K", short_name2, m, s);
            }
        }
    });

    // ── Execute ──────────────────────────────────────────
    let mut interp = interpreter::Interpreter::new();
    let result = interp.run(&resolved);

    // ── Cleanup ──────────────────────────────────────────
    running.store(false, Ordering::SeqCst);
    let _ = status_thread.join();
    let _ = fs::remove_file(&pid_file);

    let elapsed = start_time.elapsed();
    let secs = elapsed.as_secs_f64();

    match result {
        Ok(_) => {
            eprintln!("\n\x1b[1;32m✓\x1b[0m Finished in {:.2}s", secs);
        }
        Err(e) => {
            eprintln!("\n\x1b[1;31m✗\x1b[0m Runtime error: {}", e);
            eprintln!("  Ran for {:.2}s before failure", secs);
            process::exit(1);
        }
    }
}

// ── Zombie prevention ────────────────────────────────────────

/// Drop guard that removes the PID file if the process panics or exits unexpectedly.
struct PidGuard(PathBuf);
impl Drop for PidGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Scan PID directory and remove entries for processes that no longer exist.
fn cleanup_stale_pids(pid_dir: &Path) {
    let entries = match fs::read_dir(pid_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("pid") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            let parts: Vec<&str> = content.splitn(3, '|').collect();
            if let Some(pid_str) = parts.first()
                && let Ok(pid) = pid_str.parse::<u32>()
                    && !signal_process(pid, 0) {
                        // Process is dead — remove stale PID file
                        let _ = fs::remove_file(&path);
                    }
        }
    }
}

/// If another `rustscript run` is already running the same file, kill it first.
/// Prevents zombie duplicates when user re-runs the same script.
fn kill_duplicate_runs(pid_dir: &Path, input_abs: &str) {
    let my_pid = process::id();
    let entries = match fs::read_dir(pid_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("pid") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            let parts: Vec<&str> = content.splitn(3, '|').collect();
            let pid: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let file = parts.get(1).unwrap_or(&"");

            // Skip our own PID
            if pid == my_pid || pid == 0 {
                continue;
            }

            // Same file AND process is alive → kill the old one
            if *file == input_abs && signal_process(pid, 0) {
                eprintln!(
                    "\x1b[1;33m⚠\x1b[0m  Stopping previous instance (pid {}) …",
                    pid
                );
                signal_process(pid, 15); // SIGTERM
                // Give it a moment to die, then force-kill if needed
                std::thread::sleep(std::time::Duration::from_millis(200));
                if signal_process(pid, 0) {
                    signal_process(pid, 9); // SIGKILL
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                let _ = fs::remove_file(&path);
            }
        }
    }
}

/// Find rustscript processes that are running but have no PID file (rogues / zombies).
/// Uses `pgrep` on Unix to find them by command name.
fn find_rogue_rustscript_pids() -> Vec<u32> {
    let my_pid = process::id();
    #[cfg(unix)]
    {
        let output = std::process::Command::new("pgrep")
            .args(["-f", "rustscript run"])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .filter(|pid| *pid != my_pid)
                .collect();
        }
    }
    #[cfg(not(unix))]
    let _ = my_pid;
    Vec::new()
}

/// `rustscript stop` — kill all running rustscript processes.
fn cmd_stop() {
    let pid_dir = pid_dir();
    let _ = fs::create_dir_all(&pid_dir);

    // Always sweep stale PIDs first
    cleanup_stale_pids(&pid_dir);

    let entries: Vec<_> = fs::read_dir(&pid_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "pid")
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        // Fallback: check for rogue processes via the process name
        let rogues = find_rogue_rustscript_pids();
        if rogues.is_empty() {
            println!("\x1b[90mNo running RustScript processes.\x1b[0m");
        } else {
            eprintln!(
                "\x1b[1;33m⚠\x1b[0m  Found {} rogue process{} without PID files — killing …",
                rogues.len(),
                if rogues.len() == 1 { "" } else { "es" }
            );
            for pid in &rogues {
                signal_process(*pid, 15);
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
            for pid in &rogues {
                if signal_process(*pid, 0) {
                    signal_process(*pid, 9); // force-kill survivors
                }
            }
            println!(
                "\x1b[1;32m✓\x1b[0m Killed {} rogue process{}",
                rogues.len(),
                if rogues.len() == 1 { "" } else { "es" }
            );
        }
        return;
    }

    let mut killed = 0;
    for entry in &entries {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(&path) {
            let parts: Vec<&str> = content.splitn(3, '|').collect();
            if let Some(pid_str) = parts.first()
                && let Ok(pid) = pid_str.parse::<u32>() {
                    let file_name = parts.get(1).unwrap_or(&"unknown");
                    let short = Path::new(file_name)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| file_name.to_string());
                    // Check if process is alive and kill it
                    if signal_process(pid, 0) {
                        // Process exists — send SIGTERM
                        signal_process(pid, 15);
                        println!(
                            "\x1b[1;31m■\x1b[0m Stopped \x1b[1m{}\x1b[0m (pid {})",
                            short, pid
                        );
                        killed += 1;
                    } else {
                        // Stale PID file
                        eprintln!(
                            "\x1b[90m  Cleaned stale PID file for {} (pid {})\x1b[0m",
                            short, pid
                        );
                    }
                }
        }
        // Remove PID file
        let _ = fs::remove_file(&path);
    }

    if killed == 0 {
        println!("\x1b[90mNo active processes found (cleaned stale PID files).\x1b[0m");
    } else {
        println!("\x1b[1;32m✓\x1b[0m Stopped {} process{}", killed, if killed == 1 { "" } else { "es" });
    }
}

/// `rustscript ps` — list running rustscript processes.
fn cmd_ps() {
    let pid_dir = pid_dir();
    let _ = fs::create_dir_all(&pid_dir);

    // Sweep stale PIDs first
    cleanup_stale_pids(&pid_dir);

    let entries: Vec<_> = fs::read_dir(&pid_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "pid")
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        println!("\x1b[90mNo running RustScript processes.\x1b[0m");
        return;
    }

    println!("\x1b[1m  PID      FILE                              STARTED\x1b[0m");
    println!("  ──────── ───────────────────────────────── ────────────────────");

    let mut count = 0;
    for entry in &entries {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(&path) {
            let parts: Vec<&str> = content.splitn(3, '|').collect();
            let pid_str = parts.first().unwrap_or(&"?");
            let file_str = parts.get(1).unwrap_or(&"?");
            let time_str = parts.get(2).unwrap_or(&"?");
            let pid: u32 = pid_str.parse().unwrap_or(0);
            let short = Path::new(file_str)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| file_str.to_string());

            let alive = signal_process(pid, 0);
            if alive {
                println!(
                    "  \x1b[1;32m●\x1b[0m {:<7} {:<37} {}",
                    pid_str, short, time_str
                );
                count += 1;
            } else {
                // Stale — clean up
                let _ = fs::remove_file(&path);
            }
        }
    }

    if count == 0 {
        println!("\x1b[90m  No active processes (cleaned stale PID files).\x1b[0m");
    } else {
        println!("\n  \x1b[90m{} process{} running. Stop with:\x1b[0m rustscript stop", count, if count == 1 { "" } else { "es" });
    }
}

// ── PID / signal helpers ─────────────────────────────────────

/// Directory where PID files are stored.
fn pid_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".rustscript").join("pids")
}

/// Get a human-readable timestamp.
fn timestamp_now() -> String {
    // Use seconds since epoch, format as readable
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Simple HH:MM:SS format
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02} UTC", hours, mins, s)
}

/// Send a signal to a process. Returns true if the process exists.
/// sig=0 just checks existence without killing.
fn signal_process(pid: u32, sig: i32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc_kill(pid as i32, sig) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = (pid, sig);
        false
    }
}

#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

#[cfg(unix)]
unsafe fn libc_kill(pid: i32, sig: i32) -> i32 {
    unsafe { kill(pid, sig) }
}

/// Register signal handlers (SIGINT + SIGTERM) — pure std, no crate needed.
fn ctrlc_register<F: FnOnce() + Send + 'static>(handler: F) {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        unsafe {
            HANDLER = Some(Box::new(handler));
        }

        #[cfg(unix)]
        unsafe {
            signal(2 /* SIGINT  */, signal_handler as *const () as usize);
            signal(15 /* SIGTERM */, signal_handler as *const () as usize);
        }
    });
}

#[cfg(unix)]
unsafe extern "C" {
    fn signal(sig: i32, handler: usize) -> usize;
}

/// Shared handler for SIGINT (Ctrl+C) and SIGTERM (kill / terminal close).
#[cfg(unix)]
extern "C" fn signal_handler(sig: i32) {
    unsafe {
        if let Some(handler) = std::ptr::addr_of_mut!(HANDLER).as_mut().and_then(|h| h.take()) {
            handler();
        }
    }
    if sig == 2 {
        eprintln!("\n\x1b[1;33m⚡\x1b[0m Interrupted — process stopped.");
        process::exit(130);
    } else {
        // SIGTERM — exit quietly
        process::exit(143);
    }
}

#[cfg(unix)]
static mut HANDLER: Option<Box<dyn FnOnce() + Send>> = None;

fn cmd_serve(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No input file specified.");
        eprintln!("Usage: rustscript serve <file.rsx> [-p port]");
        process::exit(1);
    }

    let input = &args[0];
    let mut port: u16 = 8080;

    // Parse optional -p flag
    let mut i = 1;
    while i < args.len() {
        if args[i] == "-p" && i + 1 < args.len() {
            port = args[i + 1].parse().unwrap_or_else(|_| {
                eprintln!("Error: Invalid port '{}'", args[i + 1]);
                process::exit(1);
            });
            i += 2;
        } else {
            i += 1;
        }
    }

    server::serve(input, port);
}

/// Lex and parse a single .rsx file into a Program AST.
fn parse_file(path: &str) -> ast::Program {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut lex = lexer::Lexer::new(&source);
    let tokens = match lex.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer error in '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut p = parser::Parser::new(tokens);
    match p.parse_program() {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("Parse error in '{}': {}", path, e);
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!(
        r#"
RustScript Compiler

USAGE:
  rustscript preview <file.rsx>
      Build and open in your browser instantly.

  rustscript build <file.rsx> [-o output.html]
      Compile a .rsx file to a self-contained HTML page.

  rustscript run <file.rsx>
      Interpret a .rsx file in the terminal (logic only).

  rustscript stop
      Kill all running RustScript processes.

  rustscript ps
      List all running RustScript processes.

  rustscript serve <file.rsx> [-p port]
      Start a dev server with live reload (default port: 8080).

  rustscript help
      Show this help message.

FILE EXTENSION: .rsx

EXAMPLES:
  rustscript run logic.rsx            # run in terminal
  rustscript stop                     # kill running processes
  rustscript ps                       # list running processes
  rustscript preview app.rsx          # build + open in browser
  rustscript build app.rsx            # compile to app.html
  rustscript build app.rsx -o out.html
  rustscript serve website/index.rsx  # dev server on localhost:8080
  rustscript serve app.rsx -p 3000    # custom port
"#
    );
}

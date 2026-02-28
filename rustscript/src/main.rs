/// RustScript — a language where HTML, CSS and Python had a child.
///
/// Usage:
///   rustscript preview <file.rsx>                      Build + open in browser
///   rustscript build   <file.rsx>  [-o output.html]    Compile to HTML
///   rustscript run     <file.rsx>                      Interpret in terminal
///   rustscript help                                    Show help

mod token;
mod lexer;
mod ast;
mod parser;
mod codegen;
mod interpreter;
mod server;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use base64_engine::encode as base64_encode;

/// Lightweight base64 encoder (no external crate needed).
mod base64_engine {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(data: &[u8]) -> String {
        let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
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
fn is_image_ext(ext: &str) -> bool {
    matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp")
}

/// Map file extension to MIME type.
fn mime_for_ext(ext: &str) -> &'static str {
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
fn var_name_from_path(path: &Path) -> String {
    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    stem.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
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
        "help" | "--help" | "-h" => print_usage(),
        "--version" | "-V" | "version" => {
            println!("rustscript {}", env!("CARGO_PKG_VERSION"));
        }
        other => {
            // If they just pass a file, default to preview
            if other.ends_with(".rsx") {
                cmd_preview(&args[1..]);
            } else {
                eprintln!("Unknown command: '{}'", other);
                print_usage();
                process::exit(1);
            }
        }
    }
}

fn cmd_preview(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: No input file specified.");
        eprintln!("Usage: rustscript preview <file.rsx>");
        process::exit(1);
    }

    let input = &args[0];
    let html = compile_to_html(input);

    // Write to a temp file
    let mut tmp_dir = env::temp_dir();
    let stem = PathBuf::from(input)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "rustscript_preview".to_string());
    tmp_dir.push(format!("{}.html", stem));
    let out_path = tmp_dir.to_string_lossy().to_string();

    match fs::write(&out_path, &html) {
        Ok(_) => {
            println!("Built {} ({} bytes)", input, html.len());
            println!("Opening in browser...");
            open_in_browser(&out_path);
        }
        Err(e) => {
            eprintln!("Error writing '{}': {}", out_path, e);
            process::exit(1);
        }
    }
}

fn open_in_browser(path: &str) {
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
        let _ = process::Command::new("cmd").args(["/C", "start", "", path]).spawn();
    }
}

fn compile_to_html(input: &str) -> String {
    let program = parse_file(input);
    let base_dir = Path::new(input)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let canonical = fs::canonicalize(input).unwrap_or_else(|_| PathBuf::from(input));
    let mut seen = HashSet::new();
    seen.insert(canonical);
    let resolved = resolve_imports(program, base_dir, &mut seen);

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
    let program = parse_file(input);
    let base_dir = Path::new(input.as_str())
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let canonical = fs::canonicalize(input).unwrap_or_else(|_| PathBuf::from(input.as_str()));
    let mut seen = HashSet::new();
    seen.insert(canonical);
    let resolved = resolve_imports(program, base_dir, &mut seen);

    let mut interp = interpreter::Interpreter::new();
    match interp.run(&resolved) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            process::exit(1);
        }
    }
}

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

/// Recursively resolve all `import "..."` statements by inlining the imported
/// file's AST in place. Tracks already-seen files to prevent circular imports.
fn resolve_imports(
    program: ast::Program,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
) -> ast::Program {
    let mut resolved_stmts = Vec::new();

    for stmt in program.stmts {
        match stmt {
            ast::Stmt::Import { path } => {
                let import_path = base_dir.join(&path);
                let canonical = match fs::canonicalize(&import_path) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error resolving import '{}': {}", path, e);
                        process::exit(1);
                    }
                };

                if !seen.insert(canonical.clone()) {
                    // Already imported this file, skip to avoid circular imports.
                    continue;
                }

                // Check if this is an image import
                let ext = canonical.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if is_image_ext(&ext) {
                    // Read binary, base64 encode, emit a let binding
                    let bytes = match fs::read(&canonical) {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("Error reading image '{}': {}", path, e);
                            process::exit(1);
                        }
                    };
                    let mime = mime_for_ext(&ext);
                    let data_uri = format!("data:{};base64,{}", mime, base64_encode(&bytes));
                    let var = var_name_from_path(&canonical);
                    resolved_stmts.push(ast::Stmt::Let {
                        name: var,
                        value: ast::Expr::Str(data_uri),
                    });
                    continue;
                }

                let import_str = import_path.to_string_lossy().to_string();
                let imported = parse_file(&import_str);

                let child_dir = canonical
                    .parent()
                    .unwrap_or_else(|| Path::new("."));
                let child_resolved = resolve_imports(imported, child_dir, seen);

                resolved_stmts.extend(child_resolved.stmts);
            }
            other => {
                resolved_stmts.push(other);
            }
        }
    }

    ast::Program { stmts: resolved_stmts }
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

  rustscript serve <file.rsx> [-p port]
      Start a dev server with live reload (default port: 8080).

  rustscript help
      Show this help message.

FILE EXTENSION: .rsx

EXAMPLES:
  rustscript preview app.rsx          # build + open in browser
  rustscript build app.rsx            # compile to app.html
  rustscript build app.rsx -o out.html
  rustscript run logic.rsx            # run in terminal
  rustscript serve website/index.rsx  # dev server on localhost:8080
  rustscript serve app.rsx -p 3000    # custom port
"#
    );
}

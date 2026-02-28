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

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

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

  rustscript help
      Show this help message.

FILE EXTENSION: .rsx

EXAMPLES:
  rustscript preview app.rsx          # build + open in browser
  rustscript build app.rsx            # compile to app.html
  rustscript build app.rsx -o out.html
  rustscript run logic.rsx            # run in terminal
"#
    );
}

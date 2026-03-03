//! Dev server for RustScript.
//! Serves compiled .rsx files over HTTP with live-reload on file changes.
//! Pure std library — no external dependencies.

use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::{ast, codegen, lexer, parser};

/// Compile a .rsx file to HTML, resolving imports relative to the file.
fn compile(input: &Path) -> Result<String, String> {
    let source = fs::read_to_string(input)
        .map_err(|e| format!("Error reading '{}': {}", input.display(), e))?;

    let mut lex = lexer::Lexer::new(&source);
    let tokens = lex.tokenize().map_err(|e| format!("Lexer error: {}", e))?;

    let mut p = parser::Parser::new(tokens);
    let program = p
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    let base_dir = input.parent().unwrap_or_else(|| Path::new("."));
    let canonical = fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    let mut seen = HashSet::new();
    seen.insert(canonical);
    let resolved = resolve_imports(program, base_dir, &mut seen)?;

    let mut cgen = codegen::Codegen::new();
    Ok(cgen.generate(&resolved))
}

/// Resolve imports (returns Result instead of exiting).
pub(crate) fn resolve_imports(
    program: ast::Program,
    base_dir: &Path,
    seen: &mut HashSet<PathBuf>,
) -> Result<ast::Program, String> {
    let mut resolved_stmts = Vec::new();

    for stmt in program.stmts {
        match stmt {
            ast::Stmt::Import { ref path }
                if !path.contains('.') && !path.contains('/') && !path.contains('\\') =>
            {
                // Module import (e.g. `import turbo`) — pass through to interpreter
                resolved_stmts.push(stmt);
                continue;
            }
            ast::Stmt::Import { path } => {
                let import_path = base_dir.join(&path);
                let canonical = fs::canonicalize(&import_path)
                    .map_err(|e| format!("Error resolving import '{}': {}", path, e))?;

                if !seen.insert(canonical.clone()) {
                    continue;
                }

                // Check if this is an image import
                let ext = canonical
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if crate::is_image_ext(&ext) {
                    let bytes = fs::read(&canonical)
                        .map_err(|e| format!("Error reading image '{}': {}", path, e))?;
                    let mime = crate::mime_for_ext(&ext);
                    let data_uri = format!("data:{};base64,{}", mime, crate::base64_encode(&bytes));
                    let var = crate::var_name_from_path(&canonical);
                    resolved_stmts.push(ast::Stmt::Let {
                        name: var,
                        value: ast::Expr::Str(data_uri),
                    });
                    continue;
                }

                let source = fs::read_to_string(&canonical)
                    .map_err(|e| format!("Error reading import '{}': {}", path, e))?;

                let mut lex = lexer::Lexer::new(&source);
                let tokens = lex
                    .tokenize()
                    .map_err(|e| format!("Lexer error in '{}': {}", path, e))?;

                let mut p = parser::Parser::new(tokens);
                let imported = p
                    .parse_program()
                    .map_err(|e| format!("Parse error in '{}': {}", path, e))?;

                let child_dir = canonical.parent().unwrap_or_else(|| Path::new("."));
                let child_resolved = resolve_imports(imported, child_dir, seen)?;
                resolved_stmts.extend(child_resolved.stmts);
            }
            other => {
                resolved_stmts.push(other);
            }
        }
    }

    Ok(ast::Program {
        stmts: resolved_stmts,
    })
}

/// JavaScript snippet injected into every served page.
/// Polls /__reload to detect file changes and auto-refreshes.
const LIVE_RELOAD_SCRIPT: &str = r#"
<script>
(function() {
  var lastHash = '';
  setInterval(function() {
    var x = new XMLHttpRequest();
    x.open('GET', '/__reload', true);
    x.onreadystatechange = function() {
      if (x.readyState === 4 && x.status === 200) {
        if (lastHash && lastHash !== x.responseText) {
          location.reload();
        }
        lastHash = x.responseText;
      }
    };
    x.send();
  }, 500);
})();
</script>
"#;

/// Collect modification times from the entry file and all its imported files.
fn collect_mtimes(input: &Path) -> u128 {
    let mut total: u128 = 0;

    if let Ok(meta) = fs::metadata(input)
        && let Ok(modified) = meta.modified()
    {
        total += modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
    }

    // Also scan for imported files
    if let Ok(source) = fs::read_to_string(input) {
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                // Extract path from: import "something.rsx"
                if let Some(start) = trimmed.find('"')
                    && let Some(end) = trimmed[start + 1..].find('"')
                {
                    let import_path = &trimmed[start + 1..start + 1 + end];
                    let resolved = input
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(import_path);
                    if let Ok(meta) = fs::metadata(&resolved)
                        && let Ok(modified) = meta.modified()
                    {
                        total += modified
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis();
                    }
                }
            }
        }
    }

    total
}

/// Start the dev server.
pub fn serve(input: &str, port: u16) {
    let input_path = PathBuf::from(input);
    if !input_path.exists() {
        eprintln!("Error: '{}' not found", input);
        std::process::exit(1);
    }

    let canonical = fs::canonicalize(&input_path).unwrap_or_else(|_| input_path.clone());

    // Shared mtime hash for live reload
    let mtime_hash: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    // Background thread: watch for file changes
    {
        let canonical = canonical.clone();
        let mtime_hash = mtime_hash.clone();
        thread::spawn(move || {
            loop {
                let hash = format!("{}", collect_mtimes(&canonical));
                if let Ok(mut h) = mtime_hash.lock() {
                    *h = hash;
                }
                thread::sleep(Duration::from_millis(300));
            }
        });
    }

    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error: Could not bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    println!("Serving {} on http://localhost:{}", input, port);
    println!("Live reload active. Press Ctrl+C to stop.\n");

    // Open browser
    crate::open_in_browser(&format!("http://localhost:{}", port));

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Read request
        let mut buf = [0u8; 4096];
        let n = match stream.read(&mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        let request = String::from_utf8_lossy(&buf[..n]);

        // Parse the request path from first line
        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        match path {
            "/__reload" => {
                let hash = mtime_hash.lock().map(|h| h.clone()).unwrap_or_default();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nCache-Control: no-cache\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                    hash.len(),
                    hash
                );
                let _ = stream.write_all(response.as_bytes());
            }
            _ => {
                // Compile and serve
                match compile(&canonical) {
                    Ok(mut html) => {
                        // Inject live-reload script before </body>
                        if let Some(pos) = html.rfind("</body>") {
                            html.insert_str(pos, LIVE_RELOAD_SCRIPT);
                        } else {
                            html.push_str(LIVE_RELOAD_SCRIPT);
                        }

                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-cache\r\nContent-Length: {}\r\n\r\n{}",
                            html.len(),
                            html
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                    Err(e) => {
                        // Serve a styled error page
                        let error_html = format!(
                            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>RustScript Error</title>
<style>
body {{ background: #0a0e17; color: #f87171; font-family: monospace; padding: 40px; }}
pre {{ background: #111827; border: 1px solid #7f1d1d; border-radius: 8px; padding: 20px; white-space: pre-wrap; font-size: 14px; }}
h1 {{ font-size: 1.2rem; margin-bottom: 16px; }}
</style></head>
<body><h1>Compile Error</h1><pre>{}</pre>
{}</body></html>"#,
                            html_escape(&e),
                            LIVE_RELOAD_SCRIPT,
                        );
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-cache\r\nContent-Length: {}\r\n\r\n{}",
                            error_html.len(),
                            error_html,
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                }
            }
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

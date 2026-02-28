# ──────────────────────────────────────────────────────────
#  RustScript — Compiler Deep Dive
#  Build:   rustscript build website/compiler.rsx -o compiler.html
# ──────────────────────────────────────────────────────────

import "lib/theme.rsx"
import "lib/logo.png"

page {

    style {
        bg: "#0a0e17"
        fg: "#e5e7eb"
        font: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
        pad: "0"
    }

    # ════════════════════════════════════════════════════════
    #  NAV BAR
    # ════════════════════════════════════════════════════════
    div {
        style {
            display: "flex"
            justify-content: "space-between"
            align-items: "center"
            pad: "16px 40px"
            border-bottom: "1px solid #1f2937"
            bg: "rgba(10, 14, 23, 0.95)"
            position: "sticky"
            top: "0"
            z-index: "100"
            backdrop-filter: "blur(12px)"
        }

        div {
            style { display: "flex" align-items: "center" gap: "12px" }

            img {
                src: "{logo}"
                alt: "RustScript"
                style { h: "32px" w: "auto" }
            }
            a "RustScript" {
                href: "index.html"
                style {
                    fg: "#f97316"
                    size: "1.1rem"
                    weight: "700"
                    text-decoration: "none"
                }
            }
        }

        div {
            style { display: "flex" gap: "24px" align-items: "center" }

            a "Home" {
                href: "index.html"
                style { fg: "#9ca3af" text-decoration: "none" size: "0.9rem" }
            }
            a "Compiler" {
                href: "compiler.html"
                style { fg: "#f97316" text-decoration: "none" size: "0.9rem" weight: "600" }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  HERO
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "80px 40px 60px"
            align: "center"
            maxw: "900px"
            mx: "auto"
        }

        span "COMPILER DEEP DIVE" {
            style {
                display: "inline-block"
                bg: "rgba(249, 115, 22, 0.1)"
                fg: "#f97316"
                pad: "6px 16px"
                radius: "999px"
                size: "0.75rem"
                weight: "600"
                spacing: "0.1em"
                mb: "24px"
                border: "1px solid rgba(249, 115, 22, 0.25)"
            }
        }

        h1 "How RustScript Works" {
            style {
                size: "2.8rem"
                weight: "800"
                fg: "white"
                lh: "1.15"
                mb: "16px"
            }
        }

        p "A zero-dependency Rust compiler that transforms .rsx source files into self-contained HTML with embedded CSS and JavaScript." {
            style {
                size: "1.15rem"
                fg: "#9ca3af"
                lh: "1.7"
                maxw: "650px"
                mx: "auto"
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  PIPELINE OVERVIEW
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "60px 40px"
            maxw: "900px"
            mx: "auto"
        }

        h2 "The Compilation Pipeline" {
            style { size: "1.8rem" weight: "700" fg: "white" mb: "12px" }
        }

        p "Every .rsx file passes through four distinct stages before becoming a fully functional web page. Each stage is implemented as its own Rust module with zero external dependencies." {
            style { size: "1rem" fg: "#9ca3af" lh: "1.7" mb: "40px" }
        }

        # Pipeline diagram
        div {
            style {
                display: "flex"
                justify-content: "center"
                align-items: "center"
                gap: "0"
                mb: "50px"
                flex-wrap: "wrap"
            }

            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "20px 28px"
                    align: "center"
                    minw: "120px"
                }
                p ".rsx" { style { size: "1.4rem" weight: "700" fg: "#f97316" mb: "4px" } }
                p "Source" { style { size: "0.75rem" fg: "#6b7280" } }
            }

            span " --> " { style { fg: "#374151" size: "1.2rem" pad: "0 8px" } }

            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "20px 28px"
                    align: "center"
                    minw: "120px"
                }
                p "Lexer" { style { size: "1.4rem" weight: "700" fg: "#3b82f6" mb: "4px" } }
                p "Tokens" { style { size: "0.75rem" fg: "#6b7280" } }
            }

            span " --> " { style { fg: "#374151" size: "1.2rem" pad: "0 8px" } }

            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "20px 28px"
                    align: "center"
                    minw: "120px"
                }
                p "Parser" { style { size: "1.4rem" weight: "700" fg: "#22c55e" mb: "4px" } }
                p "AST" { style { size: "0.75rem" fg: "#6b7280" } }
            }

            span " --> " { style { fg: "#374151" size: "1.2rem" pad: "0 8px" } }

            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "20px 28px"
                    align: "center"
                    minw: "120px"
                }
                p "Codegen" { style { size: "1.4rem" weight: "700" fg: "#a855f7" mb: "4px" } }
                p "HTML" { style { size: "0.75rem" fg: "#6b7280" } }
            }
        }

        # ── STAGE 1: LEXER ─────────────────────────────────
        div {
            style {
                bg: "#111827"
                border: "1px solid #1f2937"
                radius: "16px"
                pad: "36px"
                mb: "32px"
            }

            div {
                style { display: "flex" align-items: "center" gap: "12px" mb: "16px" }

                span "01" {
                    style {
                        bg: "rgba(59, 130, 246, 0.15)"
                        fg: "#3b82f6"
                        pad: "4px 12px"
                        radius: "8px"
                        size: "0.8rem"
                        weight: "700"
                    }
                }
                h3 "Lexer — Tokenization" {
                    style { size: "1.3rem" weight: "700" fg: "white" }
                }
            }

            p "The lexer (lexer.rs) scans the raw .rsx source character by character, producing a flat stream of typed tokens. It handles string literals with interpolation, numeric literals (int and float), identifiers, keywords, operators, and delimiters. Comments use the # prefix, like Python." {
                style { fg: "#9ca3af" lh: "1.7" mb: "20px" size: "0.95rem" }
            }

            div {
                style {
                    bg: "#0a0e17"
                    radius: "10px"
                    pad: "20px 24px"
                    border: "1px solid #1f2937"
                    overflow: "auto"
                }
                p "// Token types in token.rs" {
                    style { fg: "#6b7280" size: "0.82rem" font-family: "'JetBrains Mono', monospace" mb: "8px" }
                }
                pre {
                    style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }

                    span "pub enum " { style { fg: "#c084fc" } }
                    span "Token " { style { fg: "#22c55e" } }
                    span "\{\n" { style { fg: "#e5e7eb" } }
                    span "    Int(i64),\n" { style { fg: "#e5e7eb" } }
                    span "    Float(f64),\n" { style { fg: "#e5e7eb" } }
                    span "    Str(String),\n" { style { fg: "#e5e7eb" } }
                    span "    Ident(String),\n" { style { fg: "#e5e7eb" } }
                    span "    // Keywords\n" { style { fg: "#6b7280" } }
                    span "    Let, Fn, Return, If, Else,\n" { style { fg: "#e5e7eb" } }
                    span "    While, For, In, Import, Page,\n" { style { fg: "#e5e7eb" } }
                    span "    Style, On, True, False,\n" { style { fg: "#e5e7eb" } }
                    span "    // Operators\n" { style { fg: "#6b7280" } }
                    span "    Plus, Minus, Star, Slash,\n" { style { fg: "#e5e7eb" } }
                    span "    Eq, NotEq, Lt, Gt, ...\n" { style { fg: "#e5e7eb" } }
                    span "\}" { style { fg: "#e5e7eb" } }
                }
            }
        }

        # ── STAGE 2: PARSER ────────────────────────────────
        div {
            style {
                bg: "#111827"
                border: "1px solid #1f2937"
                radius: "16px"
                pad: "36px"
                mb: "32px"
            }

            div {
                style { display: "flex" align-items: "center" gap: "12px" mb: "16px" }

                span "02" {
                    style {
                        bg: "rgba(34, 197, 94, 0.15)"
                        fg: "#22c55e"
                        pad: "4px 12px"
                        radius: "8px"
                        size: "0.8rem"
                        weight: "700"
                    }
                }
                h3 "Parser — AST Construction" {
                    style { size: "1.3rem" weight: "700" fg: "white" }
                }
            }

            p "The recursive-descent parser (parser.rs) consumes the token stream and constructs a typed Abstract Syntax Tree. It handles statements (let, fn, if/else, while, for, import), expressions (binary ops, calls, method chains, indexing), and page elements (HTML tags with inline styles, events, and conditional/loop rendering)." {
                style { fg: "#9ca3af" lh: "1.7" mb: "20px" size: "0.95rem" }
            }

            div {
                style {
                    bg: "#0a0e17"
                    radius: "10px"
                    pad: "20px 24px"
                    border: "1px solid #1f2937"
                    overflow: "auto"
                }
                p "// AST node types from ast.rs" {
                    style { fg: "#6b7280" size: "0.82rem" font-family: "'JetBrains Mono', monospace" mb: "8px" }
                }
                pre {
                    style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }

                    span "pub enum " { style { fg: "#c084fc" } }
                    span "Stmt " { style { fg: "#22c55e" } }
                    span "\{\n" { style { fg: "#e5e7eb" } }
                    span "    Import \{ path: String \},\n" { style { fg: "#e5e7eb" } }
                    span "    Let \{ name: String, value: Expr \},\n" { style { fg: "#e5e7eb" } }
                    span "    FnDecl \{ name, params, body \},\n" { style { fg: "#e5e7eb" } }
                    span "    If \{ cond, then_body, else_body \},\n" { style { fg: "#e5e7eb" } }
                    span "    Page \{ elements: Vec<Element> \},\n" { style { fg: "#e5e7eb" } }
                    span "    ...\n" { style { fg: "#6b7280" } }
                    span "\}\n\n" { style { fg: "#e5e7eb" } }
                    span "pub enum " { style { fg: "#c084fc" } }
                    span "Element " { style { fg: "#22c55e" } }
                    span "\{\n" { style { fg: "#e5e7eb" } }
                    span "    Tag \{ tag, text, attrs,\n" { style { fg: "#e5e7eb" } }
                    span "          style, events, children \},\n" { style { fg: "#e5e7eb" } }
                    span "    If \{ cond, then_els, else_els \},\n" { style { fg: "#e5e7eb" } }
                    span "    For \{ var, iter, body \},\n" { style { fg: "#e5e7eb" } }
                    span "\}" { style { fg: "#e5e7eb" } }
                }
            }
        }

        # ── STAGE 3: CODEGEN ───────────────────────────────
        div {
            style {
                bg: "#111827"
                border: "1px solid #1f2937"
                radius: "16px"
                pad: "36px"
                mb: "32px"
            }

            div {
                style { display: "flex" align-items: "center" gap: "12px" mb: "16px" }

                span "03" {
                    style {
                        bg: "rgba(168, 85, 247, 0.15)"
                        fg: "#a855f7"
                        pad: "4px 12px"
                        radius: "8px"
                        size: "0.8rem"
                        weight: "700"
                    }
                }
                h3 "Codegen — HTML Generation" {
                    style { size: "1.3rem" weight: "700" fg: "white" }
                }
            }

            p "The code generator (codegen.rs) walks the AST and emits a self-contained HTML document. Variables become JavaScript, functions compile to JS functions, page elements become DOM nodes, inline styles are compiled to CSS, and event handlers wire up addEventListener calls. The output is a single .html file with everything embedded — no external dependencies." {
                style { fg: "#9ca3af" lh: "1.7" mb: "20px" size: "0.95rem" }
            }

            div {
                style {
                    bg: "#0a0e17"
                    radius: "10px"
                    pad: "20px 24px"
                    border: "1px solid #1f2937"
                    overflow: "auto"
                }
                p "// Style shorthand system from codegen.rs" {
                    style { fg: "#6b7280" size: "0.82rem" font-family: "'JetBrains Mono', monospace" mb: "8px" }
                }
                pre {
                    style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }

                    span "fn " { style { fg: "#c084fc" } }
                    span "map_style_prop" { style { fg: "#3b82f6" } }
                    span "(name, value) \{\n" { style { fg: "#e5e7eb" } }
                    span "    match name \{\n" { style { fg: "#e5e7eb" } }
                    span "        " { style { fg: "#e5e7eb" } }
                    span "\"size\"" { style { fg: "#f97316" } }
                    span "  => font-size,\n" { style { fg: "#e5e7eb" } }
                    span "        " { style { fg: "#e5e7eb" } }
                    span "\"bg\"" { style { fg: "#f97316" } }
                    span "    => background,\n" { style { fg: "#e5e7eb" } }
                    span "        " { style { fg: "#e5e7eb" } }
                    span "\"fg\"" { style { fg: "#f97316" } }
                    span "    => color,\n" { style { fg: "#e5e7eb" } }
                    span "        " { style { fg: "#e5e7eb" } }
                    span "\"pad\"" { style { fg: "#f97316" } }
                    span "   => padding,\n" { style { fg: "#e5e7eb" } }
                    span "        " { style { fg: "#e5e7eb" } }
                    span "\"row\"" { style { fg: "#f97316" } }
                    span "   => display:flex +\n" { style { fg: "#e5e7eb" } }
                    span "              flex-direction:row,\n" { style { fg: "#e5e7eb" } }
                    span "        _ => pass through as-is,\n" { style { fg: "#6b7280" } }
                    span "    \}\n" { style { fg: "#e5e7eb" } }
                    span "\}" { style { fg: "#e5e7eb" } }
                }
            }
        }

        # ── STAGE 4: IMPORT RESOLUTION ─────────────────────
        div {
            style {
                bg: "#111827"
                border: "1px solid #1f2937"
                radius: "16px"
                pad: "36px"
                mb: "32px"
            }

            div {
                style { display: "flex" align-items: "center" gap: "12px" mb: "16px" }

                span "04" {
                    style {
                        bg: "rgba(249, 115, 22, 0.15)"
                        fg: "#f97316"
                        pad: "4px 12px"
                        radius: "8px"
                        size: "0.8rem"
                        weight: "700"
                    }
                }
                h3 "Import Resolution" {
                    style { size: "1.3rem" weight: "700" fg: "white" }
                }
            }

            p "Before code generation, the compiler resolves all import statements at the AST level. Imported .rsx files are parsed and their statements are inlined in place. Image files (.png, .jpg, .svg, etc.) are automatically base64-encoded and injected as data URI variables. A HashSet tracks visited paths to prevent circular imports." {
                style { fg: "#9ca3af" lh: "1.7" mb: "20px" size: "0.95rem" }
            }

            div {
                style {
                    bg: "#0a0e17"
                    radius: "10px"
                    pad: "20px 24px"
                    border: "1px solid #1f2937"
                    overflow: "auto"
                }
                p "// Import resolution from main.rs" {
                    style { fg: "#6b7280" size: "0.82rem" font-family: "'JetBrains Mono', monospace" mb: "8px" }
                }
                pre {
                    style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }

                    span "fn " { style { fg: "#c084fc" } }
                    span "resolve_imports" { style { fg: "#3b82f6" } }
                    span "(prog, base, seen) \{\n" { style { fg: "#e5e7eb" } }
                    span "    for stmt in prog.stmts \{\n" { style { fg: "#e5e7eb" } }
                    span "        if let Import \{ path \} = stmt \{\n" { style { fg: "#e5e7eb" } }
                    span "            if " { style { fg: "#e5e7eb" } }
                    span "is_image_ext" { style { fg: "#3b82f6" } }
                    span "(ext) \{\n" { style { fg: "#e5e7eb" } }
                    span "                // Read + base64 encode\n" { style { fg: "#6b7280" } }
                    span "                let data = base64(bytes);\n" { style { fg: "#e5e7eb" } }
                    span "                let var = var_name(path);\n" { style { fg: "#e5e7eb" } }
                    span "                // Inject: let logo = \"data:...\";\n" { style { fg: "#6b7280" } }
                    span "            \} else \{\n" { style { fg: "#e5e7eb" } }
                    span "                // Parse .rsx + inline stmts\n" { style { fg: "#6b7280" } }
                    span "                if !seen.contains(path) \{\n" { style { fg: "#e5e7eb" } }
                    span "                    seen.insert(path);\n" { style { fg: "#e5e7eb" } }
                    span "                    resolve_imports(child);\n" { style { fg: "#e5e7eb" } }
                    span "                \}\n" { style { fg: "#e5e7eb" } }
                    span "            \}\n" { style { fg: "#e5e7eb" } }
                    span "        \}\n" { style { fg: "#e5e7eb" } }
                    span "    \}\n" { style { fg: "#e5e7eb" } }
                    span "\}" { style { fg: "#e5e7eb" } }
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  STRING INTERPOLATION & REACTIVITY
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "60px 40px"
            maxw: "900px"
            mx: "auto"
        }

        h2 "String Interpolation to Reactive DOM" {
            style { size: "1.8rem" weight: "700" fg: "white" mb: "12px" }
        }

        p "When you write a string like \"Count: \{count\}\" in a page element, the compiler does not simply concatenate at build time. It emits a DOM span with a unique ID for each interpolated variable, then generates a JavaScript update function that re-renders the span whenever the variable changes. This is how RustScript achieves reactivity with zero framework overhead." {
            style { size: "1rem" fg: "#9ca3af" lh: "1.7" mb: "32px" }
        }

        div {
            style { display: "grid" grid-template-columns: "1fr 1fr" gap: "20px" mb: "32px" }

            # What you write
            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "24px"
                }

                p "What you write" {
                    style { fg: "#f97316" weight: "600" size: "0.9rem" mb: "12px" }
                }

                div {
                    style {
                        bg: "#0a0e17"
                        radius: "8px"
                        pad: "16px"
                        border: "1px solid #1f2937"
                    }
                    pre {
                        style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }
                        span "h1 " { style { fg: "#3b82f6" } }
                        span "\"Score: \{points\}\"" { style { fg: "#f97316" } }
                    }
                }
            }

            # What the compiler emits
            div {
                style {
                    bg: "#111827"
                    border: "1px solid #1f2937"
                    radius: "12px"
                    pad: "24px"
                }

                p "What the compiler emits" {
                    style { fg: "#22c55e" weight: "600" size: "0.9rem" mb: "12px" }
                }

                div {
                    style {
                        bg: "#0a0e17"
                        radius: "8px"
                        pad: "16px"
                        border: "1px solid #1f2937"
                    }
                    pre {
                        style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }
                        span "<!-- HTML output -->\n" { style { fg: "#6b7280" } }
                        span "<h1>" { style { fg: "#3b82f6" } }
                        span "Score: " { style { fg: "#e5e7eb" } }
                        span "<span id=_v0>" { style { fg: "#3b82f6" } }
                        span "0" { style { fg: "#f97316" } }
                        span "</span>" { style { fg: "#3b82f6" } }
                        span "</h1>" { style { fg: "#e5e7eb" } }
                        span "\n\n" { style { fg: "#e5e7eb" } }
                        span "/* Generated JS */\n" { style { fg: "#6b7280" } }
                        span "let points = 0;\n" { style { fg: "#e5e7eb" } }
                        span "function " { style { fg: "#c084fc" } }
                        span "_update() " { style { fg: "#3b82f6" } }
                        span "\{\n" { style { fg: "#e5e7eb" } }
                        span "  _v0.textContent = points;\n" { style { fg: "#e5e7eb" } }
                        span "\}" { style { fg: "#e5e7eb" } }
                    }
                }
            }
        }

        p "Every assignment to a reactive variable automatically calls the update function. No virtual DOM, no diffing, no subscriptions — just direct DOM mutation triggered by the compiled setter." {
            style { size: "0.95rem" fg: "#6b7280" lh: "1.7" font-style: "italic" }
        }
    }

    # ════════════════════════════════════════════════════════
    #  EVENT COMPILATION
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "60px 40px"
            maxw: "900px"
            mx: "auto"
        }

        h2 "Event Handlers" {
            style { size: "1.8rem" weight: "700" fg: "white" mb: "12px" }
        }

        p "RustScript event blocks compile to addEventListener calls. The compiler walks each statement in the on block, translates RustScript assignments and expressions into JavaScript, and appends the reactive update call so the UI stays in sync." {
            style { size: "1rem" fg: "#9ca3af" lh: "1.7" mb: "32px" }
        }

        div {
            style {
                bg: "#111827"
                border: "1px solid #1f2937"
                radius: "16px"
                pad: "36px"
            }

            div {
                style { display: "grid" grid-template-columns: "1fr 1fr" gap: "20px" }

                div {
                    style {
                        bg: "#0a0e17"
                        radius: "10px"
                        pad: "20px"
                        border: "1px solid #1f2937"
                    }
                    p ".rsx source" { style { fg: "#f97316" size: "0.8rem" weight: "600" mb: "12px" } }
                    pre {
                        style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }
                        span "button " { style { fg: "#3b82f6" } }
                        span "\"Add\" \{\n" { style { fg: "#f97316" } }
                        span "  on click " { style { fg: "#c084fc" } }
                        span "\{\n" { style { fg: "#e5e7eb" } }
                        span "    count += 1\n" { style { fg: "#e5e7eb" } }
                        span "    total = count * price\n" { style { fg: "#e5e7eb" } }
                        span "  \}\n" { style { fg: "#e5e7eb" } }
                        span "\}" { style { fg: "#e5e7eb" } }
                    }
                }

                div {
                    style {
                        bg: "#0a0e17"
                        radius: "10px"
                        pad: "20px"
                        border: "1px solid #1f2937"
                    }
                    p "Compiled output" { style { fg: "#22c55e" size: "0.8rem" weight: "600" mb: "12px" } }
                    pre {
                        style { fg: "#e5e7eb" size: "0.82rem" font-family: "'JetBrains Mono', monospace" lh: "1.7" white-space: "pre" }
                        span "el.addEventListener(\n" { style { fg: "#e5e7eb" } }
                        span "  " { style { fg: "#e5e7eb" } }
                        span "\"click\"" { style { fg: "#f97316" } }
                        span ", () => \{\n" { style { fg: "#e5e7eb" } }
                        span "    count += 1;\n" { style { fg: "#e5e7eb" } }
                        span "    total = count * price;\n" { style { fg: "#e5e7eb" } }
                        span "    " { style { fg: "#e5e7eb" } }
                        span "_update();" { style { fg: "#3b82f6" } }
                        span "\n  \});" { style { fg: "#e5e7eb" } }
                    }
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  ARCHITECTURE AT A GLANCE
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "60px 40px"
            maxw: "900px"
            mx: "auto"
        }

        h2 "Architecture at a Glance" {
            style { size: "1.8rem" weight: "700" fg: "white" mb: "32px" }
        }

        div {
            style {
                display: "grid"
                grid-template-columns: "repeat(4, 1fr)"
                gap: "16px"
                mb: "32px"
            }

            div {
                style { bg: "#111827" border: "1px solid #1f2937" radius: "12px" pad: "20px" align: "center" }
                p "8" { style { size: "2rem" weight: "700" fg: "#f97316" mb: "4px" } }
                p "Source Modules" { style { size: "0.8rem" fg: "#6b7280" } }
            }

            div {
                style { bg: "#111827" border: "1px solid #1f2937" radius: "12px" pad: "20px" align: "center" }
                p "0" { style { size: "2rem" weight: "700" fg: "#22c55e" mb: "4px" } }
                p "Dependencies" { style { size: "0.8rem" fg: "#6b7280" } }
            }

            div {
                style { bg: "#111827" border: "1px solid #1f2937" radius: "12px" pad: "20px" align: "center" }
                p "~3,500" { style { size: "2rem" weight: "700" fg: "#3b82f6" mb: "4px" } }
                p "Lines of Rust" { style { size: "0.8rem" fg: "#6b7280" } }
            }

            div {
                style { bg: "#111827" border: "1px solid #1f2937" radius: "12px" pad: "20px" align: "center" }
                p "2" { style { size: "2rem" weight: "700" fg: "#a855f7" mb: "4px" } }
                p "Output Modes" { style { size: "0.8rem" fg: "#6b7280" } }
            }
        }

        p "The entire compiler ships as a single Rust binary built with cargo. HTML codegen for web output, or a tree-walking interpreter for terminal execution. Both paths share the same lexer, parser, and AST." {
            style { size: "0.95rem" fg: "#6b7280" lh: "1.7" align: "center" }
        }

        div {
            style { align: "center" mt: "24px" }

            a "View source on GitHub" {
                href: "https://github.com/hansdevs/rustscript"
                style {
                    display: "inline-block"
                    bg: "#111827"
                    fg: "#f97316"
                    border: "1px solid #1f2937"
                    pad: "12px 28px"
                    radius: "8px"
                    text-decoration: "none"
                    size: "0.9rem"
                    weight: "600"
                }
            }
        }
    }

    # ════════════════════════════════════════════════════════
    #  FOOTER
    # ════════════════════════════════════════════════════════
    div {
        style {
            pad: "40px"
            align: "center"
            border-top: "1px solid #1f2937"
        }

        p "RustScript Compiler v0.1.0 — Hackathon 2026" {
            style { fg: "#4b5563" size: "0.85rem" mb: "4px" }
        }
        p "Zero dependencies. One file in, one file out." {
            style { fg: "#374151" size: "0.75rem" }
        }
    }
}

# ──────────────────────────────────────────────────────────────
#  RustScript — Language Showcase & Landing Page
#  Preview:  rustscript preview test.rsx
# ──────────────────────────────────────────────────────────────

# ─── Fibonacci Sequence ─────────────────────────────────────
fn fib(n) {
    if n <= 1 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

fn oogabooga (){
    println("This is a function with a silly name!")
}

# ─── Factorial ──────────────────────────────────────────────
fn factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

# ─── FizzBuzz ───────────────────────────────────────────────
fn fizzbuzz(n) {
    if n % 15 == 0 {
        return "FizzBuzz"
    }
    if n % 3 == 0 {
        return "Fizz"
    }
    if n % 5 == 0 {
        return "Buzz"
    }
    return str(n)
}

# ─── State ──────────────────────────────────────────────────
let fib_count = 10
let fact_n = 6
let fizz_n = 15
let active_tab = "fibonacci"

# Pre-compute our sequences into display strings
let fib_seq = []
for i in range(fib_count) {
    push(fib_seq, fib(i))
}

let fizz_seq = []
for i in range(fizz_n) {
    push(fizz_seq, fizzbuzz(i + 1))
}

# ─── Page ───────────────────────────────────────────────────
page {

    style {
        bg: "#0a0a0f"
        fg: "#e2e8f0"
        font: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
        pad: "0"
    }

    # ── Hero Section ────────────────────────────────────────
    div {
        style {
            bg: "linear-gradient(135deg, #0d1117 0%, #161b22 50%, #1a1025 100%)"
            pad: "80px 40px"
            align: "center"
            min-height: "100vh"
        }

        div {
            style {
                maxw: "900px"
                m: "0 auto"
            }

            # Badge
            div {
                style {
                    display: "inline-block"
                    bg: "rgba(249, 115, 22, 0.1)"
                    border: "1px solid rgba(249, 115, 22, 0.3)"
                    radius: "999px"
                    pad: "6px 20px"
                    mb: "32px"
                    size: "0.85rem"
                    fg: "#f97316"
                }
                text "Turing Complete · v0.1.0"
            }

            # Title
            h1 "RustScript" {
                style {
                    size: "5rem"
                    weight: "800"
                    lh: "1.1"
                    mb: "16px"
                    bg: "linear-gradient(135deg, #f97316, #ef4444, #ec4899)"
                    background-clip: "text"
                    fg: "transparent"
                }
            }

            p "Where HTML, CSS & Python had a child." {
                style {
                    size: "1.5rem"
                    fg: "#8b949e"
                    mb: "12px"
                    weight: "300"
                }
            }

            p "Write frontend apps in a single .rsx file. Logic, layout, and style — unified." {
                style {
                    size: "1.1rem"
                    fg: "#6e7681"
                    mb: "48px"
                    maxw: "600px"
                    m: "0 auto 48px auto"
                    lh: "1.6"
                }
            }

            # CTA Buttons
            div {
                style {
                    row
                    center
                    gap: "16px"
                    mb: "64px"
                }

                button "rustscript preview app.rsx" {
                    style {
                        bg: "linear-gradient(135deg, #f97316, #ef4444)"
                        fg: "white"
                        border: "none"
                        pad: "14px 32px"
                        radius: "12px"
                        size: "1rem"
                        weight: "600"
                        pointer
                        shadow: "0 4px 24px rgba(249, 115, 22, 0.3)"
                        transition: "all 0.2s ease"
                    }
                }

                button "curl -fsSL …/install.sh | sh" {
                    style {
                        bg: "rgba(255,255,255,0.05)"
                        fg: "#e2e8f0"
                        border: "1px solid #30363d"
                        pad: "14px 32px"
                        radius: "12px"
                        size: "1rem"
                        weight: "500"
                        pointer
                        transition: "all 0.2s ease"
                    }
                }
            }

            # ── Feature Cards ───────────────────────────────
            div {
                style {
                    display: "grid"
                    cols: "repeat(3, 1fr)"
                    gap: "20px"
                    mb: "80px"
                    align: "left"
                }

                # Card 1: Comments
                div {
                    style {
                        bg: "rgba(255,255,255,0.03)"
                        border: "1px solid #21262d"
                        radius: "16px"
                        pad: "28px"
                        transition: "border-color 0.2s"
                    }

                    p "💬" { style { size: "2rem" mb: "12px" } }
                    h3 "Comments" { style { size: "1.15rem" mb: "8px" fg: "#f0f6fc" } }
                    p "Use # for line comments. Clean, Python-style." {
                        style { fg: "#8b949e" size: "0.9rem" lh: "1.5" }
                    }

                    div {
                        style {
                            bg: "#0d1117"
                            radius: "8px"
                            pad: "16px"
                            mt: "16px"
                            font: "'SF Mono', 'Fira Code', 'JetBrains Mono', monospace"
                            size: "0.8rem"
                            lh: "1.7"
                        }
                        p "# This is a comment" { style { fg: "#6a9955" } }
                        p "let x = 42" { style { fg: "#e2e8f0" } }
                        p "# Variables are dynamic" { style { fg: "#6a9955" } }
                        p "let name = \"world\"" { style { fg: "#e2e8f0" } }
                    }
                }

                # Card 2: Functions
                div {
                    style {
                        bg: "rgba(255,255,255,0.03)"
                        border: "1px solid #21262d"
                        radius: "16px"
                        pad: "28px"
                        transition: "border-color 0.2s"
                    }

                    p ">" { style { size: "2rem" mb: "12px" } }
                    h3 "Functions" { style { size: "1.15rem" mb: "8px" fg: "#f0f6fc" } }
                    p "First-class functions with recursion. Turing complete." {
                        style { fg: "#8b949e" size: "0.9rem" lh: "1.5" }
                    }

                    div {
                        style {
                            bg: "#0d1117"
                            radius: "8px"
                            pad: "16px"
                            mt: "16px"
                            font: "'SF Mono', 'Fira Code', 'JetBrains Mono', monospace"
                            size: "0.8rem"
                            lh: "1.7"
                        }
                        p "fn fib(n) \{" { style { fg: "#e2e8f0" } }
                        p "  if n <= 1 \{ return n \}" { style { fg: "#e2e8f0" } }
                        p "  return fib(n-1) + fib(n-2)" { style { fg: "#e2e8f0" } }
                        p "\}" { style { fg: "#e2e8f0" } }
                    }
                }

                # Card 3: Styling
                div {
                    style {
                        bg: "rgba(255,255,255,0.03)"
                        border: "1px solid #21262d"
                        radius: "16px"
                        pad: "28px"
                        transition: "border-color 0.2s"
                    }

                    p "🎨" { style { size: "2rem" mb: "12px" } }
                    h3 "Custom Styles" { style { size: "1.15rem" mb: "8px" fg: "#f0f6fc" } }
                    p "60+ shorthands. bg, pad, radius, bold, center..." {
                        style { fg: "#8b949e" size: "0.9rem" lh: "1.5" }
                    }

                    div {
                        style {
                            bg: "#0d1117"
                            radius: "8px"
                            pad: "16px"
                            mt: "16px"
                            font: "'SF Mono', 'Fira Code', 'JetBrains Mono', monospace"
                            size: "0.8rem"
                            lh: "1.7"
                        }
                        p "div \{" { style { fg: "#e2e8f0" } }
                        p "  style \{" { style { fg: "#e2e8f0" } }
                        p "    bg: \"#161b22\"" { style { fg: "#ce9178" } }
                        p "    radius: \"12px\"" { style { fg: "#ce9178" } }
                        p "    center" { style { fg: "#569cd6" } }
                        p "    bold" { style { fg: "#569cd6" } }
                        p "  \}" { style { fg: "#e2e8f0" } }
                        p "\}" { style { fg: "#e2e8f0" } }
                    }
                }
            }

            # ── Live Demos Section ──────────────────────────
            h2 "Live Demos" {
                style {
                    size: "2.2rem"
                    weight: "700"
                    mb: "8px"
                    fg: "#f0f6fc"
                    align: "center"
                }
            }
            p "These are computed at build time by RustScript  — not hard-coded." {
                style {
                    fg: "#6e7681"
                    mb: "40px"
                    size: "1rem"
                    align: "center"
                }
            }

            # Tab buttons
            div {
                style {
                    row
                    center
                    gap: "8px"
                    mb: "32px"
                }

                button "Fibonacci" {
                    style {
                        bg: "#238636"
                        fg: "white"
                        border: "none"
                        pad: "10px 24px"
                        radius: "8px"
                        size: "0.95rem"
                        pointer
                        weight: "500"
                    }
                    on click {
                        active_tab = "fibonacci"
                    }
                }
                button "Factorial" {
                    style {
                        bg: "#30363d"
                        fg: "#e2e8f0"
                        border: "1px solid #484f58"
                        pad: "10px 24px"
                        radius: "8px"
                        size: "0.95rem"
                        pointer
                        weight: "500"
                    }
                    on click {
                        active_tab = "factorial"
                    }
                }
                button "FizzBuzz" {
                    style {
                        bg: "#30363d"
                        fg: "#e2e8f0"
                        border: "1px solid #484f58"
                        pad: "10px 24px"
                        radius: "8px"
                        size: "0.95rem"
                        pointer
                        weight: "500"
                    }
                    on click {
                        active_tab = "fizzbuzz"
                    }
                }
            }

            # ── Fibonacci Panel ─────────────────────────────
            if active_tab == "fibonacci" {
                div {
                    style {
                        bg: "rgba(255,255,255,0.03)"
                        border: "1px solid #21262d"
                        radius: "16px"
                        pad: "32px"
                        align: "left"
                        mb: "24px"
                    }

                    div {
                        style { row justify: "space-between" items: "center" mb: "20px" }
                        h3 "Fibonacci Sequence" { style { fg: "#f0f6fc" size: "1.3rem" } }
                        p "fib(0) → fib({fib_count - 1})" { style { fg: "#6e7681" size: "0.9rem" } }
                    }

                    div {
                        style {
                            row
                            wrap: "wrap"
                            gap: "10px"
                        }

                        for val in fib_seq {
                            div {
                                style {
                                    bg: "rgba(35, 134, 54, 0.15)"
                                    border: "1px solid rgba(35, 134, 54, 0.3)"
                                    radius: "10px"
                                    pad: "12px 18px"
                                    center
                                    min-width: "56px"
                                }
                                p "{val}" {
                                    style {
                                        font: "'SF Mono', 'Fira Code', monospace"
                                        size: "1.1rem"
                                        weight: "600"
                                        fg: "#3fb950"
                                    }
                                }
                            }
                        }
                    }

                    div {
                        style {
                            bg: "#0d1117"
                            radius: "10px"
                            pad: "20px"
                            mt: "24px"
                            font: "'SF Mono', 'Fira Code', monospace"
                            size: "0.85rem"
                            lh: "1.8"
                        }
                        p "# Recursive Fibonacci" { style { fg: "#6a9955" } }
                        p "fn fib(n) \{" { style { fg: "#e2e8f0" } }
                        p "    if n <= 1 \{ return n \}" { style { fg: "#e2e8f0" } }
                        p "    return fib(n - 1) + fib(n - 2)" { style { fg: "#e2e8f0" } }
                        p "\}" { style { fg: "#e2e8f0" } }
                    }
                }
            }

            # ── Factorial Panel ─────────────────────────────
            if active_tab == "factorial" {
                div {
                    style {
                        bg: "rgba(255,255,255,0.03)"
                        border: "1px solid #21262d"
                        radius: "16px"
                        pad: "32px"
                        align: "left"
                        mb: "24px"
                    }

                    div {
                        style { row justify: "space-between" items: "center" mb: "20px" }
                        h3 "Factorial" { style { fg: "#f0f6fc" size: "1.3rem" } }
                        p "{fact_n}! = {factorial(fact_n)}" {
                            style {
                                fg: "#58a6ff"
                                font: "'SF Mono', 'Fira Code', monospace"
                                size: "1.1rem"
                                weight: "600"
                            }
                        }
                    }

                    div {
                        style {
                            bg: "rgba(88, 166, 255, 0.08)"
                            border: "1px solid rgba(88, 166, 255, 0.2)"
                            radius: "12px"
                            pad: "24px"
                            align: "center"
                        }

                        p "{fact_n}! = {factorial(fact_n)}" {
                            style {
                                size: "3rem"
                                weight: "700"
                                fg: "#58a6ff"
                                font: "'SF Mono', 'Fira Code', monospace"
                            }
                        }
                    }

                    div {
                        style { row center gap: "12px" mt: "20px" }
                        button "−" {
                            style {
                                bg: "#da3633" fg: "white" border: "none"
                                pad: "8px 20px" radius: "8px" size: "1.2rem" pointer
                            }
                            on click {
                                if fact_n > 0 {
                                    fact_n = fact_n - 1
                                }
                            }
                        }
                        p "n = {fact_n}" {
                            style {
                                fg: "#8b949e"
                                size: "1rem"
                                font: "'SF Mono', monospace"
                                minw: "60px"
                                align: "center"
                            }
                        }
                        button "+" {
                            style {
                                bg: "#238636" fg: "white" border: "none"
                                pad: "8px 20px" radius: "8px" size: "1.2rem" pointer
                            }
                            on click {
                                if fact_n < 12 {
                                    fact_n = fact_n + 1
                                }
                            }
                        }
                    }

                    div {
                        style {
                            bg: "#0d1117"
                            radius: "10px"
                            pad: "20px"
                            mt: "24px"
                            font: "'SF Mono', 'Fira Code', monospace"
                            size: "0.85rem"
                            lh: "1.8"
                        }
                        p "# Recursive Factorial" { style { fg: "#6a9955" } }
                        p "fn factorial(n) \{" { style { fg: "#e2e8f0" } }
                        p "    if n <= 1 \{ return 1 \}" { style { fg: "#e2e8f0" } }
                        p "    return n * factorial(n - 1)" { style { fg: "#e2e8f0" } }
                        p "\}" { style { fg: "#e2e8f0" } }
                    }
                }
            }

            # ── FizzBuzz Panel ──────────────────────────────
            if active_tab == "fizzbuzz" {
                div {
                    style {
                        bg: "rgba(255,255,255,0.03)"
                        border: "1px solid #21262d"
                        radius: "16px"
                        pad: "32px"
                        align: "left"
                        mb: "24px"
                    }

                    div {
                        style { row justify: "space-between" items: "center" mb: "20px" }
                        h3 "FizzBuzz" { style { fg: "#f0f6fc" size: "1.3rem" } }
                        p "1 → {fizz_n}" { style { fg: "#6e7681" size: "0.9rem" } }
                    }

                    div {
                        style {
                            row
                            wrap: "wrap"
                            gap: "8px"
                        }

                        for val in fizz_seq {
                            div {
                                style {
                                    radius: "8px"
                                    pad: "8px 14px"
                                    center
                                    min-width: "72px"
                                    bg: "rgba(236, 72, 153, 0.1)"
                                    border: "1px solid rgba(236, 72, 153, 0.25)"
                                }
                                p "{val}" {
                                    style {
                                        font: "'SF Mono', 'Fira Code', monospace"
                                        size: "0.9rem"
                                        weight: "500"
                                        fg: "#ec4899"
                                    }
                                }
                            }
                        }
                    }

                    div {
                        style {
                            bg: "#0d1117"
                            radius: "10px"
                            pad: "20px"
                            mt: "24px"
                            font: "'SF Mono', 'Fira Code', monospace"
                            size: "0.85rem"
                            lh: "1.8"
                        }
                        p "# Classic FizzBuzz" { style { fg: "#6a9955" } }
                        p "fn fizzbuzz(n) \{" { style { fg: "#e2e8f0" } }
                        p "    if n % 15 == 0 \{ return \"FizzBuzz\" \}" { style { fg: "#e2e8f0" } }
                        p "    if n % 3 == 0  \{ return \"Fizz\" \}" { style { fg: "#e2e8f0" } }
                        p "    if n % 5 == 0  \{ return \"Buzz\" \}" { style { fg: "#e2e8f0" } }
                        p "    return str(n)" { style { fg: "#e2e8f0" } }
                        p "\}" { style { fg: "#e2e8f0" } }
                    }
                }
            }

            # ── Language Features Grid ──────────────────────
            h2 "Everything in One File" {
                style {
                    size: "2.2rem"
                    weight: "700"
                    mt: "60px"
                    mb: "32px"
                    fg: "#f0f6fc"
                    align: "center"
                }
            }

            div {
                style {
                    display: "grid"
                    cols: "repeat(2, 1fr)"
                    gap: "16px"
                    mb: "80px"
                    align: "left"
                }

                # Feature: Variables
                div {
                    style { bg: "rgba(255,255,255,0.02)" border: "1px solid #21262d" radius: "12px" pad: "20px" }
                    p "Variables & Types" { style { fg: "#f0f6fc" weight: "600" mb: "6px" } }
                    p "let, int, float, str, bool, lists — all dynamic" { style { fg: "#6e7681" size: "0.85rem" } }
                }

                # Feature: Control Flow
                div {
                    style { bg: "rgba(255,255,255,0.02)" border: "1px solid #21262d" radius: "12px" pad: "20px" }
                    p "Control Flow" { style { fg: "#f0f6fc" weight: "600" mb: "6px" } }
                    p "if / else, while, for...in — Python-style blocks" { style { fg: "#6e7681" size: "0.85rem" } }
                }

                # Feature: Functions
                div {
                    style { bg: "rgba(255,255,255,0.02)" border: "1px solid #21262d" radius: "12px" pad: "20px" }
                    p "Functions & Recursion" { style { fg: "#f0f6fc" weight: "600" mb: "6px" } }
                    p "fn keyword, return values, full recursion support" { style { fg: "#6e7681" size: "0.85rem" } }
                }

                # Feature: Reactive UI
                div {
                    style { bg: "rgba(255,255,255,0.02)" border: "1px solid #21262d" radius: "12px" pad: "20px" }
                    p "Reactive UI" { style { fg: "#f0f6fc" weight: "600" mb: "6px" } }
                    p "on click / on input — auto re-renders on state change" { style { fg: "#6e7681" size: "0.85rem" } }
                }

                # Feature: Custom Styles
                div {
                    style { bg: "rgba(255,255,255,0.02)" border: "1px solid #21262d" radius: "12px" pad: "20px" }
                    p "60+ Style Shorthands" { style { fg: "#f0f6fc" weight: "600" mb: "6px" } }
                    p "bg, fg, pad, radius, bold, center, row, col..." { style { fg: "#6e7681" size: "0.85rem" } }
                }

                # Feature: String Interpolation
                div {
                    style { bg: "rgba(255,255,255,0.02)" border: "1px solid #21262d" radius: "12px" pad: "20px" }
                    p "String Interpolation" { style { fg: "#f0f6fc" weight: "600" mb: "6px" } }
                    p "\"Hello {name}!\" — embed expressions in strings" { style { fg: "#6e7681" size: "0.85rem" } }
                }
            }

            # ── Footer ──────────────────────────────────────
            div {
                style {
                    border-top: "1px solid #21262d"
                    pt: "32px"
                    mt: "20px"
                    align: "center"
                }
                p "Built with RustScript" {
                    style { fg: "#484f58" size: "0.9rem" }
                }
                p "This entire page is a single .rsx file." {
                    style { fg: "#30363d" size: "0.8rem" mt: "8px" }
                }
            }
        }
    }
}

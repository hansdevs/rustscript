# ─────────────────────────────────────────────────────────
#  RustScript — Interactive Counter App
#  Build with:  rustscript build app.rsx
#  Then open app.html in your browser!
# ─────────────────────────────────────────────────────────

# ─── State ──────────────────────────────────────────────
let count = 0
let items = ["Learn RustScript", "Build something cool", "Ship it"]

# ─── Logic ──────────────────────────────────────────────
fn get_label(n) {
    if n == 1 {
        return "time"
    }
    return "times"
}

# ─── Page (the UI) ──────────────────────────────────────
page {

    style {
        bg: "#0d1117"
        fg: "#e6edf3"
        font: "'Inter', -apple-system, sans-serif"
        pad: "40px"
    }

    div {
        style {
            maxw: "600px"
            m: "0 auto"
            align: "center"
        }

        h1 "RustScript" {
            style {
                size: "3rem"
                mb: "8px"
                bg: "linear-gradient(135deg, #f97316, #ef4444)"
                background-clip: "text"
                fg: "transparent"
            }
        }

        p "Where HTML, CSS & Python had a child" {
            style {
                fg: "#8b949e"
                mb: "32px"
                size: "1.1rem"
            }
        }

        div {
            style {
                bg: "#161b22"
                border: "1px solid #30363d"
                radius: "12px"
                pad: "32px"
                mb: "24px"
            }

            h2 "Counter" {
                style {
                    mb: "16px"
                    size: "1.5rem"
                }
            }

            p "{count}" {
                style {
                    size: "4rem"
                    bold
                    mb: "8px"
                    fg: "#58a6ff"
                }
            }

            p "Clicked {count} {get_label(count)}" {
                style {
                    fg: "#8b949e"
                    mb: "24px"
                }
            }

            button "- Decrease" {
                style {
                    bg: "#da3633"
                    fg: "white"
                    border: "none"
                    pad: "10px 24px"
                    radius: "8px"
                    size: "1rem"
                    pointer
                    mx: "8px"
                }
                on click {
                    count = count - 1
                }
            }

            button "Reset" {
                style {
                    bg: "#30363d"
                    fg: "#e6edf3"
                    border: "1px solid #484f58"
                    pad: "10px 24px"
                    radius: "8px"
                    size: "1rem"
                    pointer
                    mx: "8px"
                }
                on click {
                    count = 0
                }
            }

            button "+ Increase" {
                style {
                    bg: "#238636"
                    fg: "white"
                    border: "none"
                    pad: "10px 24px"
                    radius: "8px"
                    size: "1rem"
                    pointer
                    mx: "8px"
                }
                on click {
                    count = count + 1
                }
            }
        }

        div {
            style {
                bg: "#161b22"
                border: "1px solid #30363d"
                radius: "12px"
                pad: "32px"
                align: "left"
            }

            h2 "Todo List" {
                style {
                    mb: "16px"
                    size: "1.5rem"
                }
            }

            for item in items {
                p "• {item}" {
                    style {
                        py: "8px"
                        border-bottom: "1px solid #21262d"
                        fg: "#c9d1d9"
                    }
                }
            }

            if count > 10 {
                p "You clicked more than 10 times! Achievement unlocked!" {
                    style {
                        mt: "16px"
                        pad: "12px"
                        bg: "#1f2a1f"
                        border: "1px solid #238636"
                        radius: "8px"
                        fg: "#3fb950"
                    }
                }
            }
        }
    }
}

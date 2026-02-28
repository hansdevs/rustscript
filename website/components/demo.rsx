# ── Code Demo Section ────────────────────────────────────
# Live interactive demo state and logic.

let demo_count = 0

fn get_click_text(n) {
    if n == 0 {
        return "Click the button below"
    }
    if n == 1 {
        return "Clicked 1 time"
    }
    return "Clicked " + str(n) + " times"
}

# Code sample strings shown in the "source" panel
let sample_logic = "let count = 0\n\nfn get_click_text(n) \{\n    if n == 1 \{\n        return \"1 time\"\n    \}\n    return str(n) + \" times\"\n\}"

let sample_ui = "page \{\n    h2 \"Counter: \{count\}\"\n    button \"Click me\" \{\n        on click \{\n            count = count + 1\n        \}\n    \}\n\}"

let active_sample = "logic"

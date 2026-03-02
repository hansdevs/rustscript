# RustScript

A compiled language that turns a single `.rsx` file into a self-contained HTML page. Python-like syntax. Built-in UI. No framework needed.

**No Rust toolchain required.** Download a single binary and go.

```
.rsx source → Lexer → Tokens → Parser → AST ─→ Codegen → HTML/CSS/JS
                                                 ↓
                                           Interpreter (run mode)
```

---

## Table of Contents

- [Install](#install)
- [Quick Start](#quick-start)
- [CLI Commands](#cli-commands)
- [Language Reference](#language-reference)
  - [Comments](#comments)
  - [Variables](#variables)
  - [Types](#types)
  - [Strings & Interpolation](#strings--interpolation)
  - [Operators](#operators)
  - [Conditionals](#conditionals)
  - [While Loop](#while-loop)
  - [For Loop](#for-loop)
  - [Functions](#functions)
  - [Lists](#lists)
  - [Imports](#imports)
- [Page System](#page-system)
  - [Elements](#elements)
  - [Style Shorthands](#style-shorthands)
  - [Event Handlers](#event-handlers)
  - [Conditional & Loop Rendering](#conditional--loop-rendering)
- [Full App Example](#full-app-example)
- [VS Code Extension](#vs-code-extension)
- [Turing Completeness](#turing-completeness)
- [Building from Source](#building-from-source)
- [Architecture](#architecture)
- [Version History](#version-history)

---

## Install

### One-liner (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/hansdevs/rustscript/main/install.sh | sh
```

No Rust toolchain required. The installer auto-detects your platform, downloads the correct pre-built binary, verifies it, and adds it to your PATH.

**Supported platforms:**

| Platform | Architecture |
|----------|-------------|
| macOS | Apple Silicon (aarch64), Intel (x86_64) |
| Linux | x86_64, aarch64 |
| Windows | x86_64 |

### From source

```sh
git clone https://github.com/hansdevs/rustscript
cd rustscript && make install
```

Requires the [Rust toolchain](https://rustup.rs).

---

## Quick Start

Create `hello.rsx`:

```python
let name = "World"

page {
    style { bg: "#0a0e17" fg: "#e5e7eb" font: "'Inter', sans-serif" }

    div {
        style { pad: "40px" center maxw: "600px" mx: "auto" }

        h1 "Hello, {name}!" {
            style { size: "2rem" weight: "700" }
        }
    }
}
```

Preview it:

```sh
rustscript preview hello.rsx
```

---

## CLI Commands

| Command | Usage | Description |
|---------|-------|-------------|
| `preview` | `rustscript preview file.rsx` | Build and open in browser |
| `build` | `rustscript build file.rsx [-o out.html]` | Compile to HTML |
| `run` | `rustscript run file.rsx` | Interpret in terminal |
| `serve` | `rustscript serve file.rsx [-p port]` | Dev server with live reload |
| `help` | `rustscript help` | Show help |

Passing a `.rsx` file with no command defaults to `preview`.

---

## Language Reference

### Comments

```python
# This is a comment
```

### Variables

```python
let x = 42
let name = "RustScript"
let items = [1, 2, 3]
```

### Types

| Type | Example |
|------|---------|
| `int` | `42`, `-7` |
| `float` | `3.14`, `0.5` |
| `str` | `"hello"` |
| `bool` | `true`, `false` |
| `list` | `[1, 2, 3]` |

### Strings & Interpolation

```python
let name = "World"
let greeting = "Hello, {name}!"      # interpolation
let escaped = "Use \{braces\}"        # literal braces
let multi = "line1\nline2"            # escape sequences: \n \t \\ \"
let repeated = "abc" * 3              # "abcabcabc"
```

### Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+` `-` `*` `/` `%` |
| Comparison | `==` `!=` `<` `>` `<=` `>=` |
| Logical | `and` `or` `not` |
| Assignment | `=` `+=` `-=` |

### Conditionals

```python
if x > 10 {
    print("big")
} else if x > 5 {
    print("medium")
} else {
    print("small")
}
```

### While Loop

```python
let i = 0
while i < 10 {
    print(i)
    i += 1
}
```

### For Loop

```python
for item in ["apple", "banana", "cherry"] {
    print(item)
}

for i in range(5) {
    print(i)
}
```

### Functions

```python
fn add(a, b) {
    return a + b
}

fn factorial(n) {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}
```

### Lists

```python
let nums = [1, 2, 3, 4, 5]
print(nums[0])            # 1
print(len(nums))          # 5
let more = push(nums, 6)  # [1, 2, 3, 4, 5, 6]
let last = pop(more)      # 6
```

### Imports

```python
import "components/header.rsx"   # inline another .rsx file
import "images/logo.png"         # auto base64 encode, available as `logo`
```

Supported image types: `png`, `jpg`, `jpeg`, `gif`, `svg`, `webp`, `ico`, `bmp`.

---

## Page System

The `page` block defines HTML output. Everything inside compiles to a single self-contained HTML file.

### Elements

```python
page {
    style { bg: "#0a0e17" fg: "white" }

    div "text content" {
        style { pad: "20px" radius: "8px" }
        p "child element"
    }

    a "GitHub" {
        href: "https://github.com/hansdevs/rustscript"
        style { fg: "#58a6ff" }
    }

    img {
        src: "{logo}"
        style { w: "200px" }
    }
}
```

Supported tags: `div`, `p`, `span`, `h1`–`h6`, `a`, `button`, `input`, `img`, `ul`, `ol`, `li`, `form`, `label`, `select`, `option`, `textarea`, `header`, `footer`, `nav`, `section`, `main`, `article`, `aside`, `table`, `tr`, `td`, `th`, `pre`, `code`, `br`, `hr`, `video`, `audio`, `canvas`, `text`.

### Style Shorthands

RustScript provides shorthand names for common CSS properties:

**Typography**

| Shorthand | CSS Property |
|-----------|-------------|
| `size` | `font-size` |
| `font` | `font-family` |
| `weight` | `font-weight` |
| `lh` | `line-height` |
| `align` | `text-align` |
| `spacing` | `letter-spacing` |

Flags (no value needed): `bold`, `italic`, `underline`, `strike`, `uppercase`, `lowercase`, `capitalize`.

**Colors**

| Shorthand | CSS Property |
|-----------|-------------|
| `bg` | `background` |
| `fg` | `color` |

**Spacing**

| Shorthand | CSS |
|-----------|-----|
| `pad` | `padding` |
| `pt` `pb` `pl` `pr` | `padding-top` / `-bottom` / `-left` / `-right` |
| `px` `py` | horizontal / vertical padding |
| `m` | `margin` |
| `mt` `mb` `ml` `mr` | `margin-top` / `-bottom` / `-left` / `-right` |
| `mx` `my` | horizontal / vertical margin |

**Sizing**

| Shorthand | CSS Property |
|-----------|-------------|
| `w` `h` | `width` / `height` |
| `minw` `maxw` | `min-width` / `max-width` |
| `minh` `maxh` | `min-height` / `max-height` |

**Border & Shape**

| Shorthand | CSS Property |
|-----------|-------------|
| `radius` | `border-radius` |
| `shadow` | `box-shadow` |

**Layout Flags** (no value needed)

| Flag | CSS Output |
|------|-----------|
| `row` | `display: flex; flex-direction: row` |
| `col` | `display: flex; flex-direction: column` |
| `center` | `display: flex; justify-content: center; align-items: center` |
| `hidden` | `display: none` |
| `pointer` | `cursor: pointer` |
| `scroll` | `overflow: auto` |
| `clip` | `overflow: hidden` |
| `nowrap` | `white-space: nowrap` |
| `fixed` `absolute` `relative` `sticky` | `position: ...` |
| `inline` `block` `grid` | `display: ...` |

**Flex / Grid**

| Shorthand | CSS Property |
|-----------|-------------|
| `items` | `align-items` |
| `justify` | `justify-content` |
| `gap` | `gap` |
| `wrap` | `flex-wrap` |
| `grow` `shrink` `basis` | `flex-grow` / `-shrink` / `-basis` |
| `cols` | `grid-template-columns` |

Any unrecognized property name passes through as standard CSS (e.g. `border: "1px solid red"`, `transition: "all 0.2s"`).

### Event Handlers

```python
let count = 0

page {
    button "Click me" {
        on click {
            count = count + 1
        }
    }
}
```

Supported events: `click`, `input`, `change`, `submit`, `keydown`, `keyup`.

Access the event target value with `event.value`.

### Conditional & Loop Rendering

```python
page {
    if show_header {
        h1 "Welcome"
    }

    for item in items {
        p "{item}"
    }
}
```

---

## Full App Example

```python
let count = 0

fn get_label(n) {
    if n == 0 { return "Click the button below" }
    if n == 1 { return "Clicked 1 time" }
    return "Clicked {n} times"
}

page {
    style {
        bg: "#0a0e17"
        fg: "#e5e7eb"
        font: "'Inter', sans-serif"
    }

    div {
        style { pad: "40px" center maxw: "600px" mx: "auto" col gap: "16px" }

        h1 "Counter: {count}" {
            style { size: "2rem" weight: "700" }
        }

        p "{get_label(count)}" {
            style { fg: "#9ca3af" }
        }

        button "Click me" {
            style {
                bg: "#f97316"
                fg: "white"
                pad: "12px 24px"
                radius: "8px"
                weight: "600"
                pointer
                border: "none"
            }
            on click {
                count = count + 1
            }
        }
    }
}
```

```sh
rustscript preview counter.rsx
```

---

## VS Code Extension

The RustScript extension for VS Code adds first-class `.rsx` support:

- **Syntax highlighting** for all RustScript constructs
- **File icon** in the explorer and tabs
- **Bracket matching** and auto-closing
- **Comment toggling** (`Cmd+/` / `Ctrl+/`)

### Install from Marketplace

Search for **"RustScript"** in the VS Code Extensions tab, or run:

```
ext install Hansg123.rustscript
```

### Install from source

The extension source is in the `rustscript-vscode/` directory. To package and install locally:

```sh
cd rustscript-vscode
npx @vscode/vsce package
code --install-extension rustscript-*.vsix
```

---

## Turing Completeness

RustScript supports variables, conditionals, loops (while + for), recursion, lists, and string manipulation — making it Turing-complete. See `examples/turing_test.rsx` for a proof including Fibonacci, factorial, and FizzBuzz.

---

## Building from Source

**Prerequisites:** [Rust toolchain](https://rustup.rs)

```sh
git clone https://github.com/hansdevs/rustscript
cd rustscript
make build      # build release binary
make install    # install to ~/.cargo/bin
make clean      # clean build artifacts
```

---

## Architecture

| Module | Role |
|--------|------|
| `main.rs` | CLI, import resolution, image inlining |
| `lexer.rs` | Source to tokens |
| `token.rs` | Token types, keywords, HTML tag list |
| `parser.rs` | Recursive-descent parser (tokens to AST) |
| `ast.rs` | AST type definitions |
| `codegen.rs` | AST to self-contained HTML with embedded CSS/JS |
| `interpreter.rs` | Tree-walking interpreter for `run` mode |
| `server.rs` | Dev server with live reload for `serve` mode |

Zero external dependencies. ~3,500 lines of Rust.

---

## Built-in Functions

| Function | Description |
|----------|-------------|
| `print(args...)` | Print to stdout |
| `len(x)` | Length of list or string |
| `str(x)` | Convert to string |
| `int(x)` | Convert to integer |
| `float(x)` | Convert to float |
| `push(list, val)` | Append to list |
| `pop(list)` | Remove and return last item |
| `range(n)` / `range(a, b)` | Generate integer list |
| `type(x)` | Type name as string |
| `abs(n)` | Absolute value |
| `min(a, b)` | Minimum |
| `max(a, b)` | Maximum |

**String methods:** `.upper()`, `.lower()`, `.trim()`, `.contains(s)`, `.split(delim)`, `.length`

**List methods:** `.join(delim)`, `.length`

---

## Version History

### v0.1.1 — March 1, 2026

- **Local preview output** — `rustscript preview` now writes to `.rustscript/` alongside source files instead of the OS temp directory. Auto-generates `.gitignore` to keep build artifacts out of git.
- **Compiler cleanup** — resolved all 14 clippy warnings, ran `cargo fmt`, converted module doc comments to inner docs.
- **Deduplicated internals** — consolidated duplicate import resolver and browser-open logic into shared implementations.
- **Rust 2024 idioms** — collapsed nested if-let chains using let-chains, replaced manual `div_ceil`, removed dead code.
- **Version history on website** — added a version history section to the landing page with styled release cards.

### v0.1.0 — February 28, 2026 · Original HackUSU Edition 🏆

- Initial release.
- Full compiler pipeline: lexer → parser → AST → codegen (HTML/CSS/JS).
- Tree-walking interpreter for `run` mode.
- Dev server with live reload (`serve` mode).
- VS Code extension with syntax highlighting.
- One-line installer for macOS (ARM + Intel) and Linux.
- Image imports with auto base64 inlining.
- Style shorthand system (40+ CSS shorthands).
- Turing-complete language with functions, loops, lists, and string interpolation.

---

Parts of this project were built with the help of Claude by Anthropic.
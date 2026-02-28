# RustScript

**Where HTML, CSS & Python had a child.**

RustScript is a Turing-complete programming language. Write a single `.rsx` file with Python-like logic, HTML-like structure, and CSS-like styling — then preview it instantly in your browser.

**No Rust toolchain required.** Download a single binary and go.

---

## Install

### One-liner (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/user/rustscript/main/install.sh | sh
```

This auto-detects your OS and architecture, downloads the correct binary, and puts it in `/usr/local/bin`.

### Manual download

Grab the binary for your platform from [Releases](https://github.com/user/rustscript/releases):

| Platform | Binary |
|----------|--------|
| macOS Apple Silicon | `rustscript-darwin-aarch64` |
| macOS Intel | `rustscript-darwin-x86_64` |
| Linux x86_64 | `rustscript-linux-x86_64` |
| Linux ARM64 | `rustscript-linux-aarch64` |
| Windows x86_64 | `rustscript-windows-x86_64.exe` |

Then make it executable and move it onto your PATH:

```bash
chmod +x rustscript-darwin-aarch64
sudo mv rustscript-darwin-aarch64 /usr/local/bin/rustscript
```

### Verify

```bash
rustscript --version
```

---

## Quick Start

```bash
# Preview any .rsx file instantly in your browser
rustscript preview myapp.rsx

# Or build to a specific HTML file
rustscript build myapp.rsx -o myapp.html

# Run logic-only in the terminal
rustscript run myapp.rsx
```

---

## Commands

| Command | What it does |
|---------|--------------|
| `rustscript preview <file.rsx>` | Build + open in browser automatically |
| `rustscript build <file.rsx>` | Compile to `.html` (same name by default) |
| `rustscript build <file.rsx> -o out.html` | Compile to a specific output path |
| `rustscript run <file.rsx>` | Interpret logic in the terminal |
| `rustscript help` | Show help message |

---

## Language Reference

### Comments

```python
# This is a comment
# Comments start with # and go to end of line
```

### Variables

Declare with `let`. No type annotations needed — types are inferred.

```python
let name = "RustScript"      # string
let age = 25                  # integer
let pi = 3.14                 # float
let active = true             # boolean
let items = [1, 2, 3]         # list
```

Reassign without `let`:

```python
let x = 10
x = 20
x += 5       # compound assignment (also -=)
```

### Types

| Type | Example | Description |
|------|---------|-------------|
| `int` | `42`, `-7`, `0` | 64-bit integer |
| `float` | `3.14`, `0.5` | 64-bit float |
| `str` | `"hello"` | UTF-8 string |
| `bool` | `true`, `false` | Boolean |
| `list` | `[1, 2, 3]` | Dynamic array |
| `null` | *(no literal)* | Absence of value |

### Strings & Interpolation

Strings use double quotes. Use `{expression}` inside strings to embed values:

```python
let name = "world"
let greeting = "Hello, {name}!"           # → "Hello, world!"
let math = "2 + 2 = {2 + 2}"              # → "2 + 2 = 4"
let info = "{name} has {len(name)} chars"  # → "world has 5 chars"
```

Escape sequences:

| Escape | Character |
|--------|-----------|
| `\"` | Double quote |
| `\\` | Backslash |
| `\n` | Newline |
| `\t` | Tab |
| `\{` | Literal `{` (no interpolation) |
| `\}` | Literal `}` |

### Operators

**Arithmetic:**

| Operator | Meaning | Example |
|----------|---------|---------|
| `+` | Add / concat | `3 + 4` → `7`, `"a" + "b"` → `"ab"` |
| `-` | Subtract | `10 - 3` → `7` |
| `*` | Multiply | `4 * 5` → `20`, `"ha" * 3` → `"hahaha"` |
| `/` | Divide | `10 / 3` → `3` |
| `%` | Modulo | `10 % 3` → `1` |

**Comparison:**

| Operator | Meaning |
|----------|---------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less or equal |
| `>=` | Greater or equal |

**Logical:**

| Operator | Meaning |
|----------|---------|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT |

**Assignment:**

| Operator | Meaning |
|----------|---------|
| `=` | Assign |
| `+=` | Add and assign |
| `-=` | Subtract and assign |

### Conditionals

```python
if x > 10 {
    print("big")
} else if x > 0 {
    print("small")
} else {
    print("zero or negative")
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
let fruits = ["apple", "banana", "cherry"]
for fruit in fruits {
    print(fruit)
}

for i in range(5) {
    print(i)    # 0, 1, 2, 3, 4
}
```

### Functions

```python
fn greet(name) {
    return "Hello, {name}!"
}

print(greet("world"))
```

Functions support recursion:

```python
fn factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

print(factorial(5))   # 120
```

### Lists

```python
let nums = [1, 2, 3, 4, 5]

# Access by index (0-based)
print(nums[0])        # 1
nums[2] = 99          # assign to index

# Built-in list functions
print(len(nums))      # 5
push(nums, 6)         # add to end
pop(nums)             # remove last

# Iterate
for n in nums {
    print(n)
}
```

### Built-in Functions

| Function | Description | Example |
|----------|-------------|---------|
| `print(args...)` | Print values to console | `print("hi", x)` |
| `len(x)` | Length of list or string | `len([1,2,3])` → `3` |
| `str(x)` | Convert to string | `str(42)` → `"42"` |
| `int(x)` | Convert to integer | `int("42")` → `42` |
| `float(x)` | Convert to float | `float("3.14")` → `3.14` |
| `type(x)` | Get type name as string | `type(42)` → `"int"` |
| `range(n)` | List `[0, 1, ..., n-1]` | `range(3)` → `[0, 1, 2]` |
| `range(a, b)` | List `[a, a+1, ..., b-1]` | `range(2, 5)` → `[2, 3, 4]` |
| `push(list, val)` | Append value to list | `push(items, "new")` |
| `pop(list)` | Remove & return last item | `pop(items)` |
| `abs(n)` | Absolute value | `abs(-5)` → `5` |
| `min(a, b)` | Minimum of two values | `min(3, 7)` → `3` |
| `max(a, b)` | Maximum of two values | `max(3, 7)` → `7` |

### String Methods

```python
"hello".upper()          # "HELLO"
"HELLO".lower()          # "hello"
"  hi  ".trim()          # "hi"
"hello".contains("ell")  # true
"a,b,c".split(",")       # ["a", "b", "c"]
["a", "b"].join("-")     # "a-b"
"hello".length           # 5
```

### Member Access & Indexing

```python
let items = [10, 20, 30]
items[0]                 # 10
items.length             # 3

let name = "hello"
name[0]                  # "h"
name.length              # 5
```

---

## Page — Building UI

The `page { }` block defines a reactive web interface. When you `preview` or `build` a file containing a page block, it compiles to a self-contained HTML file.

### Basic Structure

```python
page {
    h1 "Hello World"
}
```

This generates a complete HTML page with a `<h1>` tag.

### Available HTML Tags

| Category | Tags |
|----------|------|
| **Layout** | `div`, `span`, `header`, `footer`, `nav`, `section`, `main`, `article`, `aside` |
| **Text** | `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `p`, `a`, `text` |
| **Form** | `button`, `input`, `textarea`, `select`, `option`, `label`, `form` |
| **List** | `ul`, `ol`, `li` |
| **Table** | `table`, `tr`, `td`, `th` |
| **Media** | `img`, `video`, `audio`, `canvas` |
| **Other** | `br`, `hr` |

### Text Content

Put a string after the tag name:

```python
page {
    h1 "Welcome"
    p "This is a paragraph."
    p "Count is: {count}"       # interpolation works here too
}
```

### Inline Styles

Use `style { }` inside any element:

```python
page {
    h1 "Styled Heading" {
        style {
            fg: "#58a6ff"
            size: "2rem"
            mb: "16px"
        }
    }
}
```

RustScript has its own shorthand styling system. You can use the shorthands **or** standard CSS property names — both work. Values are quoted strings. Flag properties (like `bold`, `pointer`, `center`) need no value.

### Style Shorthand Reference

#### Typography

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `size` | `font-size` | `size: "1.5rem"` |
| `font` | `font-family` | `font: "'Inter', sans-serif"` |
| `weight` | `font-weight` | `weight: "600"` |
| `bold` | `font-weight: bold` | `bold` *(flag)* |
| `italic` | `font-style: italic` | `italic` *(flag)* |
| `underline` | `text-decoration: underline` | `underline` *(flag)* |
| `strike` | `text-decoration: line-through` | `strike` *(flag)* |
| `uppercase` | `text-transform: uppercase` | `uppercase` *(flag)* |
| `lowercase` | `text-transform: lowercase` | `lowercase` *(flag)* |
| `capitalize` | `text-transform: capitalize` | `capitalize` *(flag)* |
| `spacing` | `letter-spacing` | `spacing: "2px"` |
| `lh` | `line-height` | `lh: "1.6"` |
| `align` | `text-align` | `align: "center"` |
| `indent` | `text-indent` | `indent: "2em"` |

#### Colors & Background

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `bg` | `background` | `bg: "#0d1117"` |
| `fg` | `color` | `fg: "#e6edf3"` |

#### Spacing

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `pad` | `padding` | `pad: "16px"` |
| `pt` | `padding-top` | `pt: "8px"` |
| `pb` | `padding-bottom` | `pb: "8px"` |
| `pl` | `padding-left` | `pl: "8px"` |
| `pr` | `padding-right` | `pr: "8px"` |
| `px` | `padding-left` + `padding-right` | `px: "16px"` |
| `py` | `padding-top` + `padding-bottom` | `py: "8px"` |
| `m` | `margin` | `m: "0 auto"` |
| `mt` | `margin-top` | `mt: "16px"` |
| `mb` | `margin-bottom` | `mb: "8px"` |
| `ml` | `margin-left` | `ml: "8px"` |
| `mr` | `margin-right` | `mr: "8px"` |
| `mx` | `margin-left` + `margin-right` | `mx: "auto"` |
| `my` | `margin-top` + `margin-bottom` | `my: "16px"` |

#### Sizing

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `w` | `width` | `w: "100%"` |
| `h` | `height` | `h: "200px"` |
| `minw` | `min-width` | `minw: "300px"` |
| `maxw` | `max-width` | `maxw: "600px"` |
| `minh` | `min-height` | `minh: "100vh"` |
| `maxh` | `max-height` | `maxh: "500px"` |

#### Border & Shape

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `radius` | `border-radius` | `radius: "8px"` |
| `shadow` | `box-shadow` | `shadow: "0 4px 12px rgba(0,0,0,0.3)"` |
| `outline` | `outline` | `outline: "2px solid blue"` |

#### Layout (Flag Properties)

These expand to one or more CSS rules with no value needed:

| Flag | CSS Output |
|------|-----------|
| `row` | `display: flex; flex-direction: row` |
| `col` | `display: flex; flex-direction: column` |
| `center` | `display: flex; justify-content: center; align-items: center` |
| `hidden` | `display: none` |
| `inline` | `display: inline` |
| `block` | `display: block` |
| `grid` | `display: grid` |
| `pointer` | `cursor: pointer` |
| `nowrap` | `white-space: nowrap` |
| `clip` | `overflow: hidden` |
| `scroll` | `overflow: auto` |
| `fixed` | `position: fixed` |
| `absolute` | `position: absolute` |
| `relative` | `position: relative` |
| `sticky` | `position: sticky` |

#### Flex & Grid

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `items` | `align-items` | `items: "center"` |
| `justify` | `justify-content` | `justify: "space-between"` |
| `self-align` | `align-self` | `self-align: "flex-end"` |
| `grow` | `flex-grow` | `grow: "1"` |
| `shrink` | `flex-shrink` | `shrink: "0"` |
| `basis` | `flex-basis` | `basis: "200px"` |
| `wrap` | `flex-wrap` | `wrap: "wrap"` |
| `gap` | `gap` | `gap: "16px"` |
| `cols` | `grid-template-columns` | `cols: "1fr 1fr 1fr"` |

#### Position & Layers

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `pos` | `position` | `pos: "relative"` |
| `z` | `z-index` | `z: "10"` |

#### Effects

| Shorthand | CSS Property | Example |
|-----------|-------------|---------|
| `opacity` | `opacity` | `opacity: "0.5"` |
| `transition` | `transition` | `transition: "all 0.2s"` |
| `transform` | `transform` | `transform: "scale(1.1)"` |
| `filter` | `filter` | `filter: "blur(4px)"` |
| `backdrop` | `backdrop-filter` | `backdrop: "blur(10px)"` |

> **Standard CSS passthrough:** Any property name not listed above is passed through as-is. So `border: "1px solid red"`, `font-family: "monospace"`, `background-clip: "text"` etc. all work.

### Page-Level Styles

Put `style { }` directly inside `page { }` to style the body:

```python
page {
    style {
        bg: "#0d1117"
        fg: "#e6edf3"
        font: "'Inter', sans-serif"
        pad: "40px"
    }

    h1 "Hello"
}
```

### Attributes

Set HTML attributes with `name: value` inside the element body:

```python
page {
    a "Visit Google" {
        href: "https://google.com"
        style {
            color: "#58a6ff"
        }
    }

    input {
        type: "text"
        placeholder: "Enter your name"
    }
}
```

### Nesting Elements

Put child elements inside the parent's `{ }`:

```python
page {
    div {
        style { pad: "20px" }
        h1 "Title"
        p "Paragraph inside the div"
        div {
            p "Nested deeper"
        }
    }
}
```

### Event Handlers

Use `on <event> { }` to handle DOM events. The page re-renders automatically after each event.

```python
let count = 0

page {
    p "Count: {count}"
    button "Click me" {
        on click {
            count = count + 1
        }
    }
}
```

Supported events: `click`, `input`, `change`, `submit`, `keydown`, `keyup`.

Inside an event handler, `event.value` gives the current value of the element (useful for inputs):

```python
let name = ""

page {
    input {
        type: "text"
        placeholder: "Your name"
        on input {
            name = event.value
        }
    }
    p "Hello, {name}!"
}
```

### Conditional Rendering

Use `if` / `else` inside a page to show/hide elements:

```python
let logged_in = true

page {
    if logged_in {
        p "Welcome back!"
    } else {
        p "Please log in."
    }
}
```

### List Rendering

Use `for` inside a page to render lists:

```python
let items = ["Apples", "Bananas", "Cherries"]

page {
    ul {
        for item in items {
            li "{item}"
        }
    }
}
```

---

## Full App Example

Here's a complete interactive app in one `.rsx` file:

```python
# app.rsx — A complete RustScript app

let count = 0
let todos = ["Learn RustScript", "Build something", "Ship it"]

fn label(n) {
    if n == 1 {
        return "time"
    }
    return "times"
}

page {
    style {
        bg: "#0d1117"
        fg: "#e6edf3"
        font: "'Inter', sans-serif"
        pad: "40px"
    }

    div {
        style { maxw: "600px"  m: "0 auto" }

        h1 "My App" {
            style { fg: "#58a6ff"  size: "2.5rem" }
        }

        p "Clicked {count} {label(count)}"

        button "+ Add" {
            style {
                bg: "#238636"
                fg: "white"
                border: "none"
                pad: "10px 20px"
                radius: "8px"
                pointer
            }
            on click {
                count = count + 1
            }
        }

        for todo in todos {
            p "• {todo}" {
                style { py: "8px" }
            }
        }

        if count > 5 {
            p "🎉 Nice clicking!" {
                style { fg: "#3fb950" }
            }
        }
    }
}
```

Preview it: `rustscript preview app.rsx`

---

## How It Works

```
.rsx source → Lexer → Tokens → Parser → AST ─→ Codegen → HTML/CSS/JS
                                                ↓
                                          Interpreter (run mode)
```

| Component | File | What it does |
|-----------|------|-------------|
| Tokens | `src/token.rs` | Token types, keyword list, HTML tag registry |
| Lexer | `src/lexer.rs` | Converts source text to token stream |
| AST | `src/ast.rs` | Abstract syntax tree node types |
| Parser | `src/parser.rs` | Recursive descent parser |
| Codegen | `src/codegen.rs` | Compiles AST to self-contained HTML + JS + CSS |
| Interpreter | `src/interpreter.rs` | Tree-walking interpreter for terminal mode |
| CLI | `src/main.rs` | Command-line interface |

## Turing Completeness

RustScript is Turing complete. It supports:
- Arbitrary integers and dynamic lists (unbounded storage)
- Conditional branching (`if` / `else if` / `else`)
- Iteration (`while`, `for`)
- Recursion (functions calling themselves)

This is sufficient to simulate any Turing machine.

## File Extension

RustScript files use the `.rsx` extension. A VS Code extension is included in `rustscript-vscode/` for syntax highlighting.

---

## Build from Source (contributors only)

Most users should use the [install script](#install) above. If you want to build from source:

**Prerequisites:** Rust ≥ 1.85 (`edition = "2024"`)

```bash
cd rustscript
cargo build --release
./target/release/rustscript --version

# Install to ~/.cargo/bin
make install
```

### Cutting a Release

Tag a version and push — GitHub Actions will build binaries for all platforms:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Binaries appear on the [Releases](https://github.com/user/rustscript/releases) page automatically.

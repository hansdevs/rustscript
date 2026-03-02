# RustScript

Language support for [RustScript](https://github.com/hansdevs/rustscript) — a compiled language that turns a single `.rsx` file into a self-contained HTML page.

## Features

- **Syntax highlighting** for `.rsx` files
- **File icon** in the explorer and tabs
- **Bracket matching** and auto-closing
- **Comment toggling** (`Cmd+/` / `Ctrl+/`)

## What is RustScript?

RustScript is a compiled language with Python-like syntax that produces self-contained HTML pages — complete with embedded CSS and JavaScript. No framework, no bundler, no dependencies.

```
page {
    style {
        bg: "#0a0e17"
        fg: "#e5e7eb"
        font: "'Inter', sans-serif"
    }

    h1 "Hello, world!" {
        style { size: "2rem" weight: "700" }
    }

    p "Built with RustScript."
}
```

One file in, one file out.

## Install the Compiler

```sh
curl -fsSL https://raw.githubusercontent.com/hansdevs/rustscript/main/install.sh | sh
```

Or visit the [GitHub repo](https://github.com/hansdevs/rustscript) for more options.

## Commands

| Command | Description |
|---------|-------------|
| `rustscript preview file.rsx` | Build and open in browser |
| `rustscript serve file.rsx` | Dev server with live reload |
| `rustscript build file.rsx` | Compile to HTML |
| `rustscript run file.rsx` | Run logic in terminal |

## Links

- [GitHub](https://github.com/hansdevs/rustscript)
- [Documentation](https://github.com/hansdevs/rustscript#readme)
- [Changelog](https://github.com/hansdevs/rustscript/releases)

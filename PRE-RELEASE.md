# RustScript — Pre-Release Ideaboard

> For language docs and install instructions, see [readme.md](readme.md).

---

## The Idea

Make RustScript the compiled alternative to Python for ML. Same readable syntax, but it compiles to native code — no interpreter, no GIL, no speed ceiling.

---

## Why

| Python | RustScript |
|--------|-----------|
| Interpreted | Compiled |
| GIL bottleneck | Native parallelism |
| Needs C++ for speed | Fast by default |
| Ships with runtime | Ships as one binary |

---

## Roadmap

- **Tensors & math** — native tensor type, matrix ops, SIMD
- **Autograd** — automatic differentiation built into the language
- **Training** — optimizers, loss functions, dataloaders
- **GPU** — Metal (Apple Silicon), CUDA (NVIDIA)
- **Ecosystem** — ONNX import, Python interop, package manager

---

## Open Questions

- Tensor memory layout?
- Static vs dynamic computation graphs?
- ONNX as bridge or native format?

---

*Living ideaboard. Nothing here is promised.*


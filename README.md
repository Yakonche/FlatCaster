# FlatCaster

> [Lire en français](README_fr.md)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition, stable recommended)
- A GPU driver compatible with **Vulkan**, **Metal**, **DX12**, or **DX11**
- Windows 10/11 (build targeting Windows with GUI subsystem)

```
rustup update stable
```

---

## Build

### Debug

```
cargo build
```

The executable is generated at `target/debug/FlatCaster.exe`.

### Release (optimized)

```
cargo build --release
```

The executable is generated at `target/release/FlatCaster.exe`.

---

## Run

```
cargo run --release
```

---

## Checks

### Compile without running

```
cargo check
```

### Unit tests

```
cargo test
```

### Linter (Clippy)

```
cargo clippy -- -D warnings
```

### Code formatting

Check:
```
cargo fmt --check
```

Apply:
```
cargo fmt
```

---

## Notable dependencies

| Crate | Role |
|---|---|
| `wgpu 22` | GPU rendering (Vulkan / Metal / DX12 / DX11) |
| `winit 0.30` | Window and events |
| `glyphon 0.6` | GPU text rendering |
| `gilrs 0.10` | Gamepad support |
| `winres` | Windows icon integration (build.rs) |

---

## License

Source code: **MIT** — see [`LICENSE`](LICENSE)
Jersey 10 font: **OFL-1.1** — see [`assets/OFL.txt`](assets/OFL.txt)

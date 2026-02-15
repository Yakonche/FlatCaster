# FlatCaster

## Prérequis

- [Rust](https://www.rust-lang.org/tools/install) (édition 2021, stable recommandé)
- Un pilote GPU compatible **Vulkan**, **Metal**, **DX12** ou **DX11**
- Windows 10/11 (build ciblant Windows avec sous-système GUI)

```
rustup update stable
```

---

## Compilation

### Debug

```
cargo build
```

L'exécutable est généré dans `target/debug/FlatCaster.exe`.

### Release (optimisée)

```
cargo build --release
```

L'exécutable est généré dans `target/release/FlatCaster.exe`.

---

## Lancer

```
cargo run --release
```

---

## Vérifications

### Compiler sans exécuter

```
cargo check
```

### Tests unitaires

```
cargo test
```

### Linter (Clippy)

```
cargo clippy -- -D warnings
```

### Formatage du code

Vérification :
```
cargo fmt --check
```

Application :
```
cargo fmt
```

---

## Dépendances notables

| Crate | Rôle |
|---|---|
| `wgpu 22` | Rendu GPU (Vulkan / Metal / DX12 / DX11) |
| `winit 0.30` | Fenêtre et événements |
| `glyphon 0.6` | Rendu de texte GPU |
| `gilrs 0.10` | Manette de jeu |
| `winres` | Intégration de l'icône Windows (build.rs) |

---

## Licence

Code source : **MIT** — voir [`LICENSE`](LICENSE)
Police Jersey 10 : **OFL-1.1** — voir [`assets/OFL.txt`](assets/OFL.txt)

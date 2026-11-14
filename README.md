# Türkçe Dilbilgisi — Turkish Grammar Map

An interactive radial mind-map of Turkish grammar, written in Rust, compiled to WebAssembly and drawn on an HTML5 canvas.

- 10 primary rings and 293 rules, each with a suffix formula, harmony notes, exceptions, a CEFR level and three example sentences
- Click to expand or collapse branches, scroll to zoom at the cursor, drag to pan
- Live search that expands the path to matching rules
- Rule Inspector drawer with highlighted suffixes
- **Study by Level (beta):** the same rules arranged as a house of six CEFR floors, A1 on the ground floor and C2 at the top
- Frequency dictionary of 200 verbs and 150 adjectives with its own search (ranking is approximate)

## Run

The compiled WebAssembly is committed in `pkg/`, so any static file server works:

```bash
python -m http.server 5173
```

Then open http://localhost:5173.

| Address | View |
|---|---|
| `/main/` | Radial map grouped by topic |
| `/by-level/` | House of six floors grouped by CEFR level (beta) |
| `/` | Redirects to `/main/` |

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cargo test
wasm-pack build --target web --release
```

On Windows, if the project sits in a very long path, set a short build directory first:

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\tgm-target"
```

## Layout

| File | Role |
|---|---|
| `src/grammar_data.rs`, `src/grammar_data/extra.rs` | The grammar tree. Add a rule with one `l(...)` call. |
| `src/dictionary.rs` | Verb and adjective lists. Add a word with one tuple. |
| `src/layout.rs` | Radial tree layout and the house-of-floors layout. |
| `src/lib.rs` | State, camera, input, animation, search, rendering. |
| `main/index.html`, `by-level/index.html` | The two pages; `<body data-view>` picks the view. |
| `index.html` | Redirect to `main/`. |
| `styles.css`, `main.js` | Shared HUD, drawers and the WASM bootstrapper. |

## Keys

`/` search · `D` dictionary · `E` expand all · `C` collapse all · `0` reset camera · arrows pan · `+` / `-` zoom · `Esc` close

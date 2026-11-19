# Türkçe Dilbilgisi — Turkish Grammar Map

An interactive radial mind-map of Turkish grammar, written in Rust, compiled to WebAssembly and drawn on an HTML5 canvas.

- 10 primary rings and 350 rules, each with a suffix formula, harmony notes, exceptions, a CEFR level and three example sentences
- Click to expand or collapse branches, scroll to zoom at the cursor, drag to pan
- Live search that expands the path to matching rules
- Rule Inspector drawer with highlighted suffixes
- **Study by Level (beta):** the same rules arranged as a house of six CEFR floors, A1 on the ground floor and C2 at the top
- **Player (beta):** study any video with Turkish and English subtitles; click a Turkish word to see its meaning, save it and replay it. Subtitles can be loaded from `.srt` files or created automatically (speech recognition + translation on your own device, or with your own OpenAI / Groq / Qwen API key). Web version of the [TRLearn desktop player](https://github.com/ghost545a/trlearn-player).
- Frequency dictionary of 200 verbs and 150 adjectives with its own search (ranking is approximate)

## Run

The compiled WebAssembly is committed in `pkg/`, so any static file server works:

```bash
python tools/serve.py
```

`tools/serve.py` is a plain static server that also answers HTTP Range requests, which browsers need to seek inside videos (`python -m http.server` does not). Any static host with Range support works, GitHub Pages included.

Then open http://localhost:5173.

| Address | View |
|---|---|
| `/main/` | Radial map grouped by topic |
| `/by-level/` | House of six floors grouped by CEFR level (beta) |
| `/player/` | Video player with clickable subtitles and automatic subtitles (beta) |
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
| `src/grammar_data.rs`, `src/grammar_data/extra.rs`, `src/grammar_data/institutes.rs` | The grammar tree. Add a rule with one `l(...)` call. |
| `src/dictionary.rs` | Verb and adjective lists. Add a word with one tuple. |
| `src/layout.rs` | Radial tree layout and the house-of-floors layout. |
| `src/lib.rs` | State, camera, input, animation, search, rendering. |
| `main/index.html`, `by-level/index.html` | The two pages; `<body data-view>` picks the view. |
| `index.html` | Redirect to `main/`. |
| `styles.css`, `main.js` | Shared HUD, drawers and the WASM bootstrapper. |
| `src/player/` | SRT parser and Turkish-aware dictionary lookup for the player (ported from the desktop app). |
| `player/` | The player page: `player.js` (playback, subtitles, saved words), `autosubs.js` + `asr-worker.js` + `audio-utils.js` (automatic subtitles), `tr_en_dictionary.tsv`, `samples/`. |

## Keys

`/` search · `D` dictionary · `E` expand all · `C` collapse all · `0` reset camera · arrows pan · `+` / `-` zoom · `Esc` close

## Player data and models

- `player/tr_en_dictionary.tsv` comes from the [kaikki.org](https://kaikki.org/dictionary/Turkish/) extract of English Wiktionary and is available under CC BY-SA 4.0 and the GFDL.
- Automatic subtitles on the visitor's device use [Transformers.js](https://github.com/huggingface/transformers.js) 3.8.1 from jsDelivr, the multilingual Whisper models from `onnx-community` (MIT) and `Xenova/opus-mt-tr-en` (Apache 2.0), downloaded from Hugging Face on first use; desktop Chrome's built-in translator is used when available.
- YouTube videos can be studied with your own `.srt` files, but cannot be subtitled automatically: other websites cannot read YouTube's audio.

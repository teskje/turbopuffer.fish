# turbopuffer-fish

Interactive ASCII pufferfish simulation — Rust + WebAssembly, no server, no bundler.

## Commands

```bash
mise run dev            # Build + watch + serve at http://localhost:8080
mise run build          # Build WASM + JS bindings (debug)
mise run build-release  # Optimized build (includes wasm-opt -O3)
mise run serve          # Python HTTP server on :8080 serving www/
mise run lint           # clippy (warnings = errors) + rustfmt check
```

**After any Rust changes, run `mise run lint` and fix all issues before finishing.**

## Project Layout

Source:
- `src/lib.rs` — `Game` struct, `Vec2`, `Rect`, game loop (`tick`), collision detection, event dispatch
- `src/entity.rs` — `Entity` enum dispatching to `Puffer`/`Bubble` via `match`
- `src/puffer.rs` — `Puffer`: state machine (Normal/Startled/Puffed), movement, fin animation, ASCII rendering, bubble spawning
- `src/bubble.rs` — `Bubble`: rising glyph, lifecycle
- `src/canvas.rs` — `Canvas` wrapper over `CanvasRenderingContext2d`, font/color helpers, HiDPI resize
- `src/palette.rs` — Color constants (`OCEAN_SURFACE`, `OCEAN_MID`, `OCEAN_DEEP`, `PUFFER`, `BUBBLE`, `SPIKES`)

Web:
- `www/index.html` — Minimal shell with `<canvas id="canvas">`
- `www/index.js` — Font preloading, WASM init, event handlers (mouse + touch), `requestAnimationFrame` loop
- `www/style.css` — JuliaMono font-face declarations (jsDelivr CDN), canvas styling

Generated (gitignored):
- `www/pkg/` — wasm-bindgen output (ES modules)

Config:
- `Cargo.toml` — Edition 2024, web-sys features explicitly listed per API type, wasm-bindgen pinned to 0.2.114
- `mise.toml` — Tool versions (rust+wasm target, wasm-bindgen, binaryen, watchexec) and task definitions

## Architecture

**Game loop**: JS `requestAnimationFrame` → `Game::tick(timestamp)` → update all entities → collect spawned children → prune dead → resolve collisions → render.

**Entity system**: `Entity` enum (Puffer | Bubble) with explicit `match` dispatch — no trait objects. Each entity implements:
- `update(delta_s, world_dim) -> Vec<Entity>` — state changes + movement; returns spawned entities (Puffers spawn Bubbles)
- `is_alive()` — Puffers are always alive; Bubbles die when off-screen or popped
- `bounds()` — AABB for hit testing and collision
- `render(canvas)` — draws ASCII art
- `on_click()` / `on_hover()` — input response
- `collide(other)` — type-pair behavior: Puffer×Puffer bounces, Puffer×Bubble pops bubble

**Collision**: Sweep-and-prune — sort entities by left x edge, skip pairs once x-gap exceeds width, check full AABB overlap.

**Puffer state machine**: Normal →(hover)→ Startled (1s) →(timeout)→ Normal; Normal/Startled →(click)→ Puffed (10s) →(timeout)→ Normal. State affects speed, ASCII body, bubble spawn rate, and rendering dimensions.

**Rendering**: Canvas 2D `fillText` with JuliaMono. Character width measured once via `measureText("M")` ratio stored in `OnceLock`. Bold weight used for puffers.

## Conventions

- **Entity enum, not traits.** Add new entity types as enum variants with `match` arms in `entity.rs`.
- **web-sys features**: Each Web API type needs an explicit feature in `Cargo.toml`. "Method not found" on a web-sys type usually means a missing feature.
- **Char width**: Ratio-based from `OnceLock`, never hardcoded pixel values.
- **Colors**: Use `palette::` constants, don't hardcode hex.
- **Comments**: No obvious "what" comments. "Why" comments welcome when intent isn't clear from code.
- **Format strings**: Prefer `format!("{width}px")` when the variable exists. Use positional args (`format!("{}px", width)`) for method calls/expressions — don't introduce variables just to inline.
- **Keep this file current** when adding entity types, modules, build steps, or architectural patterns.

## Gotchas

- **y is baseline**: `Canvas::fill_text` y coordinate is the text baseline (bottom of glyphs), not top-left. Entity positions are top-left, so rendering adds `FONT_SIZE` to y.
- **Font preload order**: `www/index.js` loads JuliaMono Regular + Bold via `document.fonts.load()` *before* creating `Game`. `Canvas::new()` calls `measureText` for the char width ratio, which requires the font to be loaded. If font loading moves or breaks, char widths will be wrong.
- **wasm-bindgen version pinning**: `Cargo.toml` pins `=0.2.114` and `mise.toml` installs the matching CLI. These must stay in sync or builds will fail with version mismatch errors.
- **Delta time clamping**: `tick()` clamps delta to 0–100ms to prevent physics explosions from tab-backgrounding or debugger pauses.

## Adding a New Entity Type

1. Create `src/<type>.rs` with a struct implementing: `new()`, `update()`, `is_alive()`, `render()`, `bounds()`, `on_click()`, `on_hover()`, `collide()`.
2. Add variant to `Entity` enum in `entity.rs` and add `match` arms to every dispatched method.
3. Define collision behavior for all entity-pair combinations in `Entity::collide`.
4. If it needs new Web APIs, add the web-sys features to `Cargo.toml`.
5. If it needs new colors, add constants to `palette.rs`.
6. Add `mod <type>;` to `lib.rs`.
7. Update this file.

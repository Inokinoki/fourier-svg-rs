# Fix Compilation Errors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all compilation errors and warnings in the workspace so all packages compile cleanly.

**Architecture:** Minimal targeted fixes - add missing dependency, add type annotations, remove unused imports.

**Tech Stack:** Rust, Cargo

---

## File Structure

- `gpui-app/Cargo.toml` - Add `anyhow` dependency
- `gpui-app/src/main.rs` - Fix type annotations in closures
- `tauri-app/src/main.rs` - Remove unused imports

---

### Task 1: Fix gpui-app missing anyhow dependency

**Files:**
- Modify: `gpui-app/Cargo.toml`

- [ ] **Step 1: Add anyhow dependency to Cargo.toml**

Add `anyhow = "1"` to the `[dependencies]` section:

```toml
[dependencies]
fourier-svg = { path = "../fourier-svg" }
gpui = "0.2.2"
serde_json = { workspace = true }
chrono = { workspace = true }
anyhow = "1"
```

- [ ] **Step 2: Verify the change**

Run: `cargo check -p gpui-app 2>&1 | grep "anyhow"`
Expected: No "unresolved module" error for anyhow

---

### Task 2: Fix gpui-app type annotations

**Files:**
- Modify: `gpui-app/src/main.rs:118-143`

- [ ] **Step 1: Add type annotations to cx.spawn closure**

Replace the `load_svg_dialog` method's spawn block (lines 118-143) with proper type annotations:

```rust
fn load_svg_dialog(&mut self, cx: &mut Context<Self>) {
    let options = PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Open SVG File".into()),
    };

    let rx = cx.prompt_for_paths(options);
    cx.spawn(|this: gpui::AsyncWindowContext, mut cx| async move {
        match rx.await {
            Ok(Ok(Some(paths))) => {
                if let Some(path) = paths.first() {
                    let file_path = path.to_string_lossy().to_string();
                    this.update(&mut cx, |this: &mut FourierApp, cx| {
                        this.load_svg_as_layers(&file_path, cx);
                    })?;
                }
            }
            Ok(Ok(None)) => {
                this.update(&mut cx, |this: &mut FourierApp, cx| {
                    this.status = "File selection cancelled".into();
                    cx.notify();
                })?;
            }
            Ok(Err(e)) => {
                this.update(&mut cx, |this: &mut FourierApp, cx| {
                    this.status = format!("Dialog error: {}", e).into();
                    cx.notify();
                })?;
            }
            Err(_) => {}
        }
        anyhow::Ok(())
    }).detach();
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p gpui-app 2>&1`
Expected: No errors, compilation successful

---

### Task 3: Remove unused imports from tauri-app

**Files:**
- Modify: `tauri-app/src/main.rs:17-26`

- [ ] **Step 1: Remove unused imports**

Remove these lines from `tauri-app/src/main.rs`:

```rust
// Remove these lines:
use fourier_svg::visualizer::gif_visualizer::GIFVisualizer;
use fourier_svg::visualizer::html_visualizer::HTMLVisualizer;
use fourier_svg::visualizer::Visualizer;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
```

- [ ] **Step 2: Verify no warnings**

Run: `cargo check -p tauri-app 2>&1 | grep "warning"`
Expected: No unused import warnings

---

### Task 4: Final verification

- [ ] **Step 1: Check entire workspace**

Run: `cargo check --workspace 2>&1`
Expected: No errors, no warnings (except the svg crate future incompatibility which is external)

- [ ] **Step 2: Run tests**

Run: `cargo test -p fourier-svg 2>&1`
Expected: All 4 tests pass

---

## Notes

- The `svg` crate (v0.9.2) future incompatibility warning is an external dependency issue and cannot be fixed without upgrading the crate or switching to an alternative.
- The core library (`fourier-svg`) and CLI (`fourier-cli`) already compile cleanly.

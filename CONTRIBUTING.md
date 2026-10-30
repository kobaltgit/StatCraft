# StatCraft Contributor Guide (CONTRIBUTING.md)
### The ArtCraft / storytold Ecosystem Engineering Standard

Welcome to **StatCraft** — a next-generation, high-performance, open-source statistical analysis package designed as a modern, safe, and uncompromising alternative to IBM SPSS Statistics, built from the ground up in pure Rust.

This document establishes the **authoritative development regulations, architectural invariants, and Definition of Done (DoD)** that govern all contributions, pull requests, and automated agent workflows. Every commit and pull request must strictly pass the automated CI quality gate (`cargo xtask ci`).

---

## Table of Contents
1. [ArtCraft Ecosystem Foundations](#1-artcraft-ecosystem-foundations)
2. [System Resource Guard (Compilation Limits)](#2-system-resource-guard-compilation-limits)
3. [Reliability: The Never Crash Mandate](#3-reliability-the-never-crash-mandate)
4. [Layered Engine Architecture (L0–L6)](#4-layered-engine-architecture-l0l6)
5. [Command Architecture (Commands, Not Handlers)](#5-command-architecture-commands-not-handlers)
6. [Dialog Design Standard (DialogKit) & Vector-First UI](#6-dialog-design-standard-dialogkit--vector-first-ui)
7. [Zero-Hardcode i18n & Fluent Localization](#7-zero-hardcode-i18n--fluent-localization)
8. [File Size Limits (File Size Guard)](#8-file-size-limits-file-size-guard)
9. [Testing & The Mathematical Golden Oracle](#9-testing--the-mathematical-golden-oracle)
10. [Automated Quality Gates & Tooling (`xtask`)](#10-automated-quality-gates--tooling-xtask)
11. [Definition of Done (DoD) Checklist](#11-definition-of-done-dod-checklist)
12. [Binary Distributions & Native Installers](#12-binary-distributions--native-installers)
13. [Step-by-Step Procedure Cookbook](#13-step-by-step-procedure-cookbook)

---

## 1. ArtCraft Ecosystem Foundations

### 1.1. 100% Pure Safe Rust
* The entire project is written **exclusively in Rust** (2021 edition).
* **Handwritten JavaScript or TypeScript is strictly prohibited.** WebAssembly targets must use `wasm-bindgen` with automatically generated glue code.
* The codebase enforces `#![forbid(unsafe_code)]`. No `unsafe` blocks are permitted in production application code.

### 1.2. Licensing & Attribution
* The codebase is dual-licensed under: **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`).
* All third-party assets (such as Lucide icons or open fonts) must carry permissive licenses, with copies retained alongside the assets and registered in `ATTRIBUTION.md`.
* ArtCraft brand identity, logos, and trademarks in `assets/` and `docs/brand/` are proprietary and protected under `docs/brand/LICENSE-brand.txt`.

### 1.3. The Clean-Room Principle
* **STRICTLY FORBIDDEN:** Copying source code, shaders, binaries, icons, or proprietary assets from commercial software (such as IBM SPSS Statistics).
* All algorithms, file formats, and behaviors are implemented **solely via independent observation, open specifications, and academic literature**:
  - Statistical algorithms — classical literature (Kendall, Stuart, Agresti, Siegel, Fayyad & Irani);
  - Data formats (`.sav`, `.zsav`) — open GNU PSPP specifications and the ReadStat library (MIT);
  - Command syntax and menus — public user documentation without borrowing binary resources.

---

## 2. System Resource Guard (Compilation Limits)

To prevent resource exhaustion and protect developer machines and CI runners from freezing, the project enforces a **mandatory single-thread compilation constraint**:

* **NEVER execute `cargo build`, `cargo check`, `cargo test`, or `cargo clippy` without the `-j 1` flag:**
  ```bash
  cargo test --workspace -j 1
  cargo check --workspace -j 1
  cargo clippy --workspace --all-targets -j 1 -- -D warnings
  cargo build --package statcraft_app --release -j 1
  ```
* Running with `-j 2` is permissible only with explicit maintainer approval.
* Global configurations in `.cargo/config.toml` and `Cargo.toml` permanently fix `jobs = 1` and `codegen-units = 1`.

---

## 3. Reliability: The Never Crash Mandate

StatCraft is engineered for scientists, researchers, and data professionals working with mission-critical data. A crash resulting in data loss is unacceptable under any circumstances.

### 3.1. Prohibited Constructs in Production Code
Outside of `#[cfg(test)]` modules, the following constructs are **categorically prohibited**:
* ❌ `unwrap()`
* ❌ `expect(...)`
* ❌ `panic!(...)`
* ❌ `unreachable!(...)`
* ❌ `todo!(...)`
* ❌ `unimplemented!(...)`

### 3.2. Error Handling & Boundary Defense
1. Every fallible operation returns a typed `Result<T, E>`.
2. Errors are propagated upward using the `?` operator into strongly typed application domain errors (`CommandError`, `IoError`, `MathError`).
3. All indices, column offsets, matrix slices, and file boundaries must be defensively checked:
   ```rust
   // CORRECT:
   let col_idx = match dataset.find_column_index(var_name) {
       Some(idx) => idx,
       None => return Err(CommandError::ExecutionFailed(format!("Column not found: {}", var_name))),
   };

   // STRICTLY PROHIBITED:
   let col_idx = dataset.find_column_index(var_name).unwrap();
   ```
4. Mathematical algorithms must gracefully handle edge cases:
   - Zero division ($s = 0$, $df = 0$, singular matrices) $\to$ return `f64::NAN`, `None`, or `Null`, never panic;
   - Degenerate samples ($N = 0$, $N = 1$);
   - Missing data (`RawValue::Null`) — rigorous Listwise or Pairwise exclusion without panic.

---

## 4. Layered Engine Architecture (L0–L6)

StatCraft enforces a strict downward dependency hierarchy. Upward or circular dependencies are forbidden and caught by `cargo xtask layers`.

```text
L6: statcraft_app        (Desktop GUI: egui, eframe, egui_dock, DialogKit, Viewports)
 └── L5: statcraft_syntax (Engine CommandBus, .sps syntax parser & serializer)
      ├── L4: statcraft_reporting (Multi-level Pivot Tables, HTML/TSV/TXT/MD export)
      ├── L3: statcraft_charts    (Vector plotting, egui_plot, Brushing & Linking)
      └── L2: statcraft_algo      (Pure statistical algorithms, statrs, faer)
           └── L1: statcraft_core (In-memory structures: DataSet, VariableDictionary)
                └── L0: statcraft_io (Codecs: Parquet .stc, SPSS .sav, CSV, Excel .xlsx)

Cross-Cutting Modules:
* statcraft_i18n   (Zero-hardcode Fluent localization engine)
* statcraft_bridge (Model Context Protocol / MCP server and local IPC socket)
* xtask            (Automation harness: ci, layers, file-guard, design-guard, parity)
```

**Architecture Rules:**
* Layers L0 through L2 have zero knowledge of GUI frameworks (`egui`, `eframe`, windowing);
* The statistical engine (`statcraft_algo`) does not depend on the command bus (`statcraft_syntax`) or reporting views;
* Every new crate must be registered in `xtask/src/layers.rs`.

---

## 5. Command Architecture (Commands, Not Handlers)

StatCraft adheres strictly to the **Commands, Not Handlers** paradigm. The UI is completely decoupled from state mutation and cannot directly modify datasets or metadata.

### 5.1. Command Requirements
1. **Hierarchical Command Identifier:** Canonical SPSS-inspired path (`stat.descriptives`, `stat.ttest.independent`, `data.sort_cases`, `transform.optimal_binning`).
2. **Schema for AI Agents (`specs()`):** Every command exposes a machine-readable JSON Schema of its input parameters for autonomous invocation via MCP and CLI.
3. **Availability Predicate (`enabled()`):** Contextual validation logic determining whether the command is currently executable (e.g., Independent T-Test requires at least one scale variable and one grouping variable).
4. **History & Undo/Redo:** Execution produces a typed outcome and appends exactly one discrete transaction to the undo stack (`undo()`). User interactions (such as dragging or typing) must coalesce.
5. **Syntax Equivalence (`to_sps()`):** Every command must serialize to and deserialize from authentic SPSS syntax (`.sps`).
6. **Parity Registration:** New procedures must be tracked in `docs/parity-checklist.md` (verified by `cargo xtask parity`).

---

## 6. Dialog Design Standard (DialogKit) & Vector-First UI

### 6.1. DialogKit Canonical Components
All modal dialogs must be built using the canonical components in `crates/statcraft_app/src/dialogs/common/`:
* **`DialogShell`**: Manages window dimensions, centering, and modality:
  - `DialogSize::Standard` — $560 \times 430$ px (Standard procedure dialogs);
  - `DialogSize::Subdialog` — $360 \times 300$ px (Modal subdialogs: Statistics, Options);
  - `DialogSize::Properties` — $460 \times 380$ px;
  - `DialogSize::Compute` — $620 \times 480$ px.
* **`VariablePicker`**: Unified variable list displaying measurement scale icons and formatted as `Label [Name]`.
* **`TransferButton`**: Adaptive vector transfer button (`->` / `<-`), automatically oriented to the focused list.
* **`DialogActionBar`**: Canonical Type 1 action row (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`), uniformly sized ($\ge 80 \times 26$ px, corner radius 3 px). Raw `ui.button()` calls for primary actions are forbidden.
* **`SubdialogActionBar`**: Type 2 subdialog action row (`[ Continue ] [ Cancel ] [ Help ]`).
* **`SubdialogSidebar`**: Right-hand settings column (`[ Statistics... ]`, `[ Options... ]`, $90 \times 24$ px).
* **`GroupBox`**: Visual frame grouping related options and radios.

### 6.2. Vector-First UI: Ban on Emoji and Non-Standard Glyphs
`egui`'s default fonts do not bundle color emoji or obscure Unicode glyphs. Attempting to render them results in broken tofu boxes (`□`):
* **STRICTLY PROHIBITED — Color Emoji in UI text:**
  ❌ `📊`, `📈`, `⏳`, `⚠️`, `💡`, `📁`, `💾`, `⚙`, `🔍`, `✨`.
* **STRICTLY PROHIBITED — Obscure Unicode symbols:**
  ❌ `●` (U+25CF), `•` (U+2022), `✓`, `✔`, `✕`, `✖`.
* **RENDERING RULES:**
  1. Measurement scale icons (Nominal, Ordinal, Scale), connection status dots, and badges must be drawn **using vector primitives** via `ui.painter()` (`circle_filled`, `line_segment`, `rect_stroke`) or cached SVGs (`SvgIconCache`).
  2. Busy/Loading indicators must use `"..."` with active glow or an animated vector spinner.

---

## 7. Zero-Hardcode i18n & Fluent Localization

### 7.1. Prohibition of Natural Language String Literals
In GUI source code (`crates/statcraft_app`, dialogs, menus, tooltips), **string literals in natural languages (English, Russian, or any other) are strictly forbidden**.

```rust
// CORRECT:
ui.label(t!("dialog-optimal-binning-title"));
ui.heading(t!("menu-transform-optimal-binning"));

// STRICTLY PROHIBITED:
ui.label("Optimal Binning");
ui.heading("Оптимальная группировка");
```

### 7.2. Fluent Translation Catalogs (`.ftl`)
* Translations reside in declarative Fluent files: `locales/en.ftl` and `locales/ru.ftl`.
* New keys must be added **synchronously to both language catalogs**.
* Placeholder values (`TODO`, `FIXME`, `???`, `untranslated`) and missing keys cause `crates/statcraft_i18n/tests/i18n_lint_test.rs` to fail.
* External contributors can introduce new language packs (e.g., `locales/de.ftl`, `locales/es.ftl`) simply by placing the file in `locales/`, without recompiling the program.

---

## 8. File Size Limits (File Size Guard)

To preserve codebase maintainability and prevent context degradation for LLMs and human reviewers, strict file size ceilings are enforced:

| Module Classification | Hard Line Limit | Target Locations |
|---|---|---|
| **Leaf Functional Modules** | $\le 500$ lines | `crates/statcraft_algo/**`, `dialogs/**`, `views/output_view/**` |
| **Complex Interactive Editors** | $\le 1000$ lines | `data_view.rs`, `variable_view.rs`, `syntax_editor.rs` |
| **Root Application Coordinator** | $\le 1500$ lines | `crates/statcraft_app/src/app.rs` |
| **All Other Modules** | $\le 1000$ lines | `commands/**`, `model.rs`, `lib.rs` |

### "Decompose, Don't Prune" (Anti-Goodhart Counterweight)
* **Under no circumstances** may a module be squeezed under the line limit by stripping features, deleting table columns, or compressing logic into unreadable one-liners.
* The only valid solution for an oversized module is **architectural decomposition**: splitting into submodules (e.g., creating a subdirectory with `mod.rs` and focused submodules), extracting subdialogs, or modularizing builders.
* Verification: `cargo run -p xtask -j 1 -- file-guard`.

---

## 9. Testing & The Mathematical Golden Oracle

### 9.1. Mathematical Golden Oracle Verification
Every statistical procedure implemented in `statcraft_algo` must include unit tests verifying numerical outputs against reference benchmarks (Python `scipy.stats`, GNU PSPP, or NIST reference datasets):
* Test statistics ($t$, $F$, $\chi^2$, $z$, $U$), degrees of freedom ($df$), $p$-values, effect sizes ($\eta^2$), and confidence intervals must match reference standards to within **$\le 10^{-6}$** absolute tolerance;
* Edge cases must be tested: zero variance, identical ranks with tie-breaking, missing data (`Null`), and boundary sample sizes.

### 9.2. Interface Invariant Test Suites
* `crates/statcraft_app/tests/output_viewer_invariants.rs` — verifies that all table styles, Outline navigation trees, and TSV clipboard exports remain unbroken across changes;
* `crates/statcraft_app/tests/dialog_invariants.rs` — verifies that all modal windows adhere to the DialogKit geometry and component standards.

---

## 10. Automated Quality Gates & Tooling (`xtask`)

StatCraft bundles an automated maintenance suite accessible via `cargo xtask`.

### Full Quality Gate (Run this before committing):
```bash
cargo run -p xtask -j 1 -- ci
```
This single command executes the complete CI suite:
1. `cargo fmt --all -- --check` (Formatting);
2. `cargo clippy --workspace --all-targets -j 1 -- -D warnings` (Compiler linter across all targets);
3. `xtask layers` (L0..L6 architectural layering rules);
4. `xtask file-guard` (500 / 1000 / 1500 line limits);
5. `xtask design-guard` (DialogKit component compliance);
6. `cargo test --workspace -j 1` (100% test suite);
7. `xtask parity` (SPSS feature parity synchronization).

### Dedicated Commands:
```bash
# Check architectural layering rules
cargo run -p xtask -j 1 -- layers

# Check file size limits
cargo run -p xtask -j 1 -- file-guard

# Check DialogKit standard compliance
cargo run -p xtask -j 1 -- design-guard

# Verify SPSS feature parity catalog
cargo run -p xtask -j 1 -- parity

# Regenerate assets/icon.ico and PNGs from assets/icon.svg
cargo run -p xtask -j 1 -- icon
```

---

## 11. Definition of Done (DoD) Checklist

A pull request, feature, or bugfix is considered complete **only when 100% of the following requirements are fulfilled**:

- [ ] **1. Compilation & Formatting:**
  ```bash
  cargo check --workspace -j 1
  cargo clippy --workspace --all-targets -j 1 -- -D warnings
  cargo fmt --all -- --check
  ```
- [ ] **2. 100% Test Suite Pass:**
  ```bash
  cargo test --workspace -j 1
  ```
- [ ] **3. Automated Quality Gate Pass:**
  ```bash
  cargo run -p xtask -j 1 -- ci
  ```
- [ ] **4. File Size Compliance:** Every file strictly adheres to the 500 / 1000 / 1500 line limits (`xtask file-guard`).
- [ ] **5. Design Guard Compliance:** Dialogs use `DialogKit` exclusively with zero raw action buttons (`xtask design-guard`).
- [ ] **6. Zero-Hardcode i18n:** Zero natural language literals in GUI code; all keys mirrored in `locales/en.ftl` and `locales/ru.ftl`.
- [ ] **7. Documentation Synchronization:**
  - `docs/CHECKLIST.md` — completed items checked;
  - `docs/parity-checklist.md` — new procedures registered;
  - `docs/CHRONICLE.md` & `docs/CHRONICLE.en.md` — synchronous bilingual entries logged with timestamps.

---

## 12. Binary Distributions & Native Installers

StatCraft generates both portable archives and native operating system installers for all major desktop platforms:

* **Windows:**
  - Portable: `statcraft-windows-x64.zip`
  - Installer: `statcraft-windows-x64-installer.exe` (Inno Setup via `packaging/windows/installer.iss`)
* **Linux:**
  - Portable: `statcraft-linux-x64.tar.gz`
  - Package: `statcraft-linux-x64.deb` (`dpkg-deb` with menu integration via `packaging/linux/statcraft.desktop`)
* **macOS:**
  - Portable: `statcraft-macos-arm64.tar.gz` (Apple Silicon) & `statcraft-macos-x64.tar.gz` (Intel)
  - Disk Image: `statcraft-macos-arm64.dmg` & `statcraft-macos-x64.dmg` (`StatCraft.app` bundle via `hdiutil`)

### Local Release Build:
```bash
cargo build --package statcraft_app --release -j 1
```
The compiled binary is placed at `target/release/statcraft_app.exe` (Windows) or `target/release/statcraft_app` (Unix).

---

## 13. Step-by-Step Procedure Cookbook

Follow this 5-step workflow when adding any new statistical procedure or data transformation:

1. **L2: Algorithmic Core (`crates/statcraft_algo/src/<name>.rs`):**
   - Write pure mathematical calculations using Safe Rust without panics;
   - Implement Golden Oracle unit tests verified against SciPy ($\le 10^{-6}$);
   - Ensure file size is $\le 500$ lines;
   - Register module in `crates/statcraft_algo/src/lib.rs`.

2. **L5: Engine Command (`crates/statcraft_syntax/src/commands/<name>_cmd.rs`):**
   - Implement the `Command` trait (`execute`, `undo`, `specs`, `to_sps`);
   - Register the command in `CommandBus` (`crates/statcraft_syntax/src/bus.rs`);
   - Add command definition to `docs/parity-checklist.md`.

3. **Localization (`locales/en.ftl` & `locales/ru.ftl`):**
   - Add dialog titles, labels, option strings, and status messages in both English and Russian.

4. **L6: Graphical Interface (`crates/statcraft_app/src/dialogs/<name>_dialog.rs`):**
   - Implement the modal dialog using `DialogShell::standard`;
   - Employ `VariablePicker`, `TransferButton`, `GroupBox`, and `DialogActionBar`;
   - Hook dialog trigger into `app_menus` and event handler into `app_dialogs.rs`;
   - Add invariant verification test to `tests/dialog_invariants.rs`.

5. **Verification & Audit:**
   - Execute `cargo run -p xtask -j 1 -- ci`;
   - Append bilingual entries to `docs/CHRONICLE.md` and `docs/CHRONICLE.en.md`;
   - Commit following Conventional Commits format (`feat(category): description`).

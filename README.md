<div align="center">
  <img src="assets/icon-128.png" width="128" height="128" alt="StatCraft Logo" />
  <h1>StatCraft</h1>
  <p><strong>High-performance, memory-safe, and open-source statistical analysis package</strong></p>
  <p><em>ArtCraft Ecosystem Standard | Built with 100% Safe Rust</em></p>

  <p>
    <a href="README.md"><strong>English</strong></a> |
    <a href="README.ru.md">Русский</a>
  </p>

  <p>
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust" alt="Rust 1.80+" /></a>
    <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License: MIT OR Apache-2.0" /></a>
    <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg" alt="Platform: Windows | Linux | macOS" />
    <img src="https://img.shields.io/badge/Architecture-100%25%20Safe%20Rust-brightgreen.svg" alt="100% Safe Rust" />
    <img src="https://img.shields.io/badge/Tests-Passing%20(90%2B)-success.svg" alt="Tests: 90+ Passing" />
    <img src="https://img.shields.io/badge/i18n-EN%20%7C%20RU-informational.svg" alt="i18n: EN | RU" />
  </p>
</div>

---

## Overview

**StatCraft** is a free, modern, open-source statistical software package built entirely in **100% pure Safe Rust**. Designed as an uncompromising, clean-room alternative to proprietary statistical suites such as IBM SPSS Statistics, StatCraft combines academic precision, instantaneous launch times, and guaranteed crash immunity.

Whether you are conducting psychological research, analyzing clinical trials, teaching university statistics, or manipulating enterprise datasets, StatCraft provides an intuitive graphical interface alongside a deterministic, syntax-driven analytical engine.

---

## Core Pillars & Design Principles

* **100% Safe Rust & Never Crash Guarantee:** Zero `unsafe` blocks, zero panics (`panic!`, `unwrap()`, `expect()`). Malformed datasets, empty samples, or division by zero yield explicit, typed `Result<T, E>` errors rather than application crashes.
* **Commands, Not Handlers:** Every analytical procedure and user interaction is an atomic engine command (`EngineCommand`). All commands support non-destructive Undo/Redo, serialize to standard SPSS syntax (`.sps`), and can be scripted headlessly.
* **Agent-First Architecture:** Integrated Model Context Protocol (MCP) server and local IPC socket allow AI agents (Antigravity, Claude, local LLMs) to discover variable schemas, execute complex pipelines, and insert interpreted results into the Output Viewer.
* **SPSS-Style Canonical Output Viewer:** Hierarchical navigation outline tree paired with authentic multi-level pivot tables. Results can be copied to the clipboard with one click as structured TSV data that pastes cleanly into Microsoft Excel without distorted columns or unwanted date conversions.
* **Vector-First UI:** Rendered with native `egui` via `ui.painter()` and vector SVG icons. Strict zero-emoji and zero-unsupported Unicode symbol policy eliminates tofu squares (`□`) across all platforms.
* **Zero-Hardcode i18n:** 100% of user-facing strings are decoupled into Project Fluent catalogs (`locales/en.ftl`, `locales/ru.ftl`) with real-time runtime language switching.
* **Resource Guard:** Thread-limited builds (`-j 1`, `codegen-units = 1`) protect development workstations from memory exhaustion.

---

## Key Features

### 1. Dual-View Data Management
* **Data View:** High-performance, virtualized tabular grid displaying cases and variables. Supports instant toggling between raw numeric values and human-readable value labels (`1/A`).
* **Variable View:** Comprehensive metadata dictionary managing:
  - **Name:** Normalized variable identifiers.
  - **Type:** 9 canonical SPSS types (Numeric, Comma, Dot, Scientific, Date, Dollar, Custom Currency, String, Restricted Numeric).
  - **Width & Decimals:** Visual width and precision control with custom vector spinners.
  - **Labels:** Descriptive variable titles displayed in dialogs and tables.
  - **Values:** Discrete and range value labels with dedicated modal editors.
  - **Missing Values:** Configurable discrete missing values and continuous missing ranges.
  - **Columns & Align:** Table layout and alignment (Left, Center, Right).
  - **Measure:** Measurement level classification (Scale, Ordinal, Nominal) with custom vector scale icons.

### 2. Parametric & Descriptive Statistical Procedures
* **Descriptive Statistics:** Mean, Std. Error Mean, Standard Deviation, Variance, Minimum, Maximum, Range, Sum, Skewness (with SE), Kurtosis (with SE), and Valid N (listwise).
* **Frequencies:** Frequency counts, percentages, valid percentages, cumulative percentages, discrete missing value breakdown, percentiles, and quartiles.
* **One-Sample T-Test:** Student's $t$-test against customizable test values, degrees of freedom, two-tailed significance, mean differences, confidence intervals ($95\%$ default or user-defined), and dual-table SPSS output.
* **Independent-Samples T-Test:** Two-group comparison with automatic Levene's Test for Equality of Variances ($F$, $p$), Student's $t$-test (Equal Variances Assumed), Welch's $t$-test (Equal Variances Not Assumed), and full Group Statistics table.
* **Paired-Samples T-Test:** Dependent group comparison providing Paired Samples Statistics, Pearson Correlation coefficients ($r$, $p$), and Paired Differences test tables.
* **One-Way ANOVA:** One-Way Analysis of Variance with Fisher's $F$-test, between-groups and within-groups sums of squares, Levene's Test of Homogeneity of Variances, and descriptive statistics.

### 3. Canonical Output Viewer
* **Outline Tree Navigation:** Interactive document tree on the left panel to jump directly to any procedure, log, or table.
* **Pivot Tables:** Multi-tier nested tables with authentic SPSS headers, alternating row striping, and right-aligned numeric formatting.
* **TSV Clipboard Export:** One-click `[ Copy Table (TSV) ]` button with instant visual feedback and right-click context menus. Exports tables with clean headers and row separators specifically structured for Excel import.
* **Execution Protocols:** Monospace log blocks documenting executed SPSS syntax (`GET DATA`, `DESCRIPTIVES`, `T-TEST`, `ONEWAY`).

---

## Layered Architecture (L0 – L6)

StatCraft is engineered in strict compliance with the **ArtCraft Layering Law**. Dependencies flow strictly downwards:

```text
L6: statcraft_app        Desktop GUI (egui / eframe / DialogKit / Output Viewer)
 └── L5: statcraft_syntax CommandBus, SPS syntax parser, engine transactions
      ├── L4: statcraft_reporting Pivot table models, TSV / PDF / LaTeX export
      ├── L3: statcraft_charts    Vector graphics, egui_plot, brushing & linking
      └── L2: statcraft_algo      Pure statistical math (Golden Oracle vs SciPy)
           └── L1: statcraft_core In-memory dataset, Arrow buffers, VariableDictionary
                └── L0: statcraft_io  Data codecs (CSV/TSV streaming, Parquet, SPSS .SAV)

Cross-cutting utility crates:
* statcraft_i18n   Fluent localization engine with zero hardcoded strings
* xtask            Developer tools: file-guard, layers-guard, design-guard, parity
```

---

## Getting Started

### Prerequisites
* [Rust toolchain](https://rustup.rs/) (version 1.80 or newer recommended).

### Building and Running
Clone the repository and launch the desktop application:

```bash
git clone https://github.com/storytold/StatCraft.git
cd StatCraft

# Run StatCraft in release mode (strictly single-threaded compiler guard)
cargo run --package statcraft_app --release -j 1
```

### Running Tests and Quality Guards

StatCraft enforces a strict **Definition of Done (DoD)**. All tests and architectural linters must pass with zero warnings:

```bash
# Run all workspace unit, integration, and golden oracle tests
cargo test --workspace -j 1

# Check file size limits across all crates (anti-monolith guard)
cargo run -p xtask -j 1 -- file-guard

# Check architectural layering rules (L0 - L6 strict downwards dependencies)
cargo run -p xtask -j 1 -- layers

# Check dialog UI design system compliance (DialogKit guard)
cargo run -p xtask -j 1 -- design-guard

# Verify SPSS feature parity status
cargo run -p xtask -j 1 -- parity
```

---

## Project Structure

| Crate / Path | Responsibility |
|---|---|
| [`crates/statcraft_app`](crates/statcraft_app) | Desktop user interface, docking windows, DialogKit, data grid, output viewer |
| [`crates/statcraft_syntax`](crates/statcraft_syntax) | Engine command bus, transactions, undo/redo, SPSS syntax serializer |
| [`crates/statcraft_algo`](crates/statcraft_algo) | Mathematical algorithms (t-tests, ANOVA, Levene, frequencies, descriptives) |
| [`crates/statcraft_core`](crates/statcraft_core) | In-memory `DataSet`, `VariableDictionary`, measurement scales, metadata |
| [`crates/statcraft_io`](crates/statcraft_io) | Fast CSV/TSV streaming reader, upcoming SPSS `.sav` and Parquet codecs |
| [`crates/statcraft_i18n`](crates/statcraft_i18n) | Localization engine powered by Project Fluent (`locales/*.ftl`) |
| [`xtask`](xtask) | CI/CD automation: `file-guard`, `layers`, `design-guard`, `parity` |
| [`docs/`](docs) | Architecture guides, design specs, roadmaps, and development chronicle |

---

## Documentation

* [SPSS Feature Parity Checklist](docs/parity-checklist.md) — Comprehensive tracking of implemented SPSS commands and menus.
* [Strategic Roadmap](docs/ROADMAP.md) — Long-term technical phases and module milestones.
* [Canonical Design Guide](docs/DESIGN_GUIDE.md) — Standard layout specifications for all modal dialogs (`DialogKit`).
* [Development Chronicle (EN)](docs/CHRONICLE.en.md) / [Хроника разработки (RU)](docs/CHRONICLE.md) — Detailed historical devlog of every architectural milestone.

---

## License

This project is dual-licensed under:
* **MIT License** ([LICENSE-MIT](LICENSE-MIT))
* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

You may choose either license at your option.

All brand assets and logos in `assets/` and `docs/brand/` are proprietary to the **storytold / ArtCraft** project and are subject to the brand guidelines.

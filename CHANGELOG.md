# Changelog

All notable changes to the **StatCraft** statistical computing application will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.7] - 2026-11-25

### Added
- **Bootstrap Resampling Core & Mersenne Twister PRNG (Feature 020 — Stage 1):**
  - Implemented deterministic 100% Safe Rust 32-bit Mersenne Twister (`Mt19937`) in `statcraft_core::rng` with unbiased rejection sampling (`gen_range_usize`) and `from_optional_seed()` fallback.
  - Implemented reusable bootstrap engine in `statcraft_algo::bootstrap` (`options.rs`, `resampler.rs`, `ci.rs`, `engine.rs`) supporting `Simple` and `Stratified` (preserving exact stratum sizes $n_h$) resampling with replacement.
  - Implemented `Bias`, `Std. Error`, `Percentile` CI, and `BCa` CI (bias correction $z_0$, jackknife acceleration $\hat{a}$ with safe zero-denominator fallback, and Cody rational `normal_cdf`).
  - Added SPSS `SET MTINDEX=...` and `BOOTSTRAP /SAMPLING ... /CRITERIA ...` syntax generation and one-shot `pending_bootstrap` batch modifier in `statcraft_syntax::bootstrap` and `SyntaxDispatcher::execute_batch`.
- **Reusable Bootstrap Dialog & Frequencies Integration (Feature 021 — Stage 2):**
  - Built reusable `BootstrapDialog` (`crates/statcraft_app/src/dialogs/bootstrap_dialog/`) using `DialogKit` (`DialogShell`, `GroupBox`, `VariablePicker`, `TransferButton`, `SubdialogActionBar`) with transactional `backup`/`commit`/`cancel` and validation.
  - Added full Fluent localization in `locales/en.ftl` and `locales/ru.ftl` (zero hardcoded UI strings).
  - Enabled `[ Bootstrap... ]` in `FrequenciesDialog`, wired `compute_frequencies_bootstrap` (`statcraft_algo::frequencies::bootstrap`), and added `Bootstrap Specifications` + bootstrap statistics/frequency tables with TSV export in Output Viewer (`tables_frequencies/bootstrap_tables.rs`).
- **Multi-Procedure Bootstrap Expansion & Golden Oracle (Feature 022 — Stage 3):**
  - Extracted shared Output Viewer bootstrap rendering and TSV helpers in `crates/statcraft_app/src/views/output_view/tables_bootstrap.rs`.
  - Connected `BootstrapDialog`, `statcraft_algo`, `statcraft_syntax` (`to_sps_with_bootstrap`), and Output Viewer across 8 additional procedures: `Descriptives`, `Means`, `One-Sample T-Test`, `Independent-Samples T-Test`, `Paired-Samples T-Test`, `One-Way ANOVA`, `Bivariate Correlations` (Pearson, Spearman, Kendall), and `Linear Regression` (OLS).
  - Added Golden Oracle verification (`crates/statcraft_algo/tests/bootstrap_golden_oracle.rs`) matching `scipy.stats.bootstrap` (`Percentile` and `BCa`) within $\le 10^{-6}$, plus end-to-end Output Viewer & TSV tests in `crates/statcraft_app/tests/output_viewer_bootstrap_invariants.rs`.

---

## [0.4.6] - 2026-11-24

### Added
- **Table Style & Conditional Formatting Parity (Feature 019 / IBM SPSS OUTPUT MODIFY):**
  - Implemented the complete typed conditional formatting engine in `statcraft_reporting::styling` (`CellStyleFormat`, `CellCondition`, `TableStyleRule`, `CellTargetScope`, `ConditionalFormattingEngine`).
  - Added support for 12 value predicate conditions: `All`, `Equal`, `NotEqual`, `LessThan`, `LessOrEqual`, `GreaterThan`, `GreaterOrEqual`, `Between`, `NotBetween`, `TopN`, `BottomN`, `SysMis`.
  - Added target dimension selection (`Columns`, `Rows`, `Both`) and value targeting (`AllDataCells`, label filter).
  - Added 3-way application scope in `format_subdialog.rs`: `Only Cells` (default), `Entire Column`, and `Entire Row`.
  - Added replacement value substitution (`replace_value`) allowing users to replace matching cell values with custom strings.
- **Authentic IBM SPSS Table Style Dialog Suite (DialogKit):**
  - Main Dialog `TableStyleDialog`: 5 in-cell ComboBox columns (*Table, Value, Dimension, Condition, Format*), live rule summary indicators, rule reordering (`[ Move Up ]`, `[ Move Down ]`), rule cloning (`[ Duplicate ]`), and transactional rollback on cancel.
  - Condition Subdialog `TableStyleConditionSubdialog`: Value targeting, `[ ] Absolute Value` checkbox for modulus magnitude comparisons, lower/upper range bounds, and Top/Bottom N inputs.
  - Format Subdialog `TableStyleFormatSubdialog`:
    - Top `Preview` box with live sample text `123` reflecting font style, colors, and borders in real-time.
    - Typography toolbar with active selection states: `[ B ]`, `[ I ]`, `[ U ]`.
    - Authentic 5x6 SPSS Color Matrix Palette (`color_palette.rs`) with dedicated swatches for White, Black, No Color (diagonal red slash), 30 canonical SPSS color shades, and custom RGB picker.
    - Comprehensive 34 cell format masks (`format_masks.rs`): numbers, decimals, scientific notation, percentages, quarters, weeks, timestamps, and named days/months.
    - Decimals spinner and TableLook defaults reset switch.
- **Output Viewer Styled Table Rendering & Reporting:**
  - High-performance vector rendering in Output Viewer (`tables_common.rs`) with cell background fills, custom text colors, and synthetic faux bold (`egui::text::LayoutJob` with overstrike) preserving italic slant and underline simultaneously without font suppression (`BUG-019`).
  - Implemented auxiliary "Conditional Styles Report" audit table summarizing applied targets, conditions, and modified cell tallies.
  - Added canonical IBM SPSS `OUTPUT MODIFY` syntax generation upon Paste action (`/SELECT TABLES /IF SUBTYPES=['Frequencies'] /TABLECELLS SELECT=[DIMENSION=...] CONDITION=... STYLE=[...]`).
  - Updated HTML report export to serialize inline CSS styles preserving cell formatting, background colors, and underlines.
- **Zero-Hardcode Fluent Localization:**
  - Added 44+ semantic translation keys in `locales/ru.ftl` and `locales/en.ftl` covering all 34 format masks, colors, and dialog labels.

### Fixed
- **Architectural Decomposition of App Dialog Coordinator (BUG-018):**
  - Proactively decomposed `crates/statcraft_app/src/app_dialogs.rs` to prevent exceeding the 1000-line coordinator ceiling (Rule 12.1/12.2), extracting canonical descriptive procedures into `crates/statcraft_app/src/app_dialogs_stats.rs` (284 lines).
- **Typography Rendering in Output Viewer & Format Preview (BUG-019):**
  - Resolved `egui` italic and bold conflict where `painter.text` suppressed italics and `RichText::strong` was overridden by custom text colors. Synthesized robust faux bold over slanted `Galley` via `LayoutJob`.

---

## [0.4.5] - 2026-11-22

### Added
- **Frequencies: Charts Subdialog & Complete SPSS Parity (Feature 017):**
  - Implemented authentic canonical SPSS subdialog «Frequencies: Charts» (`Частоты: Диаграммы`) via DialogKit `DialogShell::subdialog(360, 300)` (`crates/statcraft_app/src/dialogs/frequencies_dialog/charts_subdialog.rs`).
  - Added radio group *Chart Type*: None (default), Bar charts, Pie charts, and Histograms with indented checkbox *Show normal curve on histogram*.
  - Added radio group *Chart Values*: Frequencies (default) and Percentages with contextual interactivity (authentically enabled only for Bar and Pie charts, disabled when None or Histograms is selected).
  - Added subdialog state management with cancellation rollback and reset support in `FrequenciesDialogState`.
  - Added bidirectional SPSS syntax support: serialization to `/BARCHART [FREQ|PERCENT]`, `/PIECHART [FREQ|PERCENT]`, and `/HISTOGRAM [NORMAL]` on Paste, and syntax parser recognition in `statcraft_run_syntax`.
- **Vector-First Chart Rendering in Output Viewer (`statcraft_app`):**
  - Implemented 100% pure vector chart rendering using `ui.painter()` (zero emojis, zero non-standard Unicode glyphs) with tokens-aware theme styling:
    - `freq_chart_bar_pie.rs`: Bar charts with category X-axis, frequency/percentage Y-ticks, hover highlights, value labels, and Pie charts with vector polygon slice fans, adaptive palette, and percentage legend.
    - `freq_chart_histogram.rs`: Continuous contiguous bins, smooth overlaid Gaussian density curve, and authentic top-right SPSS Statistics Legend Box (`Mean`, `Std. Dev.`, `N`).
  - Integrated charts into Output Viewer Outline navigation tree with dedicated vector chart icons (`paint_output_chart_icon`).
  - Added structured tabular and textual fallback summaries for chart items during TSV and HTML export.
- **Mathematical Histogram Binning & Normal Density Scaling (`statcraft_algo`):**
  - Implemented pure algorithmic module `crates/statcraft_algo/src/frequencies/charts.rs`:
    - Adaptive Freedman-Diaconis and Scott bin width calculation with "nice" step rounding ($1, 2, 2.5, 5, 10 \times 10^k$).
    - Scaled Gaussian density curve overlay matching histogram frequency area: $y = \frac{N \cdot h}{\sigma \sqrt{2\pi}} \exp\left(-\frac{(x - \mu)^2}{2\sigma^2}\right)$.
    - Robust edge-case handling for zero variance ($s = 0$), sample sizes $N < 2$, missing values, and non-numeric variables.
  - Added Golden Oracle validation suite in `frequencies_charts_oracle_test.rs` matching SciPy reference outputs.

### Fixed
- **Frequencies Summary Statistics on Numeric Nominal Variables (Feature 016 Parity):**
  - Removed artificial suppression of summary statistics and percentiles for numeric variables marked with `Measure::Nominal`, bringing full parity with IBM SPSS Statistics.
  - Replaced raw technical identifiers with authentic SPSS Variable Labels (`Label [Name]`) in frequency table titles and summary statistics column headers.
  - Synchronized Output Viewer executed syntax log with exact `/STATISTICS` and `/PERCENTILES` subcommands.

---

## [0.4.4] - 2026-11-22

### Fixed
- **Windows 10 Borderless Window Artifacts & Mouse Cursor Offset (BUG-017):**
  - Integrated safe Win32 API wrapper `winsafe = "0.0.29"` in `statcraft_app` (100% Safe Rust without `unsafe` blocks in application code).
  - Implemented window frame guard `crates/statcraft_app/src/window_frame.rs` following the PhotoCraft canon.
  - Dynamically strips `WS_CAPTION | WS_BORDER` styles forced by `winit` on Windows 10 and forces frame recalculation (`SWP_FRAMECHANGED`) on every frame, eliminating the 26 px black bar at the top and side border artifacts.
  - Restored 1:1 mouse hit-test precision across menus, tabs, and controls without vertical offset.
  - Added `keep_clear_of_the_taskbar` to prevent maximized custom-frame windows from clipping behind or over the Windows taskbar, with support for both standard and auto-hiding taskbars.
- **Modal Dialog Bloating Protection on Long Variable Labels (BUG-017):**
  - Added `.max_size(size)` and `.constrain_to(ctx.screen_rect())` to `DialogShell::show`, strictly preventing procedure dialogs from stretching beyond screen boundaries.
  - Constrained `ScrollArea::vertical().max_width(inner_size.x)` in `variable_picker.rs`.
  - Added `.truncate()` to selectable list item buttons and attached full variable descriptions via `.on_hover_text(...)` tooltips so long survey questions (70–90+ characters) never push action buttons off-screen.

### Added
- **Window Frame Unit Tests (`window_frame.rs`):**
  - Added 7 unit tests covering borderless style bitmasks, taskbar clamping on all 4 monitor edges, auto-hiding taskbar 1-pixel reservation, and monitor edge detection.

---

## [0.4.3] - 2026-11-20

### Fixed
- **Engine Calculation Precision & Full-Precision Cell Access (PR #5 by @jjohare):**
  - Eliminated inadvertent numerical truncation caused by reading cell values through display formatting (`get_cell_display`). Statistical procedures now compute on full-precision 64-bit IEEE 754 stored values.
  - Added `DataSet::get_cell_numeric` to `statcraft_core`, directly returning `Option<f64>` without intermediate heap string allocations or decimal precision loss.
  - Migrated 21 procedure commands in `statcraft_syntax`: DESCRIPTIVES, REGRESSION, CORRELATIONS, PARTIAL CORR, ANOVA (One-Way), GLM (Univariate, Multivariate, Repeated Measures), t-tests (One-Sample, Independent, Paired), Nonparametric tests (Mann-Whitney, Kruskal-Wallis, Related Samples), CURVEFIT, FACTOR, RELIABILITY, LOGISTIC REGRESSION, MULTINOMIAL LOGISTIC, MEANS, and SELECT IF.
  - Fixed calculations on columns formatted as `Dollar` or `Comma` (e.g. `$1,234.56`), which previously caused float parsing failures.
  - Preserved display-string matching for categorical factor levels, grouping keys, and value labels.
- **GUI Dialog Category Level Precision (`statcraft_app`):**
  - Updated distinct dependent variable level counting in `logistic_regression_dialog.rs` and `multinomial_logistic_dialog/mod.rs` to use `ds.get_cell_numeric`, preventing distinct small-delta numeric categories (e.g., 0.001 vs 0.002) from collapsing under default `F8.2` print formats.

### Added
- **High-Precision Regression & Statistical Tests:**
  - Added `crates/statcraft_syntax/tests/numeric_precision_tests.rs`: verified exact numerical equality to within $10^{-9}$ for mean, minimum, slope, intercept, and Pearson $r$.
  - Added `crates/statcraft_app/tests/numeric_dialog_precision_tests.rs`: verified distinct category counting and currency format handling.

---

## [0.4.2] - 2026-11-19

### Added
- **Dynamic Dataset Colors & Container Persistence (Spec-Kit 015):**
  - High-performance, collision-aware random contrast color generator on 64-bit SplitMix64 and golden angle distribution ($137.50777^\circ$, $\ge 45^\circ$ angular distance between open tabs).
  - Cross-pane color identity: DocumentTabStrip active underlines, vector grid icons, Outline Tree procedure chips, and Notes summary tables synchronized via `workspace.session_color_by_name`.
  - Full `.stc` project container persistence (`pub tab_color: Option<String>`) with 100% backward compatibility with legacy containers.

### Fixed
- **Tab Dirty Marker Positioning & Overlap Elimination (BUG-016):**
  - Removed hardcoded character truncation (`min(12) * 6.5`) that caused asterisk markers to render directly over filename characters on tabs with titles longer than 12 characters.
  - Added guaranteed right padding reservation (32 px), clean ellipsis truncation via `truncate_at_width`, and vertical centering with `Align2::LEFT_CENTER`.
- **Titlebar False Dirty Marker & Unicode Compliance (BUG-016):**
  - Completely removed hardcoded Unicode bullet symbol ` •` (U+2022) from window titlebar in accordance with Vector-First UI rules.
  - Normalized window title to render strictly `file_name` when clean and `file_name *` when modified.
  - Refactored `has_unsaved_changes()` to semantically isolate active session state from global checkpoints, eliminating false dirty triggers upon switching between datasets with differing column/row counts.
  - Fixed CSV/TSV/TXT saving in `save_to_path` to properly clear the session dirty flag via `sess.mark_clean()`.

---

## [0.4.1] - 2026-11-19

### Added
- **Multi-File Workspaces & Concurrent Document Sessions (Feature 010):**
  - Open, edit, and switch between multiple datasets simultaneously in a unified workspace with isolated Undo/Redo histories, filter states, and split-file contexts.
  - Lossless in-memory compression: `NullBitmask`, `DictionaryEncodedColumn`, and `LosslessCompressedColumn` retaining full `f64` precision.
  - SPSS syntax commands for datasets: `DATASET NAME`, `DATASET ACTIVATE`, `DATASET CLOSE`, and `DATASET DISPLAY`.
  - Vector document tab strip (`DocumentTabStrip`) with custom vector dataset icons, unsaved markers, close buttons, mouse wheel scrolling, chevron overflow navigators, and `Ctrl+Tab` / `Ctrl+Shift+Tab` / `Ctrl+W` shortcuts.
- **Cross-Platform Memory Guard (Feature 011):**
  - Native physical RAM probing across Windows (`GlobalMemoryStatusEx`), Linux (`/proc/meminfo`), macOS (`sysctl`), and WebAssembly.
  - Preflight memory safety warnings before opening large files (>250 MB) or batch operations (>10 files) when available RAM is critically low (<500 MB or <15%).
- **Data Merging: Add Cases & Add Variables (Feature 012):**
  - Add Cases (`ADD FILES`): vertical concatenation from external files (`.sav`, `.zsav`, `.stc`, `.csv`, `.xlsx`) or active workspace tabs with schema alignment, type reconciliation, `/IN=var` tracking, and `/BY` sorting.
  - Add Variables (`MATCH FILES`): horizontal merging across datasets (1:1 key merges, 1:N lookup tables `/TABLE`, sequential merges without keys) with collision resolution and `/IN` flags.
  - Canonical DialogKit interfaces (`AddCasesDialog`, `AddVarsDialog`).
- **Grid Selection & Interaction Modernization (Spec-Kit 013):**
  - Preserved multi-cell selection on right-click.
  - Zero Pollution invariant on phantom rows (`*`): empty input commits never allocate unwanted rows with system-missing values.
  - Physical case deletion via `Delete` key on full rows, and cell clearing on rectangular selections.
  - Row and column header context menus (`ContextMenuTarget`).
  - Click-outside dismissal for popups and context menus without requiring `Escape`.

### Fixed
- **SPSS Canon Range Selection & Fill Handle Removal (Spec-Kit 014 / BUG-014):**
  - Eliminated the corner fill handle click interceptor, converting all grid dragging to strictly safe, read-only range selection (`SelectRange`).
  - Constrained `apply_auto_fill` strictly to existing dataset rows, completely barring phantom row allocations.

---

## [0.4.0] - 2026-11-16

### Added
- **Virtualized 4-Quadrant 2D Grid (Feature 006):**
  - Dedicated spatial geometry engine `grid/` separating top-left corner, column variable headers (`quad_top_right`), row number headers (`quad_bottom_left`), and central cell grid (`quad_center`).
  - Prefix-sum spatial line indexer (`LineIndex`) providing $O(\log N)$ coordinate searches with zero heap allocation in uniform mode.
  - Interactive column width drag-resizing with dynamic cursor feedback and guideline rendering.
  - Double-click column Auto-fit calculating optimal width from content.
  - Two-way column width synchronization with SPSS dictionary metadata (`VariableMetadata.columns`).
- **Range Selection, Keyboard Navigation & Direct Typing (Feature 007):**
  - Fluid mouse marquee selection and Shift-click / Shift-drag range expansion.
  - Complete keyboard navigation (Arrow keys, `Tab`, `Enter`, `Home`, `End`, `Ctrl+Home`, `Ctrl+End`).
  - Direct Typing: Typing alphanumeric keys immediately begins inline cell editing without prior mouse clicking.
  - Floating inline cell editor with distinct focus ring and auto-commit on Enter/Tab.
  - Rectangular TSV/CSV clipboard transfer (`Ctrl+C` / `Ctrl+V`) with automatic dataset row expansion.
  - Series Auto-Fill (Fill Handle): $6 \times 6$ px square marker calculating linear regression progressions for numeric series and repetition for categorical data.
- **Column AutoFilter, Quick Sort & Data Integrity Guards (Feature 008):**
  - Column header AutoFilter button with vector chevron and active funnel indicators.
  - Floating filter dropdown with live search, "Select All" / "Clear All" toggles, unique values checklist, and explicit "(Blanks)" handling.
  - Quick Sort Ascending (A–Z) and Descending (Z–A) via engine command `data.sort_cases` with 100% reversible Undo/Redo and SPSS syntax logging (`SORT CASES BY ...`).
  - Cell right-click context menu (Filter by selected value, Clear column filter, Sort Ascending/Descending, Copy/Paste).
  - Visual strikethrough for filtered inactive rows and muted text in cells (SPSS `FILTER BY` parity).
  - Data integrity inspection: Automated detector `is_number_stored_as_text` with Excel-parity green corner triangle badge and instant click-to-convert to numeric.
- **Tab Lifecycle Management & Core Tab Retention (Feature 005):**
  - Non-closeable protection (`is_closeable = false`) for Data View, Variable View, and Output Viewer tabs.
  - Closeable auxiliary tabs (Syntax Editor) with standard close buttons.
  - Window menu for fast restoration and tab switching.
  - Automatic Output Viewer focus when executing SPSS syntax.

### Fixed
- Fixed opaque selection background covering cell text by rendering background fill underneath the text layer.
- Fixed cell double-click race condition to ensure instant editor focus and caret placement.
- Localized column border hit-testing strictly to header quadrant to prevent accidental drag triggers inside data rows.

---

## [0.3.1] - 2026-11-15

### Added
- **Advanced Regression & Partial Correlations (Feature 001):**
  - Partial correlations (`PARTIAL CORR` / `stat.correlations.partial`) with Schur complement and Student's $t$ $p$-values.
  - Binary logistic regression (`LOGISTIC REGRESSION` / `stat.regression.logistic`) via IRLS with Newton-Raphson optimizer and Hosmer-Lemeshow test.
  - Curve estimation (`CURVEFIT` / `stat.regression.curve`) with 6 canonical SPSS curves.
- **General Linear Model (Feature 002):**
  - Univariate ANOVA/ANCOVA (`UNIANOVA` / `stat.glm.univariate`) with Type III Sum of Squares.
  - Multivariate ANOVA (`GLM Multivariate` / `stat.glm.multivariate` — MANOVA) with Wilks' $\Lambda$, Pillai's, Hotelling-Lawley, and Roy's tests.
  - Repeated measures ANOVA (`GLM Repeated Measures` / `stat.glm.repeated`) with Mauchly's sphericity test and Greenhouse-Geisser / Huynh-Feldt corrections.
- **Cluster Analysis, Reports & Survival (Feature 003):**
  - K-Means cluster analysis (`QUICK CLUSTER` / `stat.cluster.kmeans`) with Maximin initialization and dynamic variable generation (`QCL_1`, `QCL_2`).
  - Variable codebook (`CODEBOOK` / `stat.reports.codebook`).
  - Case summaries (`SUMMARIZE` / `stat.reports.summarize`).
  - Kaplan-Meier survival analysis (`KM` / `stat.survival.km`) with Log-Rank test.
  - Actuarial life tables (`SURVIVAL` / `stat.survival.lifetables`).
- **Windows Silent Launch:**
  - Added `#![windows_subsystem = "windows"]` to eliminate black terminal window when launching GUI.

---

## [0.3.0] - 2026-11-07

### Added
- Model Context Protocol (MCP) server & local TCP socket bridge (port 18493) for autonomous AI assistants.
- Universal Theme System (Dark Pro, SPSS Classic Light, System Auto) with lock-free atomic switching.
- System language autodetection via native OS APIs with fallback to English.

---

## [0.2.0] - 2026-10-31

### Added
- Classical statistical procedures: Descriptives, Frequencies, Crosstabs, One-Sample, Independent-Samples, and Paired-Samples T-Tests, One-Way ANOVA.
- Bivariate Pearson/Spearman correlations, Linear Regression, Non-Parametric tests (Chi-Square, Binomial, Runs, Mann-Whitney U, Wilcoxon, Kruskal-Wallis).
- Dynamic pivot tables in Output Viewer with multi-tier hierarchies and formatting.

---

## [0.1.0] - 2026-10-28

### Added
- Initial workspace scaffolding: L0 I/O (CSV/TSV), L1 Core (`DataSet`, `VariableDictionary`), L5 Syntax, and L6 GUI (`egui_dock`).
- Bilingual localization engine (`statcraft_i18n`) with Fluent catalogs (`en.ftl`, `ru.ftl`).

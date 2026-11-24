# StatCraft — IBM SPSS Statistics Canonical UI & Design Audit (Phase 3)

**English (Primary)** | [ Русский ](SPSS_UI_CANON_AUDIT_RU.md)

> **Document Status:** Official evidence-backed GUI, Pivot Tables, DialogKit, and Distribution Audit Protocol.  
> **Methodology:** Clean-Room Reverse Engineering & Verification with [Craft Anything (`storytold/craft-anything`)](https://github.com/storytold/craft-anything) MCP.  
> **Reference Standards:** IBM SPSS Statistics Desktop Interface Guide, ArtCraft Canonical Design System ([`AGENTS.md`](../AGENTS.md)), and [`docs/DESIGN_GUIDE.md`](DESIGN_GUIDE.md).  
> **Resource Guard:** Strictly compiled and tested under `-j 1`.

---

## 1. Audit Overview & Verification Scope

Phase 3 concludes the comprehensive audit of StatCraft, evaluating the user-facing presentation tier, interactive data manipulation surfaces, procedural dialogs, localization system, and native deployment packages across 13 specialized microplans (M-21 through M-33):

```text
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        PHASE 3 AUDIT: OUTPUT VIEWER, SCREENS, DIALOGKIT & PACKAGING                    │
├────────────────────────────┬─────────────────────────────┬────────────────────────┬────────────────────┤
│         SECTION E          │          SECTION F          │        SECTION G       │      SECTION H     │
│     Output Viewer Tables   │     Interactive Screens     │    DialogKit (SPSS)    │  i18n & Packaging  │
│  • M-21: Base Statistics   │  • M-25: Data View Grid     │  • M-28: Common Shell  │  • M-30: Fluent FTL│
│  • M-22: Compare Means     │  • M-26: Variable View      │  • M-29: 37 Dialogs    │  • M-31: Vector UI │
│  • M-23: Advanced / Nonpar │  • M-27: Syntax Editor      │    (Design Guard)      │  • M-32: Session   │
│  • M-24: Outline Tree      │                             │                        │  • M-33: Installer │
└────────────────────────────┴─────────────────────────────┴────────────────────────┴────────────────────┘
```

---

## 2. Section E: Output Viewer & Canonical Pivot Tables

### M-21: Base Output Statistics Tables & Vector Charts
* **Implementation:** `crates/statcraft_app/src/views/output_view/` (`tables_descriptives.rs`, `tables_frequencies/` (`freq_table.rs`, `stats_summary_table.rs`, `freq_chart_bar_pie.rs`, `freq_chart_histogram.rs`, `mod.rs`), `tables_crosstabs.rs`, `tables_means.rs`, `tables_common.rs`)
* **SPSS Reference:** *Base Statistics Output Tables Specification*
* **Craft Anything Hypothesis:** `hyp_33` (Supported)
* **Audited Invariants & Evidence:**
  1. **Descriptives Table:** Renders standard columns (`N`, `Minimum`, `Maximum`, `Mean`, `Std. Deviation`, `Variance`, `Skewness`, `Kurtosis`) with proper right-alignment for values and dynamic width scaling via `i18n Shield`. Listwise `Valid N` footnote is preserved.
  2. **Frequencies Table:** Implements the 4-column standard (`Frequency`, `Percent`, `Valid Percent`, `Cumulative Percent`) with dedicated rows for `Total Valid`, `Missing (System / User)`, and `Total`.
  3. **Crosstabulation Table:** Multi-tier nested layout displaying `Count`, `Expected Count`, `% within Row`, `% within Column`, `% of Total`, and `Residuals`. Integrated `Chi-Square Tests` sub-table (Pearson, Continuity Correction, Likelihood Ratio, Fisher's Exact, Linear-by-Linear).
  4. **P-Value Formatting:** Significance values below $0.001$ are formatted with leading-zero suppression as `< .001` per APA / SPSS conventions.
  5. **Vector-First Frequencies Charts:**
     - **Bar Charts (`BarChart`):** Automated Y-axis nice tick scaling (`calculate_nice_step`), subtle horizontal gridlines, interactive bar hover highlight, value labels (frequencies or percentages) above bars, and category labels along the X-axis.
     - **Pie Charts (`PieChart`):** Sector triangulation via `convex_polygon` starting at 12 o'clock, canonical 10-color `PIE_PALETTE`, and vector legend on the right with color swatches, category labels, and percentage shares.
     - **Histograms (`Histogram`):** Continuous X-axis with contiguous touching bins, theoretical Gaussian normal density curve overlay $\mathcal{N}(\mu, \sigma^2)$ in authentic SPSS orange (`#ED7D31`), and distribution statistics badge in the top-right corner (`Mean`, `Std. Dev.`, `N`).
     - **Rendering Standard:** 100% vector primitives via `ui.painter()`, zero raster images, zero emojis, and zero unsupported Unicode glyphs.
  6. **Table Suppression Notice (`table_suppressed`):** When categories exceed threshold `LIMIT(n)`, displays an authentic italic notification in `Tokens::text_muted()` (*"The frequency table for variable [NAME] is not displayed because the number of categories exceeds the specified limit."*), preserving the summary Statistics table and vector charts.
  7. **Multi-Variable Output Organization (`MultipleVarsMode`):** Supports `Compare` (single unified summary table, sequential frequency tables, separate charts node) and `Variable` (consecutive blocks per variable: personal statistics $\to$ frequency table $\to$ inline chart). Verified via `output_viewer_frequencies_invariants.rs` (8/8 passing).
  8. **Table Styling & Conditional Formatting Engine:** Implemented at reporting layer L4 (`statcraft_reporting::styling`). Provides declarative `TableStyleRule` mechanisms for cell styling based on numeric predicates (`All`, `Equal`, `NotEqual`, `LessThan`, `LessOrEqual`, `GreaterThan`, `GreaterOrEqual`, `Between`, `NotBetween`, `TopN`, `BottomN`, `SysMis`), dimension targeting (`Columns`, `Rows`, `Both`), label tokens (with substring false-positive protection), and HTML CSS export. Verified via `styling_tests.rs` (8/8 passing).
  9. **Output Viewer Styled Tables & Audit Report:**
     - Integration of `ReportTable` into `OutputItemKind` viewer hierarchy.
     - Styled cell rendering in `tables_common.rs` with custom background fill `bg_color` via `ui.painter().rect_filled`, text color `text_color`, and bold typography `bold`.
     - Automated auxiliary *«Conditional Styles Report»* audit table (`dialog-style-report-title`) with columns *Table*, *Target*, *Condition*, *Format*, *Modified Cells Count* when `create_style_report` is enabled.
     - Canonical SPSS `OUTPUT MODIFY` syntax generation upon Paste action (`/SELECT TABLES /IF SUBTYPES=['Frequencies']` and `/TABLECELLS SELECT=[DIMENSION=...] CONDITION=... STYLE=[...]`).
     - End-to-end styling retention in TSV and HTML export with inline CSS generation (`font-weight`, `color`, `background-color`).
     - Verified via end-to-end test suite `table_style_e2e_test.rs` (5/5 passing) and syntax generation test `frequencies_cmd/tests.rs`.

### M-22: Compare Means Output Tables
* **Implementation:** `tables_ttest_onesample.rs`, `tables_ttest_indep.rs`, `tables_ttest_paired.rs`, `tables_anova.rs`
* **SPSS Reference:** *Compare Means Output Tables Specification*
* **Craft Anything Hypothesis:** `hyp_34` (Supported)
* **Audited Invariants & Evidence:**
  1. **Independent Samples Test:** Two-tier header grouping *Levene's Test for Equality of Variances* ($F$, Sig.) and *t-test for Equality of Means* ($t$, $df$, Sig., Mean Diff, Std. Error Diff, 95% CI). Features the two mutually exclusive canonical rows: `Equal variances assumed` and `Equal variances not assumed`.
  2. **Paired Samples Test:** Renders the 3 distinct SPSS tables: *Paired Samples Statistics*, *Paired Samples Correlations*, and *Paired Samples Test* (Paired Differences).
  3. **One-Way ANOVA:** Renders *Descriptives*, *Test of Homogeneity of Variances* (Levene), and the 3-section *ANOVA* table (`Between Groups`, `Within Groups`, `Total` with $SS$, $df$, $MS$, $F$, Sig.).

### M-23: Advanced & Nonparametric Output Tables
* **Implementation:** `tables_correlations.rs`, `tables_regression.rs`, `tables_factor.rs`, `tables_reliability.rs`, `tables_kruskal.rs`, `tables_nonparam.rs`, `tables_nonparam_related.rs`, `tables_adp.rs`, `tables_programmability.rs`
* **SPSS Reference:** *Advanced Statistics Output Tables Specification*
* **Craft Anything Hypothesis:** `hyp_35` (Supported)
* **Audited Invariants & Evidence:**
  1. **Bivariate Correlations:** Outputs correlation coefficient, 1-tailed/2-tailed significance, sample size $N$, and significance flag asterisks (`*` $p < .05$, `**` $p < .01$) with footnotes.
  2. **Linear Regression:** 4 standard tables: *Variables Entered/Removed*, *Model Summary* ($R, R^2, \text{Adj } R^2, SE$), *ANOVA*, and *Coefficients* ($B$, Std. Error, $\beta$, $t$, Sig., 95% CI, Tolerance, VIF).
  3. **Factor Analysis:** *KMO and Bartlett's Test*, *Communalities* (Initial, Extraction), *Total Variance Explained*, and *Component Matrix*.
  4. **Reliability Analysis:** *Case Processing Summary*, *Reliability Statistics* (Cronbach's $\alpha$), *Item Statistics*, and *Item-Total Statistics*.
  5. **Nonparametric Suite:** Ranks and Test Statistics tables for Mann-Whitney $U$, Wilcoxon Signed Ranks ($W$), and Kruskal-Wallis ($H$).

### M-24: Outline Tree Navigation & Output Export
* **Implementation:** `outline.rs`, `blocks.rs`, `builder.rs`, `export.rs`, `model.rs`, `widgets/output_icons.rs`
* **SPSS Reference:** *SPSS Viewer DOM & Export Framework*
* **Craft Anything Hypothesis:** `hyp_36` (Supported)
* **Audited Invariants & Evidence:**
  1. **Outline Hierarchy:** Document Tree structure: `Output Root` $\to$ `Procedure Node` $\to$ `Log (.sps syntax)` $\to$ `Notes` $\to$ `Pivot Tables` $\to$ `FrequencyCharts` (vector icon `paint_output_chart_icon`: 3 columns on a baseline).
  2. **Interactive Navigation:** Clicking any tree item smooth-scrolls the canvas to target table or chart.
  3. **Context Menu Actions:** Right-click support on individual nodes: Copy as TSV, Copy as Text, and Delete.
  4. **Export Engine:** Full document or individual table/chart export into TSV, styled HTML, and Plain Text via `statcraft_reporting`. Supports tabular chart distribution export to TSV (`frequency_charts_to_tsv`).

---

## 3. Section F: Interactive Data Screens

### M-25: Data View (Virtualized Grid)
* **Implementation:** `crates/statcraft_app/src/views/data_view.rs`
* **SPSS Reference:** *SPSS Data Editor — Data View*
* **Craft Anything Hypothesis:** `hyp_37` (Supported)
* **Audited Invariants & Evidence:**
  1. **Grid Virtualization:** High-performance `ScrollArea::vertical().show_rows(...)` rendering only visible rows, guaranteeing smooth 60 FPS scrolling on large datasets.
  2. **Inline Cell Editing:** Double-click cell activation, keyboard navigation (Tab, Enter, Arrows), and atomic undo-history generation.
  3. **Value Labels Toggle:** Toolbar button (`1/A`) toggling between raw codes and descriptive value labels.
  4. **Filter Strikethrough:** Canonical diagonal strikethrough slash drawn across row header numbers for filtered (inactive) cases when `FILTER BY` is active.

### M-26: Variable View (Metadata Dictionary Editor)
* **Implementation:** `crates/statcraft_app/src/views/variable_view.rs`
* **SPSS Reference:** *SPSS Data Editor — Variable View*
* **Craft Anything Hypothesis:** `hyp_38` (Supported)
* **Audited Invariants & Evidence:**
  1. **11 Metadata Columns:** Name, Type, Width, Decimals, Label, Values, Missing, Columns, Align, Measure, and Role.
  2. **SPSS Name Validation:** Enforces naming restrictions (up to 64 chars, cannot begin with digit/period, no whitespace/symbols, forbids reserved words `ALL`, `AND`, `BY`, `EQ`, etc.).
  3. **Modal Triggers:** Ellipsis buttons (`...`) invoking dedicated modal dialogs (`ValueLabelsDialog`, `MissingValuesDialog`, `VariableTypeDialog`).

### M-27: Syntax Editor
* **Implementation:** `crates/statcraft_app/src/views/syntax_editor.rs`
* **SPSS Reference:** *SPSS Syntax Editor & Batch Processor*
* **Craft Anything Hypothesis:** `hyp_39` (Supported)
* **Audited Invariants & Evidence:**
  1. **Monospace Code Surface:** Dedicated text area with automatic session header and timestamps.
  2. **Command Execution:** Integration with `CommandBus` for executing whole scripts or selections, logging syntax echos to Output Viewer.
  3. **Paste Pipeline:** Every procedural dialog features a `[ Paste ]` button generating canonical `.sps` commands into the syntax editor.

---

## 4. Section G: DialogKit Architecture & 37 Modal Dialogs

### M-28: DialogKit Canonical Primitives
* **Implementation:** `crates/statcraft_app/src/dialogs/common/` (`dialog_shell.rs`, `action_bar.rs`, `subdialog_action_bar.rs`, `subdialog_sidebar.rs`, `variable_picker.rs`, `transfer_button.rs`, `group_box.rs`)
* **SPSS Reference:** *SPSS Canonical Dialog Architecture & Ergonomics*
* **Craft Anything Hypothesis:** `hyp_40` (Supported)
* **Audited Invariants & Evidence:**
  1. **Fixed Dialog Sizes (`DialogSize`):** Standard ($560 \times 430$), Subdialog ($360 \times 300$), Properties ($460 \times 380$), Compute ($620 \times 480$), Alert ($440 \times 170$).
  2. **Type 1 Action Bar (`DialogActionBar`):** Canonical sequence `[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`.
  3. **i18n Shield:** Uniform button width auto-calculated across localized labels ($\ge 80 \times 26$ px, 3px rounding).
  4. **Precondition Guard:** `[ OK ]` button automatically disables when procedural inputs are invalid.
  5. **VariablePicker & TransferButton:** Vector measure icons (Scale, Ordinal, Nominal) and adaptive transfer buttons (`->` / `<-`).

### M-29: Procedural Dialogs Suite (37 Dialogs, Subdialogs & Table Style)
* **Implementation:** `crates/statcraft_app/src/dialogs/*.rs` (including `frequencies_dialog/format_subdialog.rs`, `charts_subdialog.rs`, `statistics_subdialog.rs`, `table_style_dialog/` (`mod.rs`, `state.rs`, `condition_subdialog.rs`, `format_subdialog.rs`), coordinator `app_dialogs.rs` and decomposed module `app_dialogs_stats.rs`)
* **SPSS Reference:** *SPSS Procedural Dialogs Suite*
* **Craft Anything Hypothesis:** `hyp_41` (Supported)
* **Audited Invariants & Evidence:**
  1. **Design Guard Linter (`cargo run -p xtask -j 1 -- design-guard`):** 0 violations. Zero raw button calls, 100% DialogKit compliance.
  2. **Subdialog Canonical Architecture:** Type 2 modal subdialogs (including *Frequencies: Statistics*, *Frequencies: Charts*, and *Frequencies: Format*, $360 \times 300$ px) use `DialogShell::subdialog`, `GroupBox` and `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`) with rollback on cancel.
     - *Frequencies: Format*: two-column selection of category ordering (`CategoryOrder` — 4 radio options), multi-variable organization (`MultipleVarsMode` — 2 radio options), checkbox and dependent threshold input for category table suppression `LIMIT(n)`;
     - *Frequencies: Charts*: mutual exclusivity rules (Histogram vs Bar/Pie, normal curve gating).
  3. **Table Style & Conditional Formatting Dialog Suite:**
     - Main rules dialog `TableStyleDialog` ($640 \times 420$ px): 5-column interactive rules grid (Enabled, Scope, Condition, Format, Delete), action buttons for rule addition, duplication, reordering (Up / Down), summary report generation toggle, and rule clearing.
     - Condition subdialog `TableStyleConditionDialog` ($400 \times 340$ px): dimension targeting (`Columns`, `Rows`, `Both`), 11 numeric comparison operators with single/interval inputs.
     - Format subdialog `TableStyleFormatDialog` ($420 \times 420$ px): typography (Bold, Italic), token palette swatches for text and background colors, interactive live preview sample box `SamplePreviewBox`.
     - Transactionality: complete transactional snapshot mechanisms (`backup` / `cancel` / `confirm`) guaranteeing rollbacks on cancel across all subdialogs and the main dialog.
     - Integration in `FrequenciesDialog`: activated `[ Style... ]` (`dialog-btn-style`) sidebar button, rule JSON serialization, and pipeline action dispatch.
  4. **Coordinator Architectural Decomposition (Rules 12.1 & 12.2 Anti-Goodhart):**
     - Upon integrating `table_styles`, `crates/statcraft_app/src/app_dialogs.rs` reached 994 lines (approaching the 1000-line ceiling).
     - In strict adherence to Rule 12.2 ("Decompose, Never Cut"), an architectural decomposition was conducted: the canonical SPSS Descriptive Statistics group (*Descriptives, Frequencies, Explore, Crosstabs*) was extracted into `crates/statcraft_app/src/app_dialogs_stats.rs` (284 lines).
     - `app_dialogs.rs` was reduced to **723 lines** (277 lines headroom) with 100% feature and logic retention, fully satisfying `xtask file-guard`.
  5. **Reset Idempotence:** Clicking `[ Reset ]` restores all parameters and lists to exact defaults across all dialogs.
  6. **Automated Suite:** Verified via `cargo test -p statcraft_app --test dialog_invariants -j 1` (**49/49 passing**, including 6 tests in `table_style_invariants.rs`: rendering, rule manipulation, cancellation rollback, condition subdialog, format subdialog, frequencies integration).

---

## 5. Section H: Localization, Vector UI, Session & Packaging

### M-30: Fluent Localization (Zero-Hardcode i18n)
* **Implementation:** `locales/en.ftl`, `locales/ru.ftl`, `crates/statcraft_i18n/`
* **ArtCraft Standard:** *Zero-Hardcode i18n Regulation*
* **Craft Anything Hypothesis:** `hyp_42` (Supported)
* **Audited Invariants & Evidence:**
  1. **100% Key Parity:** Full synchronization between English and Russian catalogs with zero missing or empty keys.
  2. **Zero Raw UI Strings:** Strict linter check (`i18n_lint_test.rs`) confirms zero natural language literals in UI crates.
  3. **On-the-Fly Switching:** Language dynamically toggles at runtime without restarting the application.

### M-31: Vector-First UI & Tofu Glyph Prohibition
* **Implementation:** `scale_icon.rs`, `vector_arrow.rs`, `output_icons.rs`, `theme.rs`
* **ArtCraft Standard:** *Vector-First UI Regulation*
* **Craft Anything Hypothesis:** `hyp_43` (Supported)
* **Audited Invariants & Evidence:**
  1. **Vector Primitives:** All icons (Scale ruler, Ordinal bars, Nominal circles, arrows, folders, tables) drawn via `ui.painter()`.
  2. **Zero Emoji / Tofu Prohibition:** 100% absence of Unicode emoji (`📊`, `📈`, `⚙`, `⚠️`, `💡`) and non-standard symbols (`●`, `•`, `✓`, `✔`, `✕`, `✖`), eliminating tofu squares (`□`) across all operating systems.

### M-32: Session Management & Data Protection (Session Guard)
* **Implementation:** `app_session.rs`, `recent_files.rs`, `confirm_save_dialog.rs`
* **ArtCraft Standard:** *Data Safety & Session Invariants*
* **Craft Anything Hypothesis:** `hyp_44` (Supported)
* **Audited Invariants & Evidence:**
  1. **Dirty Flag Tracking:** Modifications trigger the unsaved indicator (`*`) in the window title.
  2. **ConfirmSaveDialog Guard:** Prevents data loss during file open, new project, or application shutdown.
  3. **Recent Files Queue:** Safe MRU tracking with file validation. Verified by `session_lifecycle_tests.rs` (12/12 passing).

### M-33: Native Packaging & CI Distribution
* **Implementation:** `packaging/windows/installer.iss`, `packaging/macos/Info.plist`, `packaging/linux/control`, `.github/workflows/release.yml`, `Cargo.toml`
* **ArtCraft Standard:** *Native Desktop Packaging Canon*
* **Craft Anything Hypothesis:** `hyp_45` (Supported)
* **Audited Invariants & Evidence:**
  1. **Version Synchronization:** Version `0.2.1` is synchronized byte-for-byte across all 4 manifests.
  2. **File Associations:** Native shell handlers configured for `.sav`, `.zsav`, `.stc`, and `.sps`.
  3. **Cross-Platform CI:** Automated matrix builds producing Windows installer (Inno Setup) + portable ZIP, Linux `.deb` + tarball, and macOS DMG + `.app` bundle.

---

## 6. Audit Verdict & DoD Sign-Off

* **Total Microplans Audited:** 13 (M-21 through M-33)
* **Craft Anything Hypotheses Confirmed:** 13/13 (100% Supported)
* **Automated Invariant Tests:**
  - `dialog_invariants`: 49/49 PASSED (including 6 tests for the Table Style DialogKit suite)
  - `output_viewer_invariants`: 11/11 PASSED
  - `output_viewer_frequencies_invariants`: 8/8 PASSED
  - `table_styling_tests`: 8/8 PASSED (in `statcraft_reporting`)
  - `table_style_e2e_tests`: 5/5 PASSED (in `statcraft_app`, end-to-end rendering, audit table, OUTPUT MODIFY, and HTML)
  - `session_lifecycle_tests`: 12/12 PASSED
  - `i18n_lint_test`: 18/18 PASSED (in `statcraft_i18n`)
  - `xtask design-guard`: 0 violations PASSED
  - `xtask file-guard`: 508 files PASSED (including verification of `app_dialogs_stats.rs` decomposition)
  - `xtask layers`: PASSED
  - `xtask parity`: PASSED
  - `xtask sync-site --check`: PASSED
  - `cargo clippy -p statcraft_app -j 1 -- -D warnings`: 0 warnings PASSED
  - `cargo fmt --check`: PASSED
* **Overall Phase 3 Verdict:** **FULL SPSS CANONICAL COMPLIANCE ACHIEVED**.

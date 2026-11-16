# Changelog

All notable changes to the **StatCraft** statistical computing application will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

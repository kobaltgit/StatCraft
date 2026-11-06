# StatCraft Feature Guide (v0.2.2)

This document serves as an exhaustive reference of all implemented capabilities, analytical procedures, mathematical algorithms, and user interface features available in the **StatCraft** statistical package.

---

## 1. Dual-View Data Management & Tabular Editors

StatCraft provides a canonical SPSS-style workspace with instantaneous switching between the observational data grid (**Data View**) and the metadata dictionary (**Variable View**).

### 1.1. Data Import & Creation
* **Streaming CSV / TSV Ingest:** High-performance streaming reader with automatic column type inference, configurable field delimiters (comma, semicolon, tab), and decimal point formatting.
* **Microsoft Excel Import (`.xlsx`, `.xls`, `.ods`):** Direct workbook reader powered by the open-source Calamine engine, providing automated header sanitization to SPSS naming rules and column type inference.
* **Blank Dataset Mode (`Blank Dataset`):** Instant instantiation of an empty observation matrix (40 cases by default) for manual tabular data entry without opening external files.

### 1.2. Data View
* **Virtualized Tabular Grid:** Fluid scrolling across tens of thousands of rows and hundreds of columns powered by `ScrollArea::show_rows`.
* **In-Place Cell Editing:** Direct cell input with `Enter` (navigate down) and `Tab` (navigate right) keys, with automatic delimiter and currency normalization.
* **Auto-Expanding Rows:** Automatic generation of new blank observations upon entering values into the bottom-most row.
* **Insert & Delete Cases:** Row context menus and keyboard shortcuts for inserting (`Insert Case`) and deleting (`Delete Case`) observations.
* **Value Labels Toggle (`1/A`):** Toolbar button for instant toggling between raw numeric codes and human-readable descriptive text labels across the entire grid.
* **Filter Exclusion Masking:** Diagonal vector strike-through over row number headers for cases excluded by an active sample filter.

### 1.3. Variable View
Comprehensive variable metadata dictionary compliant with canonical IBM SPSS Statistics standards:
* **Name (`Name`):** Strict identifier validation according to SPSS rules (letter start, prohibition of whitespace and special characters).
* **Type (`Type`):** Full support for all 9 canonical variable types:
  1. *Numeric* (Standard floating-point / integer numbers);
  2. *Comma* (Numbers with comma-separated thousands and decimal period);
  3. *Dot* (Numbers with dot-separated thousands and decimal comma);
  4. *Scientific* (Exponential notation `1.23E+02`);
  5. *Date* (Calendar dates formatted as `dd.mm.yyyy`, `yyyy-mm-dd`, etc.);
  6. *Dollar* (Currency format with `\$` prefix);
  7. *Custom Currency* (Configurable user currency formats);
  8. *String* (Arbitrary text character variables);
  9. *Restricted Numeric* (Integer values with fixed leading zeros).
* **Width & Decimals (`Width` & `Decimals`):** Dedicated vector spinners `▲ / ▼` for instant adjustment of column display width and decimal precision.
* **Variable Label (`Label`):** Descriptive titles displayed in dialog pickers and analytical pivot table headers.
* **Value Labels (`Values`):** Modal dialog mapping discrete numeric codes to descriptive category labels (e.g., `1 = "Male"`, `2 = "Female"`).
* **Missing Values (`Missing`):** Configurable discrete user-missing values (up to 3 distinct codes) and continuous missing ranges (e.g., `90 through 99`).
* **Alignment (`Align`):** Text and numeric cell alignment (*Left / Center / Right*).
* **Measurement Scale (`Measure`):** Nominal (*Nominal*), Ordinal (*Ordinal*), and Scale (*Scale*) measurement classifications with authentic vector icons.

---

## 2. Dataset Management & Case Manipulations (Data Menu)

* **Sort Cases (`Sort Cases`):**
  * Multi-column stable sorting across an unlimited number of sorting keys.
  * Independent sorting direction per key: Ascending or Descending.
  * Strict adherence to SPSS sorting standards: System-missing (`SYSMIS`, `NaN`) values sort first in ascending order and last in descending order.
* **Select Cases (`Select Cases`):**
  * Case filtering based on conditional expressions via an integrated modal condition calculator.
  * Non-destructive filtering mode creating an active binary `filter_$` variable while preserving excluded cases.
  * Permanent deletion mode for cases not satisfying the filtering condition.
  * Status bar vector indicator: `Filter: ON (var)`.
* **Split File (`Split File`):**
  * Stratified analysis segmented across categorical subgroups (supporting multiple grouping variables).
  * Modes: *Compare groups* (side-by-side grouped output tables) and *Organize output by groups* (separate sequential output blocks).
  * Status bar vector indicator: `Split File: ON (var)`.
* **Weight Cases (`Weight Cases`):**
  * Frequency and sampling case weighting using values from a designated numeric variable.
  * Automatic propagation of fractional and integer weights across all descriptive, frequency, and parametric procedures.
  * Status bar vector indicator: `Weight: ON (var)`.

---

## 3. Data Transformations (Transform Menu)

All data transformations are powered by standardized `DialogKit` interfaces, syntax generators, and non-destructive Undo support (`Ctrl+Z`):

* **Compute Variable (`Compute Variable`):**
  * Computation of new target variables or replacement of existing columns using mathematical expressions.
  * Built-in Shunting-yard expression parser (Reverse Polish Notation / RPN).
  * Support for addition, subtraction, multiplication, division, exponentiation (`**`, `^`), and unary operators.
  * Standard function library: Arithmetic (`ABS`, `EXP`, `LN`, `LOG10`, `RND`, `SQRT`, `TRUNC`) and Statistical (`SUM`, `MEAN`, `MIN`, `MAX`).
* **Recode into Same Variables (`Recode into Same Variables`):**
  * In-place value reassignment for selected variables without creating new columns.
* **Recode into Different Variables (`Recode into Different Variables`):**
  * Generation of new transformed variables based on conditional recoding rules.
  * *Old and New Values* subdialog: exact values, continuous ranges (*Range*), open-ended ranges (*Range Lowest through...* / *Range... through Highest*), system- and user-missing values, and fallback rules (*All other values*).
* **Automatic Recode (`Automatic Recode`):**
  * Conversion of string or non-contiguous numeric codes into consecutive integers $1, 2, \dots, K$.
  * Automatic generation of descriptive value labels preserving source categories.
  * Direction selection: *Lowest value* or *Highest value*.
* **Shift Values (`Shift Values`):**
  * Creation of lagged (*Lag*) and lead (*Lead*) time series columns shifted by $K$ intervals.
* **Count Values within Cases (`Count Values within Cases`):**
  * Row-wise counting of occurrences matching specific values or ranges across a list of variables.
* **Create Dummy Variables (`Create Dummy Variables` / One-Hot Encoding):**
  * Binary (0/1) indicator generation for categorical scales.
  * Configurable root names, prefixes, and reference category omission ($k-1$ indicators for linear models).
* **Random Number Seed (`Set Random Number Seed`):**
  * Thread-safe global pseudo-random number generator initialization for reproducible sampling and simulations.
* **Replace Missing Values (`Replace Missing Values`):**
  * Imputation algorithms: Series Mean, Mean of nearby points, Median of nearby points, Linear interpolation, and Linear trend at point.
* **Create Time Series (`Create Time Series`):**
  * First-order differences (`DIFF`), seasonal differences (`SDIFF`), simple moving averages (`SMA`), and cumulative sums (`CSUM`).
* **Visual Binning (`Visual Binning`):**
  * Continuous variable discretization: Equal Width intervals, Equal Percentiles (quantiles), and Standard Deviation cutpoints.
  * Automatic generation of descriptive interval labels (e.g., `< 20`, `20 - 40`, `> 40`).
* **Optimal Binning (`Optimal Binning`):**
  * Supervised discretization of continuous predictors against a categorical target using the Fayyad & Irani MDLP entropy algorithm.
* **Rank Cases (`Rank Cases`):**
  * Rank assignment with tie handling: Mean (mid-ranks), Low, High, and Sequential.
  * Rank types: Simple ranks, fractional ranks, percentages, Ntiles, and normal scores (Blom, Tukey, Rankit, Van der Waerden) with Acklam approximation ($\le 1.15 \times 10^{-9}$).
* **Date and Time Wizard (`Date and Time Wizard`):**
  * Date component extraction: Year, Month, Day, Hour, Minute, Second, Day of week, Quarter.
  * Calculation of duration between two dates (`DATEDIFF`) in days, months, or years.
  * Addition or subtraction of time intervals (`DATESUM`).
* **Automated Data Preparation (`Automated Data Preparation / ADP`):**
  * Comprehensive pipeline: predictor screening, outlier handling (winsorizing or set to sysmis), feature scaling (Z-score, Min-Max), sparse category merging, pairwise interactions, and validation partitioning.
* **Programmability Transformations (`Programmability Transformations`):**
  * Multi-line transformation script editor.
  * Conditional `IF ... THEN ... ELSE` branching, vector assignments, and real-time syntax validation generating `BEGIN PROGRAM ... END PROGRAM.` blocks.

---

## 4. Statistical Procedures (Analyze Menu)

All statistical algorithms in StatCraft are implemented in **100% pure Safe Rust** and validated against Python `SciPy` and canonical *IBM SPSS Statistics 24 Algorithms Guide* standards to an absolute tolerance threshold of $\le 10^{-6}$.

### 4.1. Descriptive Statistics & Cross-Tabulations
* **Descriptives (`Descriptives`):**
  * Spicer (1972) recurrence moments: Valid $N$, Sum, Mean, Std. Error of Mean, Variance, Std. Deviation, Range, Minimum, Maximum, Skewness (with SE), and Bliss sample excess Kurtosis (with SE).
* **Frequencies (`Frequencies`):**
  * Authentic SPSS frequency tables: Counts, Percent, Valid Percent (excluding missings), Cumulative Percent.
  * Quartiles ($Q_1, Q_2, Q_3$) and custom percentiles computed via the canonical HAVERAGE interpolation standard ($r = p(N+1)$).
* **Crosstabs (`Crosstabs`):**
  * Two-way $R \times C$ contingency tables.
  * Observed counts, expected counts, unstandardized residuals, row percentages, column percentages, and total percentages.
  * Pearson Chi-Square ($\chi^2$), Likelihood Ratio ($G^2$).
  * Yates continuity correction and two-tailed Fisher's exact test for $2 \times 2$ tables.
  * Symmetric association measures: Phi ($\phi$) and Cramér's $V$.

### 4.2. Mean Comparisons & Analysis of Variance (Compare Means)
* **Means Breakdown (`Means Breakdown`):**
  * Subgroup descriptive breakdown (Mean, $N$, Variance, Median, Min, Max) across categorical factors.
  * One-way ANOVA table (Between Groups, Within Groups, Total Sum of Squares, $df$, Mean Squares, $F$-statistic, significance).
  * Association measures: $\eta$ (Eta) and $\eta^2$ (Eta Squared).
* **One-Sample T-Test (`One-Sample T Test`):**
  * Student's $t$-test comparing sample mean to a specified reference test value.
  * Degrees of freedom, two-tailed $p$-value via regularized incomplete beta function, mean difference, and 95% (or custom) confidence interval of the difference.
* **Independent-Samples T-Test (`Independent-Samples T Test`):**
  * Two-group comparison with *Define Groups* subdialog.
  * Automatic Levene's Test for Equality of Variances ($F$, Sig).
  * Dual-row output table: Student's $t$-test (Equal variances assumed) and Welch's $t$-test (Equal variances not assumed with Satterthwaite fractional degrees of freedom).
* **Paired-Samples T-Test (`Paired-Samples T Test`):**
  * Comparison of matched pairs and repeated measures.
  * Paired statistics, intra-pair Pearson correlation ($r$, Sig), and paired differences test ($t$, $df$, confidence interval).
* **One-Way ANOVA (`One-Way ANOVA`):**
  * Fisher's $F$-test comparing multiple treatment groups.
  * Between-groups and within-groups sums of squares ($SS$) and mean squares ($MS$).
  * Levene's Test of Homogeneity of Variances.
  * Group descriptive statistics with 95% confidence intervals.

### 4.3. Bivariate Correlations (Correlate)
* **Bivariate Correlations (`Bivariate Correlations`):**
  * Pearson linear correlation coefficient ($r$).
  * Spearman rank-order correlation coefficient ($\rho$).
  * Kendall rank-order correlation coefficient ($\tau_b$) with full tie adjustments ($v_0, v_1, v_2$).
  * One-tailed and two-tailed significance levels.
  * Automatic significance flagging (`*` for $p < 0.05$, `**` for $p < 0.01$).
  * Cross-product deviations and covariances, pairwise and listwise missing exclusion modes.

### 4.4. Nonparametric Tests (Nonparametric Tests)
* **Two Independent Samples (`Two Independent Samples`):**
  * Mann-Whitney $U$ test, Wilcoxon rank sum $W$.
  * Pooled mid-ranking with tie adjustment $\sum(t_i^3 - t_i)$.
  * Calculation of expected values, adjusted variances, $Z$-score, and asymptotic two-tailed significance.
* **K Independent Samples (`K Independent Samples`):**
  * Kruskal-Wallis $H$ test with tie factor correction $H_{adj}$.
  * Asymptotic significance computed from the $\chi^2$ distribution with $df = k - 1$.
* **Two Related Samples (`Two Related Samples`):**
  * Wilcoxon Signed-Rank Test with tie-adjusted variance and continuity correction.
  * Sign Test with exact two-tailed binomial significance via the beta distribution.

### 4.5. Linear Regression (Regression)
* **Multiple Linear Regression (`Linear Regression`):**
  * Ordinary Least Squares (OLS) and Weighted Least Squares (WLS) solvers.
  * *Model Summary* table: Multiple correlation $R$, $R^2$, Adjusted $R^2$, Std. Error of the Estimate.
  * *ANOVA* table: Regression and residual sums of squares, mean squares, Fisher's $F$, significance.
  * *Coefficients* table: Unstandardized coefficients $B$ and standard errors, standardized coefficients $\beta$, $t$-statistics, significance levels, and 95% confidence intervals.
  * Collinearity diagnostics: Tolerance and Variance Inflation Factor (VIF).

### 4.6. Scale Reliability Analysis (Scale)
* **Reliability Analysis (`Reliability Analysis`):**
  * Cronbach's Alpha ($\alpha$) and standardized item alpha.
  * *Item Statistics*: Mean, standard deviation, and valid $N$ per item.
  * *Item-Total Statistics*: Scale mean if item deleted, scale variance if item deleted, corrected item-total correlation, Cronbach's alpha if item deleted.
  * Overall summary *Scale Statistics*.

### 4.7. Dimension Reduction (Dimension Reduction)
* **Factor Analysis & Principal Component Analysis (`Factor Analysis / PCA`):**
  * Principal Component Analysis via cyclical Jacobi eigensolver for symmetric correlation matrices.
  * Kaiser-Meyer-Olkin (KMO) Measure of Sampling Adequacy.
  * Bartlett's Test of Sphericity with exact $\chi^2$ and significance.
  * *Communalities* table: Initial and Extraction values.
  * *Total Variance Explained* table: Eigenvalues, % of variance, and cumulative variance.
  * Component Matrix showing unrotated factor loadings.

---

## 5. Output Management & Clipboard Integration (Output Viewer)

* **Interactive Outline Navigation:**
  * Tree panel on the left reflecting document structure across procedures, syntax logs, and tables.
  * Smooth scrolling and focus ring highlighting upon selecting any item in the tree.
  * Synchronized two-way selection: clicking a table in the scroll canvas highlights its node in the outline.
  * Outline toolstrip: collapse all, expand all, clear entire output, and delete selected nodes with the `Delete` key.
* **Authentic Pivot Tables:**
  * High-fidelity reproduction of IBM SPSS Statistics typography and layouts: nested multi-tier headers, alternating row striping, right-aligned numeric formatting, and criterion footnotes.
* **One-Click Excel TSV Export:**
  * Dedicated `[ Copy Table (TSV) ]` button in each table header.
  * Structured Tab-Separated Values (TSV) format engineered for Microsoft Excel and Google Sheets: pastes cleanly with zero column shifting, date corruption, or text truncation.
* **Syntax Execution Protocols (Log):**
  * Automatic monospace log blocks documenting executed SPSS syntax (`GET DATA`, `DESCRIPTIVES`, `CROSSTABS`, `REGRESSION`, etc.) with timestamps.

---

## 6. SPSS Command Syntax (`.sps`) & Command Bus

* **Typed Command Bus (`CommandBus`):**
  * Every user action is encapsulated as a typed `EngineCommand` providing parameter schemas for external agents (`specs()`).
* **Canonical `[ Paste ]` Action:**
  * Clicking `[ Paste ]` in any modal dialog translates configured parameters into valid SPSS command syntax and directs it into the Syntax Editor or Log.
* **Syntax Editor:**
  * Dedicated code editing tab with monospace styling, line execution, and batch processing.
* **Non-Destructive History (`Undo / Redo`):**
  * In-memory `DeltaBuffer` transaction engine enabling instant undo/redo of data edits and metadata modifications via `Ctrl+Z` / `Ctrl+Y`.

---

## 7. AI Readiness (Agent-Ready) & MCP Bridge Status

* **Parameter Schemas for Agents (`specs()`):**
  * Every statistical and data engine procedure (`statcraft_syntax`) provides strongly-typed JSON parameter schemas via `specs()`, allowing external AI agents to validate arguments and construct pipelines headlessly.
* **Batch Syntax Scripting:**
  * AI assistants can assemble canonical SPSS command syntax (`.sps`) and run it through the engine command bus, yielding complete execution logs and authentic Output Viewer tables.
* **Live Network Bridge Status (MCP & IPC Socket):**
  * *In Development (`InDev` / Roadmap):* Direct runtime communication over a local IPC socket / embedded MCP server to control the running desktop GUI is currently in active development (the menu entry `Extensions -> MCP Bridge / Copilot` is flagged as `[dev]`). Live socket connectivity will arrive in subsequent roadmap phases.

---

## 8. Design System, Reliability, and Localization

* **Never-Crash Invariant (100% Safe Rust):**
  * Complete absence of `unsafe`, `panic!`, `unwrap()`, and `expect()` in non-test code. Malformed data, singular matrices, or division by zero yield typed errors without terminating the application.
* **Vector-First User Interface:**
  * 100% vector rendering using `ui.painter()` and hardware-rasterized SVGs. Zero Unicode emojis or unsupported glyphs, completely eliminating "tofu" boxes (`□`) across all platforms.
* **Zero-Hardcode Localization:**
  * 100% of UI strings are externalized into Project Fluent catalogs (`locales/en.ftl`, `locales/ru.ftl`).
  * On-the-fly language switching (English $\leftrightarrow$ Russian) without application restart.
* **Multi-Platform Support:**
  * Single codebase with verified native binaries for Windows (x64), Linux (x64), and macOS (Apple Silicon ARM64 & Intel x64).

---

## 9. Roadmap & Features in Development (InDev / Planned)

To guarantee complete technical honesty and transparency, the following modules and menu actions are currently under development in v0.2.2 (`MenuItemStatus::InDev`):

* **Vector Charting (`statcraft_charts`):**
  * Interactive Chart Builder, histograms, scatter plots, line charts, boxplots, pie charts, and tabular brushing & linking.
* **Runtime AI Network Bridge (`statcraft_bridge`):**
  * Local IPC socket and native Model Context Protocol (MCP) server for live bidirectional communication with external LLMs and IDE assistants.
* **Report Exporters:**
  * Output Viewer export to PDF and LaTeX (v0.2.2 supports one-click TSV for Excel, HTML, and Plain Text).
* **Advanced Statistical Procedures (Analyze):**
  * General Linear Models (`GLM`: Univariate, Multivariate, Repeated Measures).
  * Logistic Regression (Binary, Multinomial) and Curve Estimation.
  * Cluster Analysis (*K-Means, Hierarchical Clustering*).
  * Survival Analysis (*Life Tables, Kaplan-Meier, Cox Regression*).
  * Exploratory Data Analysis (*Explore*) and Case Summaries / Codebook reports.
* **Dataset Operations (Data):**
  * File Merging (*Merge Files: Add Cases / Add Variables*).
  * Data Restructuring (*Restructure*) and Aggregation (*Aggregate*).
* **Clipboard & Search Tools (Edit):**
  * Global cell Find & Replace, Go to Case / Variable navigation, application Preferences.
* **Visual Theming (View):**
  * Light / Dark / System theme toggles and display Zoom controls.


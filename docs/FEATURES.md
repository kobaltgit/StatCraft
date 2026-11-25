# StatCraft Feature Guide (v0.4.2)

This document serves as an exhaustive reference of all implemented capabilities, analytical procedures, mathematical algorithms, and user interface features available in the **StatCraft** statistical package.

---

## 1. Dual-View Data Management & Tabular Editors

StatCraft provides a canonical SPSS-style workspace with instantaneous switching between the observational data grid (**Data View**) and the metadata dictionary (**Variable View**).

### 1.1. Data Import & Creation
* **Streaming CSV / TSV Ingest:** High-performance streaming reader with automatic column type inference, configurable field delimiters (comma, semicolon, tab), and decimal point formatting.
* **Microsoft Excel Import (`.xlsx`, `.xls`, `.ods`):** Direct workbook reader powered by the open-source Calamine engine, providing automated header sanitization to SPSS naming rules and column type inference.
* **Blank Dataset Mode (`Blank Dataset`):** Instant instantiation of an empty observation matrix (40 cases by default) for manual tabular data entry without opening external files.

### 1.2. Data View (Virtualized 4-Quadrant 2D Grid)
* **Spatial 4-Quadrant Layout:** Pinned corner (`quad_top_left`), pinned column headers (`quad_top_right`), pinned row numbers (`quad_bottom_left`), and 2D central cell grid (`quad_center`). Row numbers remain anchored during horizontal scrolling.
* **Prefix-Sum Indexing (`LineIndex`):** $O(\log N)$ binary search from screen pixels to logical cells with zero heap allocations in `Uniform` mode.
* **Interactive Column Resizing & Auto-Fit:** Visual drag resizing with guidelines; double-click on header separator calculates optimal Auto-fit width from column data and name; synchronized with SPSS dictionary metadata (`VariableMetadata.columns`).
* **Multi-Cell Range Selection:** Fluid marquee box selection via mouse dragging or Shift-click / Shift-drag with active cursor differentiation.
* **Full Keyboard Ergonomics:** Arrow navigation, Tab/Enter workflows, boundary jumps (`Home`/`End`, `Ctrl+Home`/`Ctrl+End`), Shift-expansion, and **Direct Typing** (typing alphanumeric keys immediately activates the inline editor).
* **Focused Inline Cell Editor:** In-place `TextEdit` with visual focus ring, auto-commit on Enter/Tab, and cancellation on Escape.
* **Rectangular TSV/CSV Clipboard:** Native clipboard copy/paste (`Ctrl+C` / `Ctrl+V`) interoperable with Excel, Sheets, and Calc with automatic row expansion.
* **Series Auto-Fill (Fill Handle):** Bottom-right corner marker computes linear arithmetic progressions for numeric series and repetition for categorical data.
* **Column AutoFilter & Dropdowns:** Header filter button with vector chevron/funnel opening a dropdown with live search, checkboxes, and "(Blanks)" handling.
* **Quick Sort via CommandBus:** Instant Ascending (A–Z) and Descending (Z–A) sorting via canonical `data.sort_cases` with 100% reversible Undo/Redo and SPSS syntax logging (`SORT CASES BY ...`).
* **Cell Context Menu:** Right-click menu for filtering by cell value, clearing filters, sorting, and clipboard operations.
* **SPSS FILTER BY Visualization Parity:** Inactive / filtered cases display diagonal strikethrough over row headers and muted text in cells.
* **Data Integrity Guards:** Automated detection of numbers stored as text (`is_number_stored_as_text`) with Excel-parity green corner triangle and 1-click conversion to numeric.
* **Value Labels Toggle (`1/A`):** Toolbar button for instant toggling between raw numeric codes and descriptive text labels across the entire grid.
* **Auto-Expanding Rows:** Automatic generation of new blank observations upon entering values into the bottom-most row.

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

### 2.1. Multi-File Workspaces & Concurrent Document Sessions
* **Multi-File Workspace (`WorkspaceManager`):**
  * Simultaneous handling of multiple datasets in a single window with isolated Undo/Redo delta buffers, active filters, split-file groupings, and selection states.
  * In-memory lossless compression: `NullBitmask`, `DictionaryEncodedColumn`, and `LosslessCompressedColumn` retaining full `f64` numerical fidelity.
  * Native SPSS syntax dataset commands: `DATASET NAME`, `DATASET ACTIVATE`, `DATASET CLOSE`, and `DATASET DISPLAY` with `WINDOW=ASIS` validation.
* **Vector Document Tab Strip (`DocumentTabStrip`):**
  * Horizontal tab bar with custom vector dataset icons, unsaved modification markers (`*`), close buttons (`✕`), smooth wheel scrolling, chevron overflow navigators, and quick dataset creation (`[ ➕ ]`).
  * Keyboard navigation: `Ctrl+Tab` / `Ctrl+Shift+Tab` (switch datasets), `Ctrl+W` (close active dataset).
  * **Precision Dirty Markers:** Clean text rendering with guaranteed right padding (32 px) and ellipsis truncation (`truncate_at_width`), ensuring unsaved asterisks (`*`) never collide with filename text.
  * **Normalized Titlebar:** Clean application window title displaying active filename and dirty status without invalid Unicode glyphs.
* **Dynamic Contrast Colors & Container Persistence (Spec-Kit 015):**
  * Collision-free random color generation via 64-bit SplitMix64 and golden angle distribution ($137.50777^\circ$, $\ge 45^\circ$ distance between tabs), preventing visual clustering for datasets with similar prefixes (`merge_base.csv`, `merge_add_cases.csv`, `merge_add_vars.csv`).
  * Cross-pane color identity: Active tab underlines, vector grid icons, Outline Tree procedure chips, and Notes summary tables query session colors via `workspace.session_color_by_name`.
  * Complete persistence in `.stc` project files (`pub tab_color: Option<String>`) with 100% backward compatibility.
* **Cross-Platform Memory Guard (`system_memory.rs`):**
  * Real-time physical RAM polling across Windows (`GlobalMemoryStatusEx`), Linux (`/proc/meminfo`), macOS (`sysctl`), and WebAssembly.
  * Preflight safety alerts proactively warning users before opening large files (>250 MB) or batch operations (>10 files) when available RAM is critically low (<500 MB or <15%).

### 2.2. Data Merging: Add Cases & Add Variables (Feature 012)
* **Add Cases (`ADD FILES`):**
  * Vertical observation concatenation from external files (`.sav`, `.zsav`, `.stc`, `.csv`, `.xlsx`) or active workspace tabs.
  * Automatic schema alignment, type reconciliation, `/IN=var` source indicator tracking, and optional interleaved case ordering via `/BY`.
* **Add Variables (`MATCH FILES`):**
  * Horizontal column merging across datasets: 1:1 key merges, 1:N lookup tables (`/TABLE`), and sequential merges without keys.
  * Automated resolution of duplicate column collisions with clean rename suggestions and presence indicator flags (`/IN`).
* **Canonical DialogKit Interface:**
  * Clean-room dialog windows (`AddCasesDialog`, `AddVarsDialog`) with dual-pane variable pickers, active workspace tab selectors, file browse dialogs, and instant SPSS syntax generation.

### 2.3. Case Filtering, Sorting & Weighting
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
  * Canonical SPSS DialogKit UI: `Save standardized values as variables` checkbox, `Style...` (`TableStyleState`) and `Bootstrap...` sidebar buttons, and the grouped `Descriptives: Options` subdialog (`Mean`, `Sum`, `Dispersion`, `Distribution`, and `Display Order`).
  * Standardized Z-scores (`/SAVE`, creating `Z<var>` or collision-safe `ZSC001+` with full `Undo` support), display sorting (`/SORT=MEAN|NAME (A)|(D)`), `MISSING=LISTWISE` with exact `listwise_n`, and full SPSS syntax round-trip (`SUM`, `ALL`, `SAVE`, `MISSING`, `SORT`).
  * Dynamic Output Viewer & Reporting (`TSV`, `HTML`, `TXT`): renders strictly the selected statistics (`N`, `Range`, `Minimum`, `Maximum`, `Sum`, `Mean`, `S.E. Mean`, `Std. Deviation`, `Variance`, `Skewness`, `SE Skewness`, `Kurtosis`, `SE Kurtosis`), `Valid N (listwise)` summary row, and conditional `TableStyleRule` formatting with optional style audit report.
  * Reusable `Bootstrap` subdialog (`Percentile` / `BCa`, `Simple` / `Stratified`, `MT19937` seed) with `Bias`, `Std. Error`, and confidence intervals in Output Viewer & TSV export.
* **Frequencies (`Frequencies`):**
  * Authentic SPSS frequency tables: Counts, Percent, Valid Percent (excluding missings), Cumulative Percent.
  * DialogKit modal subdialogs: Statistics, Charts, Format (ordering, variable grouping, suppression threshold with dependent validation), Style, and Bootstrap (`Percentile` / `BCa`, `Simple` / `Stratified`, `MT19937` seed).
  * Category sorting: ascending/descending values (`AVALUE`/`DVALUE`) or frequencies (`AFREQ`/`DFREQ`) with dynamic cumulative percent recalculation.
  * Table suppression threshold (`LIMIT(n)`): automatically suppresses categorical tables when category count exceeds threshold $n$, displaying an authentic italic notice in the Output Viewer while preserving statistics and charts.
  * Multi-variable output layout: `COMPARE` (single unified summary table across all variables, sequential frequency tables, dedicated outline Charts node) vs `VARIABLE` (autonomous consecutive blocks per variable: personal statistics, frequency table, inline chart).
  * Quartiles ($Q_1, Q_2, Q_3$) and custom percentiles computed via the canonical HAVERAGE interpolation standard ($r = p(N+1)$).
  * Vector-first distribution charts: Bar charts, Pie charts, and Histograms with optional Gaussian normal curve overlay.
* **Crosstabs (`Crosstabs`):**
  * Two-way $R \times C$ contingency tables.
  * Observed counts, expected counts, unstandardized residuals, row percentages, column percentages, and total percentages.
  * Pearson Chi-Square ($\chi^2$), Likelihood Ratio ($G^2$).
  * Yates continuity correction and two-tailed Fisher's exact test for $2 \times 2$ tables.
  * Symmetric association measures: Phi ($\phi$) and Cramér's $V$.

### 4.2. Mean Comparisons & Analysis of Variance (Compare Means)
*Note: All procedures below (`Means`, `One-Sample T Test`, `Independent-Samples T Test`, `Paired-Samples T Test`, and `One-Way ANOVA`) support the reusable `Bootstrap` subdialog (`Percentile` / `BCa`, `Simple` / `Stratified`, `MT19937` seed) with `Bootstrap Specifications` and CI/Bias/SE tables in Output Viewer & TSV export.*
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

### 4.3. Correlations (Correlate)
* **Bivariate Correlations (`Bivariate Correlations`):**
  * Pearson linear correlation coefficient ($r$).
  * Spearman rank-order correlation coefficient ($\rho$).
  * Kendall rank-order correlation coefficient ($\tau_b$) with full tie adjustments ($v_0, v_1, v_2$).
  * One-tailed and two-tailed significance levels.
  * Automatic significance flagging (`*` for $p < 0.05$, `**` for $p < 0.01$).
  * Cross-product deviations and covariances, pairwise and listwise missing exclusion modes.
  * Reusable `Bootstrap` subdialog (`Percentile` / `BCa`, `Simple` / `Stratified`, `MT19937` seed) for correlation coefficients.
* **Partial Correlations (`Partial Correlations` / `PARTIAL CORR`):**
  * Linear relationship evaluation between variables while controlling for one or more continuous covariates.
  * First-order and higher-order partial correlation matrix computed via matrix inversion of the sub-covariance matrix.
  * Zero-order correlation reference matrix, degrees of freedom ($df = N - k - 2$), and two-tailed significance levels.

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

### 4.5. Regression Analysis (Regression)
* **Multiple Linear Regression (`Linear Regression`):**
  * Ordinary Least Squares (OLS) and Weighted Least Squares (WLS) solvers.
  * *Model Summary* table: Multiple correlation $R$, $R^2$, Adjusted $R^2$, Std. Error of the Estimate.
  * *ANOVA* table: Regression and residual sums of squares, mean squares, Fisher's $F$, significance.
  * *Coefficients* table: Unstandardized coefficients $B$ and standard errors, standardized coefficients $\beta$, $t$-statistics, significance levels, and 95% confidence intervals.
  * Reusable `Bootstrap` subdialog (`Percentile` / `BCa`, `Simple` / `Stratified`, `MT19937` seed) for regression coefficients $B$.
  * Collinearity diagnostics: Tolerance and Variance Inflation Factor (VIF).
* **Binary Logistic Regression (`Binary Logistic Regression` / `LOGISTIC REGRESSION`):**
  * Maximum Likelihood Estimation (MLE) via Iteratively Reweighted Least Squares (IRLS).
  * Real-time binary cardinality guard: interactive detection of target distinct values with auto-lock on non-binary variables (protecting against execution crashes).
  * *Omnibus Tests of Model Coefficients* table ($\chi^2$, $df$, $p$-value).
  * *Model Summary* table: $-2 \text{ Log Likelihood}$, Cox & Snell $R^2$, and Nagelkerke $R^2$.
  * *Hosmer and Lemeshow Test* goodness-of-fit contingency table across risk deciles.
  * *Classification Table* with configurable cut value (default 0.5), computing observed vs. predicted counts, sensitivity, specificity, and overall percentage correct.
  * *Variables in the Equation* table: Parameter estimates $B$, Std. Error, Wald statistic, degrees of freedom, significance, Odds Ratio $\text{Exp}(B)$, and 95% confidence intervals for $\text{Exp}(B)$.
* **Multinomial Logistic Regression (`Multinomial Logistic Regression` / `NOMREG`):**
  * High-performance Newton-Raphson optimizer (Fisher Scoring) with backtracking line search, numerically stable `log-sum-exp` implementation, and Ridge regularization ($\lambda = 10^{-6}$) to prevent divergence under multicollinearity or quasi-complete separation (enforcing the `Never Crash` rule).
  * Multi-category polytomous target support with flexible reference category selection (last, first, or custom numeric category).
  * Categorical factors support with automated indicator contrast coding (dummy variables) and continuous covariates.
  * Real-time target variable cardinality validation in UI dialog: automatic verification of $\ge 2$ categories with responsive warning banners and action button lock.
  * Complete suite of 7 authentic IBM SPSS Pivot Tables:
    * *Case Processing Summary*: counts and percentages across target and factor categories.
    * *Model Fitting Information*: $-2 \text{ Log Likelihood}$, $\chi^2$, degrees of freedom, and $p$-value comparing Final model against Intercept Only.
    * *Goodness-of-Fit*: Pearson and Deviance $\chi^2$ goodness-of-fit tests over subpopulations.
    * *Pseudo R-Square*: Cox & Snell, Nagelkerke, and McFadden pseudo-$R^2$ statistics.
    * *Likelihood Ratio Tests*: likelihood ratio tests for each effect in the model.
    * *Parameter Estimates*: unstandardized coefficients $B$, Std. Error, Wald $\chi^2$, degrees of freedom, significance, Odds Ratio $\text{Exp}(B)$, and 95% confidence intervals for $\text{Exp}(B)$ relative to the reference category.
    * *Classification Table*: observed vs. predicted contingency table with overall classification accuracy percentage.
* **Curve Estimation (`Curve Estimation` / `CURVEFIT`):**
  * Simultaneous fitting and comparison of 11 regression models: Linear, Logarithmic, Inverse, Quadratic, Cubic, Power, Compound, S-curve, Logistic, Growth, and Exponential.
  * Comprehensive *Model Summary & Parameter Estimates* table comparing $R^2$, $F$, degrees of freedom, significance, constant $b_0$, and regression parameters $b_1, b_2, b_3$.

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

### 4.8. General Linear Model (GLM)
* **Univariate ANOVA/ANCOVA (`Univariate ANOVA` / `UNIANOVA`):**
  * General Linear Model accommodating fixed factors, random factors, and continuous covariates.
  * *Tests of Between-Subjects Effects* table: Corrected Model, Intercept, Main Effects, Factor-Covariate Interactions, Error, Total, Corrected Total.
  * Type I and Type III Sums of Squares ($SS$), degrees of freedom ($df$), Mean Squares ($MS$), $F$-statistics, significance levels, and Partial Eta Squared ($\eta_p^2$).
  * Levene's Test of Equality of Error Variances.
* **Multivariate MANOVA (`Multivariate MANOVA` / `GLM Multivariate`):**
  * Simultaneous evaluation of multiple dependent variables across categorical factors and covariates.
  * Canonical multivariate criteria: Wilks' Lambda ($\Lambda$), Pillai's Trace, Hotelling's Trace, and Roy's Largest Root.
  * Multivariate $F$-approximations, hypothesis $df$, error $df$, significance, and Partial Eta Squared.
  * Follow-up Univariate Tests of Between-Subjects Effects for each dependent variable.
* **Repeated Measures ANOVA (`Repeated Measures ANOVA` / `GLM Repeated Measures`):**
  * Analysis of within-subjects repeated measurement factors and treatment conditions.
  * Mauchly's Test of Sphericity ($W$, Approx. $\chi^2$, $df$, Sig).
  * Automatic sphericity epsilon corrections: Greenhouse-Geisser ($\hat{\epsilon}$), Huynh-Feldt ($\tilde{\epsilon}$), and Lower-bound.
  * *Tests of Within-Subjects Effects* table evaluated under Sphericity Assumed, Greenhouse-Geisser, Huynh-Feldt, and Lower-bound assumptions.

### 4.9. Cluster Analysis (Classify)
* **K-Means Cluster Analysis (`K-Means Cluster Analysis` / `QUICK CLUSTER`):**
  * Fast partitioning of observations into $K$ disjoint clusters via Lloyd's Euclidean distance algorithm.
  * Configurable cluster count $K$, maximum iteration threshold, and convergence criterion.
  * *Initial Cluster Centers* and *Iteration History* tables.
  * *Final Cluster Centers* table and cluster distances.
  * *ANOVA* table comparing between-cluster and within-cluster mean squares ($F$, Sig) to evaluate variable separation power.
  * *Number of Cases in Each Cluster* distribution table.

### 4.10. Summaries & Reports (Reports)
* **Codebook (`Codebook` / `CODEBOOK`):**
  * Comprehensive automated data dictionary document for dataset auditing and documentation.
  * Variable metadata block: Variable Name, Label, Storage Type, Format, Measurement Scale, and Missing Values.
  * Categorical and discrete variables: Value labels, raw codes, valid counts, and percentages.
  * Scale and continuous variables: Valid $N$, Mean, Standard Deviation, and Quartiles ($Q_1, Q_2, Q_3$).
* **Case Summaries (`Case Summaries` / `SUMMARIZE`):**
  * Subgroup case listings and descriptive summary tables grouped by categorical factors.
  * Optional case-level display with maximum case count limiting ($N$ first cases).
  * Summary statistics per group: Total $N$, Mean, Median, Standard Deviation, Variance, Sum, Minimum, Maximum, and % of Total Cases.

### 4.11. Survival Analysis (Survival)
* **Kaplan-Meier Survival Analysis (`Kaplan-Meier` / `KM`):**
  * Nonparametric estimation of survival functions and cumulative survival distributions over time.
  * Time-to-event variable paired with a categorical status variable (user-defined single event code or range).
  * Detailed *Survival Table*: Time, Status (Terminal Event vs. Censored), Cumulative Survival Probability, Standard Error, Cumulative Events, and Number of Remaining Cases.
  * *Mean and Median Survival Time* estimates with standard errors and 95% confidence intervals.
  * Subgroup factor comparisons: Log-Rank (Mantel-Cox) test ($\chi^2, df, p$), Breslow (Generalized Wilcoxon) test, and Tarone-Ware test.
* **Actuarial Life Tables (`Life Tables` / `SURVIVAL`):**
  * Interval-based actuarial survival and hazard estimation for large cohorts and epidemiological studies.
  * Configurable interval start, end, and step size.
  * Canonical actuarial table columns: Number Entering Interval, Number Withdrawn during Interval, Number Exposed to Risk, Number of Terminal Events, Proportion Terminating, Proportion Surviving, Cumulative Proportion Surviving at End of Interval, Probability Density, Hazard Rate, and Median Survival Time.
  * Subgroup comparison via Wilcoxon (Gehan) statistic.
* **Cox Proportional Hazards Regression (`Cox Regression` / `COXREG`):**
  * Semiparametric modeling of time-to-event outcomes with multiple continuous and categorical covariates.
  * Partial Likelihood maximization via Newton-Raphson with backtracking line search and Never Crash guarantees.
  * Canonical tie-handling methods: Breslow (SPSS default) and Efron approximations.
  * Stratified analysis support: shared regression coefficients across multiple clinical strata with stratum-specific baseline hazard functions.
  * Authentic IBM SPSS Pivot Tables:
    * *Case Processing Summary*: Total N, Events, Censored cases and percentages per stratum and overall.
    * *Omnibus Tests of Model Coefficients*: $-2 \text{ Log Likelihood}$, $\chi^2$, degrees of freedom, and $p$-value.
    * *Variables in the Equation*: Parameter estimates $B$, Std. Error, Wald $\chi^2$, degrees of freedom, significance, Hazard Ratio $\text{Exp}(B)$, and 95% confidence intervals for $\text{Exp}(B)$.
    * *Survival Table at Mean of Covariates*: Cumulative baseline hazard $\hat{H}_0(t)$, baseline survival $\hat{S}_0(t)$, and adjusted survival at covariate means.
  * Vector step-function Survival Plot rendered using design system tokens.

---

## 5. Output Management & Clipboard Integration (Output Viewer)

* **Interactive Outline Navigation:**
  * Tree panel on the left reflecting document structure across procedures, syntax logs, and tables.
  * Smooth scrolling and focus ring highlighting upon selecting any item in the tree.
  * Synchronized two-way selection: clicking a table in the scroll canvas highlights its node in the outline.
  * Outline toolstrip: collapse all, expand all, clear entire output, and delete selected nodes with the `Delete` key.
* **Authentic Pivot Tables:**
  * High-fidelity reproduction of IBM SPSS Statistics typography and layouts: nested multi-tier headers, alternating row striping, right-aligned numeric formatting, and criterion footnotes.
* **Table Style & Conditional Formatting Engine:**
  * Declarative typed engine applying conditional formatting across pivot table cells based on value predicates (`All`, `Equal`, `NotEqual`, `LessThan`, `LessOrEqual`, `GreaterThan`, `GreaterOrEqual`, `Between`, `NotBetween`, `TopN`, `BottomN`, `SysMis`).
  * Dimension and value targeting across table subtypes, columns, rows, and label tokens with substring protection.
  * Visual formatting options (bold, italic, underline, RGB text color, RGB background color, decimal places, value replacement) and TableLook defaults reset.
  * Canonical IBM SPSS Format subdialog parity: 34 cell format masks (numbers, currencies, scientific, dates, times, days, months), authentic 5x6 SPSS color matrix with special White/Black/No Color swatches and custom RGB picker, and 3-way application scope (*Only Cells, Entire Column, Entire Row*).
  * Authentic IBM SPSS DialogKit interface: 5 in-cell ComboBox columns (*Table, Value, Dimension, Condition, Format*), live sample preview box, typography toolbar (`[ B ] [ I ] [ U ]`), rule reordering, and transactional rollback on cancel.
  * Output Viewer rendering with custom background fills, custom typography colors, and dynamic column layouts.
  * Auxiliary "Conditional Styles Report" audit table summarizing applied targets, conditions, and modified cell tallies.
  * Canonical SPSS `OUTPUT MODIFY` syntax generation upon Paste action.
  * Rich HTML report export preserving applied cell styling via inline CSS attributes.
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

## 7. AI Readiness (Agent-Ready) & Model Context Protocol (MCP) Bridge

StatCraft features a production-ready, native **Model Context Protocol (MCP)** server and IPC bridge, allowing AI assistants (Antigravity, Claude, ChatGPT, and local LLMs) to programmatically control the desktop environment and run automated statistical pipelines.

* **Two Native Operation Modes:**
  * **Headless CLI Server (`statcraft --mcp`):** Standard input/output (stdio) JSON-RPC transport fully compliant with the open Model Context Protocol specification, enabling external coding agents to read variables, calculate models, and inspect datasets without launching a graphical window.
  * **Live Desktop GUI Bridge (Port 18493):** Built-in background server enabling bidirectional communication with the running GUI application (`Extensions -> MCP Bridge / Copilot`), updating the live Data View and injecting output items in real time.
* **Core Agent Tools:**
  * `get_dataset_info`: Returns a complete JSON structural dictionary of the active dataset: row count $N$, column count $P$, variable names, measurement scales, missing value rules, and value label dictionaries.
  * `execute_syntax`: Parses and executes canonical SPSS command scripts (`.sps`) through the engine `CommandBus`, returning execution status, error logs, and formatted Pivot Table representations.
  * `insert_report`: Dynamically generates and injects rich narrative markdown reports, clinical interpretations, or executive summaries directly into the Output Viewer tree and scroll canvas.
* **Agent Parameter Schemas (`specs()`):**
  * Every engine command (`statcraft_syntax`) exports strongly-typed parameter schemas via `specs()`, allowing LLMs to auto-generate valid argument payloads with zero syntactic hallucinations.

---

## 8. Design System, Reliability, Localization, and Theming

* **Universal Theming Engine (Tokens):**
  * Three authentic visual themes designed for statistical workflows:
    * **Dark Pro (Default):** High-contrast ergonomic dark theme engineered for extended data entry and low-glare nighttime analysis.
    * **SPSS Classic Light:** Authentic ivory/slate palette matching classic desktop IBM SPSS Statistics typography and layouts.
    * **System Auto:** Seamless automatic synchronization with host operating system appearance settings.
* **Dockable Multi-Doc Workspace Lifecycle:**
  * **Protected Core Tabs:** Fundamental dataset spaces (*Data View* and *Variable View*) are non-closable (`closeable = false`), eliminating accidental workspace loss.
  * **Closable Auxiliary Tabs:** Dynamic Syntax Editor tabs and secondary Output Viewers can be freely created, docked, split, and closed with `[x]`.
  * **Window Menu Orchestration:** Full tab discovery and focus activation via the top menu bar (`Window -> Focus...`).
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

## 9. Roadmap & Future Features (InDev / Planned)

To guarantee complete technical honesty and transparency, the following modules and menu actions represent genuine future roadmap items currently scheduled for upcoming releases:

* **Vector Charting Engine (`statcraft_charts`):**
  * Interactive Chart Builder powered by `egui_plot`: Histograms with normal curve overlays, Scatter Plots with regression fit lines, Boxplots with outlier detection, Bar Charts with 95% error bars, and interactive Brushing & Linking with Data View.
* **Advanced Document Exporters:**
  * Native rendering and direct export of Output Viewer documents to formatted PDF, Microsoft Word (`.docx`), and academic LaTeX formats (TSV for Excel, HTML, and Plain Text are fully implemented).
* **Dataset Restructuring & File Operations (Data Menu):**
  * Data Restructuring (*Restructure Wide to Long / Long to Wide*) and Data Aggregation (*Aggregate*).
* **Time Series & Advanced Forecasting:**
  * ARIMA / SARIMA modeling, Exponential Smoothing (Holt-Winters), and autocorrelation diagnostics (ACF / PACF).
* **Multilevel & Complex Modeling:**
  * Linear Mixed Models (LMM / HLM) with random intercepts/slopes, Complex Samples multistage survey weighting, and Classification Decision Trees (CHAID, CRT).


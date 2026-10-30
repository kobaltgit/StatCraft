# StatCraft — SPSS Feature Parity Checklist

Этот документ отслеживает соответствие функционала StatCraft меню и командам IBM SPSS Statistics. Обновляется автоматически через `cargo xtask parity`.

| SPSS Menu Path | Command ID | Status | Notes |
|---|---|---|---|
| File > Open > Data (CSV) | `data.import_csv` | In Progress | Streaming CSV/TSV parser |
| File > Import Data > Excel | `data.import_excel` | Done | Calamine Excel (.xlsx, .xls, .ods) reader, type inference, SPSS sanitized headers |
| Edit > Undo | `edit.undo` | Planned | DeltaBuffer undo |
| Edit > Redo | `edit.redo` | Planned | DeltaBuffer redo |
| View > Value Labels | `view.toggle_labels` | Done | Toggle code/label rendering (`1/A` toolbar button) |
| Variable > Set Label | `var.set_label` | Done | Update column label |
| Variable > Set Measure | `var.set_measure` | Done | Set Scale / Ordinal / Nominal |
| Variable > Set Value Labels | `var.set_value_labels` | Done | Value labels dialog and mapping with Undo/Redo |
| Variable > Set Missing Values | `var.set_missing_values` | Done | Discrete and range missing values with Undo/Redo |
| Variable > Rename | `var.rename` | Done | Rename variable and column |
| Variable > Set Type | `var.set_type` | Done | 9 SPSS variable types dialog and formatting |
| Variable > Set Decimals | `var.set_decimals` | Done | Set decimal places with ▲/▼ spinners |
| Variable > Set Width | `var.set_width` | Done | Set column width with ▲/▼ spinners |
| Variable > Set Alignment | `var.set_align` | Done | Set Left / Right / Center |
| Data > Insert Variable | `var.add` | Done | Insert new variable row and column |
| Data > Delete Variable | `var.delete` | Done | Delete variable and column |
| Data > Insert Case | `data.add_row` | Done | Insert case / empty row |
| Data > Delete Case | `data.delete_row` | Done | Delete case / row |
| Data > Sort Cases | `data.sort_cases` | Done | Multi-column sort (A/D), SPSS sysmis rules, DialogKit dialog, Undo/Redo |
| Data > Select Cases | `data.select_cases` | Done | Filter/Delete cases, filter_$ variable, relational & logical expression builder, slash across excluded rows, status bar indicator, Undo/Redo |
| Data > Split File | `data.split_file` | Done | Segment analysis by sub-groups (Compare groups / Organize output), SPSS SPLIT FILE BY / OFF syntax, status bar indicator, DialogKit dialog, Undo/Redo |
| Data > Weight Cases | `data.weight_cases` | Done | Frequency & sampling weighting, SPSS WEIGHT BY / WEIGHT OFF syntax, status bar indicator, DialogKit dialog, Undo/Redo |
| Transform > Compute Variable | `transform.compute` | Done | Expression evaluator, DialogKit calculator dialog, undo support |
| Transform > Recode into Different Variables | `transform.recode.diff` | Done | Mapping engine (exact, ranges, sysmis, else), DialogKit dialog & Old and New Values subdialog, undo support |
| Transform > Recode into Same Variables | `transform.recode.same` | Done | In-place recode, DialogKit dialog & Old and New Values subdialog, undo support |
| Transform > Automatic Recode | `transform.auto_recode` | Done | Sequential integer coding (1, 2, ..., K) with automatic value labels, Lowest/Highest order, DialogKit dialog, SPSS AUTORECODE syntax, undo support |
| Transform > Shift Values | `transform.shift_values` | Done | Lead/Lag values shift (K steps), DialogKit dialog with direction/step, SPS syntax, undo support |
| Transform > Count Values within Cases | `transform.count_values` | Done | Count occurrences of values/ranges/sysmis across cases, DialogKit dialog & Values subdialog, SPSS COUNT syntax, undo support |
| Transform > Create Dummy Variables | `transform.create_dummies` | Done | Indicator (0/1) binary variables generation (One-Hot Encoding), root name / prefix, reference category omission (k-1 dummies), DialogKit dialog, SPSSINC CREATE DUMMIES syntax, undo support |
| Transform > Set Random Number Seed | `transform.random_seed` | Done | Global RNG state & thread-safe seed lifecycle, DialogKit dialog, SPSS SET SEED syntax, undo support |
| Transform > Replace Missing Values | `transform.replace_missing` | Done | Imputation methods (Series Mean, Nearby Mean, Nearby Median, Linear Interpolation, Trend), DialogKit dialog with method selection & span, SPSS RMV syntax, undo support |
| Transform > Create Time Series | `transform.create_time_series` | Done | Time series functions (DIFF, SDIFF, SMA, CSUM), DialogKit dialog with function & order/span settings, SPSS CREATE syntax, batch undo support |
| Transform > Visual Binning | `transform.visual_binning` | Done | Continuous variable discretization (Equal Width, Equal Percentiles, Std Dev), DialogKit dialog & Make Cutpoints subdialog, auto/custom Value Labels, SPSS RECODE/VALUE LABELS syntax, undo support |
| Transform > Optimal Binning | `transform.optimal_binning` | Done | Supervised discretization via Fayyad & Irani MDLP entropy optimization, max bins parameter, variable prefix, automatic Value Labels, DialogKit dialog, SPSS OPTIMAL BINNING syntax, undo support |
| Transform > Rank Cases | `transform.rank_cases` | Done | Rank calculation with ties handling (Mean, Low, High, Sequential), direction (Ascending/Descending), types (Simple, Fractional, Percent, Ntiles, Normal: Blom, Tukey, Rankit, VW), DialogKit dialog & Types subdialog, SPSS RANK syntax, batch undo support |
| Transform > Date and Time Wizard | `transform.date_time_wizard` | Done | Extract date parts (Year, Month, Day, Hour, Minute, Second, Day of week, Quarter), interval between two dates, duration summation/subtraction, DialogKit dialog, SPSS COMPUTE XDATE/DATEDIFF/DATESUM syntax, undo support |
| Analyze > Descriptive Statistics > Frequencies | `stat.frequencies` | Done | Frequency tables, percentiles, quartiles, dialog & SPSS tables |
| Analyze > Descriptive Statistics > Descriptives | `stat.descriptives` | Done | Mean, StdDev, Variance, Min, Max, Range, Skewness, Kurtosis, SE Mean, Dialog & Pivot Table |
| Analyze > Descriptive Statistics > Crosstabs | `stat.crosstabs` | Done | Contingency tables, Pearson Chi-Square, Fisher's Exact, Continuity Correction, Likelihood Ratio, Phi & Cramér's V, DialogKit dialog & 4 authentic SPSS Pivot Tables |
| Analyze > Compare Means > Means | `stat.means` | Done | Subgroup means breakdown, cell statistics (Mean, N, StdDev, Variance, Median, Min, Max), ANOVA table (Between, Within, Total), Measures of Association (Eta, Eta Squared), DialogKit dialog & SPSS TABLES BY syntax |
| Analyze > Compare Means > One-Sample T Test | `stat.ttest.onesample` | Done | Student's One-Sample t-test, test value, df, p-value, mean diff, 95% CI, dialog & SPSS Pivot Tables |
| Analyze > Compare Means > Independent-Samples T Test | `stat.ttest.indep` | Done | Independent-Samples t-test, Levene test (F, Sig), Student's t (equal var) & Welch's t (unequal var), df, p-value, 95% CI, Define Groups dialog & two-tier SPSS Pivot Tables |
| Analyze > Compare Means > Paired-Samples T Test | `stat.ttest.paired` | Done | Paired-Samples t-test, Paired Descriptives (Mean, StdDev, SE), Pearson Correlation & Sig, Paired Differences (Mean, StdDev, SE, 95% CI, t, df, Sig), Paired Variables Dialog & 3 SPSS Pivot Tables |
| Analyze > Compare Means > One-Way ANOVA | `stat.anova.oneway` | Done | One-Way ANOVA, Fisher's F-test, p-value via incomplete beta, Levene's test of homogeneity of variances, Descriptives with 95% CI, dialog & 3 authentic SPSS Pivot Tables |
| Analyze > Correlate > Bivariate | `stat.correlations` | Done | Pearson r, Spearman rho, Kendall tau-b, 1-tailed / 2-tailed p-values, significance flags (* / **), cross-product deviations & covariances, pairwise/listwise missing handling, DialogKit dialog & authentic SPSS Pivot Tables |
| Analyze > Nonparametric Tests > Two Independent Samples | `stat.npar.mann_whitney` | Done | Mann-Whitney U, Wilcoxon W, Z-statistic, Asymptotic 2-tailed p-value with tie adjustment, Define Groups subdialog, authentic Ranks and Test Statistics tables |
| Analyze > Nonparametric Tests > K Independent Samples | `stat.npar.kruskal_wallis` | Done | Kruskal-Wallis H test, tie adjustment H_adj, df = k - 1, asymptotic p-value from chi-square distribution, Define Range subdialog (min/max), authentic SPSS Ranks and Test Statistics tables |
| Analyze > Nonparametric Tests > Two Related Samples | `stat.npar.related2` | Done | Wilcoxon Signed-Rank Test & Sign Test, Z-statistic, Asymptotic & Exact 2-tailed p-values with ties adjustment, DialogKit paired selection dialog, authentic Ranks, Frequencies, and Test Statistics tables |
| Analyze > Regression > Linear | `stat.regression.linear` | Done | Multiple linear regression (OLS & WLS), Model Summary (R, R2, Adj R2, Std. Error), ANOVA (Regression, Residual, Total, F, Sig), Coefficients (B, Std. Error, Beta, t, Sig, 95% CI, Collinearity Tolerance & VIF), authentic SPSS Pivot Tables, DialogKit dialog, Golden Oracle verified against SciPy |
| Analyze > Scale > Reliability Analysis | `stat.scale.reliability` | Done | Scale reliability analysis (Cronbach's Alpha, Standardized Alpha, Case Processing Summary, Item Statistics, Item-Total Statistics, Scale Statistics), authentic SPSS Pivot Tables, DialogKit dialog & SPSS syntax |
| Analyze > Dimension Reduction > Factor | `stat.factor` | Done | Factor Analysis / PCA (Principal Components), KMO and Bartlett's test of sphericity, Communalities (Initial & Extraction), Total Variance Explained, Component Matrix, DialogKit dialog & authentic SPSS tables, SPSS FACTOR syntax |

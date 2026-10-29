# StatCraft — SPSS Feature Parity Checklist

Этот документ отслеживает соответствие функционала StatCraft меню и командам IBM SPSS Statistics. Обновляется автоматически через `cargo xtask parity`.

| SPSS Menu Path | Command ID | Status | Notes |
|---|---|---|---|
| File > Open > Data (CSV) | `data.import_csv` | In Progress | Streaming CSV/TSV parser |
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
| Analyze > Descriptive Statistics > Frequencies | `stat.frequencies` | Done | Frequency tables, percentiles, quartiles, dialog & SPSS tables |
| Analyze > Descriptive Statistics > Descriptives | `stat.descriptives` | Done | Mean, StdDev, Variance, Min, Max, Range, Skewness, Kurtosis, SE Mean, Dialog & Pivot Table |
| Analyze > Descriptive Statistics > Crosstabs | `stat.crosstabs` | Done | Contingency tables, Pearson Chi-Square, Fisher's Exact, Continuity Correction, Likelihood Ratio, Phi & Cramér's V, DialogKit dialog & 4 authentic SPSS Pivot Tables |
| Analyze > Compare Means > One-Sample T Test | `stat.ttest.onesample` | Done | Student's One-Sample t-test, test value, df, p-value, mean diff, 95% CI, dialog & SPSS Pivot Tables |
| Analyze > Compare Means > Independent-Samples T Test | `stat.ttest.indep` | Done | Independent-Samples t-test, Levene test (F, Sig), Student's t (equal var) & Welch's t (unequal var), df, p-value, 95% CI, Define Groups dialog & two-tier SPSS Pivot Tables |
| Analyze > Compare Means > Paired-Samples T Test | `stat.ttest.paired` | Done | Paired-Samples t-test, Paired Descriptives (Mean, StdDev, SE), Pearson Correlation & Sig, Paired Differences (Mean, StdDev, SE, 95% CI, t, df, Sig), Paired Variables Dialog & 3 SPSS Pivot Tables |
| Analyze > Compare Means > One-Way ANOVA | `stat.anova.oneway` | Done | One-Way ANOVA, Fisher's F-test, p-value via incomplete beta, Levene's test of homogeneity of variances, Descriptives with 95% CI, dialog & 3 authentic SPSS Pivot Tables |
| Analyze > Correlate > Bivariate | `stat.correlations` | Done | Pearson r, Spearman rho, Kendall tau-b, 1-tailed / 2-tailed p-values, significance flags (* / **), cross-product deviations & covariances, pairwise/listwise missing handling, DialogKit dialog & authentic SPSS Pivot Tables |
| Analyze > Nonparametric Tests > Two Independent Samples | `stat.npar.mann_whitney` | Done | Mann-Whitney U, Wilcoxon W, Z-statistic, Asymptotic 2-tailed p-value with tie adjustment, Define Groups subdialog, authentic Ranks and Test Statistics tables |
| Analyze > Nonparametric Tests > K Independent Samples | `stat.npar.kruskal_wallis` | Done | Kruskal-Wallis H test, tie adjustment H_adj, df = k - 1, asymptotic p-value from chi-square distribution, Define Range subdialog (min/max), authentic SPSS Ranks and Test Statistics tables |
| Analyze > Nonparametric Tests > Two Related Samples | `stat.npar.related2` | Done | Wilcoxon Signed-Rank Test & Sign Test, Z-statistic, Asymptotic & Exact 2-tailed p-values with ties adjustment, DialogKit paired selection dialog, authentic Ranks, Frequencies, and Test Statistics tables |

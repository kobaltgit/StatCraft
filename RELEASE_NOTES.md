# StatCraft v0.4.3

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.3 is a critical precision and numerical stability release that restores full 64-bit IEEE 754 precision across all analytical procedures and dialogs, completely eliminating inadvertent decimal rounding from variable display formats.

### 🎯 Full-Precision Procedure Calculations (PR #5 by @jjohare)
* **Direct Numeric Access (`DataSet::get_cell_numeric`):**
  * Core data storage in `statcraft_core` now provides `get_cell_numeric(&self, row: usize, col: usize) -> Option<f64>`, extracting stored numbers directly from memory.
  * **Zero Allocation & High Performance:** Replaces costly `format!` string allocations and subsequent float parsing with direct pointer dereferencing, providing significant speedups on large datasets.
  * **Print Format Decoupling:** Variable print formats (`decimals`, e.g. `F8.2`) now strictly govern screen and export display only, adhering to the IBM SPSS canon where display formats never alter underlying computation.
* **Refactoring Across 21 Analytical Procedures (`statcraft_syntax`):**
  * Completely eliminated precision loss across all analytical modules:
    * **Descriptives & Moments:** `DESCRIPTIVES`, `MEANS`.
    * **Correlation & Regression:** `CORRELATIONS`, `PARTIAL CORR`, `REGRESSION` (Linear), `CURVEFIT`.
    * **Group Comparisons & ANOVA:** `ONE-WAY ANOVA`, `T-TEST` (One-Sample, Independent, Paired).
    * **General Linear Model:** `GLM UNIVARIATE`, `GLM MULTIVARIATE`, `GLM REPEATED`.
    * **Nonparametric Tests:** `MANN-WHITNEY`, `KRUSKAL-WALLIS`, `RELATED SAMPLES` (Wilcoxon/Sign).
    * **Multivariate & Advanced:** `FACTOR`, `RELIABILITY`, `LOGISTIC REGRESSION`, `MULTINOMIAL LOGISTIC` (`NOMREG`), `SELECT IF`.
* **Currency & Formatted Number Computation:**
  * Fixed an issue where variables with currency (`Dollar`, `CustomCurrency`) or thousands separators (`Comma`, `Dot`) failed to parse in numerical procedures. All formatted numeric columns now compute accurately.
* **Categorical Key Preservation:**
  * Factor variables, grouping keys, and value labels intentionally remain evaluated via display strings to preserve exact text-matching behavior for categorical groups.

### 🖥️ GUI Dialog Precision Polish (`statcraft_app`)
* **Distinct Level Detection in Dialogs:**
  * Updated category counting in `logistic_regression_dialog` and `multinomial_logistic_dialog` to use `ds.get_cell_numeric`.
  * Prevents distinct category levels with small deltas (e.g. `0.001` vs `0.002`) from collapsing into a single level under default `F8.2` display formats.

### 🧪 Golden Oracle Precision Verification
* **Precision Invariant Tests (`numeric_precision_tests.rs`):**
  * Added regression test suite verifying exact numerical tolerance $\le 10^{-9}$ for mean, minimum, slope, intercept, and Pearson $r$.
* **Dialog Category Tests (`numeric_dialog_precision_tests.rs`):**
  * Automated testing verifying category separation, currency format compatibility, and missing-value isolation.

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive (silent GUI launch).
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.3 — это критическое обновление математической точности и стабильности, восстанавливающее вычисления с полной 64-битной точностью IEEE 754 во всех статистических процедурах и диалогах, полностью устраняя неявное округление чисел через экранные форматы переменных.

### 🎯 Вычисления с полной машинной точностью (PR #5 от @jjohare)
* **Прямой числовой доступ (`DataSet::get_cell_numeric`):**
  * В хранилище данных `statcraft_core` добавлен метод `get_cell_numeric(&self, row: usize, col: usize) -> Option<f64>`, извлекающий сырые числа `RawValue::Number` напрямую из памяти.
  * **Нулевые аллокации и ускорение:** Устранены промежуточные выделения строк в куче `format!` и обратный парсинг строк, что кратно ускоряет расчеты процедур на больших массивах данных.
  * **Разделение отображения и вычислений:** Спецификация формата печати (`decimals`, например `F8.2`) теперь управляет исключительно представлением на экране, строго по канону IBM SPSS, где формат печати никогда не искажает точность математического ядра.
* **Рефакторинг 21 процедуры синтаксиса (`statcraft_syntax`):**
  * Полностью устранена потеря точности во всех аналитических процедурах:
    * **Описательные статистики:** `DESCRIPTIVES`, `MEANS`.
    * **Корреляции и регрессия:** `CORRELATIONS`, `PARTIAL CORR`, `REGRESSION` (линейная), `CURVEFIT`.
    * **Сравнение средних и ANOVA:** `ONE-WAY ANOVA`, `T-TEST` (одновыборочный, двухвыборочный, парный).
    * **Общая линейная модель:** `GLM UNIVARIATE`, `GLM MULTIVARIATE`, `GLM REPEATED`.
    * **Непараметрические критерии:** `MANN-WHITNEY`, `KRUSKAL-WALLIS`, `RELATED SAMPLES` (Вилкоксон / Знаки).
    * **Многомерные и специализированные:** `FACTOR`, `RELIABILITY`, `LOGISTIC REGRESSION`, `MULTINOMIAL LOGISTIC` (`NOMREG`), `SELECT IF`.
* **Корректные вычисления по колонкам с валютами и разделителями:**
  * Устранена проблема, из-за которой переменные с форматами валют (`Dollar`, `CustomCurrency`) или разделителями тысяч (`Comma`, `Dot`) выдавали ошибку парсинга в процедурах. Теперь расчеты по валютным колонкам выполняются безошибочно.
* **Сохранение группирующего поведения:**
  * Факторные переменные, ключи группировки и метки значений осознанно сохранены на строковом сопоставлении для точной группировки категориальных уровней.

### 🖥️ Точность в диалоговых окнах GUI (`statcraft_app`)
* **Определение уникальных уровней в диалогах:**
  * Подсчет уникальных категорий зависимой переменной в `logistic_regression_dialog` и `multinomial_logistic_dialog` переведен на `ds.get_cell_numeric`.
  * Исключено ошибочное объединение близких числовых категорий (например, `0.001` и `0.002`) при формате печати `F8.2`.

### 🧪 Верификация тестами Золотого Оракула
* **Тесты математической точности (`numeric_precision_tests.rs`):**
  * Внедрен тестовый пакет с допуском $\le 10^{-9}$ для среднего, минимума, наклона, интерсепта и корреляции Пирсона $r$.
* **Тесты диалогов (`numeric_dialog_precision_tests.rs`):**
  * Автоматизированная проверка разделения категорий, совместимости с форматами валют и изоляции пропусков.

### 📦 Мультиплатформенные артефакты релиза
* **Windows (x64):** Инсталлятор Inno Setup (`.exe`) + портативный ZIP-архив (тихий запуск GUI).
* **Linux (x64):** Пакет Debian (`.deb`) + портативный архив TAR.GZ.
* **macOS (Apple Silicon ARM64 и Intel x64):** Образ DMG + бандл приложения (`.app`) TAR.GZ.

# StatCraft v0.4.7

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.7 introduces full **Bootstrap Resampling** parity with IBM SPSS Statistics (**Features 020–022**): a clean-room `Mt19937` Mersenne Twister engine, `Simple` and `Stratified` resampling, `Percentile` and `BCa` confidence intervals, a reusable `BootstrapDialog` across 9 statistical procedures, `SET MTINDEX` / `BOOTSTRAP` SPSS syntax support, and Golden Oracle verification against `scipy.stats.bootstrap` ($\le 10^{-6}$).

### 🎲 Bootstrap Resampling Suite (Features 020–022)
* **Core Engine (`statcraft_core::rng` & `statcraft_algo::bootstrap`):**
  * 100% Safe Rust 32-bit Mersenne Twister (`Mt19937`) with optional fixed seed (`1..=2,000,000,000`, default `2,000,000`).
  * `Simple` and `Stratified` (preserving exact stratum sizes $n_h$) resampling with replacement.
  * `Bias`, `Std. Error`, `Percentile`, and `BCa` (Bias-Corrected and Accelerated with jackknife acceleration $\hat{a}$) confidence intervals.
* **Reusable `BootstrapDialog` (`DialogKit` + Fluent i18n):**
  * Modal subdialog (`[ Perform bootstrapping ]`, `Number of samples`, `Set seed for Mersenne Twister`, `Confidence Intervals` level & `Percentile`/`BCa` selector, `Sampling` `Simple`/`Stratified` + `Strata Variables` picker) with transactional `backup`/`commit`/`cancel`.
* **9 Integrated Statistical Procedures & Output Viewer:**
  * `Frequencies`, `Descriptives`, `Means`, `One-Sample T-Test`, `Independent-Samples T-Test`, `Paired-Samples T-Test`, `One-Way ANOVA`, `Bivariate Correlations` (Pearson, Spearman, Kendall), and `Linear Regression` (OLS).
  * Unified `Bootstrap Specifications` table, `Bias` / `Std. Error` / `CI Lower` / `CI Upper` tables, and TSV export in Output Viewer.
* **SPSS Syntax & Golden Oracle:**
  * `SET MTINDEX=...` and `BOOTSTRAP /SAMPLING ... /CRITERIA ...` generation and one-shot `SyntaxDispatcher::execute_batch` support.
  * Verified against `scipy.stats.bootstrap` (`bootstrap_golden_oracle.rs`) within $\le 10^{-6}$.

### 📋 Full Changelog for v0.4.7
```markdown
## [0.4.7] - 2026-11-25

### Added
- **Bootstrap Resampling Core & Mersenne Twister PRNG (Feature 020 — Stage 1):**
  - 100% Safe Rust Mt19937 PRNG in `statcraft_core::rng` with unbiased index sampling.
  - Reusable `statcraft_algo::bootstrap` engine (`Simple` & `Stratified`, `Percentile` & `BCa` CI with jackknife acceleration).
  - `SET MTINDEX` / `BOOTSTRAP` SPSS syntax emitter and one-shot `pending_bootstrap` batch modifier in `statcraft_syntax`.
- **Reusable Bootstrap Dialog & Frequencies Integration (Feature 021 — Stage 2):**
  - `BootstrapDialog` (`DialogKit`, transactional backup/restore, Fluent `en.ftl`/`ru.ftl`).
  - End-to-end `Frequencies` bootstrap computation, `BOOTSTRAP` + `FREQUENCIES` syntax, and Output Viewer / TSV export.
- **Multi-Procedure Bootstrap Expansion & Golden Oracle (Feature 022 — Stage 3):**
  - Expanded bootstrap across `Descriptives`, `Means`, `One-Sample T-Test`, `Independent-Samples T-Test`, `Paired-Samples T-Test`, `One-Way ANOVA`, `Bivariate Correlations`, and `Linear Regression`.
  - Added `bootstrap_golden_oracle.rs` ($\le 10^{-6}$ vs SciPy) and `output_viewer_bootstrap_invariants.rs`.
```

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.7 добавляет полноценный **Бутстреп-ресэмплинг (Bootstrap)** с паритетом IBM SPSS Statistics (**Features 020–022**): чистый генератор Вихрь Мерсенна `Mt19937`, простую и стратифицированную выборку, доверительные интервалы `Percentile` и `BCa`, переиспользуемый диалог `BootstrapDialog` в 9 статистических процедурах, синтаксис `SET MTINDEX` / `BOOTSTRAP` и верификацию Golden Oracle против `scipy.stats.bootstrap` ($\le 10^{-6}$).

### 🎲 Подсистема бутстреп-ресэмплинга (Features 020–022)
* **Математическое ядро (`statcraft_core::rng` и `statcraft_algo::bootstrap`):**
  * 32-битный генератор `Mt19937` на 100% Safe Rust с опциональным начальным значением (`1..=2 000 000 000`, по умолчанию `2 000 000`).
  * Простая (`Simple`) и расслоенная (`Stratified` с сохранением размеров слоёв $n_h$) выборка с возвращением.
  * Расчёт `Bias`, `Std. Error`, а также доверительных интервалов `Percentile` и `BCa` (с коррекцией смещения $z_0$ и jackknife-ускорением $\hat{a}$).
* **Переиспользуемый поддиалог `BootstrapDialog` (`DialogKit` + Fluent i18n):**
  * Все элементы канонического окна SPSS (`Выполнить бутстреп`, `Количество выборок`, `Задать начальное значение для генератора вихрь Мерсенна`, `Доверительные интервалы` `Процентиль`/`BCa`, `Выборка` `Простая`/`Расслоенная` + `Переменные слоёв`) с транзакционным откатом при отмене.
* **Интеграция с 9 процедурами и Output Viewer:**
  * `Frequencies`, `Descriptives`, `Means`, `One-Sample T-Test`, `Independent-Samples T-Test`, `Paired-Samples T-Test`, `One-Way ANOVA`, `Bivariate Correlations` (Pearson, Spearman, Kendall) и `Linear Regression` (OLS).
  * Единая таблица `Bootstrap Specifications`, таблицы `Bias` / `Std. Error` / `CI Lower` / `CI Upper` и экспорт в TSV.
* **Синтаксис SPSS и Golden Oracle:**
  * Генерация `SET MTINDEX=...` + `BOOTSTRAP ...` и поддержка одноразового модификатора в `SyntaxDispatcher::execute_batch`.
  * Эталонная сверка с `scipy.stats.bootstrap` (`bootstrap_golden_oracle.rs`) с точностью $\le 10^{-6}$.

---

### 📦 Дистрибутивы релиза (Artifacts)
* **Windows (x64):** Автономный инсталлятор (`statcraft-windows-x64-setup.exe`) + Портативный архив (`statcraft-windows-x64.zip`).
* **Linux (x64):** Пакет Debian (`statcraft_0.4.7_amd64.deb`) + Портативный TAR.GZ (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon & Intel):** Подписанный DMG-образ (`statcraft-macos.dmg`).

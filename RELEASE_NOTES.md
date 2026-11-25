# StatCraft v0.4.8

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.8 delivers full IBM SPSS Statistics parity for **Descriptives** (**Features 023–025**), **Explore** (**Features 026–028**), and **Crosstabs** (**Features 029–031**) — spanning clean-room statistical algorithms, `.sps` syntax round-trip, `DialogKit` subdialogs, `TableStyle` & `Bootstrap` integration, and `egui_plot` charts.

### 📊 Crosstabs (`CROSSTABS`) Full Parity (Features 029–031)
* **Multi-Layer Tables & Format:** `Layer 1 of N` pagination (`BY` layer chaining), `Display layer variables in table layers`, `Row order` (`Ascending` / `Descending`), `Suppress tables` (`/NOTABLES`), and `Display clustered bar charts` (`/BARCHART`).
* **Exact & Monte Carlo Tests:** `Asymptotic only`, `Monte Carlo` (Patefield $r \times c$ sampler with confidence interval), and bounded `Exact` with configurable time limit.
* **Complete Statistics Suite:** `Chi-square` (including `Linear-by-Linear Association`), `Correlations` (Pearson's $r$, Spearman's $\rho$), Nominal (`Contingency Coefficient`, `Phi & Cramér's V`, `Lambda`, `Uncertainty Coefficient`), Ordinal (`Gamma`, `Somers' d`, `Kendall's tau-b/c`), `Eta`, `Kappa`, `Risk` (OR & Cohort RR with 95% CI), `McNemar`, and stratified `Cochran's & Mantel-Haenszel` ($2 \times 2 \times K$).
* **Cells, Style & Bootstrap:** `Hide small counts`, column proportion `z-test` with Bonferroni adjustment and subscript letters (`a, b, c...`), 3 residual types (`Unstandardized`, `Standardized`, `Adjusted`), `APA style`, 5 `Noninteger Weights` modes, `TableStyle`, `Bootstrap`, and `egui_plot` Clustered Bar Charts.

### 🔍 Explore (`EXAMINE`) & Descriptives (`DESCRIPTIVES`) Parity (Features 023–028)
* **Explore (`EXAMINE`):** M-estimators (Huber, Tukey, Hampel, Andrews), HAVERAGE Percentiles & Tukey's Hinges, `Label Cases by`, `Missing=Report`, 4-row Levene Spread-vs-Level test, `Bootstrap...`, and `egui_plot` Boxplots / Stem-and-Leaf / Histograms / Q-Q / Spread-vs-Level plots.
* **Descriptives (`DESCRIPTIVES`):** Standardized `Z*` score saving, full `Options...` subdialog, and Output Viewer / TSV/HTML/TXT export.

### 📋 Full Changelog for v0.4.8
```markdown
## [0.4.8] - 2026-11-25

### Added
- **Crosstabs (Features 029–031):** Multi-layer tables, Exact/Monte Carlo (Patefield), full Nominal/Ordinal/Eta/Kappa/Risk/McNemar/CMH statistics, z-test + Bonferroni, 3 residual types, 5 non-integer weight modes, 6 subdialogs (incl. Style & Bootstrap), Clustered Bar Charts, and full CROSSTABS .sps round-trip.
- **Explore (Features 026–028):** M-estimators, HAVERAGE Percentiles & Tukey's Hinges, Label Cases by, Missing=Report, 4-row Levene Spread-vs-Level, Bootstrap integration, and egui_plot charts.
- **Descriptives (Features 023–025):** Z-score variable saving, Options subdialog parity, and Output Viewer / export parity.
```

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.8 обеспечивает полный паритет с IBM SPSS Statistics для процедур **Описательные статистики (Descriptives)** (**Features 023–025**), **Исследовать (Explore)** (**Features 026–028**) и **Таблицы сопряжённости (Crosstabs)** (**Features 029–031**) — от чистых математических алгоритмов и синтаксиса `.sps` до диалогов `DialogKit`, стилей таблиц, бутстрепа и векторных графиков `egui_plot`.

### 📊 Таблицы сопряжённости (`CROSSTABS`) — полный паритет (Features 029–031)
* **Многослойные таблицы и формат:** пагинация `Слой 1 из N` (`BY`-слои), `Показать переменные слоёв в слоях таблицы`, `Порядок строк` (`По возрастанию` / `По убыванию`), `Не выводить таблицы` (`/NOTABLES`) и `Кластеризованные столбчатые диаграммы` (`/BARCHART`).
* **Точные критерии и Монте-Карло:** `Только асимптотически`, `Монте-Карло` (алгоритм Патефилда $r \times c$ с доверительным интервалом) и `Точные` с ограничением времени расчёта.
* **Полный набор статистик:** `Хи-квадрат` (вкл. линейно-линейную связь), `Корреляции` (Пирсон, Спирмен), номинальные (`Коэфф. сопряжённости`, `Фи и V Крамера`, `Лямбда`, `Коэфф. неопределённости`), порядковые (`Гамма`, `d Сомерса`, `Тау-b/c Кендалла`), `Эта`, `Каппа`, `Риск` (OR/RR с 95% ДИ), `Макнемара`, а также `Кокрена и Мантеля-Хенцеля` ($2 \times 2 \times K$ с тестами Бреслоу-Дея и Тароне).
* **Ячейки, Стиль и Бутстреп:** скрытие малых частот, `z-критерий` сравнения долей с поправкой Бонферрони и подстрочными буквами (`a, b, c...`), 3 вида остатков, стиль `APA`, 5 режимов нецелочисленных весов, интеграция `TableStyle` и `Bootstrap`, кластеризованные диаграммы `egui_plot`.

### 🔍 Исследовать (`EXAMINE`) и Описательные (`DESCRIPTIVES`) (Features 023–028)
* **Исследовать (`EXAMINE`):** M-оценки (Хубера, Тьюки, Хампеля, Эндрюса), процентили `HAVERAGE` и петли Тьюки, маркировка наблюдений (`Label Cases by`), `Missing=Report`, 4-строчный тест Левена Spread-vs-Level, `Бутстреп...` и графики `egui_plot` (ящичковые, стебель-листья, гистограммы, Q-Q, разброс-уровень).
* **Описательные (`DESCRIPTIVES`):** сохранение стандартизованных `Z*`-оценок, поддиалог `Параметры...` и экспорт TSV/HTML/TXT.

---

### 📦 Дистрибутивы релиза (Artifacts)
* **Windows (x64):** Автономный инсталлятор (`statcraft-windows-x64-installer.exe`) + Портативный архив (`statcraft-windows-x64.zip`).
* **Linux (x64):** Пакет Debian (`statcraft_0.4.8_amd64.deb`) + Портативный TAR.GZ (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon & Intel):** Подписанный DMG-образ (`statcraft-macos-arm64.dmg`, `statcraft-macos-x64.dmg`).

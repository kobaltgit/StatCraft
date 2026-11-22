# StatCraft v0.4.5

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.5 delivers full parity with IBM SPSS Statistics for frequency visualization, introducing the canonical **«Frequencies: Charts»** dialog, bidirectional syntax support (`/BARCHART`, `/PIECHART`, `/HISTOGRAM [NORMAL]`), adaptive statistical histogram binning with Gaussian curve overlays, and authentic Vector-First chart rendering in the Output Viewer.

### 📊 Frequencies: Charts Subdialog & Complete SPSS Parity (Feature 017)
* **Canonical Modal Subdialog (`charts_subdialog.rs`):**
  * Implemented authentic $360 \times 300$ px subdialog adhering strictly to `DialogKit` standards.
  * **Chart Type:** Radio selection between *None* (default), *Bar charts*, *Pie charts*, and *Histograms* (with indented checkbox *Show normal curve on histogram*).
  * **Chart Values:** Radio selection between *Frequencies* (default) and *Percentages*. Contextually interactive: authentically enabled only when *Bar charts* or *Pie charts* is selected, and disabled when *None* or *Histograms* is chosen (as histograms in SPSS are always frequency-based).
  * Full modal state management with cancellation rollback and global reset integration.
* **SPSS Syntax & CommandBus Integration:**
  * Bidirectional syntax generation on *Paste* and batch execution via `statcraft_run_syntax`:
    - `/BARCHART FREQ` and `/BARCHART PERCENT`
    - `/PIECHART FREQ` and `/PIECHART PERCENT`
    - `/HISTOGRAM` and `/HISTOGRAM NORMAL`
  * Complete Undo/Redo matrix preservation across all 41 engine commands.

### 📈 Vector-First Chart Rendering in Output Viewer (`statcraft_app`)
* **100% Vector Rendering via `ui.painter()`:**
  * Zero emojis, zero external web/raster dependencies, zero tofu square glyphs.
  * **Bar Charts (`freq_chart_bar_pie.rs`):** Category labels along the X-axis, frequency/percentage Y-ticks, hover highlights, value labels, and theme-adaptive coloring.
  * **Pie Charts (`freq_chart_bar_pie.rs`):** High-precision vector polygon slice arcs, balanced palette colors from `Tokens`, and dedicated category legend with percentage shares.
  * **Histograms (`freq_chart_histogram.rs`):** Continuous contiguous bins, smooth overlaid Gaussian density curve in accent color, and authentic top-right SPSS Statistics Legend Box (`Mean`, `Std. Dev.`, `N`).
* **Outline Tree Navigation:**
  * Dedicated vector chart miniature icon (`paint_output_chart_icon`) in the Outline navigation tree, enabling one-click smooth scrolling directly to charts.
  * Structured summary fallback for chart entries in TSV and HTML export pipelines.

### 🔬 Mathematical Histogram Binning & Normal Curve Scaling (`statcraft_algo`)
* **Adaptive Bin Calculation (`charts.rs`):**
  * Scott's normal reference rule ($h = 3.49 \cdot s \cdot N^{-1/3}$) and Freedman-Diaconis rule with automatic "nice" rounding ($1, 2, 2.5, 5, 10 \times 10^k$).
  * Scaled Gaussian probability density curve matching histogram frequency area: $y = \frac{N \cdot h}{\sigma \sqrt{2\pi}} \exp\left(-\frac{(x - \mu)^2}{2\sigma^2}\right)$.
  * Comprehensive edge-case immunity: zero variance ($s = 0$), sample sizes $N < 2$, missing values, and graceful warnings for non-numeric variables.
  * Validated through Golden Oracle regression tests (`frequencies_charts_oracle_test.rs`) against reference `scipy.stats` outputs.

### 📋 Full Changelog for v0.4.5
```markdown
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
```

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`statcraft-windows-x64-installer.exe`) + Portable ZIP archive (`statcraft-windows-x64.zip`).
* **Linux (x64):** Debian package (`statcraft_0.4.5_amd64.deb`) + Portable TAR.GZ archive (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon ARM64 & Intel x64):** DMG Disk Images + Portable TAR.GZ Application Bundles (`.app`).

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.5 обеспечивает 100% паритет с IBM SPSS Statistics в области визуализации одномерных распределений: добавлен канонический поддиалог **«Частоты: Диаграммы»**, поддержка двунаправленного синтаксиса (`/BARCHART`, `/PIECHART`, `/HISTOGRAM [NORMAL]`), адаптивный алгоритм биннинга гистограмм с наложением кривой Гаусса и векторный рендеринг в Output Viewer.

### 📊 Поддиалог «Частоты: Диаграммы» и паритет с SPSS (Feature 017)
* **Каноническое диалоговое окно (`charts_subdialog.rs`):**
  * Реализован поддиалог размером $360 \times 300$ px в строгом соответствии с дизайн-системой `DialogKit`.
  * **Тип диаграммы:** выбор между *«Нет»* (по умолчанию), *«Столбчатые»*, *«Круговые»* и *«Гистограммы:»* (со смещенным чекбоксом *«Показать на гистограмме нормальную кривую»*).
  * **Значения на диаграмме:** переключатель *«Частоты»* (по умолчанию) и *«Проценты»*. Контекстная активность: активен только для столбчатых и круговых диаграмм; при выборе гистограмм блокируется (в SPSS гистограммы строятся строго по частотам).
  * Управление состоянием: корректный откат изменений при нажатии «Отмена» и сброс настроек по кнопке «Сброс».
* **Синтаксис SPSS и командная шина:**
  * Генерация синтаксиса по кнопке «Вставить» (Paste) и прямое пакетное исполнение через `statcraft_run_syntax`:
    - `/BARCHART FREQ` и `/BARCHART PERCENT`
    - `/PIECHART FREQ` и `/PIECHART PERCENT`
    - `/HISTOGRAM` и `/HISTOGRAM NORMAL`
  * Сохранение 100% обратимости Undo/Redo для всех 41 процедур движка.

### 📈 Векторный рендеринг диаграмм в Output Viewer (`statcraft_app`)
* **100% Vector-First рендеринг через `ui.painter()`:**
  * Полное отсутствие сторонних веб-движков, растровых картинок и символов-эмодзи (защита от квадратов «тофу»).
  * **Столбчатые диаграммы (`freq_chart_bar_pie.rs`):** подписи категорий по оси $X$, шкала частот/процентов по оси $Y$, подсветка столбцов при наведении, значения над столбцами.
  * **Круговые диаграммы (`freq_chart_bar_pie.rs`):** векторные сектора круга, адаптивная палитра цветов из `Tokens` и боковая легенда с процентными долями.
  * **Гистограммы (`freq_chart_histogram.rs`):** смежные столбцы карманов, плавная наложенная кривая Гаусса акцентным цветом и аутентичная плашка статистик SPSS в правом верхнем углу (`Mean`, `Std. Dev.`, `N`).
* **Интеграция с деревом структуры (Outline):**
  * Специальная векторная иконка диаграммы (`paint_output_chart_icon`) в левой панели Output Viewer с плавной навигацией по клику.
  * Текстовое и табличное резюме при экспорте отчетов в форматы TSV и HTML.

### 🔬 Математический биннинг и масштабирование нормальной кривой (`statcraft_algo`)
* **Адаптивный алгоритм расчета карманов (`charts.rs`):**
  * Реализованы правила Скотта ($h = 3.49 \cdot s \cdot N^{-1/3}$) и Фридмана-Диакониса с округлением шага сетки до «красивых» круглых чисел ($1, 2, 2.5, 5, 10 \times 10^k$).
  * Точное масштабирование кривой плотности вероятности Гаусса к площади гистограммы: $y = \frac{N \cdot h}{\sigma \sqrt{2\pi}} \exp\left(-\frac{(x - \mu)^2}{2\sigma^2}\right)$.
  * Защита от сбоев и паник: обработка константных выборок ($s = 0$), малых выборок ($N < 2$), пропусков и предупреждения для строковых данных.
  * Сверка расчетов с эталонным оракулом SciPy в тестах `frequencies_charts_oracle_test.rs`.

### 📋 Полный журнал изменений (Changelog v0.4.5)
```markdown
## [0.4.5] - 2026-11-22

### Добавлено
- **Поддиалог «Частоты: Диаграммы» и паритет с SPSS (Feature 017):**
  - Поддиалог «Частоты: Диаграммы» ($360 \times 300$ px) на базе DialogKit с каноническими группами «Тип диаграммы» и «Значения на диаграмме».
  - Контекстная блокировка радиокнопок значений для гистограмм и чекбокса нормальной кривой.
  - Двунаправленный синтаксис SPSS: генерация и распознавание подкоманд `/BARCHART`, `/PIECHART`, `/HISTOGRAM [NORMAL]`.
- **Векторный рендеринг диаграмм в Output Viewer:**
  - 100% Vector-First отрисовка через `ui.painter()` столбчатых, круговых диаграмм и гистограмм с наложением кривой Гаусса.
  - Аутентичная плашка описательных статистик SPSS (`Mean`, `Std. Dev.`, `N`) на гистограмме.
  - Отображение графиков в дереве Outline с векторной иконкой диаграммы и навигацией по клику.
- **Математический биннинг и кривая Гаусса (L2 statcraft_algo):**
  - Адаптивный расчет карманов гистограмм (Scott / Freedman-Diaconis) с округлением шага сетки.
  - Точное масштабирование кривой нормального распределения Гаусса.
  - 100% прохождение тестов Golden Oracle против SciPy.

### Исправлено
- **Описательные статистики и процентили для номинальных числовых шкал (Feature 016):**
  - Устранено искусственное подавление расчетов описательных статистик и процентилей для номинальных числовых переменных.
  - Отображение аутентичных меток переменных (`Метка [Имя]`) в заголовках частотных таблиц и сводных статистик.
  - Синхронизация журнала исполненного синтаксиса в Output Viewer.
```

### 📦 Артефакты сборки
* **Windows (x64):** Установщик Inno Setup (`statcraft-windows-x64-installer.exe`) + Портативный ZIP-архив (`statcraft-windows-x64.zip`).
* **Linux (x64):** Пакет Debian (`statcraft_0.4.5_amd64.deb`) + Портативный TAR.GZ (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon ARM64 & Intel x64):** Образы дисков DMG + Пакеты приложений (`.app`) в TAR.GZ.

# StatCraft v0.2.2

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.2.2 publishes the comprehensive, exhaustive bilingual feature catalog documentation ([`docs/FEATURES.md`](docs/FEATURES.md) and [`docs/FEATURES_RU.md`](docs/FEATURES_RU.md)) covering all analytical, data manipulation, and transformation procedures implemented across the entire application, alongside version synchronization and automated release packaging.

### 📖 Comprehensive Feature Guide (`FEATURES.md` & `FEATURES_RU.md`)
* Published detailed, end-to-end documentation of every feature available in StatCraft v0.2.2:
  * **Dual-View Data Management:** Streaming CSV/TSV reader, Microsoft Excel reader (Calamine), Data View virtualized tabular grid with inline editing, row strike-through for filtered cases, and Variable View supporting all 9 canonical SPSS types, value labels, and missing value ranges.
  * **Data Manipulation Suite (Data Menu):** Multi-column stable case sorting (`Sort Cases`), conditional case filtering (`Select Cases`), subgroup stratification (`Split File`), and frequency/sampling case weighting (`Weight Cases`).
  * **Complete Transformation Suite (Transform Menu):** Variable computation (`Compute Variable` via RPN parser), in-place and target recoding (`Recode`), automatic integer recoding (`Automatic Recode`), lead/lag shifts (`Shift Values`), case value counting (`Count Values`), dummy variable generation (`Create Dummy Variables`), missing value imputation (`Replace Missing Values`), time series filters (`Create Time Series`), visual & optimal binning (`Visual Binning` & `Optimal Binning`), rank cases (`Rank Cases` with tie adjustments and normal scores), date/time wizard (`Date and Time Wizard`), automated data preparation (`ADP`), and programmability scripts (`Programmability Transformations`).
  * **Statistical Calculation Suite (Analyze Menu):** Descriptives, Frequencies, Crosstabs (with $\chi^2$, Fisher's exact, Likelihood Ratio, and Cramér's $V$), Means breakdown with ANOVA and $\eta^2$, One-Sample $t$-test, Independent-Samples $t$-test (with Levene's test and Welch's $t$), Paired-Samples $t$-test, One-Way ANOVA, Bivariate Correlations (Pearson $r$, Spearman $\rho$, Kendall $\tau_b$), Nonparametric tests (Mann-Whitney $U$, Kruskal-Wallis $H$, Wilcoxon Signed-Rank, and Sign test), Multiple Linear Regression (OLS/WLS with Collinearity Tolerance & VIF), Scale Reliability Analysis (Cronbach's $\alpha$ and Item-Total statistics), and Factor Analysis / PCA (with KMO, Bartlett's test, and Component Matrix).
  * **Output Viewer & Export:** Authentic multi-tier SPSS Pivot Tables with one-click TSV clipboard export for Excel, collapsible Outline navigation tree, and command syntax execution logging.
  * **SPSS Command Syntax & Agent Readiness:** Batch `.sps` syntax execution, script file loading/saving, non-destructive Undo/Redo (`Ctrl+Z` / `Ctrl+Y`), and typed parameter schemas `specs()` for AI agents.

### 📦 Multi-Platform Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive.
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.2.2 публикует исчерпывающее двуязычное руководство по всем возможностям статистического пакета ([`docs/FEATURES_RU.md`](docs/FEATURES_RU.md) и [`docs/FEATURES.md`](docs/FEATURES.md)), охватывающее все аналитические процедуры, алгоритмы манипуляций и трансформаций данных, реализованные в системе, а также синхронизирует версии сборки и релизные пакеты.

### 📖 Полный справочник возможностей (`FEATURES_RU.md` и `FEATURES.md`)
* Опубликовано детальное сквозное описание всех функциональных модулей StatCraft v0.2.2:
  * **Двухрежимное управление данными:** потоковый загрузчик CSV/TSV, импорт Excel (Calamine), виртуализированный грид Data View с инлайн-редактированием ячеек, перечеркиванием отфильтрованных строк, и Variable View с поддержкой всех 9 типов переменных SPSS, меток значений и диапазонов пропусков.
  * **Модуль манипуляций с данными (меню Data):** многоуровневая сортировка (`Sort Cases`), выборка и фильтрация по условиям (`Select Cases`), расщепление файла на подгруппы (`Split File`) и взвешивание наблюдений (`Weight Cases`).
  * **Полный блок преобразований данных (меню Transform):** вычисление переменных (`Compute Variable` через Shunting-yard RPN), перекодирование в те же и другие переменные (`Recode`), автоматическое перекодирование (`Automatic Recode`), сдвиг значений (`Shift Values`), подсчет значений в наблюдениях (`Count Values`), дамми-переменные (`Create Dummy Variables`), восполнение пропущенных значений (`Replace Missing Values`), временные ряды (`Create Time Series`), визуальная и оптимальная дискретизация (`Visual Binning` и `Optimal Binning`), ранжирование наблюдений (`Rank Cases`), мастер даты и времени (`Date and Time Wizard`), автоматическая подготовка данных (`ADP`) и скриптовые трансформации (`Programmability Transformations`).
  * **Блок статистических процедур (меню Analyze):** описательные статистики (`Descriptives`), частоты (`Frequencies`), таблицы сопряженности (`Crosstabs` с $\chi^2$, точным тестом Фишера, поправкой Йейтса и мерами $\phi$/$V$), сравнение средних (`Means Breakdown` с ANOVA и $\eta^2$), одновыборочный $t$-критерий, $t$-критерий для независимых выборок (с критерием Левена и Уэлча), парный $t$-критерий, однофакторный дисперсионный анализ (`One-Way ANOVA`), парные корреляции (Пирсон, Спирмен, Кендалл $\tau_b$), непараметрические критерии (Манн-Уитни $U$, Краскел-Уоллис $H$, знаковые ранги Вилкоксона и критерий знаков), множественная линейная регрессия (OLS/WLS с оценками мультиколлинеарности Tolerance и VIF), анализ надежности шкалы (альфа Кронбаха и статистики пунктов), факторный анализ и метод главных компонент (PCA с тестами KMO и Бартлетта).
  * **Модуль вывода и интеграция с Excel:** аутентичные сводные таблицы SPSS (Pivot Tables) с копированием в TSV в один клик без поломки колонок в Excel, интерактивное дерево оглавления Outline и журнал синтаксиса.
  * **Синтаксис SPSS и готовность к агентам:** исполнение синтаксиса `.sps`, сохранение и открытие скриптов на диске, кнопка `[ Paste ]` во всех диалогах, откат правок Undo/Redo (`Ctrl+Z`) и типизированные схемы параметров `specs()` для внешних скриптов и ИИ-ассистентов.

### 📦 Дистрибутивы для всех операционных систем
* **Windows (x64):** Инсталлятор Inno Setup (`.exe`) + Портативный ZIP-архив.
* **Linux (x64):** Пакет Debian (`.deb`) + Портативный архив TAR.GZ.
* **macOS (Apple Silicon ARM64 и Intel x64):** Образ диска DMG + Приложение (`.app`) в архиве TAR.GZ.

---

## Предыдущие релизы / Previous Releases

### StatCraft v0.2.1
* Доказательный Clean-Room математический аудит вычислительного ядра против *IBM SPSS Statistics 24 Algorithms Guide* (1271 стр.) с точностью «Золотого оракула» ($\le 10^{-6}$).
* Финализация блоков `adp` (Automated Data Preparation) и `transform.programmability`.

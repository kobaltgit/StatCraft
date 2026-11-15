# StatCraft v0.3.1

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.3.1 is a major statistical release that significantly expands the analytical capabilities of the platform to match professional econometrics and biostatistics suites. This release delivers **Package 1 (Advanced Regression & Partial Correlations)**, **Package 2 (General Linear Model: UNIANOVA, MANOVA, and Repeated Measures)**, and **Package 3 (Cluster Analysis, Reports, and Survival Analysis)**, alongside desktop polishing with silent Windows subsystem launch.

### 🔬 Advanced Regression & Partial Correlations (Feature 001)
* **Partial Correlations (`PARTIAL CORR` / `stat.correlations.partial`):**
  * Symmetrized Schur complement of the covariance matrix ($S = R_{XX} - R_{XZ} R_{ZZ}^{-1} R_{ZX}$);
  * Exact two-tailed and one-tailed $p$-values via Student's $t$-distribution ($df = N - k - 2$);
  * Zero-order correlation matrices with Listwise and Pairwise missing value exclusion;
  * Validated against Python `scipy.stats` ($10^{-12}$ precision).
* **Binary Logistic Regression (`LOGISTIC REGRESSION` / `stat.regression.logistic`):**
  * Iteratively Reweighted Least Squares (IRLS / Newton-Raphson) with adaptive Fisher information damping ($\lambda = 10^{-11} \dots 10^{-5}$);
  * Asymptotic standard errors, Wald $\chi^2$ statistics, and 95% confidence intervals for odds ratios $\text{Exp}(B)$;
  * Goodness-of-fit: Cox & Snell and Nagelkerke pseudo-$R^2$, Hosmer-Lemeshow decile test, and $2 \times 2$ classification table;
  * Validated against Python `statsmodels.Logit`.
* **Curve Estimation (`CURVEFIT` / `stat.regression.curve`):**
  * Simultaneous estimation of 6 canonical SPSS models: Linear, Logarithmic, Inverse, Quadratic, Power, and Exponential via analytical linearization;
  * Model summary ($R^2, F, df_1, df_2, p$) and parameter estimates table ($b_0, b_1, b_2$).

### 📐 General Linear Model (Feature 002)
* **Univariate ANOVA/ANCOVA (`UNIANOVA` / `stat.glm.univariate`):**
  * Orthogonal design matrix with deviation contrasts (sum-to-zero) and continuous covariates;
  * Type III Sum of Squares (Type III SS) via marginal model reduction and Jacobi SVD pseudo-inversion;
  * Tests of Between-Subjects Effects with $MS, F, p$, and Partial $\eta^2$;
  * Validated against Python `statsmodels.api.stats.anova_lm(typ=3)`.
* **Multivariate ANOVA (`GLM Multivariate` / `stat.glm.multivariate` — MANOVA):**
  * Multivariate extension for $p \ge 2$ dependent variables with Hypothesis ($H$) and Error ($E$) cross-product matrices;
  * Generalized eigenvalue decomposition of $E^{-1/2} H E^{-1/2}$;
  * 4 canonical test criteria: Wilks' $\Lambda$ (with Rao's $F$-approximation), Pillai's Trace, Hotelling-Lawley Trace, and Roy's Largest Root.
* **Repeated Measures ANOVA (`GLM Repeated Measures` / `stat.glm.repeated`):**
  * Within-subjects factor analysis using orthonormal Helmert contrast matrices ($k \times (k-1)$);
  * Mauchly's Test of Sphericity ($W, \text{approx. } \chi^2, df, p$);
  * Epsilon corrections: Greenhouse-Geisser ($\hat{\varepsilon}$), Huynh-Feldt ($\tilde{\varepsilon}$), and lower-bound ($1/(k-1)$) with adjusted degrees of freedom and $p$-values.

### 🧩 Cluster Analysis, Reports & Survival (Feature 003)
* **K-Means Cluster Analysis (`QUICK CLUSTER` / `stat.cluster.kmeans`):**
  * Deterministic initial center selection via Maximin (furthest-point heuristic);
  * Iterative reassignment cycle minimizing Euclidean distances;
  * Final cluster centers, cluster distances, and one-way ANOVA decomposition with $F$-tests and Fisher $p$-values;
  * **Dynamic Variable Generation with 100% Reversible Undo/Redo:** Dynamically appends `QCL_1` (Cluster Membership) and `QCL_2` (Distance from Center) to the active dataset with full transaction rollback.
* **Variable Codebook (`CODEBOOK` / `stat.reports.codebook`):**
  * Full dataset dictionary extraction: variable names, labels, formats, measurement levels;
  * Discrete frequency distributions with Counts, Percents, and Valid Percents;
  * Scale statistics: Mean, StdDev, Median, and Quartiles via SPSS HAVERAGE interpolation.
* **Case Summaries (`SUMMARIZE` / `stat.reports.summarize`):**
  * Row-by-row data listing with optional case limit truncation;
  * Hierarchical factor grouping with intermediate subtotals and grand totals ($N$, Mean, Sum, StdDev).
* **Kaplan-Meier Survival Analysis (`KM` / `stat.survival.km`):**
  * Product-limit survival function $\hat{S}(t) = \prod (1 - d_i/n_i)$;
  * Greenwood's standard error formula $\widehat{\text{Var}}(\hat{S}(t)) = \hat{S}(t)^2 \sum \frac{d_i}{n_i(n_i - d_i)}$;
  * Mean and median survival time with 95% confidence intervals;
  * Mantel-Cox Log-Rank $\chi^2$ test ($df = G - 1, p$) for strata comparison;
  * Validated against Python `lifelines.KaplanMeierFitter` ($\le 10^{-6}$ precision).
* **Actuarial Life Tables (`SURVIVAL` / `stat.survival.lifetables`):**
  * Actuarial interval boundaries $[t_k, t_{k+1})$;
  * Effective sample size under risk ($n'_k = n_k - w_k/2$), conditional mortality ($q_k$) and survival ($p_k$), cumulative survival ($P_k$), event probability density ($f_k$), and hazard rate ($h_k$).

### 🖥️ Windows Desktop Polish: Silent GUI Subsystem Launch
* Added `#![windows_subsystem = "windows"]` to the desktop binary entry point;
* The Windows executable compiles with PE Subsystem 2 (`IMAGE_SUBSYSTEM_WINDOWS_GUI`), completely eliminating the black console/terminal window when launching StatCraft from desktop shortcuts or File Explorer.

### 🤖 Model Context Protocol (MCP) & Syntax Integration
* All new statistical procedures are fully integrated into the StatCraft MCP Bridge:
  * Syntax execution (`statcraft_run_syntax`) parses `QUICK CLUSTER`, `CODEBOOK`, `SUMMARIZE`, `KM`, `SURVIVAL`, `UNIANOVA`, `GLM`, `PARTIAL CORR`, `LOGISTIC REGRESSION`, `CURVEFIT`;
  * Direct command invocation (`statcraft_execute_command`) supports all registered engine command IDs;
  * Live introspection (`statcraft_list_commands` and `statcraft://commands`) dynamically exposes schemas and parameter specs to AI assistants.

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive (both launch cleanly without console window).
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.3.1 — это масштабное расширение математических и аналитических возможностей платформы, выводящее StatCraft на уровень профессиональных пакетов эконометрики и биостатистики. В релиз вошли **Пакет 1 (Продвинутая регрессия и частные корреляции)**, **Пакет 2 (Общая линейная модель: UNIANOVA, MANOVA и повторные измерения)**, **Пакет 3 (Кластерный анализ, отчёты и анализ выживаемости)**, а также устранение терминального окна при запуске приложения в Windows.

### 🔬 Продвинутая регрессия и частные корреляции (Feature 001)
* **Частные корреляции (`PARTIAL CORR` / `stat.correlations.partial`):**
  * Симметризованное дополнение Шура для субковариационной матрицы ($S = R_{XX} - R_{XZ} R_{ZZ}^{-1} R_{ZX}$);
  * Точные двусторонние и односторонние $p$-значения через распределение Стьюдента ($df = N - k - 2$);
  * Матрицы нулевого порядка с режимами исключения пропусков Listwise и Pairwise;
  * Верифицировано против Python `scipy.stats` с точностью $10^{-12}$.
* **Бинарная логистическая регрессия (`LOGISTIC REGRESSION` / `stat.regression.logistic`):**
  * Оптимизатор IRLS (Newton-Raphson) с адаптивным диагональным демпфированием матрицы Фишера ($\lambda = 10^{-11} \dots 10^{-5}$);
  * Асимптотические стандартные ошибки, статистика Вальда, 95% доверительные интервалы для отношений шансов $\text{Exp}(B)$;
  * Меры согласия: псевдо-$R^2$ Кокса-Снелла и Нагелькерке, критерий Хосмера-Лемешова по децилям риска и матрица классификации $2 \times 2$;
  * Верифицировано против Python `statsmodels.Logit`.
* **Оценка канонических кривых (`CURVEFIT` / `stat.regression.curve`):**
  * Одновременная оценка 6 канонических моделей: линейной, логарифмической, обратной, квадратичной, степенной и экспоненциальной;
  * Сводка модели ($R^2, F, df_1, df_2, p$) и таблица оценок параметров ($b_0, b_1, b_2$).

### 📐 Общая линейная модель (Feature 002)
* **Одномерный дисперсионный/ковариационный анализ (`UNIANOVA` / `stat.glm.univariate`):**
  * Ортогонализованная матрица плана с кодированием факторов контрастами отклонений (deviation contrasts, sum-to-zero) и ковариатами;
  * Суммы квадратов типа III (Type III SS) через маргинальную редукцию моделей и псевдообращение Jacobi SVD;
  * Таблица Between-Subjects Effects с $MS, F, p$ и частной эта-квадрат ($\text{Partial } \eta^2$);
  * Верифицировано против Python `statsmodels.api.stats.anova_lm(typ=3)`.
* **Многомерный дисперсионный анализ (`GLM Multivariate` / `stat.glm.multivariate` — MANOVA):**
  * Многомерное обобщение для $p \ge 2$ зависимых переменных с матрицами сумм квадратов и векторных произведений гипотез ($H$) и ошибки ($E$);
  * Обобщенное спектральное разложение $E^{-1/2} H E^{-1/2}$;
  * 4 канонических критерия: Wilks' $\Lambda$ (с $F$-аппроксимацией Рао), Pillai's Trace, Hotelling-Lawley Trace и Roy's Largest Root.
* **Анализ повторных измерений (`GLM Repeated Measures` / `stat.glm.repeated`):**
  * Внутригрупповой фактор с ортонормированной матрицей контрастов Хельмерта ($k \times (k-1)$);
  * Критерий сферичности Мокли ($W, \text{прибл. } \chi^2, df, p$);
  * Эпсилон-поправки Гринхауса-Гейссера ($\hat{\varepsilon}$), Хюйнха-Фельдта ($\tilde{\varepsilon}$) и нижней границы ($1/(k-1)$) с пересчетом степеней свободы и $p$-значений.

### 🧩 Кластерный анализ, отчёты и анализ выживаемости (Feature 003)
* **Кластерный анализ методом K-средних (`QUICK CLUSTER` / `stat.cluster.kmeans`):**
  * Детерминированная инициализация центров методом максимального разброса (Maximin / furthest point heuristic);
  * Итеративный цикл пересчета центров и переназначения по евклидову расстоянию;
  * Финальные центры, расстояния между кластерами и однофакторная ANOVA для каждого признака с $F$-критериями и $p$-значениями Фишера;
  * **Динамическая генерация переменных и 100% обратимость Undo/Redo:** Динамически создает колонки `QCL_1` (номер кластера) и `QCL_2` (расстояние до центра) в активном датасете с полным откатом транзакции при отмене.
* **Кодовая книга словаря датасета (`CODEBOOK` / `stat.reports.codebook`):**
  * Полное извлечение паспорта словаря: имена, метки, форматы, шкалы измерений;
  * Частотные таблицы со стандартными и валидными процентами для категориальных шкал;
  * Дескрипторы для количественных шкал: Mean, StdDev, Median и квартили по стандарту SPSS HAVERAGE.
* **Итоги по наблюдениям (`SUMMARIZE` / `stat.reports.summarize`):**
  * Построчный листинг данных с опциональным отсечением лимита первых $N$ строк;
  * Иерархическая группировка по факторам с расчетом подитогов и общего итога ($N$, Mean, Sum, StdDev).
* **Анализ выживаемости Каплана-Мейера (`KM` / `stat.survival.km`):**
  * Множительная оценка функции дожития $\hat{S}(t) = \prod (1 - d_i/n_i)$;
  * Стандартная ошибка по формуле Гринвуда $\widehat{\text{Var}}(\hat{S}(t)) = \hat{S}(t)^2 \sum \frac{d_i}{n_i(n_i - d_i)}$;
  * Среднее и медианное время дожития с 95% доверительными интервалами;
  * Лог-ранговый тест Мантеля-Кокса ($\chi^2, df = G - 1, p$) для сравнения страт;
  * Верифицировано против Python `lifelines.KaplanMeierFitter` ($\le 10^{-6}$).
* **Актуарные таблицы смертности / дожития (`SURVIVAL` / `stat.survival.lifetables`):**
  * Актуарный расчет по заданным временным интервалам $[t_k, t_{k+1})$;
  * Эффективный объем под риском ($n'_k = n_k - w_k/2$), условная смертность ($q_k$) и выживаемость ($p_k$), кумулятивная вероятность дожития ($P_k$), плотность вероятности события ($f_k$) и коэффициент риска ($h_k$).

### 🖥️ Windows: Чистый запуск графического окна без консоли
* Входная точка приложения снабжена атрибутом `#![windows_subsystem = "windows"]`;
* Исполняемый файл для Windows компилируется с флагом PE Subsystem 2 (`IMAGE_SUBSYSTEM_WINDOWS_GUI`), исключая появление чёрного консольного окна при запуске StatCraft из проводника или с ярлыка.

### 🤖 Интеграция с Model Context Protocol (MCP) и синтаксисом
* Все новые аналитические процедуры бесшовно доступны ИИ-агентам:
  * Выполнение синтаксиса (`statcraft_run_syntax`) поддерживает синтаксические команды `QUICK CLUSTER`, `CODEBOOK`, `SUMMARIZE`, `KM`, `SURVIVAL`, `UNIANOVA`, `GLM`, `PARTIAL CORR`, `LOGISTIC REGRESSION`, `CURVEFIT`;
  * Прямой вызов (`statcraft_execute_command`) принимает структурированный JSON параметров для любого зарегистрированного ID;
  * Интроспекция (`statcraft_list_commands` и `statcraft://commands`) динамически транслирует схемы и параметры в контекст ИИ-ассистентов.

### 📦 Мультиплатформенные установочные пакеты
* **Windows (x64):** Установщик Inno Setup (`.exe`) + Портативный ZIP-архив (оба запускаются без открытия консоли).
* **Linux (x64):** Пакет Debian (`.deb`) + Портативный TAR.GZ-архив.
* **macOS (Apple Silicon ARM64 & Intel x64):** Образ диска Apple DMG + Бандл приложения (`.app`) TAR.GZ.

# StatCraft v0.2.1

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.2.1 delivers an evidence-backed Clean-Room mathematical audit of our statistical calculation engine against the canonical *IBM SPSS Statistics 24 Algorithms Guide*, alongside complete production activation of the **Transform** procedure suite.

### 🔬 Clean-Room Mathematical Audit (Craft Anything)
All statistical algorithms implemented in `statcraft_algo` have undergone a page-by-page verification against the official *IBM SPSS Statistics 24 Algorithms Guide* (1,271 pp., IBM Corporation) using the open-source **[Craft Anything (`storytold/craft-anything`)](https://github.com/storytold/craft-anything)** verification engine operating in `clean-room: true` mode.

Every mathematical formula, degrees-of-freedom rule, tie-correction variance, and significance level generator was confirmed to achieve **100% mathematical equivalence** under our Golden Oracle precision threshold ($\le 10^{-6}$):
1. **Descriptives, Frequencies, Crosstabs & Means:** Spicer (1972) provisional moments, Bliss sample excess kurtosis, HAVERAGE quartiles ($r = p(N+1)$), Yates continuity correction, Pearson $\chi^2$, Fisher's exact test, and ANOVA $\eta^2$.
2. **Mean Comparisons & ANOVA:** Mean-centered Levene test of variance equality, Student's pooled $t$-test, Welch's separate variance $t$-test with Satterthwaite fractional $df$, paired-samples $t$-test, and One-Way ANOVA.
3. **Correlations & Multivariates:** Bivariate Pearson $r$, Spearman $\rho$, Kendall $\tau_b$ with full tie corrections ($v_0, v_1, v_2$), OLS/WLS multiple regression with Tolerance & VIF diagnostics, cyclic Jacobi eigensolver for PCA, Bartlett's test of sphericity, Kaiser-Meyer-Olkin (KMO) measure, and Cronbach's $\alpha$.
4. **Nonparametric Tests:** Mann-Whitney $U$, Wilcoxon rank sum $W$, Kruskal-Wallis $H_{adj}$ with tie factor, Wilcoxon signed-rank test with tie-corrected variance, and exact binomial Sign test.
5. **Data Transformations & Imputations:** Blom, Tukey, Rankit, and Van der Waerden normal scores via Peter J. Acklam's rational approximation ($\le 1.15 \times 10^{-9}$ error), MDLP entropy discretization (Fayyad & Irani), and missing value imputation (linear interpolation, series mean, mean/median of nearby points, linear trend).

Comprehensive bilingual audit records are available in:
* Primary English Guide: [`docs/SPSS_ALGORITHMS_AUDIT.md`](docs/SPSS_ALGORITHMS_AUDIT.md)
* Russian Mirror: [`docs/SPSS_ALGORITHMS_AUDIT_RU.md`](docs/SPSS_ALGORITHMS_AUDIT_RU.md)

### ⚙️ Transform Suite Finalization
* **Automated Data Preparation (`adp`):** Supervised feature screening, optimal binning, and cross-validation partition breakdown with 4 authentic SPSS pivot tables.
* **Programmability Transformations (`transform.programmability`):** Transactional script engine for custom variable computations and conditional rules (`IF ... THEN ... ELSE`), syntax generator (`BEGIN PROGRAM ... END PROGRAM.`), and real-time vector syntax verification.
* All `[dev]` badges removed from the **Transform** menu; 100% of procedures now fully operational with DialogKit modal interfaces and Output Viewer pivot tables.

### 📦 Multi-Platform Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive.
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.2.1 представляет результаты доказательного Clean-Room математического аудита вычислительного ядра на полное соответствие каноническому руководству *IBM SPSS Statistics 24 Algorithms Guide*, а также завершает ввод в эксплуатацию всех процедур раздела **«Преобразование»** (`Transform`).

### 🔬 Инструментальный математический аудит Clean-Room (Craft Anything)
Все статистические алгоритмы, реализованные в ядре `statcraft_algo`, прошли постраничную сверку с официальным математическим руководством *IBM SPSS Statistics 24 Algorithms Guide* (1271 стр., IBM Corporation) с помощью открытого движка **[Craft Anything (`storytold/craft-anything`)](https://github.com/storytold/craft-anything)** в режиме `clean-room: true`.

Для всех формул, расчетов степеней свободы, поправок на связки рангов и уровней значимости подтверждена **100% математическая эквивалентность** при соблюдении точности «Золотого оракула» ($\le 10^{-6}$):
1. **Описательные статистики, частоты, таблицы сопряженности и средние:** рекуррентные моменты Спайсера (1972), эксцесс Блисса, квартили HAVERAGE ($r = p(N+1)$), поправка Йейтса на непрерывность, критерий Пирсона $\chi^2$, точный тест Фишера, ANOVA $\eta^2$.
2. **Сравнение средних и ANOVA:** центрированный по среднему критерий Левена, критерий Стьюдента, критерий Уэлча с раздельными дисперсиями и дробными степенями свободы Саттертуэйта, парный $t$-критерий, однофакторный дисперсионный анализ.
3. **Корреляции и многомерные методы:** парный Пирсон, ранговый Спирмен, Кендалл $\tau_b$ со всеми тремя поправками на связки ($v_0, v_1, v_2$), множественная регрессия OLS/WLS с диагностикой мультиколлинеарности (Tolerance, VIF), циклический метод Якоби для PCA, тест сферичности Бартлетта, индекс KMO, коэффициент надежности $\alpha$ Кронбаха.
4. **Непараметрические критерии:** Манн-Уитни $U$, Уилкоксон $W$, Краскел-Уоллис $H_{adj}$ с коэффициентом связок, знаково-ранговый критерий Уилкоксона со скорректированной дисперсией, точный биномиальный критерий знаков.
5. **Трансформации данных и восполнение пропусков:** нормальные метки Блома, Тьюки, Ранкита, Ван дер Вардена через аппроксимацию Питера Дж. Акклама (погрешность $\le 1.15 \times 10^{-9}$), дискретизация MDLP по информационной энтропии Шеннона (Файяд и Ирани), методы восполнения пропусков (линейная интерполяция, среднее ряда, среднее/медиана окрестности, линейный тренд).

Полная двуязычная документация аудита доступна в репозитории:
* Основной документ на английском языке: [`docs/SPSS_ALGORITHMS_AUDIT.md`](docs/SPSS_ALGORITHMS_AUDIT.md)
* Русскоязычная версия: [`docs/SPSS_ALGORITHMS_AUDIT_RU.md`](docs/SPSS_ALGORITHMS_AUDIT_RU.md)

### ⚙️ Финализация процедур преобразования данных (`Transform`)
* **Автоматическая подготовка данных (`adp`):** отбор предикторов, оптимальная дискретизация и разбиение на выборки с 4 аутентичными сводными таблицами вывода SPSS.
* **Программируемые преобразования (`transform.programmability`):** транзакционный скриптовый движок пользовательских правил (`IF ... THEN ... ELSE`), генерация синтаксиса `BEGIN PROGRAM ... END PROGRAM.` и векторная проверка синтаксиса на лету.
* Сняты все служебные бейджи `[dev]` в меню «Преобразование»; 100% процедур оснащены интерфейсами DialogKit и сводными таблицами Output Viewer.

### 📦 Дистрибутивы для всех операционных систем
* **Windows (x64):** Инсталлятор Inno Setup (`.exe`) + Портативный ZIP-архив.
* **Linux (x64):** Пакет Debian (`.deb`) + Портативный архив TAR.GZ.
* **macOS (Apple Silicon ARM64 и Intel x64):** Образ диска DMG + Приложение (`.app`) в архиве TAR.GZ.

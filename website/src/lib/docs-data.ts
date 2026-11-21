export interface DocSection {
  slug: string;
  titleEn: string;
  titleRu: string;
  descriptionEn: string;
  descriptionRu: string;
  category: "getting-started" | "data-management" | "procedures" | "audit" | "agent-lab";
  categoryTitleEn: string;
  categoryTitleRu: string;
  oracleStatus?: string;
  spssPage?: string;
  contentEn: string;
  contentRu: string;
  syntaxExample?: string;
  toc: { id: string; titleEn: string; titleRu: string }[];
}

export const DOC_SECTIONS: DocSection[] = [
  {
    slug: "experiment-manifest",
    category: "getting-started",
    categoryTitleEn: "Getting Started",
    categoryTitleRu: "Начало работы",
    titleEn: "The Research Manifest",
    titleRu: "Манифест эксперимента",
    descriptionEn: "Why StatCraft is built as an open exploration in memory-safe statistical computing.",
    descriptionRu: "Почему StatCraft создается как открытый исследовательский эксперимент.",
    toc: [
      { id: "premise", titleEn: "1. The Premise", titleRu: "1. Идея эксперимента" },
      { id: "memory-vs-correctness", titleEn: "2. Memory Safety vs. Statistical Correctness", titleRu: "2. Безопасность памяти vs. Корректность" },
      { id: "community-validation", titleEn: "3. Invitation to Academic Validation", titleRu: "3. Призыв к валидации" },
    ],
    contentEn: `### 1. The Premise
Statistical software powers critical clinical, economic, and social decisions. Yet legacy statistical packages written decades ago in C/C++ remain susceptible to memory vulnerabilities, obscure crashes on malformed data, and steep commercial license fees that burden public universities.

StatCraft investigates a core hypothesis:
**Can a complete statistical suite be rewritten in 100% pure Safe Rust with zero \`unsafe\` blocks, zero panics, and native Model Context Protocol (MCP) integration without sacrificing speed or SPSS analytical fidelity?**

### 2. Memory Safety vs. Statistical Correctness
As highlighted by independent technical reviews (including remio.ai's analysis *"StatCraft Rust Rewrite Targets IBM SPSS With a Different Safety Promise"*):
> **Language memory safety does NOT automatically guarantee statistical correctness.**

A program can avoid \`SIGSEGV\` while silently producing flawed degrees of freedom or misinterpreting missing value codes. This is why StatCraft enforces the **Golden Oracle Invariant**: every statistical procedure is formally audited and verified against Python \`scipy.stats\`, \`statsmodels\`, and the *IBM SPSS Statistics 24 Algorithms Guide* within an absolute error tolerance of $\\le 10^{-6}$.

### 3. Invitation to Academic Validation
StatCraft is an active research experiment (v0.4.x). We explicitly invite academic researchers, professors, and data scientists to challenge our numerical precision:
* Run real-world \`.sav\` and \`.csv\` datasets through StatCraft;
* Cross-check $p$-values and test statistics against SPSS, R, and Stata;
* Report any discrepancies via our anonymous submission engine without needing a GitHub account.`,
    contentRu: `### 1. Идея эксперимента
Статистический софт лежит в основе важнейших медицинских, экономических и социологических исследований. Однако проприетарные легаси-пакеты, написанные десятилетия назад на C/C++, подвержены сбоям памяти, зависаниям на нестандартных данных и требуют неподъемных лицензионных платежей от университетов.

StatCraft исследует фундаментальную гипотезу:
**Можно ли воссоздать классический статистический рабочий процесс на 100% безопасном Rust (без блоков \`unsafe\`, без паник и сбоев) и объединить его с современными ИИ-агентами через Model Context Protocol?**

### 2. Безопасность памяти vs. Статистическая корректность
Как справедливо подчеркивают независимые исследователи (включая разбор издания remio.ai):
> **Безопасность памяти на уровне языка НЕ РАВНА математической истинности.**

Программа может безупречно управлять памятью, но выдать неверные степени свободы или неправильно учесть пропуски данных. Именно поэтому в StatCraft действует железное правило **«Золотого оракула»**: каждая процедура проходит построчный математический аудит со сверкой против SciPy, Statsmodels и канона *IBM SPSS Statistics 24 Algorithms Guide* с абсолютной погрешностью $\\le 10^{-6}$.

### 3. Призыв к академической валидации
StatCraft — это развивающийся научный эксперимент (v0.4.x). Мы приглашаем исследователей, преподавателей и дата-сайентистов проверить наши расчеты:
* Прогоняйте реальные массивы данных (\`.sav\`, \`.csv\`);
* Сверяйте $p$-значения и таблицы вывода с SPSS и R;
* Сообщайте о любых расхождениях через форму обратной связи прямо на сайте без регистрации на GitHub.`,
  },
  {
    slug: "independent-t-test",
    category: "procedures",
    categoryTitleEn: "Statistical Procedures",
    categoryTitleRu: "Статистические процедуры",
    titleEn: "Independent-Samples T-Test",
    titleRu: "Т-критерий для независимых выборок",
    descriptionEn: "Comparison of means between two independent groups with Levene's test and Satterthwaite degrees of freedom.",
    descriptionRu: "Сравнение средних двух независимых групп с тестом Левена и степенями свободы Саттертуэйта.",
    oracleStatus: "Verified vs SciPy (≤ 10⁻⁶)",
    spssPage: "IBM SPSS Algorithms pp. 1063–1065",
    syntaxExample: `T-TEST GROUPS=gender(1 2)
  /MISSING=ANALYSIS
  /VARIABLES=salary
  /CRITERIA=CI(.95).`,
    toc: [
      { id: "overview", titleEn: "1. Overview", titleRu: "1. Обзор процедуры" },
      { id: "math-formulas", titleEn: "2. Mathematical Specification", titleRu: "2. Математические формулы" },
      { id: "levene-test", titleEn: "3. Levene's Test for Homogeneity", titleRu: "3. Критерий равенства дисперсий Левена" },
      { id: "syntax", titleEn: "4. SPSS Syntax Equivalence", titleRu: "4. Синтаксис SPSS (.sps)" },
    ],
    contentEn: `### 1. Overview
The Independent-Samples T-Test procedure tests the significance of the difference between the means of two independent groups. StatCraft automatically produces a canonical two-row SPSS Pivot Table containing both:
1. **Equal variances assumed** (Student's t-test with pooled variance);
2. **Equal variances not assumed** (Welch-Satterthwaite test with adjusted degrees of freedom).

### 2. Mathematical Specification
Let sample 1 have size $n_1$, mean $\\bar{X}_1$, and variance $s_1^2$, and sample 2 have size $n_2$, mean $\\bar{X}_2$, and variance $s_2^2$.

The Welch t-statistic is:
$$t = \\frac{\\bar{X}_1 - \\bar{X}_2}{\\sqrt{\\frac{s_1^2}{n_1} + \\frac{s_2^2}{n_2}}}$$

The Satterthwaite degrees of freedom are computed as:
$$df = \\frac{\\left(\\frac{s_1^2}{n_1} + \\frac{s_2^2}{n_2}\\right)^2}{\\frac{(s_1^2 / n_1)^2}{n_1 - 1} + \\frac{(s_2^2 / n_2)^2}{n_2 - 1}}$$

### 3. Levene's Test for Homogeneity
Before evaluating the t-statistic, StatCraft evaluates Levene's test of homogeneity of variance. Absolute deviations from group means $z_{ij} = |x_{ij} - \\bar{x}_i|$ are subjected to an analysis of variance $F$-test.`,
    contentRu: `### 1. Обзор процедуры
Процедура Т-критерия для независимых выборок проверяет гипотезу о равенстве средних двух независимых групп. В аутентичной сводной таблице вывода StatCraft формирует параллельно две строки:
1. **Предполагается равенство дисперсий** (классический критерий Стьюдента со взвешенной объединенной дисперсией);
2. **Не предполагается равенство дисперсий** (критерий Уэлча с поправкой степеней свободы Саттертуэйта).

### 2. Математические формулы
Пусть выборка 1 имеет объем $n_1$, среднее $\\bar{X}_1$ и дисперсию $s_1^2$, а выборка 2 — $n_2$, среднее $\\bar{X}_2$ и дисперсию $s_2^2$.

Статистика критерия Уэлча:
$$t = \\frac{\\bar{X}_1 - \\bar{X}_2}{\\sqrt{\\frac{s_1^2}{n_1} + \\frac{s_2^2}{n_2}}}$$

Степени свободы Саттертуэйта (Welch-Satterthwaite df):
$$df = \\frac{\\left(\\frac{s_1^2}{n_1} + \\frac{s_2^2}{n_2}\\right)^2}{\\frac{(s_1^2 / n_1)^2}{n_1 - 1} + \\frac{(s_2^2 / n_2)^2}{n_2 - 1}}$$

### 3. Критерий равенства дисперсий Левена
Для проверки гипотезы о гомоскедастичности $H_0: \\sigma_1^2 = \\sigma_2^2$ вычисляются абсолютные отклонения $z_{ij} = |x_{ij} - \\bar{x}_i|$ от групповых средних, к которым применяется однофакторный дисперсионный $F$-критерий.`,
  },
  {
    slug: "open-math-audit",
    category: "audit",
    categoryTitleEn: "Open Math Audit",
    categoryTitleRu: "Открытый математический аудит",
    titleEn: "20-Block Verification Protocol",
    titleRu: "20 блоков доказательного аудита",
    descriptionEn: "Complete specification of all 20 verified computational and analytical blocks.",
    descriptionRu: "Исчерпывающая спецификация всех 20 проверенных блоков математического ядра.",
    oracleStatus: "100% Golden Oracle Verified (≤ 10⁻⁶)",
    spssPage: "IBM SPSS Statistics 24 Algorithms (1,271 pp.)",
    toc: [
      { id: "audit-matrix", titleEn: "1. Audit Progress Matrix", titleRu: "1. Сводная матрица аудита" },
      { id: "clean-room-method", titleEn: "2. Clean-Room Methodology", titleRu: "2. Методология чистой комнаты" },
      { id: "golden-oracle", titleEn: "3. Golden Oracle Pipeline", titleRu: "3. Пайплайн Золотого оракула" },
    ],
    contentEn: `### 1. Audit Progress Matrix
All 20 mathematical, transformation, I/O, and reporting blocks of StatCraft have undergone rigorous, line-by-line Clean-Room auditing against the official *IBM SPSS Statistics 24 Algorithms Guide* (1,271 pp.):
* **Block 1–4:** Basic Statistics, Mean Comparisons, Correlations & Nonparametric Tests;
* **Block 5–12:** Transformations, Automated Data Preparation, Expression Evaluator (Kleene 3VL), Time Series, Date Wizard, Recoding & Subsetting;
* **Block 13–15:** I/O Codecs (.sav/.zsav, Excel, CSV), Pivot Tables & Exploratory Data Analysis (Shapiro-Wilk Royston, Lilliefors KS);
* **Block 16–20:** Advanced Regression (Partial Corr, Binary Logistic, Curvefit), General Linear Model (UNIANOVA, MANOVA, Repeated Measures), Clustering, Multinomial Logistic (NOMREG) and Cox Proportional Hazards (COXREG).

### 2. Clean-Room Methodology
In accordance with Rule 1.3 of our project charter:
* Proprietary IBM SPSS binaries are never disassembled, decompiled, or inspected;
* All behaviors and formulas are derived strictly from open academic publications and public algorithm guides;
* Automatic regression suites run under strict thread restrictions (\`-j 1\`) with Golden Oracle tolerances $\\le 10^{-6}$.`,
    contentRu: `### 1. Сводная матрица аудита
Все 20 ключевых блоков математического ядра, преобразований данных, кодеков и сводных отчетов StatCraft прошли исчерпывающий инструментальный Clean-Room аудит против руководства *IBM SPSS Statistics 24 Algorithms Guide* (1271 стр.):
* **Блоки 1–4:** Базовые дескриптивные статистики, сравнение средних, парные корреляции и непараметрика;
* **Блоки 5–12:** Трансформации данных, ADP, оценщик выражений (логика Клини 3VL), временные ряды, мастер дат, перекодирование и фильтрация;
* **Блоки 13–15:** Кодеки ввода-вывода (.sav/.zsav, Excel, CSV), сводные Pivot Tables и эксплораторный анализ (Шапиро-Уилк по Ройстону, Колмогоров-Смирнов / Лиллиефорс);
* **Блоки 16–20:** Продвинутая регрессия (частные корреляции, бинарная логистика, подгонка кривых), общая линейная модель GLM (UNIANOVA, MANOVA, повторные измерения), кластеризация, мультиномиальная логистика (NOMREG) и регрессия Кокса (COXREG).

### 2. Методология чистой комнаты (Clean-Room)
В строгом соответствии с каноном экосистемы:
* Проприетарные бинарники IBM SPSS никогда не декомпилируются;
* Все формулы воспроизводятся по открытым академическим источникам;
* Регрессионные тесты проверяются против SciPy и Statsmodels с допуском $\\le 10^{-6}$.`,
  },
  {
    slug: "mcp-agent-bridge",
    category: "agent-lab",
    categoryTitleEn: "Agent Lab & MCP",
    categoryTitleRu: "ИИ-лаборатория и MCP",
    titleEn: "Model Context Protocol & IPC Socket",
    titleRu: "Протокол MCP и локальный IPC-сокет",
    descriptionEn: "Complete specification for connecting Claude Code, Antigravity, and local LLMs to StatCraft.",
    descriptionRu: "Исчерпывающая спецификация подключения Claude Code, Antigravity и локальных LLM к StatCraft.",
    oracleStatus: "MCP 1.0 JSON-RPC Specification",
    spssPage: "Crates: statcraft_bridge / tokio async socket",
    syntaxExample: `{
  "method": "statcraft.procedure.execute",
  "params": {
    "command": "DESCRIPTIVES VARIABLES=salary age /STATISTICS=MEAN STDDEV MIN MAX."
  }
}`,
    toc: [
      { id: "ipc-architecture", titleEn: "1. Local IPC Architecture", titleRu: "1. Архитектура локального сокета" },
      { id: "mcp-tools", titleEn: "2. MCP Toolset Specification", titleRu: "2. Спецификация инструментов MCP" },
      { id: "output-injection", titleEn: "3. Direct Output Injection", titleRu: "3. Вставка результатов в Output Viewer" },
    ],
    contentEn: `### 1. Local IPC Architecture
When StatCraft is launched, the \`statcraft_bridge\` module spins up an asynchronous JSON-RPC 2.0 server bound to localhost port \`18492\`.
This allows local AI assistants to establish a secure, zero-cloud connection to the running dataset session without exposing sensitive patient or research data over the public internet.

### 2. MCP Toolset Specification
External agents utilize the standard Model Context Protocol (MCP) to interact with StatCraft:
* \`statcraft_get_dataset_summary\`: Returns row/column dimensions, column names, storage types, and missing value counts.
* \`statcraft_audit_variables\`: Returns variable dictionary metadata (Measurement scale, labels, categorical value-label mappings).
* \`statcraft_execute_syntax\`: Submits SPSS \`.sps\` syntax to the engine's \`CommandBus\`.
* \`statcraft_get_procedure_results\`: Retrieves generated Pivot Tables in JSON or LaTeX format.
* \`statcraft_insert_output\`: Appends APA-formatted interpretations and tables directly into the desktop Output tree.

### 3. Direct Output Injection
Unlike generic chatbots where analytical results remain trapped in the chat sidebar, StatCraft AI integrations write directly into the official scientific report tree, creating persistent, printable, and exportable research artifacts.`,
    contentRu: `### 1. Архитектура локального сокета
При запуске StatCraft модуль \`statcraft_bridge\` инициализирует асинхронный сервер JSON-RPC 2.0 на порту \`18492\` (localhost).
Это позволяет локальным ИИ-ассистентам безопасно взаимодействовать с текущей сессией данных без передачи конфиденциальной информации в публичные облака.

### 2. Спецификация инструментов MCP
Внешние агенты используют стандарт Model Context Protocol (MCP) со следующим набором инструментов:
* \`statcraft_get_dataset_summary\`: Возвращает размерность массива, типы колонок и число пропусков.
* \`statcraft_audit_variables\`: Возвращает словарь переменных (шкалы измерения, текстовые метки, таблицы кодировок).
* \`statcraft_execute_syntax\`: Отправляет синтаксис SPSS (\`.sps\`) на исполнение в шину \`CommandBus\`.
* \`statcraft_get_procedure_results\`: Получает сводные таблицы Pivot Tables в форматах JSON или LaTeX.
* \`statcraft_insert_output\`: Вставляет научные интерпретации по стандартам APA прямо в дерево отчета программы.

### 3. Вставка результатов в Output Viewer
В отличие от обычных чат-ботов, где результаты теряются в ленте диалога, интеграция StatCraft выводит результаты прямо в итоговый научный отчет пользователя, доступный для печати и экспорта в PDF/DOCX.`,
  },
  {
    slug: "safe-rust-architecture",
    category: "getting-started",
    categoryTitleEn: "Getting Started",
    categoryTitleRu: "Начало работы",
    titleEn: "100% Safe Rust & Arrow Engine",
    titleRu: "100% Safe Rust и движок Arrow",
    descriptionEn: "Memory safety guarantees, layered crate boundaries, and zero-panic error handling.",
    descriptionRu: "Гарантии безопасности памяти, строгая иерархия слоев и отказ от аварийных паник.",
    oracleStatus: "Compile-Time Enforced (cargo xtask layers)",
    spssPage: "Crates: statcraft_core, statcraft_algo",
    toc: [
      { id: "zero-panic", titleEn: "1. The Zero-Panic Contract", titleRu: "1. Контракт нулевых паник" },
      { id: "arrow-polars", titleEn: "2. Apache Arrow Memory Model", titleRu: "2. Модель памяти Apache Arrow" },
      { id: "layered-crates", titleEn: "3. Layer Hierarchy (L0–L6)", titleRu: "3. Иерархия слоев (L0–L6)" },
    ],
    contentEn: `### 1. The Zero-Panic Contract
StatCraft strictly bans the following constructs in production code:
\`unwrap()\`, \`expect(...)\`, \`panic!(...)\`, \`unreachable!(...)\`, \`todo!()\`, and \`unsafe\`.
All potential failures—such as singular covariance matrices, zero variances, or division by zero—are captured as typed \`Result<T, StatError>\` variants and presented gracefully in the user interface.

### 2. Apache Arrow Memory Model
Instead of allocating separate heap structures for individual data cells, StatCraft represents variables as continuous, contiguous Apache Arrow arrays:
* **SIMD Vectorization**: Vector operations run across CPU registers without pointer chasing;
* **Polars Acceleration**: Fast multi-threaded group-by, filtering, and aggregation;
* **Zero GC**: Deterministic memory deallocation without garbage collector stutter.

### 3. Layer Hierarchy (L0–L6)
The repository enforces a strict unidirectional dependency hierarchy checked via \`cargo xtask layers\`:
* L0: \`statcraft_io\` (Codecs)
* L1: \`statcraft_core\` (DataSets & Dictionaries)
* L2: \`statcraft_algo\` (Numerical Algorithms)
* L3: \`statcraft_charts\` (Vector Charts)
* L4: \`statcraft_reporting\` (Pivot Tables)
* L5: \`statcraft_syntax\` (CommandBus & SPSS Parser)
* L6: \`statcraft_app\` (Desktop GUI)`,
    contentRu: `### 1. Контракт нулевых паник
В рабочем коде StatCraft категорически запрещены аварийные конструкции:
\`unwrap()\`, \`expect(...)\`, \`panic!(...)\`, \`unreachable!(...)\`, \`todo!()\` и блоки \`unsafe\`.
Все потенциальные ошибки — вырожденные матрицы, деление на ноль или нулевая дисперсия — передаются в виде типизированных \`Result<T, StatError>\` и выводятся пользователю без падения программы.

### 2. Модель памяти Apache Arrow
Вместо выделения памяти под каждую ячейку таблицы StatCraft хранит столбцы в виде непрерывных массивов Apache Arrow:
* **SIMD-векторизация**: Математические операции задействуют регистры процессора напрямую;
* **Движок Polars**: Быстрые многопоточные группировки и фильтрации;
* **Без GC**: Предсказуемое освобождение ресурсов без пауз сборщика мусора.

### 3. Иерархия слоев (L0–L6)
В проекте действует строгая слоистая архитектура, проверяемая утилитой \`cargo xtask layers\`:
* L0: \`statcraft_io\` (Кодеки файлов)
* L1: \`statcraft_core\` (Датасеты и словари)
* L2: \`statcraft_algo\` (Математические алгоритмы)
* L3: \`statcraft_charts\` (Векторные графики)
* L4: \`statcraft_reporting\` (Сводные таблицы)
* L5: \`statcraft_syntax\` (Шина CommandBus и парсер .sps)
* L6: \`statcraft_app\` (Графический интерфейс egui)`,
  },
];

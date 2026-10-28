
---

### Этап 6.2: T-критерий для независимых выборок (Independent-Samples T Test)
*Время: 28 октября 2026, 10:20 (~40 минут)*

Реализован ключевой для прикладных исследований и A/B-тестирования статистический метод — **T-критерий для двух независимых выборок** (Analyze -> Compare Means -> Independent-Samples T Test...):
1. **Математическое ядро L2 (statcraft_algo::ttest_indep):**
   - Полная реализация критерия равенства дисперсий Левена (SPSS-канонический $-тест с центрированием по среднему center=\'mean\');
   - Автоматический расчет двух строк критерия (канон IBM SPSS Statistics):
     1. *Equal variances assumed* — классический критерий Стьюдента с объединенной оценкой дисперсии ^2$ и  = N_1 + N_2 - 2$;
     2. *Equal variances not assumed* — критерий Уэлча с раздельными дисперсиями и дробными степенями свободы Сазертуэйта {welch}$;
   - Расчет дескриптивных статистик групп (, \bar{x}, SD, SE$), точной разности средних $\Delta$, стандартной ошибки разности {diff}$ и доверительных интервалов;
   - **Golden Oracle:** Сверка с SciPy (scipy.stats.levene и scipy.stats.ttest_ind(equal_var=True/False)) с абсолютной погрешностью $\le 10^{-6}$;
   - Устойчивость Never Crash при малых группах ( < 2$), вырожденных дисперсиях и строковых кодах групп (1 / 2, 1.0, M / F).
2. **Командная шина и синтаксис L5 (statcraft_syntax::commands::ttest_indep_cmd):**
   - Команда IndependentSamplesTTestCommand (stat.ttest.indep) с валидацией парности наблюдений и JSON-спецификацией параметров для внешних AI-агентов;
   - Генератор канонического синтаксиса SPSS:
     `	ext
     T-TEST GROUPS=gender(1 2)
       /CRITERIA=CI(0.95)
       /VARIABLES=salary score.
     `
3. **Графический интерфейс L6 (statcraft_app):**
   - Модальный диалог IndependentSamplesTTestDialog ( \times 420$ px) со списками проверяемых и группирующей переменной и векторными стрелками переноса;
   - Поддиалог Define Groups... для задания меток группы 1 и группы 2;
   - Активирован пункт меню Analyze -> Compare Means -> Independent-Samples T Test...;
   - В Output Viewer выводятся две аутентичные сводные таблицы SPSS:
     1. **Group Statistics:** Variable, Group, N, Mean, Std. Deviation, Std. Error Mean;
     2. **Independent Samples Test:** двухъярусный заголовок с секциями *Levene\'s Test for Equality of Variances* ($, Sig.) и *t-test for Equality of Means* ($, $, Sig., Mean Diff, Std. Error Diff, 95% CI Lower/Upper) для обеих строк (Equal variances assumed / not assumed);
   - Экспорт таблиц в TSV (Excel/Sheets) через контекстное меню и интеграция в дерево оглавления.
4. **Контроль качества (DoD):**
   - Тесты воркспейса cargo test --workspace -j 1 — 100% успешно;
   - Архитектурные слои cargo run -p xtask -j 1 -- layers — 0 нарушений;
   - Каталог паритета cargo run -p xtask -j 1 -- parity — команда зарегистрирована со статусом Done;
   - Линтер локализации i18n_lint_test — 2/2 passed (0% сырой кириллицы);
   - cargo clippy --workspace -j 1 -- -D warnings — 0 предупреждений.

---

### Этап 6.3: Парный T-критерий Стьюдента для связанных выборок (Paired-Samples T Test)
*Время: 28 октября 2026, 11:00 (~35 минут)*

Реализована третья процедура канонического блока **Compare Means** пакета IBM SPSS Statistics — **Парный T-критерий Стьюдента для связанных выборок** (*Paired-Samples T Test*, пункт меню `Анализ -> Сравнение средних -> Парный T-критерий...`):
1. **Математическое ядро L2 (`statcraft_algo::ttest_paired`):**
   - Расчет дескриптивных статистик обеих переменных пары ($N, \bar{x}_1, \bar{x}_2, s_1, s_2, SE_1, SE_2$);
   - Расчет парного коэффициента линейной корреляции Пирсона ($r$), $t$-статистики корреляции и двусторонней значимости ($p_r$) при $df = N - 2$;
   - Анализ парных разностей $d_i = x_i - y_i$: средняя разность $\bar{d}$, несмещенное стандартное отклонение разностей $s_d$, стандартная ошибка разности $SE_d$, $t$-статистика критерия Стьюдента, $df = N - 1$, двусторонний $p$-value и $100(1-\alpha)\%$ доверительный интервал для разности средних;
   - **Golden Oracle:** Сверка дескрипторов, корреляции и метрик разностей с Python `scipy.stats` (`ttest_rel` и `pearsonr`) с погрешностью $\le 10^{-6}$;
   - Защита **Never Crash**: корректная обработка выборок с $N < 2$, вырожденной нулевой дисперсии разностей ($s_d = 0$) и построчное исключение пропусков (Listwise deletion per pair) на 100% safe Rust без `unwrap`/`expect`/`panic`.
2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::ttest_paired_cmd`):**
   - Типизированная команда движка `PairedSamplesTTestCommand` (`stat.ttest.paired`) со спецификацией параметров `specs()` для внешних AI-агентов;
   - Генератор канонического синтаксиса SPSS:
     ```text
     T-TEST PAIRS=before WITH after (PAIRED)
       /CRITERIA=CI(0.95).
     ```
3. **Графический интерфейс L6 (`statcraft_app`):**
   - Модальное окно `PairedSamplesTTestDialog` ($520 \times 420$ px) со строгим соответствием канону верстки десктопных диалогов StatCraft (исходный список $196 \times 240$ px, векторные кнопки переноса `>` и `<`, целевой список пар с блоком текущего выбора `Variable 1` / `Variable 2`, поле доверительного интервала и горизонтальная строка действий `[ ОК ] [ Вставить ] [ Сброс ] [ Отмена ]` внизу);
   - Активирован пункт меню `Analyze -> Compare Means -> Paired-Samples T Test...`;
   - В `Output Viewer` выводятся три аутентичные сводные таблицы SPSS:
     1. **Paired Samples Statistics:** Pair $k$, Variable, Mean, N, Std. Deviation, Std. Error Mean;
     2. **Paired Samples Correlations:** Pair $k$ ($v_1$ & $v_2$), N, Correlation, Sig.;
     3. **Paired Samples Test:** двухъярусный заголовок с секцией *Paired Differences* (Mean, Std. Deviation, Std. Error Mean, 95% Confidence Interval of the Difference [Lower, Upper]), а также $t$, $df$, $Sig. (2\text{-tailed})$;
   - Экспорт таблиц в TSV (буфер обмена для Excel/Google Sheets) и интеграция в дерево оглавления.
4. **Контроль качества (Definition of Done):**
   - Полный прогон всех тестов воркспейса `cargo test --workspace -j 1` — 100% успешно;
   - Проверка линтера локализации `i18n_lint_test` — 0% нелокализованных строк;
   - Проверка архитектурных слоев `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - Каталог паритета `cargo run -p xtask -j 1 -- parity` — команда зарегистрирована со статусом Done;
   - Проверка чистоты компилятором `cargo clippy --workspace -j 1 -- -D warnings` — 0 ошибок и 0 предупреждений;
   - Стандарт форматирования `cargo fmt --check` — чисто.

---

### Этап 6.4: Однофакторный дисперсионный анализ (One-Way ANOVA) и модульная архитектура Output Viewer
*Время: 28 октября 2026, 13:40 (~40 минут)*

Завершен четвертый финальный компонент блока **Compare Means** пакета IBM SPSS Statistics — **Однофакторный дисперсионный анализ** (*One-Way ANOVA*, пункт меню `Анализ -> Сравнение средних -> Однофакторный дисперсионный анализ...`), а также проведена кардинальная модульная реструктуризация подсистемы Output Viewer:
1. **Модульная архитектура Output Viewer (`statcraft_app::views::output_view`):**
   - Устранена критическая проблема монолитного файла `output_view.rs` (3000+ строк), приводившая к исчерпанию контекста и сбоям диффов субагентов;
   - Подсистема вывода реорганизована в модульный пакет с файлами до 350 строк:
     * `model.rs`: модель данных `OutputViewerState`, `OutputEntry`, типизированный `OutputPayload`;
     * `outline.rs`: интерактивное дерево оглавления, векторные иконки папок/таблиц/документа, тулбар (Очистить, Развернуть/Свернуть всё);
     * `tables_descriptives.rs`: таблица описательных статистик SPSS + TSV-экспорт;
     * `tables_frequencies.rs`: таблицы частот и перцентилей SPSS + TSV-экспорт;
     * `tables_ttest.rs`: таблицы всех трех t-критериев (One-Sample, Independent, Paired) + TSV-экспорт;
     * `tables_anova.rs`: таблицы One-Way ANOVA (Descriptives, Levene, ANOVA table) + TSV-экспорт;
     * `mod.rs`: координация раскладки, реэкспорт и маршрутизация рендера.
2. **Математическое ядро L2 (`statcraft_algo::anova_oneway`):**
   - Расчет дисперсий: межгрупповая ($SS_{between}$), внутригрупповая ($SS_{within}$) и общая сумма квадратов ($SS_{total}$);
   - Степени свободы: $df_1 = k - 1$, $df_2 = N - k$, $df_{total} = N - 1$;
   - Средние квадраты ($MS_{between}, MS_{within}$) и статистика Фишера-Снедекора $F = \frac{MS_{between}}{MS_{within}}$;
   - Точный расчет $p$-value функции выживания $F$-распределения через регуляризованную неполную бета-функцию $I_x(a, b)$;
   - Критерий однородности дисперсий Левена (*Levene's Test of Homogeneity of Variances*);
   - Дескриптивные статистики по уровням фактора ($N, \bar{y}, s, SE$, 95% ДИ, Min, Max + итоговая строка Total);
   - **Golden Oracle:** 100% сверка результатов с эталоном SciPy (`scipy.stats.f_oneway`) с абсолютной погрешностью $\le 10^{-6}$.
3. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::anova_oneway_cmd`):**
   - Типизированная команда `OneWayAnovaCommand` (`stat.anova.oneway`) со схемой параметров для внешних AI-агентов;
   - Генерация классического синтаксиса SPSS:
     ```text
     ONEWAY salary bonus BY jobcat
       /STATISTICS DESCRIPTIVES HOMOGENEITY
       /MISSING ANALYSIS.
     ```
4. **Графический интерфейс L6 (`statcraft_app`):**
   - Модальное окно `OneWayAnovaDialog` ($520 \times 440$ px) с каноническим выбором списка зависимых переменных и категориального фактора, чекбоксами Descriptives / Homogeneity и настройкой доверительного интервала;
   - Активирован пункт меню `Analyze -> Compare Means -> One-Way ANOVA...`;
   - Вывод трех аутентичных сводных таблиц SPSS в Output Viewer («Descriptives», «Test of Homogeneity of Variances», «ANOVA»), копирование в буфер обмена (TSV) и интеграция в оглавление.
5. **Контроль качества (Definition of Done):**
   - Полный прогон всех тестов воркспейса `cargo test --workspace -j 1` — 100% успешно;
   - Линтер локализации `i18n_lint_test` — 3/3 passed (0% захардкоженных строк);
   - Архитектурная строгость слоев `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - Реестр паритета `cargo run -p xtask -j 1 -- parity` — команда зарегистрирована со статусом Done;
   - Компиляторная чистота `cargo clippy --workspace -j 1 -- -D warnings` — 0 ошибок, 0 предупреждений;
   - Стиль кода `cargo fmt --check` — чисто.

---

### Этап 6.5: Защита от раздутия файлов и регрессий функционала (File Size Guard & Regression Shield)
*Время: 28 октября 2026, 15:00 (~35 минут)*

Реализована комплексная многоуровневая система превентивной защиты кодовой базы от раздутия файлов, деградации контекста LLM и скрытых регрессий интерфейса:
1. **Инструментальный рубеж (`xtask/src/file_guard.rs`):**
   - Разработан и внедрен линтер исходного кода `cargo run -p xtask -j 1 -- file-guard` с дифференцированными порогами:
     * Листовые функциональные модули (`algo/`, `syntax/commands/`, `dialogs/`, `views/output_view/`): жесткий лимит **500 строк**;
     * Комплексные интерактивные редакторы (`data_view.rs`, `variable_view.rs`, `syntax_editor.rs`): лимит **1000 строк**;
     * Корневой координатор приложения (`app.rs`): потолок **1500 строк**;
     * Общие модули: лимит **1000 строк**;
   - Инструктивная формулировка ошибки при превышении лимита: категорический запрет на сжатие кода или урезание колонок/таблиц при требовании модульной декомпозиции (принцип «Разделяй, а не урезай»).
2. **Архитектурная декомпозиция таблиц и моделей (`output_view/`):**
   - Монолит `tables_ttest.rs` (571 строка) разделен на три узкоспециализированных модуля:
     * `tables_ttest_onesample.rs` (127 строк): таблицы One-Sample Statistics и One-Sample Test + TSV;
     * `tables_ttest_indep.rs` (180 строк): двухъярусные таблицы Group Statistics и Independent Samples Test (Levene + Welch/Student) + TSV;
     * `tables_ttest_paired.rs` (204 строки): таблицы Paired Statistics, Correlations и Paired Differences + TSV;
     * `tables_common.rs` (90 строк): переиспользуемые функции расчета ширин и отрисовки ячеек;
   - Модель `model.rs` (616 строк) декомпозирована на чистые структуры данных (`model.rs`, 175 строк) и процедурные конструкторы (`builder.rs`, 456 строк);
   - Выделен поддиалог `ttest_indep_define_groups.rs` (46 строк) из `ttest_indep_dialog.rs` (462 строки);
   - Вынесен модуль форматирования `var_format_cmds.rs` (270 строк) из `var_cmds.rs` (294 строки);
   - Итог: 100% файлов воркспейса укладываются в установленные лимиты (76 файлов проверено).
3. **Тестовый рубеж интерфейсных инвариантов (`crates/statcraft_app/tests/output_viewer_invariants.rs`):**
   - Разработан пакет Headless UI-тестов, исключающий риск затирания предыдущих процедур и регрессий вывода:
     1. `test_continuous_history_accumulation`: непрерывное накопление всех анализов (Descriptives, Frequencies, 3 t-теста, ANOVA), строгая монотонность идентификаторов, сохранение предшествующих блоков Notes/Log;
     2. `test_exhaustive_enum_payloads_render_without_panics`: headless-рендер всех 11 вариантов `OutputItemKind` в `egui` (включая граничные случаи: нулевые степени свободы, пустые списки, нулевая дисперсия) с гарантией 0 паник;
     3. `test_tsv_export_golden_headers`: эталонная проверка структуры TSV-экспорта (наличие колонок `Mean`, `Std. Deviation`, `Valid N (listwise)`, `Valid Percent`, `Total`, `Levene`, `Equal variances assumed / not assumed`, `Paired Differences`, `Between Groups`, `Sum of Squares`, `F`, `Sig.`);
     4. `test_outline_tree_navigation_integrity`: синхронизация фокуса дерева Outline и скролла процедур, проверка корректного удаления элементов.
4. **Нормативное закрепление (`AGENTS.md`):**
   - Добавлен раздел 12 «Защита от раздутия файлов и инварианты сохранения функционала» в генеральный регламент проекта (`AGENTS.md` и `.agents/rules/AGENTS.md`);
   - Команды `cargo run -p xtask -j 1 -- file-guard` и `cargo test -p statcraft_app --test output_viewer_invariants -j 1` стали обязательными критериями Definition of Done (DoD).
5. **Контроль качества (DoD):**
   - Прогон `cargo run -p xtask -j 1 -- file-guard` — 76 файлов проверено, 0 нарушений;
   - Инвариантные тесты `output_viewer_invariants` — 4/4 passed;
   - Все тесты воркспейса `cargo test --workspace -j 1` — 100% успешно;
   - Архитектурные слои `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - Реестр паритета `cargo run -p xtask -j 1 -- parity` — чисто;
   - Линтер локализации `i18n_lint_test` — 3/3 passed;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 ошибок и предупреждений;
   - Сборка чистого релиза `cargo build --package statcraft_app --release -j 1` — успешно (1m 12s).

---

### Этап 6.6: Стандартизация дизайн-системы диалоговых окон и защита от халтуры (DialogKit & Design Guard)
*Время: 28 октября 2026, 16:05 (~40 минут)*

Решена давняя проблема визуального и архитектурного разнобоя диалоговых окон: разнокалиберные размеры (`500x380`, `500x420`, `520x440`), «гуляющие» кнопки без фиксированной сетки и самодельная верстка переноса переменных заменены канонической дизайн-системой IBM SPSS Statistics с автоматическим линтингом:
1. **Канонический стандарт верстки (`docs/DESIGN_GUIDE.md`):**
   - Формализованы 4 базовых архетипа диалогов StatCraft: Type 1 (Statistical Procedure, $560 \times 430$ px), Type 2 (Subdialog/Options, $360 \times 300$ px), Type 3 (Properties, $460 \times 380$ px), Type 4 (Compute, $620 \times 480$ px);
   - Утверждены сетка, отступы (padding $12$ px, gap $8$/$12$ px) и типографика;
   - Зафиксировано правило адаптивности **`i18n Shield`**: равная ширина кнопок ряда действий с отступами $\ge 10$ px, предотвращающая смещение при смене языков (RU $\leftrightarrow$ EN).
2. **Библиотека компонентов `DialogKit` (`crates/statcraft_app/src/dialogs/common/`):**
   - `DialogShell`: универсальная обертка для окон модального типа с фиксированными типоразмерами, центрированием и модальностью;
   - `TransferButton`: векторная адаптивная кнопка переноса ($28 \times 28$ px, скругление $3$ px), автоматически разворачивающая стрелку (`->` / `<-`) в зависимости от активного списка;
   - `VariablePicker`: двухпанельный список переменных с векторными пиктограммами шкал и форматом `Метка [Имя]`;
   - `DialogActionBar`: нижний ряд действий Type 1 (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`) с единой шириной кнопок ($\ge 80 \times 26$ px, скругление $3$ px);
   - `SubdialogActionBar`: нижняя панель для поддиалогов (`[ Continue ] [ Cancel ] [ Help ]`);
   - `SubdialogSidebar`: правая вертикальная колонка кнопок вызова настроек SPSS (`[ Statistics... ]`, `[ Options... ]`, $90 \times 24$ px);
   - `GroupBox`: визуальная группировка чекбоксов, радиокнопок и полей ввода.
3. **Инструментальный рубеж защиты (`xtask/src/design_guard.rs`):**
   - Создан статический линтер `cargo run -p xtask -j 1 -- design-guard`, сканирующий все файлы диалогов:
     * Проверка 1: Запрет сырых вызовов `ui.button(...)` / `Button::new(...)` для системных действий `OK`, `Paste`, `Reset`, `Cancel`, `Continue`;
     * Проверка 2: Запрет прямого вызова `Window::new(...)` и хардкода произвольных размеров окон в обход `DialogShell`;
   - Линтер успешно отловил все 50 первоначальных нарушений и подтвердил идеальную чистоту после рефакторинга.
4. **Тестовый рубеж инвариантов (`crates/statcraft_app/tests/dialog_invariants.rs`):**
   - Разработан пакет Headless UI-тестов:
     * Проверка канонических размеров `DialogSize` и констант `SubdialogSidebar`;
     * Headless-рендеринг всех процедурных окон (Descriptives, Frequencies, One-Way ANOVA, 3 T-критерия, Value Labels, Missing Values, Variable Type) в `egui::Context` с гарантией 0 паник;
     * Проверка блокировки `can_run` при пустом наборе переменных;
     * Проверка сброса состояния `Reset` к заводским дефолтам SPSS.
5. **Полная унификация всех окон приложения:**
   - Переведены на `DialogKit` все диалоги анализа и свойств переменных (`frequencies_dialog.rs`, `descriptives_dialog.rs`, `anova_oneway_dialog.rs`, `ttest_onesample_dialog.rs`, `ttest_indep_dialog.rs`, `ttest_indep_define_groups.rs`, `ttest_paired_dialog.rs`, `missing_values_dialog.rs`, `value_labels_dialog.rs`, `variable_type_dialog.rs`);
   - Опции Descriptives, ANOVA и T-тестов вынесены в канонические модальные поддиалоги.
6. **Нормативное закрепление (`AGENTS.md`):**
   - Добавлен раздел 13 «Канон дизайн-системы диалоговых окон и защита от халтуры (DialogKit & Design Guard)» в генеральный регламент проекта;
   - Запуск `cargo run -p xtask -j 1 -- design-guard` и `cargo test -p statcraft_app --test dialog_invariants -j 1` стал обязательным условием Definition of Done (DoD).
7. **Контроль качества (DoD):**
   - `cargo run -p xtask -j 1 -- design-guard` — 0 нарушений стандарта;
   - `cargo run -p xtask -j 1 -- file-guard` — 86 файлов проверено, 0 превышений лимита строк;
   - Инвариантные тесты диалогов `dialog_invariants` — 9/9 passed;
   - Все тесты воркспейса `cargo test --workspace -j 1` — 93 теста успешно (100%);
   - Слои архитектуры `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - Каталог паритета `cargo run -p xtask -j 1 -- parity` — чисто;
   - Линтер локализации `i18n_lint_test` — 3/3 passed;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 предупреждений;
   - `cargo fmt --check` — чисто;
   - Сборка чистого релиза `cargo build --package statcraft_app --release -j 1` — успешно (1m 14s).

---

### Этап 6.7: Гибридная система локализации — вшитые словари и динамический подхват внешних .ftl файлов
*Время: 28 октября 2026, 18:45 (~25 минут)*

Реализована гибридная архитектура интернационализации (i18n), сочетающая каноническую автономность одного исполняемого файла экосистемы ArtCraft со свободой пользовательских переводов:
1. **Гарантированная база (Zero-Dependency Single Binary):**
   - Официальные словари `ru.ftl` и `en.ftl` вшиты в бинарник через `include_str!`. Программа на 100% автономна, работает из любого каталога и никогда не падает при отсутствии внешних файлов.
2. **Динамическое сканирование каталогов (`discover_and_load_locales`):**
   - При старте движок сканирует внешние источники: переменную окружения `STATCRAFT_LOCALES_DIR`, папку `locales/` рядом с исполняемым файлом (`current_exe()`) и локальную папку `./locales`.
   - Любой найденный `.ftl` файл (например, `de.ftl`, `es.ftl`, `fr.ftl`) регистрируется на лету и автоматически появляется в меню `Вид -> Язык` без перекомпиляции программы.
3. **Бесшовное перекрытие (Resource Overriding):**
   - Если пользователь кладет файл существующего языка (например, пользовательский `ru.ftl`), ключи из внешнего файла перекрывают встроенные через `add_resource_overriding` без потери непереведенных строк.
4. **Интеллектуальное отображение имени языка (`language_name`):**
   - Самоназвание считывается из бандла самого языка (`lang-name-{lang}` или `lang-name`, например `Deutsch`, `Español`);
   - При отсутствии — переводится через текущий/английский словарь;
   - В крайнем случае отображается код языка в верхнем регистре (`DE`, `ES`). Полностью исключены сырые ключи вида `[lang-name-xx]`.
5. **Автоматизация релизов (`.github/workflows/release.yml`):**
   - В релизный ZIP-архив `statcraft-windows-x64.zip` добавлено автоматическое копирование каталога `locales/` рядом со `statcraft_app.exe`.
6. **Тестирование и контроль качества (DoD):**
   - Создан интеграционный тест `crates/statcraft_app/tests/i18n_external_locales_test.rs`;
   - Добавлены 4 новых unit-теста в `crates/statcraft_i18n/src/lib.rs`;
   - Полный прогон `cargo run -p xtask -j 1 -- ci` завершен со статусом `ALL CI QUALITY GATES PASSED!`;
   - Сборка релизного бинарника `cargo build --package statcraft_app --release -j 1` — успешно (1m 08s).



---

### Этап 7.1: Таблицы сопряжённости (Crosstabs) и критерий независимости Хи-квадрат Пирсона ($\chi^2$)
*Время: 28 октября 2026, 20:55 (~35 минут)*

Реализована фундаментальная процедура анализа категориальных и дискретных данных — **Таблицы сопряжённости** (*Crosstabs / Contingency Tables*) со статистиками связи и критерием независимости Хи-квадрат Пирсона ($\chi^2$, пункт меню `Анализ -> Описательные статистики -> Таблицы сопряжённости...`):
1. **Математическое ядро L2 (`statcraft_algo::crosstabs` и `crosstabs_types`):**
   - Расчет категориальной сетки $R \times C$ с поддержкой строковых и числовых меток значений;
   - Вычисление наблюдаемых ($O_{ij}$) и ожидаемых частот ($E_{ij} = \frac{R_i \cdot C_j}{N}$), остатков ($O_{ij} - E_{ij}$) и процентных долей (по строкам, столбцам и от общего итога таблицы);
   - Критерий независимости Хи-квадрат Пирсона: $\chi^2 = \sum \frac{(O_{ij} - E_{ij})^2}{E_{ij}}$ при $df = (R - 1)(C - 1)$;
   - Отношение правдоподобия (Likelihood Ratio): $G^2 = 2 \sum O_{ij} \ln(O_{ij} / E_{ij})$ для $O_{ij} > 0$;
   - Точный расчет $p$-value функции выживания $\chi^2$-распределения через верхнюю регуляризованную гамма-функцию $Q(a, x)$ без внешних зависимостей и без паник;
   - Для таблиц $2 \times 2$:
     * Поправка Йейтса на непрерывность (Continuity Correction): $\chi^2_{Yates} = \frac{N (|ad - bc| - N/2)^2}{(a+b)(c+d)(a+c)(b+d)}$;
     * Двусторонний точный критерий Фишера (Fisher's Exact Test) с логарифмированием факториалов через `ln_gamma` и суммированием гипергеометрических вероятностей $P(k) \le P(a) + 10^{-11}$;
   - Меры связи для номинальных шкал: $\phi = \sqrt{\chi^2 / N}$ и $V$ Крамера с расчетом приближенной значимости;
   - **Golden Oracle:** Unit-тесты со сверкой эталонных матриц с Python `scipy.stats` (`chi2_contingency`, `fisher_exact`) с точностью $\le 10^{-6}$.
2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::crosstabs_cmd`):**
   - Типизированная команда `CrosstabsCommand` (`stat.crosstabs`) с JSON-схемой параметров для внешних ИИ-агентов;
   - Трансляция в канонический синтаксис SPSS:
     ```text
     CROSSTABS
       /TABLES=gender BY jobcat
       /STATISTICS=CHISQ PHI
       /CELLS=COUNT EXPECTED ROW COLUMN TOTAL RESID.
     ```
3. **Графический интерфейс L6 (`statcraft_app::dialogs::crosstabs_dialog`):**
   - Полнофункциональный диалог `CrosstabsDialog` ($560 \times 430$ px) строго на переиспользуемых примитивах `DialogKit`:
     * Исходный список `VariablePicker` слева;
     * Целевые списки строк (`Row(s)`) и столбцов (`Column(s)`) по центру с адаптивными кнопками переноса `TransferButton`;
     * Боковая колонка `SubdialogSidebar`: кнопки `[ Ячейки... ]` (`Cells...`) и `[ Статистики... ]` (`Statistics...`);
     * Модальные поддиалоги `Cells` (Counts: Observed, Expected, Residuals; Percentages: Row, Column, Total) и `Statistics` (Chi-square, Phi and Cramér's V) с `SubdialogActionBar`;
     * Нижний ряд `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Архитектурная декомпозиция координатора (`app_dialogs.rs`):**
   - Чтобы не превысить жесткий лимит 1500 строк на `app.rs`, все обработчики модальных окон вынесены в `app_dialogs.rs`;
   - Размер `app.rs` снижен с 1485 до 1138 строк (запас более 350 строк).
5. **Отображение в Output Viewer (`tables_crosstabs.rs`):**
   - Четыре аутентичные сводные таблицы SPSS:
     1. **Case Processing Summary:** сводка наблюдений (валидные, пропущенные, всего с процентами);
     2. **Crosstabulation Table:** многослойная матрица $R \times C$ со всеми запрошенными слоями ячеек и итоговыми строками/столбцами;
     3. **Chi-Square Tests:** Pearson Chi-Square, Continuity Correction, Likelihood Ratio, Fisher's Exact Test, N of Valid Cases;
     4. **Symmetric Measures:** Phi, Cramér's V и приближенная значимость;
   - Полноценный экспорт в TSV для каждой таблицы и бесшовная навигация в Outline Pane.
6. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Все 96 тестов воркспейса завершились успешно;
   - Линтеры `design-guard`, `file-guard`, `layers`, `parity` — 0 нарушений;
   - Линтер `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Форматирование `cargo fmt --check` — чисто.

---

### Этап 7.2: Парные корреляции (Bivariate Correlations: Pearson $r$, Spearman $\rho$, Kendall $\tau$-b)
*Время: 28 октября 2026, 21:30 (~35 минут)*

Реализован ключевой статистический аппарат для анализа взаимосвязей количественных и порядковых переменных — **Парные корреляции** (*Bivariate Correlations*, пункт меню `Анализ -> Корреляции -> Парные...`):
1. **Математическое ядро L2 (`statcraft_algo::correlations` и `correlations_types`):**
   - Расчет линейной корреляции Пирсона ($r = \frac{\sum (x_i - \bar{x})(y_i - \bar{y})}{\sqrt{\sum (x_i - \bar{x})^2 \sum (y_i - \bar{y})^2}}$), сумм квадратов отклонений ($SS_{xy}$) и несмещенной выборочной ковариации ($Cov(x, y)$);
   - Расчет ранговой корреляции Спирмена ($\rho$) с дробным усреднением совпадающих рангов (fractional ranking);
   - Расчет ранговой корреляции Кендалла ($\tau$-b) с подсчетом конкордантных ($P$) и дискордантных ($Q$) пар и аналитической поправкой на связки $T_x, T_y$;
   - Точный расчет $p$-value для двусторонних (2-tailed) и односторонних (1-tailed) критериев через функцию выживания распределения Стьюдента (Пирсон и Спирмен) и асимптотическую $Z$-статистику (Кендалл);
   - Автоматическая маркировка значимости звёздочками в ячейках (`*` при $p < 0.05$, `**` при $p < 0.01$);
   - Обработка пропущенных значений: попарное (*Pairwise*) и полное (*Listwise*) исключение;
   - Защита **Never Crash**: безопасная обработка нулевых дисперсий, одинаковых константных векторов и малых выборок ($N < 2$) на 100% safe Rust;
   - **Golden Oracle:** Модуль тестов `correlations_tests.rs` со сверкой матриц и $p$-value с SciPy (`scipy.stats.pearsonr`, `spearmanr`, `kendalltau`) с точностью $\le 10^{-6}$.
2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::correlations_cmd`):**
   - Команда `CorrelationsCommand` (`stat.correlations`) с JSON-спецификацией параметров для ИИ-агентов;
   - Генерация аутентичного синтаксиса IBM SPSS Statistics (`CORRELATIONS /VARIABLES=... /PRINT=TWOTAIL SIG /STATISTICS DESCRIPTIVES XPROD /MISSING=PAIRWISE.` и `NONPAR CORR`).
3. **Графический интерфейс L6 (`statcraft_app::dialogs::correlations_dialog`):**
   - Модальный диалог `CorrelationsDialog` ($560 \times 430$ px) на базе дизайн-системы `DialogKit`:
     * Левый список `VariablePicker` с иконками шкал измерения;
     * Векторная кнопка переноса `TransferButton`;
     * Центральный список переменных `Variables:`;
     * Блок чекбоксов `Correlation Coefficients` (Pearson, Kendall's tau-b, Spearman);
     * Блок `Test of Significance` (радиокнопки Two-tailed / One-tailed) и чекбокс `Flag significant correlations`;
     * Боковая панель `SubdialogSidebar`: кнопка `[ Параметры... ]` (`Options...`);
     * Модальный поддиалог `Options` (Statistics: Means and standard deviations, Cross-product deviations and covariances; Missing Values: Exclude cases pairwise / listwise) с `SubdialogActionBar`;
     * Нижний ряд `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Сводные таблицы в Output Viewer (`tables_correlations.rs`):**
   - **Descriptive Statistics:** переменная, среднее, стандартное отклонение, $N$;
   - **Correlations Matrix Table:** каноническая многоуровневая сетка $K \times K$ со строками коэффициента со звездочками, $p$-value, $N$, суммами квадратов отклонений, ковариациями и сносками значимости;
   - Экспорт таблиц в TSV и отображение в Outline Pane.
5. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 100 модульных и интеграционных тестов воркспейса завершились успешно (0 сбоев);
   - Линтеры архитектуры, дизайна и размера файлов (`design-guard`, `file-guard`, `layers`, `parity`) — 0 замечаний;
   - Линтер компилятора `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Форматирование `cargo fmt --check` — чисто.

---

### Этап 7.3: Непараметрические критерии для двух выборок (Mann-Whitney $U$ и Wilcoxon $W$)
*Время: 28 октября 2026, 22:15 (~45 минут)*

Реализован фундаментальный непараметрический аппарат для проверки различий между двумя группами в порядковых шкалах или при нарушении нормальности распределения — **Критерии для двух независимых выборок** (*Two-Independent-Samples Tests*, пункт меню `Анализ -> Непараметрические критерии -> Две независимые выборки...`):
1. **Математическое ядро L2 (`statcraft_algo::nonparam_twosample` и `nonparam_twosample_types`):**
   - Алгоритм объединенного ранжирования с дробным усреднением совпадающих значений (fractional mid-ranks) и суммой связок $T = \sum (t_i^3 - t_i)$;
   - Расчет сумм рангов ($W_1, W_2$) и средних рангов ($\bar{R}_1, \bar{R}_2$) по группам;
   - Расчет статистики Манна-Уитни $U = \min(U_1, U_2)$, где $U_1 = W_1 - \frac{n_1(n_1 + 1)}{2}$;
   - Определение статистики Вилкоксона $W$ в соответствии со стандартом IBM SPSS Statistics (сумма рангов группы с меньшим средним рангом);
   - Расчет математического ожидания $\mu_U = \frac{n_1 n_2}{2}$ и дисперсии $\sigma_U^2 = \frac{n_1 n_2}{12}\left[ (N + 1) - \frac{\sum (t_i^3 - t_i)}{N(N - 1)} \right]$ со строгой поправкой на связки;
   - Вычисление асимптотической $Z$-статистики и точного двустороннего $p$-value через регуляризованную неполную гамма-функцию $Q(0.5, Z^2 / 2)$;
   - Реализация парного критерия знаковых рангов Вилкоксона (*Wilcoxon Signed-Rank Test*) с ранжированием абсолютных разностей $|d_i|$, исключением нулевых связок и вычислением $W = \min(W^+, W^-)$;
   - Защита **Never Crash**: корректная обработка малых выборок ($N < 2$), вырожденных групп с нулевой дисперсией рангов и строковых кодовых обозначений групп через предикат `matches_group` на 100% safe Rust;
   - **Golden Oracle:** Модуль тестов `nonparam_twosample_tests.rs` со сверкой рангов, $U$, $W$, $Z$ и $p$-value с эталонами SciPy (`mannwhitneyu`, `wilcoxon`) и ручными SPSS-бенчмарками с точностью $\le 10^{-6}$.
2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::nonparam_twosample_cmd`):**
   - Команда `MannWhitneyCommand` (`stat.npar.mann_whitney`) с JSON-спецификацией параметров `specs()` для внешних ИИ-агентов;
   - Генерация канонического синтаксиса IBM SPSS Statistics:
     ```text
     NPAR TESTS
       /M-W= salary age BY gender(1 2)
       /MISSING=ANALYSIS.
     ```
3. **Графический интерфейс L6 (`statcraft_app::dialogs::nonparam_twosample_dialog`):**
   - Модальный диалог `Two-Independent-Samples Tests` ($560 \times 430$ px) на базе дизайн-системы `DialogKit`:
     * Список доступных переменных `VariablePicker` с векторными иконками шкал;
     * Векторная кнопка переноса `TransferButton` для целевых и группирующей переменных;
     * Список анализируемых переменных `Test Variable List:`;
     * Поле `Grouping Variable:` с отображением выбранных кодов `var(val1 val2)`;
     * Боковая колонка `SubdialogSidebar` с кнопкой `[ Задать группы... ]` (`Define Groups...`);
     * Модальный поддиалог `Define Groups` на базе `DialogShell::subdialog` с валидацией непустых и несовпадающих кодов;
     * Блок `Test Type` с чекбоксами `[x] Mann-Whitney U`, `[ ] Wilcoxon W`, `[ ] Kolmogorov-Smirnov Z`;
     * Нижний ряд кнопок действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Сводные таблицы в Output Viewer (`tables_nonparam.rs`):**
   - Таблица **Ranks:** переменная, группа, $N$, средний ранг (*Mean Rank*), сумма рангов (*Sum of Ranks*), строка Total;
   - Таблица **Test Statistics:** каноническая многоколоночная матрица (Mann-Whitney U, Wilcoxon W, Z, Asymp. Sig. 2-tailed) со сноской группирующей переменной;
   - Экспорт таблиц в TSV и интеграция в Outline Pane.
5. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 105 модульных и интеграционных тестов воркспейса завершились успешно (0 сбоев);
   - Линтеры архитектуры, дизайна и размера файлов (`design-guard`, `file-guard`, `layers`, `parity`) — 0 замечаний;
   - Линтер компилятора `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Форматирование `cargo fmt --check` — чисто.


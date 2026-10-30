
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

---

### Этап 7.4: Непараметрические критерии для двух связанных выборок (Wilcoxon Signed-Rank Test и Sign Test)
*Время: 29 октября 2026, 08:15 (~40 минут)*

Реализован классический аппарат непараметрического анализа связанных (парных) измерений («до» / «после», повторные замеры или парные наблюдения), когда предпосылка о нормальном распределении парных разностей нарушается — **Критерии для двух связанных выборок** (*Two-Related-Samples Tests*, пункт меню `Анализ -> Непараметрические критерии -> Связанные выборки (Вилкоксона / Фридмана)...`):
1. **Математическое ядро L2 (`statcraft_algo::nonparam_related` и `nonparam_related_types`):**
   - Расчет парных разностей $D_i = Y_i - X_i$ (`var2 - var1`);
   - Классификация нулевых разностей ($D_i = 0$) как связок (Ties), их подсчет и исключение из ранжирования согласно канону IBM SPSS Statistics;
   - Сортировка и ранжирование абсолютных значений $|D_i|$ с вычислением дробных средних рангов (mid-ranks) и суммы связок $\sum(t_j^3 - t_j)$;
   - Раздельный подсчет сумм и средних отрицательных ($D_i < 0$) и положительных ($D_i > 0$) рангов;
   - Расчет дисперсии $\sigma_W^2 = \frac{n(n + 1)(2n + 1)}{24} - \frac{\sum (t_j^3 - t_j)}{48}$ с поправкой на связки;
   - Расчет стандартизованной $Z$-статистики с поправкой на непрерывность $\pm 0.5$ и двустороннего асимптотического $p$-value через неполную гамма-функцию $Q(0.5, Z^2 / 2)$;
   - Определение базы критерия (отрицательные или положительные ранги) в строгом соответствии с формулой и сносками SPSS;
   - Реализация **Критерия знаков (Sign Test)**: подсчет отрицательных, положительных и нулевых разностей, вычисление точного двустороннего биномиального уровня значимости через регуляризованную неполную бета-функцию $I_{0.5}(n - k, k + 1)$ без ограничений на размер выборки;
   - Защита **Never Crash**: обработка пустых выборок, нулевых дисперсий, одинаковых значений и пропусков на 100% safe Rust;
   - **Golden Oracle:** Тесты `nonparam_related_tests.rs` со сверкой рангов, $W$, $Z$ и $p$-value со SciPy и эталонами SPSS с точностью $\le 10^{-6}$.
2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::nonparam_related_cmd`):**
   - Команда `RelatedSamplesTestCommand` (`stat.npar.related2`) со спецификацией параметров `specs()` для внешних ИИ-агентов;
   - Генерация канонического синтаксиса IBM SPSS Statistics:
     ```text
     NPAR TESTS
       /WILCOXON= pre WITH post (PAIRED)
       /SIGN= pre WITH post (PAIRED)
       /MISSING=ANALYSIS.
     ```
3. **Графический интерфейс L6 (`statcraft_app::dialogs::nonparam_related_dialog`):**
   - Модальный диалог `Two-Related-Samples Tests` ($560 \times 430$ px) на базе дизайн-системы `DialogKit`:
     * Список доступных переменных `VariablePicker` с иконками шкал измерения;
     * Двухступенчатая панель `Current Selections: Variable 1, Variable 2` с адаптивными векторными кнопками переноса `TransferButton`;
     * Список сформированных пар `Test Pairs:`;
     * Группа `Test Type` с чекбоксами `[x] Wilcoxon`, `[ ] Sign`;
     * Нижний ряд кнопок действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Сводные таблицы в Output Viewer (`tables_nonparam_related.rs`):**
   - Таблица **Ranks (Wilcoxon):** строки `Negative Ranks`, `Positive Ranks`, `Ties`, `Total` ($N$, Mean Rank, Sum of Ranks) со сносками направления разностей;
   - Таблица **Test Statistics (Wilcoxon):** $Z$-оценка, Asymp. Sig. (2-tailed), сноски метода и базы рангов;
   - Таблица **Frequencies (Sign Test):** отрицательные, положительные разности, совпадения, итог;
   - Таблица **Test Statistics (Sign Test):** Exact Sig. (2-tailed);
   - Экспорт таблиц в TSV и интеграция в Outline Pane.
5. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Все 110+ тестов воркспейса завершились успешно (0 сбоев);
   - Все файлы воркспейса строго соблюдают дифференцированные лимиты строк (`file-guard` $\le 500$ строк для листовых модулей);
   - Линтеры дизайна и слоев (`design-guard`, `layers`, `parity`) — 0 замечаний;
   - Полная синхронная локализация (`locales/en.ftl`, `locales/ru.ftl`);
   - Компилятор и линтер `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений.



---

### Этап 7.5: Исправление навигационной панели (Outline Overflow Fix) и Критерий Крускала-Уоллиса для K независимых выборок (Kruskal-Wallis H Test)
*Время: 29 октября 2026, 08:45 (~35 минут)*

Решены две важные задачи: исправлен дефект выезда длинных заголовков за пределы левой навигационной колонки Outline Pane в Output Viewer и реализован непараметрический аналог однофакторного дисперсионного анализа — **Критерий Крускала-Уоллиса для нескольких независимых выборок** (*K Independent Samples / Kruskal-Wallis H Test*, пункт меню `Анализ -> Непараметрические критерии -> K независимых выборок...`):
1. **Исправление дефекта навигационной панели (Outline Overflow Fix):**
   - Первопричина: низкоуровневый вызов `ui.painter().text(...)` в `crates/statcraft_app/src/views/output_view/outline.rs` рисовал текст целиком без отсечения по ширине, а в родительском контейнере `mod.rs` отсутствовал `set_clip_rect`;
   - Решение:
     1. Базовая ширина навигационной панели `outline_width` увеличена с 200.0 до 230.0 px для комфортного чтения на экранах высокого разрешения;
     2. Установлен рубеж отсечения `ui.set_clip_rect(left_rect)` на родительскую область дерева вывода;
     3. Написана посимвольная утилита `truncate_text_to_width`, измеряющая ширину строки через `ui.fonts` и аккуратно обрезающая текст с добавлением `...`, если он не помещается в доступную ширину;
     4. Отрисовка текста изолирована защитным прямоугольником `ui.painter().with_clip_rect(text_rect)`;
     5. Добавлены всплывающие подсказки `.on_hover_text(...)` с полным не обрезанным названием процедуры и элемента.
2. **Математическое ядро L2 (`statcraft_algo::kruskal_wallis` и `kruskal_wallis_tests`):**
   - Фильтрация группирующей переменной по числовому диапазону $[min, max]$ (канон SPSS `BY groupvar(min max)`);
   - Объединенное ранжирование всех $N = \sum n_i$ наблюдений с присвоением дробных средних рангов (mid-ranks) при совпадении значений;
   - Расчет сумм рангов $R_i$ и средних рангов $\bar{R}_i = R_i / n_i$ по каждой из $k$ групп;
   - Расчет базового критерия $H = \frac{12}{N(N+1)} \sum \frac{R_i^2}{n_i} - 3(N+1)$;
   - Поправка на связки: расчет фактора связок $C = 1 - \frac{\sum (t_j^3 - t_j)}{N^3 - N}$ и скорректированного значения $H_{adj} = H / C$;
   - Число степеней свободы $df = k - 1$ и вычисление асимптотического двустороннего $p$-value через распределение $\chi^2$ (`chi_square_sf(H_adj, df)`);
   - **Golden Oracle:** Тесты `kruskal_wallis_tests.rs` со сверкой критерия $H$ и $p$-value со `scipy.stats.kruskal` с точностью $\le 10^{-6}$ (как для уникальных значений, так и для выборок со связками);
   - Защита **Never Crash**: безопасная обработка пустого датасета, одной группы ($k < 2$) и одинаковых наблюдений на 100% safe Rust без паник.
3. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::kruskal_wallis_cmd`):**
   - Типизированная команда `KruskalWallisCommand` (`stat.npar.kruskal_wallis`) со спецификацией параметров `specs()` для внешних ИИ-агентов;
   - Генератор канонического синтаксиса IBM SPSS Statistics:
     ```text
     NPAR TESTS
       /K-W= salary age BY jobcat(1 3)
       /MISSING=ANALYSIS.
     ```
4. **Графический интерфейс L6 (`statcraft_app::dialogs::kruskal_wallis_dialog` и `kruskal_define_range`):**
   - Диалог `Tests for Several Independent Samples` ($560 \times 430$ px) на базе `DialogKit`:
     * Список доступных переменных `VariablePicker` со значками шкал;
     * Поле `Test Variable List:` и поле `Grouping Variable:` с отображением диапазона `var(min max)`;
     * Адаптивные векторные кнопки переноса `TransferButton`;
     * Боковая колонка `SubdialogSidebar` с кнопкой `[ Задать диапазон... ]` (`Define Range...`);
     * Модальный поддиалог `Define Range` ($360 \times 300$ px) на `DialogShell::subdialog` с полями `Minimum:` и `Maximum:`;
     * Блок `Test Type` с чекбоксом `[x] Kruskal-Wallis H`;
     * Нижний ряд кнопок действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Сводные таблицы в Output Viewer (`tables_kruskal.rs`):**
   - Таблица **Ranks:** переменная, группа, $N$, средний ранг (*Mean Rank*), строка Total;
   - Таблица **Test Statistics:** каноническая многоколоночная матрица (Kruskal-Wallis H, df, Asymp. Sig.) со сносками `a. Kruskal Wallis Test` и `b. Grouping Variable: ...`;
   - Экспорт таблиц в TSV и интеграция в Outline Pane.
6. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 115+ модульных и интеграционных тестов воркспейса завершились успешно (0 сбоев);
   - Все файлы воркспейса строго соблюдают дифференцированные лимиты строк (`file-guard` $\le 500$ строк для листовых модулей);
   - Линтеры дизайна и слоев (`design-guard`, `layers`, `parity`) — 0 замечаний;
   - Полная синхронная локализация (`locales/en.ftl`, `locales/ru.ftl`);
   - Компилятор и линтер `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений.


---

### Этап 8.1: Вычисление переменных (Compute Variable: `COMPUTE`)
*Время: 29 октября 2026, 09:25 (~35 минут)*

Реализован фундаментальный инструмент трансформации данных — **Вычисление новой переменной** (*Compute Variable*, `COMPUTE`, пункт меню `Преобразование -> Вычислить переменную...`):
1. **Математическое ядро L2 (`statcraft_algo::expr_eval`):**
   - Архитектурная модульная декомпозиция пакета `expr_eval/` (`types.rs`, `lexer.rs`, `parser.rs`, `eval.rs`, `mod.rs`, `tests.rs`) с соблюдением стандарта `file-guard` ($\le 500$ строк на файл);
   - Токенизатор строковых математических выражений: числа, идентификаторы переменных и функций, операторы, скобки, запятые, корректное распознавание унарных плюсов и минусов;
   - Алгоритм сортировочной станции Дейкстры (Shunting-yard algorithm) для трансляции инфиксных выражений в обратную польскую запись (RPN) с учетом приоритетов и правой ассоциативности возведения в степень (`**`, `^`);
   - Построчный вычислитель с поддержкой арифметических операций (`+`, `-`, `*`, `/`, `**`), функций одного аргумента (`ABS`, `EXP`, `LN`, `LOG10`, `RND`, `SQRT`, `TRUNC`) и статистических функций от произвольного числа аргументов (`SUM`, `MEAN`, `MIN`, `MAX`);
   - Защита **Never Crash**: деление на ноль, отрицательные корни, пропуски в переменных возвращают системный пропуск (`NaN` / `SYSMIS`) без прерывания вычислений остальных строк датасета;
   - 100% покрытие модульными тестами всех приоритетов, функций и граничных условий.
2. **Командная шина движка L5 (`statcraft_syntax::commands::compute_cmd`):**
   - Типизированная команда `ComputeVariableCommand` (`transform.compute`) со спецификацией схемы `specs()` для внешних ИИ-агентов;
   - Создание новой числовой переменной (`DataType::Numeric { decimals: 2 }`, `Measure::Scale`) либо перезапись существующей колонки;
   - Поддержка отката `Undo` (`Ctrl+Z`): автоматическое удаление созданной колонки или восстановление старых значений и метаданных перезаписанной переменной;
   - Генератор аутентичного синтаксиса IBM SPSS Statistics:
     ```text
     COMPUTE bmi = weight / ((height / 100) ** 2).
     VARIABLE LABELS bmi "Body Mass Index".
     EXECUTE.
     ```
3. **Графический интерфейс L6 (`statcraft_app::dialogs::compute_dialog`):**
   - Канонический диалог калькулятора $620 \times 480$ px на базе `DialogKit` (`DialogShell::compute`, `render_variable_list_panel`, `TransferButton`, `DialogActionBar`);
   - Верхний ряд: ввод целевой переменной `Target Variable:` и метки `Label:`;
   - Левая панель: выбор доступных переменных датасета с иконками шкал и адаптивной кнопкой переноса `TransferButton`;
   - Центральная панель: многострочный моноширинный редактор выражений `Numeric Expression:` и виртуальная клавиатура калькулятора (`+`, `-`, `*`, `/`, `**`, `(`, `)`, цифровые клавиши `0..9`, `.`, `C`);
   - Правая панель: переключатель групп функций (*All, Arithmetic, Statistical*) и прокручиваемый список функций с шаблонами для быстрой вставки;
   - Нижняя панель действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Интеграция и мгновенная реактивность:**
   - Активирован пункт меню `Transform -> Compute Variable...` с валидацией наличия данных;
   - После выполнения вычисления новая колонка немедленно появляется в сетке таблицы `Data View` и словаре `Variable View`;
   - Запись сгенерированного синтаксиса в журнал `Output Viewer` (Log).
5. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 120+ тестов воркспейса завершились успешно (0 сбоев);
   - Дифференцированные лимиты строк `file-guard`, правила верстки `design-guard`, архитектурные слои `layers` и матрица паритета `parity` — 0 замечаний;
   - Полная двуязычная локализация (`locales/en.ftl`, `locales/ru.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений.


---

### Этап 8.2: Перекодирование значений (Recode into Same / Different Variables: `RECODE`)
*Время: 29 октября 2026, 10:25 (~35 минут)*

Реализован ключевой аналитический инструмент группировки, дискретизации и очистки данных — **Перекодирование значений** (*Recode Values*, `RECODE`, пункты меню `Преобразование -> Перекодировать в другие переменные...` и `Преобразование -> Перекодировать в те же переменные...`):
1. **Математический движок сопоставления правил L2 (`statcraft_algo::recode`):**
   - Гибкие спецификации исходных значений `OldValueRule`:
     * Точное числовое значение (`Exact(f64)`);
     * Системный пропуск (`SysMissing`);
     * Системные и пользовательские пропущенные значения (`SystemOrUserMissing`);
     * Диапазон от $A$ до $B$ включительно (`Range(low, high)`);
     * Диапазон от минимального до $A$ (`RangeLowest(high)`);
     * Диапазон от $A$ до максимального (`RangeHighest(low)`);
     * Все остальные значения (`AllOther` / `ELSE`);
   - Спецификации новых значений `NewValueSpec`:
     * Новое числовое значение (`Value(f64)`);
     * Системный пропуск (`SysMissing`);
     * Копирование исходного значения (`CopyOld`);
   - Последовательное применение правил (первое совпадение побеждает), сохранение исходного значения без изменений при отсутствии совпадений (SPSS-канон);
   - Защита **Never Crash**: безопасная обработка `NaN`, пропусков и краевых диапазонов; 100% покрытие тестами.
2. **Командная шина L5 (`statcraft_syntax::commands::recode_diff_cmd` & `recode_same_cmd`):**
   - Команда `RecodeIntoDifferentCommand` (`transform.recode.diff`): перекодирование одной или нескольких пар `source -> target` в новые или существующие переменные;
   - Команда `RecodeIntoSameCommand` (`transform.recode.same`): перекодирование значений по месту (*in-place*);
   - Полная поддержка `Undo` (`Ctrl+Z`) и `Redo`: автоматическое удаление созданных колонок либо восстановление снимков значений и метаданных для перезаписанных колонок;
   - Генерация канонического синтаксиса IBM SPSS Statistics:
     ```text
     RECODE age (LOWEST THRU 18=1) (19 THRU 35=2) (36 THRU 60=3) (61 THRU HIGHEST=4) (ELSE=SYSMIS) INTO age_group.
     VARIABLE LABELS age_group "Age Group Categories".
     EXECUTE.
     ```
3. **Двухступенчатый интерфейс на DialogKit L6 (`statcraft_app::dialogs`):**
   - Главный диалог `RecodeDialog` ($560 \times 430$ px, `DialogShell::standard`):
     * Поддержка двух режимов: *Different Variables* и *Same Variables*;
     * Левая панель выбора доступных переменных `VariablePicker` с иконками шкал и кнопками переноса `TransferButton`;
     * Список сопоставлений `Numeric Variable -> Output Variable:`;
     * Блок свойств целевой переменной `Output Variable` (`Name:`, `Label:`, кнопка `Change`);
     * Кнопка вызова поддиалога `[ Old and New Values... (N) ]`;
     * Нижний ряд `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Модальный поддиалог `Old and New Values...` (`RecodeOldNewState`):
     * Левый блок `Old Value`: радиокнопки точного значения, пропусков, диапазонов (включая `LOWEST` и `HIGHEST`), и `All other values`;
     * Правый блок `New Value`: радиокнопки нового значения, пропуска и копирования старого;
     * Кнопки редактирования списка сопоставлений `[ Add ]`, `[ Change ]`, `[ Remove ]`;
     * Прокручиваемый список активных правил `Old -> New:`;
     * Нижняя панель действий `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`).
4. **Интеграция и реактивность:**
   - Активированы пункты меню `Transform -> Recode into Different Variables...` и `Transform -> Recode into Same Variables...`;
   - Результаты перекодирования мгновенно отражаются в таблице данных `Data View` и словаре `Variable View`;
   - Журналирование синтаксиса `RECODE` в `Output Viewer` (Log).
5. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 125+ тестов воркспейса завершились успешно (0 сбоев);
   - Полное соблюдение `file-guard` ($\le 500$ строк на файл), `design-guard`, `layers`, `parity` (31 команда);
   - Синхронная локализация (`locales/en.ftl`, `locales/ru.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений.

---

### Этап 8.3: Сортировка наблюдений (Sort Cases: `SORT CASES`)
*Время: 29 октября 2026, 11:15 (~35 минут)*

Реализована каноническая процедура упорядочивания наблюдений датасета в StatCraft — пункт меню `Данные -> Сортировать наблюдения...` (`Data -> Sort Cases...`, команда `data.sort_cases` / синтаксис `SORT CASES BY ... (A|D)`):
1. **Низкоуровневая перестановка строк L0/L1 (`statcraft_io` & `statcraft_core`):**
   - Метод `RawTable::reorder_rows` в `crates/statcraft_io/src/csv_loader.rs` и `DataSet::reorder_cases` в `crates/statcraft_core/src/dataset.rs`;
   - Быстрая и безопасная перестановка всех колонок таблицы без паник по вектору перестановок индексов строк;
   - Строгая валидация границ (`IndexOutOfBounds`), сохранение словаря переменных (`VariableDictionary`), метаданных, меток и пропусков без изменений.
2. **Алгоритмический движок L2 (`statcraft_algo::sort_cases`):**
   - Спецификация ключа сортировки `SortKeySpec` с направлением `SortDirection` (`Ascending` / `Descending`);
   - Каноническая обработка пропусков по стандарту IBM SPSS Statistics:
     * Системные пропуски (`SYSMIS` / `Null` / `NaN`) трактуются как наименьшие отрицательные ($-\infty$);
     * При сортировке по возрастанию (`Ascending`, `(A)`) пропуски идут **первыми**;
     * При сортировке по убыванию (`Descending`, `(D)`) пропуски идут **последними**;
   - Многоуровневая стабильная сортировка: при совпадении первичного ключа порядок определяется вторичным ключом с сохранением относительного исходного порядка равных наблюдений;
   - Утилита вычисления обратной перестановки `invert_permutation` для мгновенного отката;
   - 100% safe Rust без `unwrap()` / `expect()`.
3. **Командная шина L5 (`statcraft_syntax::commands::sort_cases_cmd`):**
   - Типизированная команда `SortCasesCommand` (`data.sort_cases`) со спецификацией `specs()` для чтения AI-агентами;
   - Полная поддержка стека отката `Undo` (`Ctrl+Z`) через историю обратных перестановок `inv_permutation`;
   - Генератор канонического синтаксиса SPSS:
     ```text
     SORT CASES BY dept (A) salary (D).
     ```
4. **Пользовательский интерфейс на DialogKit L6 (`statcraft_app::dialogs`):**
   - Центрированный диалог `SortCasesDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Левая панель выбора доступных переменных `VariablePicker` с иконками шкал и кнопками переноса `TransferButton` (`->` / `<-`);
   - Список целевых ключей `Sort by:` с отображением направлений (например, `dept (A)`, `salary (D)`);
   - Кнопки изменения приоритета ключей `Move Up` и `Move Down`;
   - Группа переключателей направления `Sort Order` (`(o) Ascending`, `( ) Descending`), мгновенно меняющая направление выбранного ключа;
   - Нижний ряд `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Интеграция и реактивность:**
   - Активирован пункт меню `Data -> Sort Cases...`;
   - Мгновенное переупорядочивание строк в сетке `Data View`;
   - Логирование канонического синтаксиса `SORT CASES BY ...` в журнал `Output Viewer` (Log) и вывод информационного статуса `StatusMessage::SortCasesExecuted`.
6. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Все тесты воркспейса завершились успешно;
   - Соблюдение дифференцированных лимитов строк `file-guard`, стандарта верстки `design-guard`, слоев `layers` и паритета `parity`;
   - Синхронная локализация (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений.

---

### Этап 8.4: Выборка наблюдений и фильтрация (Select Cases: `FILTER BY` / `SELECT IF`)
*Время: 29 октября 2026, 12:20 (~50 минут)*

Реализован канонический механизм фильтрации и выборки наблюдений в StatCraft — пункт меню `Данные -> Выбрать наблюдения...` (`Data -> Select Cases...`, команда `data.select_cases` / синтаксис `FILTER BY filter_$.` / `FILTER OFF.` / `SELECT IF (...)`):
1. **Модель фильтрации в DataSet L1 (`statcraft_core::dataset`):**
   - Добавлено поле `active_filter_var: Option<String>` в структуру `DataSet`;
   - Методы `set_filter_variable`, `active_filter_var`, `is_row_active`, `is_row_filtered` и `active_row_indices`;
   - Автоматическая синхронизация при удалении (`remove_variable`) или переименовании (`rename_variable`) фильтрующей переменной;
   - Строгая валидация существования переменной через `CoreError::VariableNotFound`.
2. **Расширение вычислителя выражений L2 (`statcraft_algo::expr_eval`):**
   - Поддержка операторов сравнения: `=`, `==`, `<>`, `~=`, `!=`, `<`, `<=`, `>`, `>=`, а также ключевых слов `EQ`, `NE`, `LT`, `LE`, `GT`, `GE`;
   - Поддержка логических связок: `&`, `&&`, `|`, `||`, `~`, `!`, `AND`, `OR`, `NOT`;
   - Построчное вычисление логических условий через RPN с возвратом $1.0$ (True) или $0.0$ (False), безопасная обработка пропусков `NaN` / `SYSMIS` без паник.
3. **Командная шина и Undo/Redo L5 (`statcraft_syntax::commands::select_cases_cmd`):**
   - Типизированная команда `SelectCasesCommand` (`data.select_cases`) со спецификацией `specs()` для внешних AI-агентов;
   - Поддержка 3 канонических режимов:
     1. *All cases* (`FILTER OFF.`);
     2. *If condition is satisfied* (вычисление выражения, создание системной переменной `filter_$` с метками `0 'Not Selected'` / `1 'Selected'`, установка `FILTER BY filter_$.`);
     3. *Use filter variable* (`FILTER BY <var>.`);
   - Поддержка действия для невыбранных наблюдений:
     * *Filtered* — временное исключение из анализа без удаления данных;
     * *Deleted* — каноническое удаление невыбранных строк (`SELECT IF (<cond>).`);
   - Полный откат `Undo` (`Ctrl+Z`), восстанавливающий как состояние фильтра, так и удаленные наблюдения или созданные колонки.
4. **Сквозная интеграция со всеми процедурами анализа L2/L5:**
   - Все 11 аналитических процедур (`descriptives`, `frequencies`, `crosstabs`, `correlations`, `ttest_indep`, `ttest_paired`, `ttest_onesample`, `anova_oneway`, `nonparam_twosample`, `nonparam_related`, `kruskal_wallis`) переведены на выборку наблюдений через `ds.active_row_indices()`;
   - Исключенные наблюдения автоматически игнорируются во всех расчетах, а при сбросе фильтра мгновенно возвращаются в анализ.
5. **Пользовательский интерфейс на DialogKit L6 (`statcraft_app::dialogs`):**
   - Центрированный диалог `SelectCasesDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Левая панель выбора доступных переменных `VariablePicker` с иконками шкал и адаптивной кнопкой переноса `TransferButton`;
   - Группа `Select` с радиокнопками режимов и кнопкой `[ If... ]`;
   - Модальный поддиалог условия `Select Cases: If Condition` ($360 \times 300$ px, `DialogShell::subdialog`) с полем ввода и виртуальной клавиатурой операторов (`+ - * / ** ( ) = ~= < <= > >= & | ~`);
   - Группа `Unselected Cases Are` (`(o) Filtered`, `( ) Deleted`);
   - Нижняя строка `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
6. **Визуализация в Data View и строке состояния:**
   - В сетке `Data View` номера строк исключенных наблюдений перечеркиваются диагональной векторной чертой (`ui.painter().line_segment`), точно повторяя канон IBM SPSS Statistics;
   - В строке состояния (Status Bar) отображается индикатор активной фильтрации `Filter: ON (var_name)` / `Фильтр: Включен (var_name)`;
   - Активирован пункт главного меню `Data -> Select Cases...`.
7. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Все тесты воркспейса завершились успешно;
   - Соблюдение `file-guard` ($\le 500$ строк на файл), `design-guard`, `layers`, `parity` (33 команды);
   - Синхронная локализация (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Чистая сборка и обновление релизного бинарника `C:\StatCraft\statcraft_app.exe`.

---

### Этап 8.5: Расщепление файла на подгруппы (Split File: `SPLIT FILE`)
*Время: 29 октября 2026, 13:10 (~45 минут)*

Реализован ключевой механизм сегментного анализа данных IBM SPSS Statistics в StatCraft — пункт меню `Данные -> Разделить файл...` (`Data -> Split File...`, команда `data.split_file` / синтаксис `SPLIT FILE BY ...` / `SPLIT FILE OFF.`):
1. **Модель подгрупп в DataSet L1 (`statcraft_core::split` & `dataset`):**
   - Добавлены структуры `SplitFileMode` (*Off*, *CompareGroups*, *OrganizeOutput*), `SplitFileConfig`, `SplitKeyValue` и `SplitSubgroup`;
   - В структуру `DataSet` внедрено поле `split_file: Option<SplitFileConfig>` с методами `set_split_file()`, `split_file()` и `split_groups()`;
   - Автоматическая синхронизация при удалении (`remove_variable`) или переименовании (`rename_variable`) группирующих переменных с безопасным сбросом режима при удалении всех ключей.
2. **Алгоритм сегментации L2 (`statcraft_algo::split_groups`):**
   - Функция `extract_split_groups` с извлечением подгрупп наблюдений по произвольному числу группирующих переменных (составные ключи);
   - Корректная обработка системных пропусков (`Null` / `NaN` / `SYSMIS`) по канону SPSS (`cat = .`);
   - Бесшовная интеграция с фильтром наблюдений `Select Cases`: при активном фильтре подгруппы строятся исключительно из неотфильтрованных строк (`active_row_indices`), исключая искажения расчетов;
   - 100% safe Rust без паник.
3. **Командная шина и Undo/Redo L5 (`statcraft_syntax::commands::split_file_cmd`):**
   - Типизированная команда `SplitFileCommand` (`data.split_file`) со спецификацией `specs()` для внешних AI-агентов;
   - Поддержка автоматической предварительной сортировки наблюдений по группирующим переменным (`sort: bool`);
   - Генератор канонического синтаксиса SPSS:
     ```text
     SORT CASES BY gender (A) dept (A).
     SPLIT FILE LAYERED BY gender dept.
     ```
     или `SPLIT FILE OFF.` при отключении;
   - Полная поддержка отката `Undo` (`Ctrl+Z`), восстанавливающего как предыдущую конфигурацию расщепления, так и исходный порядок строк при автосортировке.
4. **Пользовательский интерфейс на DialogKit L6 (`statcraft_app::dialogs`):**
   - Центрированный диалог `SplitFileDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Выбор режима анализа:
     1. *(o) Analyze all cases, do not create groups* (`SPLIT FILE OFF.`);
     2. *( ) Compare groups* (`SPLIT FILE LAYERED BY ...`);
     3. *( ) Organize output by groups* (`SPLIT FILE SEPARATE BY ...`);
   - Селектор переменных `Groups Based on:` на базе `VariablePicker` и адаптивной кнопки `TransferButton` (`->` / `<-`);
   - Группа опций упорядочивания `(o) Sort the file by grouping variables` / `( ) File is already sorted`;
   - Нижняя строка `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Архитектурная декомпозиция и строгий лимит строк:**
   - Для предотвращения превышения лимита `file-guard` $\le 1000$ строк на файл в модулях управления диалогами проведена декомпозиция `crates/statcraft_app/src/app_dialogs.rs`: диалоги управления данными вынесены в чистый модуль `app_dialogs_data.rs` (455 строк).
6. **Интеграция, статусная строка и реактивность:**
   - В строке состояния (Status Bar) отображается векторный индикатор активного расщепления `Split File: ON (var_name)` / `Файл разделен: Включен (var_name)`;
   - Активирован пункт меню `Data -> Split File...`;
   - Логирование канонического синтаксиса в `Output Viewer` (Log).
7. **Контроль качества (Definition of Done):**
   - Полный пайплайн `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Все тесты воркспейса завершились успешно (0 сбоев);
   - Соблюдение `file-guard`, `design-guard`, `layers`, `parity` (34 команды);
   - Синхронная локализация (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Чистая сборка и развертывание релизного бинарника `C:\StatCraft\statcraft_app.exe`.

---

### Этап 8.6: Взвешивание наблюдений (Weight Cases: `WEIGHT BY` / `WEIGHT OFF`)
*Время: 29 октября 2026, 14:05 (~40 минут)*

Реализован один из важнейших аналитических механизмов IBM SPSS Statistics для социологии, маркетинга и работы с агрегированными табличными данными — процедура **Взвешивания наблюдений** (*Weight Cases*, пункт меню `Данные -> Взвесить наблюдения...` / `Data -> Weight Cases...`, команда `data.weight_cases` / синтаксис `WEIGHT BY var.` / `WEIGHT OFF.`):

1. **Модель взвешивания в DataSet L1 (`statcraft_core::dataset`):**
   - В структуру `DataSet` внедрено поле `weight_var: Option<String>` с методами `set_weight_variable()`, `weight_var()`, `is_weighted()` и `row_weight(row_idx)`;
   - Реализованы строгие канонические правила SPSS: веса обязаны быть неотрицательными ($w_i \ge 0$), наблюдения с нулевыми ($w_i = 0$), отрицательными ($w_i < 0$) или системно пропущенными (`NaN` / `SYSMIS`) весами автоматически исключаются из расчетов;
   - Бесшовная синергия с фильтром `Select Cases`: при активной фильтрации вес строки учитывается только для активных наблюдений (`active_row_indices`), исключая любые коллизии;
   - Автоматическая синхронизация словаря: при удалении (`remove_variable`) или переименовании (`rename_variable`) весовой переменной состояние взвешивания безопасно обновляется или сбрасывается без паник.

2. **Математическое ядро L2 (`statcraft_algo`):**
   - **Частоты (`frequencies.rs`):** расчет взвешенных частот категорий ($f_w = \sum w_i$), взвешенных валидных и накопленных процентов, а также перцентилей и квартилей с накоплением весов;
   - **Описательные статистики (`descriptives.rs`):** модуль `compute_descriptives_weighted` с вычислением взвешенного среднего $\bar{x}_w = \frac{\sum w_i x_i}{\sum w_i}$, взвешенной дисперсии $s_w^2 = \frac{\sum w_i (x_i - \bar{x}_w)^2}{W - 1}$ (где $W = \sum w_i$), стандартной ошибки среднего, а также взвешенных оценок асимметрии (skewness) и эксцесса (kurtosis);
   - **Кросстабуляция (`crosstabs.rs`):** расчет взвешенных матриц сопряженности $R \times C$, маргинальных итогов и критерия независимости хи-квадрат Пирсона с учетом весов частот;
   - 100% safe Rust, нулевой оверхед при выключенном взвешивании.

3. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::weight_cases_cmd`):**
   - Типизированная команда `WeightCasesCommand` (`data.weight_cases`) со спецификацией параметров `specs()` для внешних AI-агентов;
   - Генератор канонического синтаксиса SPSS:
     ```text
     WEIGHT BY count.
     ```
     или `WEIGHT OFF.` при отключении режима;
   - Полная поддержка истории `Undo` / `Redo` (`Ctrl+Z` / `Ctrl+Y`), восстанавливающая предыдущую весовую переменную.

4. **Пользовательский интерфейс на DialogKit L6 (`statcraft_app::dialogs`):**
   - Центрированный диалог `WeightCasesDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Левая панель `VariablePicker` с отображением иконок шкал измерения и векторная кнопка переноса `TransferButton` (`->` / `<-`);
   - Группа опций режима:
     1. *(o) Do not weight cases* (`WEIGHT OFF.`);
     2. *( ) Weight cases by:* с целевым полем переменной частоты;
   - Информационный блок текущего состояния выборки (`Current Status: Weight Off` / `Weight on by <var>`);
   - Нижний ряд действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

5. **Статусная строка, меню и реактивность:**
   - В строке состояния (Status Bar) выведен четкий индикатор: `Weight: ON (var)` / `Вес: Включен (var)`;
   - Активирован пункт главного меню `Data -> Weight Cases...`;
   - Логирование канонического синтаксиса в журнал `Output Viewer` (Log).

6. **Контроль качества (Definition of Done):**
   - 100% прохождение полного CI-пайплайна `cargo run -p xtask -j 1 -- ci` (149 файлов Rust проверены в `file-guard`, `design-guard`, `layers`, `parity` — 35 команд);
   - 100% тестов воркспейса завершились успешно (0 сбоев);
   - Синхронная локализация на трех языках (`locales/ru.ftl`, `locales/en.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Сборка и развертывание релизного бинарника `C:\StatCraft\statcraft_app.exe`.

---

### Этап 9.1: Линейная множественная регрессия (Multiple Linear Regression: `REGRESSION`)
*Время: 29 октября 2026, 16:30 (~45 минут)*

Реализован один из фундаментальных математических аппаратов статистического моделирования — процедура **Множественной линейной регрессии** (*Multiple Linear Regression*, пункт меню `Анализ -> Регрессия -> Линейная...` / `Analyze -> Regression -> Linear...`, команда движка `stat.regression.linear` / синтаксис `REGRESSION /DEPENDENT=... /METHOD=ENTER ...`):

1. **Математическое ядро L2 (`statcraft_algo::regression` и `regression_types`):**
   - Обычный МНК (OLS) и взвешенный МНК (WLS) с поддержкой частотных весов (`row_weight`) и активных выборок (`active_row_indices`);
   - Численно устойчивое обращение матриц нормальных уравнений $(X^T W X)$ методом Гаусса-Жордана с частичным выбором ведущего элемента (`invert_matrix`) на 100% safe Rust без внешних C-библиотек;
   - Оценка вектора параметров $\mathbf{B} = (X^T W X)^{-1} X^T W \mathbf{Y}$ и стандартных ошибок $SE(B_j) = s_e \sqrt{c_{jj}}$;
   - Стандартизованные коэффициенты бета: $\beta_j = B_j \frac{s_{X_j}}{s_Y}$;
   - Статистика Стьюдента $t_j = B_j / SE(B_j)$ и двусторонние уровни значимости $p$ через регулярную неполную бета-функцию;
   - Настраиваемые доверительные интервалы для коэффициентов $B$ с задаваемым уровнем (по умолчанию 95%);
   - Полный дисперсионный анализ регрессии (ANOVA): суммы квадратов $SS_{reg}, SS_{resid}, SS_{total}$, степени свободы $df_{reg} = p, df_{resid} = N - p - 1, df_{total} = N - 1$, средние квадраты $MS$, критерий Фишера $F$ и его асимптотическая значимость;
   - Сводка модели (Model Summary): $R = \sqrt{R^2}$, $R^2 = SS_{reg} / SS_{total}$, скорректированный $R^2_{adj} = 1 - \frac{MS_{resid}}{MS_{total}}$, стандартная ошибка оценки $s_e = \sqrt{MS_{resid}}$;
   - Диагностика мультиколлинеарности: толерантность ($Tolerance_j = 1 / r^{jj}$) и коэффициент инфляции дисперсии ($VIF_j = 1 / Tolerance_j$) через обращение корреляционной матрицы предикторов $R_{xx}^{-1}$;
   - **Golden Oracle:** Набор тестов `regression_tests.rs` с валидацией коэффициентов, $t$-статистик, $R^2$, ANOVA $F$, толерантности и VIF против эталонных вычислений Python `scipy.stats` и `numpy.linalg.lstsq` с точностью $\le 10^{-6}$;
   - Защита **Never Crash**: безопасная обработка сингулярных матриц, мультиколлинеарности, малых выборок ($N \le p + 1$), нулевой дисперсии предикторов и пропущенных значений на 100% safe Rust без паник.

2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::regression_linear_cmd`):**
   - Типизированная команда `LinearRegressionCommand` (`stat.regression.linear`) со спецификацией параметров `specs()` для внешних ИИ-агентов;
   - Построчное исключение наблюдений с пропусками (Listwise missing deletion) по всем зависимым и независимым переменным модели;
   - Полная совместимость с фильтрацией `Select Cases` (`active_row_indices`) и взвешиванием `Weight Cases` (`row_weight`);
   - Генератор канонического синтаксиса IBM SPSS Statistics:
     ```text
     REGRESSION
       /DEPENDENT=salary
       /METHOD=ENTER educ salbegin
       /STATISTICS=COEFF CI(95) R ANOVA COLLIN.
     ```
   - Полная поддержка отката `Undo` / `Redo`.

3. **Графический интерфейс на DialogKit L6 (`statcraft_app::dialogs::regression_dialog`):**
   - Центрированный диалог $560 \times 430$ px на базе `DialogShell::standard`;
   - Выбор переменных `VariablePicker` с иконками шкал и адаптивными векторными кнопками переноса `TransferButton` (`->` / `<-`);
   - Целевые поля: `Dependent:` (1 зависимая переменная) и `Independent(s):` (список предикторов);
   - Выпадающий список метода включения: `Method: Enter`;
   - Боковая панель `SubdialogSidebar` с кнопкой `[ Statistics... ]` (`[ Статистика... ]`);
   - Модальный поддиалог настроек статистики ($360 \times 300$ px, `DialogShell::subdialog`):
     * Чекбоксы `[x] Estimates` (Оценки коэффициентов), `[x] Confidence intervals` (Доверительные интервалы с настраиваемым уровнем 95%), `[x] Model fit` (Качество модели), `[x] Collinearity diagnostics` (Диагностика коллинеарности);
     * Нижний ряд кнопок `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Нижний ряд кнопок действий диалога `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Сводные таблицы в Output Viewer (`tables_regression.rs`):**
   - Таблица **Variables Entered/Removed:** модель, введенные переменные, удаленные переменные, метод;
   - Таблица **Model Summary:** Model, $R$, $R$ Square, Adjusted $R$ Square, Std. Error of the Estimate;
   - Таблица **ANOVA:** Model, Regression / Residual / Total ($SS, df, MS, F$, Sig.);
   - Таблица **Coefficients:** Model, Unstandardized Coefficients ($B, Std. Error$), Standardized Coefficients ($Beta$), $t$, Sig., 95.0% Confidence Interval for $B$ (Lower / Upper Bound), Collinearity Statistics ($Tolerance, VIF$);
   - Экспорт таблиц в TSV и интеграция в Outline Pane со специальными процедурными иконками.

5. **Реактивность, меню и статусная строка:**
   - Активирован пункт главного меню `Analyze -> Regression -> Linear...` (`Анализ -> Регрессия -> Линейная...`);
   - Автоматическое логирование канонического синтаксиса в журнал `Output Viewer` (Log);
   - Информационное уведомление в строке состояния (`Linear regression executed: <var>`).

6. **Контроль качества (Definition of Done):**
   - Полный пайплайн качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 150+ тестов воркспейса завершились успешно (0 сбоев);
   - Все файлы воркспейса строго соблюдают дифференцированные лимиты строк (`file-guard` $\le 500$ строк для листовых модулей, декомпозиция тестовых данных в `output_viewer_invariants/fixtures.rs`);
   - Линтеры дизайна и слоев (`design-guard`, `layers`, `parity` — 36 документированных процедур) — 0 замечаний;
   - Полная синхронная локализация на трех языках (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Компилятор и линтер `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Сборка и развертывание релизного бинарника `C:\StatCraft\statcraft_app.exe`.




---

### Этап 10.1: Анализ надежности шкалы (Reliability Analysis: `RELIABILITY`)
*Время: 29 октября 2026, 17:35 (~40 минут)*

Реализована фундаментальная процедура психометрического, социологического и прикладного статистического анализа шкал и опросников — **Анализ надежности** (*Reliability Analysis*, пункт меню `Анализ -> Шкала -> Анализ надежности...` / `Analyze -> Scale -> Reliability Analysis...`, команда движка `stat.scale.reliability` / синтаксис `RELIABILITY /VARIABLES=... /SCALE('ALL VARIABLES') ALL /MODEL=ALPHA`):

1. **Математическое ядро L2 (`statcraft_algo::reliability` и `reliability_types`):**
   - Расчет коэффициента внутренней согласованности $\alpha$ Кронбаха: $\alpha = \frac{k}{k - 1} \left( 1 - \frac{\sum s_i^2}{s_{total}^2} \right)$;
   - Расчет стандартизованного коэффициента $\alpha$ на основе средней межпунктовой корреляции $\bar{r}$: $\alpha_{std} = \frac{k \bar{r}}{1 + (k - 1) \bar{r}}$;
   - Построчное исключение пропущенных данных (Listwise deletion) по всем пунктам шкалы;
   - Сводка обработки наблюдений (*Case Processing Summary*): валидные ($N, \%$), исключенные ($N, \%$) и всего ($N, 100.0\%$);
   - Статистика пунктов (*Item Statistics*): среднее ($\bar{x}_i$), стандартное отклонение ($s_i$), валидное число наблюдений ($N$);
   - Статистика пункта и шкалы (*Item-Total Statistics*):
     * Среднее шкалы при исключении пункта: $\mu_{-i} = \mu_{total} - \bar{x}_i$;
     * Дисперсия шкалы при исключении пункта: $s_{-i}^2 = s_{total}^2 + s_i^2 - 2 \text{Cov}(X_i, X_{total})$;
     * Скорректированная корреляция пункта с общим баллом: $r_{i(total - i)} = \frac{\text{Cov}(X_i, X_{total}) - s_i^2}{\sqrt{s_i^2 \cdot s_{-i}^2}}$;
     * Альфа Кронбаха при исключении пункта: $\alpha_{-i} = \frac{k - 1}{k - 2} \left( 1 - \frac{\sum_{j \neq i} s_j^2}{s_{-i}^2} \right)$ при $k > 2$;
   - Статистика суммарной шкалы (*Scale Statistics*): среднее, дисперсия, стандартное отклонение, число пунктов;
   - Полная поддержка взвешенных наблюдений (`row_weight`) и фильтров (`active_row_indices`);
   - **Golden Oracle:** Набор тестов `reliability_tests.rs` со сверкой формул против эталонов с аналитической точностью $\le 10^{-6}$;
   - Защита **Never Crash**: безопасная обработка $k < 2$, вырожденных нулевых дисперсий, константных ответов и пропусков на 100% safe Rust без паник.

2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::reliability_cmd`):**
   - Типизированная команда `ReliabilityCommand` (`stat.scale.reliability`) со спецификацией `specs()` для внешних ИИ-агентов;
   - Генератор канонического синтаксиса IBM SPSS Statistics:
     ```text
     RELIABILITY
       /VARIABLES=q1 q2 q3 q4
       /SCALE('ALL VARIABLES') ALL
       /MODEL=ALPHA
       /STATISTICS=DESCRIPTIVES SCALE
       /SUMMARY=TOTAL.
     ```
   - Полная поддержка `Undo` / `Redo`.

3. **Графический интерфейс на DialogKit L6 (`statcraft_app::dialogs::reliability_dialog`):**
   - Центрированный диалог $560 \times 430$ px на базе `DialogShell::standard`;
   - Двухпанельный `VariablePicker` со шкалами измерения и векторными кнопками переноса `TransferButton` (`->` / `<-`);
   - Список анализируемых пунктов `Items:` (минимум 2 переменные для активации кнопки `[ OK ]`);
   - Выпадающий список выбора модели: `Model: Alpha`;
   - Поле ввода метки шкалы: `Scale label:` (по умолчанию `ALL VARIABLES`);
   - Боковая панель `SubdialogSidebar` с кнопкой `[ Statistics... ]` (`[ Статистика... ]`);
   - Модальный поддиалог настроек статистики ($360 \times 300$ px, `DialogShell::subdialog`):
     * Чекбоксы `[x] Item` (Пункта), `[x] Scale` (Шкалы), `[x] Scale if item deleted` (Шкалы при исключении пункта);
     * Нижний ряд кнопок `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Нижний ряд действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Сводные таблицы в Output Viewer (`tables_reliability.rs`):**
   - Таблица **Case Processing Summary:** Cases (Valid, Excluded, Total) с $N$ и $\%$, примечание о построчном исключении;
   - Таблица **Reliability Statistics:** Cronbach's Alpha, Cronbach's Alpha Based on Standardized Items, N of Items;
   - Таблица **Item Statistics:** Item, Mean, Std. Deviation, N;
   - Таблица **Item-Total Statistics:** Item, Scale Mean if Item Deleted, Scale Variance if Item Deleted, Corrected Item-Total Correlation, Cronbach's Alpha if Item Deleted;
   - Таблица **Scale Statistics:** Mean, Variance, Std. Deviation, N of Items;
   - TSV-экспорт в буфер обмена и интеграция в Outline Tree.

5. **Реактивность, меню и статусная строка:**
   - Активирован пункт меню `Analyze -> Scale -> Reliability Analysis...` (`Анализ -> Шкала -> Анализ надежности...`);
   - Автоматическая запись синтаксиса в журнал Output Viewer;
   - Уведомление в строке состояния (`Reliability analysis calculated for <scale> (<n> items)`).

6. **Контроль качества (Definition of Done):**
   - Полный CI-пайплайн `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 160+ тестов воркспейса завершились успешно (0 сбоев);
   - Соблюдение предельных размеров файлов (`file-guard` $\le 500$ строк на файл);
   - Соблюдение стандартов дизайна (`design-guard` — 100% канонический DialogKit);
   - Проверка слоев `layers` и паритета `parity` (37 документированных процедур) — 0 нарушений;
   - Полная синхронная локализация на трех языках (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Сборка и развертывание релизного бинарника `C:\StatCraft\statcraft_app.exe`.

---

### Этап 11.1: Факторный анализ и метод главных компонент (Factor Analysis / PCA: `FACTOR`)
*Время: 29 октября 2026, 18:25 (~35 минут)*

Реализован ключевой компонент блока снижения размерности (Dimension Reduction) экосистемы IBM SPSS Statistics — **Факторный анализ / Метод главных компонент** (`Analyze -> Dimension Reduction -> Factor...` / `Анализ -> Снижение размерности -> Факторный анализ...`), дополняющий анализ надежности шкалы (Cronbach's Alpha) для комплексной психометрической и многомерной валидизации:

1. **Математическое ядро L2 (`statcraft_algo::factor`, `factor_types`, `factor_tests`):**
   - Построчное исключение пропусков (Listwise missing deletion) и поддержка весов наблюдений (`row_weight`);
   - Расчет парных коэффициентов корреляции Пирсона и взвешенной корреляционной матрицы $R$ ($p \times p$);
   - Расчет мер факторизуемости и выборочной адекватности:
     * **Kaiser-Meyer-Olkin (KMO) Measure of Sampling Adequacy**: вычисление антиобразных частных корреляций $a_{ij} = -s_{ij} / \sqrt{s_{ii} s_{jj}}$ через численное обращение матрицы $S = R^{-1}$ и расчет критерия $\text{KMO} = \frac{\sum_{i \ne j} r_{ij}^2}{\sum_{i \ne j} r_{ij}^2 + \sum_{i \ne j} a_{ij}^2}$;
     * **Bartlett's Test of Sphericity**: критерий сферичности Бартлетта $\chi^2 = -(n - 1 - (2p + 5)/6) \ln |R|$ при $df = p(p - 1)/2$ с расчетом точного $p$-value через функцию выживания распределения хи-квадрат;
   - Метод выделения главных компонент (**PCA**):
     * Спектральное разложение вещественной симметричной матрицы $R$ через циклический метод Якоби со сдвигами на 100% safe Rust без внешних зависимостей (`jacobi_eigen_symmetric`);
     * Каноническое правило Кайзера (Kaiser criterion: отбор компонент с собственными значениями $\lambda \ge 1.0$) либо фиксированное число факторов;
     * Каноническая ориентация знаков факторных нагрузок для положительной доминантности;
     * Расчет факторных нагрузок $l_{ik} = \sqrt{\lambda_k} \cdot v_{ik}$;
     * Общности (*Communalities*): начальные $1.0$ для всех переменных, выделенные $h_i^2 = \sum_{k=1}^m l_{ik}^2$;
     * Полная объясненная дисперсия (*Total Variance Explained*): начальные собственные значения, процент дисперсии, кумулятивный процент и суммы квадратов нагрузок извлечения;
     * Матрица компонент (*Component Matrix*);
   - **Golden Oracle:** Тесты `factor_tests.rs` со сверкой собственных значений, дисперсий, факторных нагрузок, KMO и критерия Бартлетта против Python `scipy.linalg.eigh` и `factor_analyzer` с аналитической точностью $\le 10^{-6}$;
   - Защита **Never Crash**: безопасная обработка сингулярных матриц, малого $n < 2$, вырожденных нулевых дисперсий и пропусков.

2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::factor_cmd`):**
   - Типизированная команда `FactorCommand` (`stat.factor`) со спецификацией параметров `specs()` для внешних ИИ-агентов;
   - Генератор и парсер канонического синтаксиса IBM SPSS Statistics:
     ```text
     FACTOR
       /VARIABLES q1 q2 q3 q4 q5
       /MISSING LISTWISE
       /ANALYSIS q1 q2 q3 q4 q5
       /PRINT INITIAL EXTRACTION KMO
       /CRITERIA MINEIGEN(1)
       /EXTRACTION PC
       /ROTATION NOROTATE.
     ```
   - Полная поддержка `Undo` / `Redo`.

3. **Графический интерфейс на DialogKit L6 (`statcraft_app::dialogs::factor_dialog`):**
   - Центрированный диалог $560 \times 430$ px на базе `DialogShell::standard`;
   - Двухпанельный выбор `VariablePicker` со шкалами измерения и векторными кнопками переноса `TransferButton` (`->` / `<-`);
   - Список `Variables:` (минимум 2 переменные для активности кнопки `[ OK ]`);
   - Боковая панель `SubdialogSidebar` с вызовами настроек:
     * `[ Descriptives... ]`: модальный поддиалог с чекбоксом `[x] KMO and Bartlett's test of sphericity`;
     * `[ Extraction... ]`: модальный поддиалог выбора метода (PCA) и критерия отсечения факторов (Eigenvalues over 1.0 / Fixed number of factors);
     * Нижний ряд кнопок `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Нижний ряд действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Сводные таблицы в Output Viewer (`tables_factor.rs`):**
   - Таблица **KMO and Bartlett's Test:** Kaiser-Meyer-Olkin Measure of Sampling Adequacy, Bartlett's Test of Sphericity (Approx. Chi-Square, df, Sig.);
   - Таблица **Communalities:** Variable, Initial (1.000), Extraction;
   - Таблица **Total Variance Explained:** Component, Initial Eigenvalues (Total, % of Variance, Cumulative %), Extraction Sums of Squared Loadings;
   - Таблица **Component Matrix:** переменные $\times$ компоненты;
   - Экспорт таблиц в TSV (буфер обмена для Excel/Sheets) и навигация в Outline Tree.

5. **Реактивность, меню и статусная строка:**
   - Активирован пункт главного меню `Analyze -> Dimension Reduction -> Factor...` (`Анализ -> Снижение размерности -> Факторный анализ...`);
   - Журналирование синтаксиса в Output Viewer (Log);
   - Уведомление в строке состояния (`Factor analysis computed for <n> case(s), <m> component(s) extracted`).

6. **Контроль качества (Definition of Done):**
   - Полный CI-пайплайн `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Соблюдение дифференцированных лимитов строк (`file-guard` $\le 500$ строк на функциональный модуль);
   - Соблюдение стандартов дизайна (`design-guard` — 100% DialogKit);
   - Проверка архитектурных слоев `layers` и паритета `parity` (38 документированных процедур) — 0 нарушений;
   - Полная синхронная локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`);
   - Компиляция `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Сборка и развертывание бинарника в `C:\StatCraft\statcraft_app.exe`.

---

### Этап 12.1: Таблицы групповых средних и однофакторный дисперсионный анализ (Subgroup Means Breakdown: `MEANS`)
*Время: 29 октября 2026, 19:40 (~45 минут)*

Реализована базовая разведочная процедура для анализа подгрупп канонического блока **Compare Means** пакета IBM SPSS Statistics — **Таблицы групповых средних** (`Analyze -> Compare Means -> Means...` / `Анализ -> Сравнение средних -> Средние...`), объединяющая расчет расширенных дескриптивных статистик по уровням факторов, классический однофакторный дисперсионный анализ (ANOVA) и меры ассоциации:

1. **Математическое ядро L2 (`statcraft_algo::means`, `means_types`, `means_tests`):**
   - Построчное исключение пропусков (Listwise per table) и полная поддержка весов наблюдений (`row_weight`);
   - Поддержка числовых и категориальных группирующих факторов с естественной сортировкой (числовой порядок для чисел, лексикографический для строк);
   - Вычисление полного набора ячеечных статистик для каждой группы и сводной строки Total:
     * Количество наблюдений ($N$), среднее взвешенное ($\bar{x}$), несмещенное стандартное отклонение ($s$), выборочная дисперсия ($s^2$);
     * Медиана (с интерполяцией и учетом порядка), минимум ($\min$), максимум ($\max$), взвешенная сумма значений ($\sum x$);
   - **Таблица однофакторного дисперсионного анализа (ANOVA Table):**
     * Межгрупповая вариация (Between Groups): $SS_{between} = \sum n_j (\bar{x}_j - \bar{x})^2$, $df = k - 1$;
     * Внутригрупповая вариация (Within Groups): $SS_{within} = \sum \sum (x_{ij} - \bar{x}_j)^2$, $df = N - k$;
     * Общая вариация (Total): $SS_{total} = SS_{between} + SS_{within}$, $df = N - 1$;
     * Средние квадраты ($MS$), критерий Фишера ($F = MS_{between} / MS_{within}$) и точное $p$-value через функцию выживания распределения Фишера (`f_distribution_sf`);
   - **Меры ассоциации (Measures of Association):**
     * Коэффициент детерминации $\eta^2 = SS_{between} / SS_{total}$ (доля объясненной дисперсии зависимой переменной фактором) и корреляционное отношение $\eta = \sqrt{\eta^2}$;
   - **Golden Oracle:** Тесты `means_tests.rs` со сверкой ячеечных статистик, ANOVA и $\eta$ против аналитических эталонов и Python `scipy.stats.f_oneway` с точностью $\le 10^{-6}$;
   - Защита **Never Crash**: безопасная обработка пустых выборок, групп размера $N < 2$, вырожденной нулевой дисперсии и пропущенных значений на 100% safe Rust.

2. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::means_cmd`):**
   - Типизированная команда `MeansCommand` (`stat.means`) со спецификацией параметров `specs()` для внешних ИИ-агентов;
   - Генератор канонического синтаксиса IBM SPSS Statistics:
     ```text
     MEANS TABLES=salary score BY educ
       /CELLS=MEAN COUNT STDDEV VARIANCE MEDIAN MIN MAX SUM
       /STATISTICS ANOVA.
     ```
   - Полная поддержка `Undo` / `Redo`.

3. **Графический интерфейс на DialogKit L6 (`statcraft_app::dialogs::means_dialog`):**
   - Центрированный диалог $560 \times 430$ px на базе `DialogShell::standard`;
   - Источник переменных `VariablePicker` со значками шкал измерения;
   - Адаптивные векторные стрелки переноса `TransferButton` (`->` / `<-`);
   - Целевые списки `Dependent List:` (зависимые переменные) и `Layer 1 of 1:` (независимые группирующие переменные);
   - Боковая панель `SubdialogSidebar` с кнопкой `[ Options... ]`:
     * Модальный поддиалог со списком включаемых статистик (Mean, Number of Cases, Std. Deviation, Variance, Median, Minimum, Maximum, Sum) и чекбоксом `[x] Anova table and eta`;
     * Нижний ряд кнопок `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Нижний ряд действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Сводные таблицы в Output Viewer (`tables_means.rs`):**
   - Таблица **Case Processing Summary:** Зависимая $\times$ Группирующая, Included ($N$, %), Excluded ($N$, %), Total ($N$, %);
   - Таблица **Report:** групповая разбивка уровней фактора со статистиками (Mean, N, Std. Deviation, Variance, Median, Minimum, Maximum, Sum) и итоговой строкой Total;
   - Таблица **ANOVA Table:** Sum of Squares, df, Mean Square, F, Sig. для Between Groups, Within Groups и Total;
   - Таблица **Measures of Association:** Eta, Eta Squared;
   - Экспорт таблиц в TSV (буфер обмена для Excel/Sheets) и навигация в Outline Tree.

5. **Реактивность, меню и статусная строка:**
   - Активирован пункт главного меню `Analyze -> Compare Means -> Means...` (`Анализ -> Сравнение средних -> Средние...`);
   - Журналирование канонического синтаксиса в Output Viewer (Log);
   - Уведомление в строке состояния (`Means breakdown computed for <n> case(s)`).

6. **Контроль качества (Definition of Done):**
   - Полный CI-пайплайн `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Соблюдение дифференцированных лимитов строк (`file-guard` $\le 500$ строк на функциональный модуль);
   - Соблюдение стандартов дизайна (`design-guard` — 100% DialogKit);
   - Проверка архитектурных слоев `layers` и паритета `parity` (39 документированных процедур) — 0 нарушений;
   - Полная синхронная локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Компиляция `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений;
   - Сборка и развертывание бинарника в `C:\StatCraft\statcraft_app.exe`.

---

### Этап MP-1: Жизненный цикл сессии и меню «Файл» (Session Lifecycle & File Menu)
*Время: 29 октября 2026, 21:30 (~25 минут)*

Оживлены базовые пункты управления жизненным циклом сессии и навигации в главном меню `File` приложения StatCraft (`crates/statcraft_app`), переведенные из статуса `MenuItemStatus::InDev` в полноценно рабочий `MenuItemStatus::Ready`:
1. **Менеджер недавних файлов L6 (`recent_files.rs`):**
   - Структура `RecentFiles` с хранением очереди до 10 последних открытых файлов;
   - Автоматическая ротация: при открытии файл помещается на первое место списка, дубликаты поднимаются наверх, при превышении 10 записей старые отсекаются;
   - Двойная персистентность: сохранение в `.statcraft/recent_files.json` и синхронизация с `eframe::Storage` при закрытии;
   - Интерактивное подменю `File -> Recent Files`: всплывающие подсказки с полными путями, неактивное состояние `(No recent files)` при пустом списке и разделитель с кнопкой `Clear Recent Files`.
2. **Жизненный цикл сессии и координация L6 (`app_session.rs`):**
   - `File -> New -> Data` (`Ctrl+N`): сброс датасета в чистый бланк, очистка `current_file_path`, сброс командной шины `CommandBus` и параметров фильтрации/весов/расщепления, переключение на вкладку `Data View`, уведомление в строке состояния и запись команды `NEW DATASET.` в журнал `Output Viewer`;
   - `File -> New -> Syntax`: перевод фокуса на редактор синтаксиса `Syntax Editor`, инициализация канонического стартового заголовка с текущей датой и временем `* StatCraft Syntax Editor. / * Created: <timestamp>.`, уведомление в статусе;
   - `File -> New -> Project` (`Ctrl+Shift+N`): комплексный сброс приложения (датасет, журнал вывода, синтаксис, путь к файлу);
   - `File -> Close Project`: закрытие текущего проекта с очисткой данных и отчетов, возврат статус-бара в `Ready`;
   - Проверка несохраненных изменений `has_unsaved_changes()`: при попытке создания или открытия при наличии данных/правок открывается центрированный модальный диалог подтверждения `ConfirmSaveDialog` (`[ Save ] [ Don't Save ] [ Cancel ]`) на базе `DialogShell::subdialog`.
3. **Редактор синтаксиса L6 (`views/syntax_editor.rs`):**
   - Интерактивный компонент `SyntaxEditorState` и функция отрисовки `render_syntax_editor`;
   - Моноширинный многострочный редактор кода, зарегистрированный в докинг-системе приложения (`Tab::SyntaxEditor`);
   - Быстрый переход через `Window -> Syntax Editor` (`Ctrl+4`).
4. **Контроль качества (Definition of Done):**
   - Создан отдельный набор модульных и интеграционных тестов `crates/statcraft_app/tests/session_lifecycle_tests.rs` (ротация recent files, проверка детекции правок, комплексные сценарии сброса сессий, headless-рендер);
   - Полное прохождение пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - Жесткое соблюдение `file-guard`: координатор `app.rs` строго в рамках лимита (1407 из 1500 строк), вспомогательные модули $\le 500$ строк;
   - Строгий линтер дизайна `design-guard` — 100% OK;
   - Полная синхронная локализация без единого хардкода строк (`locales/en.ftl`, `locales/ru.ftl`);
   - `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений.

---

### Этап MP-3: Экспорт данных и отчетов вывода (Export Data & Export Output)
*Время: 29 октября 2026, 22:15 (~35 минут)*

Реализованы полноценные функции экспорта данных и результатов статистического анализа для внешнего использования, публикаций и обмена в меню `File -> Export`:
1. **Экспорт данных L0/L1 (`statcraft_io::csv_writer`, `statcraft_core::csv_export`):**
   - Модуль `csv_writer.rs` в слое L0 с поддержкой структуры настроек `CsvExportOptions` (разделители запятая, точка с запятой, табуляция, кавычки, экранирование спецсимволов `\n`, `\r`, `"`, десятичные разделители и представление системных пропусков `sysmis_repr`);
   - Модуль `csv_export.rs` в слое L1 с функцией `export_dataset_to_csv` для экспорта `DataSet` с автоматической подстановкой меток значений `ValueLabels` (или исходных числовых кодов), безопасной обработкой пустых наборов данных (0 строк/колонок) и потоковой записью в `std::io::Write`;
   - Регистрация фильтров диалога сохранения: `Comma-Separated Values (*.csv)`, `Tab-Delimited Values (*.tsv, *.txt)`, `Semicolon-Separated Values (*.csv)` с автоопределением параметров по расширению.
2. **Архитектурный слой отчетов L4 (`crates/statcraft_reporting`):**
   - Создан новый независимый крейт слоя L4 `statcraft_reporting` (зарегистрирован в `xtask/src/layers.rs` с соблюдением строгой иерархии зависимостей L4 $\to$ Utility, L6 $\to$ L4);
   - Универсальная модель отчетов: `ReportDocument`, `ReportSection`, `ReportItem` (Log, Notes, Table, Text, Warning), `ReportTable`, `ReportNotes`, `CellAlignment`;
   - Генератор HTML-отчетов `html_export.rs`: единый самодостаточный HTML5-документ со встроенными аутентичными стилями таблиц SPSS (гарнитура sans-serif, рамки шапки таблицы, чередование строк, акцентные заголовки, сноски мелким курсивом, темные блоки логов синтаксиса, безопасное экранирование спецсимволов HTML);
   - Генератор моноширинного текста `txt_export.rs`: адаптивное форматирование в ASCII-таблицы с рамками и выравниванием;
   - Генератор пакетного TSV `tsv_export.rs`: объединение всех таблиц сессии через пустые строки для прямой вставки в электронные таблицы (Excel, Google Sheets).
3. **Графический интерфейс и интеграция L6 (`crates/statcraft_app`):**
   - Модуль `views/output_view/export.rs`: преобразование `OutputViewerState` в `ReportDocument` с извлечением заголовков, данных и сносок для всех процедур;
   - Модуль `export_handlers.rs`: пункты меню `File -> Export -> Export Output...` и `File -> Export -> Export Data...` (с предикатом доступности данных `requires_dataset_status`);
   - Вызовы нативных диалогов сохранения `rfd::FileDialog::save_file()`;
   - Уведомления в строке состояния (`status-data-exported`, `status-output-exported`);
   - Протоколирование команд в журнале синтаксиса `Output Viewer`: `SAVE TRANSLATE OUTFILE='...' /TYPE=... /REPLACE /FIELDNAMES.` и `OUTPUT EXPORT /CONTENTS=ALL /TYPE=... /DESTINATION='...'`.
4. **Контроль качества (Definition of Done):**
   - 100% прохождение полного CI gate: `cargo run -p xtask -j 1 -- ci` (`fmt`, `clippy -D warnings`, `cargo test --workspace -j 1`, `parity`);
   - Строгая проверка архитектурных слоев `cargo run -p xtask -j 1 -- layers` (L4Reporting);
   - Соблюдение лимитов строк `file-guard` ($\le 500$ строк на новые модули);
   - Соблюдение дизайн-системы `design-guard`;
   - Полная синхронная локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`);
   - Покрытие новыми тестами: `statcraft_io::csv_writer`, `statcraft_core::csv_export`, `statcraft_reporting` (HTML, TXT, TSV), `statcraft_app::export_handlers` и `output_view::export`.

---

### Этап MP-5: Импорт бинарных файлов SPSS (.sav / .zsav)
*Время: 29 октября 2026, 22:20 (~30 минут)*

Реализована полноценная поддержка чтения оригинальных бинарных файлов IBM SPSS Statistics (`.sav` и сжатых `.zsav`) с полным извлечением данных и словаря метаданных без использования сторонних проприетарных библиотек:
1. **Низкоуровневый декодер L0 (`crates/statcraft_io::spss_loader`):**
   - Модульная архитектура: `header.rs`, `records.rs`, `decompress.rs`, `mod.rs` (каждый файл $\le 500$ строк по правилам `file-guard`);
   - Чтение системного заголовка Record 0 (`$FL2` / `$FL3`, автоопределение порядка байт Little/Big Endian по `layout_code`, коэффициент сжатия `bias = 100.0`, индекс весовой переменной `weight_index`, число наблюдений `case_cnt`);
   - Обработка записей переменных Record 2: типы данных (числовые и строки), метки переменных, коды пользовательских пропусков (1–3 дискретных значения, диапазоны значений);
   - Обработка меток значений Record 3 и 4: парные списки `(f64, label)` и привязка к соответствующим переменным;
   - Обработка записей расширений Record 7: подтип 4 (Display parameters: шкалы измерения Measure Nominal/Ordinal/Scale, выравнивание Left/Right/Center, ширина колонок), подтип 13 (Long Variable Names: отображение коротких 8-байтовых имен в современные длинные имена колонок в UTF-8), подтип 20 (Character Encoding), подтип 24 (ZLIB metadata для файлов `.zsav`);
   - Потоковый распаковщик байт-кодов Record 999: декодирование команд 1..251 (`code - bias`), 252 (EOF), 253 (8-байтовые IEEE 754 float / строковые блоки), 254 (пробелы), 255 (системные пропуски `SYSMIS` / `Null`);
   - Поддержка формата `.zsav`: чтение трейлера `ztrailer`, пакетная распаковка сжатых блоков через `flate2::read::ZlibDecoder` в единый поток байт-кодов.
2. **Интеграция с ядром L1 (`crates/statcraft_core::spss`):**
   - Модуль `spss.rs`: методы `DataSet::from_spss_data`, `DataSet::load_spss_from_path`, `DataSet::load_spss_from_reader`;
   - Конвертация структуры `SpssVariableInfo` в канонические типы `VariableMetadata`: 9 типов данных SPSS, шкалы `Measure`, выравнивание `Alignment`, словарь меток значений `ValueLabels`, пропуски `MissingValues`, переменная весов `weight_var`;
   - Строгая типизация ошибок `CoreError::Io(#[from] statcraft_io::IoError)`.
3. **Командная шина L5 (`crates/statcraft_syntax`):**
   - Типизированная команда `ImportSpssCommand` (`data.import_spss`) со спецификацией `specs()`, выполнением и поддержкой Undo;
   - Регистрация в `CommandBus::with_defaults()`;
   - Генератор канонического синтаксиса `GET FILE='<path>'.`;
   - Расширение команды `OpenFileCommand` (`data.open_file`) автоматической поддержкой расширений `.sav` и `.zsav`.
4. **Графический интерфейс L6 (`crates/statcraft_app`):**
   - Активирован пункт меню `File -> Import Data -> SPSS Data...` (`menu-file-import-spss`) с вызовом диалога выбора файлов SPSS Statistics Data (`*.sav, *.zsav`);
   - Добавлен фильтр файлов данных SPSS в общий диалог `File -> Open Data...` (`open_data_dialog`);
   - В `app_session.rs` метод `load_file_from_path` автоматически определяет формат `.sav`/`.zsav`, импортирует датасет через командную шину, протоколирует команду `GET FILE='<path>'.` в Output Viewer, добавляет файл в `RecentFiles` и переключает фокус на вкладку `Data View`.
5. **Контроль качества (Definition of Done):**
   - 100% прохождение полного CI gate: `cargo run -p xtask -j 1 -- ci` (`fmt`, `clippy -D warnings`, `cargo test --workspace -j 1`, `parity`, `layers`, `file-guard`, `design-guard`);
   - Модульные тесты `statcraft_io::spss_loader::tests`: байт-коды, несжатый формат, обработка поврежденных заголовков и некорректных записей без паник;
   - Интеграционный тест жизненного цикла `crates/statcraft_app/tests/session_lifecycle_tests.rs::test_load_spss_sav_session_lifecycle`;
   - 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`);
   - 100% Safe Rust без `unwrap`/`expect`/`panic` в нетестовом коде.

---

### Этап MP-4: Импорт файлов Microsoft Excel (.xlsx / .xls / .ods)
*Время: 29 октября 2026, 23:05 (~35 минут)*

Реализована возможность прямого открытия и импорта электронных таблиц Microsoft Excel и OpenDocument без предварительной конвертации в CSV:
1. **Слой L0 (`statcraft_io::excel_loader`):**
   - Интеграция современного крейта `calamine = "0.37"` (100% safe Rust, потоковое чтение без вызовов COM или системных библиотек Office);
   - Функции `load_excel_from_path`, `load_excel_from_reader` и `load_excel_from_range` с поддержкой форматов `.xlsx`, `.xlsm`, `.xls`, `.ods`;
   - Алгоритм извлечения структуры листа:
     * Выбор первого листа книги по умолчанию (`workbook.sheet_names()[0]`) либо по явно переданному имени;
     * Каноническая санитизация и дедупликация имен переменных первой строки (`sanitize_and_dedup_headers`): замена недопустимых символов на `_`, добавление префикса `V_` для чисел, генерация `VAR00001` для пустых ячеек и суффиксов `_1`, `_2` при коллизиях;
     * Распознавание типов ячеек `calamine::Data` (целые числа `Int`, вещественные `Float`, булевы `Bool` -> `1.0`/`0.0`, даты/время `DateTime`/`DateTimeIso`, текстовые `String`, пропуски `Empty`/`Error`/`"NA"`/`"."` -> `RawValue::Null`);
     * Автоопределение типов колонок: если все непустые ячейки колонки являются числами -> колонка типизируется как `RawDataType::Numeric`, иначе все числовые значения приводятся к тексту и колонка становится `RawDataType::Text`;
   - Защита **Never Crash**: безопасная обработка пустых книг, листов без данных (0 строк), листов с одними заголовками.
2. **Командная шина движка L5 (`statcraft_syntax::commands::import_excel`):**
   - Типизированная команда `ImportExcelCommand` (`data.import_excel`) со спецификацией `specs()` (параметры `file_path`, `sheet_name`, `dataset_name`) для внешних ИИ-агентов;
   - Загрузка в активный `DataSet` и полная поддержка `Undo` (сброс датасета);
   - Регистрация в `CommandBus::with_defaults()`.
3. **Графический интерфейс и интеграция L6 (`crates/statcraft_app`):**
   - Активация пунктов меню `File -> Import Data -> Excel...` (`menu-file-import-excel`) и `File -> Import Data -> CSV...` (`menu-file-import-csv`);
   - Метод `open_excel_dialog` с системным диалогом `rfd::FileDialog` и фильтрами `Excel Spreadsheets (*.xlsx, *.xls, *.ods, *.xlsm)`;
   - Расширение обработчика `load_file_from_path`: автоопределение расширения файла (`xlsx`, `xls`, `ods`, `xlsm`), вызов команды `data.import_excel`, добавление пути в историю недавно открытых файлов `RecentFiles`;
   - Протоколирование канонической команды в Output Viewer:
     ```text
     GET DATA /TYPE=XLSX /FILE='<path>' /FIRSTROW=YES.
     ```
   - Уведомление в строке состояния: `Dataset imported: <name> (<n> cases, <m> variables)`.
4. **Контроль качества (Definition of Done):**
   - 100% прохождение полного пайплайна CI gate: `cargo run -p xtask -j 1 -- ci` (`cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace -j 1`, `parity`);
   - Прохождение архитектурного линтера `xtask layers` и `xtask parity` (40 команд отслеживается);
   - Полное соблюдение лимитов строк `file-guard`: модуль `excel_loader.rs` (344 строки из 500), координатор `app.rs` (1471 из 1500 строк);
   - Соблюдение дизайн-системы `design-guard`;
   - Полная синхронная локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`);
   - Модульные тесты парсера Excel в `statcraft_io`, тесты команды в `statcraft_syntax` и тесты жизненного цикла сессии.


---

### Этап MP-2: Нативный формат проектов (.stc) и команды сохранения (Native .stc Projects & Save Commands)
*Время: 29 октября 2026, 22:25 (~30 минут)*

Реализована полноценная поддержка нативного формата проектов StatCraft (`.stc`) и цепочки команд сохранения и открытия данных в главном меню `File`:
1. **Кодек нативного формата проектов L0 (`statcraft_io::stc_codec`):**
   - Структура контейнера проекта `StcContainer` (`format_version = 1`, `dataset_name`, `metadata_json`, `table: RawTable`, `active_filter_var`, `weight_var`, `split_file_json`);
   - Потоковое сжатие Gzip через `flate2::write::GzEncoder` с прозрачным распознаванием заголовка GZIP (`0x1f 0x8b`) и fallback-десериализацией открытого JSON при чтении;
   - Функции `save_stc_container`, `load_stc_container`, `save_stc_container_to_path`, `load_stc_container_from_path`;
   - Кодек CSV/TSV записи `crates/statcraft_io/src/csv_writer.rs` (`export_raw_table_to_csv_path`) с поддержкой экранирования и локализованных десятичных разделителей.
2. **Сериализация полного словаря метаданных L1 (`statcraft_core::stc_codec`):**
   - 100% сохранение и восстановление состояния датасета:
     * Все 9 канонических типов переменных (`Numeric`, `Comma`, `Dot`, `Scientific`, `Date`, `Dollar`, `CustomCurrency`, `String`, `RestrictedNumeric`);
     * Шкалы измерения (`Scale`, `Ordinal`, `Nominal`), выравнивание (`Left`, `Right`, `Center`), ширина и число знаков после запятой;
     * Полные словари меток значений `ValueLabels` (`BTreeMap<String, String>`);
     * Дискретные и диапазонные пропуски `MissingValues`;
     * Настройки активного фильтра (`active_filter_var`), переменной взвешивания (`weight_var`) и конфигурации расщепления (`split_file`);
   - Унифицированные методы `save_stc_to_writer`, `load_stc_from_reader`, `save_stc_to_path`, `load_stc_from_path` и `save_csv_to_path`.
3. **Командная шина и синтаксис L5 (`statcraft_syntax::commands::file_cmds`):**
   - Команда сохранения `SaveFileCommand` (`data.save_file`) со спецификацией параметров `specs()` и генератором канонического синтаксиса `SAVE OUTFILE='<path>'.\nEXECUTE.`;
   - Команда открытия `OpenFileCommand` (`data.open_file`) с генератором синтаксиса `GET FILE='<path>'.\nEXECUTE.`;
   - Регистрация в `CommandBus::with_defaults()`.
4. **Интеграция в главное меню и диалоги L6 (`statcraft_app::app_session` и `app.rs`):**
   - Перевод пунктов меню `File -> Save` (`Ctrl+S`), `File -> Save As...` (`Ctrl+Shift+S`), `File -> Save Data As...`, `File -> Open Project...` (`Ctrl+Shift+O`), `File -> Open Data...` (`Ctrl+O`) в статус `MenuItemStatus::Ready`;
   - Интеграция системных диалогов выбора файлов `rfd::FileDialog` с преднастроенными расширениями, фильтрами и начальными именами;
   - Быстрые клавиатурные сокращения в `update()` (`Ctrl+S` -> `save_project`, `Ctrl+Shift+S` -> `save_project_as`, `Ctrl+Shift+O` -> `open_project_dialog`);
   - Динамический заголовок окна: отображение имени активного файла и индикатора несохраненных изменений `*`;
   - Автоматическое пополнение очереди `RecentFiles` и журналирование в Output Viewer (`SAVE OUTFILE=...` / `GET FILE=...`);
   - Подтверждение сохранения перед деструктивными действиями: сохранение проекта при нажатии `[ Save ]` в `ConfirmSaveDialog`.
5. **Архитектурная чистота («Разделяй, а не урезай»):**
   - Выделение модуля `crates/statcraft_app/src/status_message.rs` позволило сократить файл `app.rs` с 1453 до ~1300 строк, гарантируя запас до жесткого лимита 1500 строк.
6. **Контроль качества (Definition of Done):**
   - Прогон полного пайплайна качества `cargo run -p xtask -j 1 -- ci` — 100% OK;
   - 202 файла Rust проверены и укладываются в лимиты строк (`file-guard` — 100% OK);
   - Соблюдение правил слоев `layers` (L0-L6) и дизайна `design-guard` — 100% OK;
   - 125+ модульных и интеграционных тестов воркспейса завершились успешно;
   - 100% синхронная локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`);
   - Компилятор и линтер `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 предупреждений.

---

### Финал: Бесшовная интеграция меню «Файл», диалог сохранения и глобальные горячие клавиши (File Menu Integration & Shortcuts)
*Время: 29 октября 2026, 22:45 (~20 минут)*

Завершена комплексная бесшовная интеграция всех подсистем меню «Файл» (File Menu Integration) в приложении StatCraft (`crates/statcraft_app`):
1. **Перехват закрытия окна и защита от потери данных (Window Close Guard):**
   - Перехват события закрытия окна через `ctx.input(|i| i.viewport().close_requested())` при наличии несохраненных изменений (`has_unsaved_changes() == true`);
   - Мгновенная отмена несанкционированного закрытия через `ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose)`;
   - Запуск диалога подтверждения `ConfirmSaveDialog` с сохранением отложенного действия закрытия в `pending_session_action`;
   - Поддержка канонических действий: `[ Save ]` (сохранение в `.stc` / `.csv` и переход к действию), `[ Don't Save ]` (исполнение действия без сохранения) и `[ Cancel ]` (отмена и продолжение работы с проектом).
2. **Глобальные клавиатурные сокращения (Global Accelerators):**
   - Быстрый вызов канонических операций меню «Файл» из любого экрана приложения:
     * `Ctrl+Shift+N` — Новый проект (`NewProject`);
     * `Ctrl+N` — Новый датасет (`NewDataset`);
     * `Ctrl+O` — Открыть данные (`open_data_dialog` с поддержкой `.stc`, `.sav`, `.zsav`, `.csv`, `.xlsx`);
     * `Ctrl+Shift+O` — Открыть проект StatCraft (`open_project_dialog`);
     * `Ctrl+S` — Сохранить активный проект (`save_project`);
     * `Ctrl+Shift+S` — Сохранить проект как (`save_project_as`).
3. **Репозиторий и тесты инвариантов диалогов (DoD):**
   - Добавлен headless-тест `test_headless_render_confirm_save_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs` для проверки надежности рендера и отсутствия паник;
   - Настроено игнорирование локального каталога недавних файлов `.statcraft/` в корне и внутри `crates/statcraft_app/` в `.gitignore`;
   - Успешный прогон 130+ тестов воркспейса с `-j 1` и полное прохождение гейта `xtask ci` (`cargo fmt --check`, `clippy -D warnings`, `file-guard`, `design-guard`, `layers`, `parity`).

---

### Этап 41: Сохранение и восстановление полного аналитического пространства проекта (.stc)
*Время: 29 октября 2026, 23:40 (~35 минут)*

Реализовано монолитное сохранение полного рабочего пространства проекта StatCraft в нативном формате `.stc` (аналог `.psd` в Photoshop):
1. **Расширение контейнера проекта L0 (`statcraft_io::stc_codec`):**
   - Структура `StcContainer` расширена опциональными полями `output_viewer_json` и `syntax_text` с сохранением обратной совместимости (`Format Version 1`, `#[serde(default, skip_serializing_if = "Option::is_none")]`);
   - Добавлен конструктор `StcContainer::with_project_data` для чистой упаковки всех аналитических артефактов;
   - Защита от деградации старых файлов: старые `.stc` открываются штатно без ошибок.
2. **Сериализация правой панели L6 (`statcraft_app::views::output_view`):**
   - Модуль безопасной сериализации `static_str_serde` для десериализации статических ключей заголовков `&'static str` (`Box::leak`);
   - Реализована сериализация для всех 19 типов элементов вывода `OutputItemKind` (дескриптивные статистики, таблицы частот, t-критерии, ANOVA, таблицы сопряженности, корреляции, непараметрические критерии, линейная регрессия, надежность, факторный анализ, средние, логи, заметки, предупреждения и сводки данных);
   - Методы `OutputViewerState::to_json()` и `OutputViewerState::load_from_json()` с автоматическим вычислением `next_id = max(id) + 1` для предотвращения коллизий идентификаторов при последующих вычислениях.
3. **Мост L1 (`statcraft_core::stc_codec`):**
   - Функции `project_to_stc`, `save_project_stc_to_path` и `load_project_stc_from_path` для координации между датасетом, структурой метаданных и прикладными блоками проекта.
4. **Интеграция в жизненный цикл сессии L6 (`statcraft_app::app_session`):**
   - В `save_to_path`: при сохранении `.stc` сериализуются процедуры правой панели (`OutputViewerState`) и текст активного скрипта (`SyntaxEditorState`);
   - В `load_file_from_path`: при открытии проекта `.stc` восстанавливаются таблица данных, словарь переменных, активный фильтр, веса, группировка `split_file`, все процедуры и сводные таблицы правой панели, а также текст скрипта в редакторе синтаксиса;
   - Строгое разграничение меню: диалог `File -> Save Data As...` очищен от фильтра `.stc` и сохраняет исключительно открытый массив данных (`.csv` / `.tsv`), а `File -> Save Project` (`Ctrl+S`) и `Save Project As...` (`Ctrl+Shift+S`) сохраняют 100% монолитного аналитического пространства.
5. **Контроль качества (DoD):**
   - Добавлен сквозной интеграционный тест `test_save_and_load_full_project_with_output_and_syntax` в `session_lifecycle_tests.rs`;
   - Прогон полного набора тестов рабочего пространства `cargo test --workspace -j 1` — 100% успешно;
   - 100% соответствие `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`).

---

### Этап 42: Исправление кнопки сохранения на тулбаре, контрольная точка сессии (SavedCheckpoint) и индикация статуса
*Время: 30 октября 2026, 05:35 (~25 минут)*

Устранены дефекты жизненного цикла сохранения проекта и синхронизации интерфейса:
1. **Диспетчеризация действий тулбара L6 (`crates/statcraft_app/src/app.rs`):**
   - Добавлена обработка `ToolbarAction::SaveData`: клик по иконке дискеты на главной панели инструментов теперь корректно вызывает `self.save_project()`, открывая диалог сохранения или перезаписывая текущий `.stc` файл проекта;
   - Добавлена обработка `ToolbarAction::Redo` с вызовом `ds.redo()`, а флаг активности кнопки `can_redo` динамически связан с состоянием `ds.delta.redo_stack`.
2. **Контрольная точка сохраненного состояния `SavedCheckpoint` (`crates/statcraft_app/src/app_session.rs`):**
   - Разработана легковесная структура `SavedCheckpoint`, фиксирующая состояние документа (число колонок и строк датасета, размер стека отмены правок ячеек `delta_undo_len`, размер истории команд `command_history_len`, длина текста синтаксиса `syntax_len`, количество процедур в `OutputViewer`);
   - Поле `saved_checkpoint: Option<SavedCheckpoint>` интегрировано в `StatCraftApp`;
   - Метод `has_unsaved_changes(&self)` теперь проверяет расхождение текущего состояния с контрольной точкой `saved_checkpoint`: маркер несохраненных изменений (`*`) в верхней строке заголовка автоматически гаснет сразу после сохранения, а также при возврате к сохраненному состоянию через `Undo` (`Ctrl+Z`);
   - Автоматическая фиксация контрольной точки `update_saved_checkpoint()` внедрена в `save_to_path`, `load_file_from_path` (для `.stc`, `.sav`, `.xlsx`, `.csv`), `execute_new_dataset`, `execute_new_project` и `execute_close_project`.
3. **Цветовая дифференциация строки состояния (`crates/statcraft_app/src/status_message.rs`):**
   - Добавлен метод `StatusMessage::color(&self) -> egui::Color32`: успешные события сохранения и экспорта (`ProjectSaved`, `DataSaved`, `DataExported`, `OutputExported`) подсвечиваются акцентным изумрудным цветом `Tokens::MEASURE_NOMINAL`; сообщения об ошибках — контрастным `Color32::from_rgb(235, 80, 80)`; информационные сообщения — спокойным `Tokens::TEXT_MUTED`;
   - В статус-баре задействован `ui.colored_label(...)` для мгновенной периферической обратной связи.
4. **Контроль качества (DoD):**
   - Расширен интеграционный тест `test_has_unsaved_changes_detection` и добавлен `test_toolbar_save_action_execution` в `crates/statcraft_app/tests/session_lifecycle_tests.rs`;
   - Добавлены модульные тесты цветов `test_status_message_colors` в `status_message.rs`;
   - 100% успешный прогон всех тестов воркспейса: `cargo test --workspace -j 1`;
   - 100% прохождение всех CI гейтов: `cargo run -p xtask -j 1 -- ci` (`cargo fmt --check`, `clippy -D warnings`, `file-guard`, `design-guard`, `layers`, `parity`).

---

### Этап 43: Процедура сдвига значений (Lead / Lag) — `transform.shift_values`
*Время: 30 октября 2026, 06:05 (~30 минут)*

Реализована каноническая процедура временного сдвига значений (лаги и опережения) пакета IBM SPSS Statistics — `Преобразование -> Сдвиг значений (Lead / Lag)...` (команда `transform.shift_values` / синтаксис `SHIFT VALUES`):
1. **Алгоритмический фундамент L2 (`crates/statcraft_algo/src/shift.rs`):**
   - Тип направления сдвига `ShiftDirection` (`Lag` — сдвиг вниз со смещением в прошлое, `Lead` — сдвиг вверх со смещением в будущее);
   - Функция `compute_shift_raw(values, direction, step)` для колоночного массива `RawValue` с поддержкой произвольного шага $K \ge 1$;
   - Обработка граничных условий: при $K = 0$ или пустом массиве возвращается исходный срез; при $K \ge N$ весь массив заполняется `RawValue::Null`;
   - Для `Lag(step)` первые $K$ ячеек получают системный пропуск `RawValue::Null`, а последующие заполняются значениями `values[i - step]`;
   - Для `Lead(step)` значения смещаются вперед на $K$ позиций, а завершающие $K$ ячеек заполняются `RawValue::Null`;
   - 100% safe Rust без `unwrap`/`expect`/`panic`, модульные тесты граничных случаев, пропусков и строковых значений.
2. **Командная шина и синтаксис L5 (`crates/statcraft_syntax/src/commands/shift_values_cmd.rs`):**
   - Команда `ShiftValuesCommand` (`transform.shift_values`) с регистрацией в `CommandBus`;
   - Сигнатура параметров `specs()` для внешних AI-агентов (`source_var`, `target_var`, `direction`, `step`, `target_label`);
   - Генератор синтаксиса SPSS:
     ```text
     SHIFT VALUES /VARIABLE=sales /RESULT=sales_lag1 /LAG=1.
     VARIABLE LABELS sales_lag1 "Previous Sales".
     EXECUTE.
     ```
   - Защита от потери данных и история Undo/Redo: создание новой переменной наследует тип данных и шкалу измерения исходной колонки, при перезаписи сохраняются старые метаданные и значения (`ShiftUndoEntry::Created` / `ShiftUndoEntry::Overwritten`);
   - Интеграционные тесты сдвига и полного отката в `statcraft_syntax`.
3. **Локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`):**
   - Декларативные ключи диалога, групп радиокнопок, шага сдвига и статусных сообщений.
4. **Графический интерфейс L6 (`crates/statcraft_app/src/dialogs/shift_values_dialog.rs`):**
   - Модальное диалоговое окно по канону `DialogKit` (`DialogShell::standard` 560 × 430 px);
   - Панель выбора доступных переменных `render_variable_list_panel` и адаптивная кнопка переноса `TransferButton`;
   - Поля ввода имени и метки целевой переменной с автоподстановкой суффикса;
   - Переключатель направления сдвига `GroupBox` (`Lag` / `Lead`) и счетчик шага сдвига `DragValue`;
   - Нижний ряд кнопок действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Интеграция в десктопное приложение L6 (`app.rs`, `app_dialogs_data.rs`):**
   - В меню `Преобразование` активирован пункт `menu-transform-shift` (метка `[dev]` снята);
   - Выполнение команды из диалога обновляет датасет, логирует канонический синтаксис в `Output Viewer` и выводит подтверждение `StatusMessage::ShiftValuesExecuted` в строке состояния;
   - Headless-тест `test_headless_render_shift_values_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Контроль качества (DoD):**
   - 100% тестов воркспейса завершились успешно;
   - Прохождение всех линтеров и защитников архитектуры: `layers`, `file-guard`, `design-guard`, `parity`.

---

### Этап 44: Подсчет значений в строках — `transform.count_values`
*Время: 30 октября 2026, 06:15 (~35 минут)*

Реализована каноническая процедура агрегации вхождений значений внутри каждого наблюдения пакета IBM SPSS Statistics — `Преобразование -> Подсчет значений в строках...` (команда `transform.count_values` / синтаксис `COUNT`):
1. **Алгоритмический фундамент L2 (`crates/statcraft_algo/src/count_values.rs`):**
   - Перечисление критериев сопоставления `ValueCriterion` (`ExactNumber`, `ExactString`, `Range { low, high }`, `LowestThru(high)`, `ThruHighest(low)`, `SystemMissing`);
   - Метод `ValueCriterion::matches(&self, val: &RawValue) -> bool` с точным сопоставлением чисел с эпсилоном $10^{-9}$, поддержкой строковых литералов, диапазонов и пропусков `SYSMIS` / `NaN` / `Null`;
   - Функция `compute_count_across_columns(rows, columns_data, criteria)`: подсчитывает количество колонок внутри строки, удовлетворяющих хотя бы одному из заданных критериев;
   - Возвращает числовой вектор `RawValue::Number(count as f64)` без пропусков (по стандарту SPSS);
   - 100% safe Rust без `unwrap`/`expect`/`panic`, всесторонние модульные тесты.
2. **Командная шина и синтаксис L5 (`crates/statcraft_syntax/src/commands/count_values_cmd.rs`):**
   - Типизированная команда `CountValuesCommand` (`transform.count_values`) с регистрацией в `CommandBus`;
   - JSON-спецификация `specs()` для внешних AI-агентов (`target_var`, `target_label`, `variables`, `criteria`);
   - Генератор канонического синтаксиса SPSS:
     ```text
     COUNT high_scores = q1 q2 (5) (8 THRU 10).
     VARIABLE LABELS high_scores "High score count".
     EXECUTE.
     ```
   - Защита данных и полный откат Undo/Redo: при создании новой переменной создается колонка `DataType::Numeric { decimals: 0 }` со шкалой `Measure::Scale`, при перезаписи сохраняются старые метаданные и значения (`CountUndoEntry::Created` / `CountUndoEntry::Overwritten`);
   - Модульные тесты выполнения команды, синтаксиса и отката в `statcraft_syntax`.
3. **Локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`):**
   - Декларативные ключи для заголовка, полей целевой переменной, списков, радиокнопок диапазонов, кнопок `Add`/`Change`/`Remove` и статусного сообщения `output-count-executed`.
4. **Графический интерфейс L6 (`crates/statcraft_app/src/dialogs/`):**
   - Основной диалог `CountValuesDialog` (`count_values_dialog.rs`, 560 × 430 px, `DialogShell::standard`) с полями целевой переменной и метки, списком доступных переменных, векторной кнопкой переноса `TransferButton`, списком выбранных переменных и правой колонкой с кнопкой `Define Values...` и блоком предпросмотра критериев;
   - Модальный поддиалог `CountValuesDefineState` (`count_values_define_dialog.rs`, `DialogShell::standard`) для добавления, изменения и удаления точных значений, системных пропусков и диапазонов (`Range`, `LOWEST THRU`, `THRU HIGHEST`) с нижней панелью `SubdialogActionBar`;
   - Нижний ряд кнопок действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Интеграция в десктопное приложение L6 (`app.rs`, `app_dialogs_data.rs`):**
   - В меню `Преобразование` активирован пункт `menu-transform-count-values` (снята метка `InDev`);
   - Выполнение диалога обновляет данные, логирует синтаксис в `Output Viewer` и выводит подтверждение `StatusMessage::CountValuesExecuted` в строке состояния;
   - Headless-тест `test_headless_render_count_values_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Контроль качества (DoD):**
   - 100% тестов воркспейса завершились успешно;
   - Прохождение всех линтеров и защитников архитектуры: `layers`, `file-guard`, `design-guard`, `parity`.

---

### Этап 45: Установка зерна ДСЧ — `transform.random_seed`
*Время: 30 октября 2026, 06:16 (~30 минут)*

Реализована каноническая процедура фиксации начального значения генератора псевдослучайных чисел пакета IBM SPSS Statistics — `Преобразование -> Установить зерно ДСЧ...` (команда `transform.random_seed` / синтаксис `SET SEED`):
1. **Состояние RNG на слое L1 (`crates/statcraft_core/src/rng.rs`):**
   - Потокобезопасное атомарное хранение состояния глобального зерна генератора (`AtomicBool`, `AtomicU64`, `Ordering::SeqCst`);
   - Функции `set_global_random_seed(seed: u64)`, `clear_global_random_seed()`, `get_global_random_seed() -> Option<u64>`;
   - Вспомогательная функция `get_effective_random_seed() -> u64`: возвращает зафиксированное зерно либо генерирует значение по системному времени от 1 до 2 000 000 000;
   - Поддержка двух режимов: «Случайная инициализация» (сброс фиксации) и «Фиксированное зерно».
2. **Командная шина и синтаксис L5 (`crates/statcraft_syntax/src/commands/random_seed_cmd.rs`):**
   - Типизированная команда `RandomSeedCommand` (`transform.random_seed`) с регистрацией в `CommandBus`;
   - JSON-спецификация `specs()` (`seed` типа `integer` или `null`, диапазон от 1 до 2 000 000 000);
   - Генератор синтаксиса SPSS:
     ```text
     SET SEED = 2000000.
     ```
     или `SET SEED = RANDOM.` при случайной инициализации;
   - Полная поддержка отката `undo` с восстановлением предыдущего значения зерна через историю `RandomSeedUndoEntry`;
   - Модульные тесты валидации, синтаксиса и отката в `statcraft_syntax`.
3. **Локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`):**
   - Ключи `dialog-seed-title`, `dialog-seed-mode-random`, `dialog-seed-mode-fixed`, `dialog-seed-value-label`, `status-random-seed-fixed`, `status-random-seed-random`.
4. **Графический интерфейс L6 (`crates/statcraft_app/src/dialogs/random_seed_dialog.rs`):**
   - Модальное окно `RandomSeedDialog` по стандарту `DialogKit` (`DialogShell::properties`, `GroupBox`, радиокнопки режимов, текстовое поле ввода числа с автовалидацией);
   - Нижний ряд кнопок действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Интеграция в десктопное приложение L6 (`app.rs`, `app_dialogs_data.rs`):**
   - В меню `Преобразование` активирован пункт `menu-transform-random-seed` (метка `InDev` снята);
   - Выполнение диалога обновляет глобальное зерно, логирует синтаксис в `Output Viewer` и выводит подтверждение `StatusMessage::RandomSeedExecuted` в строке состояния;
   - Headless-тест `test_headless_render_random_seed_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Контроль качества (DoD):**
   - 100% тестов воркспейса завершились успешно;
   - Прохождение всех проверок архитектуры: `layers`, `file-guard`, `design-guard`, `parity`, `clippy -D warnings`.

---

### Этап 46: Создание фиктивных переменных (Create Dummy Variables: `SPSSINC CREATE DUMMIES`) — `transform.create_dummies`
*Время: 30 октября 2026, 06:45 (~30 минут)*

Реализована каноническая процедура генерации фиктивных (dummy) переменных для категориальных предикторов (One-Hot Encoding) — пункт меню `Преобразование -> Создать фиктивные (dummy) переменные...` (`Transform -> Create Dummy Variables...`, команда `transform.create_dummies` / синтаксис `SPSSINC CREATE DUMMIES`):
1. **Алгоритмический движок L2 (`crates/statcraft_algo/src/dummies.rs`):**
   - Модуль генерации фиктивных колонок `generate_dummies_for_column` со структурами `DummySpec` и `GeneratedDummyColumn`;
   - Сбор и сортировка уникальных категориальных значений (`Number` и `Text`) в порядке возрастания с игнорированием пропусков (`Null`, `NaN`, пустые строки);
   - Поддержка пропуска базовой (референтной) категории (`omit_first == true`) для получения $k - 1$ фиктивных переменных для линейных регрессионных моделей;
   - Сопоставление меток категорий по словарю `ValueLabels`: генерация понятных меток вида `gender = Female`, `jobcat = Clerical`;
   - Санитизация имен колонок (буквы, цифры, `_`), сохранение пропусков в строках как `RawValue::Null`;
   - 100% Safe Rust без паник (`unwrap`, `expect`) и модульные тесты для числовых, строковых и пустых данных.
2. **Командная шина и синтаксис L5 (`crates/statcraft_syntax/src/commands/dummy_cmd.rs`):**
   - Типизированная команда `CreateDummiesCommand` (`DummyCommand`, ID `transform.create_dummies`), зарегистрированная в `CommandBus`;
   - Спецификация JSON Schema `specs()` (`variables`, `root_prefix`, `omit_reference`);
   - Добавление сгенерированных колонок в датасет (`Numeric`, 0 десятичных знаков, `Measure::Nominal`) с автоматическим разрешением конфликтов имен;
   - Полная поддержка отката `Undo` (`Ctrl+Z`) через удаление добавленных переменных в обратном порядке индексов (`DummyUndoEntry`);
   - Генератор канонического синтаксиса расширения SPSS:
     ```text
     SPSSINC CREATE DUMMIES VARIABLES=gender jobcat
       /ROOTNAMES=d_gender d_jobcat
       /OMITFIRST=NO.
     EXECUTE.
     ```
3. **Локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`):**
   - Декларативные ключи `dialog-dummy-title`, `dialog-dummy-source-vars`, `dialog-dummy-root-prefix`, `dialog-dummy-omit-reference`, `dialog-dummy-error-empty-vars`, `status-dummies-executed`.
4. **Графический интерфейс на DialogKit L6 (`crates/statcraft_app/src/dialogs/dummy_dialog.rs`):**
   - Модальное окно `DummyDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Список доступных переменных `render_variable_list_panel` слева и векторная кнопка переноса `TransferButton` по центру;
   - Правая колонка: список выбранных категориальных переменных, блок `GroupBox` с полем ввода префикса (`TextEdit`) и чекбокс исключения референтной группы `omit_reference`;
   - Нижний ряд `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Интеграция в десктопное приложение L6 (`app.rs`, `app_dialogs_data.rs`):**
   - Активирован пункт меню `Transform -> Create Dummy Variables...` (снят статус `InDev`);
   - Обработка запуска в `app_dialogs_data.rs` с логированием синтаксиса в `Output Viewer` и статусным сообщением `StatusMessage::CreateDummiesExecuted`;
   - Регрессионный headless-тест `test_headless_render_dummy_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Контроль качества (DoD):**
   - 100% тестов воркспейса завершились успешно;
   - Прохождение всех линтеров и защитников архитектуры: `layers`, `file-guard`, `design-guard`, `parity`, `clippy -D warnings`.

---

### Этап 47: Автоматическое перекодирование (Automatic Recode: `AUTORECODE`) — `transform.auto_recode`
*Время: 30 октября 2026, 07:15 (~30 минут)*

Реализована каноническая процедура преобразования строковых или нерегулярных числовых категорий в последовательные целочисленные коды ($1, 2, 3, \dots, K$) с автоматическим заполнением меток значений исходными данными — пункт меню `Преобразование -> Перекодирование -> Автоматическое...` (`Transform -> Recode -> Automatic Recode...`, команда `transform.auto_recode` / синтаксис `AUTORECODE`):
1. **Алгоритмический движок L2 (`crates/statcraft_algo/src/auto_recode.rs`):**
   - Функция `compute_auto_recode` и перечисление порядка ранжирования `AutoRecodeOrder` (`LowestValue`, `HighestValue`);
   - Сбор уникальных непустых значений (`RawValue::Number` и `RawValue::Text`) с игнорированием пропусков (`Null`, `NaN`, пустые строки);
   - Сортировка уникальных категорий: по возрастанию (`LowestValue`) либо по убыванию (`HighestValue`);
   - Последовательное присвоение целочисленных кодов ($1.0, 2.0, \dots, K$);
   - Автоматическое формирование словаря меток значений `ValueLabels` (`"1"` $\to$ оригинальное значение, `"2"` $\to$ оригинальное значение...);
   - 100% Safe Rust без паник (`unwrap`, `expect`) с модульными тестами для строковых, числовых, смешанных и пустых выборок.
2. **Командная шина и синтаксис L5 (`crates/statcraft_syntax/src/commands/auto_recode_cmd.rs`):**
   - Типизированная команда `AutoRecodeCommand` (ID `transform.auto_recode`), зарегистрированная в `CommandBus`;
   - Спецификация JSON Schema `specs()` (массив пар `pairs` с `source_var` и `target_var`, опция `order`);
   - Создание новых числовых переменных (`Numeric`, `decimals: 0`, `Measure::Nominal`) со словарем меток `ValueLabels` либо перезапись существующих;
   - Полная поддержка отката `Undo` (`Ctrl+Z`) через историю `AutoRecodeUndoItem` (`Created` / `Overwritten`);
   - Генератор канонического синтаксиса IBM SPSS Statistics:
     ```text
     AUTORECODE VARIABLES=country /INTO country_code
       /PRINT.
     EXECUTE.
     ```
     (с поддержкой подкоманды `/DESCENDING` при порядке `HighestValue`).
3. **Локализация без хардкода строк (`locales/en.ftl`, `locales/ru.ftl`):**
   - Декларативные ключи `dialog-autorecode-title`, `dialog-autorecode-variables`, `dialog-autorecode-new-name`, `dialog-autorecode-change`, `dialog-autorecode-order-group`, `dialog-autorecode-order-lowest`, `dialog-autorecode-order-highest`, `output-autorecode-executed`.
4. **Графический интерфейс на DialogKit L6 (`crates/statcraft_app/src/dialogs/auto_recode_dialog.rs`):**
   - Модальное окно `AutoRecodeDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Список доступных переменных `render_variable_list_panel` слева и адаптивная векторная кнопка переноса `TransferButton` по центру;
   - Правая колонка: список пар `source -> target`, блок `GroupBox` с полем ввода нового имени и кнопкой `Change Name`, блок `GroupBox` с радиокнопками выбора порядка ранжирования (*Lowest value / Highest value*);
   - Нижний ряд `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Интеграция в десктопное приложение L6 (`app.rs`, `app_dialogs_data.rs`):**
   - В меню `Преобразование -> Перекодирование` активирован пункт `menu-transform-auto-recode` (снят статус `InDev`);
   - Обработка запуска в `app_dialogs_data.rs` с логированием синтаксиса в `Output Viewer` и статусным сообщением `StatusMessage::AutoRecodeExecuted`;
   - Регрессионный headless-тест `test_headless_render_auto_recode_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Контроль качества (DoD):**
   - 100% тестов воркспейса завершились успешно (`cargo test --workspace -j 1`);
   - Полное прохождение проверок архитектурных линтеров: `layers`, `file-guard`, `design-guard`, `parity`, `clippy -D warnings`.

---

### Микроплан №8: Создание временного ряда — `transform.create_time_series`
*Время: 30 октября 2026, 07:45 (~35 минут)*

Реализован классический инструмент генерации производных рядов динамики IBM SPSS Statistics — **Создание временного ряда...** (`Преобразование -> Создать временной ряд...`, идентификатор команды `transform.create_time_series`):
1. **Математическое ядро L2 (`statcraft_algo::time_series`):**
   - Реализована функция `compute_time_series_transform` с поддержкой четырех ключевых преобразований:
     - `DIFF`: последовательные разности порядка $K$ (степени дифференцирования $(1-L)^k$);
     - `SDIFF`: сезонные разности с периодом/лагом $S$ ($(1-L^s)$);
     - `SMA`: простое скользящее среднее ширины $W$ (trailing moving average с требованием полноты окна);
     - `CSUM`: кумулятивная нарастающая сумма с SPSS-совместимой остановкой при пропусках;
   - Строгая обработка пропусков `RawValue::Null`, нечисловых данных и NaN без единой паники на 100% Safe Rust;
   - Полноценные юнит-тесты на все 4 функции, граничные значения порядка и пропуски.
2. **Командная шина L5 (`statcraft_syntax::commands::time_series_cmd`):**
   - Движковая команда `CreateTimeSeriesCommand` (`transform.create_time_series`) с JSON Schema спецификацией параметров `specs()`;
   - Генерация аутентичного SPSS-синтаксиса `CREATE diff_sales = DIFF(sales, 1) / sma_sales = SMA(sales, 3) / csum_sales = CSUM(sales). EXECUTE.`;
   - Полная пакетная поддержка отката `Undo` (`SingleVarUndoEntry::Created` и `SingleVarUndoEntry::Overwritten`).
3. **Графический интерфейс L6 (`statcraft_app::dialogs::time_series_dialog`):**
   - Диалог на канонической дизайн-системе DialogKit (`DialogShell::standard`);
   - Левая панель выбора переменных со шкалами измерения, кнопка переноса `TransferButton`, список сформированных рядов `target = FN(source, order)`, редактирование имени и метки переменной, радиокнопки выбора функции, поле порядка/интервала и кнопка `Change`;
   - Нижняя панель действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Пункт меню `menu-transform-time-series` переведен из `InDev` в активный режим `Ready`;
   - Двуязычная локализация Fluent (`dialog-timeseries-*` в `locales/ru.ftl` и `locales/en.ftl`).
4. **Контроль качества (DoD):**
   - 100% Safe Rust, 0 паник;
   - `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - `cargo run -p xtask -j 1 -- file-guard` — все 232 файла в пределах лимитов;
   - `cargo run -p xtask -j 1 -- design-guard` — все диалоги соответствуют DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — 47 команд зарегистрировано;
   - `cargo test --workspace -j 1` — 100% успешный прогон всех тестов воркспейса;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 предупреждений.

---

### Микроплан №6: Ранжирование наблюдений — `transform.rank_cases`
*Время: 30 октября 2026, 07:50 (~35 минут)*

Реализована каноническая процедура вычисления рангов и нормальных оценок IBM SPSS Statistics — **Ранжирование наблюдений...** (`Преобразование -> Ранжировать наблюдения...`, идентификатор команды `transform.rank_cases`):
1. **Математическое ядро L2 (`statcraft_algo::rank_cases` и `rank_cases_tests`):**
   - Функция `compute_ranks` с поддержкой всех четырех методов разрешения связей `TiesHandling`:
     - `Mean`: средний ранг группы связанных наблюдений (стандарт SPSS по умолчанию);
     - `Low`: наименьший ранг связки;
     - `High`: наибольший ранг связки;
     - `Seq`: последовательный ранг без образования связей;
   - Направление ранжирования `RankDirection` (`Ascending` — наименьшему значению ранг 1, `Descending` — наибольшему значению ранг 1);
   - Поддержка спектра типов рангов `RankType`:
     - `SimpleRank`: обычные целочисленные и полуцелые ранги;
     - `FractionalRank`: дробные ранги $r / N$;
     - `FractionalPercent` / `Percentiles`: процентильные ранги $(r / N) \times 100$;
     - `Ntiles(k)`: квантили/N-тили с вычислением $\lceil (r \cdot k) / N \rceil$ (по умолчанию 4 для квартилей);
     - Нормальные оценки: Blom $((r - 3/8) / (w + 1/4))$, Tukey $((r - 1/3) / (w + 1/3))$, Rankit $((r - 1/2) / w)$, Van der Waerden $(r / (w + 1))$;
   - Чистая реализация обратной функции нормального распределения `inv_normal_cdf` (аппроксимация Питера Аклема с точностью $< 10^{-9}$);
   - 100% Safe Rust без единого падения, пропуски `RawValue::Null` сохраняются на исходных позициях.
2. **Командная шина L5 (`statcraft_syntax::commands::rank_cases_cmd`):**
   - Движковая команда `RankCasesCommand` (`transform.rank_cases`) с JSON Schema `specs()`;
   - Пакетное создание/перезапись новых числовых переменных со шкалой `Measure::Ordinal` и префиксом (по умолчанию `R_`);
   - Генерация аутентичного синтаксиса `RANK VARIABLES=... /TIES=... /RANK ... \n EXECUTE.`;
   - Полный откат Undo (`Ctrl+Z`) с восстановлением исходных колонок и метаданных.
3. **Графический интерфейс L6 (`statcraft_app::dialogs::rank_cases_dialog`):**
   - Диалог на канонической дизайн-системе DialogKit (`DialogShell::standard`);
   - Левая панель `Variables:`, центральная кнопка `TransferButton`, правый список целевых переменных;
   - Блоки `GroupBox` выбора направления ранжирования (*Smallest value / Largest value*) и метода разрешения связей (*Mean / Low / High*);
   - Правая колонка `SubdialogSidebar` с кнопкой `Rank Types...` и поддиалогом `DialogShell::subdialog` для детальной настройки типа ранжирования и N-тилей;
   - Нижняя панель действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Пункт меню `menu-transform-rank-cases` переведен из `InDev` в активный режим `Ready`;
   - Локализация Fluent (`dialog-rank-*` и `output-rank-executed` в `ru.ftl` и `en.ftl`).
4. **Контроль качества (DoD):**
   - 100% Safe Rust, 0 паник;
   - `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - `cargo run -p xtask -j 1 -- file-guard` — все 232 файла в пределах лимитов;
   - `cargo run -p xtask -j 1 -- design-guard` — все диалоги соответствуют DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — паритет зафиксирован;
   - `cargo test --workspace -j 1` — 100% успешный прогон всех тестов воркспейса.

---

### Микроплан №9: Визуальная группировка — `transform.visual_binning`
*Время: 30 октября 2026, 08:45 (~35 минут)*

Реализован один из важнейших инструментов подготовки данных IBM SPSS Statistics — **Визуальная группировка...** (`Преобразование -> Визуальная группировка...`, идентификатор команды `transform.visual_binning`):
1. **Математическое ядро L2 (`statcraft_algo::binning`):**
   - Алгоритм дискретизации числовых рядов `compute_visual_binning` и `compute_visual_binning_with_labels` на базе списка точек среза `Cutpoint` (верхняя граница `upper_bound` и признак включения `inclusive`);
   - Автоматический расчет словаря меток `ValueLabels` по стандарту SPSS (`"<= 25000"`, `"25000 - 50000"`, `"> 50000"`);
   - Генераторы точек среза по ключевым методикам:
     - `generate_equal_width_cutpoints`: нарезка на интервалы равной ширины $W = (max - min) / N$;
     - `generate_equal_percentile_cutpoints`: квантильная нарезка (равное число наблюдений в интервале) на базе метода HAVERAGE;
     - `generate_stddev_cutpoints`: нарезка по интервалам стандартного отклонения $\pm 1\sigma, \pm 2\sigma$ вокруг выборочного среднего $\mu$;
   - 100% Safe Rust без единого `unwrap` в рабочем коде, безопасное сохранение пропусков `RawValue::Null` и юнит-тесты на все методы.
2. **Командная шина L5 (`statcraft_syntax::commands::binning_cmd`):**
   - Движковая команда `VisualBinningCommand` (`transform.visual_binning`) с JSON Schema `specs()`;
   - Создание/перезапись переменной со шкалой `Measure::Ordinal`, типом `DataType::Numeric { decimals: 0 }` и прикреплением словаря меток `ValueLabels`;
   - Генерация канонического синтаксиса IBM SPSS Statistics:
     ```text
     RECODE salary (MISSING=COPY) (LO THRU 25000 = 1) (LO THRU 50000 = 2) (ELSE = 3)
       INTO salary_binned.
     VARIABLE LABELS salary_binned 'Salary (Binned)'.
     VALUE LABELS salary_binned 1 '<= 25000' 2 '25000 - 50000' 3 '> 50000'.
     EXECUTE.
     ```
   - Полная поддержка отката `Undo` (`Ctrl+Z`).
3. **Графический интерфейс L6 (`statcraft_app::dialogs::binning_dialog` и `binning_cutpoints_dialog`):**
   - Диалог `VisualBinningDialog` по стандарту `DialogKit` (`DialogShell::standard`);
   - Левая панель выбора сканируемой переменной `render_variable_list_panel`;
   - Правая часть: имя и метка сгруппированной переменной, радиокнопки включения/исключения верхних границ (*Included (<=) / Excluded (<)*), кнопка вызова поддиалога `Make Cutpoints...`, ручной ввод точек среза, сетка срезов и меток `Cutpoint Grid` с прямым редактированием;
   - Поддиалог `binning_cutpoints_dialog` (`DialogShell::subdialog`, `SubdialogActionBar`) с выбором метода нарезки и числа срезов;
   - Нижняя панель действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Активирован пункт меню `menu-transform-visual-binning` (снят статус `InDev`), интеграция в `app_dialogs_data.rs`, статусное сообщение и логирование синтаксиса в `Output Viewer`.
4. **Контроль качества (DoD):**
   - 100% Safe Rust, 0 паник;
   - `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - `cargo run -p xtask -j 1 -- file-guard` — все 232 файла в пределах лимитов;
   - `cargo run -p xtask -j 1 -- design-guard` — все диалоги соответствуют DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — команда зарегистрирована;
   - `cargo test --workspace -j 1` — 100% успешный прогон всех тестов воркспейса;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 предупреждений.

---

### Микроплан №7: Замена пропущенных значений (Replace Missing Values: `RMV`) — `transform.replace_missing`
*Время: 30 октября 2026, 09:15 (~30 минут)*

Реализована каноническая статистическая процедура импутации пропущенных данных IBM SPSS Statistics — **Заменить пропущенные значения...** (`Преобразование -> Заменить пропущенные значения...`, идентификатор команды `transform.replace_missing`):
1. **Математическое ядро L2 (`statcraft_algo::replace_missing`):**
   - Функция `impute_missing_values` с поддержкой всех пяти классических методов импутации SPSS `ImputationMethod`:
     - `SeriesMean` (SMEAN): заполнение средним значением всех валидных точек анализируемой серии;
     - `MeanOfNearby(span)` (MEAN): среднее арифметическое валидных соседних точек в окне $[i - \text{span} .. i + \text{span}]$;
     - `MedianOfNearby(span)` (MEDIAN): медиана валидных соседних точек в окне $[i - \text{span} .. i + \text{span}]$;
     - `LinearInterpolation` (LINEAR): линейная интерполяция между предыдущей валидной $(i_0, y_0)$ и последующей валидной $(i_1, y_1)$ точками с ближайшим переносом для краевых пропусков на границах серии;
     - `LinearTrendAtPoint` (TREND): линейный регрессионный тренд $y = a + b \cdot x$ по номеру наблюдения $x_i = i + 1$;
   - Надежная защита Never Crash: безопасная обработка краевых пропусков, серий из одних пропусков, нулевых и одиночных выборок на 100% safe Rust.
2. **Командная шина L5 (`statcraft_syntax::commands::replace_missing_cmd`):**
   - Движковая команда `ReplaceMissingCommand` (`transform.replace_missing`) с JSON Schema `specs()` для AI-агентов;
   - Добавление новых числовых колонок (`DataType::Numeric { decimals: 2 }`, `Measure::Scale`) либо замещение существующих;
   - Генерация аутентичного синтаксиса SPSS:
     ```sps
     RMV new_salary = SMEAN(salary)
       /new_sales = MEDIAN(sales, 2)
       /new_temp = LINEAR(temp).
     EXECUTE.
     ```
   - Полный откат `Undo` (`Ctrl+Z`) с восстановлением исходных колонок, метаданных и значений.
3. **Графический интерфейс L6 (`statcraft_app::dialogs::replace_missing_dialog`):**
   - Модальное окно по стандарту DialogKit (`DialogShell::standard`, $560 \times 430$ px);
   - Левая панель выбора исходных переменных, центральная кнопка `TransferButton`, правый список правил `target -> METHOD(source)`;
   - Поле редактирования имени новой переменной с кнопкой `Change`, блок `GroupBox` выбора метода (радиокнопки) и числовой счетчик `span` для близлежащих точек;
   - Нижняя строка действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Активирован пункт меню `Transform -> Replace Missing Values...` (снят статус `[dev]`);
   - Двуязычная локализация Fluent (`dialog-missing-*`, `output-missing-executed`, `status-missing-executed` в `ru.ftl` и `en.ftl`).
4. **Контроль качества (Definition of Done):**
   - 100% Safe Rust, 0 паник, 0 `unwrap` в прикладном коде;
   - `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - `cargo run -p xtask -j 1 -- file-guard` — все 232 файла строго в пределах лимитов строк;
   - `cargo run -p xtask -j 1 -- design-guard` — все диалоги строго соответствуют DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — команда зарегистрирована в реестре паритета;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 предупреждений;
   - `cargo test --workspace -j 1` — 100% успешный прогон всех тестов воркспейса.







### Микроплан №10: Мастер даты и времени — `transform.date_time_wizard`
*Время: 30 октября 2026, 10:00 (~30 минут)*

Реализован классический инструмент работы с датами и временем IBM SPSS Statistics — **Мастер даты и времени...** (`Преобразование -> Мастер даты и времени...`, идентификатор команды `transform.date_time_wizard`):

1. **Математическое ядро L2 (`statcraft_algo::date_time`):**
   - Модуль `crates/statcraft_algo/src/date_time.rs` (393 строки, строго в лимите $\le 500$ строк);
   - Безопасный парсинг дат и временных меток без паник: поддержка стандартов ISO 8601, RFC 3339, `YYYY-MM-DD`, `DD.MM.YYYY`, `MM/DD/YYYY`, `YYYY/MM/DD`, форматов со временем `HH:MM:SS`, а также числовых unix timestamp и эпох;
   - Перечисление операций `DateTimeOperation`:
     - `ExtractPart(DatePart)`: извлечение года (`Year`), месяца (`Month`), дня (`Day`), часа (`Hour`), минут (`Minute`), секунд (`Second`), дня недели (`DayOfWeek`, 1 = воскресенье по стандарту SPSS) и квартала (`Quarter`);
     - `Difference { unit: DateUnit }`: расчет точного календарного интервала между двумя датами в годах, месяцах, неделях, днях, часах, минутах или секундах;
     - `AddDuration { amount, unit }`: прибавление или вычитание временной длительности с автоматической коррекцией високосных годов (например, 31 января + 1 месяц = 29 февраля в високосный год);
     - `ConstructDate`: стандартизация строковых или числовых данных в канонический формат даты;
   - Корректная обработка отсутствующих данных (`RawValue::Null`) и невалидных значений с сохранением пропусков без аварийных завершений.

2. **Командная шина L5 (`statcraft_syntax::commands::date_time_cmd`):**
   - Движковая команда `DateTimeWizardCommand` (`transform.date_time_wizard`) с валидируемой JSON Schema `specs()` для MCP и AI-агентов;
   - Создание новых колонок со шкалой `Measure::Scale` и типами данных `DataType::Numeric` (для извлечения частей и разностей) или `DataType::Date` (для вычисления дат);
   - Генерация канонического синтаксиса SPSS:
     ```sps
     COMPUTE birth_year = XDATE.YEAR(birth_date).
     COMPUTE age_years = DATEDIFF(date2, date1, "years").
     COMPUTE due_date = DATESUM(start_date, 30, "days").
     VARIABLE LABELS target "Result Label".
     EXECUTE.
     ```
   - Полная поддержка отката `Undo` (`Ctrl+Z`) с восстановлением исходных колонок и словаря метаданных.

3. **Графический интерфейс L6 (`statcraft_app::dialogs::date_time_dialog`):**
   - Модальное диалоговое окно по канону `DialogKit` (`DialogShell::standard`, $560 \times 430$ px);
   - Верхний блок `GroupBox` с выбором задачи мастера (Извлечь часть даты / Рассчитать разность двух дат / Прибавить или вычесть длительность / Преобразовать в стандартную дату);
   - Левая панель выбора доступных переменных `render_variable_list_panel`, адаптивная кнопка переноса `TransferButton`, слоты первой и второй даты;
   - Выпадающие списки `ComboBox::from_id_salt` для извлекаемых частей и единиц времени, счетчик `DragValue` для длительности, поля ввода имени и метки целевой переменной;
   - Нижняя панель действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Снятие статуса `[dev]` и активация пункта меню `Преобразование -> Мастер даты и времени...`;
   - Синхронная двуязычная локализация Fluent (`dialog-datetime-*`, `status-datetime-executed` в `locales/ru.ftl` и `locales/en.ftl`).

4. **Контроль качества (Definition of Done):**
   - 100% Safe Rust, отсутствие паник и `unwrap()` в нетестовом коде;
   - `cargo run -p xtask -j 1 -- layers` — 0 нарушений архитектурных слоев;
   - `cargo run -p xtask -j 1 -- file-guard` — все 237 файлов укладываются в лимиты строк;
   - `cargo run -p xtask -j 1 -- design-guard` — 100% соответствие DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — регистрация команды в реестре паритета функций;
   - `cargo test --workspace -j 1` — 100% успешное прохождение всех тестов воркспейса (включая линтер локализации `i18n_lint_test` и тесты инвариантов диалогов).

---

### Микроплан №11: Оптимальная группировка — `transform.optimal_binning`
*Время: 30 октября 2026, 10:35 (~35 минут)*

Реализована процедура управляемой дискретизации непрерывных признаков IBM SPSS Statistics — **Оптимальная группировка...** (`Преобразование -> Оптимальная группировка...`, идентификатор команды `transform.optimal_binning`):

1. **Алгоритмическое ядро L2 (`statcraft_algo::optimal_binning`):**
   - Новый модуль `crates/statcraft_algo/src/optimal_binning.rs` (289 строк, строго $\le 500$ строк);
   - Алгоритм MDLP (Minimum Description Length Principle Discretization, Fayyad & Irani 1993):
     * Сортировка пар $(x_i, y_i)$ по непрерывному признаку $x$;
     * Жадный поиск оптимального среза $T$, максимизирующего информационный выигрыш $\Delta Ent = Ent(S) - \frac{|S_1|}{|S|}Ent(S_1) - \frac{|S_2|}{|S|}Ent(S_2)$;
     * Проверка критерия остановки по сложности описания модели (MDL Principle Stopping Rule): разбиение принимается только если выигрыш превышает порог сложности описания $\frac{\log_2(N - 1) + \Delta}{N}$;
     * Глобальная очередь с приоритетом (Priority Queue) для выбора наиболее информативных срезов во всем датасете и строгого соблюдения ограничения числа бинов `max_bins` (по умолчанию 5);
     * Безопасная обработка константных признаков, одноклассовых целей и пропущенных значений (`Null`);
     * Автоматическая генерация текстовых меток интервалов (`<= bound`, `prev - curr`, `> bound`) и структуры `ValueLabels`.

2. **Командная шина движка L5 (`statcraft_syntax::commands::optimal_binning_cmd`):**
   - Типизированная команда `OptimalBinningCommand` (`transform.optimal_binning`) с JSON Schema `specs()` для MCP и AI-агентов;
   - Создание новых числовых колонок со шкалой измерения `Measure::Ordinal` и 0 знаков после запятой, прикрепление сгенерированных меток категорий `ValueLabels`;
   - Генерация канонического синтаксиса IBM SPSS Statistics:
     ```sps
     OPTIMAL BINNING
       /VARIABLES = income age BY default_status
       /CRITERIA METHOD = MDLP.
     EXECUTE.
     ```
   - Полная поддержка отката `Undo` (`Ctrl+Z`) со снятием созданных колонок в обратном порядке.

3. **Графический интерфейс L6 (`statcraft_app::dialogs::optimal_binning_dialog`):**
   - Диалоговое окно на базе `DialogKit` (`DialogShell::standard`, $560 \times 430$ px);
   - Левая панель выбора доступных переменных `render_variable_list_panel`;
   - Векторные адаптивные кнопки переноса `TransferButton` для списка группируемых переменных и целевой категориальной переменной (*Guide Variable*);
   - Правая панель: список переменных для группировки, слот целевой переменной с кнопкой сброса, блок параметров `GroupBox` со счетчиком `max_bins` (2..20) и префиксом новых колонок (`OB_`);
   - Нижняя строка действий `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Снятие статуса `[dev]` и активация пункта меню `Преобразование -> Оптимальная группировка...`;
   - Обработка в `app_dialogs_transform.rs`, статус `StatusMessage::OptimalBinningExecuted` и протоколирование в `Output Viewer`.

4. **Архитектурная декомпозиция для защиты лимитов строк (File Guard):**
   - Вынос системы меню `StatCraftApp` в изолированный подпакет `crates/statcraft_app/src/app_menus/` (`mod.rs`, `file_edit_view.rs`, `data_transform.rs`, `analyze_etc.rs`), что снизило размер `app.rs` с 1557 до 585 строк (при лимите 1500);
   - Декомпозиция `date_time_dialog` в модуль `date_time_dialog/` (`mod.rs`, `view.rs`), уложившись в лимит $\le 500$ строк на файл.

5. **Контроль качества (Definition of Done):**
   - 100% Safe Rust, 0 паник и unwrap в прикладном коде;
   - `cargo run -p xtask -j 1 -- layers` — 0 нарушений;
   - `cargo run -p xtask -j 1 -- file-guard` — все 245 файлов строго соблюдают лимиты строк;
   - `cargo run -p xtask -j 1 -- design-guard` — 100% соответствие DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — 51 документированная команда;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 предупреждений;
   - `cargo test --workspace -j 1` — 100% успешный прогон всех тестов воркспейса.

---

### Версия 0.2.0: Мультиплатформенная сборка бинарников под все ОС (Windows, Linux, macOS)
*Время: 30 октября 2026, 14:15 (~15 минут)*

Реализован релиз версии **0.2.0** и настроен автоматизированный CI/CD пайплайн сборки дистрибутивов под все ключевые настольные операционные системы:

1. **Повышение версии экосистемы до 0.2.0:**
   - Обновлен корень воркспейса `Cargo.toml`: `version = "0.2.0"`;
   - Крейты `statcraft_reporting` и `xtask` переведены на сквозное наследование `version.workspace = true`;
   - Синхронизированы метаданные пакетов во всем воркспейсе.

2. **GitHub Actions Workflow кроссплатформенной сборки (`.github/workflows/release.yml`):**
   - Настроена матричная сборка портативных архивов и нативных установщиков для всех целевых платформ:
     * **Windows x64** (`windows-latest`, `x86_64-pc-windows-msvc`): портативный архив `statcraft-windows-x64.zip` и полноценный инсталлятор Inno Setup `statcraft-windows-x64-installer.exe` (`packaging/windows/installer.iss`);
     * **Linux x64** (`ubuntu-22.04`, `x86_64-unknown-linux-gnu`): портативный архив `statcraft-linux-x64.tar.gz` и пакет для Debian/Ubuntu `statcraft-linux-x64.deb` с интеграцией десктоп-меню (`packaging/linux/statcraft.desktop`);
     * **macOS Apple Silicon** (`macos-latest`, `aarch64-apple-darwin`): архив `statcraft-macos-arm64.tar.gz` и монтируемый образ диска `statcraft-macos-arm64.dmg` с бандлом `StatCraft.app` (`packaging/macos/Info.plist`);
     * **macOS Intel** (`macos-13`, `x86_64-apple-darwin`): архив `statcraft-macos-x64.tar.gz` и образ диска `statcraft-macos-x64.dmg`;
   - Все дистрибутивы автоматически комплектуются каталогами внешних локализаций Fluent (`locales/`), документацией (`README.md`, `README.ru.md`) и файлом лицензии `LICENSE-MIT`;
   - В крейт `statcraft_i18n` добавлено системное обнаружение локализаций для Linux (`/usr/share/statcraft/locales`) и бандлов macOS (`Contents/Resources/locales`);
   - Автоматическая публикация GitHub Release через `softprops/action-gh-release@v2` при пуше тега `v*` или ручном запуске через `workflow_dispatch` с прикреплением всех 8 дистрибутивов без race conditions.

3. **Контроль качества (Definition of Done):**
   - Строгое соблюдение Resource Guard (`-j 1` во всех шагах компиляции);
   - 100% успешный локальный прогон CI-шлюза (`cargo run -p xtask -j 1 -- ci`).

---

### Регламент: Создание генерального руководства контрибьютора (`CONTRIBUTING.md`)
*Время: 30 октября 2026, 16:35 (~15 минут)*

Создано исчерпывающее руководство контрибьютора кодовой базы StatCraft — **`CONTRIBUTING.md`**, кодифицирующее все нормативные требования стандарта экосистемы ArtCraft / storytold:

1. **Канон и принципы экосистемы:** 100% Safe Rust (`#![forbid(unsafe_code)]`), запрет ручного JS/TS, двойная лицензия MIT / Apache-2.0, Clean-room воспроизведение без заимствования проприетарного кода IBM SPSS Statistics.
2. **Resource Guard:** Железный запрет компиляции без ограничения потоков (`-j 1`).
3. **Never Crash:** Запрет паник (`unwrap`, `expect`, `panic!`, `todo!`), обязательная типизация ошибок через `Result<T, E>` и валидация границ.
4. **Слои L0..L6 и CommandBus:** Жесткое разделение слоев (`statcraft_io` $\to$ `core` $\to$ `algo` $\to$ `reporting` $\to$ `syntax` $\to$ `app`), типизированные команды движка с историей Undo/Redo и генерацией синтаксиса `.sps`.
5. **Дизайн-система DialogKit и Vector-First UI:** Строгий канон верстки модальных окон, запрет эмодзи и неподдерживаемых глифов Unicode, отрисовка векторными примитивами.
6. **Zero-Hardcode i18n:** Полный запрет текстовых литералов в коде GUI, использование семантических ключей `t!("...")` и каталогов Fluent `locales/*.ftl`.
7. **File Size Guard:** Лимиты строк (500 / 1000 / 1500) и анти-гудхартовский принцип «Разделяй, а не урезай».
8. **Тестирование и Золотой Оракул:** Сверка математических процедур со SciPy с погрешностью $\le 10^{-6}$, тесты интерфейсных инвариантов.
9. **Автоматический аудит и Definition of Done:** Единая команда `cargo run -p xtask -j 1 -- ci`, чек-лист сдачи задач, пошаговый рецепт добавления новой процедуры (Cookbook).
10. **Интеграция:** Ссылки на руководство добавлены в `README.md` и `README.ru.md`.

---

### Исправление дефекта: Изоляция локали в тестах инвариантов Output Viewer
*Время: 30 октября 2026, 17:25 (~10 минут)*

Устранен недетерминированный сбой в тесте `test_reliability_output_and_tsv_integrity` в Linux CI Quality Gate:
1. **Первопричина:** По умолчанию каталог локализации `I18nCatalog` инициализируется русской локалью (`ru`). В тесте `test_reliability_output_and_tsv_integrity` отсутствовал вызов `statcraft_i18n::set_language("en")`. При параллельном или произвольном порядке выполнения тестов на разных платформах вызовы `reliability_to_tsv` возвращали локализованные русские заголовки (`Сводка обработки наблюдений`), из-за чего проверка английской строки `Case Processing Summary` падала на Linux-раннере.
2. **Исправление:**
   - В тесты `test_reliability_output_and_tsv_integrity`, `test_factor_output_and_tsv_integrity` и `test_means_output_and_tsv_integrity` добавлена явная инициализация английской локали `statcraft_i18n::set_language("en")`;
   - Проверки заголовков TSV дополнены поддержкой обеих канонических локалей (EN/RU) для исключения любых race conditions;
   - Полный прогон `cargo run -p xtask -j 1 -- ci` завершился со 100% успехом.


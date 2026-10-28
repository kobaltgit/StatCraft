# ТЗ для ИИ-агента: Этап 5 — Анализ связей и непараметрика (Phase 5)

> **Стандарт экосистемы:** ArtCraft (`storytold`) | Двойная лицензия MIT OR Apache-2.0  
> **Генеральный регламент:** [AGENTS.md](file:///d:/Projects/active/StatCraft/AGENTS.md)  
> **Ограничение ресурсов:** ВСЕ команды сборки и тестов запускаются строго с флагом `-j 1`.  
> **Принцип надежности:** 100% Safe Rust, Never Crash (0 `unwrap()`, 0 `expect()`, 0 `panic!()`, 0 `unsafe`).  
> **Локализация:** 0% строковых литералов на русском/английском в коде UI (строго макрос `t!()` и файлы `locales/ru.ftl`, `locales/en.ftl`).  
> **Интерфейс:** Vector-First UI (0 эмодзи, 0 квадратов «тофу»).

---

## 1. Цели задачи
1. Реализовать процедуры анализа взаимосвязей в L2 `statcraft_algo`:
   - **Таблицы сопряжённости** (*Crosstabs*):
     * Двумерная матрица наблюдаемых и ожидаемых частот;
     * Расчёт процентов по строкам (% within Row), колонкам (% within Column) и общего процента (% of Total);
     * Критерий независимости Хи-квадрат Пирсона ($\chi^2$), степени свободы $df$, точное асимптотическое $p$-value;
     * Поправка Йейтса на непрерывность для таблиц $2 \times 2$;
     * Меры связи: коэффициент сопряженности Пирсона, $V$ Крамера, $\phi$.
   - **Парные корреляции** (*Bivariate Correlations*):
     * Коэффициент линейной корреляции Пирсона ($r$);
     * Ранговый коэффициент корреляции Спирмена ($\rho$);
     * Коэффициент тау Кендалла ($\tau$);
     * Двустороннее и одностороннее $p$-значение, автоматическая разметка статистической значимости ($* p < 0.05$, $** p < 0.01$).
   - **Непараметрический критерий Манна-Уитни $U$** (*Mann-Whitney U Test*):
     * Ранжирование объединенной выборки, сумма рангов $W$, $U$-статистика, $Z$-аппроксимация, асимптотическое $p$-значение.
2. Зарегистрировать команды в L5 `statcraft_syntax`:
   - `stat.crosstabs` (синтаксис `CROSSTABS /TABLES=row_var BY col_var /STATISTICS=CHISQ PHI /CELLS=COUNT ROW COLUMN TOTAL.`).
   - `stat.correlations` (синтаксис `CORRELATIONS /VARIABLES=var1 var2 var3 /PRINT=TWOTAIL NOSIG.`).
   - `stat.npar.mann_whitney`.
3. Создать диалоги в `statcraft_app/src/dialogs/`:
   - `crosstabs_dialog.rs`: списки строк (Row), колонок (Column), чекбоксы ячеек (Counts, Row %, Col %) и статистик (Chi-square, Phi/Cramer's V);
   - `correlations_dialog.rs`: список переменных, чекбоксы методов (Пирсон, Спирмен, Кендалл), радиокнопки хвостов (двусторонний/односторонний).
4. Вывести аутентичные сводные таблицы в `Output Viewer`:
   - Итоги обработки наблюдений (*Case Processing Summary*);
   - Таблица сопряженности со счетчиками и процентами;
   - Таблица критериев Хи-квадрат (*Chi-Square Tests*);
   - Симметричная матрица корреляций со звёздочками значимости.

---

## 2. Пошаговый план изменений

### 2.1. L2 `statcraft_algo`
- Создать `crates/statcraft_algo/src/crosstabs.rs`:
  * `compute_crosstabs(row_data, col_data, options) -> CrosstabsResult`;
  * Расчёт $\chi^2 = \sum \frac{(O - E)^2}{E}$, поправка Йейтса, $V$ Крамера;
  * Golden Oracle unit-тесты со сверкой со `scipy.stats.chi2_contingency` с точностью $\le 10^{-6}$.
- Создать `crates/statcraft_algo/src/correlations.rs`:
  * `compute_bivariate_correlations(matrix, method) -> CorrelationMatrixResult`;
  * Golden Oracle unit-тесты со сверкой со `scipy.stats.pearsonr` и `scipy.stats.spearmanr`.
- Создать `crates/statcraft_algo/src/nonparametric.rs`:
  * `compute_mann_whitney_u(group1, group2) -> MannWhitneyResult`;
  * Golden Oracle со сверкой со `scipy.stats.mannwhitneyu`.

### 2.2. L5 `statcraft_syntax`
- Создать `crates/statcraft_syntax/src/commands/crosstabs_cmd.rs` и `correlations_cmd.rs`.
- Зарегистрировать в `CommandBus`.

### 2.3. L6 `statcraft_app`
- Создать `crates/statcraft_app/src/dialogs/crosstabs_dialog.rs` и `correlations_dialog.rs`.
- В `crates/statcraft_app/src/views/output_view.rs`:
  * Добавить рендеринг таблицы сопряжённости и корреляционной матрицы.
- В `crates/statcraft_app/src/app.rs`:
  * Добавить пункт меню `Analyze > Descriptive Statistics > Crosstabs...` и `Analyze > Correlate > Bivariate...`.

### 2.4. Локализация (`locales/ru.ftl` и `locales/en.ftl`)
- Все ключи меню, диалогов и таблиц на русском и английском языках.

---

## 3. Критерии приёмки и верификация (Definition of Done)
1. Строго `-j 1` на всех командах.
2. `cargo test --workspace -j 1` — 100% тестов завершились с кодом 0.
3. `cargo test --package statcraft_i18n --test i18n_lint_test -j 1` — 0% захардкоженных строк в UI.
4. `cargo fmt --check` и `cargo clippy --workspace -j 1 -- -D warnings` — чисто.
5. `cargo run --package xtask -j 1 -- layers` — без нарушений архитектуры.
6. Обновить `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md` с фиксацией выполненной работы.

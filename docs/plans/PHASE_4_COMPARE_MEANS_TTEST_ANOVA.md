# ТЗ для ИИ-агента: Этап 4 — Сравнение средних и проверка гипотез (Phase 4)

> **Стандарт экосистемы:** ArtCraft (`storytold`) | Двойная лицензия MIT OR Apache-2.0  
> **Генеральный регламент:** [AGENTS.md](file:///d:/Projects/active/StatCraft/AGENTS.md)  
> **Ограничение ресурсов:** ВСЕ команды сборки и тестов запускаются строго с флагом `-j 1`.  
> **Принцип надежности:** 100% Safe Rust, Never Crash (0 `unwrap()`, 0 `expect()`, 0 `panic!()`, 0 `unsafe`).  
> **Локализация:** 0% строковых литералов на русском/английском в коде UI (строго макрос `t!()` и файлы `locales/ru.ftl`, `locales/en.ftl`).  
> **Интерфейс:** Vector-First UI (0 эмодзи, 0 квадратов «тофу»).

---

## 1. Цели задачи
1. Реализовать параметрические критерии сравнения средних в L2 `statcraft_algo`:
   - **Двухвыборочный t-критерий для независимых выборок** (*Independent-Samples T-Test*):
     * Критерий равенства дисперсий Ливиня (*Levene's Test*) с $F$-статистикой и $p$-значением;
     * $t$-критерий Стьюдента для равных дисперсий ($df = n_1 + n_2 - 2$);
     * $t$-критерий Уэлча для неравных дисперсий с поправкой Саттертуэйта на степени свободы;
     * Разность средних, стандартная ошибка разности и 95% доверительный интервал.
   - **Парный t-критерий** (*Paired-Samples T-Test*) для зависимых выборок (до / после воздействия).
   - **Одновыборочный t-критерий** (*One-Sample T-Test*) со сравнением с тестовым числом $\mu_0$.
   - **Однофакторный дисперсионный анализ** (*One-Way ANOVA*): межгрупповая и внутригрупповая дисперсии, критерий Фишера $F$, точное $p$-value, критерий Тьюки (*Tukey HSD*).
2. Зарегистрировать команды в L5 `statcraft_syntax`:
   - `stat.ttest.independent`, `stat.ttest.paired`, `stat.ttest.one_sample`, `stat.anova.oneway`.
   - Поддержка генерации классического синтаксиса SPSS:
     * `T-TEST GROUPS=gender(1 2) /VARIABLES=salary.`
     * `T-TEST PAIRS=pre_test WITH post_test (PAIRED).`
     * `ONEWAY salary BY educ /STATISTICS DESCRIPTIVES.`
3. Создать диалоги в `statcraft_app/src/dialogs/`:
   - `ttest_ind_dialog.rs`: выбор тестовых переменных, группирующей переменной и кнопка «Задать группы...» (*Define Groups*);
   - `ttest_paired_dialog.rs`: выбор связанных пар переменных;
   - `anova_dialog.rs`: выбор зависимых переменных и категориального фактора.
4. Вывести аутентичные сводные таблицы в `Output Viewer`:
   - **Статистики групп** (*Group Statistics*): $N$, Mean, Std. Dev, Std. Error Mean;
   - **Критерий для независимых выборок** (*Independent Samples Test*): двухстрочная таблица (с предположением и без предположения равенства дисперсий);
   - **Таблица дисперсионного анализа** (*ANOVA Table*): Sum of Squares, df, Mean Square, F, Sig.

---

## 2. Пошаговый план изменений

### 2.1. L2 `statcraft_algo`
- Создать `crates/statcraft_algo/src/ttest.rs`:
  * Функции: `compute_independent_ttest`, `compute_paired_ttest`, `compute_one_sample_ttest`;
  * Вспомогательный расчет критерия Ливиня (`compute_levene_test`);
  * Golden Oracle unit-тесты со сверкой со `scipy.stats.ttest_ind`, `scipy.stats.levene` и `scipy.stats.ttest_rel` с точностью $\le 10^{-6}$.
- Создать `crates/statcraft_algo/src/anova.rs`:
  * Функция `compute_oneway_anova`;
  * Golden Oracle unit-тесты со сверкой со `scipy.stats.f_oneway`.
- Зарегистрировать модули в `crates/statcraft_algo/src/lib.rs`.

### 2.2. L5 `statcraft_syntax`
- Создать `crates/statcraft_syntax/src/commands/ttest_cmds.rs`:
  * `IndependentTTestCommand`, `PairedTTestCommand`, `OneSampleTTestCommand`, `OneWayAnovaCommand`;
  * Валидация предиката `enabled` (например, для независимого t-теста в данных обязана быть хотя бы одна числовая колонка и одна группирующая переменная минимум с 2 группами);
  * Генерация `.sps` синтаксиса.
- Зарегистрировать в `CommandBus`.

### 2.3. L6 `statcraft_app`
- Создать `crates/statcraft_app/src/dialogs/ttest_ind_dialog.rs`:
  * Компактное модальное окно;
  * Список числовых переменных $\to$ поле «Проверяемые переменные» (*Test Variables*);
  * Поле «Группирующая переменная» (*Grouping Variable*);
  * Вложенное модальное окно «Задать группы» (*Define Groups: Group 1, Group 2*).
- Создать `crates/statcraft_app/src/dialogs/anova_dialog.rs`.
- В `crates/statcraft_app/src/views/output_view.rs`:
  * Добавить рендеринг таблицы `Group Statistics` и двухстрочной таблицы `Independent Samples Test`.
- В `crates/statcraft_app/src/app.rs`:
  * Добавить пункты в меню `Analyze > Compare Means > Independent-Samples T Test...` и `One-Way ANOVA...`.

### 2.4. Локализация (`locales/ru.ftl` и `locales/en.ftl`)
- Все ключи меню, диалогов и заголовков сводных таблиц.

---

## 3. Критерии приёмки и верификация (Definition of Done)
1. Строго `-j 1` на всех командах.
2. `cargo test --workspace -j 1` — 100% тестов завершились с кодом 0.
3. `cargo test --package statcraft_i18n --test i18n_lint_test -j 1` — 0% захардкоженных строк в UI.
4. `cargo fmt --check` и `cargo clippy --workspace -j 1 -- -D warnings` — чисто.
5. `cargo run --package xtask -j 1 -- layers` — без нарушений архитектуры.
6. Обновить `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md` с фиксацией выполненной работы.

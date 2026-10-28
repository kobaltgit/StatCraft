# ТЗ для ИИ-агента: Этап 3 — Частотный анализ и словарь метаданных (Phase 3)

> **Стандарт экосистемы:** ArtCraft (`storytold`) | Двойная лицензия MIT OR Apache-2.0  
> **Генеральный регламент:** [AGENTS.md](file:///d:/Projects/active/StatCraft/AGENTS.md)  
> **Ограничение ресурсов:** ВСЕ команды сборки и тестов запускаются строго с флагом `-j 1`.  
> **Принцип надежности:** 100% Safe Rust, Never Crash (0 `unwrap()`, 0 `expect()`, 0 `panic!()`, 0 `unsafe`).  
> **Локализация:** 0% строковых литералов на русском/английском в коде UI (строго макрос `t!()` и файлы `locales/ru.ftl`, `locales/en.ftl`).  
> **Интерфейс:** Vector-First UI (0 эмодзи, 0 квадратов «тофу»).

---

## 1. Цели задачи
1. Реализовать статистическую процедуру **Частотного анализа** (*Frequencies*):
   - Расчёт частот, процентов, валидных процентов и накопленных (кумулятивных) процентов;
   - Расчёт квартилей ($Q_1 = 25\%$, $Q_2 = 50\%$ медиана, $Q_3 = 75\%$) и перцентилей.
2. Создать типизированную команду `FrequenciesCommand` (`stat.frequencies`) в L5 с генерацией SPSS-синтаксиса `FREQUENCIES VARIABLES=... /STATISTICS=STDDEV MEAN MEDIAN /ORDER=ANALYSIS.`.
3. Создать центрированный диалог выбора переменных `FrequenciesDialog` в `statcraft_app/src/dialogs/frequencies_dialog.rs`.
4. Реализовать рендеринг классических таблиц частот SPSS в `Output Viewer` на всю высоту вкладки.
5. Реализовать диалог **«Метки значений»** (*Value Labels*) при нажатии на кнопку `[...]` в колонке «Значения» в представлении переменных (*Variable View*):
   - Добавление, редактирование и удаление пар `Код -> Метка` (например, `1 = Мужской`, `2 = Женский`).
   - Команда `VarSetValueLabelsCommand` с Undo/Redo.
6. Реализовать диалог **«Пропущенные значения»** (*Missing Values*) при клике на `[...]` в колонке «Пропуски»:
   - Задание до 3 дискретных значений пропусков или диапазона.
   - Команда `VarSetMissingValuesCommand` с Undo/Redo.
7. Подключить кнопку `1/A` на панели инструментов Data View для мгновенного переключения между сырыми кодами и метками значений.

---

## 2. Пошаговый план изменений

### 2.1. L2 `statcraft_algo` (`crates/statcraft_algo/src/frequencies.rs`)
- Расширить структуру `FrequencyResult`:
  * Список строк категорий `FrequencyCategory { value: String, count: usize, percent: f64, valid_percent: f64, cumulative_percent: f64 }`;
  * Квартили `quartiles: Option<(f64, f64, f64)>` для числовых переменных;
  * Общее число валидных $N$ и пропущенных наблюдений.
- Написать модульные тесты `#[cfg(test)] mod tests` со сверкой с Python `scipy.stats` / `pandas.value_counts(normalize=True)`.

### 2.2. L5 `statcraft_syntax` (`crates/statcraft_syntax/src/commands/`)
- Создать `frequencies_cmd.rs`:
  * Реализация `EngineCommand` для `stat.frequencies`;
  * Сериализация спецификации параметров `specs()`;
  * Метод `to_sps_syntax(&self) -> String`;
  * Зарегистрировать команду в `CommandBus::with_defaults()`.
- Создать команды метаданных `var_metadata_cmds.rs`:
  * `VarSetValueLabelsCommand` (`var.set_value_labels`);
  * `VarSetMissingValuesCommand` (`var.set_missing_values`).

### 2.3. L6 `statcraft_app`
- Создать `crates/statcraft_app/src/dialogs/frequencies_dialog.rs`:
  * Центрированное окно $500 \times 420$ px;
  * Списки доступных и выбранных переменных с иконками шкал и двойным кликом;
  * Чекбокс «Выводить таблицы частот» (*Display frequency tables*);
  * Кнопки `OK`, `Вставить (Paste)`, `Сброс (Reset)`, `Отмена (Cancel)`.
- Создать `crates/statcraft_app/src/dialogs/value_labels_dialog.rs`:
  * Модальное окно: поле ввода значения (Value), поле ввода метки (Label);
  * Кнопки `[ Добавить ]`, `[ Изменить ]`, `[ Удалить ]`;
  * Список уже назначенных меток с прокруткой.
- Создать `crates/statcraft_app/src/dialogs/missing_values_dialog.rs`:
  * Радиокнопки: «Нет пропущенных значений», «Дискретные пропущенные значения» (3 поля ввода), «Диапазон плюс одно дискретное значение».
- В `crates/statcraft_app/src/views/output_view.rs`:
  * Добавить рендеринг таблицы частот SPSS: колонки `Значение | Частота | Процент | Валидный процент | Накопленный процент`.

### 2.4. Локализация (`locales/ru.ftl` и `locales/en.ftl`)
- Добавить все ключи для диалогов частот, меток значений и пропусков на русском и английском языках.

---

## 3. Критерии приёмки и верификация (Definition of Done)
1. Строго `-j 1` на всех командах.
2. `cargo test --workspace -j 1` — 100% тестов завершились с кодом 0.
3. `cargo test --package statcraft_i18n --test i18n_lint_test -j 1` — 0% захардкоженных строк в UI.
4. `cargo fmt --check` и `cargo clippy --workspace -j 1 -- -D warnings` — чисто.
5. `cargo run --package xtask -j 1 -- layers` — без нарушений архитектуры.
6. Обновить `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md` с фиксацией выполненной работы.

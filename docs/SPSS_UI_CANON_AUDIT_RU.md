# StatCraft — Канонический аудит интерфейса, Output Viewer и дизайн-системы IBM SPSS Statistics (Этап 3)

[ English ](SPSS_UI_CANON_AUDIT.md) | **Русский (Основной)**

> **Статус документа:** Официальный доказательный протокол аудита графического интерфейса пользователя (Desktop GUI), генераторов таблиц Output Viewer, подсистемы DialogKit и дистрибуции.  
> **Методология:** Clean-Room исследование и доказательная верификация с помощью MCP-сервера [Craft Anything (`storytold/craft-anything`)](https://github.com/storytold/craft-anything).  
> **Эталонные стандарты:** IBM SPSS Statistics Desktop Interface Guide, канон дизайн-системы ArtCraft ([`AGENTS.md`](../AGENTS.md)) и [`docs/DESIGN_GUIDE.md`](DESIGN_GUIDE.md).  
> **Ограничение ресурсов (Resource Guard):** Сборка и тесты строго под `-j 1`.

---

## 1. Обзор аудита и область проверки

Этап 3 завершает комплексный аудит StatCraft, охватывая пользовательский интерфейс, экранные формы редактирования данных, процедурные модальные окна, каталоги локализации и инсталляционные пакеты в рамках 13 специализированных микропланов (M-21 — M-33):

```text
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                        ЭТАП 3 АУДИТА: ИНТЕРФЕЙС, ТАБЛИЦЫ ВЫВОДА, DIALOGKIT И СБОРКА                    │
├────────────────────────────┬─────────────────────────────┬────────────────────────┬────────────────────┤
│         РАЗДЕЛ E           │          РАЗДЕЛ F           │        РАЗДЕЛ G        │      РАЗДЕЛ H      │
│     Output Viewer Tables   │     Interactive Screens     │    DialogKit (SPSS)    │  i18n & Packaging  │
│  • M-21: Базовые таблицы   │  • M-25: Data View Grid     │  • M-28: Common Shell  │  • M-30: Fluent FTL│
│  • M-22: Сравнение средних │  • M-26: Variable View      │  • M-29: 37 Dialogs    │  • M-31: Vector UI │
│  • M-23: Связи и многомерн │  • M-27: Syntax Editor      │    (Design Guard)      │  • M-32: Session   │
│  • M-24: Outline Tree      │                             │                        │  • M-33: Installer │
└────────────────────────────┴─────────────────────────────┴────────────────────────┴────────────────────┘
```

---

## 2. Раздел E: Output Viewer и канонические сводные таблицы

### M-21: Таблицы базовой описательной статистики и диаграммы
* **Файлы реализации:** `crates/statcraft_app/src/views/output_view/` (`tables_descriptives.rs`, `tables_frequencies/` (`freq_table.rs`, `stats_summary_table.rs`, `freq_chart_bar_pie.rs`, `freq_chart_histogram.rs`, `mod.rs`), `tables_crosstabs.rs`, `tables_means.rs`, `tables_common.rs`)
* **Канон SPSS:** *Base Statistics Output Tables Specification*
* **Гипотеза Craft Anything:** `hyp_33` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Таблица описательных статистик (Descriptives):** Стандартный состав колонок (`N`, `Minimum`, `Maximum`, `Mean`, `Std. Deviation`, `Variance`, `Skewness`, `Kurtosis`) с выравниванием чисел по правому краю и динамическим расчетом ширины по правилу `i18n Shield`. Подвальная строка `Valid N (listwise)` сохранена.
  2. **Таблица частот (Frequencies):** Реализует 4-колоночный канон SPSS (`Frequency`, `Percent`, `Valid Percent`, `Cumulative Percent`) с отдельными строками для валидных категорий, пропусков `Missing` (System / User) и итогов `Total`.
  3. **Таблица сопряженности (Crosstabs):** Многоярусный вложенный макет с выводом `Count`, `Expected Count`, `% within Row`, `% within Column`, `% of Total` и `Residuals`. Интегрированная таблица `Chi-Square Tests` (критерий Пирсона, поправка Йейтса, отношение правдоподобия, точный критерий Фишера, линейно-линейная связь).
  4. **Форматирование вероятностей (p-value):** Уровни значимости ниже $0.001$ выводятся с подавлением нуля как `< .001` по стандарту APA / SPSS.
  5. **Векторный рендеринг диаграмм частот (Vector-First Charts):**
     - **Столбчатые диаграммы (`BarChart`):** Автоподбор шага оси $Y$ (`calculate_nice_step`), горизонтальные линии координатной сетки, интерактивная подсветка столбца (`hover`), вывод значений над столбцами (частоты или проценты) и подписи категорий по оси $X$.
     - **Круговые диаграммы (`PieChart`):** Секторная заливка методом триангуляции `convex_polygon` по часовой стрелке с 12:00, каноническая 10-цветовая палитра `PIE_PALETTE`, векторная легенда справа со свотчами и процентами.
     - **Гистограммы (`Histogram`):** Непрерывная шкала $X$ с касающимися бинами, наложение нормальной кривой Гаусса $\mathcal{N}(\mu, \sigma^2)$ канонического оранжевого цвета SPSS (`#ED7D31`) и статистический бейдж распределения в правом верхнем углу (`Mean`, `Std. Dev.`, `N`).
     - **Стандарт рендеринга:** 100% векторная графика `ui.painter()`, нулевое использование растра, эмодзи и неподдерживаемых глифов Unicode.
  6. **Индикация подавления таблиц (`table_suppressed`):** При превышении лимита категорий `LIMIT(n)` выводится аутентичное курсивное уведомление цвета `Tokens::text_muted()` (*«Таблица частот для переменной [NAME] не выводится, так как число категорий превышает заданный лимит.»*) с полным сохранением сводной таблицы «Статистика» и диаграмм.
  7. **Организация вывода нескольких переменных (`MultipleVarsMode`):** Поддерживает режимы `Compare` (единая сводная таблица описательных статистик, последовательные таблицы частот, отдельный узел графиков) и `Variable` (автономные блоки по каждой переменной: персональная статистика $\to$ таблица частот $\to$ диаграмма). Проверено через `output_viewer_frequencies_invariants.rs` (8/8 успешно).

### M-22: Таблицы сравнения средних
* **Файлы реализации:** `tables_ttest_onesample.rs`, `tables_ttest_indep.rs`, `tables_ttest_paired.rs`, `tables_anova.rs`
* **Канон SPSS:** *Compare Means Output Tables Specification*
* **Гипотеза Craft Anything:** `hyp_34` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Independent Samples Test:** Двухъярусный заголовок, объединяющий *Критерий равенства дисперсий Левена* ($F$, Sig.) и *t-критерий равенства средних* ($t$, $df$, Sig., Mean Diff, Std. Error Diff, 95% ДИ). Присутствуют две канонические строки: `Equal variances assumed` и `Equal variances not assumed`.
  2. **Paired Samples Test:** Выводятся 3 независимые таблицы SPSS: *Paired Samples Statistics*, *Paired Samples Correlations* и *Paired Samples Test* (Paired Differences).
  3. **One-Way ANOVA:** Генерируются *Descriptives*, *Test of Homogeneity of Variances* (тест Левена) и 3-секционная сводная таблица *ANOVA* (`Between Groups`, `Within Groups`, `Total` со столбцами $SS$, $df$, $MS$, $F$, Sig.).

### M-23: Таблицы многомерного анализа и непараметрики
* **Файлы реализации:** `tables_correlations.rs`, `tables_regression.rs`, `tables_factor.rs`, `tables_reliability.rs`, `tables_kruskal.rs`, `tables_nonparam.rs`, `tables_nonparam_related.rs`, `tables_adp.rs`, `tables_programmability.rs`
* **Канон SPSS:** *Advanced Statistics Output Tables Specification*
* **Гипотеза Craft Anything:** `hyp_35` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Парные корреляции (Correlations):** Вывод коэффициентов корреляции, 1-сторонней / 2-сторонней значимости, размера выборки $N$ и канонических звездочек значимости (`*` $p < .05$, `**` $p < .01$) со сносками внизу таблицы.
  2. **Линейная регрессия:** Блок из 4 таблиц SPSS: *Variables Entered/Removed*, *Model Summary* ($R, R^2, \text{Adj } R^2, SE$), *ANOVA* и *Coefficients* ($B$, Std. Error, $\beta$, $t$, Sig., 95% CI, коллинеарность: Tolerance, VIF).
  3. **Факторный анализ:** Таблицы *KMO and Bartlett's Test*, *Communalities* (начальные и выделенные), *Total Variance Explained* и матрица факторных нагрузок *Component Matrix*.
  4. **Анализ надежности (Reliability):** *Case Processing Summary*, *Reliability Statistics* ($\alpha$ Кронбаха), *Item Statistics* и *Item-Total Statistics*.
  5. **Непараметрический блок:** Таблицы рангов (Ranks) и статистики критериев (Test Statistics) для критериев Манна-Уитни ($U$), Вилкоксона ($W$) и Краскела-Уоллиса ($H$).

### M-24: Дерево оглавления и экспорт вывода
* **Файлы реализации:** `outline.rs`, `blocks.rs`, `builder.rs`, `export.rs`, `model.rs`, `widgets/output_icons.rs`
* **Канон SPSS:** *SPSS Viewer DOM & Export Framework*
* **Гипотеза Craft Anything:** `hyp_36` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Иерархия оглавления:** Документное дерево `Output Root` $\to$ `Procedure Node` $\to$ `Log (синтаксис .sps)` $\to$ `Notes` $\to$ `Pivot Tables` $\to$ `FrequencyCharts` (векторная иконка `paint_output_chart_icon`: 3 столбца на базовой линии).
  2. **Интерактивная навигация:** Клик по любому узлу дерева плавно прокручивает рабочую область к целевой таблице или диаграмме.
  3. **Контекстные операции:** Меню правой кнопки мыши: копирование таблицы или диаграммы в TSV, копирование текста и удаление узла.
  4. **Модуль экспорта:** Полный экспорт журнала, выбранной таблицы или диаграмм в форматы TSV, стилизованный HTML и Plain Text через крейт `statcraft_reporting`. Поддержка табличного экспорта распределения диаграмм в TSV (`frequency_charts_to_tsv`).

---

## 3. Раздел F: Интерактивные редакторы данных

### M-25: Data View (Виртуализированный грид данных)
* **Файл реализации:** `crates/statcraft_app/src/views/data_view.rs`
* **Канон SPSS:** *SPSS Data Editor — Data View*
* **Гипотеза Craft Anything:** `hyp_37` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Виртуализация отображения:** Высокопроизводительный метод `ScrollArea::vertical().show_rows(...)`, отрисовывающий исключительно видимый диапазон строк и обеспечивающий 60 FPS на массивах данных.
  2. **Инлайн-редактирование ячеек:** Активация по двойному клику, полная поддержка навигации с клавиатуры (Tab, Enter, стрелки) и атомарная запись команд в историю Undo.
  3. **Переключатель Value Labels:** Кнопка панели инструментов (`1/A`), переключающая отображение между числовыми кодами и текстовыми метками значений.
  4. **Перечеркивание неактивных строк:** Канонический диагональный слэш поверх номеров строк в заголовках при включенном фильтре `FILTER BY`.

### M-26: Variable View (Редактор словаря метаданных)
* **Файл реализации:** `crates/statcraft_app/src/views/variable_view.rs`
* **Канон SPSS:** *SPSS Data Editor — Variable View*
* **Гипотеза Craft Anything:** `hyp_38` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **11 колонок метаданных:** Name, Type, Width, Decimals, Label, Values, Missing, Columns, Align, Measure, Role.
  2. **Валидация имен переменных:** Жесткое соблюдение правил SPSS (длина до 64 символов, не начинается с цифры или точки, отсутствие пробелов и спецсимволов, запрет зарезервированных слов `ALL`, `AND`, `BY`, `EQ` и др.).
  3. **Модальные вызовы:** Кнопки с многоточием (`...`) вызывают модальные диалоги (`ValueLabelsDialog`, `MissingValuesDialog`, `VariableTypeDialog`).

### M-27: Syntax Editor (Редактор синтаксиса)
* **Файл реализации:** `crates/statcraft_app/src/views/syntax_editor.rs`
* **Канон SPSS:** *SPSS Syntax Editor & Batch Processor*
* **Гипотеза Craft Anything:** `hyp_39` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Моноширинная область кода:** Поле ввода синтаксиса с автоматической генерацией заголовка сессии и временных меток.
  2. **Исполнение команд:** Интеграция с `CommandBus` для пакетного выполнения всего скрипта или выделенного фрагмента с записью синтаксического эхо в лог вывода.
  3. **Шина вставки (Paste):** Каждое процедурное диалоговое окно снабжено кнопкой `[ Paste ]`, генерирующей точный синтаксис `.sps` в редактор.

---

## 4. Раздел G: Дизайн-система DialogKit и 37 модальных диалогов

### M-28: Канонические примитивы DialogKit
* **Файлы реализации:** `crates/statcraft_app/src/dialogs/common/` (`dialog_shell.rs`, `action_bar.rs`, `subdialog_action_bar.rs`, `subdialog_sidebar.rs`, `variable_picker.rs`, `transfer_button.rs`, `group_box.rs`)
* **Канон SPSS:** *SPSS Canonical Dialog Architecture & Ergonomics*
* **Гипотеза Craft Anything:** `hyp_40` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Типоразмеры окон (`DialogSize`):** Standard ($560 \times 430$), Subdialog ($360 \times 300$), Properties ($460 \times 380$), Compute ($620 \times 480$), Alert ($440 \times 170$).
  2. **Нижний ряд кнопок Type 1 (`DialogActionBar`):** Каноническая последовательность `[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`.
  3. **i18n Shield:** Одинаковая расчетная ширина всех кнопок ряда actions на базе максимальной длины локализованного текста ($\ge 80 \times 26$ px, скругление 3px).
  4. **Блокировка кнопки OK:** Кнопка `[ OK ]` автоматически становится неактивной при отсутствии обязательных входных переменных.
  5. **Компоненты выбора переменных:** Векторные значки шкал (Scale, Ordinal, Nominal) и адаптивная кнопка переноса `TransferButton` (`->` / `<-`).

### M-29: Пакет процедурных диалогов (37 диалогов и поддиалоги)
* **Файлы реализации:** `crates/statcraft_app/src/dialogs/*.rs` (включая `frequencies_dialog/format_subdialog.rs`, `charts_subdialog.rs` и `statistics_subdialog.rs`)
* **Канон SPSS:** *SPSS Procedural Dialogs Suite*
* **Гипотеза Craft Anything:** `hyp_41` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Статический линтер Design Guard (`cargo run -p xtask -j 1 -- design-guard`):** 0 нарушений. Полный запрет сырых кнопок `ui.button(...)`, 100% покрытие `DialogKit`.
  2. **Каноническая архитектура поддиалогов:** Модальные поддиалоги Type 2 (включая *«Частоты: Статистики»*, *«Частоты: Диаграммы»* и *«Частоты: Формат»*, $360 \times 300$ px) строятся на базе `DialogShell::subdialog`, `GroupBox` и `SubdialogActionBar` (`[ Продолжить ] [ Отмена ] [ Справка ]`) с надежным откатом изменений при отмене.
     - *«Частоты: Формат»*: двухколоночный выбор порядка категорий (`CategoryOrder` — 4 радиокнопки), организации вывода переменных (`MultipleVarsMode` — 2 радиокнопки), чекбокс и зависимое поле порога подавления таблиц со многими категориями `LIMIT(n)`;
     - *«Частоты: Диаграммы»*: правила взаимной блокировки (Гистограммы vs Столбчатые/Круговые, доступность нормальной кривой).
  3. **Идемпотентность кнопки Reset:** Нажатие `[ Reset ]` гарантированно возвращает все списки и флаги в исходное состояние во всех диалогах.
  4. **Автоматический набор тестов:** Пройдено через `cargo test -p statcraft_app --test dialog_invariants -j 1` (43/43 успешно, включая `test_frequencies_format_subdialog_transactionality_on_cancel` и `test_frequencies_format_subdialog_commit_and_action_run`).

---

## 5. Раздел H: Локализация, векторный UI, сессия и дистрибуция

### M-30: Каталоги локализации Fluent (Zero-Hardcode i18n)
* **Файлы реализации:** `locales/en.ftl`, `locales/ru.ftl`, `crates/statcraft_i18n/`
* **Канон ArtCraft:** *Zero-Hardcode i18n Regulation*
* **Гипотеза Craft Anything:** `hyp_42` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **100% паритет ключей:** Полная синхронизация между каталогами `en.ftl` и `ru.ftl` без пропущенных или пустых ключей.
  2. **Нулевой хардкод строк:** Тест `i18n_lint_test.rs` подтверждает полное отсутствие строковых литералов естественных языков в кодовой базе интерфейса.
  3. **Смена языка на лету:** Переключение языка интерфейса в меню происходит без перезапуска программы и без потери открытого проекта.

### M-31: Векторный рендеринг (Vector-First UI)
* **Файлы реализации:** `scale_icon.rs`, `vector_arrow.rs`, `output_icons.rs`, `theme.rs`
* **Канон ArtCraft:** *Vector-First UI Regulation*
* **Гипотеза Craft Anything:** `hyp_43` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Векторные примитивы:** Все иконки (линейка Scale, столбики Ordinal, кружки Nominal, стрелки, папки, лог) отрисованы через `ui.painter()`.
  2. **Запрет квадратов «тофу»:** 100% исключение Unicode эмодзи (`📊`, `📈`, `⚙`, `⚠️`, `💡`) и редких спец-символов (`●`, `•`, `✓`, `✔`, `✕`, `✖`), гарантирующее идеальное отображение на любых ОС.

### M-32: Управление сессией и защита данных (Session Guard)
* **Файлы реализации:** `app_session.rs`, `recent_files.rs`, `confirm_save_dialog.rs`
* **Канон ArtCraft:** *Data Safety & Session Invariants*
* **Гипотеза Craft Anything:** `hyp_44` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Dirty Flag:** Любая модификация датасета выставляет флаг изменений с символом `*` в заголовке окна.
  2. **Диалог ConfirmSaveDialog:** Предотвращает потерю несохраненных правок при создании нового проекта, открытии файла или закрытии программы.
  3. **Очередь Recent Files:** Безопасное отслеживание недавно открытых файлов с проверкой их доступности. Проверено в `session_lifecycle_tests.rs` (12/12 успешно).

### M-33: Кроссплатформенная сборка и инсталляторы
* **Файлы реализации:** `packaging/windows/installer.iss`, `packaging/macos/Info.plist`, `packaging/linux/control`, `.github/workflows/release.yml`, `Cargo.toml`
* **Канон ArtCraft:** *Native Desktop Packaging Canon*
* **Гипотеза Craft Anything:** `hyp_45` (Подтверждена / Supported)
* **Проверенные инварианты и доказательства:**
  1. **Синхронизация версий:** Версия `0.2.1` синхронизирована байт-в-байт в 4 файлах манифестов.
  2. **Ассоциации расширений:** Настроена регистрация системных расширений `.sav`, `.zsav`, `.stc`, `.sps`.
  3. **Матричный CI:** Автоматизированная матричная сборка под Windows (Inno Setup installer + ZIP), Linux (.deb + tar.gz) и macOS (DMG + .app bundle).

---

## 6. Итоговый вердикт аудита

* **Всего проаудировано микропланов:** 13 (M-21 — M-33)
* **Подтверждено гипотез Craft Anything:** 13/13 (100% Supported)
* **Результаты автоматических проверок (-j 1):**
  - `dialog_invariants`: 37/37 УСПЕШНО
  - `output_viewer_invariants`: 11/11 УСПЕШНО
  - `session_lifecycle_tests`: 12/12 УСПЕШНО
  - `i18n_lint_test`: 4/4 УСПЕШНО
  - `xtask design-guard`: 0 нарушений УСПЕШНО
  - `xtask file-guard`: 267 файлов проверено УСПЕШНО
  - `xtask layers`: УСПЕШНО
* **Вердикт по Этапу 3:** **ПОЛНОЕ СООТВЕТСТВИЕ КАНОНУ ИНТЕРФЕЙСА IBM SPSS STATISTICS И РЕГЛАМЕНТУ ARTCRAFT ПОДТВЕРЖДЕНО**.

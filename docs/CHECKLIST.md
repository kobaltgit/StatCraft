# StatCraft — Чек-лист разработки (Phase 1: Scaffolding & Core MVP)

## 1. Инфраструктура и качество
- [x] Инициализация Git-репозитория и ветки `main`
- [x] Генеральный регламент разработки `AGENTS.md` (Канон ArtCraft)
- [x] Конфигурация Cargo с Resource Guard (`jobs = 1`, `codegen-units = 1`)
- [x] Двойная лицензия MIT / Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, `ATTRIBUTION.md`)
- [x] Архитектурный валидатор слоев `cargo xtask layers`
- [x] Матрица паритета `cargo xtask parity` и `docs/parity-checklist.md`

## 2. Локализация без хардкода (statcraft_i18n)
- [x] Каталоги Fluent `locales/en.ftl` и `locales/ru.ftl`
- [x] Менеджер локализации `I18nCatalog` и макрос `t!()`
- [x] Модульные тесты перевода, fallback и интерполяции параметров (100% функций)
- [x] Автоматический линтер UI-кода `i18n_lint_test` (0% сырой кириллицы в коде интерфейса)

## 3. Ядро данных и метаданных (statcraft_core)
- [x] Модель метаданных `VariableDictionary` (Measure: Scale/Ordinal/Nominal, ValueLabels, MissingValues)
- [x] Колоночное хранилище `DataSet` с буфером транзакций `DeltaBuffer` (Undo/Redo ячеек)
- [x] Модульные тесты каждой функции ядра (гранулированные правки, границы, сериализация)

## 4. Кодеки и импорт данных (statcraft_io)
- [x] Потоковый CSV/TSV загрузчик с автоматическим выводом типов колонок
- [x] Обработка некорректных форматов без паник (`Never Crash`)
- [x] Модульные тесты парсера (валидные, битые строки, разделители, пропущенные значения)

## 5. Командная шина движка (statcraft_syntax)
- [x] Трейт `EngineCommand` (id, specs для ИИ-агентов, enabled, execute, undo)
- [x] Шина команд `CommandBus` с историей Undo/Redo
- [x] Реализация базовых команд (`data.import_csv`, `var.set_label`, `var.set_measure`, `data.cell_edit`)
- [x] Модульные тесты команд и спецификаций параметров

## 6. Графический интерфейс на egui (statcraft_app)
- [x] Док-среда `egui_dock` (Data View, Variable View, Output Viewer)
- [x] Виртуализированный грид таблицы `Data View` (скролл больших массивов, переключение меток)
- [x] Табличный редактор словаря `Variable View` (шкалы, имена, типы, метки)
- [x] Векторные индикаторы шкал (Scale, Nominal, Ordinal) без эмодзи (Vector-First UI)
- [x] Очищенная брендовая векторная иконка `assets/icon.svg` и нативный виджет `paint_app_icon`
- [x] Динамическое переключение языков интерфейса «на лету» (RU $\leftrightarrow$ EN)
- [x] Проверка компиляции и всех тестов с флагом `-j 1` (38/38 тестов пройдено)

## 7. Этап 2: Описательные статистики (Descriptives)
- [x] L2 `statcraft_algo`: процедура `compute_descriptives` со сверкой с Python `scipy.stats` (Golden Oracle)
- [x] L5 `statcraft_syntax`: типизированная команда `stat.descriptives` и генерация синтаксиса `DESCRIPTIVES VARIABLES=...`
- [x] L6 `statcraft_app`: компактный центрированный диалог выбора переменных `DescriptivesDialog`
- [x] L6 `statcraft_app`: отображение сводных таблиц (Pivot Table) на всю высоту `Output Viewer`

## 8. Этап 3: Частотный анализ и словарь метаданных (Frequencies & Metadata)
- [x] L2 `statcraft_algo`: расширенная процедура `compute_frequencies_detailed` с поддержкой меток значений и кодов пропусков
- [x] L2 `statcraft_algo`: вычисление перцентилей и квартилей ($Q_1, Q_2, Q_3$) по стандарту SPSS HAVERAGE
- [x] L5 `statcraft_syntax`: команда `stat.frequencies` с трансляцией в синтаксис `FREQUENCIES VARIABLES=... /STATISTICS=STDDEV MEAN MEDIAN /ORDER=ANALYSIS.`
- [x] L5 `statcraft_syntax`: команды метаданных `var.set_value_labels` и `var.set_missing_values` с историей Undo/Redo
- [x] L6 `statcraft_app`: центрированный диалог частотного анализа `FrequenciesDialog` ($500 \times 420$ px)
- [x] L6 `statcraft_app`: модальный диалог меток значений `ValueLabelsDialog` (добавление, изменение, удаление)
- [x] L6 `statcraft_app`: модальный диалог пропущенных значений `MissingValuesDialog` (дискретные и диапазонные пропуски)
- [x] L6 `statcraft_app`: вызов редакторов метаданных через аутентичные кнопки `[...]` в Variable View
- [x] L6 `statcraft_app`: переключатель `1/A` на панели инструментов Data View для мгновенного отображения меток значений
- [x] L6 `statcraft_app`: рендеринг классических таблиц частот SPSS и перцентилей в Output Viewer
- [x] 100% покрытие локализацией без хардкода строк в `locales/en.ftl` и `locales/ru.ftl` (i18n lint test)
- [x] Полный прогон тестов с флагом `-j 1` (100% тестов завершились с кодом 0)

## 9. Фаза 4: Полноценный двусторонний реактивный редактор (Variable View & Data View)
- [x] L0 `statcraft_io`: расширение `RawTable` операциями добавления/удаления/переименования столбцов и строк
- [x] L1 `statcraft_core`: поддержка всех 9 канонических типов переменных SPSS (`Numeric`, `Comma`, `Dot`, `Scientific`, `Date`, `Dollar`, `CustomCurrency`, `String`, `RestrictedNumeric`)
- [x] L1 `statcraft_core`: форматирование чисел в `get_cell_display` с группировкой тысяч, знаками после запятой и префиксами
- [x] L1 `statcraft_core`: парсинг введенного текста `set_cell_parsed` с автоматической нормализацией разделителей и валют
- [x] L1 `statcraft_core`: режим «Чистый лист» (`DataSet::blank(40, 0)`) для мгновенной готовности к работе без открытия файла
- [x] L5 `statcraft_syntax`: команды `var.rename`, `var.set_type`, `var.set_decimals`, `var.set_width`, `var.set_align`, `var.add`, `var.delete`, `data.add_row`, `data.delete_row`
- [x] L6 `statcraft_app`: центрированное модальное окно `VariableTypeDialog` (9 радиокнопок, настройка ширины, точности, форматов дат и кодов валют)
- [x] L6 `statcraft_app`: полнофункциональный `Variable View` (инлайн-редактирование имени и метки без рамки, интерактивные спиннеры `▲ / ▼` для `Width` и `Decimals`, выбор `Align`, `Measure`, `Role`, контекстное меню строки и строка добавления новой переменной `*`)
- [x] L6 `statcraft_app`: полнофункциональный `Data View` (инлайн-ввод ячеек с фокусом, навигация `Enter` и `Tab`, авторазогрев строк при заполнении нижней строки, контекстное меню строки)
- [x] 100% прохождение 63 тестов воркспейса с `-j 1`, чистота `clippy -D warnings`, `cargo fmt` и `xtask layers`

## 10. Фаза 5: Система вывода и журналирования (Output Viewer & Logging System)
- [x] L6 `statcraft_app`: иерархическая объектная модель документа (`OutputProcedure`, `OutputNotes`, `OutputItem`, `OutputItemKind`, `SelectedTarget`)
- [x] L6 `statcraft_app`: непрерывный бесконечный свиток вывода без замещения предыдущих анализов при навигации
- [x] L6 `statcraft_app`: интерактивное дерево оглавления (Outline Pane) с векторными пиктограммами папок, синтаксиса, примечаний, таблиц и предупреждений
- [x] L6 `statcraft_app`: плавный скроллинг к выбранному элементу при клике в оглавлении с акцентной подсветкой рамки фокуса (`Tokens::ACCENT`)
- [x] L6 `statcraft_app`: автоматическое выделение элемента в оглавлении при клике на соответствующий блок в свитке
- [x] L6 `statcraft_app`: экспорт сводных таблиц и частот в буфер обмена в формате TSV (совместимость с Excel / Google Sheets)
- [x] L6 `statcraft_app`: верхняя панель инструментов оглавления: «Очистить все», «Развернуть все», «Свернуть все»
- [x] L6 `statcraft_app`: удаление процедур и элементов через клавишу `Delete` и контекстное меню
- [x] L6 `statcraft_app`: журнал синтаксиса (Log) и примечания к анализу (Notes) с метаданными датасета и временной меткой
- [x] 100% покрытие локализацией через семантические ключи Fluent, чистота `clippy -D warnings`, `cargo fmt` и `xtask layers`

## 11. Фаза 5.1: Устранение переполнения макетов при смене языка (i18n Layout Shield)
- [x] L6 `statcraft_app`: динамический замер ширин колонок сводных таблиц через `layout_no_wrap` по локализованному тексту
- [x] L6 `statcraft_app`: горизонтальная прокрутка таблиц вывода (`ScrollArea::horizontal`) с отсечением содержимого ячеек (`with_clip_rect`)
- [x] L6 `statcraft_app`: замена текстовых кнопок оглавления на компактные векторные пиктограммы ($24 \times 22$ px) с всплывающими подсказками (`paint_icon_trash`, `paint_icon_expand_all`, `paint_icon_collapse_all`)
- [x] L6 `statcraft_app`: жесткое отсечение границ левой панели оглавления и правого холста (`ui.set_clip_rect`), исключающее пересечение разделителя
- [x] L6 `statcraft_app`: автоматическая адаптация ширин колонок таблицы переменных (`Variable View`) к длине заголовков
- [x] 100% прохождение 65 тестов воркспейса с `-j 1`, чистота `clippy -D warnings`, `cargo fmt` и `xtask layers`

## 12. Фаза 5.3: Каркас 12 канонических меню с бейджами готовности (IBM SPSS 12-Menu System)
- [x] L6 `statcraft_app`: виджет `menu_item.rs` с типизированными состояниями `MenuItemStatus` (`Ready`, `ContextDisabled`, `InDev`) и выравниванием бейджей `[dev]` и горячих клавиш
- [x] `locales/en.ftl` и `locales/ru.ftl`: 100% синхронизация 170+ ключей всех пунктов и подпунктов 12 меню (File, Edit, View, Data, Transform, Analyze, Direct Marketing, Graphs, Utilities, Extensions, Window, Help)
- [x] L6 `statcraft_app`: внедрение полной структуры 12 меню в главное окно приложения с привязкой реальных действий (`Open CSV`, `Undo`, `Value Labels`, `Language`, `Descriptives`, `Frequencies`, `Reset Layout`, `Tab switching`, `Exit`)
- [x] Защита от пустых кликов и подсказки `on_disabled_hover_text` для пунктов в разработке и процедур, требующих датасета
- [x] 100% прохождение всех тестов воркспейса (`cargo test --workspace -j 1`), линтера локализации (`i18n_lint_test`), `cargo xtask layers` и сборка чистого релиза

## 13. Фаза 5.4: Монолитная кастомная панель заголовка (CSD / Title Bar & Top Menus)
- [x] L6 `statcraft_app`: монолитный бар заголовка (Client-Side Decoration / CSD) высотой 34 px в стиле PhotoCraft / Photoshop без системной рамки ОС
- [x] L6 `statcraft_app`: укрупненная брендовая иконка $24 \times 24$ px на левом краю без лишнего дублирующего текста программы
- [x] L6 `statcraft_app`: идеальное вертикальное центрирование иконки и меню (`Layout::left_to_right(Align::Center)`)
- [x] L6 `statcraft_app`: адаптивное схлопывание меню при сужении окна в компактную кнопку переполнения `»` (`menu-overflow`) с вложенными подменю
- [x] L6 `statcraft_app`: перемещение окна за свободное пространство бара заголовка (`ViewportCommand::StartDrag`) и разворачивание/восстановление по двойному клику (`ViewportCommand::Maximized`)
- [x] L6 `statcraft_app`: центрированное отображение имени проекта/датасета (`Untitled-1 •`) в свободном пространстве с мягким отсечением
- [x] L6 `statcraft_app`: векторные кнопки управления окном (Свернуть, Развернуть/Восстановить, Закрыть) с аутентичной hover-подсветкой
- [x] 100% прохождение всех тестов воркспейса с `-j 1`, чистота `clippy -D warnings`, линтер локализации `i18n_lint_test` и релизная сборка

## 14. Фаза 5.6: Панель инструментов (Toolbar / Toolstrip) с векторными SVG-иконками (Канон PhotoCraft)
- [x] L6 `statcraft_app`: интеграция официального загрузчика `egui_extras` (фича `"svg"`) с аппаратной растеризацией под DPI монитора
- [x] L6 `statcraft_app`: локальный каталог `assets/icons/*.svg` и вкомпилирование байтов без растровых файлов и внешних зависимостей (`icon_data.rs`)
- [x] L6 `statcraft_app`: модуль управления иконками `icons.rs` (`SvgIconSet`) с нормализацией цвета к белому и поддержкой динамического тинтинга (`.tint(color)`)
- [x] L6 `statcraft_app`: виджет `toolbar.rs` с поддержкой переключения положения (`Top` / `Left` / `Hidden`) через меню `View -> Toolbars`
- [x] L6 `statcraft_app`: полный канонический набор SPSS из 16 кнопок в 6 логических группах с разделителями (Open, Save, Print, Recall, Undo, Redo, GoTo Case, GoTo Var, Variables Info, Find, Insert Row, Insert Var, Split File, Select Cases, Weight Cases, 1/A Value Labels, Descriptives, Frequencies)
- [x] L6 `statcraft_app`: интерактивные состояния кнопок (Hover, Pressed, Active toggle для `1/A`, Disabled серый цвет для процедур в разработке и пустых датасетов)
- [x] 100% покрытие локализацией всплывающих подсказок с горячими клавишами в `locales/en.ftl` и `locales/ru.ftl` (чистота `i18n_lint_test`)
- [x] 100% прохождение тестов воркспейса с `-j 1`, `cargo xtask layers`, `clippy -D warnings` и сборка чистого релиза `target/release/statcraft_app.exe`

## 15. Этап 6.1: Одновыборочный T-критерий Стьюдента (One-Sample T Test)
- [x] L2 `statcraft_algo`: модуль `ttest_onesample.rs` — расчет функции распределения Стьюдента и обратной квантильной функции через регуляризованную неполную бета-функцию $I_x(a, b)$ и аппроксимацию Ланцоша на 100% чистом safe Rust без внешних зависимостей
- [x] L2 `statcraft_algo`: дескриптивные статистики ($N, \bar{x}, SD, SE$) и метрики критерия ($t, df, p$-value, $\Delta$, 95% ДИ разности) с обработкой граничных случаев ($N < 2$, нулевая дисперсия, пропуски)
- [x] L2 `statcraft_algo`: модульные тесты и сверка Golden Oracle с эталоном SciPy (`stats.ttest_1samp`) с погрешностью $< 10^{-6}$
- [x] L5 `statcraft_syntax`: команда `OneSampleTTestCommand` (`stat.ttest.onesample`) с валидацией параметров и генерацией синтаксиса `T-TEST /TESTVAL=... /CRITERIA=CI(...) /VARIABLES=...`
- [x] L6 `statcraft_app`: модальный диалог `OneSampleTTestDialog` с переносом числовых переменных, настройкой Test Value и % доверительного интервала
- [x] L6 `statcraft_app`: аутентичные сводные таблицы SPSS в Output Viewer («One-Sample Statistics» и «One-Sample Test»), экспорт в TSV/Excel и интеграция в дерево оглавления
- [x] L6 `statcraft_app`: подключение пункта меню `Analyze -> Compare Means -> One-Sample T Test...` с предикатом активности по наличию данных
- [x] Полное покрытие двуязычной локализацией в `locales/en.ftl` и `locales/ru.ftl` с прохождением линтера `i18n_lint_test`
- [x] 100% прохождение тестов воркспейса с `-j 1`, слоев `cargo run -p xtask -j 1 -- layers`, трекера `parity` и чистоты `cargo clippy --workspace -j 1 -- -D warnings`

## 16. Этап 6.2: T-критерий для независимых выборок (Independent-Samples T Test)
- [x] L2 `statcraft_algo`: модуль `ttest_indep.rs` — расчет критерия Левена на равенство дисперсий ($F$, $p$-value) и параллельный расчет двух строк критерия: Student's t (equal variances assumed) и Welch's t (equal variances not assumed со степенями свободы Сазертуэйта)
- [x] L2 `statcraft_algo`: дескриптивные метрики групп ($N, \bar{x}, SD, SE$), разность средних $\Delta$, стандартная ошибка разности и доверительные интервалы для обеих строк
- [x] L2 `statcraft_algo`: сверка Golden Oracle с эталоном SciPy (`levene(center='mean')` и `ttest_ind(equal_var=True/False)`) с точностью $\le 10^{-6}$
- [x] L5 `statcraft_syntax`: команда `IndependentSamplesTTestCommand` (`stat.ttest.indep`) с поддержкой синтаксиса `T-TEST GROUPS=var(val1 val2) /CRITERIA=CI(...) /VARIABLES=...`
- [x] L6 `statcraft_app`: двухпанельный диалог `IndependentSamplesTTestDialog` с переносом проверяемых и группирующей переменной и поддиалогом `Define Groups`
- [x] L6 `statcraft_app`: отображение аутентичных двухъярусных сводных таблиц SPSS («Group Statistics» и «Independent Samples Test» с супер-заголовками Levene и t-test), TSV-экспорт и оглавление
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Compare Means -> Independent-Samples T Test...`
- [x] Полная двуязычная локализация (`en.ftl`, `ru.ftl`), 100% прохождение тестов воркспейса с `-j 1`, `xtask layers`, `xtask parity`, `clippy -D warnings` и линтера i18n

## 17. Этап 6.3: Парный T-критерий Стьюдента для связанных выборок (Paired-Samples T Test)
- [x] L2 `statcraft_algo`: модуль `ttest_paired.rs` — расчет дескриптивных статистик связанных пар ($N, \bar{x}_1, \bar{x}_2, s_1, s_2, SE_1, SE_2$), парных коэффициентов корреляции Пирсона ($r$, $t_r$, $p_r$) и метрик разностей $d_i = x_i - y_i$ ($\bar{d}, s_d, SE_d, t, df, p$-value, доверительный интервал разности)
- [x] L2 `statcraft_algo`: обработка граничных случаев ($N < 2$, нулевая дисперсия разностей, построчное исключение пропусков per-pair listwise) на 100% safe Rust без паник
- [x] L2 `statcraft_algo`: сверка Golden Oracle с эталоном SciPy (`ttest_rel` и `pearsonr`) с точностью $\le 10^{-6}$
- [x] L5 `statcraft_syntax`: команда `PairedSamplesTTestCommand` (`stat.ttest.paired`) с валидацией пар и синтаксисом `T-TEST PAIRS=var1 WITH var2 (PAIRED) /CRITERIA=CI(...).`
- [x] L6 `statcraft_app`: модальный диалог `PairedSamplesTTestDialog` ($520 \times 420$ px) с формированием пар через `Variable 1` и `Variable 2`, векторными кнопками переноса, настройкой % ДИ и канонической горизонтальной панелью кнопок внизу
- [x] L6 `statcraft_app`: три аутентичные сводные таблицы SPSS в Output Viewer («Paired Samples Statistics», «Paired Samples Correlations», «Paired Samples Test» с супер-заголовком Paired Differences), экспорт в TSV/Excel и интеграция в Outline Pane
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Compare Means -> Paired-Samples T Test...` с предикатом активности по наличию числовых колонок
- [x] Полная двуязычная локализация в `locales/en.ftl` и `locales/ru.ftl` со строгим прохождением `i18n_lint_test`
- [x] 100% прохождение всех тестов воркспейса с `-j 1`, чистота `clippy -D warnings`, `cargo fmt` и `xtask layers`

## 18. Этап 6.4: Однофакторный дисперсионный анализ (One-Way ANOVA) и модульная архитектура Output Viewer
- [x] L2 `statcraft_algo`: модуль `anova_oneway.rs` — расчет межгрупповой, внутригрупповой и общей дисперсии ($SS_{between}, SS_{within}, SS_{total}$), степеней свободы ($df_1 = k - 1, df_2 = N - k$), средних квадратов ($MS$), критерия Фишера-Снедекора $F$ и точного $p$-value через регуляризованную неполную бета-функцию $I_x(a, b)$
- [x] L2 `statcraft_algo`: критерий однородности дисперсий Левена (Levene's Test of Homogeneity of Variances) и расчет дескриптивных статистик по группам фактора ($N, \bar{y}, s, SE$, 95% ДИ, Min, Max + Total) на 100% safe Rust без паник
- [x] L2 `statcraft_algo`: сверка Golden Oracle с эталоном SciPy (`scipy.stats.f_oneway`) и замкнутыми формами с точностью $\le 10^{-6}$
- [x] L5 `statcraft_syntax`: команда `OneWayAnovaCommand` (`stat.anova.oneway`) с валидацией факторов и генерацией классического синтаксиса `ONEWAY dep_vars BY factor /STATISTICS DESCRIPTIVES HOMOGENEITY /MISSING ANALYSIS.`
- [x] L6 `statcraft_app`: модальный диалог `OneWayAnovaDialog` ($520 \times 440$ px) с переносом зависимых переменных и категориального фактора, опциями Descriptives и Homogeneity
- [x] L6 `statcraft_app`: архитектурный рефакторинг Output Viewer из монолита в модульный пакет `output_view/` (`model.rs`, `outline.rs`, `tables_descriptives.rs`, `tables_frequencies.rs`, `tables_ttest.rs`, `tables_anova.rs`), предотвращающий срывы контекста
- [x] L6 `statcraft_app`: отображение трех аутентичных сводных таблиц SPSS в Output Viewer («Descriptives», «Test of Homogeneity of Variances», «ANOVA»), TSV-экспорт и интеграция в Outline Pane
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Compare Means -> One-Way ANOVA...`
- [x] Полная двуязычная локализация (`en.ftl`, `ru.ftl`), 100% прохождение всех тестов воркспейса с `-j 1`, `xtask layers`, `xtask parity`, `clippy -D warnings` и `i18n_lint_test`

## 19. Защита от раздутия файлов и регрессий функционала (File Size Guard & Regression Shield)
- [x] Инструментальный рубеж: линтер предельного размера файлов `cargo run -p xtask -j 1 -- file-guard` с дифференцированными лимитами (500 строк для функциональных модулей, 1000 для вьюх, 1500 для координатора)
- [x] Директива «Разделяй, а не урезай»: инструктивные ошибки линтера, обязывающие декомпозировать код на подмодули при строгом запрете сокращения логики
- [x] Архитектурная декомпозиция таблиц T-критерия: разделение монолита `tables_ttest.rs` на три изолированных модуля `tables_ttest_onesample.rs`, `tables_ttest_indep.rs`, `tables_ttest_paired.rs` и `tables_common.rs` ($\le 204$ строк каждый)
- [x] Декомпозиция `model.rs` в `output_view/`: вынос процедурных билдеров в `builder.rs` ($\le 456$ строк)
- [x] Декомпозиция `ttest_indep_dialog.rs` и `var_cmds.rs`: выделение поддиалога `Define Groups` и модуля `var_format_cmds.rs`
- [x] Тестовый рубеж (Оракул полноты): пакет Headless UI-тестов `crates/statcraft_app/tests/output_viewer_invariants.rs` (аккумуляция истории, headless-рендер всех вариантов `OutputItemKind`, золотые TSV-заголовки, навигация Outline)
- [x] Нормативное закрепление правил и инвариантов сохранения функционала в генеральном регламенте `AGENTS.md`
- [x] 100% прохождение тестов воркспейса с `-j 1`, чистота `clippy -D warnings`, `cargo fmt --check`, `file-guard`, `layers`, `parity` и сборка чистого релиза

## 20. Стандартизация дизайн-системы диалоговых окон и защита от халтуры (DialogKit & Design Guard)
- [x] Разработка официального канонического стандарта дизайна окон `docs/DESIGN_GUIDE.md` (архетипы 1–4, сетка, отступы, геометрия кнопок, правило i18n Shield)
- [x] Создание ядра переиспользуемых типизированных компонентов `DialogKit` (`crates/statcraft_app/src/dialogs/common/`):
  - `DialogShell`: единая обертка для окон модального типа (`Standard` 560x430, `Subdialog` 360x300, `Properties` 460x380, `Compute` 620x480)
  - `TransferButton`: векторная адаптивная кнопка переноса (`->` / `<-`) со скруглением 3 px и автоматической ориентацией
  - `VariablePicker`: двухпанельный список переменных с векторными иконками шкал и отображением `Метка [Имя]`
  - `DialogActionBar`: нижний ряд кнопок Type 1 (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`) с защитой i18n Shield и единой шириной
  - `SubdialogActionBar`: нижняя панель для поддиалогов (`[ Continue ] [ Cancel ] [ Help ]`)
  - `SubdialogSidebar`: правая вертикальная колонка поддиалогов SPSS (`[ Statistics... ]`, `[ Options... ]`, 90 × 24 px)
  - `GroupBox`: визуальная группировка чекбоксов, радиокнопок и полей ввода
- [x] Инструментальный рубеж: статический линтер дизайна `cargo run -p xtask -j 1 -- design-guard` (запрет сырых `ui.button` для системных действий и произвольных размеров окон)
- [x] Тестовый рубеж: пакет Headless UI-тестов `crates/statcraft_app/tests/dialog_invariants.rs` (проверка канонических размеров, headless-рендеринг, can_run гейтинг, сброс к дефолтам)
- [x] Полная унификация всех существующих диалогов приложения:
  - `frequencies_dialog.rs` (3-колоночная сетка, SubdialogSidebar, DialogActionBar)
  - `descriptives_dialog.rs` (вынос опций в модальный поддиалог `Descriptives: Options`, DialogActionBar)
  - `anova_oneway_dialog.rs` (вынос опций в модальный поддиалог `One-Way ANOVA: Options`, DialogActionBar)
  - `ttest_onesample_dialog.rs` (вынос % ДИ в модальный поддиалог, DialogActionBar)
  - `ttest_indep_dialog.rs` и `ttest_indep_define_groups.rs` (SubdialogSidebar, SubdialogActionBar, DialogActionBar)
  - `ttest_paired_dialog.rs` (SubdialogSidebar, DialogActionBar)
  - `missing_values_dialog.rs` (DialogShell::properties, GroupBox, DialogActionBar)
  - `value_labels_dialog.rs` (DialogShell::properties, GroupBox, DialogActionBar)
  - `variable_type_dialog.rs` (DialogShell::properties, DialogActionBar)
- [x] Нормативное закрепление требований DialogKit в Разделе 13 `AGENTS.md` и `.agents/rules/AGENTS.md`
- [x] 100% прохождение всех тестов воркспейса с `-j 1` (93 теста), `design-guard`, `file-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`

## 21. Этап 7.1: Таблицы сопряжённости (Crosstabs) и критерий независимости Хи-квадрат Пирсона ($\chi^2$)
- [x] L2 `statcraft_algo`: модули `crosstabs.rs` и `crosstabs_types.rs` — расчет таблицы сопряженности $R \times C$, наблюдаемых и ожидаемых частот ($O_{ij}, E_{ij}$), остатков и процентов (по строкам, столбцам, от общего итога)
- [x] L2 `statcraft_algo`: расчет критерия Хи-квадрат Пирсона ($\chi^2$), отношения правдоподобия (Likelihood Ratio $G^2$), точного $p$-value через верхнюю регуляризованную гамма-функцию $Q(a, x)$, поправки Йейтса на непрерывность и двустороннего точного критерия Фишера для таблиц $2 \times 2$, а также симметричных мер связи ($\phi$ и $V$ Крамера) на 100% safe Rust без паник
- [x] L2 `statcraft_algo`: модуль тестов `crosstabs_tests.rs` — сверка Golden Oracle с эталоном SciPy (`chi2_contingency`, `fisher_exact`) с точностью $\le 10^{-6}$
- [x] L5 `statcraft_syntax`: команда `CrosstabsCommand` (`stat.crosstabs`) со спецификацией параметров `specs()` и генерацией канонического синтаксиса `CROSSTABS /TABLES=rows BY cols /STATISTICS=CHISQ PHI /CELLS=COUNT EXPECTED ROW COLUMN TOTAL RESID.`
- [x] L6 `statcraft_app`: диалог `CrosstabsDialog` ($560 \times 430$ px) на базе `DialogKit` (`DialogShell::standard`, `TransferButton`, `SubdialogSidebar`, `SubdialogActionBar`, `DialogActionBar`, `GroupBox`), модальные поддиалоги `Cells...` и `Statistics...`
- [x] L6 `statcraft_app`: архитектурный вынос обработчиков модальных окон в `app_dialogs.rs`, снизивший объем координатора `app.rs` с 1485 до 1138 строк (с запасом до лимита в 1500 строк)
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_crosstabs.rs` — Case Processing Summary, аутентичная матрица сопряженности Crosstabulation, таблица критериев Chi-Square Tests и симметричных мер связи, TSV-экспорт и интеграция в Outline
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Descriptive Statistics -> Crosstabs...`
- [x] Полная двуязычная локализация (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение всех тестов воркспейса с `-j 1` (96 тестов), `design-guard`, `file-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`

## 22. Этап 7.2: Парные корреляции (Bivariate Correlations: Pearson $r$, Spearman $\rho$, Kendall $\tau$-b)
- [x] L2 `statcraft_algo`: модуль `correlations.rs` и `correlations_types.rs` — линейная корреляция Пирсона ($r$), ранговая корреляция Спирмена ($\rho$) с дробным ранжированием, ранговая корреляция Кендалла ($\tau$-b) с поправкой на связки, суммы квадратов отклонений и ковариации
- [x] L2 `statcraft_algo`: точные $p$-value для двусторонних (2-tailed) и односторонних (1-tailed) критериев значимости, маркировка уровней значимости звёздочками (`*` при $p < 0.05$, `**` при $p < 0.01$), попарное (Pairwise) и полное (Listwise) исключение пропусков
- [x] L2 `statcraft_algo`: модуль тестов `correlations_tests.rs` — сверка Golden Oracle с эталоном SciPy (`pearsonr`, `spearmanr`, `kendalltau`) с точностью $\le 10^{-6}$
- [x] L5 `statcraft_syntax`: команда `CorrelationsCommand` (`stat.correlations`) со спецификацией `specs()` и генерацией канонического синтаксиса `CORRELATIONS /VARIABLES=... /PRINT=TWOTAIL SIG /STATISTICS DESCRIPTIVES XPROD /MISSING=PAIRWISE.` и `NONPAR CORR`
- [x] L6 `statcraft_app`: диалог `CorrelationsDialog` ($560 \times 430$ px) на базе `DialogKit` (`DialogShell::standard`, `TransferButton`, `SubdialogSidebar`, `SubdialogActionBar`, `DialogActionBar`, `GroupBox`), модальный поддиалог `Options...` (Descriptives, Cross-product deviations and covariances, Pairwise/Listwise missing)
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_correlations.rs` — Descriptive Statistics, аутентичные корреляционные матрицы $K \times K$ со звездочками, уровнями значимости, $N$, ковариациями и сносками значимости, TSV-экспорт и интеграция в Outline Pane
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Correlate -> Bivariate...`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (100 тестов воркспейса, `file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 23. Этап 7.3: Непараметрические критерии для двух выборок (Mann-Whitney $U$ и Wilcoxon $W$)
- [x] L2 `statcraft_algo`: модули `nonparam_twosample.rs` и `nonparam_twosample_types.rs` — объединенное ранжирование с дробными средними рангами (mid-ranks), поправка на связки $\sum(t_i^3 - t_i)$, расчет сумм и средних рангов, статистик $U$ Манна-Уитни, $W$ Вилкоксона, математического ожидания, скорректированной дисперсии, $Z$-оценки и двустороннего асимптотического $p$-value через регуляризованную неполную гамма-функцию
- [x] L2 `statcraft_algo`: модуль `nonparam_twosample.rs` — критерий знаковых рангов Вилкоксона для связанных выборок с ранжированием $|d_i|$, исключением нулевых связок и оценкой $Z$
- [x] L2 `statcraft_algo`: модуль тестов `nonparam_twosample_tests.rs` — сверка Golden Oracle с эталоном SciPy (`mannwhitneyu`, `wilcoxon`) и точными SPSS-бенчмарками
- [x] L5 `statcraft_syntax`: команда `MannWhitneyCommand` (`stat.npar.mann_whitney`) со спецификацией `specs()` и генерацией канонического синтаксиса `NPAR TESTS /M-W= vars BY grp(1 2) /MISSING=ANALYSIS.`
- [x] L6 `statcraft_app`: диалог `NonparamTwoSampleDialog` ($560 \times 430$ px) на базе `DialogKit` (`DialogShell::standard`, `TransferButton`, `SubdialogSidebar`, `SubdialogActionBar`, `DialogActionBar`, `GroupBox`), поддиалог `Define Groups...`
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_nonparam.rs` — аутентичные таблицы SPSS `Ranks` ($N$, Mean Rank, Sum of Ranks) и `Test Statistics` (Mann-Whitney U, Wilcoxon W, Z, Asymp. Sig. 2-tailed), сноска группирующей переменной, TSV-экспорт и интеграция в Outline Pane
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Nonparametric Tests -> Two Independent Samples...`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (105 тестов воркспейса, `file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 24. Этап 7.4: Непараметрические критерии для двух связанных выборок (Wilcoxon Signed-Rank Test и Sign Test)
- [x] L2 `statcraft_algo`: модули `nonparam_related.rs` и `nonparam_related_types.rs` — расчет разностей $D_i = Y_i - X_i$, учет нулевых разностей как связок (Ties), ранжирование $|D_i|$ с вычислением дробных средних рангов (mid-ranks), расчет сумм и средних отрицательных и положительных рангов
- [x] L2 `statcraft_algo`: дисперсия с поправкой на связки $\sigma_W^2 = \frac{n(n+1)(2n+1)}{24} - \frac{\sum(t_j^3 - t_j)}{48}$, $Z$-оценка с непрерывной поправкой $\pm 0.5$, двусторонний асимптотический $p$-value через неполную гамма-функцию
- [x] L2 `statcraft_algo`: критерий знаков (Sign Test) — подсчет отрицательных, положительных и нулевых разностей, точный двусторонний биномиальный критерий значимости через регуляризованную неполную бета-функцию $I_{0.5}(n - k, k + 1)$
- [x] L2 `statcraft_algo`: модуль тестов `nonparam_related_tests.rs` — сверка Golden Oracle с эталонами SciPy и точными бенчмарками SPSS
- [x] L5 `statcraft_syntax`: команда `RelatedSamplesTestCommand` (`stat.npar.related2`) со спецификацией `specs()` и генерацией канонического синтаксиса `NPAR TESTS /WILCOXON=... /SIGN=... /MISSING=ANALYSIS.`
- [x] L6 `statcraft_app`: диалог `NonparamRelatedDialog` ($560 \times 430$ px) на базе `DialogKit` (`DialogShell::standard`, `VariablePicker`, `TransferButton`, `DialogActionBar`, `GroupBox`), выбор `Variable 1` и `Variable 2`, формирование списка `Test Pairs`, чекбоксы `[x] Wilcoxon`, `[ ] Sign`
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_nonparam_related.rs` — аутентичные таблицы SPSS `Ranks` (Negative, Positive, Ties, Total), `Test Statistics` (Z, Asymp. Sig. 2-tailed, сноски направления рангов), `Frequencies` (Sign Test) и `Test Statistics` (Exact Sig. 2-tailed), экспорт в TSV и Outline Pane
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Nonparametric Tests -> Related Samples (Wilcoxon / Friedman)...`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)


## 25. Исправление навигационной панели (Outline Overflow Fix) и Этап 7.5: Критерий Крускала-Уоллиса для K независимых выборок (Kruskal-Wallis H Test)
- [x] L6 `statcraft_app`: исправление выезда текста в Outline View (`output_view/mod.rs` и `outline.rs`) — расширение панели до 230 px, установка жесткого рубежа отсечения `ui.set_clip_rect(left_rect)`, посимвольная утилита усечения `truncate_text_to_width` с многоточием `...`, отрисовка внутри `painter.with_clip_rect` и всплывающие подсказки полного заголовка `.on_hover_text(...)`
- [x] L2 `statcraft_algo`: модули `kruskal_wallis.rs` и `kruskal_wallis_tests.rs` — расчет сумм и средних рангов по $k$ группам, статистика $H$, поправка на совпадающие ранги (Tie correction) $\sum(t_j^3 - t_j) / (N^3 - N)$, степени свободы $df = k - 1$, точный асимптотический $p$-value через $\chi^2$ распределение (`chi_square_sf`)
- [x] L2 `statcraft_algo`: модуль тестов `kruskal_wallis_tests.rs` — сверка Golden Oracle с эталоном SciPy (`scipy.stats.kruskal`) с точностью $\le 10^{-6}$
- [x] L5 `statcraft_syntax`: команда `KruskalWallisCommand` (`stat.npar.kruskal_wallis`) со спецификацией `specs()` и генерацией канонического синтаксиса `NPAR TESTS /K-W= vars BY grp(min max) /MISSING=ANALYSIS.`
- [x] L6 `statcraft_app`: диалог `KruskalWallisDialog` ($560 \times 430$ px) на базе `DialogKit` (`DialogShell::standard`, `VariablePicker`, `TransferButton`, `SubdialogSidebar`, `SubdialogActionBar`, `DialogActionBar`, `GroupBox`), модальный поддиалог `Define Range...` (Min/Max коды групп)
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_kruskal.rs` — аутентичные таблицы SPSS `Ranks` ($N$, Mean Rank, Total) и `Test Statistics` (Kruskal-Wallis H, df, Asymp. Sig.), сноски критерия и группирующей переменной, TSV-экспорт и интеграция в Outline Pane
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Nonparametric Tests -> K Independent Samples...`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 26. Этап 8.1: Вычисление переменных (Compute Variable: `COMPUTE`)
- [x] L2 `statcraft_algo`: модульный пакет `expr_eval/` (`types.rs`, `lexer.rs`, `parser.rs`, `eval.rs`, `mod.rs`, `tests.rs`) — токенизация, алгоритм сортировочной станции Дейкстры (Shunting-yard) для построения обратной польской записи (RPN), построчный вычислитель с поддержкой арифметических операций (`+`, `-`, `*`, `/`, `**`, `^`, унарные `+`/`-`), скобок и функций одного/многих аргументов (`ABS`, `EXP`, `LN`, `LOG10`, `RND`, `SQRT`, `TRUNC`, `SUM`, `MEAN`, `MIN`, `MAX`)
- [x] L2 `statcraft_algo`: строгая защита **Never Crash** и работа с пропусками — деление на ноль, корень из отрицательного и пропуски в данных возвращают системный пропуск (`NaN` / `SYSMIS`) без паник
- [x] L5 `statcraft_syntax`: типизированная команда `ComputeVariableCommand` (`transform.compute`) с параметрами `specs()`, созданием новой колонки или перезаписью существующей, поддержкой отката Undo (`Ctrl+Z`) и генерацией канонического синтаксиса `COMPUTE target = expression. \n VARIABLE LABELS ... \n EXECUTE.`
- [x] L6 `statcraft_app`: центрированный диалог `ComputeVariableDialog` ($620 \times 480$ px) на базе `DialogKit` (`DialogShell::compute`, `render_variable_list_panel`, `TransferButton`, `DialogActionBar`) с многострочным редактором выражений, виртуальной клавиатурой калькулятора (`7 8 9`, `4 5 6`, `1 2 3`, `0 . C`, операторы) и списком функций с группировкой (*All, Arithmetic, Statistical*)
- [x] L6 `statcraft_app`: активация пункта меню `Transform -> Compute Variable...`, мгновенная реактивность и обновление `Data View` и `Variable View`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 27. Этап 8.2: Перекодирование значений (Recode into Same / Different Variables: `RECODE`)
- [x] L2 `statcraft_algo`: модуль `recode.rs` — сопоставление правил `OldValueRule` (`Exact`, `SysMissing`, `SystemOrUserMissing`, `Range`, `RangeLowest`, `RangeHighest`, `AllOther`) и спецификаций `NewValueSpec` (`Value`, `SysMissing`, `CopyOld`), безопасная обработка пропусков и краевых значений без паник
- [x] L5 `statcraft_syntax`: типизированные команды `RecodeIntoDifferentCommand` (`transform.recode.diff`) и `RecodeIntoSameCommand` (`transform.recode.same`), поддержка `specs()`, полный откат `Undo` (`Ctrl+Z`), генерация канонического синтаксиса `RECODE var (old=new)... INTO target. \n EXECUTE.`
- [x] L6 `statcraft_app`: центрированный диалог `RecodeDialog` ($560 \times 430$ px, `DialogShell::standard`) с поддержкой двух режимов (*Different Variables* и *Same Variables*), сопоставлением `Numeric Variable -> Output Variable:`, блоком свойств `Output Variable` (`Name:`, `Label:`, `Change`)
- [x] L6 `statcraft_app`: модальный поддиалог `RecodeOldNewState` ($530 \times 430$ px, `DialogShell::standard`) с радиокнопками `Old Value` и `New Value`, кнопками `Add` / `Change` / `Remove`, списком сопоставлений `Old -> New:` и нижней панелью `SubdialogActionBar`
- [x] L6 `statcraft_app`: активация пунктов меню `Transform -> Recode into Different Variables...` и `Transform -> Recode into Same Variables...`, реактивное обновление `Data View`, `Variable View` и логирование в `Output Viewer`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci`

## 28. Этап 8.3: Сортировка наблюдений (Sort Cases: `SORT CASES`)
- [x] L0 `statcraft_io` & L1 `statcraft_core`: методы `RawTable::reorder_rows` и `DataSet::reorder_cases` — безопасная перестановка строк всех колонок таблицы по массиву индексов без паник с сохранением словаря переменных и метаданных
- [x] L2 `statcraft_algo`: модуль `sort_cases.rs` — спецификация ключей `SortKeySpec` (`Ascending` / `Descending`), каноническая обработка пропусков по стандарту SPSS (системные пропуски `SYSMIS` / `Null` / `NaN` идут первыми при сортировке по возрастанию и последними при сортировке по убыванию), многоуровневая стабильная сортировка и утилита `invert_permutation`
- [x] L5 `statcraft_syntax`: типизированная команда `SortCasesCommand` (`data.sort_cases`) со спецификацией параметров `specs()`, полным стеком отката `Undo` (`Ctrl+Z`) через обратную перестановку индексов и генерацией канонического синтаксиса `SORT CASES BY var1 (A) var2 (D).`
- [x] L6 `statcraft_app`: центрированный диалог `SortCasesDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`render_variable_list_panel`, `TransferButton`, `GroupBox`, `DialogActionBar`), выбор ключей, кнопки изменения порядка `Move Up` / `Move Down`, радиокнопки направления `(o) Ascending` / `( ) Descending`
- [x] L6 `statcraft_app`: активация пункта меню `Data -> Sort Cases...`, мгновенная реактивность переупорядочивания грида в `Data View`, логирование синтаксиса в `Output Viewer` и статусное сообщение в строке состояния
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci`

## 29. Этап 8.4: Выборка наблюдений и фильтрация (Select Cases: `FILTER BY` / `SELECT IF`)
- [x] L1 `statcraft_core`: поддержка фильтрации в `DataSet` (`active_filter_var`, `set_filter_variable`, `is_row_active`, `is_row_filtered`, `active_row_indices`), автоматическая синхронизация при переименовании и удалении переменных
- [x] L2 `statcraft_algo`: расширение `expr_eval` поддержкой операторов сравнения (`=`, `==`, `<>`, `~=`, `!=`, `<`, `<=`, `>`, `>=`, `EQ`, `NE`, `LT`, `LE`, `GT`, `GE`), логических операторов (`&`, `&&`, `|`, `||`, `~`, `!`, `AND`, `OR`, `NOT`) и построчным вычислением логических условий
- [x] L5 `statcraft_syntax`: типизированная команда `SelectCasesCommand` (`data.select_cases`) со спецификацией `specs()`, поддержкой 3 режимов (*All cases / If condition is satisfied / Use filter variable*), действий (*Filtered / Deleted*), генерацией канонического синтаксиса `FILTER BY filter_$.`, `FILTER OFF.`, `SELECT IF (...)` и полным стеком отката `Undo` (`Ctrl+Z`)
- [x] L5 `statcraft_syntax`: автоматический учет активной маски фильтра во всех аналитических процедурах (`descriptives`, `frequencies`, `crosstabs`, `correlations`, `ttest_indep`, `ttest_paired`, `ttest_onesample`, `anova_oneway`, `nonparam_twosample`, `nonparam_related`, `kruskal_wallis`) через `ds.active_row_indices()`
- [x] L6 `statcraft_app`: центрированный диалог `SelectCasesDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`VariablePicker`, `TransferButton`, `GroupBox`, `DialogActionBar`), модальный поддиалог условия `[ If... ]` с виртуальной клавиатурой операторов и `SubdialogActionBar`
- [x] L6 `statcraft_app`: визуальное перечеркивание номеров исключенных наблюдений векторной чертой в `Data View` и индикатор `Filter: ON (var)` в строке состояния (Status Bar)
- [x] L6 `statcraft_app`: активация пункта меню `Data -> Select Cases...`, журналирование синтаксиса в `Output Viewer`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (140 файлов, 0 предупреждений Clippy, все тесты пройдены)

## 30. Этап 8.5: Расщепление файла на подгруппы (Split File: `SPLIT FILE`)
- [x] L1 `statcraft_core`: поддержка расщепления файла в `DataSet` (`split_file: Option<SplitFileConfig>`, `set_split_file`, `split_file`, `split_groups`), типы `SplitFileMode` (*Off, CompareGroups, OrganizeOutput*), `SplitKeyValue`, `SplitSubgroup`, автоматическая синхронизация при переименовании и удалении переменных
- [x] L2 `statcraft_algo`: модуль `split_groups.rs` — извлечение подгрупп наблюдений по набору группирующих переменных (`extract_split_groups`), поддержка составных ключей (multi-variable grouping), системных пропусков `SYSMIS` / `Null` / `NaN` и бесшовный учет активного фильтра `Select Cases` (`active_filter_var`)
- [x] L5 `statcraft_syntax`: типизированная команда `SplitFileCommand` (`data.split_file`) со спецификацией параметров `specs()`, поддержкой автоматической сортировки (`sort: bool`), генерацией канонического синтаксиса `SPLIT FILE OFF.`, `SPLIT FILE LAYERED BY var1 var2.`, `SPLIT FILE SEPARATE BY var1 var2.` и полным стеком отката `Undo` (`Ctrl+Z`) с восстановлением предыдущей конфигурации и перестановки строк
- [x] L6 `statcraft_app`: центрированный диалог `SplitFileDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`VariablePicker`, `TransferButton`, `GroupBox`, `DialogActionBar`), выбор режимов (*Analyze all cases / Compare groups / Organize output by groups*), опции сортировки (*Sort the file by grouping variables / File is already sorted*)
- [x] L6 `statcraft_app`: архитектурная декомпозиция — выделение модуля `app_dialogs_data.rs` для сохранения жесткого лимита `file-guard` $\le 1000$ строк на файл в координаторах диалогов
- [x] L6 `statcraft_app`: векторный индикатор состояния `Split File: ON (var)` в строке состояния (Status Bar), активация пункта меню `Data -> Split File...`, логирование в `Output Viewer`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 31. Этап 8.6: Взвешивание наблюдений (Weight Cases: `WEIGHT BY`)
- [x] L1 `statcraft_core`: поддержка переменной взвешивания в `DataSet` (`weight_var: Option<String>`, `set_weight_variable`, `weight_var`, `is_weighted`, `row_weight`), канонические правила SPSS ($\ge 0$, исключение пропусков и отрицательных весов, учет активного фильтра строк), автоматическая синхронизация при переименовании и удалении переменных
- [x] L2 `statcraft_algo`: поддержка весов в процедурах частотного анализа (`frequencies.rs`: взвешенные частоты, проценты, кумулятивные проценты, взвешенные квартили), описательных статистиках (`descriptives.rs`: `compute_descriptives_weighted` — взвешенное среднее, несмещенная дисперсия, стандартное отклонение, стандартная ошибка среднего, асимметрия, эксцесс) и таблицах сопряженности (`crosstabs.rs`: накопление весов в ячейках)
- [x] L5 `statcraft_syntax`: типизированная команда `WeightCasesCommand` (`data.weight_cases`), спецификация `specs()`, канонический синтаксис `WEIGHT BY var.` и `WEIGHT OFF.`, полный стек отката `Undo/Redo` (`Ctrl+Z`), проброс `ds.row_weight(r)` во все аналитические процедуры
- [x] L6 `statcraft_app`: центрированный диалог `WeightCasesDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`VariablePicker`, `TransferButton`, `GroupBox`, `DialogActionBar`), режимы *Do not weight cases* и *Weight cases by*, отображение текущего статуса
- [x] L6 `statcraft_app`: индикатор состояния `Weight: ON (var)` в строке состояния (Status Bar), активация пункта меню `Data -> Weight Cases...`, журналирование синтаксиса в `Output Viewer`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 32. Этап 9.1: Линейная регрессия (Linear Regression: `REGRESSION`)
- [x] L2 `statcraft_algo`: модули `regression.rs`, `regression_types.rs` и `regression_tests.rs` — МНК (OLS) и взвешенный МНК (WLS) через метод Гаусса-Жордана с частичным выбором главного элемента, обращение матрицы $A = X^T W X$ на 100% safe Rust без паник
- [x] L2 `statcraft_algo`: расчет коэффициента множественной корреляции ($R$), коэффициента детерминации ($R^2$), скорректированного $R^2$, стандартной ошибки оценки, ANOVA (Regression, Residual, Total sums of squares, df, Mean Squares, $F$-критерий, $p$-value), коэффициентов ($B$, Std. Error, $\beta$, $t$, Sig, 95% CI) и диагностик коллинеарности (Tolerance и VIF)
- [x] L2 `statcraft_algo`: сверка Golden Oracle с эталоном SciPy / NumPy с точностью $\le 10^{-6}$ (множественная и взвешенная линейная регрессия)
- [x] L5 `statcraft_syntax`: типизированная команда `LinearRegressionCommand` (`stat.regression.linear`) со спецификацией `specs()`, фильтрацией `active_row_indices()`, весами `row_weight(r)` и генерацией канонического синтаксиса `REGRESSION /MISSING LISTWISE /STATISTICS COEFF OUTS R ANOVA COLLIN /CRITERIA=PIN(.05) POUT(.10) /NOORIGIN /DEPENDENT ... /METHOD=ENTER ...`
- [x] L6 `statcraft_app`: диалог `LinearRegressionDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`VariablePicker`, `TransferButton`, `SubdialogSidebar`, `SubdialogActionBar`, `DialogActionBar`, `GroupBox`), модальный поддиалог `[ Statistics... ]`
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_regression.rs` — 4 аутентичные таблицы SPSS (*Variables Entered/Removed, Model Summary, ANOVA, Coefficients*), TSV-экспорт для буфера обмена и интеграция в Outline Tree
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Regression -> Linear...`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 33. Этап 10.1: Анализ надежности шкалы (Reliability Analysis: `RELIABILITY`)
- [x] L2 `statcraft_algo`: модули `reliability.rs`, `reliability_types.rs` и `reliability_tests.rs` — расчет коэффициента надежности $\alpha$ Кронбаха и $\alpha$ на основе стандартизованных пунктов, построчное исключение пропущенных данных (Listwise deletion), поддержка весов наблюдений на 100% safe Rust без паник
- [x] L2 `statcraft_algo`: расчет статистик пунктов (*Item Statistics*: Mean, Std. Dev, $N$) и характеристик при удалении пунктов (*Item-Total Statistics*: Scale Mean if Item Deleted, Scale Variance if Item Deleted, Corrected Item-Total Correlation, Cronbach's Alpha if Item Deleted), а также описательных статистик шкалы (*Scale Statistics*)
- [x] L2 `statcraft_algo`: сверка Golden Oracle с эталонами Python (`pingouin` / матричные формулы SPSS) с точностью $\le 10^{-6}$, защита Never Crash при $k < 2$, вырожденных нулевых дисперсиях и сингулярных матрицах
- [x] L5 `statcraft_syntax`: типизированная команда `ReliabilityCommand` (`stat.scale.reliability`) со спецификацией параметров `specs()`, фильтрацией `active_row_indices()`, весами `row_weight(r)` и генератором канонического синтаксиса `RELIABILITY /VARIABLES=... /SCALE('ALL VARIABLES') ALL /MODEL=ALPHA /STATISTICS=DESCRIPTIVES SCALE /SUMMARY=TOTAL.`
- [x] L6 `statcraft_app`: модальный диалог `ReliabilityDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`VariablePicker`, `TransferButton`, `SubdialogSidebar`, `SubdialogActionBar`, `DialogActionBar`, `GroupBox`), модальный поддиалог `[ Statistics... ]`
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_reliability.rs` — 4–5 аутентичных таблиц SPSS (*Case Processing Summary, Reliability Statistics, Item Statistics, Item-Total Statistics, Scale Statistics*), TSV-экспорт для буфера обмена и интеграция в Outline Tree
- [x] L6 `statcraft_app`: активация пункта главного меню `Analyze -> Scale -> Reliability Analysis...` (`Анализ -> Шкала -> Анализ надежности...`)
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 34. Этап 11.1: Факторный анализ / Метод главных компонент (Factor Analysis / PCA: `FACTOR`)
- [x] L2 `statcraft_algo`: модули `factor.rs`, `factor_types.rs` и `factor_tests.rs` — спектральное разложение симметричной матрицы корреляций через циклический метод Якоби (`jacobi_eigen_symmetric`), извлечение главных компонент (PCA), правило Кайзера ($\lambda \ge 1.0$) и отбор заданного числа факторов на 100% safe Rust без паник
- [x] L2 `statcraft_algo`: расчет мер применимости факторизации — Kaiser-Meyer-Olkin (KMO) Measure of Sampling Adequacy через обращение матрицы корреляций и антиобразные частные корреляции, критерий сферичности Бартлетта (Bartlett's Test of Sphericity: Approx. Chi-Square, df, Sig через неполную гамма-функцию)
- [x] L2 `statcraft_algo`: расчет общностей (*Communalities*: Initial $1.0$, Extraction), объясненной дисперсии (*Total Variance Explained*: Initial Eigenvalues, % of Variance, Cumulative %, Extraction Sums of Squared Loadings) и матрицы факторных нагрузок (*Component Matrix*)
- [x] L2 `statcraft_algo`: сверка Golden Oracle с эталонами Python (`scipy.linalg.eigh`, `factor_analyzer`), инварианты Never Crash при нулевых дисперсиях, малом $n < 2$ и сингулярных матрицах
- [x] L5 `statcraft_syntax`: типизированная команда `FactorCommand` (`stat.factor`) со спецификацией `specs()`, фильтрацией `active_row_indices()`, весами `row_weight(r)` и генератором канонического синтаксиса `FACTOR /VARIABLES ... /MISSING LISTWISE /ANALYSIS ... /PRINT INITIAL EXTRACTION KMO /CRITERIA MINEIGEN(1) /EXTRACTION PC /ROTATION NOROTATE.`
- [x] L6 `statcraft_app`: модальный диалог `FactorDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`VariablePicker`, `TransferButton`, `SubdialogSidebar`, `SubdialogActionBar`, `DialogActionBar`, `GroupBox`), модальные поддиалоги `[ Descriptives... ]` (KMO и Бартлетт) и `[ Extraction... ]` (PCA, собственные значения > 1.0 / число факторов)
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_factor.rs` — 4 аутентичные таблицы SPSS (*KMO and Bartlett's Test, Communalities, Total Variance Explained, Component Matrix*), TSV-экспорт для буфера обмена и интеграция в Outline Tree
- [x] L6 `statcraft_app`: активация пункта главного меню `Analyze -> Dimension Reduction -> Factor...` (`Анализ -> Снижение размерности -> Факторный анализ...`)
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 35. Этап 12.1: Сравнение средних по подгруппам (Means Breakdown: `MEANS`)
- [x] L2 `statcraft_algo`: модули `means.rs`, `means_types.rs` и `means_tests.rs` — расчет описательных статистик ячеек (Mean, Count, Std. Deviation, Variance, Median, Min, Max, Sum) в разрезе категориальных факторов с поддержкой весов `row_weight(r)` и числовой/лексической сортировкой групп
- [x] L2 `statcraft_algo`: расчет таблицы однофакторного дисперсионного анализа (ANOVA: Between Groups, Within Groups, Total Sum of Squares, df, Mean Square, F-критерий и p-значение) и мер связи $\eta$ (Eta) и $\eta^2$ (Eta Squared)
- [x] L2 `statcraft_algo`: сверка Golden Oracle с эталонами SciPy / SPSS с точностью $\le 10^{-6}$, строгая защита Never Crash при пустых группах, $n=1$ и нулевых дисперсиях
- [x] L5 `statcraft_syntax`: типизированная команда `MeansCommand` (`stat.means`) со спецификацией параметров `specs()`, фильтрацией `active_row_indices()`, весами `row_weight(r)` и генератором канонического синтаксиса `MEANS TABLES=... BY ... /CELLS=MEAN COUNT STDDEV MEDIAN /STATISTICS ANOVA.`
- [x] L6 `statcraft_app`: центрированный диалог `MeansDialog` ($560 \times 430$ px, `DialogShell::standard`) на базе `DialogKit` (`VariablePicker`, две целевые панели `Dependent List:` и `Layer 1 of 1 / Independent List:`, адаптивные `TransferButton`, `SubdialogSidebar`, модальный поддиалог `[ Options... ]`, `SubdialogActionBar`, `DialogActionBar`)
- [x] L6 `statcraft_app`: модуль вывода `output_view/tables_means.rs` — аутентичные таблицы SPSS (*Case Processing Summary, Report, ANOVA Table, Measures of Association*), TSV-экспорт для буфера обмена и интеграция в Outline Tree
- [x] L6 `statcraft_app`: активация пункта меню `Analyze -> Compare Means -> Means...` (`Анализ -> Сравнение средних -> Средние...`)
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 36. Этап MP-1: Жизненный цикл сессии и навигация меню «Файл» (Session Lifecycle & File Menu)
- [x] L6 `statcraft_app`: перевод пунктов главного меню `File -> New -> Data` (`menu-file-new-dataset`), `File -> New -> Syntax` (`menu-file-new-syntax`), `File -> New -> Project` (`menu-file-new-project`), `File -> Close Project` (`menu-file-close-project`) и `File -> Recent Files` (`menu-file-recent`) из `MenuItemStatus::InDev` в полноценно рабочий `MenuItemStatus::Ready` / динамический список
- [x] L6 `statcraft_app`: модуль `crates/statcraft_app/src/recent_files.rs` (`RecentFiles`, ротация до 10 файлов, дедупликация с подъемом наверх, очистка, сохранение в `.statcraft/recent_files.json` и `eframe::Storage`)
- [x] L6 `statcraft_app`: модуль `crates/statcraft_app/src/app_session.rs` (`PendingSessionAction`, управление жизненным циклом сессии, детекция несохраненных изменений `has_unsaved_changes()`, `load_file_from_path`)
- [x] L6 `statcraft_app`: интерактивный редактор синтаксиса `crates/statcraft_app/src/views/syntax_editor.rs` (`SyntaxEditorState`, `render_syntax_editor`, канонический стартовый заголовок `* StatCraft Syntax Editor.`, интеграция вкладки в DockState и пункт `Window -> Syntax Editor (Ctrl+4)`)
- [x] L6 `statcraft_app`: модальный диалог подтверждения `crates/statcraft_app/src/dialogs/confirm_save_dialog.rs` (`DialogShell::subdialog`, `[ Save ] [ Don't Save ] [ Cancel ]`)
- [x] L6 `statcraft_app`: логирование `NEW DATASET.` в Output Viewer и уведомления в Status Bar (`status-new-dataset-created`, `status-new-syntax-created`, `status-new-project-created`)
- [x] Тесты жизненного цикла: `crates/statcraft_app/tests/session_lifecycle_tests.rs` (очередь recent files, сброс датасета, синтаксис, новый проект, закрытие проекта, проверка несохраненных изменений, headless-рендер меню)
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)


## 38. Этап MP-4: Импорт файлов Microsoft Excel (.xlsx / .xls / .ods)
- [x] L0 `statcraft_io`: интеграция крейта `calamine = "0.37"`, модуль `crates/statcraft_io/src/excel_loader.rs` (`load_excel_from_path`, `load_excel_from_reader`, `load_excel_from_range`)
- [x] L0 `statcraft_io`: санитизация и дедупликация имен колонок по каноническим правилам SPSS (`sanitize_and_dedup_headers`), автовывод типов колонок (`Numeric` vs `Text`), безопасная обработка пустых листов, пропусков и смешанных типов
- [x] L5 `statcraft_syntax`: типизированная команда `ImportExcelCommand` (`data.import_excel`) со спецификацией параметров `specs()`, выполнением и поддержкой Undo
- [x] L6 `statcraft_app`: метод `open_excel_dialog` с фильтрами `*.xlsx, *.xls, *.ods, *.xlsm`, активация пункта меню `File -> Import Data -> Excel...` (`menu-file-import-excel`) и `File -> Import Data -> CSV...` (`menu-file-import-csv`)
- [x] L6 `statcraft_app`: расширение `load_file_from_path` с определением формата по расширению, добавлением в `RecentFiles`, протоколированием в Output Viewer канонического синтаксиса `GET DATA /TYPE=XLSX /FILE='...' /FIRSTROW=YES.`
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)


## 39. Этап MP-5: Импорт бинарных файлов SPSS (.sav / .zsav)
- [x] L0 `statcraft_io`: подмодуль `spss_loader/` (`header.rs`, `records.rs`, `decompress.rs`, `mod.rs`, `tests.rs` — полное чтение системных заголовков `$FL2`/`$FL3`, детекция endianness, чтение переменных Record 2, меток значений Record 3/4, расширений Record 7 (подтипы 4 display params, 13 long variable names, 20 encoding, 24 zlib metadata), распаковка простых байт-кодов с bias=100 и ZLIB-блоков `.zsav` через `flate2`)
- [x] L1 `statcraft_core`: модуль `spss.rs` (`DataSet::from_spss_data`, `DataSet::load_spss_from_path`, `DataSet::load_spss_from_reader` с конвертацией форматов в 9 типов переменных SPSS, шкал Measure, выравнивания Alignment, словарей меток `ValueLabels`, пропусков `MissingValues` и переменной весов `weight_var`)
- [x] L5 `statcraft_syntax`: типизированная команда `ImportSpssCommand` (`data.import_spss`) со спецификацией `specs()`, регистрацией в `CommandBus::with_defaults()`, генератором канонического синтаксиса `GET FILE='<path>'.` и поддержка `.sav`/`.zsav` в `OpenFileCommand`
- [x] L6 `statcraft_app`: активация пункта главного меню `File -> Import Data -> SPSS Data...` (`menu-file-import-spss`) и добавление фильтра `.sav, .zsav` в диалог общего открытия данных `File -> Open Data...` (`open_data_dialog`)
- [x] L6 `statcraft_app`: обработка импорта в `app_session.rs` с записью протокола `GET FILE='<path>'.` в Output Viewer, обновление статуса `DatasetImported`, добавление в `RecentFiles` и переключение на вкладку `Data View`
- [x] Интеграционные тесты: модуль `spss_loader/tests.rs` в `statcraft_io` (синтетический буфер `.sav` с байт-кодами, несжатый `.sav`, обработка поврежденных/усеченных заголовков без паник) и тест в `crates/statcraft_app/tests/session_lifecycle_tests.rs` (`test_load_spss_sav_session_lifecycle`)
- [x] 100% покрытие локализацией (`locales/en.ftl`, `locales/ru.ftl`), 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 40. Финал: Интеграция меню «Файл», диалог сохранения и горячие клавиши (File Menu Final Integration)
- [x] L6 `statcraft_app`: перехват закрытия окна `ctx.input(|i| i.viewport().close_requested())` при модифицированном датасете (`has_unsaved_changes()`), отмена закрытия через `ViewportCommand::CancelClose` и отображение `ConfirmSaveDialog` (`[ Save ] [ Don't Save ] [ Cancel ]`)
- [x] L6 `statcraft_app`: метод `handle_confirm_save_dialog` в `crates/statcraft_app/src/app_session.rs` для сохранения при необходимости (`save_project` / `save_project_as`) и последующего исполнения отложенного действия `pending_session_action`
- [x] L6 `statcraft_app`: глобальные клавиатурные сокращения (`Ctrl+Shift+N` — Новый проект, `Ctrl+N` — Новый датасет, `Ctrl+O` — Открыть данные, `Ctrl+Shift+O` — Открыть проект, `Ctrl+S` — Сохранить, `Ctrl+Shift+S` — Сохранить как)
- [x] L6 `statcraft_app`: тест инвариантов диалога `test_headless_render_confirm_save_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] Репозиторий: добавление правил игнорирования `/.statcraft` и `crates/statcraft_app/.statcraft` в `.gitignore`
- [x] 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (130+ тестов, `fmt`, `clippy -D warnings`, `file-guard`, `design-guard`, `layers`, `parity`)

## 41. Сохранение и восстановление полного состояния проекта (.stc)
- [x] L0 `statcraft_io`: расширение структуры `StcContainer` опциональными полями `output_viewer_json` и `syntax_text` с `#[serde(default, skip_serializing_if = "Option::is_none")]`, метод `StcContainer::with_project_data`
- [x] L1 `statcraft_core`: методы `project_to_stc`, `save_project_stc_to_path` и `load_project_stc_from_path` с извлечением и сохранением полного состояния аналитического пространства
- [x] L6 `statcraft_app`: безопасная сериализация/десериализация процедур и элементов `OutputViewerState` (`static_str_serde`, `to_json()`, `load_from_json()` с автоматическим пересчетом `next_id`)
- [x] L6 `statcraft_app`: интеграция в жизненный цикл сессии `app_session.rs` (`save_to_path` сериализует правую панель и текст синтаксиса, `load_file_from_path` восстанавливает 100% состояния)
- [x] L6 `statcraft_app`: разграничение диалогов сохранения — `Save Project` (`Ctrl+S`) / `Save Project As...` сохраняют монолитный `.stc`, а `Save Data As...` сохраняет исключительно массив данных в `.csv` (фильтр `.stc` исключен)
- [x] Интеграционные тесты: сквозной тест `test_save_and_load_full_project_with_output_and_syntax` в `session_lifecycle_tests.rs`, тесты roundtrip в `statcraft_io` и `statcraft_core`, тесты сериализации в `OutputViewerState`
- [x] 100% прохождение полного CI gate `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`)

## 42. Процедура сдвига значений (Lead / Lag) — `transform.shift_values`
- [x] L2 `statcraft_algo`: алгоритмическая основа сдвига `compute_shift_raw` и перечисление `ShiftDirection` (`Lag`, `Lead`) в `crates/statcraft_algo/src/shift.rs` с поддержкой шага `step`, граничных случаев и пропусков
- [x] L5 `statcraft_syntax`: типизированная команда движка `ShiftValuesCommand` (`transform.shift_values`) с регистрацией в `CommandBus`, генерацией синтаксиса `SHIFT VALUES /VARIABLE=... /RESULT=... /LAG=... /LEAD=...` и полным откатом Undo/Redo
- [x] Локализация: декларативные ключи диалога и сообщений журнала в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `ShiftValuesDialog` по стандарту `DialogKit` (`DialogShell::standard`, `VariableListItem`, `TransferButton`, `GroupBox`, `DialogActionBar`)
- [x] L6 `statcraft_app`: интеграция диалога в меню «Преобразование» (снята метка `[dev]`), привязка к CommandBus и логирование в Output Viewer
- [x] Тесты инвариантов: headless-тест `test_headless_render_shift_values_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 43. Подсчет значений в строках (Count Occurrences of Values within Cases: `COUNT`) — `transform.count_values`
- [x] L2 `statcraft_algo`: модуль `count_values.rs` — тип критерия `ValueCriterion` (`ExactNumber`, `ExactString`, `Range`, `LowestThru`, `ThruHighest`, `SystemMissing`), метод `matches()` и вычисление `compute_count_across_columns`
- [x] L5 `statcraft_syntax`: типизированная команда движка `CountValuesCommand` (`transform.count_values`) со спецификацией `specs()`, созданием или перезаписью числовой переменной с 0 десятичных знаков, шкалой `Scale`, поддержкой Undo/Redo и генератором канонического синтаксиса `COUNT target = var1 var2 (val1) (val2). \n VARIABLE LABELS ... \n EXECUTE.`
- [x] Локализация: декларативные ключи `dialog-count-*` и `output-count-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `CountValuesDialog` ($560 \times 430$ px) по стандарту `DialogKit` (`DialogShell::standard`, `render_variable_list_panel`, `TransferButton`, `GroupBox`, `DialogActionBar`) и модальный поддиалог `CountValuesDefineState` (`render_count_values_define_dialog`) для определения значений и диапазонов подсчета
- [x] L6 `statcraft_app`: интеграция диалога в меню `Transform -> Count Values within Cases...` (снят статус InDev), подключение в `app_dialogs_data.rs`, статус-сообщение `StatusMessage::CountValuesExecuted` и логирование в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_count_values_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 44. Установка зерна ДСЧ (Set Random Number Seed: `SET SEED`) — `transform.random_seed`
- [x] L1 `statcraft_core`: потокобезопасное глобальное состояние зерна генератора случайных чисел (`set_global_random_seed`, `get_global_random_seed`, `clear_global_random_seed`, `get_effective_random_seed`) с поддержкой режима случайной инициализации и фиксированного зерна
- [x] L5 `statcraft_syntax`: типизированная команда движка `RandomSeedCommand` (`transform.random_seed`) с JSON Schema `specs()`, валидацией диапазона 1 - 2 000 000 000, генерацией синтаксиса `SET SEED = <val>.` / `SET SEED = RANDOM.` и поддержкой Undo/Redo
- [x] Локализация: декларативные ключи `dialog-seed-*` и `status-random-seed-*` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `RandomSeedDialog` по стандарту `DialogKit` (`DialogShell::properties`, `GroupBox`, `DialogActionBar`) с переключением между случайной инициализацией и фиксированным зерном
- [x] L6 `statcraft_app`: интеграция диалога в меню `Transform -> Set Random Number Seed...` (снят статус InDev), подключение в `app_dialogs_data.rs`, статус-сообщение `StatusMessage::RandomSeedExecuted` и логирование синтаксиса в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_random_seed_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 45. Автоматическое перекодирование (Automatic Recode: `AUTORECODE`) — `transform.auto_recode`
- [x] L2 `statcraft_algo`: модуль `crates/statcraft_algo/src/auto_recode.rs` — функция `compute_auto_recode`, перечисление `AutoRecodeOrder` (`LowestValue`, `HighestValue`), сбор уникальных непустых значений, сортировка, последовательное присвоение целочисленных кодов ($1, 2, \dots, K$) и генерация словаря меток `ValueLabels`
- [x] L5 `statcraft_syntax`: типизированная команда движка `AutoRecodeCommand` (`transform.auto_recode`) со спецификацией `specs()`, добавлением новых числовых переменных (`decimals: 0`, `Measure::Nominal`) со словарем меток значений, поддержкой Undo/Redo и генератором канонического синтаксиса `AUTORECODE VARIABLES=... /INTO ... [/DESCENDING] /PRINT. \n EXECUTE.`
- [x] Локализация: декларативные ключи `dialog-autorecode-*` и `output-autorecode-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `AutoRecodeDialog` ($560 \times 430$ px) по стандарту `DialogKit` (`DialogShell::standard`, `render_variable_list_panel`, `TransferButton`, `GroupBox`, `DialogActionBar`, список пар `source -> target`, блок изменения имени и выбор направления ранжирования)
- [x] L6 `statcraft_app`: интеграция диалога в меню `Transform -> Recode -> Automatic Recode...` (снят статус `[dev]`), подключение в `app_dialogs_data.rs`, статус-сообщение `StatusMessage::AutoRecodeExecuted` и логирование в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_auto_recode_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 46. Создание фиктивных переменных (Create Dummy Variables: `SPSSINC CREATE DUMMIES`) — `transform.create_dummies`
- [x] L2 `statcraft_algo`: модуль `crates/statcraft_algo/src/dummies.rs` — функция `generate_dummies_for_column`, структуры `DummySpec`, `GeneratedDummyColumn`, сортировка уникальных категорий, сопоставление меток `ValueLabels`, генерация бинарных колонок (0.0/1.0/Null), пропуск референтной категории при `omit_first == true`
- [x] L5 `statcraft_syntax`: типизированная команда движка `CreateDummiesCommand` (`transform.create_dummies`) со спецификацией `specs()`, добавлением новых числовых переменных (`decimals: 0`, `Measure::Nominal`) со словарем меток, поддержкой полного стека отката `Undo` (`Ctrl+Z`), генератором синтаксиса `SPSSINC CREATE DUMMIES VARIABLES=... /ROOTNAMES=... /OMITFIRST=... \n EXECUTE.`
- [x] Локализация: декларативные ключи `dialog-dummy-*` и `status-dummies-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `DummyDialog` ($560 \times 430$ px) по стандарту `DialogKit` (`DialogShell::standard`, `render_variable_list_panel`, `TransferButton`, `GroupBox`, `DialogActionBar`, список выбранных категориальных переменных, поле ввода базового префикса `root_prefix` и чекбокс исключения референтной группы)
- [x] L6 `statcraft_app`: интеграция диалога в меню `Transform -> Create Dummy Variables...` (снят статус `[dev]`), подключение в `app_dialogs_data.rs`, статус-сообщение `StatusMessage::CreateDummiesExecuted` и логирование синтаксиса в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_dummy_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 47. Замена пропущенных значений (Replace Missing Values: `RMV`) — `transform.replace_missing`
- [x] L2 `statcraft_algo`: модуль `crates/statcraft_algo/src/replace_missing.rs` — функция `impute_missing_values`, методы импутации `ImputationMethod` (`SeriesMean`, `MeanOfNearby`, `MedianOfNearby`, `LinearInterpolation`, `LinearTrendAtPoint`), обработка краевых случаев и пропусков
- [x] L5 `statcraft_syntax`: типизированная команда движка `ReplaceMissingCommand` (`transform.replace_missing`) со спецификацией JSON Schema `specs()`, добавлением или заменой числовых переменных в датасете, поддержкой Undo/Redo и генератором канонического синтаксиса SPSS `RMV new_var = METHOD(src_var, span). \n EXECUTE.`
- [x] Локализация: декларативные ключи `dialog-missing-*`, `output-missing-executed` и `status-missing-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `ReplaceMissingDialog` ($560 \times 430$ px) по стандарту `DialogKit` (`DialogShell::standard`, `render_variable_list_panel`, `TransferButton`, список сформированных пар `target -> METHOD(source)`, поле изменения имени, переключатели методов и счетчик `span`, панель действий `DialogActionBar`)
- [x] L6 `statcraft_app`: интеграция диалога в меню `Transform -> Replace Missing Values...` (снят статус `[dev]`), подключение в `app_dialogs_data.rs`, статус-сообщение `StatusMessage::ReplaceMissingExecuted` и логирование синтаксиса в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_replace_missing_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 48. Ранжирование наблюдений (Rank Cases: `RANK`) — `transform.rank_cases`
- [x] L2 `statcraft_algo`: модули `crates/statcraft_algo/src/rank_cases.rs` и `rank_cases_tests.rs` — функция `compute_ranks`, обработка связей `TiesHandling` (`Mean`, `Low`, `High`, `Seq`), направления `RankDirection` (`Ascending`, `Descending`), типы ранжирования `RankType` (`SimpleRank`, `FractionalRank`, `FractionalPercent`, `Ntiles`, `NormalBlom`, `NormalTukey`, `NormalRankit`, `NormalVanderWaerden`), обратная функция нормального распределения `inv_normal_cdf` (аппроксимация Аклема)
- [x] L5 `statcraft_syntax`: типизированная команда движка `RankCasesCommand` (`transform.rank_cases`) со спецификацией JSON Schema `specs()`, пакетным созданием/перезаписью ранговых переменных со шкалой `Measure::Ordinal`, полной поддержкой отката Undo (`Ctrl+Z`), генератором синтаксиса `RANK VARIABLES=... /TIES=... /RANK ... \n EXECUTE.`
- [x] Локализация: декларативные ключи `dialog-rank-*` и `output-rank-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `RankCasesDialog` ($560 \times 430$ px) по стандарту `DialogKit` (`DialogShell::standard`, `render_variable_list_panel`, `TransferButton`, `GroupBox` для направления и связок, `SubdialogSidebar` с вызовом поддиалога типов рангов `Rank Types...`, нижняя панель `DialogActionBar`)
- [x] L6 `statcraft_app`: интеграция диалога в меню `Transform -> Rank Cases...` (снят статус `InDev`), подключение в `app_dialogs_data.rs`, логирование канонического синтаксиса и результата в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_rank_cases_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`)

## 49. Визуальная группировка (Visual Binning) — `transform.visual_binning`
- [x] L2 `statcraft_algo`: модуль `crates/statcraft_algo/src/binning.rs` — структуры `Cutpoint`, `BinningResult`, функции `compute_visual_binning`, `compute_visual_binning_with_labels`, генераторы срезов `generate_equal_width_cutpoints`, `generate_equal_percentile_cutpoints`, `generate_stddev_cutpoints`, автогенератор меток `generate_bin_labels`
- [x] L5 `statcraft_syntax`: типизированная команда движка `VisualBinningCommand` (`transform.visual_binning`) со спецификацией JSON Schema `specs()`, созданием или перезаписью переменной типа `Numeric` с измерением `Ordinal` и автоприкреплением словаря меток `ValueLabels`, полной поддержкой Undo/Redo и генератором канонического синтаксиса SPSS `RECODE ... INTO ... \n VARIABLE LABELS ... \n VALUE LABELS ... \n EXECUTE.`
- [x] Локализация: ключи `dialog-binning-*` и `output-binning-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: диалог `VisualBinningDialog` по стандарту `DialogKit` (`DialogShell::standard`, `render_variable_list_panel`, имя и метка целевой переменной, переключатель верхних границ включительно/исключительно, таблица срезов с авто/кастомными метками, ручное добавление срезов, поддиалог `Make Cutpoints...`, `DialogActionBar`)
- [x] L6 `statcraft_app`: поддиалог `Make Cutpoints...` (`DialogShell::subdialog`, `SubdialogActionBar`, режимы равной ширины, равных процентилей и стандартных отклонений)
- [x] L6 `statcraft_app`: снятие статуса `[dev]` и активация пункта меню `Transform -> Visual Binning...`, подключение в `app_dialogs_data.rs`, статус-сообщение `StatusMessage::VisualBinningExecuted` и логирование в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_visual_binning_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 48. Ранжирование наблюдений (Rank Cases: `RANK`) — `transform.rank_cases`
- [x] L2 `statcraft_algo`: модули `crates/statcraft_algo/src/rank_cases.rs` и `rank_cases_tests.rs` — функция `compute_ranks`, обработка связей `TiesHandling` (`Mean`, `Low`, `High`, `Seq`), направления `RankDirection` (`Ascending`, `Descending`), типы ранжирования `RankType` (`SimpleRank`, `FractionalRank`, `FractionalPercent`, `Ntiles`, `NormalBlom`, `NormalTukey`, `NormalRankit`, `NormalVanderWaerden`), обратная функция нормального распределения `inv_normal_cdf` (аппроксимация Аклема)
- [x] L5 `statcraft_syntax`: типизированная команда движка `RankCasesCommand` (`transform.rank_cases`) со спецификацией JSON Schema `specs()`, пакетным созданием/перезаписью ранговых переменных со шкалой `Measure::Ordinal`, полной поддержкой отката Undo (`Ctrl+Z`), генератором синтаксиса `RANK VARIABLES=... /TIES=... /RANK ... \n EXECUTE.`
- [x] Локализация: декларативные ключи `dialog-rank-*` и `output-rank-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `RankCasesDialog` ($560 \times 430$ px) по стандарту `DialogKit` (`DialogShell::standard`, `render_variable_list_panel`, `TransferButton`, `GroupBox` для направления и связок, `SubdialogSidebar` с вызовом поддиалога типов рангов `Rank Types...`, нижняя панель `DialogActionBar`)
- [x] L6 `statcraft_app`: интеграция диалога в меню `Transform -> Rank Cases...` (снят статус `InDev`), подключение в `app_dialogs_data.rs`, логирование канонического синтаксиса и результата в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_rank_cases_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`)


## 50. Мастер даты и времени (Date and Time Wizard) — `transform.date_time_wizard`
- [x] L2 `statcraft_algo`: модуль `crates/statcraft_algo/src/date_time.rs` — вычисления с датами, парсинг ISO 8601 / RFC 3339 / `DD.MM.YYYY` / `MM/DD/YYYY` / `YYYY/MM/DD` / эпохи timestamp без паник, извлечение компонентов `DatePart` (Year, Month, Day, Hour, Minute, Second, DayOfWeek, Quarter), расчет разности двух дат `DateUnit` (Years, Months, Weeks, Days, Hours, Minutes, Seconds), сложение и вычитание длительностей `AddDuration`
- [x] L5 `statcraft_syntax`: типизированная команда движка `DateTimeWizardCommand` (`transform.date_time_wizard`) со спецификацией JSON Schema `specs()`, созданием новой переменной (`Numeric` или `Date`) со шкалой `Measure::Scale`, полной поддержкой Undo/Redo и генератором канонического синтаксиса SPSS `COMPUTE target = XDATE.PART(var).`, `COMPUTE target = DATEDIFF(var1, var2, "unit").`, `COMPUTE target = DATESUM(var, amt, "unit").`
- [x] Локализация: ключи `dialog-datetime-*` и `status-datetime-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `DateTimeDialog` ($560 \times 430$ px) по стандарту `DialogKit` (`DialogShell::standard`, выбор задачи мастера в `GroupBox`, панель доступных переменных `render_variable_list_panel`, адаптивная кнопка переноса `TransferButton`, селекторы компонентов даты и единиц времени, нижняя панель `DialogActionBar`)
- [x] L6 `statcraft_app`: снятие статуса `[dev]` и активация пункта меню `Transform -> Date and Time Wizard...`, подключение обработчика в `app_dialogs_transform.rs`, статус-сообщение `StatusMessage::DateTimeExecuted` и логирование в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_date_time_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs` и интеграционные тесты `crates/statcraft_syntax/tests/date_time_cmd_tests.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`)

## 51. Оптимальная группировка (Optimal Binning) — `transform.optimal_binning`
- [x] L2 `statcraft_algo`: модуль `crates/statcraft_algo/src/optimal_binning.rs` — алгоритм MDLP (Fayyad & Irani, 1993) оптимизации энтропии и информационного выигрыша с остановкой по критерию длины описания, ограничение `max_bins`, безопасная обработка константных признаков, одноклассовых целей и `Null`-значений, автогенерация меток интервалов `ValueLabels`
- [x] L5 `statcraft_syntax`: типизированная команда движка `OptimalBinningCommand` (`transform.optimal_binning`) со спецификацией JSON Schema `specs()`, созданием переменных с измерением `Measure::Ordinal`, автоматическим прикреплением `ValueLabels`, поддержкой отката Undo (`Ctrl+Z`) и генератором канонического синтаксиса SPSS `OPTIMAL BINNING /VARIABLES = ... BY ... /CRITERIA METHOD = MDLP [MAXBINS = ...]. \n EXECUTE.`
- [x] Локализация: ключи `dialog-optbin-*`, `output-optbin-executed`, `status-optbin-executed` в `locales/en.ftl` и `locales/ru.ftl`
- [x] L6 `statcraft_app`: канонический диалог `OptimalBinningDialog` по стандарту `DialogKit` (`DialogShell::standard`, доступные переменные, целевая переменная-направитель `Guide Variable`, параметры `max_bins` и `prefix` в `GroupBox`, кнопки переноса `TransferButton`, нижняя панель `DialogActionBar`)
- [x] L6 `statcraft_app`: снятие статуса `[dev]` и активация пункта меню `Transform -> Optimal Binning...`, подключение обработчика в `app_dialogs_transform.rs`, статус-сообщение `StatusMessage::OptimalBinningExecuted` и логирование в `Output Viewer`
- [x] Тесты инвариантов: headless-тест `test_headless_render_optimal_binning_dialog` в `crates/statcraft_app/tests/dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `cargo clippy --workspace -j 1 -- -D warnings`)


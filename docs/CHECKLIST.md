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


## 52. Подготовка данных для моделирования (Automated Data Preparation / ADP) — `transform.adp`
- [x] L2 `statcraft_algo`: подмодуль `adp` (`types.rs`, `screening.rs`, `outliers.rs`, `transform.rs`, `partition.rs`, `predictive_power.rs`, `mod.rs`, `tests.rs`) — многомерный скрининг качества данных (пропуски, квазиконстанты, идентификаторы), детекция выбросов по Z-score и IQR с винзоризацией (Winsorizing) или заменой на системные пропуски, Z-score / Min-Max масштабирование, слияние редких категорий (< 5%), супервизорный MDLP-биннинг, парные кросс-произведения (взаимодействия), предиктивная сила признаков (Pearson r, ANOVA eta^2), детерминированное стратифицированное/случайное разбиение выборки (Train 70% / Test 20% / Validate 10%)
- [x] L5 `statcraft_syntax`: типизированная команда движка `PrepareDataModelingCommand` (`transform.adp`) со спецификацией JSON Schema `specs()`, генератором канонического синтаксиса SPSS `ADP /FIELDS ... /OBJECTIVE ... /PREPAREINPUTS ... /OUTLIERS ... /RESCALE ... /CATEGORICAL ... /TRANSFORM ... /CONSTRUCT ... /PARTITION ... \n EXECUTE.`, интеграцией в `CommandBus`, добавлением преобразованных признаков `P_*` и столбца `_Partition_` со словарем меток в `DataSet`, 100% поддержкой отката Undo (`Ctrl+Z`) и тестами `adp_cmd_tests.rs`
- [x] L4 / L6 Output Viewer: сводные таблицы SPSS (`tables_adp.rs`) — Automated Data Preparation Summary, Action Log / Field Processing Details, Predictive Power / Feature Importance с векторными индикаторами, Partition Breakdown, копирование и экспорт в TSV (`adp_to_tsv`)
- [x] Локализация: полная двуязычная поддержка (`locales/en.ftl` и `locales/ru.ftl`) без хардкода строк для диалогов, поддиалогов и Output Viewer (`dialog-adp-*`, `output-adp-*`, `status-adp-*`)
- [x] L6 `statcraft_app`: канонический диалог `AdpDialog` по стандарту `DialogKit` (`DialogShell::standard`, выбор целевой переменной, предикторов, весов, выбор цели в `GroupBox`, кнопки переноса `TransferButton`, правая панель поддиалогов `SubdialogSidebar`, нижняя панель `DialogActionBar`) и модальные поддиалоги `ScreeningSubdialog`, `TransformSubdialog`, `PartitionSubdialog`
- [x] L6 `statcraft_app`: снятие статуса `[dev]` и активация пункта меню `Transform -> Prepare Data for Modeling (ADP)...`, подключение обработчика в `app_dialogs_transform.rs`, статус-сообщение `StatusMessage::AdpExecuted` и логирование в `Output Viewer`
- [x] Тесты инвариантов: тесты диалогов и таблиц вывода `test_headless_render_adp_dialog`, `test_adp_output_viewer_and_tsv` в `crates/statcraft_app/tests/adp_dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `cargo clippy --workspace -j 1 -- -D warnings`)

## 53. Программируемые преобразования (Programmability Transformations) — `transform.programmability`
- [x] L2 `statcraft_algo`: скриптовый движок `programmability.rs` (`ProgrammabilityScript`, `ProgrammabilityRule`, `ProgrammabilityResult`) — многострочные формулы, условные ветвления `IF / THEN / ELSE`, `:` и `=>`, векторные вычисления, строгая валидация синтаксиса без паник (`Never Crash`), создание новых переменных и модификация существующих
- [x] L5 `statcraft_syntax`: типизированная команда движка `ProgrammabilityCommand` (`transform.programmability`) со спецификацией JSON Schema `specs()`, предикатом доступности, генерацией канонического синтаксиса `BEGIN PROGRAM ... END PROGRAM.\nEXECUTE.`, регистрацией в `CommandBus`, полной поддержкой отката Undo (`Ctrl+Z`) и тестами `programmability_cmd_tests.rs`
- [x] L6 `statcraft_app`: диалог `render_programmability_dialog` по канону `DialogKit` (`DialogShell::standard`, `VariablePicker`, кнопки быстрого добавления шаблонов формул, редактор с живым векторным индикатором валидности синтаксиса, нижняя панель `DialogActionBar`)
- [x] L4 / L6 Output Viewer: сводные таблицы SPSS (`tables_programmability.rs`) — Script Execution Summary, Execution Action Log, копирование и экспорт в TSV (`programmability_to_tsv`)
- [x] Меню и интерфейс: снятие последнего статуса `[dev]` в меню Transform (`menu-transform-programmability`), статус-сообщение `StatusMessage::ProgrammabilityExecuted`
- [x] Локализация: полная двуязычная поддержка (`locales/en.ftl` и `locales/ru.ftl`) без хардкода строк (`dialog-programmability-*`, `output-programmability-*`)
- [x] Тесты инвариантов: тесты диалога и таблиц `crates/statcraft_app/tests/programmability_dialog_invariants.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`)

## 54. Этап Clean-Room Аудита ядра (Блоки 7–10: Вычисления, Временные ряды, Даты, Перекодирование)
- [x] L2 `statcraft_algo`: Блок 7 (`COMPUTE` / `expr_eval/`) — Shunting-Yard AST, правоассоциативное возведение в степень, функции `SQRT`, `EXP`, `LN`, `LOG10`, `ABS`, `RND`, `TRUNC`, статистические агрегаты `SUM`, `MEAN`, `MIN`, `MAX`, трехзначная логика и изоляция `NaN` / `SYSMIS` (`Never Crash`)
- [x] L2 `statcraft_algo`: Блок 8 (`time_series.rs`, `shift.rs`) — операторы разностей `DIFF(d)`, сезонных разностей `SDIFF(s)`, скользящих средних `SMA(k)`, кумулятивных сумм `CSUM` и сдвигов `LAG(k)` / `LEAD(k)`
- [x] L2 `statcraft_algo`: Блок 9 (`date_time.rs`) — мастер дат и времени, календарные разности `DATEDIFF` в днях, неделях, месяцах, годах, прибавление интервалов `DATEADD`, извлечение компонентов `XDATE.*`, мультиформатный парсинг ISO 8601, европейского и американского форматов
- [x] L2 `statcraft_algo`: Блок 10 (`recode.rs`, `auto_recode.rs`, `dummies.rs`, `count_values.rs`) — `RECODE (SAME / DIFFERENT)` с диапазонами `LOWEST THRU` и `THRU HIGHEST`, `AUTORECODE` со сбором словаря `ValueLabels`, генерация дамми-переменных с опцией `omit_first`, подсчет вхождений `COUNT VALUES`
- [x] L5 `statcraft_syntax`: валидация и 100% покрытие командами движка (`ComputeCommand`, `TimeSeriesCommand`, `ShiftValuesCommand`, `DateTimeWizardCommand`, `RecodeSameCommand`, `RecodeDifferentCommand`, `AutoRecodeCommand`, `DummyCommand`, `CountValuesCommand`) со 100% поддержкой отката Undo (`Ctrl+Z`)
- [x] Документация: синхронное обновление двуязычных протоколов аудита `docs/SPSS_ALGORITHMS_AUDIT.md` и `docs/SPSS_ALGORITHMS_AUDIT_RU.md` с фиксацией результатов построчной инструментальной верификации (Craft Anything MCP) и зафиксированных расхождений (Блоки 7 и 8)
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`)

## 55. Завершение Фазы 2 Clean-Room Аудита (Блоки 11–14: Микропланы M-09 — M-20)
- [x] L2/L5: Блок 11 (M-09 Programmability, M-10 `SET SEED` & RNG) — парсинг скриптов `BEGIN PROGRAM`, условные ветвления, атомарность, диапазон сида $[1, 2\,000\,000\,000]$ и применение SplitMix64
- [x] L5/L1/L2: Блок 12 (M-11 `SELECT CASES`, M-12 `SORT CASES`, M-13 `WEIGHT BY`, M-14 `SPLIT FILE`, M-15 Metadata) — `active_row_indices`, фильтрация и удаление с откатом Undo, стабильная сортировка с `Ordering::Less` для пропусков, отсечение $w_i \le 0$, комбинаторный сплит с автосортировкой, 11 атрибутов словаря
- [x] L0 `statcraft_io`: Блок 13 (M-16 `spss_loader/`, M-17 `excel_loader.rs`, M-18 `csv_loader.rs`, M-19 `stc_codec.rs`) — Record Types 1 ($FL2/$FL3), 2, 3/4, 7 (подтипы 3, 4, 11, 13, 20), байт-код и zsav ZLIB декомпрессия, санирование колонок Excel (префикс `V_`), RFC 4180 и контейнер `.stc`
- [x] L4 `statcraft_reporting`: Блок 14 (M-20 Pivot Tables & Exporters) — двухъярусные супер-заголовки, подавление лидирующего нуля `.000` для $p$-value, соответствие экспорта TSV/HTML/TXT
- [x] Craft Anything MCP: фиксация гипотез `hyp_16` .. `hyp_32` со статусом `supported`, обновление протокола `docs/CRAFT_ANYTHING_AUDIT_REPORT.md`
- [x] Документация: полное обновление сводной матрицы и разделов в `docs/SPSS_ALGORITHMS_AUDIT.md` и `docs/SPSS_ALGORITHMS_AUDIT_RU.md` (14 блоков, M-01 — M-20)
- [x] 100% прохождение всех тестов Definition of Done (`cargo test --workspace -j 1`, `xtask file-guard`, `xtask layers`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 56. Завершение Фазы 3 Clean-Room Аудита (Интерфейс, Таблицы вывода, DialogKit, Локализация и Сборка: Микропланы M-21 — M-33)
- [x] L6/L4 Output Viewer: Раздел E (M-21 — M-24) — базовые таблицы вывода (Descriptives, Frequencies с 4 каноническими колонками, Crosstabs с двухъярусными заголовками и остатками, Means с ассоциациями), таблицы сравнения средних (M-22: Independent Samples с тестом Левена и 2 взаимоисключающими строками, Paired Samples с 3 таблицами, One-Way ANOVA с 3 секциями), многомерный анализ (M-23: Correlations со звездочками значимости, Linear Regression с 4 таблицами и Tol/VIF, Factor с KMO/коммунальностями, Reliability с альфа Кронбаха, непараметрика, ADP, Programmability), иерархическое дерево оглавления (M-24: Outline Tree с навигацией, контекстным меню TSV/текста и экспортом TSV/HTML/TXT). 11/11 тестов `output_viewer_invariants` пройдены
- [x] L6 Interactive Screens: Раздел F (M-25 — M-27) — Data View (M-25: виртуализация строк `show_rows` на базе ScrollArea, инлайн-редактирование, переключение Value Labels 1/A, диагональное перечеркивание неактивных номеров строк при фильтре `FILTER BY`), Variable View (M-26: 11 канонических колонок метаданных словаря SPSS, строгая валидация имен по канону SPSS, вызов модальных редакторов типов, меток и пропусков), Syntax Editor (M-27: моноширинный редактор кода, интеграция с `CommandBus`, пакетное исполнение команд и шина вставки `[ Paste ]`)
- [x] L6 DialogKit & 37 Dialogs: Раздел G (M-28 — M-29) — единые стандартизированные размеры `DialogSize` (Standard 560x430, Subdialog 360x300, Properties 460x380, Compute 620x480, Alert 440x170), центрирование окон, нижний ряд действий Type 1 (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`) с динамическим выравниванием ширины кнопок по правилу `i18n Shield` ($\ge 80 \times 26$ px), компоненты выбора переменных `VariablePicker` с векторными иконками шкал и адаптивная `TransferButton`. Статический линтер `xtask design-guard` подтвердил 0 нарушений во всех 37 процедурных диалогах, 37/37 тестов `dialog_invariants` пройдены
- [x] L6/i18n/Packaging: Раздел H (M-30 — M-33) — 100% паритет ключей Fluent между `en.ftl` и `ru.ftl`, нулевой хардкод естественных языков в исходном коде UI, 100% чистый векторный UI без Unicode эмодзи и спец-символов (предотвращение квадратов «тофу» во всех средах), надежная защита данных `Session Guard` (dirty flag `*`, диалог `ConfirmSaveDialog`, очередь `RecentFiles`), и синхронизация версий 0.2.1 во всех 4 манифестах (`Cargo.toml`, `installer.iss`, `Info.plist`, `control`) с поддержкой кроссплатформенной CI сборки (Windows Inno Setup, macOS DMG/.app, Linux .deb/tar.gz)
- [x] Craft Anything MCP: регистрация и доказательное подтверждение 13 гипотез `hyp_33` .. `hyp_45` со статусом `supported`, протоколирование в `core_handoff_write`
- [x] Документация: создание официальных двуязычных протоколов аудита интерфейса `docs/SPSS_UI_CANON_AUDIT.md` (EN) и `docs/SPSS_UI_CANON_AUDIT_RU.md` (RU)
- [x] 100% прохождение всех тестов Definition of Done (`cargo test --workspace -j 1`, `xtask file-guard`, `xtask layers`, `xtask design-guard`, `xtask parity`, `clippy -D warnings`)

## 57. Корпусный аудит форматов данных (Real-World Corpora & Codecs Audit: Микропланы C-01 — C-05)
- [x] L0 `statcraft_io` (C-01): SPSS `.sav`/`.zsav` Stream — полная матрица байт-кодов 0..255, исправление декодирования опкода 254 (`0.0`), верификация опкода 255 (`SYSMIS`), блочная ZLIB декомпрессия zheader/ztrailer, поддержка порядка байт Little/Big Endian (`BUG-003`)
- [x] L0 `statcraft_io` (C-02): Метаданные, кодировки и Very Long Strings — каноническая поддержка Record 7 Subtype 11 (шкала, ширина, выравнивание, `BUG-004`), Subtype 14 (длинные строки $> 255$ байт), Subtype 3 (codepage), Subtype 20 (encoding) и транскодер `encoding.rs` для Windows-1251 (русский), Windows-1252 и KOI8-R
- [x] L0 `statcraft_io` (C-03): Excel Sheets Ingest — обработка "грязных" колонок (числа + текст), безопасная трансляция формульных ошибок (`#VALUE!`, `#REF!`, `#DIV/0!`, `#N/A`, `#NAME?`) в `RawValue::Null`, парсинг экстремальных дат, санирование и дедупликация заголовков по канону SPSS
- [x] L0 `statcraft_io` (C-04): CSV/TSV кодек RFC 4180 — разбор многострочных ячеек в кавычках (`"line 1\nline 2"`), экранированных кавычек (`""`), автоопределение разделителей (`,`, `;`, `\t`, `|`), отсечение сигнатуры UTF-8 BOM (`\u{FEFF}`) и инвариант обратимости (round-trip)
- [x] Утилита сборщика (C-05): реализация раннера `cargo xtask test-corpus` и стресс-тестирование фаззингом (11 аномальных потоков, усеченные заголовки, битые ZLIB/JSON/BOM) с подтверждением инварианта `Never Crash` (0 паник, 100% safe Err)
- [x] Документация: создание двуязычных отчетов `docs/CORPUS_AUDIT_REPORT.md` (EN) и `docs/CORPUS_AUDIT_REPORT_RU.md` (RU), регистрация дефектов `BUG-003` и `BUG-004` в `docs/BUGS_AND_ISSUES.md`
- [x] 100% прохождение всех тестов Definition of Done (`cargo test --workspace -j 1`, `xtask file-guard`, `xtask layers`, `xtask design-guard`, `xtask parity`, `xtask test-corpus`, `clippy --workspace -j 1 -- -D warnings`)

## 58. Пакетный синтаксический движок и аудит (Batch Syntax Engine & Parser Audit: Микропланы S-01 — S-05)
- [x] L5 `statcraft_syntax` (S-01): `SyntaxLexer` — разбиение скриптов на команды по завершающим точкам `.`, исключение точек в числах (`3.14`, `0.95`) и путях в кавычках (`'path.sav'`), разбор комментариев `*` и `COMMENT`, экранированные кавычки `''` и `""`
- [x] L5 `statcraft_syntax` (S-02): `SyntaxParser` — разбор подкоманд `/SUBCOMMAND` и позиционных параметров, токенизация параметров в скобках, разворачивание диапазонов `VAR1 TO VAR5` по порядку колонок `VariableDictionary` и ключевого слова `ALL`
- [x] L5 `statcraft_syntax` (S-03): `SyntaxDispatcher` — модульные мапперы `mappers/stats.rs` и `mappers/data.rs` в вызовы `CommandBus::execute` с типизированными JSON-параметрами для 15+ канонических процедур SPSS (`DESCRIPTIVES`, `FREQUENCIES`, `CROSSTABS`, `T-TEST`, `ONEWAY`, `CORRELATIONS`, `REGRESSION`, `RELIABILITY`, `FACTOR`, `MEANS`, `SORT CASES`, `WEIGHT`, `FILTER`, `SPLIT FILE`, `COMPUTE`, `GET`, `SAVE`, `EXECUTE`)
- [x] L6 `statcraft_app` (S-04): Пакетный раннер и GUI (`syntax_runner.rs`, `views/syntax_editor.rs`) — кнопки действий `[ Run All ]`, `[ Run Selection ]`, `[ Clear ]`, эхо команд в журнал `Output Viewer` с префиксом `> ` и автоматическое прикрепление сводных таблиц результатов
- [x] Clean-room аудит (S-05): фикстуры `fixtures/syntax/`, 100% round-trip интеграционные тесты и стресс-тестирование фаззингом без паник (`Never Crash`), создание отчетов `docs/SPSS_SYNTAX_PARSER_AUDIT.md` (EN) и `docs/SPSS_SYNTAX_PARSER_AUDIT_RU.md` (RU)
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask file-guard`, `xtask layers`, `xtask design-guard`, `xtask parity`, `clippy --workspace -j 1 -- -D warnings`)

## 59. Открытие и сохранение файлов синтаксиса SPSS (`.sps`) (Syntax File I/O)
- [x] L6 `statcraft_app::views::syntax_editor`: поле `current_file_path: Option<PathBuf>`, расширение `SyntaxEditorAction` вариантами `Open` и `Save`, кнопки тулбара `[ Открыть .sps ]` (`syntax-open`) и `[ Сохранить .sps ]` (`syntax-save`) с отображением имени текущего файла
- [x] L6 `statcraft_app::app_session`: безопасные диалоги файлового ввода-вывода `open_syntax_dialog`, `save_syntax`, `save_syntax_as`, `write_syntax_to_path` через `rfd::FileDialog`, поддержка расширения `.sps` в `load_file_from_path` с интеграцией в `RecentFiles` и автоматическим переключением на вкладку `SyntaxEditor`
- [x] L6 Меню `Файл`: добавление канонических пунктов `menu-file-open-syntax` («Открыть синтаксис...») и `menu-file-save-syntax-as` («Сохранить синтаксис как...»)
- [x] L6 Статусная строка: варианты `StatusMessage::SyntaxOpened` и `StatusMessage::SyntaxSaved` с зеленой индикацией `Tokens::MEASURE_NOMINAL`
- [x] Локализация: двуязычные ключи в `locales/en.ftl` и `locales/ru.ftl` (`syntax-open`, `syntax-save`, `menu-file-open-syntax`, `menu-file-save-syntax-as`, `dialog-open-syntax-title`, `dialog-save-syntax-title`, `dialog-filter-sps`, `status-syntax-opened`, `status-syntax-saved`)
- [x] Тесты жизненного цикла: интеграционный тест `test_syntax_save_and_load_lifecycle` в `session_lifecycle_tests.rs` и тесты состояния `syntax_editor.rs`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask file-guard`, `xtask layers`, `xtask design-guard`, `xtask parity`, `clippy --workspace -j 1 -- -D warnings`)

## 60. Релиз v0.2.2: Полный каталог функциональных возможностей и синхронизация релизных манифестов
- [x] Документация: создание исчерпывающего двуязычного справочника всех возможностей программы `docs/FEATURES_RU.md` (RU) и `docs/FEATURES.md` (EN) с подробным описанием всех 12 меню, редакторов, алгоритмов и аналитических процедур
- [x] Документация: внедрение заметных блоков со ссылками на справочники возможностей в разделах «Key Features» / «Основной функционал» и «Documentation» / «Документация» файлов `README.md` и `README.ru.md`
- [x] Релизные заметки: актуализация двуязычного файла `RELEASE_NOTES.md` под версию v0.2.2 для автоматической публикации релиза через GitHub Actions
- [x] Синхронизация версий (Packaging): синхронизация версии `0.2.2` байт-в-байт в 4 файлах манифестов (`Cargo.toml`, `packaging/windows/installer.iss`, `packaging/macos/Info.plist`, `packaging/linux/control`) и обновление `Cargo.lock`
- [x] Хроника разработки: синхронное добавление записи о релизе v0.2.2 в `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask file-guard`, `xtask layers`, `xtask design-guard`, `xtask parity`, `clippy --workspace -j 1 -- -D warnings`)

## 61. Сетевой мост StatCraft: Code Mode, Семантический лог & Agent Kit (`statcraft_bridge`)
- [x] L-Utility `statcraft_bridge`: архитектура Code Mode (6 компактных инструментов: `statcraft_run_syntax`, `statcraft_load_dataset`, `statcraft_list_variables`, `statcraft_get_data_preview`, `statcraft_insert_output_note`, `statcraft_set_active_tab`), протокол JSON-RPC 2.0, TCP-сервер на 127.0.0.1:18493 с неблокирующим сокетом, `BridgeClient` и stdio MCP-сервер (`statcraft.exe --mcp`)
- [x] L6 `statcraft_app::bridge_service`: сервис моста в `StatCraftApp`, канал запросов `BridgeAppRequest` и ответов `oneshot`, обработка в кадре `update()`, диспетчеризация пакетного синтаксиса, прямой импорт сырых CSV в память, кольцевой буфер `BridgeLogRing`
- [x] L6 UI `dialogs/mcp_bridge_dialog`: модульный диалог на базе `DialogKit` (`DialogShell::standard`, `GroupBox`, `SubdialogActionBar`), векторный индикатор статуса, копирование конфигурации и навыка, прокручиваемый семантический журнал активности с динамической локализацией на лету
- [x] Меню и флаги CLI: активация пункта меню `menu-extensions-mcp-bridge`, флаг командной строки `--mcp` в `main.rs` для запуска stdio MCP без создания графического окна
- [x] Agent Kit и локализация: системный навык `.agents/skills/statcraft-assistant/SKILL.md`, синхронизированные ключи в `locales/en.ftl` и `locales/ru.ftl`
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard`, `xtask design-guard`, `xtask parity`, `clippy --workspace -j 1 -- -D warnings`)

## 62. Модернизация моста StatCraft Bridge v2: Автономный Code Mode, Output Return & Dynamic Self-Documentation
- [x] L5 `statcraft_syntax`: исправление маппингов процедур `ONEWAY` (`dependent_vars`, плоские `descriptives`/`homogeneity`), `RELIABILITY` (`variables`), `FACTOR` (`variables`) и `MEANS` (плоские `cell_*` опции) под схемы параметров `CommandBus`
- [x] L5 `statcraft_syntax`: синтаксические мапперы для словаря переменных `VARIABLE LEVEL`, `VARIABLE LABELS`, `VALUE LABELS`, `MISSING VALUES` с поддержкой назначения по именам столбцов и блочной передачи
- [x] L5 `statcraft_syntax`: синтаксическая самодокументация `HELP.` и `HELP <COMMAND>.`, команда `HelpCommand` (`util.help`) и подсказки самокоррекции `Did you mean ...?` на базе расстояния Левенштейна
- [x] L-Utility `statcraft_bridge`: поддержка MCP Resources (`resources/list`, `resources/read`) для динамического справочника `statcraft://syntax-reference` и возврат отформатированных ASCII-таблиц `output_text` прямо в ответе `statcraft_run_syntax`
- [x] L6 `statcraft_app`: расширение `syntax_runner::run_syntax_batch` структурой `SyntaxBatchOutcome`, автоматический экспорт созданных процедур через `export_procedures_to_txt` и передача в `bridge_service`
- [x] Документация и навыки: рефакторинг манифеста `.agents/skills/statcraft-assistant/SKILL.md` под автономный Code Mode REPL
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `xtask layers`, `xtask file-guard` (294 файла в лимитах), `xtask design-guard`, `cargo fmt --check`, `clippy --workspace -j 1 -- -D warnings`)








## 63. Самодостаточный интроспективный движок моста и универсальный Code Mode (Zero-Maintenance Introspective Bridge)
- [x] L5 `statcraft_syntax`: добавление `syntax_names` и `category` в типаж `EngineCommand`, динамическая интроспекция `CommandBus::list_specs()`, `find_by_syntax_keyword` и `all_syntax_keywords`
- [x] L5 `statcraft_syntax`: реализация универсальной директивы `RUN COMMAND "<command_id>" { <json_params> }.` и `CALL "<command_id>" { ... }.` в `SyntaxParser` и `SyntaxDispatcher` для прямого вызова любых зарегистрированных команд ядра
- [x] L-Utility `statcraft_bridge`: универсальный инструмент вызова `statcraft_execute_command`, инструмент интроспекции `statcraft_list_commands` и динамический ресурс `statcraft://commands` с форматированием каталога в Markdown
- [x] L6 `statcraft_app`: ликвидация хардкода в `syntax_runner.rs`, внедрение универсального элемента вывода `OutputItemKind::GenericReport` и модуля `tables_generic.rs` (Zero-Match Fallback для любых существующих и будущих процедур)
- [x] L6 `statcraft_app::bridge_service`: обработка запросов `ListCommands` и `ExecuteCommand` с автоматической регистрацией результатов в `OutputViewer` и отдачей форматированного текста `output_text`
- [x] Локализация: добавление ключей `output-generic-title` и `output-table` в `locales/en.ftl` и `locales/ru.ftl`
- [x] 100% прохождение проверок Definition of Done (`cargo test --workspace -j 1`, `cargo test -p statcraft_i18n -j 1`, `xtask layers`, `xtask file-guard` [295 файлов], `xtask design-guard`, `cargo fmt --check`, `clippy --workspace -j 1 -- -D warnings`)

## 64. Автоматическое определение языка системы и сохранение настроек сессии (Auto-Locale & Storage Persistence)
- [x] L-Cross `statcraft_i18n`: изолированный модуль автоопределения языка системы `detect.rs` (`STATCRAFT_LANG`, нативный вызов через локальный `sys-locale`, переменные `LC_ALL`, `LC_MESSAGES`, `LANG`), нормализация тегов `normalize_language_tag` и автоматическая активация при старте
- [x] L-Cross `statcraft_i18n`: гарантированная ленивая инициализация каталога при любых вызовах `ensure_initialized()` для исключения гонок потоков в тестах
- [x] L6 `statcraft_app::app`: ключи хранилища `STORAGE_KEY_LANGUAGE` и `STORAGE_KEY_BRIDGE_AUTOSTART`
- [x] L6 `statcraft_app::app`: сохранение в `save(&mut self, storage)` и восстановление в `new(cc)` сохранённого языка и состояния чекбокса автозапуска MCP моста
- [x] L6 `statcraft_app`: автономный интеграционный тест `tests/storage_persistence_test.rs` для проверки сохранения и восстановления параметров сессии
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `cargo test -p statcraft_i18n -j 1`, `xtask layers`, `xtask file-guard` [296 файлов], `xtask design-guard`, `cargo fmt --check`, `clippy --workspace -j 1 -- -D warnings`)
- [x] Обновление релизного бинарника `target/release/statcraft_app.exe` и `target/release/statcraft.exe`

## 65. Универсальная система тем оформления (Dark, Light, System Theme System)
- [x] L6 `statcraft_app::theme`: масштабируемый движок тем `ThemeMode` (`Dark`, `Light`, `System`), палитры `DARK_PALETTE` и `LIGHT_PALETTE` (SPSS Classic Light с WCAG-контрастностью AAA $\ge 7:1$ для основного текста и AA $\ge 4.5:1$ для вторичного)
- [x] L6 `statcraft_app::theme`: динамические методы `Tokens::bg_app()`, `Tokens::bg_panel()`, `Tokens::border()`, `Tokens::text_primary()` и др. на базе атомарного безопасного переключателя `IS_DARK_MODE: AtomicBool` (100% Safe Rust, zero-allocation на горячем пути рендеринга egui)
- [x] L6 `statcraft_app`: миграция всех 53 файлов кодовой базы с констант на динамические методы `Tokens::*()`, гарантирующая автоматическую адаптацию всех существующих и будущих диалогов `DialogKit`
- [x] L6 `statcraft_app::app`: FPS Guard — вызов `ctx.set_visuals` и `set_dark_mode` только при фактическом изменении режима темы, кэширование в `applied_dark_mode`
- [x] L6 `statcraft_app::app`: сохранение `STORAGE_KEY_THEME = "statcraft_theme"` в `eframe::Storage` и восстановление темы при перезапуске
- [x] L6 `statcraft_app::app_menus::file_edit_view`: активация переключения тем в меню `Вид -> Тема` (`menu-view-theme-light`, `menu-view-theme-dark`, `menu-view-theme-system`) строго по семантическим ключам Fluent без хардкода строк
- [x] 100% прохождение всех проверок Definition of Done (`cargo test --workspace -j 1`, `cargo test -p statcraft_app -j 1`, `xtask layers`, `xtask file-guard` [297 файлов], `xtask design-guard`, `cargo fmt --check`, `clippy --workspace -j 1 -- -D warnings`)

## 66. Релиз v0.3.0: Сетевой мост MCP, универсальные темы и автоопределение языка (Release v0.3.0)
- [x] Синхронизация версии `0.3.0` байт-в-байт в 4 манифестах пакетирования (`Cargo.toml`, `packaging/windows/installer.iss`, `packaging/macos/Info.plist`, `packaging/linux/control`) и обновление `Cargo.lock`
- [x] Обновление тега релиза по умолчанию в `.github/workflows/release.yml` (`v0.3.0`)
- [x] Подробное описание релиза в `RELEASE_NOTES.md` на русском и английском языках (Bridge, Theme System, Auto-Locale, Universal Code Mode)
- [x] Интеграция подробного описания сетевого моста MCP и работы с ИИ-агентами под спойлером в `README.ru.md` и `README.md`
- [x] Сборка оптимизированных релизных бинарников `target/release/statcraft_app.exe` и `target/release/statcraft.exe`
- [x] Хроника разработки: синхронное добавление записи о релизе v0.3.0 в `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md`

## 67. Матричный аудит обратимости командной шины (Undo / Redo Exhaustive Matrix)
- [x] L0 `statcraft_io` & L1 `statcraft_core`: методы вставки по произвольному индексу (`RawTable::insert_column`, `RawTable::insert_row`, `VariableDictionary::insert`, `DataSet::restore_variable`, `DataSet::insert_row`, `set_variable_label`, `set_variable_measure`), очистка ссылок на удаляемые колонки из `DeltaBuffer`
- [x] L5 `statcraft_syntax::commands`: реализация полноценных `undo()` для структурных команд и метаданных (`var.add`, `var.delete`, `var.rename`, `var.set_type`, `var.set_width`, `var.set_decimals`, `var.set_align`, `var.set_label`, `var.set_measure`, `data.add_row`, `data.delete_row`)
- [x] L5 `statcraft_syntax::bus`: реализация `redo_stack`, методов `bus.redo(&mut ds)`, `can_undo()`, `can_redo()`, очистка redo при выполнении новых команд (ветвление истории), лимит глубины 100 шагов с вытеснением по FIFO и коалесценция правок ячеек `execute_coalesced`
- [x] L6 `statcraft_app`: сквозное связывание `can_redo` и вызова `command_bus.redo` в главном тулбаре (`ToolbarAction::Redo`) и меню `Правка -> Повторить` (`menu-edit-redo`)
- [x] Тестовый пакет L5: 38 модульных матричных тестов в `crates/statcraft_syntax/tests/undo_matrix/` с проверкой математической эквивалентности `DataSet0 == Undo(Execute(DataSet0))` и `DataSet1 == Redo(...)` (100% PASS)
- [x] Xtask & CI: автоматический раннер матрицы обратимости `cargo xtask test-undo`, регистрация в `main.rs` и включение в качестве шлюза #9 в `cargo xtask ci`
- [x] Документация: формирование исчерпывающего двуязычного отчета об аудите `docs/UNDO_REDO_AUDIT_REPORT.md` (EN) и `docs/UNDO_REDO_AUDIT_REPORT_RU.md` (RU)
- [x] 100% прохождение всех проверок Definition of Done (`cargo run -p xtask -j 1 -- ci`: fmt, clippy, layers, file-guard [307 файлов], design-guard, full test suite, parity, corpus, test-undo)


## 38. Финализация операций меню редактирования и табличной навигации (Edit & Grid Navigation)
- [x] L1 `statcraft_core`: добавление метода `DataSet::insert_variable` со сдвигом столбцов и синхронизацией `VariableDictionary` и `RawTable`
- [x] L5 `statcraft_syntax`: поддержка позиционной вставки строк и переменных в `DataAddRowCommand` (`data.add_row`) и `VarAddCommand` (`var.add`) с полным циклом Undo/Redo
- [x] L5 `statcraft_syntax`: типизированная команда пакетной и одиночной замены `BatchReplaceCommand` (`data.replace_values`) с атомарной транзакцией Undo/Redo и поддержкой поиска/замены подстрок и полных ячеек
- [x] L6 `statcraft_app`: модальный диалог поиска и замены `FindReplaceDialog` (`dialogs/find_replace_dialog.rs`) по канону DialogKit (`DialogShell::subdialog`, вкладки Find/Replace, направление поиска Down/Up, точное совпадение ячеек, учет регистра, фильтр по столбцу)
- [x] L6 `statcraft_app`: модальный диалог навигации `GoToDialog` (`dialogs/goto_dialog.rs`) по канону DialogKit (`DialogShell::subdialog`, `SubdialogActionBar`, вкладки Go to Case с валидацией $1 \le k \le N$ и Go to Variable с фильтрацией и векторными иконками шкал)
- [x] L6 `statcraft_app`: реализация навигационных смещений в `DataViewState` (`scroll_to_row`, `scroll_to_col`) и `VariableViewState` (`scroll_to_var`, `selected_var`) для мгновенного скролла и визуальной подсветки
- [x] L6 `statcraft_app`: снятие статуса `InDev` и подключение всех пунктов меню `Edit` (Cut, Copy, Copy with Variable Names, Paste, Clear, Find, Replace, Go to Case, Go to Variable, Insert Case, Insert Variable) с клавиатурными комбинациями (Ctrl+F, Ctrl+H, Ctrl+G, Ctrl+Shift+C, Ctrl+C, Ctrl+X, Ctrl+V, Delete)
- [x] Синхронная локализация: добавление ключей диалогов поиска, замены и навигации в `locales/en.ftl` и `locales/ru.ftl`
- [x] Тестирование и верификация: тестовый модуль `edit_navigation_tests.rs`, 100% прохождение тестов воркспейса, `xtask design-guard` (0 нарушений) и `xtask file-guard` (313 файлов).

## 39. Автоматический локальный Quality Gate и версионируемый Git pre-push хук (Pre-Push Hook)
- [x] Версионируемый хук `.githooks/pre-push`: автоматический вызов всех 9 ступеней Quality Gate `cargo run -p xtask -j 1 -- ci` при попытке `git push` и немедленная блокировка отправки при любой ошибке
- [x] Команда развертывания `cargo xtask install-hooks`: реализация в `xtask/src/hooks.rs` с настройкой `git config core.hooksPath .githooks`
- [x] Регламент `AGENTS.md`: Раздел 14 со строгим запретом обхода хуков через `--no-verify`
- [x] 100% прохождение всех проверок Definition of Done (`cargo run -p xtask -j 1 -- ci`).

## 40. Устранение расхождений аудита и 100% каноничность IBM SPSS (Divergences Fix с Craft Anything MCP)
- [x] L2 `statcraft_algo::expr_eval::parser`: каноническая иерархия приоритетов операторов SPSS Universals (степень `**` опережает унарный минус/плюс: `-2**2 == -4.0`, отношения опережают `NOT`: `NOT a = b` -> `NOT (a = b)`)
- [x] L2 `statcraft_algo::expr_eval::mod`: строгая трехзначная логика Клини (Kleene 3VL) при наличии `NaN` (`TRUE OR SYSMIS == 1.0`, `FALSE AND SYSMIS == 0.0`, `NOT SYSMIS == SYSMIS`)
- [x] L2 `statcraft_algo::expr_eval::eval`: реализация функций `SIN`, `COS`, `TAN` (с возвратом `NaN` в точках сингулярности), остатка `MOD(x, y)` с защитой от деления на ноль, канонического алиаса `LG10` и строгого режима `MEAN` (требует отсутствия пропусков без суффикса `.n`)
- [x] L2 `statcraft_algo::time_series`: разделение на центрированное скользящее среднее `SimpleMovingAverage` (`SMA`, с весами 0.5 на границах для четных $k = 2m$) и предшествующее окно `PriorMovingAverage` (`PMA`)
- [x] L5 `statcraft_syntax::commands::time_series_cmd`: поддержка функций `SMA` и `PMA` в синтаксисе команды `CREATE` и схеме параметров `specs()`
- [x] Журнал дефектов `docs/BUGS_AND_ISSUES.md`: регистрация и перевод в статус `RESOLVED` дефектов `BUG-006`, `BUG-007`, `BUG-008`, `BUG-009`
- [x] Документы аудита `docs/SPSS_ALGORITHMS_AUDIT.md` и `docs/SPSS_ALGORITHMS_AUDIT_RU.md`: перевод всех 14 аналитических блоков в статус **100% Verified (0 Divergences)**
- [x] Двуязычная хроника проекта: добавление записи в `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md` (Этап / Phase 71)
- [x] 100% успешный прогон всех ступеней Quality Gate `cargo run -p xtask -j 1 -- ci` (fmt, clippy, layers, file-guard [314 файлов], design-guard, workspace tests, parity, corpus, test-undo).

## 41. Разведочный анализ данных (Exploratory Data Analysis / EXAMINE & Normality Tests — Ветка C)
- [x] Доказательный Clean-Room аудит алгоритмов через Craft Anything MCP (`core_brief_write`, `core_plan_update`, `core_hypothesis_record` hyp_1..hyp_3 `supported`, `core_handoff_write`)
- [x] L2 `statcraft_algo::explore`:
  - [x] Дескриптивные метрики `descriptives.rs`: каноническое 5% усеченное среднее (5% Trimmed Mean) с дробным взвешиванием граничных элементов, среднее, SE среднего, 95% ДИ среднего через квантили Стьюдента, медиана, дисперсия, SD, минимум, максимум, размах, межквартильный размах ($IQR$ через HAVERAGE), асимметрия с SE, эксцесс с SE
  - [x] Критерий нормальности Шапиро-Уилка `normality_sw.rs`: аппроксимации полиномами Ройстона (1992) для $3 \le N \le 5000$, Blom normal order statistics scores, точный расчет $p$-value (сверка с Golden Oracle `scipy.stats.shapiro` с точностью $\le 10^{-6}$)
  - [x] Критерий нормальности Лиллиефорса `normality_ks.rs`: поправка Колмогорова-Смирнова по формуле Даллала-Уилкинсона с каноническим клампингом SPSS $[0.001, 0.20]$
  - [x] Экстремальные значения `extremes.rs`: 5 наименьших и 5 наибольших наблюдений с сохранением исходных 1-based номеров строк и корректной обработкой связок (ties)
  - [x] Координатор процедуры `mod.rs`: группировка по факторам (BY factor), режимы обработки пропусков Listwise/Pairwise, фильтрация активных наблюдений
- [x] L5 `statcraft_syntax`:
  - [x] Команда движка `ExploreCommand` (`stat.explore`): схема параметров `specs()`, чтение входных переменных и факторов, генерация SPSS-синтаксиса `to_sps()`
  - [x] Синтаксический парсер `mappers/stats.rs`: маппинг `EXAMINE` и `EXPLORE` с поддержкой `VARIABLES=... BY ...`, `/PLOT NPPLOT`, `/STATISTICS`, `/MISSING`
  - [x] Регистрация в `bus.rs` и интеграционный roundtrip-тест в `syntax_roundtrip_and_fuzz_tests.rs`
- [x] L4/L6 `statcraft_app::views::output_view`:
  - [x] Структурированное представление `OutputItemKind::Explore`
  - [x] Архитектурная декомпозиция модуля `tables_explore/` (`mod.rs`, `tables.rs`, `tsv.rs` — все файлы $\le 500$ строк)
  - [x] Отрисовка 4 аутентичных сводных таблиц SPSS Pivot Tables: Case Processing Summary, Descriptives, Tests of Normality, Extreme Values
  - [x] Поддержка экспорта в TSV (`explore_to_tsv`)
  - [x] Интеграция в `OutputViewerState::add_explore`, `extract_item_tsv` и `syntax_runner`
- [x] UI диалог `ExploreDialog` (`crates/statcraft_app/src/dialogs/explore_dialog.rs`):
  - [x] Строгое соответствие канону `DialogKit`: `DialogShell::standard(560, 430)`, `VariablePicker`, кнопки переноса `TransferButton`, поддиалоги `SubdialogSidebar` (Statistics, Plots, Options), нижняя панель `DialogActionBar`
  - [x] Интеграция в координатор диалогов `app_dialogs.rs` и главное окно `app.rs`
- [x] Главное меню:
  - [x] Активация пункта «Анализ -> Описательные статистики -> Исследовать...» (`menu-analyze-explore`) в `app_menus/analyze_etc.rs`
- [x] Локализация:
  - [x] Полные двуязычные каталоги Fluent в `locales/en.ftl` и `locales/ru.ftl` (100% покрытие, 0 сырых строк, 0 эмодзи)
- [x] Тестирование и Quality Gate:
  - [x] Модуль интерфейсных инвариантов `explore_invariants.rs`
  - [x] 100% успешный прогон всех 9 ступеней Quality Gate `cargo run -p xtask -j 1 -- ci`

## 42. Продвинутая регрессия и частные корреляции (Partial Correlations, Logistic Regression & Curve Estimation)
- [x] Доказательный Clean-Room аудит алгоритмов через Craft Anything MCP (`core_brief_write`, `core_plan_update`, `core_hypothesis_record` hyp_1..hyp_4 `supported`, `core_handoff_write`)
- [x] L2 `statcraft_algo`:
  - [x] Модуль частных корреляций `partial_corr/` (матрица ковариаций Шура, расчет $r, p, df = N - k - 2$, режимы Listwise и Pairwise, Golden Oracle vs SciPy с расхождением $< 10^{-12}$)
  - [x] Модуль бинарной логистической регрессии `logistic/` (IRLS / Newton-Raphson, адаптивное диагональное демпфирование Фишера, критерий Вальда, $\text{Exp}(B)$ 95% ДИ, псевдо-$R^2$ Кокса-Снелла и Нагелькерке, критерий согласия Хосмера-Лемешова, матрица классификации $2 \times 2$, Golden Oracle vs Statsmodels)
  - [x] Модуль оценки кривых `curvefit/` (6 канонических моделей: Linear, Logarithmic, Inverse, Quadratic, Power, Exponential с расчетом $R^2, F, df_1, df_2, p, b_0, b_1, b_2$, Golden Oracle vs SciPy/NumPy)
- [x] L5 `statcraft_syntax`:
  - [x] Команды движка `PartialCorrCommand` (`stat.correlations.partial`), `LogisticRegressionCommand` (`stat.regression.logistic`), `CurveFitCommand` (`stat.regression.curve`)
  - [x] Парсеры синтаксиса и мапперы: регистрация составных имен `PARTIAL CORR`, `LOGISTIC REGRESSION`, `CURVEFIT` в `parser.rs` и маппинг в `mappers/advanced_regression.rs`
  - [x] Интеграционные тесты `advanced_regression_tests.rs`
- [x] L4/L6 `statcraft_app::views::output_view`:
  - [x] Представления `OutputItemKind::PartialCorrelations`, `OutputItemKind::LogisticRegression`, `OutputItemKind::CurveFit`
  - [x] Сводные таблицы Pivot Tables: `tables_partial_corr.rs`, `tables_logistic.rs`, `tables_curvefit.rs` с поддержкой экспорта в TSV, TXT, HTML и документа отчета
  - [x] Тесты интерфейсных инвариантов `advanced_regression_output_invariants.rs`
- [x] UI диалоги на базе DialogKit:
  - [x] `PartialCorrDialog` (`dialogs/partial_corr_dialog.rs`): выбор переменных и контролируемых факторов, переключатель одно/двустороннего теста, нулевой порядок
  - [x] `LogisticRegressionDialog` (`dialogs/logistic_regression_dialog.rs`): зависимая бинарная переменная, список ковариат, порог классификации, ДИ, критерий Хосмера-Лемешова
  - [x] `CurveFitDialog` (`dialogs/curvefit_dialog.rs`): зависимая и независимая переменные, выбор комбинации из 6 моделей, константа в уравнении
  - [x] Интеграция в `app_dialogs_advanced.rs` и главное окно `app.rs`
  - [x] Тесты интерфейсных инвариантов `advanced_regression_dialog_invariants.rs`
- [x] Главное меню:
  - [x] Активация пунктов «Анализ -> Корреляции -> Частные...» (`menu-analyze-correlate-partial`), «Анализ -> Регрессия -> Бинарная логистическая...» (`menu-analyze-regression-binary`), «Анализ -> Регрессия -> Оценка кривых...» (`menu-analyze-regression-curve`)
- [x] Локализация:
  - [x] 100% двуязычное покрытие в `locales/en.ftl` и `locales/ru.ftl` (0 сырых строк, 0 эмодзи, Vector-First UI)
- [x] Тестирование и Quality Gate:
  - [x] 100% успешный прогон всех 9 ступеней Quality Gate `cargo run -p xtask -j 1 -- ci` (fmt, clippy, layers, file-guard [355 файлов], design-guard, workspace tests, parity, corpus, test-undo).

## 43. Общая линейная модель (General Linear Model: UNIANOVA, MANOVA & Repeated Measures)
- [x] Доказательный Clean-Room аудит алгоритмов через Craft Anything MCP (`core_brief_write`, `core_plan_update`, `core_hypothesis_record`, `core_handoff_write`)
- [x] L2 `statcraft_algo::glm`:
  - [x] Архитектура типов и контрастов: `types.rs`, генератор матриц плана `design_matrix.rs` с контрастами отклонений (deviation sum-to-zero) и псевдообращением OLS через Jacobi SVD
  - [x] Вычисление сумм квадратов типа III: `type3_ss.rs` через маргинальную редукцию моделей, $F$-критерии и частную эта-квадрат ($\text{Partial } \eta^2$)
  - [x] Одномерный анализ `univariate.rs`: `UNIANOVA` (ANCOVA с факторами, ковариатами и взаимодействиями $A \times B$, таблица Between-Subjects Effects, $R^2$ и Adjusted $R^2$, Golden Oracle vs Statsmodels)
  - [x] Многомерный спектральный анализ `eigen.rs`: обобщенное спектральное разложение $H v = \lambda E v$ симметризованной пары $E^{-1/2} H E^{-1/2}$ и 4 критерия (Wilks' $\Lambda$, Pillai's Trace, Hotelling-Lawley Trace, Roy's Largest Root с аппроксимацией Рао)
  - [x] Многомерный анализ `multivariate.rs`: `GLM Multivariate` (MANOVA) для $p \ge 2$ зависимых переменных с вычислением матриц SSCP гипотез и ошибки
  - [x] Анализ повторных измерений `repeated.rs`: `GLM Repeated Measures` с матрицей контрастов Хельмерта, критерием сферичности Мокли ($W, \chi^2, df, p$) и поправками Гринхауса-Гейссера ($\hat{\varepsilon}$), Хюйнха-Фельдта ($\tilde{\varepsilon}$) и нижней границы
- [x] L5 `statcraft_syntax`:
  - [x] Команды движка: `UnivariateGlmCommand` (`stat.glm.univariate`), `MultivariateGlmCommand` (`stat.glm.multivariate`), `RepeatedGlmCommand` (`stat.glm.repeated`)
  - [x] Парсеры синтаксиса и мапперы: регистрация составных имен `UNIANOVA`, `GLM`, `GLM REPEATED` в `parser.rs` и маппинг в `mappers/glm.rs`
  - [x] Интеграционные тесты `glm_syntax_tests.rs` и 100% обратимость в `undo_redo_matrix_tests`
- [x] L4/L6 `statcraft_app::views::output_view`:
  - [x] Представления `OutputItemKind::UnivariateGlm`, `OutputItemKind::MultivariateGlm`, `OutputItemKind::RepeatedGlm`
  - [x] Модуль сводных таблиц `tables_glm/`: декомпозиция на `mod.rs`, `univariate.rs`, `multivariate.rs`, `repeated.rs` (все файлы $\le 500$ строк)
  - [x] Отрисовка Pivot Tables: Between-Subjects Effects, Multivariate Tests (4 критерия), Mauchly's Test of Sphericity, Tests of Within-Subjects Effects
  - [x] Поддержка экспорта в TSV (`univariate_glm_to_tsv`, `multivariate_glm_to_tsv`, `repeated_glm_to_tsv`)
- [x] UI диалоги на базе DialogKit:
  - [x] `UnivariateGlmDialog`: зависимая переменная, фиксированные факторы, ковариаты, параметры модели (константа, дескрипторы, тест Ливиня)
  - [x] `MultivariateGlmDialog`: множественные зависимые переменные, фиксированные факторы, ковариаты
  - [x] `RepeatedGlmDialog`: имя внутригруппового фактора, список замеров повторных измерений
  - [x] Интеграция в координатор `app_dialogs_glm.rs` и главное окно `app.rs`
  - [x] Тесты интерфейсных инвариантов `glm_dialog_invariants.rs`
- [x] Главное меню:
  - [x] Активация пунктов «Анализ -> Общая линейная модель -> Одномерная...» (`menu-analyze-glm-univariate`), «Многомерная...» (`menu-analyze-glm-multivariate`), «Повторные измерения...» (`menu-analyze-glm-repeated`)
- [x] Локализация:
  - [x] 100% двуязычное покрытие в `locales/en.ftl` и `locales/ru.ftl` (0 сырых строк, 0 эмодзи, Vector-First UI, прохождение `i18n_lint_test`)
- [x] Тестирование и Quality Gate:
  - [x] 100% успешный прогон всех 9 ступеней Quality Gate `cargo run -p xtask -j 1 -- ci` (fmt, clippy, layers, file-guard [378 файлов], design-guard, workspace tests, parity, corpus, test-undo).

## 44. Кластерный анализ, отчёты и анализ выживаемости (Feature 003: QUICK CLUSTER, CODEBOOK, SUMMARIZE, KM, SURVIVAL)
- [x] Доказательный Clean-Room аудит алгоритмов по открытому руководству IBM SPSS Algorithms Guide и верификация формул (K-Means Maximin, Greenwood SE, Actuarial Life Tables)
- [x] L2 `statcraft_algo`:
  - [x] K-Means кластеризация `cluster_kmeans.rs`: инициализация центров методом максимального разброса (Maximin), итеративный цикл пересчета, расстояния между центрами, ANOVA-таблица $F$-критериев со значимостью Фишера
  - [x] Кодовая книга словаря датасета `codebook_core.rs`: извлечение метаданных переменных, частоты и проценты для категориальных шкал, описательные статистики (Mean, StdDev, Median, Quartiles via HAVERAGE) для количественных
  - [x] Итоги по наблюдениям `summarize_core.rs`: построчный листинг наблюдений, ограничение числа строк (`case_limit`), группировка по дискретным факторам, расчет промежуточных итогов (Subtotals) и общего итога (Total)
  - [x] Функция выживаемости Каплана-Мейера `survival_km.rs`: расчет функции дожития $S(t)$, стандартная ошибка Гринвуда, медианное и среднее время дожития с 95% ДИ, лог-ранговый тест Мантеля-Кокса ($\chi^2, df, p$) для сравнения страт
  - [x] Актуарные таблицы смертности / дожития `survival_lifetables.rs`: разбиение на временные интервалы $[t_k, t_{k+1})$, эффективный объем под риском $n'_k$, условные вероятности смерти $q_k$ и дожития $p_k$, кумулятивная вероятность $P_k$, плотность вероятности $f_k$, коэффициент риска $h_k$
  - [x] Тесты Золотого Оракула `feature_003_golden_oracle.rs` со сверкой против SciPy и Lifelines с точностью $\le 10^{-6}$
- [x] L5 `statcraft_syntax`:
  - [x] Команды движка: `KMeansClusterCommand` (`stat.cluster.kmeans`), `CodebookCommand` (`stat.reports.codebook`), `SummarizeCommand` (`stat.reports.summarize`), `KaplanMeierCommand` (`stat.survival.km`), `LifeTablesCommand` (`stat.survival.lifetables`)
  - [x] Динамическое создание колонок `QCL_1` (Cluster Membership) и `QCL_2` (Distance from Cluster Center) в `DataSet` и `VariableDictionary` при выполнении K-Means со 100% транзакционной обратимостью (Undo/Redo)
  - [x] Парсеры синтаксиса и мапперы: распознавание составных команд `QUICK CLUSTER`, `LIFE TABLES`, `KAPLAN MEIER` в `parser.rs` и маппинг в `mappers/pkg3.rs`
  - [x] Интеграционные тесты `feature_003_syntax_tests.rs`
- [x] L4/L6 `statcraft_app::views::output_view`:
  - [x] Представления `OutputItemKind::KMeans`, `OutputItemKind::Codebook`, `OutputItemKind::Summarize`, `OutputItemKind::KaplanMeier`, `OutputItemKind::LifeTables`
  - [x] Модули сводных таблиц Pivot Tables: `tables_cluster.rs`, `tables_reports.rs`, `tables_survival.rs` с поддержкой экспорта в TSV
  - [x] Архитектурная декомпозиция `views/output_view/mod.rs` и выделение `view.rs` со строгим соблюдением лимита $\le 500$ строк на файл (Rule 12.1)
- [x] UI диалоги на базе DialogKit:
  - [x] `KMeansDialog` (`dialogs/kmeans_dialog.rs`): выбор переменных анализа, число кластеров $K$, максимальное число итераций, флаги сохранения новых переменных и вывода ANOVA
  - [x] `CodebookDialog` (`dialogs/codebook_dialog.rs`): выбор переменных датасета, флаги вывода информации о словаре, статистик и частот
  - [x] `SummarizeDialog` (`dialogs/summarize_dialog.rs`): переменные отчета, группирующие факторы, лимит первых $N$ наблюдений, флаги вывода дескрипторов
  - [x] `KaplanMeierDialog` (`dialogs/survival_km_dialog.rs`): переменная времени, переменная статуса с кодом события, фактор стратификации
  - [x] `LifeTablesDialog` (`dialogs/survival_lifetables_dialog.rs`): переменная времени, начальное значение, конечное значение, ширина интервала, переменная статуса с кодом события
  - [x] Координатор диалогов `app_dialogs_pkg3.rs` и интеграция в главное окно `app.rs`
  - [x] Тесты интерфейсных инвариантов `feature_003_pkg3_invariants.rs`
- [x] Главное меню:
  - [x] Активация пунктов «Анализ -> Отчёты -> Кодовая книга...» (`menu-analyze-reports-codebook`), «Анализ -> Отчёты -> Итоги по наблюдениям...» (`menu-analyze-reports-summarize`), «Анализ -> Классификация -> Кластерный анализ методом K-средних...» (`menu-analyze-classify-kmeans`), «Анализ -> Выживаемость -> Таблицы дожития...» (`menu-analyze-survival-lifetables`), «Анализ -> Выживаемость -> Каплана-Мейера...» (`menu-analyze-survival-km`)
- [x] Локализация:
  - [x] 100% двуязычное покрытие в `locales/en.ftl` и `locales/ru.ftl` (пройден `i18n_lint_test`, 0 сырых строк, 0 эмодзи, Vector-First UI)
- [x] Тестирование и Quality Gate:
  - [x] 100% успешный прогон всех 9 ступеней Quality Gate `cargo run -p xtask -j 1 -- ci` (fmt, clippy [0 warnings], layers, file-guard [402 файла], design-guard, workspace tests, parity, corpus, test-undo).

## 45. Управление жизненным циклом вкладок и защита основных экранов (Feature 005)
- [x] Защита ключевых экранов данных от случайного закрытия:
  - [x] Реализация трейта `TabViewer::closeable`: для `DataView` и `VariableView` возвращается `false` (скрыта кнопка `[x]`); для `OutputViewer` и `SyntaxEditor` возвращается `true`.
  - [x] Изолированная функция `is_tab_closeable` для валидации логики закрытия.
- [x] Гарантированное восстановление и активация вкладок (`ensure_dock_tab_open` и `StatCraftApp::ensure_tab_open`):
  - [x] Активация существующей вкладки при её наличии в дереве `DockState`.
  - [x] Динамическое переоткрытие закрытой вкладки `OutputViewer` (боковой сплит справа `split_right` с коэффициентом 0.35 или `push_to_first_leaf`).
  - [x] Динамическое переоткрытие закрытой вкладки `SyntaxEditor` (`push_to_first_leaf`).
  - [x] Исключение дублирования вкладок при повторных вызовах.
- [x] Глобальные горячие клавиши (бесконфликтный отраслевой стандарт `Alt+1..4` без коллизий с Windows / ZoomIt):
  - [x] `Alt+1` $\to$ Data View (`Tab::DataView`)
  - [x] `Alt+2` $\to$ Variable View (`Tab::VariableView`)
  - [x] `Alt+3` $\to$ Output Viewer (`Tab::OutputViewer`)
  - [x] `Alt+4` $\to$ Syntax Editor (`Tab::SyntaxEditor`)
- [x] Сквозная интеграция:
  - [x] Меню «Окно» (`analyze_etc.rs`): вызовы `ensure_tab_open` для всех 4 вкладок с отображением подсказок `Alt+1..4`.
  - [x] Создание и открытие синтаксиса (`file_edit_view.rs`, `app_session.rs`): гарантированное отображение редактора.
  - [x] Запуск пакетного синтаксиса и фрагментов (`RunAll`, `RunSelection`): автоматическое переоткрытие `OutputViewer`.
  - [x] MCP Bridge (`bridge_service.rs`): метод `ui.set_active_tab` восстанавливает закрытые вкладки через `ensure_dock_tab_open`.
- [x] Интеграционные тесты и Quality Gate:
  - [x] Интеграционный тест `crates/statcraft_app/tests/tab_lifecycle_invariants.rs` (5 тестов: неснимаемость базовых экранов, закрываемость вспомогательных, roundtrip закрытия/восстановления, отсутствие дубликатов).
  - [x] 100% успешный прогон всех ступеней `cargo run -p xtask -j 1 -- ci` (fmt, clippy, file-guard [404 файла], design-guard, тесты воркспейса, undo matrix).

## 46. Виртуализированная 2D-сетка из 4 квадрантов и интерактивный ресайз колонок (Feature 006)
- [x] Изолированный модульный пакет `crates/statcraft_app/src/grid/` (Safe Rust, <= 500 строк на модуль):
  - [x] `LineIndex` (`line_index.rs`): префиксные суммы для $O(\log N)$ поиска строк/колонок, компактный `Uniform` (0 аллокаций памяти), visible range и хит-тестинг границ.
  - [x] `Geo` (`geo.rs`): расчет геометрии 4 квадрантов (`quad_top_left`, `quad_top_right`, `quad_bottom_left`, `quad_center`), трансформации координат ячеек, видимые видовые экраны и `scroll_to_cell`.
  - [x] `QuadrantPaint` (`quadrant.rs`): изолированная отрисовка 4 квадрантов с раздельным клиппингом, векторные иконки шкал, зачеркивание неактивных строк и вертикальная направляющая линия ресайза.
  - [x] `GridInteract` (`interact.rs`): хит-тестинг границ колонок, курсор `ResizeHorizontal`, drag-and-drop перетаскивание `GridDrag::ResizeCol`, расчет auto-fit по двойному клику и 2D-скроллинг с ползунками.
  - [x] `render_grid` (`mod.rs`): единая координирующая функция рендеринга и обработки событий `GridEvent`.
- [x] Интеграция в `DataView` (`views/data_view.rs`):
  - [x] Замена вложенных `ScrollArea` на единую 4-квадрантную сетку с замороженными заголовками строк и столбцов.
  - [x] Двусторонняя синхронизация ширины столбцов: пиксели в `DataViewState.col_widths` и символы в `VariableMetadata.columns` (SPSS паритет для сохранения в `.sav`).
  - [x] Auto-fit ширины столбца по двойному клику с анализом имени переменной и содержимого ячеек.
- [x] Тестирование и Quality Gate:
  - [x] Юнит-тесты модулей `line_index`, `geo`, `quadrant`, `interact`;
  - [x] Интеграционный тест `crates/statcraft_app/tests/grid_quadrant_invariants.rs` (изоляция квадрантов, масштаб 100k элементов, паритет ширины SPSS, рендеринг датасета);
  - [x] 100% успешный прогон всех тестов воркспейса `cargo test --workspace -j 1`, `xtask layers` и `xtask file-guard` (410 файлов).

## 47. Прямоугольное выделение, навигация, буфер обмена TSV и автозаполнение (Feature 007)
- [x] Модель диапазона ячеек и фокуса (`selection.rs`):
  - [x] `CellRange` с нормализацией границ `from_corners`, подсчетом строк/столбцов и хит-тестингом `contains_cell`.
  - [x] `GridSelection` с активной ячейкой, ячейкой привязки (anchor), выделением полных строк/колонок и всей таблицы.
  - [x] Векторная отрисовка `paint_selection`: полупрозрачная фоновая заливка диапазона, акцентная рамка 1.5 px со скруглением и угловой маркер протяжки `Fill Handle` ($6 \times 6$ px).
- [x] Клавиатурная навигация и шорткаты (`keyboard.rs`):
  - [x] Клавиши со стрелками (Up, Down, Left, Right) с автоматическим скроллингом к ячейке.
  - [x] Клавиши `Tab` (вправо / Shift+Tab влево) и `Enter` (вниз / Shift+Enter вверх).
  - [x] Навигация `Home` / `End` (начало/конец строки), `Ctrl+Home` / `Ctrl+End` (первая/последняя ячейка датасета).
  - [x] Расширение диапазона с удержанием `Shift` + стрелки.
  - [x] Выделение всего датасета по `Ctrl+A`.
  - [x] Direct Typing: мгновенное открытие инлайн-редактора ячейки при вводе алфавитно-цифровых символов без предварительного клика или Enter.
  - [x] Очистка ячеек диапазона по `Delete` / `Backspace`.
- [x] Инлайн-редактор ячейки (`editor.rs`):
  - [x] Плавающий текстовый ввод `TextEdit::singleline` поверх целевой ячейки с фокусом.
  - [x] Сохранение значения и переход по Enter/Tab/стрелкам или отмена по Escape.
- [x] Прямоугольный буфер обмена TSV/CSV (`clipboard.rs`):
  - [x] Сериализация диапазона `serialize_range_to_tsv` с экранированием кавычек, табуляций и переносов строк.
  - [x] Парсинг TSV-матрицы `parse_tsv_matrix`.
  - [x] Вставка матрицы ячеек `paste_matrix_into_dataset` в позицию курсора с автоматическим расширением строк датасета.
  - [x] Серийное автозаполнение `apply_auto_fill`: линейная арифметическая прогрессия для числовых серий и циклическое повторение для текстовых/категориальных значений (строго без аллокации новых строк в DataSet).
- [x] Интерактивные манипуляции (`interact.rs`):
  - [x] Выделение протяжкой мыши (Marquee Drag Selection).
  - [x] Канон SPSS: выделение всегда Read-Only, маркер Fill Handle отключен в UI для исключения случайной порчи данных.
- [x] Тестирование и Quality Gate:
  - [x] Юнит-тесты модулей `selection`, `keyboard`, `clipboard`, `editor`;
  - [x] Интеграционный тест `crates/statcraft_app/tests/grid_selection_clipboard_invariants.rs` (границы и нормализация `CellRange`, TSV copy/paste roundtrip, протяжка числовых серий, жизненный цикл редактора);
  - [x] 100% успешный прогон `cargo run -p xtask -j 1 -- ci` (fmt, clippy, file-guard [415 файлов], design-guard, 100% тестов воркспейса, матрица Undo/Redo 41 команды).

## 48. Колоночный автофильтр, быстрая сортировка и контроль целостности данных (Feature 008)
- [x] L1 `statcraft_core::dataset`:
  - [x] Метод `unique_values(&self, col_idx, show_labels)` для извлечения упорядоченного списка уникальных значений столбца и наличия пустых ячеек.
  - [x] Предикат `is_row_matched_by_filter(...)` для фильтрации наблюдений по разрешенным значениям.
- [x] L-Cross `statcraft_i18n`:
  - [x] Двуязычные ключи фильтра и проверки целостности в `locales/en.ftl` и `locales/ru.ftl` (`grid-filter-sort-asc`, `grid-filter-sort-desc`, `grid-filter-search`, `grid-filter-select-all`, `grid-filter-blanks`, `grid-filter-apply`, `grid-filter-clear`, `grid-menu-filter-value`, `grid-menu-clear-filter`, `grid-integrity-num-stored-as-text`, `grid-integrity-convert`).
- [x] L6 `statcraft_app::grid`:
  - [x] `integrity.rs`: эвристика `is_number_stored_as_text` для детекции чисел, сохраненных строками, конвертер `convert_text_to_number`, векторный зеленый треугольник предупреждения `paint_integrity_badge` в стиле Excel и хит-тест `hit_test_integrity_badge`.
  - [x] `filter_menu.rs`: состояние `GridFilterState`, векторная кнопка фильтра/воронки `paint_filter_button`, плавающий выпадающий попап `render_filter_popup` (быстрая сортировка ASC/DESC, живой поиск, выбор всех значений, фильтрация пустых ячеек, очистка и применение).
  - [x] `context_menu.rs`: состояние `GridContextMenuState`, плавающее контекстное меню ячейки `render_context_menu` («Фильтровать по значению этой ячейки», «Очистить фильтр», сортировка, Copy/Paste).
  - [x] `quadrant.rs`: интеграция кнопок фильтра в шапку колонок, векторных маркеров предупреждений в ячейки и приглушенной подсветки текста неактивных строк.
  - [x] `interact.rs`: хит-тестинг кнопок фильтра в заголовках, вызов контекстного меню по правой кнопке мыши и клик по маркеру предупреждения для мгновенной конвертации типа ячейки.
- [x] Интеграция в `DataView` (`views/data_view.rs`) и координатор приложения (`app.rs`):
  - [x] Хранение состояний фильтров `filter_state` и контекстного меню `context_menu_state` в `DataViewState`.
  - [x] Визуальное перечеркивание неактивных номеров строк и приглушение цвета текста отфильтрованных наблюдений (полное соответствие канону SPSS `FILTER BY`).
  - [x] Связывание быстрой сортировки из меню фильтра и контекстного меню с командой `data.sort_cases` через `CommandBus` с Undo/Redo и протоколом синтаксиса `SORT CASES BY ...`.
- [x] Тестирование и Quality Gate:
  - [x] Юнит-тесты в `dataset/tests.rs`, `integrity.rs`, `filter_menu.rs`, `context_menu.rs`;
  - [x] Интеграционный тест `crates/statcraft_app/tests/grid_autofilter_integrity_invariants.rs` (маска фильтрации, детекция и конвертация строковых чисел, headless-рендеринг выпадающих попапов);
  - [x] 100% успешный прогон Quality Gate `cargo run -p xtask -j 1 -- ci` (fmt, clippy, file-guard [419 файлов], design-guard, layers, 100% тестов воркспейса, матрица обратимости Undo/Redo 41 команды).

## 49. Стабилизация табличной сетки и авторелиз v0.4.0
- [x] Исправление дефектов отображения и интерактивности:
  - [x] BUG-010: Перенос полупрозрачной заливки выделения под текст ячеек в `paint_data_cells`, ликвидировано перекрытие текста.
  - [x] BUG-011: Двойной клик по ячейке мгновенно открывает плавающий `TextEdit` с фокусом и кареткой.
  - [x] BUG-012: Локализация хит-теста ресайза столбцов строго в шапку `quad_top_right`.
- [x] Официальный релиз v0.4.0:
  - [x] Синхронизация версий в 5 файлах (`Cargo.toml`, `installer.iss`, `control`, `Info.plist`, `release.yml`).
  - [x] Формирование `RELEASE_NOTES.md` и `CHANGELOG.md`.
  - [x] Публикация тега `v0.4.0` и запуск GitHub Actions CI/CD релизной сборки.
- [x] Глобальная актуализация документации:
  - [x] `FEATURES.md` и `FEATURES_RU.md`: синхронизация всех аналитических процедур, сетевого моста MCP, тем оформления и 4-квадрантной сетки.
  - [x] `ROADMAP.md` и `ROADMAP.en.md`: актуализация выполненных этапов (M1–M14).
  - [x] `README.md` и `README.ru.md`: описание возможностей грида и обновление бейджа на 300+ тестов.
  - [x] `BUGS_AND_ISSUES.md`: регистрация дефектов BUG-010, BUG-011, BUG-012 со статусом RESOLVED.
  - [x] `CHRONICLE.md` и `CHRONICLE.en.md`: синхронные записи этапа 80.

## 56. Мультиномиальная логистическая регрессия (Feature 009 — NOMREG)
- [x] L2 `statcraft_algo::multinomial_logistic` (Safe Rust, <= 500 строк на файл):
  - [x] `types.rs`: структуры `MultinomialLogisticOptions`, `ReferenceCategory`, `MultinomialCaseProcessingSummary`, `ModelFittingInfo`, `GoodnessOfFit`, `PseudoRSquared`, `LikelihoodRatioTestRow`, `ParameterEstimateRow`, `MultinomialClassificationTable`, `MultinomialLogisticResult`.
  - [x] `engine.rs`: итератор Ньютона-Рафсона (Fisher Scoring), стабильный log-sum-exp, расчет матрицы Фишера, регуляризация Риджа $\lambda = 10^{-6}$ при вырождении Гессиана (Never Crash), бэктрекинг-поиск шага.
  - [x] `diagnostics.rs`: тесты отношения правдоподобия (LRT) для каждого эффекта, субпопуляционные группировки для критериев согласия Пирсона и девианса, статистики Вальда, Exp(B) с 95% ДИ, матрица классификации.
  - [x] `mod.rs`: единая точка входа `compute_multinomial_logistic` с поддержкой факторов (Indicator dummy coding с базовым последним уровнем) и непрерывных ковариат.
  - [x] `tests.rs`: модульные тесты 3-категориальных моделей, кодирования факторов, выбора референтной категории (First/Last), обработки крайних случаев.
- [x] L5 `statcraft_syntax`:
  - [x] `MultinomialLogisticCommand` (`stat.regression.multinomial`): JSON-схема `specs()`, предикат доступности `enabled()`, выполнение с интеграцией в шину `CommandBus` и Output Viewer, генератор канонического синтаксиса `NOMREG ... (BASE=LAST) BY ... WITH ...`.
  - [x] Синтаксический парсер `NOMREG`: маппер подкоманд `MODEL`, `CRITERIA`, `PRINT`, поддержка параметров `(BASE=LAST)` и `(BASE=FIRST)`.
  - [x] Тесты парсера и выполнения команды в `advanced_regression_tests.rs`.
- [x] L6 `statcraft_app`:
  - [x] `multinomial_logistic_dialog.rs`: канонический диалог на базе `DialogKit` ($560 \times 430$ px), `DialogShell::standard`, `VariablePicker`, адаптивные кнопки переноса `TransferButton`, поддиалог выбора референтной категории `ReferenceCategoryDialog`, поддиалог статистик `StatisticsDialog`, нижняя панель `DialogActionBar`.
  - [x] `tables_multinomial_logistic.rs`: рендеринг 7 аутентичных сводных таблиц SPSS Pivot Tables (Case Processing Summary, Model Fitting Information, Goodness-of-Fit, Pseudo R-Square, Likelihood Ratio Tests, Parameter Estimates, Classification Table) и экспорт в TSV.
  - [x] Меню `Анализ ▶ Регрессия ▶ Мультиномиальная логистическая...`: активация пункта с привязкой вызова диалога.
  - [x] `app.rs`, `app_dialogs_advanced.rs`, `status_message.rs`: интеграция диалога, действий Run и Paste в Syntax Editor, статусных сообщений.
- [x] i18n & Quality Gate:
  - [x] Синхронизация семантических ключей в `locales/en.ftl` и `locales/ru.ftl` (100% покрытие, 0% сырой кириллицы).
  - [x] Инвариантные тесты диалогов и рендеринга вывода (`advanced_regression_dialog_invariants.rs`, `multinomial_logistic_tests.rs`).
  - [x] 100% успешное прохождение всех проверок: `file-guard` (428 файлов), `design-guard`, `layers`, `parity`.

## 57. Защита диалогов логистической регрессии и дерева вывода (BUG-013)
- [x] Динамическая валидация мощности зависимой переменной в бинарной логистической регрессии (`logistic_regression_dialog.rs`):
  - [x] Подсчет уникальных значений `dep_distinct_count` по активным строкам выборки;
  * Индикаторы: `(бинарная, 2 значения)` (зеленый) и `({count} значений — требуется ровно 2)` (оранжевый);
  * Блокировка кнопок `OK` и `Вставить` при $count \ne 2$ с предупреждением и рекомендацией использовать мультиномиальную регрессию.
- [x] Валидация мощности в мультиномиальной логистической регрессии (`multinomial_logistic_dialog/mod.rs`):
  - [x] Проверка числа категорий $\ge 2$, блокировка при пустой колонке или константе.
- [x] Защита от сырых ключей в Outline Tree (`views/output_view/model.rs`):
  - [x] Безопасный fallback `output-frequencies-title` $\to$ `output-frequencies` («Частоты») при открытии сессий.
- [x] Тестирование и Quality Gate:
  - [x] Тест `test_logistic_regression_validation_with_cases` в `advanced_regression_dialog_invariants.rs`.
  - [x] 100% прохождение `xtask design-guard`, `xtask file-guard` (429 файлов).

## 58. Регрессия пропорциональных рисков Кокса (Feature 010 — COXREG / Cox Proportional Hazards Regression)
- [x] L2 `statcraft_algo::cox` (Safe Rust, <= 500 строк на файл):
  - [x] `types.rs`: структуры `TieMethod` (`Breslow`, `Efron`), `CoxRegressionOptions`, `CoxDataPoint`, `CoxCaseSummary`, `OmnibusTestsResult`, `CoxVariableInEquation`, `BaselineSurvivalPoint`, `CoxRegressionResult`.
  - [x] `engine.rs`: итератор Ньютона-Рафсона с бэктрекинг-поиском шага, вычисление логарифма частичного правдоподобия, вектора градиента (Score) и информационной матрицы Фишера, обработка связанных исходов (Breslow и Efron), поддержка множественной стратификации (Strata).
  - [x] `baseline.rs`: базовые функции риска $\hat{H}_{0s}(t)$, базовой выживаемости $\hat{S}_{0s}(t)$ и выживаемости на среднем ковариат $\hat{S}(t \mid \bar{x})$.
  - [x] `mod.rs`: единая точка входа `compute_cox_regression`, валидация данных, аппроксимация Аклема для квантилей нормального распределения.
  - [x] `tests.rs`: бенчмарк Golden Oracle против `lifelines.CoxPHFitter`, проверка методов Breslow и Efron, многослойная стратификация, валидация граничных условий.
- [x] L5 `statcraft_syntax`:
  - [x] `CoxRegressionCommand` (`stat.survival.cox`): схема `specs()`, предикат доступности `enabled()`, выполнение с интеграцией в шину `CommandBus` и Output Viewer, откат `undo()`, генератор канонического синтаксиса `COXREG ... /STATUS=... /STRATA=... /METHOD=ENTER ... /CRITERIA=...`.
  - [x] Синтаксический парсер `COXREG` в `pkg3.rs`: маппер подкоманд `STATUS`, `STRATA`, `METHOD`, `CRITERIA`, `PRINT`, `PLOT`.
  - [x] Регистрация в `commands/mod.rs`, `bus.rs` и `lib.rs`.
- [x] L6 `statcraft_app`:
  - [x] `cox_regression_dialog.rs`: канонический диалог на базе `DialogKit` ($560 \times 430$ px), `DialogShell::standard`, `VariablePicker`, адаптивные кнопки переноса `TransferButton`, селектор метода обработки связей (Breslow/Efron), доверительный интервал для `Exp(B)`, поддиалог задания события `EventStatusDialog`, поддиалог настроек `CoxOptionsDialog`, нижняя панель `DialogActionBar`.
  - [x] `tables_cox.rs`: рендеринг аутентичных сводных таблиц SPSS Pivot Tables (Case Processing Summary, Omnibus Tests of Model Coefficients, Variables in the Equation с коэффициентами, ошибками, статистикой Вальда, p-value, Exp(B) и 95% ДИ, Baseline Survival Table) и векторный график кривой выживаемости на средних ковариат на базе `egui_plot`. Экспорт результатов в TSV.
  - [x] Меню `Анализ ▶ Выживаемость ▶ Регрессия Кокса...`: активация пункта с привязкой вызова диалога.
  - [x] `app.rs`, `app_dialogs_pkg3.rs`, `status_message.rs`: интеграция диалога, действий Run и Paste в Syntax Editor, статусных сообщений.
- [x] i18n & Quality Gate:
  - [x] Синхронизация семантических ключей в `locales/en.ftl` и `locales/ru.ftl` (100% покрытие, 0% сырой кириллицы).
  - [x] Инвариантные тесты диалогов и рендеринга вывода (`output_viewer_cox_invariants.rs`).
  - [x] 100% успешное прохождение всех проверок: `file-guard` (438 файлов), `design-guard`, `layers`, `parity`, `cargo test --workspace -j 1`, `ci`.

## 59. Мультифайловый воркспейс, вкладки датасетов и Memory Guard (Feature 010 — Multi-File Workspace, Dataset Tabs & Memory Guard)
- [x] L0 `statcraft_io::stc_codec`:
  - [x] Поддержка контейнера `StcContainer::with_workspace` и записей `StcDatasetRecord` с сохранением полной обратной совместимости с одиночными файлами `.stc`.
  - [x] Сохранение и извлечение нескольких датасетов с метаданными и состоянием в рамках единого `.stc`-файла.
- [x] L1 `statcraft_core`:
  - [x] `statcraft_core::workspace`: структуры `DocumentSession`, `WorkspaceManager`, переключение активных наборов, именование `DataSet1`, `DataSet2`..., отслеживание dirty-состояния (`is_dirty`).
  - [x] `statcraft_core::compression`: сжатие в памяти без потерь — `NullBitmask` (1 бит на пропуск), `DictionaryEncodedColumn` (словарь строк), `LosslessCompressedColumn` со строгим сохранением `f64` без потери точности и без приведения к f32.
  - [x] Методы `undo` и `redo` в `DeltaBuffer` с отслеживанием контрольной длины стека.
- [x] L5 `statcraft_syntax`:
  - [x] Команды `DATASET NAME`, `DATASET ACTIVATE`, `DATASET CLOSE`, `DATASET DISPLAY` в `statcraft_syntax::commands::dataset`.
  - [x] Парсер канонического синтаксиса SPSS `.sps` и интеграция в `CommandBus`.
- [x] L6 `statcraft_app`:
  - [x] `statcraft_app::system_memory`: кроссплатформенный Memory Guard (Windows `GlobalMemoryStatusEx`, Linux `/proc/meminfo`, macOS `sysctl`, WASM), префлайт-чеки для тяжелых пакетов (>10 файлов или >250 МБ) с предупреждением при низкой доступной RAM (<500 МБ или <15%).
  - [x] `views::document_tabs`: векторная панель вкладок датасетов (высота 28 px, Vector-First примитивы, без тофу и эмодзи, индикатор `*`, кнопка `✕`, шевроны `[ ‹ ] [ › ]`, кнопка `[ ➕ ]`).
  - [x] Шорткаты: `Ctrl+Tab` (след. датасет), `Ctrl+Shift+Tab` (пред. датасет), `Ctrl+W` (закрыть текущий датасет), `Ctrl+N` (новый датасет), `Ctrl+Shift+N` (новый проект).
  - [x] Декомпозиция `app.rs`: вынос логики воркспейса в `app_workspace.rs`, шорткатов в `app_shortcuts.rs`, просмотрщика дока в `app_tab_viewer.rs` (сокращение `app.rs` с 1100+ до ~800 строк).
  - [x] Поддержка сохранения активной вкладки (`Ctrl+S`) и полного проекта (`Ctrl+Shift+S`) в `app_session.rs`, диалог подтверждения сохранения при закрытии измененных вкладок.
- [x] i18n & Quality Gate:
  - [x] Полная локализация во Fluent (`locales/en.ftl`, `locales/ru.ftl`) без дубликатов ключей.
  - [x] 100% успешный прогон всех проверок: `xtask ci` (fmt, clippy с 0 warnings, layers, file-guard [454 файла], design-guard, workspace tests, parity, corpus, test-undo).

## 60. Цветовая идентификация датасетов и сквозная визуальная связка (Feature 011 — Dataset Color Identity & Cross-Pane Visual Linking)
- [x] L6 `statcraft_app::theme`:
  - [x] Детерминированный цветовой генератор `dataset_color(name)` на базе HSL золотого сечения (Golden Ratio HSL distribution, FNV-1a hash + golden angle $137.508^\circ$).
  - [x] Гарантия стабильности цвета при перезапусках, высокая контрастность и адаптивность к темной и светлой темам.
  - [x] Хелпер `color_with_alpha` для работы с полупрозрачными подложками и рамками.
  - [x] Unit-тесты детерминированности, дифференциации и альфа-канала (`theme::tests`).
- [x] L6 `statcraft_app::views::document_tabs`:
  - [x] Окрашивание векторной иконки таблицы `[田]` в индивидуальный цвет датасета `ds_color`.
  - [x] Индикаторная полоса активной вкладки цветом `ds_color`.
  - [x] Элегантная полупрозрачная фоновая подсветка активной вкладки (`color_with_alpha(ds_color, 25)`).
- [x] L6 `statcraft_app::views::output_view`:
  - [x] Поле `#[serde(default)] pub dataset_name: Option<String>` в структуре `OutputProcedure` с сохранением 100% обратной совместимости при десериализации старых файлов проектов.
  - [x] Синхронизация имени активного набора данных `active_dataset_name` в `OutputViewerState` и привязка к сессиям воркспейса.
  - [x] Векторные чипы-бейджи датасета в строках процедур дерева навигации (Outline Tree) с подсветкой активного контекста (Brushing & Linking).
  - [x] Векторная плашка-чип в ячейке `Active Dataset` таблицы «Примечания» (Notes) с рамкой и фоном соответствующего цвета.
- [x] Quality Gate & Definition of Done:
  - [x] 100% успешный прогон всех проверок `cargo run -p xtask -j 1 -- ci` (fmt, clippy с 0 warnings, layers, file-guard на 454 файла, design-guard, тесты воркспейса, corpus, test-undo).

## 61. Слияние файлов (Feature 012 — Merge Files: Add Cases & Add Variables)
- [x] L1 `statcraft_core::dataset::merge`:
  - [x] `types.rs`: структуры `AddCasesConfig`, `AddVarsConfig`, `MergeMethod` (OneToOneWithKeys, OneToManyTable, Sequential).
  - [x] `cases.rs`: процедура `add_cases` (вертикальное объединение строк, автосопоставление совместимых типов, индикатор источника `/IN=source01`, расширение строковых ширин, заполнение `RawValue::Null`).
  - [x] `vars.rs`: процедура `add_variables` (горизонтальное сопоставление по ключам `/BY`, табличное соответствие 1-к-N `/TABLE`, последовательное объединение по порядку строк, разрешение коллизий имен колонок).
  - [x] `source.rs`: универсальная загрузка вторичного датасета из файлов (`.sav`, `.zsav`, `.csv`, `.stc`, `.xlsx`) или открытых сессий воркспейса.
  - [x] Модульные тесты `dataset::merge::tests` с проверкой стабильности, слияния и отсутствия паник.
- [x] L5 `statcraft_syntax`:
  - [x] Типизированные команды `data.merge_cases` (`MergeCasesCommand`) и `data.merge_vars` (`MergeVarsCommand`) с регистрацией в `CommandBus`.
  - [x] Генерация канонического синтаксиса SPSS `ADD FILES` и `MATCH FILES` с поддержкой `/RENAME`, `/IN`, `/BY`, `/TABLE`, `/DROP`.
  - [x] Поддержка обратимости операций через `undo` в истории команд.
  - [x] Интеграция в парсер синтаксиса SPSS в `syntax_parser::mappers::data`.
- [x] L6 `statcraft_app`:
  - [x] Диалоговые окна `AddCasesDialog` и `AddVarsDialog` на базе `DialogKit` (`DialogShell::standard`, `VariablePicker`, `TransferButton`, `DialogActionBar`, `GroupBox`).
  - [x] `source_panel`: переиспользуемая панель выбора источника данных (открытый датасет из вкладок воркспейса или внешний файл через `rfd::FileDialog`).
  - [x] Выделенный модуль `app_dialogs_merge.rs` для диспетчеризации действий диалогов слияния с соблюдением лимитов строк.
  - [x] Активация пунктов меню `menu-data-merge-cases` и `menu-data-merge-vars` в `app_menus/data_transform.rs` со снятием метки `InDev`.
  - [x] Сообщения статуса `MergeCasesExecuted` и `MergeVarsExecuted` в статусной строке и логирование синтаксиса в Output Viewer.
- [x] i18n & Quality Gate:
  - [x] 100% покрытие локализацией во Fluent (`locales/en.ftl`, `locales/ru.ftl`) без сырого текста в UI.
  - [x] Прохождение проверок: `file-guard` (468 файлов в лимитах), `design-guard`, `layers`, `dialog_invariants`.

## 61. Модернизация сетки данных и устранение багов взаимодействия (Spec-Kit 013 / Grid UX Invariants)
- [x] Сохранение прямоугольного выделения при правом клике:
  - [x] Метод `GridSelection::contains(row, col)` сохраняет диапазон, если клик попал внутрь него;
  - [x] Контекстное меню адаптируется к размеру диапазона: «Копировать диапазон ({count} яч.)», «Очистить содержимое».
- [x] Защита от загрязнения набора данных фантомными строками (Zero Pollution Invariant):
  - [x] В `data_view.rs` при потере фокуса с пустым буфером (`trimmed.is_empty() && row >= ds.row_count()`) строка не аллоцируется в датасете;
  - [x] Заголовок строки добавления отображает каноническую звездочку `*` (SPSS/GridCraft).
- [x] Физическое удаление наблюдений через CommandBus:
  - [x] Клавиша `Delete` при выделении строк через заголовок (`is_full_rows`) физически удаляет наблюдения через L5 `data.delete_row` с уменьшением выборки $N$ и поддержкой `undo`;
  - [x] Клавиша `Delete` при обычном прямоугольном выделении ячеек очищает только значения в `Null`, сохраняя строки;
  - [x] Пункт контекстного меню «Удалить выбранные наблюдения ({count})».
- [x] Контекстные меню заголовков строк и колонок (`ContextMenuTarget`):
  - [x] `RowHeader`: «Вставить наблюдение выше» (`data.add_row`), «Удалить выбранные наблюдения» (`data.delete_row`), «Очистить значения в наблюдениях»;
  - [x] `ColHeader`: сортировка ASC/DESC, «Вставить переменную» (`var.add`), «Автоподбор ширины столбца»;
  - [x] Передача позиции клика `pos` в событиях `GridEvent::ContextMenuRow` и `ContextMenuCol`.
- [x] Устранение модальной ловушки (Click-Outside Dismissal):
  - [x] Флаг `just_opened` защищает от ложного закрытия в кадр открытия;
  - [x] Контекстное меню и всплывающее окно автофильтра (`render_filter_popup`) мгновенно закрываются при клике вне их прямоугольника без необходимости нажатия `Escape`.
- [x] Тестирование и Quality Gate:
  - [x] Интеграционный тестовый набор `crates/statcraft_app/tests/grid_selection_and_phantom_row_invariants.rs` (5 тестов);
  - [x] 100% прохождение `file-guard` (469 файлов), `design-guard`, `layers`, `cargo clippy --workspace -j 1 -- -D warnings`, `cargo fmt --check`.

## 62. Динамическая генерация случайных цветов датасетов и сохранение в .stc (Spec-Kit 015 — Dynamic Dataset Colors & Persistence)
- [x] L0 `statcraft_io::stc_codec`:
  - [x] Поле `pub tab_color: Option<String>` в `StcDatasetRecord` и `StcContainer` с `#[serde(default, skip_serializing_if = "Option::is_none")]`.
  - [x] Сохранение и загрузка цвета вкладок со 100% обратной совместимостью со старыми файлами `.stc`.
  - [x] Тесты `test_stc_container_with_workspace_roundtrip` и `test_stc_legacy_container_without_colors_backward_compat`.
- [x] L1 `statcraft_core::workspace`:
  - [x] Поле `pub tab_color: Option<String>` в `DocumentSession`, методы `set_tab_color`, `tab_color`.
  - [x] Методы `WorkspaceManager::session_color_by_name`, `existing_tab_colors`, `sessions_mut`.
  - [x] Математический генератор `generate_contrast_hex` на базе 64-битного SplitMix64 с угловой дистанцией $\ge 45^\circ$ по золотому сечению ($137.50777^\circ$).
  - [x] Автоматическое присвоение цвета при вызове `add_dataset`.
  - [x] Персистентность цветов в `statcraft_core::stc_codec` при сериализации/десериализации воркспейса.
  - [x] Тесты `test_dynamic_contrast_colors_for_similar_prefixes` и `test_save_load_multi_file_workspace_stc_color_persistence`.
- [x] L6 `statcraft_app`:
  - [x] Функции `theme::hex_to_color32`, `color32_to_hex`, `color32_to_hsl`, `generate_contrast_color`, обновленный `dataset_color`.
  - [x] Отрисовка вкладок `DocumentTabStrip` с чтением сохраненного цвета сессии.
  - [x] Сквозная привязка цветов в `OutputViewer`: вызов `render_output_view_with_workspace`, отображение бейджей в Outline Tree и сводной таблицы Notes через `workspace.session_color_by_name`.
  - [x] Интеграционный тест `dataset_color_persistence_tests.rs`.
- [x] Quality Gate & Definition of Done:
  - [x] 100% успешный прогон всех проверок: `file-guard` (470 файлов), `design-guard`, `layers`, `cargo clippy --workspace -j 1 -- -D warnings`, `cargo fmt --check`, `cargo test --workspace -j 1`.

## 63. Исправление позиционирования маркера несохраненности на вкладках и устранение ложного маркера в шапке окна (BUG-016)
- [x] L6 `statcraft_app::views::document_tabs`:
  - [x] Устранение наложения звездочки на текст названия вкладки: удален хардкод `min(12) * 6.5`.
  - [x] Резервирование гарантированного места справа (`right_reserved = 32.0 px`), аккуратное усечение имени файла через `truncate_at_width`.
  - [x] Размещение звездочки `*` строго за текстом с вертикальным центрированием `Align2::LEFT_CENTER`.
- [x] L6 `statcraft_app::app`:
  - [x] Полное удаление захардкоженного глифа ` •` (U+2022) из заголовка окна программы в соответствии с п. 8 AGENTS.md.
  - [x] Отображение заголовка строго в каноническом формате: `file_name` (для сохраненного) или `file_name *` (для измененного).
- [x] L6 `statcraft_app::app_session`:
  - [x] Рефакторинг `has_unsaved_changes()`: семантическая изоляция проверок изменений активной сессии от глобального чекпоинта; устранено ложное срабатывание при переключении между датасетами с разным числом строк/колонок.
  - [x] Сброс флага `sess.mark_clean()` при сохранении данных в CSV/TSV/TXT в `save_to_path`.
  - [x] Актуализация `delta_undo_checkpoint_len` в `statcraft_core::workspace::DocumentSession::mark_clean()`.
- [x] Тестирование и Quality Gate:
  - [x] Новый интеграционный тест `test_multi_dataset_dirty_marker_isolation_and_csv_clean` в `session_lifecycle_tests.rs`.
  - [x] 100% прохождение полного Quality Gate `cargo run -p xtask -j 1 -- ci` (fmt, clippy с 0 warnings, layers, file-guard на 470 файлах, design-guard, тесты воркспейса, матрица обратимости undo/redo).

## 64. Устранение потери точности вычислений процедур ядра и переход на get_cell_numeric (PR #5 от @jjohare)
- [x] L1 `statcraft_core::dataset`:
  - [x] Внедрение метода `DataSet::get_cell_numeric(&self, row: usize, col: usize) -> Option<f64>` для прямого извлечения вещественных чисел без строкового форматирования по `decimals`.
  - [x] Безопасная обработка пропусков (`Null`, `""`, `"."`, `"NA"`) и парсинг числовых строк без паник.
  - [x] Поддержка прямого извлечения чисел из ячеек с форматами `Dollar` и `Comma` без сбоев строкового парсинга.
- [x] L5 `statcraft_syntax::commands` (21 процедура):
  - [x] Замена `get_cell_display(...).parse::<f64>()` на `get_cell_numeric(...)` в `anova_oneway`, `correlations`, `curvefit`, `descriptives`, `factor`, `glm_multivariate`, `glm_repeated`, `glm_univariate`, `kruskal_wallis`, `logistic_regression`, `means`, `multinomial_logistic`, `nonparam_related`, `nonparam_twosample`, `partial_corr`, `regression_linear`, `reliability`, `select_cases`, `ttest_indep`, `ttest_onesample`, `ttest_paired`.
  - [x] Сохранение группирующего поведения по строковым меткам на `get_cell_display`.
- [x] L6 `statcraft_app::dialogs`:
  - [x] Перевод подсчета уникальных уровней зависимой переменной в `logistic_regression_dialog` и `multinomial_logistic_dialog` на `get_cell_numeric`.
- [x] Тестирование и Quality Gate:
  - [x] Интеграционный тест точности ядра `crates/statcraft_syntax/tests/numeric_precision_tests.rs` (сверка с эталоном с точностью до $10^{-9}$).
  - [x] Интеграционный тест точности диалогов `crates/statcraft_app/tests/numeric_dialog_precision_tests.rs`.
  - [x] 100% прохождение Quality Gate: `cargo test --workspace -j 1`, `xtask file-guard`, `xtask design-guard`.

## 65. Официальный веб-портал, интерактивная вики и серверный движок анонимных тикетов (website/)
- [x] Веб-инфраструктура на Next.js 15 (App Router), React 19, Tailwind CSS v4, TypeScript:
  - [x] Чертежная архитектурная эстетика Blueprint из PhotoCraft Companion с канонической палитрой StatCraft.
  - [x] Академическое позиционирование без коммерческого хайпа, предупреждение об ограничениях и открытый призыв к сотрудничеству.
- [x] Интерактивный лендинг (`/` и `/ru`):
  - [x] Секция `Hero`: манифест открытой науки, статус сборки v0.4.3, 4 чипа архитектурных гарантий.
  - [x] Секция `OracleBenchmark`: матрица 20 блоков аудита со сверкой против SciPy ($\Delta \le 10^{-6}$), поиском и формулами в KaTeX.
  - [x] Секция `AgentLab`: демонстрация протокола Model Context Protocol (MCP) и локального IPC-сокета 18492 для ИИ-агентов.
  - [x] Секция `ArchitectureGuarantee`: слоистая архитектура L0–L6 и гарантии 100% Safe Rust без паник.
  - [x] Секция `CallForResearchers`: приглашение исследователей и разработчиков к тестированию и предоставлению массивов данных.
- [x] Высокоэргономичная 3-колоночная база знаний и Вики (`/docs` и `/ru/docs`):
  - [x] Левая колонка с категориями, древовидной навигацией и живым поиском.
  - [x] Центральная область чтения с рендерингом математики в KaTeX и копируемыми блоками синтаксиса SPSS (.sps).
  - [x] Правая колонка оглавления страницы («On this page») с быстрыми ссылками.
- [x] Серверный движок анонимных тикетов для исследователей (`/api/create-issue`):
  - [x] Vercel Serverless Function с защитой honeypot и лимитом по IP.
  - [x] Прямое создание тикета в GitHub с лейблами `community-report` и `needs-triage`.
- [x] Контроль качества и сборка:
  - [x] 100% статическая генерация Next.js: 18 из 18 страниц собраны чисто с кодом 0 (`npx next build`).
  - [x] Инварианты Rust: `xtask file-guard` (472 файла в норме) и `output_viewer_invariants` (11/11 passed).

## 66. Устранение дефектов безрамочного окна Windows 10 и защита диалогов от раздутия (BUG-017)
- [x] L6 `statcraft_app::window_frame`:
  - [x] Интеграция безопасных системных оберток `winsafe = "0.0.29"` и `raw-window-handle = "0.6"` в `Cargo.toml`.
  - [x] Реализация стража `window_frame.rs` по канону PhotoCraft: безопасное получение HWND через `EnumWindows`, снятие `WS_CAPTION | WS_BORDER` через `strip()` с пересчетом фрейма `SWP_FRAMECHANGED` каждый кадр.
  - [x] Функция `keep_clear_of_the_taskbar`: защита от наезда развернутого окна на панель задач Windows с учетом как стандартных, так и автоматически скрываемых таскбаров.
  - [x] Полное устранение 26 px черной полосы сверху, полос по бокам и вертикального сдвига курсора мыши на Windows 10 при сохранении 100% CSD-дизайна.
  - [x] Пакет модульных тестов `window_frame` (7 тестов).
- [x] L6 `statcraft_app::dialogs::common`:
  - [x] `dialog_shell.rs`: добавление `.max_size(size)` и `.constrain_to(ctx.screen_rect())` для предотвращения неконтролируемого раздутия модальных окон шире канонических размеров.
  - [x] `variable_picker.rs`: ограничение `ScrollArea::vertical().max_width(inner_size.x)`, усечение текста кнопок списка через `.truncate()`, тултип полного названия переменной при наведении `.on_hover_text(...)`.
- [x] Тестирование и Quality Gate:
  - [x] 38 тестов инвариантов диалогов `crates/statcraft_app/tests/dialog_invariants.rs`.
  - [x] 11 тестов инвариантов Output Viewer `crates/statcraft_app/tests/output_viewer_invariants.rs`.
  - [x] 100% прохождение Quality Gate: `cargo run -p xtask -j 1 -- ci` (fmt, clippy с 0 warnings, layers, file-guard на 473 файла, design-guard, тесты воркспейса, матрица обратимости undo/redo).

## 67. Декомпозиция ядра FREQUENCIES, 14 одномерных статистик по стандарту SPSS и Golden Oracle
- [x] L2 `statcraft_algo::frequencies`:
  - [x] Архитектурная декомпозиция монолита `frequencies.rs` (405 строк) на модульный пакет `crates/statcraft_algo/src/frequencies/` с жестким соблюдением лимита $\le 500$ строк на файл (Rule 12 File Size Guard):
    - [x] `options.rs`: структуры `FrequenciesStatisticsOptions` и `PercentilesOptions`;
    - [x] `percentiles.rs`: вычисление процентилей по методу SPSS HAVERAGE (Weibull / Type 6), N-тили (`compute_ntiles`), пользовательские срезы (`compute_custom_percentiles`), коррекция для сгруппированных данных (`group_midpoints`);
    - [x] `stats.rs`: полный расчет 14 одномерных статистик (Mean, SE Mean, Median, Mode, Sum, Std Dev, Variance, Range, Min, Max, Skewness, SE Skewness, Kurtosis, SE Kurtosis) с каноническим правилом Mode Tie-Breaker (наименьшее значение при мультимодальности + флаг `has_multiple_modes`);
    - [x] `table.rs`: расчет частот, процентов, валидных процентов и кумулятивных процентов с поддержкой весов и пользовательских пропусков;
    - [x] `mod.rs`: единый публичный фасад со 100% обратной совместимостью для L5 (`statcraft_syntax`) и L6 (`statcraft_app`).
- [x] Тестирование и Quality Gate:
  - [x] Прецизионный Golden Oracle тест `crates/statcraft_algo/tests/frequencies_oracle_test.rs` со сверкой с эталонами SciPy/SPSS ($\Delta \le 10^{-6}$).
  - [x] 100% успешный прогон тестов: 218 тестов `statcraft_algo`, 11 тестов `output_viewer_invariants`, `output_viewer_navigation`.
  - [x] `cargo run -p xtask -j 1 -- file-guard` (все 478 файлов Rust в пределах лимитов строк).
  - [x] `cargo run -p xtask -j 1 -- layers` (все слои L0–L6 строго соблюдены).
  - [x] `cargo clippy -p statcraft_algo --tests -j 1 -- -D warnings` (0 предупреждений).
  - [x] `cargo fmt --check` (чисто).

## 68. Командная шина `statcraft_syntax` и двусторонний SPSS-синтаксис FREQUENCIES (Этап 2)
- [x] L5 `statcraft_syntax::commands::frequencies_cmd`:
  - [x] Расширение `specs()`: валидационная JSON-схема для объектов `statistics` (12 дескриптивных метрик), `percentiles` (квартили, N-тили, custom cutpoints), `group_midpoints`, `display_tables` и `order`.
  - [x] Реализация `execute()`: извлечение параметров в `FrequenciesStatisticsOptions`, вызов `statcraft_algo::compute_frequencies_full`, подавление параметрических статистик для шкалы `Nominal` и каноническая пересортировка категорий (`apply_order` для `dvalue`, `afreq`, `dfreq`) с пересчетом кумулятивных процентов.
  - [x] Каноническая генерация синтаксиса `to_sps_from_params` (`FREQUENCIES VARIABLES=... /FORMAT=... /STATISTICS=... /PERCENTILES=... /NTILES=... /ORDER=...`).
  - [x] Декомпозиция монолита `frequencies_cmd.rs` на подпапку `commands/frequencies_cmd/` (`mod.rs`, `sps.rs`, `tests.rs`) с соблюдением лимита $\le 500$ строк на файл (Rule 12 File Size Guard).
- [x] L5 `statcraft_syntax::syntax_parser::mappers::stats`:
  - [x] Декомпозиция модуля `stats.rs` (490 строк) на подпапку `mappers/stats/` (`mod.rs`, `frequencies.rs`, `ttest.rs`) с жестким соблюдением лимита $\le 500$ строк.
  - [x] Поддержка ключевых слов SPSS для `/STATISTICS`: макросы `DEFAULT`, `ALL`, `NONE` и индивидуальные метрики `MEAN`, `SEMEAN`, `MEDIAN`, `MODE`, `SUM`, `STDDEV`, `VARIANCE`, `RANGE`, `MINIMUM`, `MAXIMUM`, `SKEWNESS`, `KURTOSIS`.
  - [x] Поддержка подкоманд `/PERCENTILES`, `/NTILES`, `/FORMAT` (`TABLE`, `NOTABLE`, `DVALUE`, `AFREQ`, `DFREQ`), `/ORDER` (`ANALYSIS`, `VARIABLE`).
- [x] Тестирование и Quality Gate:
  - [x] 100% прохождение 133 тестов `statcraft_syntax` и 58 интеграционных тестов синтаксиса.
  - [x] 100% прохождение 11 тестов инвариантов Output Viewer (`output_viewer_invariants`).
  - [x] `cargo run -p xtask -j 1 -- file-guard` (482 файла Rust в пределах лимитов строк).
  - [x] `cargo run -p xtask -j 1 -- layers` и `cargo run -p xtask -j 1 -- design-guard` (полное соответствие канону).

## 69. Модальный поддиалог «Частоты: Статистики» и модернизация DialogKit (Этап 3)
- [x] L6 `statcraft_app::dialogs::frequencies_dialog`:
  - [x] Архитектурная декомпозиция диалога «Частоты» на модульный подпакет `crates/statcraft_app/src/dialogs/frequencies_dialog/` (`mod.rs`, `state.rs`, `statistics_subdialog.rs`) с соблюдением лимита $\le 500$ строк на файл;
  - [x] Модальный поддиалог «Частоты: Статистики» (`statistics_subdialog.rs`, $380 \times 460$ px) на базе `DialogShell` и `SubdialogActionBar`:
    - [x] `GroupBox` «Значения процентилей»: чекбоксы `Квартили`, `Процентили для: [ 10 ] равных групп`, интерактивный редактор пользовательских процентилей с кнопками `[ Добавить ] [ Изменить ] [ Удалить ]` и компактным списком с автосортировкой;
    - [x] `GroupBox` «Положение центра распределения»: Среднее, Медиана, Мода, Сумма;
    - [x] Чекбокс «Значения — центры групп» (`group_midpoints`);
    - [x] `GroupBox` «Разброс»: Стд. отклонение, Дисперсия, Размах, Минимум, Максимум, Станд. ошибка среднего;
    - [x] `GroupBox` «Распределение»: Асимметрия, Эксцесс;
    - [x] `SubdialogActionBar`: сохранение настроек по `[ Продолжить ]` и откат к снимку по `[ Отмена ]` / закрытию окна;
  - [x] Модернизация главного диалога «Частоты»:
    - [x] Боковая панель `SubdialogSidebar`: активная кнопка `[ Статистика... ]`, кнопки `[ Диаграммы... ]`, `[ Формат... ]`, `[ Стиль... ]`, `[ Бутстреп... ]`;
    - [x] Чекбоксы `Выводить таблицы частот` и `Создать таблицы стилей APA`;
    - [x] Полная передача всех параметров статистик и процентилей в командную шину `stat.frequencies` и генератор SPSS `.sps`-синтаксиса (`to_sps_from_params`);
  - [x] Декомпозиция тестов `crates/statcraft_app/tests/dialog_invariants.rs` на подпапку `tests/dialog_invariants/` (`fixtures.rs`, `extended_dialogs.rs`) с удержанием размера файлов $\le 550$ строк;
  - [x] Полная двуязычная локализация (Zero-Hardcode i18n): 19 новых семантических ключей в `locales/en.ftl` и `locales/ru.ftl`;
  - [x] Quality Gate: 100% прохождение `xtask file-guard` (486 файлов), `xtask design-guard`, `xtask layers`, `i18n_lint_test` и 39 тестов `dialog_invariants`.

## 70. Сводная таблица «Статистики» и таблицы частот в Output Viewer (Этап 4)
- [x] L6 `statcraft_app::views::output_view::tables_frequencies`:
  - [x] Архитектурная декомпозиция монолита `tables_frequencies.rs` (375 строк) на модульный пакет `crates/statcraft_app/src/views/output_view/tables_frequencies/` с жестким соблюдением лимита $\le 500$ строк на файл (Rule 12 File Size Guard):
    - [x] `mod.rs`: координация рендеринга блока процедуры и экспорт в TSV;
    - [x] `stats_summary_table.rs`: аутентичная сводная таблица SPSS «Статистика» (Statistics Summary Table) с динамическим набором строк (N Valid, N Missing, 14 одномерных статистик, процентили и сноска о множественных модах);
    - [x] `freq_table.rs`: детальные таблицы частот категорий для каждой переменной с метками значений, процентами, валидными и кумулятивными процентами;
  - [x] Поддержка режима `/FORMAT=NOTABLE` (`display_tables: false`): подавление вывода детальных таблиц частот с сохранением сводной таблицы «Статистика»;
  - [x] Канонический экспорт `frequencies_to_tsv`: объединение сводной таблицы статистик и таблиц частот в единый форматированный поток TSV для Word/Excel;
  - [x] Инвариантные интеграционные тесты `crates/statcraft_app/tests/output_viewer_frequencies_invariants.rs`:
    - [x] Проверка канонического рендеринга в headless egui `Context` без паник;
    - [x] Проверка наличия всех заголовков, метрик, сносок о модах и меток значений в TSV;
    - [x] Проверка подавления детальных таблиц в режиме `NOTABLE`;
  - [x] Полная двуязычная локализация (Zero-Hardcode i18n): 19 новых семантических ключей в `locales/en.ftl` и `locales/ru.ftl`;
  - [x] Quality Gate: 100% прохождение `xtask file-guard` (489 файлов), `xtask design-guard`, `xtask layers`, `cargo clippy --workspace -j 1 -- -D warnings` и полного набора тестов `xtask ci`.

## 71. Восстановление статистик для номинальных шкал и отображение меток переменных
- [x] L2 `statcraft_algo::frequencies::table`:
  - [x] Расширение `FrequencyResult` полем `pub variable_label: Option<String>`;
  - [x] Реализация методов `display_title()` (`Метка [Имя]`) и `display_label_or_name()`;
  - [x] Модульные тесты форматирования заголовков и меток.
- [x] L5 `statcraft_syntax::commands::frequencies_cmd`:
  - [x] Устранение искусственного подавления статистик и процентилей для `Measure::Nominal`: расчет всех статистик для любых числовых переменных по канону SPSS;
  - [x] Проброс `var_meta.label` в результат `FrequencyResult`;
  - [x] Модульные тесты вычисления статистик для номинальных переменных и извлечения меток.
- [x] L6 `statcraft_app::views::output_view`:
  - [x] `freq_table.rs`: отображение содержательных меток переменных в заголовках таблиц частот (`Таблица частот: Метка [Имя]`);
  - [x] `stats_summary_table.rs`: отображение меток переменных в заголовках колонок сводной таблицы «Статистика» с тултипом полного имени и технического идентификатора;
  - [x] `builder.rs`: добавление метода `add_frequencies_with_log(results, sps_log)` и сохранение обратной совместимости `add_frequencies`;
  - [x] `app_dialogs.rs` и `syntax_runner.rs`: генерация и протоколирование реального SPSS-синтаксиса (`/STATISTICS=...`, `/PERCENTILES=...`, `/ORDER=...`) в журнале `Log`;
  - [x] Инвариантные интеграционные тесты отображения меток и журнала в `output_viewer_frequencies_invariants.rs`.
- [x] Quality Gate (DoD):
  - [x] 100% прохождение `cargo run -p xtask -j 1 -- ci` (`file-guard` по 489 файлам, `design-guard`, `layers`, `clippy -D warnings`, полный тестовый набор воркспейса и матрица обратимости 41 команды).

## 20. Планы будущих релизов (Backlog / Planned Features)
- [ ] Мультимониторный режим рабочего пространства (Multi-Monitor Viewports & Pop-out Tabs):
  - [ ] Механизм открепления вкладок во внешние нативные окна ОС через `ctx.show_viewport_immediate`.
  - [ ] Меню и контекстное действие выноса во внешнее окно («Move to New Window» / «Pop out»).
  - [ ] Кнопка обратного возврата в основной док («Dock Back» / «Attach to Main Window») и автоматический возврат при закрытии внешнего окна.
  - [ ] Полное сохранение существующих интерактивных маркеров сброса, сплиттеров и внутренней организации дока в родительском окне.
  - [ ] Бесшовная синхронизация общего состояния (`AppState`, `DataSet`, `CommandBus`) в рамках единого процесса без межпроцессных накладных расходов.







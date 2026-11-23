# StatCraft Russian Localization Catalog (Русский каталог)

# Application Header & Menus
app-title = StatCraft

# Common Menu Status & Badges
menu-badge-dev = [dev]
menu-item-in-dev-tooltip = В разработке (согласно Дорожной Карте)
menu-item-requires-dataset = Требуется открытый набор данных
menu-overflow = »
caption-minimize = Свернуть
caption-maximize = Развернуть
caption-restore = Восстановить
caption-close = Закрыть

# 1. File Menu
menu-file = Файл
menu-file-new = Создать
menu-file-new-project = Новый проект
menu-file-new-dataset = Новый набор данных
menu-file-new-syntax = Файл синтаксиса
menu-file-open-project = Открыть проект...
menu-file-open-data = Открыть данные...
menu-file-open-syntax = Открыть синтаксис...
menu-file-open-csv = Открыть CSV...
menu-file-import-data = Импорт данных
menu-file-import-spss = Файл данных SPSS (*.sav, *.zsav)...
menu-file-import-csv = Текстовый файл с разделителями (*.csv, *.tsv)...
menu-file-import-excel = Электронная таблица Excel (*.xlsx)...
menu-file-recent = Недавние файлы
menu-file-recent-empty = (Нет недавних файлов)
menu-file-recent-clear = Очистить список недавних файлов
menu-file-save = Сохранить проект
menu-file-save-as = Сохранить проект как...
menu-file-save-data-as = Сохранить данные как...
menu-file-save-syntax-as = Сохранить синтаксис как...
menu-file-export = Экспорт
menu-file-export-output = Экспорт отчета вывода...
menu-file-export-data = Экспорт данных в CSV / Excel...
menu-file-close-project = Закрыть проект
menu-file-exit = Выход

# 2. Edit Menu
menu-edit = Правка
menu-edit-undo = Отменить
menu-edit-redo = Повторить
menu-edit-cut = Вырезать
menu-edit-copy = Копировать
menu-edit-copy-names = Копировать с именами переменных
menu-edit-paste = Вставить
menu-edit-clear = Очистить
menu-edit-find = Найти...
menu-edit-replace = Заменить...
menu-edit-goto-case = Перейти к наблюдению...
menu-edit-goto-var = Перейти к переменной...
menu-edit-insert-case = Вставить наблюдение
menu-edit-insert-var = Вставить переменную
menu-edit-preferences = Параметры...

# 3. View Menu
menu-view = Вид
menu-view-data = Представление данных
menu-view-variables = Представление переменных
menu-view-output = Просмотр вывода
menu-view-status-bar = Строка состояния
menu-view-toolbars = Панели инструментов
menu-view-value-labels = Метки значений
menu-view-grid-lines = Сетка таблицы
menu-view-zoom-in = Увеличить шрифт
menu-view-zoom-out = Уменьшить шрифт
menu-view-zoom-reset = Сбросить масштаб
menu-view-theme = Тема оформления
menu-view-theme-light = Светлая (Классическая)
menu-view-theme-dark = Тёмная (Dark Pro)
menu-view-theme-system = Системная (Авто)
menu-view-language = Язык

lang-name-en = English
lang-name-ru = Русский

# 4. Data Menu
menu-data = Данные
menu-data-sort-cases = Сортировать наблюдения...
dialog-sort-cases-title = Сортировать наблюдения
dialog-sort-cases-sort-by = Сортировать по:
dialog-sort-cases-order-group = Порядок сортировки
dialog-sort-cases-order-ascending = По возрастанию
dialog-sort-cases-order-descending = По убыванию
dialog-sort-cases-move-up = Вверх
dialog-sort-cases-move-down = Вниз
menu-data-sort-vars = Сортировать переменные...
dialog-sort-variables-title = Сортировать переменные
dialog-sort-vars-attribute-group = Сортировать по
sort-attr-name = Имя
sort-attr-type = Тип
sort-attr-label = Метка
sort-attr-measure = Шкала измерений
sort-attr-role = Роль
sort-attr-align = Выравнивание
sort-attr-width = Ширина
sort-attr-decimals = Десятичные
output-sort-variables-executed = Переменные отсортированы по { $attribute } ({ $direction }).
menu-data-transpose = Транспонировать...
dialog-transpose-title = Транспонировать
dialog-transpose-variables = Переменные:
dialog-transpose-name-var = Переменная имен:
output-transpose-executed = Матрица данных транспонирована ({ $rows } строк, { $cols } колонок).
menu-data-merge-files = Слияние файлов
menu-data-merge-cases = Добавить наблюдения...
menu-data-merge-vars = Добавить переменные...
dialog-merge-cases-title = Добавить наблюдения
dialog-merge-vars-title = Добавить переменные
dialog-merge-source-group = Источник данных
dialog-merge-source-open-dataset = Открытый набор данных:
dialog-merge-source-external-file = Внешний файл данных:
dialog-merge-browse-btn = Обзор...
dialog-merge-select-file = Выберите файл
dialog-merge-unpaired-vars = Несопоставленные переменные:
dialog-merge-new-active-vars = Переменные в новом активном наборе данных:
dialog-merge-excluded-vars = Исключенные переменные:
dialog-merge-key-vars = Ключевые переменные:
dialog-merge-indicate-source = Указать источник наблюдения в переменной:
dialog-merge-source-var-name = Имя переменной
dialog-merge-rename-btn = Переименовать...
dialog-merge-pair-btn = Сопоставить
dialog-merge-move-to-keys-btn = Ключ ->
dialog-merge-method-group = Метод объединения
dialog-merge-method-1to1 = Сопоставление 1-к-1 по ключевым переменным
dialog-merge-method-table = Таблица-справочник 1-ко-многим
dialog-merge-method-seq = Последовательное объединение по порядку строк
dialog-merge-sort-keys = Отсортировать наблюдения по ключевым переменным перед слиянием
dialog-merge-error-no-source = Пожалуйста, выберите второй набор данных или файл.
dialog-merge-error-no-keys = Для объединения по ключам требуется хотя бы одна ключевая переменная.
output-merge-cases-executed = Наблюдения добавлены ({ $rows } строк, { $cols } колонок).
output-merge-vars-executed = Переменные добавлены ({ $rows } строк, { $cols } колонок).
status-merge-cases-executed = Добавлены наблюдения из внешнего набора данных. Всего строк: { $rows }, колонок: { $cols }.
status-merge-vars-executed = Добавлены переменные из внешнего набора данных. Всего строк: { $rows }, колонок: { $cols }.
menu-data-restructure = Реструктуризация данных...
menu-data-aggregate = Агрегировать данные...
menu-data-split-file = Разделить файл...
menu-data-filter = Выбрать наблюдения...
menu-data-weight-cases = Взвесить наблюдения...

# 5. Transform Menu
menu-transform = Преобразование
menu-transform-compute = Вычислить переменную...
menu-transform-shift = Сдвиг значений (Lead / Lag)...
menu-transform-recode = Перекодирование
menu-transform-recode-diff = Перекодировать в другие переменные...
menu-transform-recode-same = Перекодировать в те же переменные...
menu-transform-auto-recode = Автоматическое перекодирование...
menu-transform-dummy = Создать фиктивные (dummy) переменные...
menu-transform-visual-binning = Визуальная группировка...
menu-transform-optimal-binning = Оптимальная группировка...
menu-transform-adp = Подготовка данных для моделирования (ADP)...
menu-transform-rank-cases = Ранжировать наблюдения...
menu-transform-date-time = Мастер даты и времени...
menu-transform-time-series = Создать временной ряд...
menu-transform-missing-values = Заменить пропущенные значения...
menu-transform-random-seed = Установить зерно ДСЧ...
menu-transform-count-values = Подсчет значений в строках...
menu-transform-programmability = Программируемые преобразования...

# 6. Analyze Menu
menu-analyze = Анализ
menu-analyze-reports = Отчеты
menu-analyze-reports-codebook = Кодовая книга...
menu-analyze-reports-case-summaries = Итоги по наблюдениям...
menu-analyze-descriptive-stats = Описательные статистики
menu-analyze-frequencies = Частоты...
menu-analyze-descriptives = Описательные...
menu-analyze-explore = Исследовать...
menu-analyze-crosstabs = Таблицы сопряженности...
menu-analyze-compare-means = Сравнение средних
menu-analyze-ttest = Т-критерий для независимых выборок...
menu-analyze-ttest-one-sample = Одновыборочный T-критерий...
menu-analyze-ttest-indep = T-критерий для независимых выборок...
menu-analyze-ttest-paired = Парный T-критерий...
menu-analyze-anova-oneway = Однофакторный дисперсионный анализ...
menu-analyze-glm = Общая линейная модель
menu-analyze-glm-univariate = Одномерная (ANCOVA)...
menu-analyze-glm-multivariate = Многомерная (MANOVA)...
menu-analyze-glm-repeated = Повторные измерения...
menu-analyze-correlate = Корреляции
menu-analyze-correlate-bivariate = Парные...
menu-analyze-correlate-partial = Частные...
menu-analyze-regression = Регрессия
menu-analyze-regression-linear = Линейная...
menu-analyze-regression-binary = Бинарная логистическая...
menu-analyze-regression-multinomial = Мультиномиальная логистическая...
menu-analyze-regression-curve = Оценка кривых...
menu-analyze-nonparametric = Непараметрические критерии
menu-analyze-nonparam-two-indep = Две независимые выборки...
menu-analyze-nonparam-k-indep = K независимых выборок...
menu-analyze-nonparam-independent = Независимые выборки (Манна-Уитни / Краскела-Уоллиса)...
menu-analyze-nonparam-related = Связанные выборки (Вилкоксона / Фридмана)...
menu-analyze-dimension = Снижение размерности и классификация
menu-analyze-factor = Факторный анализ / PCA...
menu-analyze-cluster = Кластерный анализ (K-Means / Иерархический)...
menu-analyze-scale = Шкалы
menu-analyze-reliability = Анализ надежности (Альфа Кронбаха)...
menu-analyze-survival = Анализ выживаемости
menu-analyze-survival-life-tables = Таблицы дожития...
menu-analyze-survival-kaplan-meier = Каплана-Мейера...
menu-analyze-survival-cox = Регрессия Кокса...

# 7. Direct Marketing Menu
menu-direct-marketing = Прямой маркетинг
menu-dm-rfm = RFM-анализ (давность, частота, чек)...
menu-dm-cluster = Кластерная сегментация контактов...
menu-dm-profile = Профилирование потенциальных контактов...
menu-dm-control-package = Тестирование контрольных пакетов (A/B тест)...
menu-dm-response-rate = Анализ отклика по регионам...

# 8. Graphs Menu
menu-graphs = Графика
menu-graphs-chart-builder = Построитель диаграмм...
menu-graphs-legacy = Устаревшие диалоговые окна
menu-graphs-bar = Столбчатая диаграмма...
menu-graphs-line = Линейный график...
menu-graphs-pie = Круговая диаграмма...
menu-graphs-boxplot = Диаграмма размаха («ящик с усами»)...
menu-graphs-error-bar = Планки погрешностей...
menu-graphs-scatter = Диаграмма рассеяния...
menu-graphs-histogram = Гистограмма...
menu-graphs-pyramid = Половозрастная пирамида...

# 9. Utilities Menu
menu-utilities = Сервис
menu-utilities-variables = Карточка переменной...
menu-utilities-oms = Наборы переменных...
menu-utilities-comments = Комментарии к датасету...
menu-utilities-codebook = Генератор кодовой книги...

# 10. Extensions Menu
menu-extensions = Расширения
menu-extensions-hub = Каталог расширений...
menu-extensions-install = Установить локальный пакет (.scpl)...
menu-extensions-dialog-builder = Конструктор диалогов...
menu-extensions-mcp-bridge = Центр управления AI MCP и сокет...
menu-extensions-scripting = Настройки скриптового моста (Python / R)...

# 11. Window Menu
menu-window = Окно
menu-window-reset-layout = Сбросить расположение панелей
menu-window-presets = Пресеты раскладки
menu-window-preset-split = Аналитическая раскладка (Split View)
menu-window-preset-classic = Классическая раскладка SPSS
menu-window-preset-syntax = Скриптовая раскладка
menu-window-data-editor = Представление данных
menu-window-variable-view = Представление переменных
menu-window-output-viewer = Просмотр вывода
menu-window-syntax-editor = Редактор синтаксиса
menu-window-ai-copilot = Консоль AI Copilot
menu-window-minimize-all = Свернуть все

# 12. Help Menu
menu-help = Справка
menu-help-coach = Статистический советник и AI-гид...
menu-help-algorithms = Справочник алгоритмов...
menu-help-syntax = Справочник по синтаксису...
menu-help-sample-data = Учебные датасеты
menu-help-sample-iris = Ирисы Фишера
menu-help-sample-trial = Клиническое исследование (АД)
menu-help-sample-survey = Опрос удовлетворенности
menu-help-check-updates = Проверить обновления...
menu-help-about = О программе StatCraft...

# Variable View Headers
header-row = №
header-name = Имя
header-type = Тип
header-width = Ширина
header-decimals = Дробные
header-label = Метка
header-values = Значения
header-missing = Пропущенные
header-columns = Столбцы
header-align = Выравнивание
header-measure = Шкала
header-role = Роль

# Measure Levels
measure-scale = Количественная
measure-ordinal = Порядковая
measure-nominal = Номинальная

# Data Types
type-numeric = Числовой
type-comma = Запятая
type-dot = Точка
type-scientific = Научный
type-date = Дата
type-dollar = Доллар
type-custom-currency = Пользовательская валюта
type-string = Строковый
type-restricted-numeric = Целое с ведущими нулями

# Alignments
align-left = Влево
align-right = Вправо
align-center = По центру

# Roles
role-input = Входной
role-target = Целевой
role-both = Оба
role-none = Нет
role-partition = Разбиение
role-split = Разделение

# Toolbar & Status
status-ready = Готов
status-error = Ошибка
status-rows-count = Наблюдений: { $count }
status-cols-count = Переменных: { $count }
status-cell-edited = Ячейка ({ $row }, { $col }) обновлена.
status-new-dataset-created = Создан новый набор данных.
status-new-syntax-created = Создан новый редактор синтаксиса.
status-syntax-opened = Синтаксис открыт: { $name }
status-syntax-saved = Синтаксис сохранен: { $name }
syntax-open = Открыть .sps
syntax-save = Сохранить .sps
syntax-run-all = Выполнить всё
syntax-run-selection = Выполнить выделение
syntax-clear = Очистить
syntax-batch-success = Синтаксис успешно выполнен: { $count } команд.
syntax-batch-error = Ошибка синтаксиса: { $error }
status-new-project-created = Создан новый проект.
status-data-exported = Данные успешно экспортированы в { $path }
status-output-exported = Журнал вывода успешно экспортирован в { $path }
dialog-export-data-title = Экспорт данных
dialog-export-output-title = Экспорт отчета вывода
filter-csv-comma = Значения, разделенные запятыми (*.csv)
filter-csv-semicolon = Значения, разделенные точкой с запятой (*.csv)
filter-tsv = Значения, разделенные табуляцией (*.tsv, *.txt)
filter-html = Веб-страница / Документ HTML (*.html, *.htm)
filter-txt = Текстовый документ (*.txt)
filter-tsv-bundle = Все таблицы TSV (*.tsv)
toolbar-toggle-labels = Метки значений
toolbar-open = Открыть файл данных (Ctrl+O)
toolbar-save = Сохранить этот документ (Ctrl+S)
toolbar-print = Печать (Ctrl+P)
toolbar-recall-dialogs = Вызов недавних диалогов
toolbar-undo = Отменить (Ctrl+Z)
toolbar-redo = Повторить (Ctrl+Y)
toolbar-goto-case = Перейти к наблюдению...
toolbar-goto-var = Перейти к переменной...
toolbar-var-info = Переменные...
toolbar-find = Найти в данных (Ctrl+F)
toolbar-insert-case = Вставить наблюдение
toolbar-insert-var = Вставить переменную
toolbar-split-file = Разделить файл...
toolbar-select-cases = Выбрать наблюдения...
toolbar-weight-cases = Взвесить наблюдения...
toolbar-value-labels = Метки значений (1/A)
toolbar-descriptives = Описательные статистики...
toolbar-frequencies = Частоты...
menu-view-toolbar-top = Панель инструментов вверху
menu-view-toolbar-left = Панель инструментов слева
menu-view-toolbar-hidden = Скрыть панель инструментов

# Dialogs & Output
dialog-open-csv-title = Выберите файл массива данных CSV или TSV
dialog-open-excel-title = Выберите электронную таблицу Excel (*.xlsx, *.xls, *.ods)
dialog-open-spss-title = Выберите файл данных SPSS (*.sav, *.zsav)
output-title = Журнал вывода результатов
output-log = Протокол
output-table = Отчёт
output-generic-title = Выполнение команды
output-dataset-imported = Массив данных успешно загружен: { $name } (строк: { $rows }, переменных: { $cols }).
output-no-analysis = Анализ еще не запускался. Выберите процедуру в меню «Анализ».

# Descriptives Dialog & Procedure
dialog-descriptives-title = Описательные статистики
dialog-variables-source = Переменные:
dialog-variables-target = Переменные:
dialog-stats-options = Параметры...
dialog-btn-ok = OK
dialog-btn-paste = Вставить
dialog-btn-reset = Сброс
dialog-btn-cancel = Отмена
dialog-btn-continue = Продолжить
dialog-btn-options = Параметры...
dialog-btn-statistics = Статистика...
dialog-btn-charts = Диаграммы...
dialog-btn-format = Формат...
dialog-btn-style = Стиль...
dialog-btn-bootstrap = Бутстреп...
dialog-btn-add = Добавить
dialog-btn-change = Изменить
dialog-btn-remove = Удалить
dialog-btn-define-groups = Задать группы...
dialog-desc-options-title = Описательные: Параметры
dialog-freq-statistics-title = Частоты: Статистика
dialog-btn-move-right = >
dialog-btn-move-left = <

stat-mean = Среднее
stat-std-dev = Стандартное отклонение
stat-variance = Дисперсия
stat-min = Минимум
stat-max = Максимум
stat-range = Размах
stat-se-mean = Станд. ошибка среднего
stat-skewness = Асимметрия
stat-kurtosis = Эксцесс

output-descriptives-title = Описательные статистики
output-descriptives-table = Описательные статистики
output-descriptives-executed = Процедура «Описательные статистики» выполнена для { $count } переменной(ых).
output-col-variable = Переменная
output-col-n = N
output-col-min = Минимум
output-col-max = Максимум
output-col-mean = Среднее
output-col-std = Станд. отклонение
output-col-var = Дисперсия
output-col-median = Медиана
output-col-skew = Асимметрия
output-col-kurt = Эксцесс
output-col-statistic = Статистика
output-table-valid-n = N валидных (по списку)

# Frequencies Procedure & Dialog
dialog-frequencies-title = Частоты
dialog-frequencies-display-tables = Выводить таблицы частот
dialog-freq-apa-tables = Создать таблицы стилей APA
dialog-freq-group-percentiles = Значения процентилей
dialog-freq-group-central = Положение центра распределения
dialog-freq-group-dispersion = Разброс
dialog-freq-group-distribution = Распределение
dialog-freq-group-midpoints = Значения - центры групп
dialog-freq-cutpoints-prefix = Процентили для:
dialog-freq-cutpoints-suffix = равных групп
dialog-freq-percentiles = Процентили:
dialog-freq-stat-median = Медиана
stat-quartiles = Квартили
stat-mode = Мода
stat-sum = Сумма

# Поддиалог «Частоты: Диаграммы»
dialog-freq-charts-title = Частоты: Диаграммы
dialog-freq-charts-type-group = Тип диаграммы
dialog-freq-charts-type-none = Нет
dialog-freq-charts-type-bar = Столбчатые
dialog-freq-charts-type-pie = Круговые
dialog-freq-charts-type-histogram = Гистограммы:
dialog-freq-charts-histogram-normal = Показать на гистограмме нормальную кривую
dialog-freq-charts-values-group = Значения на диаграмме
dialog-freq-charts-values-freq = Частоты
dialog-freq-charts-values-percent = Проценты

# Поддиалог «Частоты: Формат»
dialog-freq-format-title = Частоты: Формат
dialog-freq-format-order-group = Упорядочить по
dialog-freq-format-order-asc-values = Возрастанию значений
dialog-freq-format-order-desc-values = Убыванию значений
dialog-freq-format-order-asc-counts = Возрастанию количеств
dialog-freq-format-order-desc-counts = Убыванию количеств
dialog-freq-format-multivars-group = Несколько переменных
dialog-freq-format-multivars-compare = Сравнить переменные
dialog-freq-format-multivars-variable = Выводить по переменным
dialog-freq-format-suppress-checkbox = Отключить таблицы со многими категориями
dialog-freq-format-max-categories = Максимальное число категорий:
output-frequencies = Частоты
output-frequencies-title = Таблица частот: { $name }
output-frequencies-table = Таблица частот
output-freq-table-suppressed-note = Таблица частот для переменной { $name } не выводится, так как число категорий превышает заданный лимит.
output-frequencies-charts = Диаграммы
output-frequencies-executed = Процедура «Частоты» выполнена для { $count } переменной(ых).
output-chart-bar = Столбчатая диаграмма
output-chart-pie = Круговая диаграмма
output-chart-histogram = Гистограмма
chart-mean = Среднее
chart-std-dev = Стд. откл.
chart-n = N
chart-frequency = Частота
chart-percent = Процент
chart-no-data = Нет данных для отображения
output-col-value = Значение
output-col-frequency = Частота
output-col-percent = Процент
output-col-valid-percent = Валидный процент
output-col-cum-percent = Накопленный процент
output-row-total-valid = Всего валидных
output-row-system-missing = Системные пропуски
output-row-total = Всего
output-stats-title = Статистика
output-stats-row-valid = Валидные
output-stats-row-missing = Пропущенные
output-stats-row-mean = Среднее
output-stats-row-se-mean = Станд. ошибка среднего
output-stats-row-median = Медиана
output-stats-row-mode = Мода
output-stats-row-std-dev = Станд. отклонение
output-stats-row-variance = Дисперсия
output-stats-row-skewness = Асимметрия
output-stats-row-se-skewness = Станд. ошибка асимметрии
output-stats-row-kurtosis = Эксцесс
output-stats-row-se-kurtosis = Станд. ошибка эксцесса
output-stats-row-range = Размах
output-stats-row-min = Минимум
output-stats-row-max = Максимум
output-stats-row-sum = Сумма
output-stats-row-percentiles = Процентили
output-freq-mode-footnote = a. Существует несколько модальных значений. Отображается наименьшее значение.
stat-q1 = 25-й (Q1)
stat-median = 50-й (Медиана)
stat-q3 = 75-й (Q3)
stat-quartiles-title = Перцентили

# Value Labels Dialog
dialog-value-labels-title = Метки значений: { $name }
dialog-value-labels-value = Значение:
dialog-value-labels-label = Метка:
dialog-value-labels-btn-add = Добавить
dialog-value-labels-btn-change = Изменить
dialog-value-labels-btn-remove = Удалить
dialog-value-labels-list-title = Назначенные метки:
dialog-btn-close = Закрыть

# Missing Values Dialog
dialog-missing-values-title = Пропущенные значения: { $name }
dialog-missing-none = Нет пропущенных значений
dialog-missing-discrete = Дискретные пропущенные значения
dialog-missing-range = Диапазон плюс одно дискретное значение
dialog-missing-low = От:
dialog-missing-high = До:
dialog-missing-discrete-extra = Дискретное значение:

# Variable View & Data View Summaries
values-none = Нет
values-count = Метки ({ $count })
missing-none = Нет
missing-discrete-count = Дискретные ({ $count })
missing-range-summary = Диапазон { $low } - { $high }

# Variable Type Dialog
dialog-var-type-title = Тип переменной: { $name }
dialog-var-type-width = Ширина:
dialog-var-type-decimals = Десятичные знаки:
dialog-var-type-characters = Длина строки:
dialog-var-type-date-format = Формат даты:
dialog-var-type-currency = Валюта:
dialog-btn-help = Справка

# Context Menus & Actions
menu-insert-variable = Вставить переменную
menu-delete-variable = Очистить / Удалить переменную
menu-insert-case = Вставить наблюдение
menu-delete-case = Удалить наблюдение
variable-new-placeholder = Новая переменная...

# Output Viewer & Outline
output-outline-root = Вывод
output-outline-log = Журнал
output-outline-notes = Примечания
output-outline-title = Заголовок
output-outline-warnings = Предупреждения
output-notes-active-dataset = Активный массив: { $name }
output-notes-n-cases = Число наблюдений: { $count }
output-notes-missing-handling = Пропущенные значения: { $rule }
output-toolbar-clear = Очистить журнал
output-toolbar-expand-all = Развернуть всё
output-toolbar-collapse-all = Свернуть всё
output-menu-copy-tsv = Копировать таблицу (TSV)
output-menu-copied = Скопировано (TSV)
output-menu-delete = Удалить
output-copied-toast = Таблица скопирована в буфер обмена (TSV)
output-dataset-imported-title = Массив данных загружен
output-dataset-summary = Сводка массива данных
output-syntax-title = Синтаксис
output-notes-title = Примечания
output-warning-title = Предупреждение
output-missing-exclude-listwise = Исключение наблюдений по списку
output-missing-user-and-sys = Пользовательские и системные пропуски исключаются
output-missing-none-detected = Нет
output-menu-copy-text = Копировать как текст
output-menu-copy-selected = Копировать выделенное
output-notes-timestamp = Время запуска: { $time }
output-notes-variables = Число переменных: { $count }
output-log-executed-at = Время запуска: { $time }
output-root = Вывод
output-log-syntax = Журнал
output-notes = Примечания
output-warnings = Предупреждения
output-btn-clear-all = Очистить журнал
output-btn-expand-all = Развернуть всё
output-btn-collapse-all = Свернуть всё
output-notes-dataset = Набор данных: { $name }
output-notes-rows = Наблюдений: { $count }

# One-Sample T Test Dialog & Output
dialog-ttest-onesample-title = Одновыборочный T-критерий
dialog-ttest-test-variables = Проверяемые переменные:
dialog-ttest-test-value = Проверяемое значение:
dialog-ttest-ci-percentage = Доверительный интервал (%):
output-ttest-onesample-title = Одновыборочный T-критерий
output-ttest-executed = Одновыборочный T-критерий выполнен для { $count } переменной(ых).
output-ttest-stats-title = Статистика одновыборочного критерия
output-ttest-test-title = Одновыборочный критерий (Проверяемое значение = { $val })
output-col-t = t
output-col-df = ст.св.
output-col-sig-2tailed = Знач. (2-сторон)
output-col-mean-diff = Разность средних
output-col-ci-lower = Нижняя
output-col-ci-upper = Верхняя
output-col-ci-header = { $level }% доверительный интервал для разности

# Independent-Samples T Test Dialog & Output
dialog-ttest-indep-title = Т-критерий для независимых выборок
dialog-ttest-indep-grouping-var = Группирующая переменная:
dialog-ttest-indep-define-groups-btn = Задать группы...
dialog-ttest-indep-define-groups-title = Задание групп
dialog-ttest-indep-group1 = Группа 1:
dialog-ttest-indep-group2 = Группа 2:
dialog-ttest-indep-continue = Продолжить
output-ttest-indep-title = Критерий для независимых выборок
output-ttest-indep-executed = Т-критерий для независимых выборок выполнен для { $count } переменной(ых).
output-ttest-group-stats-title = Статистика групп
output-ttest-indep-test-title = Критерий для независимых выборок
output-ttest-indep-levene-title = Критерий Ливиня равенства дисперсий
output-ttest-indep-t-title = t-критерий равенства средних
output-col-levene-f = F
output-col-levene-sig = Знач.
output-col-se-diff = Стандартная ошибка разности
output-row-equal-var-assumed = Равенство дисперсий предполагается
output-row-equal-var-not-assumed = Равенство дисперсий не предполагается
output-col-group = Группа
output-col-variances-assumed = Дисперсии
output-quartiles-title = Процентили: { $name }

# Paired-Samples T Test Dialog & Output
dialog-ttest-paired-title = Т-критерий для парных выборок
dialog-ttest-paired-current-selections = Текущий выбор:
dialog-ttest-paired-var1 = Переменная 1:
dialog-ttest-paired-var2 = Переменная 2:
dialog-ttest-paired-pairs-list = Парные переменные:
dialog-ttest-paired-pair-prefix = Пара { $num }:
output-ttest-paired-title = Т-критерий для парных выборок
output-ttest-paired-executed = Т-критерий для парных выборок выполнен для { $count } пар(ы).
output-ttest-paired-stats-title = Статистика парных выборок
output-ttest-paired-corr-title = Корреляции парных выборок
output-ttest-paired-test-title = Критерий для парных выборок
output-ttest-paired-diff-super = Парные разности
output-col-pair = Пара
output-col-correlation = Корреляция
output-col-sig = Знач.

# One-Way ANOVA Dialog & Output
dialog-anova-oneway-title = Однофакторный дисперсионный анализ
dialog-anova-dependent-list = Зависимые переменные:
dialog-anova-factor = Фактор:
dialog-anova-options-title = Параметры
dialog-anova-descriptives = Описательные статистики
dialog-anova-homogeneity = Критерий однородности дисперсий
dialog-anova-ci-level = Доверительный интервал:
output-anova-title = Однофакторный дисперсионный анализ (ANOVA)
output-anova-descriptives-title = Описательные статистики
output-anova-homogeneity-title = Критерий однородности дисперсий
output-anova-executed = Однофакторный дисперсионный анализ выполнен для { $count } перем.
output-col-between-groups = Между группами
output-col-within-groups = Внутри групп
output-col-total = Всего
output-col-sum-of-squares = Сумма квадратов
output-col-mean-square = Средний квадрат
output-col-f = F
output-col-levene-statistic = Статистика Левена
output-col-df1 = df1
output-col-df2 = df2
output-col-std-error = Стандартная ошибка
status-anova-executed = Однофакторный дисперсионный анализ выполнен для { $count } перем.

# Crosstabs Dialog & Output
dialog-crosstabs-title = Таблицы сопряжённости
dialog-crosstabs-rows = Строки:
dialog-crosstabs-cols = Столбцы:
dialog-crosstabs-btn-cells = Ячейки...
dialog-crosstabs-btn-statistics = Статистики...
dialog-crosstabs-cells-title = Таблицы сопряжённости: Отображение ячеек
dialog-crosstabs-group-counts = Частоты
dialog-crosstabs-count-observed = Наблюдаемые
dialog-crosstabs-count-expected = Ожидаемые
dialog-crosstabs-count-residuals = Остатки
dialog-crosstabs-group-percentages = Проценты
dialog-crosstabs-pct-row = По строкам
dialog-crosstabs-pct-column = По столбцам
dialog-crosstabs-pct-total = От общего итога
dialog-crosstabs-stats-title = Таблицы сопряжённости: Статистики
dialog-crosstabs-stat-chisq = Хи-квадрат
dialog-crosstabs-stat-phi-cramer = Фи и V Крамера

output-crosstabs-title = Таблицы сопряжённости
output-crosstabs-executed = Таблицы сопряжённости построены для { $count } таблиц(ы).
output-crosstabs-summary-title = Сводка обработки наблюдений
output-crosstabs-table-title = Таблица сопряжённости { $row } * { $col }
output-crosstabs-chisq-title = Критерии хи-квадрат
output-crosstabs-symmetric-title = Симметричные меры
output-crosstabs-cases = Наблюдения
output-crosstabs-valid = Валидные
output-crosstabs-missing = Пропущенные
output-crosstabs-total = Всего
output-crosstabs-count = Количество
output-crosstabs-expected-count = Ожидаемое количество
output-crosstabs-pct-within-row = % в { $var }
output-crosstabs-pct-within-col = % в { $var }
output-crosstabs-pct-of-total = % от общего
output-crosstabs-residual = Остаток
output-crosstabs-value = Значение
output-crosstabs-df = ст.св.
output-crosstabs-asymp-sig = Асимпт. знач. (двусторонняя)
output-crosstabs-exact-sig = Точная знач. (двусторонняя)
output-crosstabs-approx-sig = Приближенная значимость
output-crosstabs-pearson-chisq = Хи-квадрат Пирсона
output-crosstabs-continuity-corr = Поправка на непрерывность
output-crosstabs-likelihood-ratio = Отношение правдоподобия
output-crosstabs-fisher-exact = Точный критерий Фишера
output-crosstabs-n-valid-cases = Количество валидных наблюдений
output-crosstabs-nominal-by-nominal = Номинальная с номинальной
output-crosstabs-phi = Фи
output-crosstabs-cramer-v = V Крамера

# Bivariate Correlations Dialog & Output
dialog-correlations-title = Парные корреляции
dialog-correlations-vars = Переменные:
dialog-correlations-coeff-group = Коэффициенты корреляции
dialog-correlations-pearson = Пирсон
dialog-correlations-kendall = тау-b Кендалла
dialog-correlations-spearman = Спирмен
dialog-correlations-sig-test-group = Критерий значимости
dialog-correlations-two-tailed = Двусторонний
dialog-correlations-one-tailed = Односторонний
dialog-correlations-flag-sig = Помечать значимые корреляции
dialog-correlations-btn-options = Параметры...
dialog-correlations-options-title = Парные корреляции: Параметры
dialog-correlations-stats-group = Статистики
dialog-correlations-stat-means = Средние и стандартные отклонения
dialog-correlations-stat-cov = Суммы квадратов отклонений и ковариации
dialog-correlations-missing-group = Пропущенные значения
dialog-correlations-missing-pairwise = Исключать наблюдения попарно
dialog-correlations-missing-listwise = Исключать наблюдения целиком

output-correlations-title = Корреляции
output-correlations-nonpar-title = Непараметрические корреляции
output-correlations-executed = Анализ корреляций выполнен для { $count } переменной(ых).
output-correlations-table-title = Корреляции
output-correlations-descriptives-title = Описательные статистики
output-correlations-pearson-r = Корреляция Пирсона
output-correlations-sig-2tailed = Знач. (2-сторон)
output-correlations-sig-1tailed = Знач. (1-сторон)
output-correlations-ss-cross = Сумма квадратов отклонений
output-correlations-covariance = Ковариация
output-correlations-n = N
output-correlations-kendall-tau = тау_b Кендалла
output-correlations-spearman-rho = ро Спирмена
output-correlations-footnote-sig05-2t = *. Корреляция значима на уровне 0.05 (двусторонняя).
output-correlations-footnote-sig01-2t = **. Корреляция значима на уровне 0.01 (двусторонняя).
output-correlations-footnote-sig05-1t = *. Корреляция значима на уровне 0.05 (односторонняя).
output-correlations-footnote-sig01-1t = **. Корреляция значима на уровне 0.01 (односторонняя).

# Диалог двух независимых выборок (Манна-Уитни)
dialog-npar-two-indep-title = Критерии для двух независимых выборок
dialog-npar-test-variables = Список тестируемых переменных:
dialog-npar-grouping-var = Группирующая переменная:
dialog-npar-define-groups-btn = Задать группы...
dialog-npar-define-groups-title = Две независимые выборки: Задать группы
dialog-npar-group1 = Группа 1:
dialog-npar-group2 = Группа 2:
dialog-npar-test-type = Тип критерия
dialog-npar-test-mann-whitney = U Манна-Уитни
dialog-npar-test-wilcoxon = W Вилкоксона
dialog-npar-test-ks = Z Колмогорова-Смирнова

# Вывод критерия Манна-Уитни в Output Viewer
output-npar-mann-whitney-title = Непараметрические критерии: Критерий Манна-Уитни
output-npar-mann-whitney-executed = U-критерий Манна-Уитни выполнен для { $count } перем.
output-npar-ranks-title = Ранги
output-npar-test-stats-title = Статистика критерия
output-col-mean-rank = Средний ранг
output-col-sum-ranks = Сумма рангов
output-row-mann-whitney-u = U Манна-Уитни
output-row-wilcoxon-w = W Вилкоксона
output-row-z = Z
output-row-asymp-sig = Асимпт. знач. (2-сторонняя)
output-row-grouping-var = Группирующая переменная

# Two Related Samples (Wilcoxon Signed Ranks / Sign Test) Dialog
dialog-npar-two-related-title = Критерии для двух связанных выборок
dialog-npar-related-test-pairs = Пары переменных:
dialog-npar-related-current-selections = Текущий выбор
dialog-npar-related-var1 = Переменная 1
dialog-npar-related-var2 = Переменная 2
dialog-npar-related-wilcoxon = Вилкоксона
dialog-npar-related-sign = Знаков

# Two Related Samples Output Tables
output-npar-related-title = Непараметрические критерии: критерий Вилкоксона / знаков
output-npar-related-executed = Анализ двух связанных выборок выполнен для { $count } пар(ы).
output-npar-ranks-negative = Отрицательные ранги
output-npar-ranks-positive = Положительные ранги
output-npar-ranks-ties = Совпадающие
output-npar-footnote-wilcoxon = Критерий знаковых рангов Вилкоксона
output-npar-footnote-based-negative = На основе отрицательных рангов.
output-npar-footnote-based-positive = На основе положительных рангов.
output-npar-sign-frequencies = Частоты
output-npar-sign-negative-diff = Отрицательные разности
output-npar-sign-positive-diff = Положительные разности
output-npar-sign-ties = Совпадения
output-npar-exact-sig-2tailed = Точн. знач. (2-сторонняя)
output-npar-footnote-sign = Критерий знаков

# Tests for Several Independent Samples (Kruskal-Wallis H) Dialog
dialog-npar-k-indep-title = Критерии для K независимых выборок
dialog-npar-define-range-btn = Задать диапазон...
dialog-npar-define-range-title = Несколько независимых выборок: Задать диапазон
dialog-npar-range-group-box = Диапазон для группирующей переменной
dialog-npar-range-min = Минимум:
dialog-npar-range-max = Максимум:
dialog-npar-test-kruskal-wallis = H Крускала-Уоллиса

# Several Independent Samples Output Tables
output-npar-kruskal-title = Непараметрические критерии: Критерий Крускала-Уоллиса
output-npar-kruskal-executed = H-критерий Крускала-Уоллиса выполнен для { $count } перем.
output-row-kruskal-wallis-h = H Крускала-Уоллиса
output-row-df = ст.св.
output-footnote-kruskal-a = a. Критерий Крускала-Уоллиса

# Compute Variable Dialog & Transform
dialog-compute-title = Вычислить переменную
dialog-compute-target-var = Целевая переменная:
dialog-compute-type-label = Тип и метка...
dialog-compute-label = Метка:
dialog-compute-expression = Численное выражение:
dialog-compute-func-group = Группа функций:
dialog-compute-functions = Функции и спец. переменные:
dialog-compute-group-all = Все
dialog-compute-group-arithmetic = Арифметика
dialog-compute-group-statistical = Статистические
output-compute-executed = Вычисление переменной '{ $target }' успешно выполнено ({ $rows } наблюдений).

# Shift Values Dialog
dialog-shift-title = Сдвиг значений (Lead / Lag)
dialog-shift-source-var = Исходная переменная:
dialog-shift-target-var = Имя новой переменной:
dialog-shift-target-label = Метка переменной (опционально):
dialog-shift-direction-group = Направление сдвига
dialog-shift-direction-lag = Lag (значения из прошлого / вниз)
dialog-shift-direction-lead = Lead (значения из будущего / вверх)
dialog-shift-step = Число шагов сдвига:
dialog-shift-error-source-required = Выберите исходную переменную.
dialog-shift-error-target-invalid = Введите корректное имя новой переменной.
output-shift-executed = Сдвиг значений '{ $target }' успешно выполнен ({ $rows } наблюдений).

# Rank Cases Dialog
dialog-rank-title = Ранжирование наблюдений
dialog-rank-variables = Переменные:
dialog-rank-direction-group = Присвоить ранг 1
dialog-rank-direction-smallest = Наименьшему значению
dialog-rank-direction-largest = Наибольшему значению
dialog-rank-ties-group = Связи
dialog-rank-ties-button = Связи...
dialog-rank-ties-title = Ранжирование: Связи
dialog-rank-ties-mean = Средний
dialog-rank-ties-low = Наименьший
dialog-rank-ties-high = Наибольший
dialog-rank-types-button = Типы рангов...
dialog-rank-types-title = Ранжирование: типы рангов
dialog-rank-type-simple = Ранг
dialog-rank-type-fractional = Дробный ранг
dialog-rank-type-percent = Дробный ранг в процентах
dialog-rank-type-ntiles = Квантили (N-тили)
dialog-rank-ntiles-groups = Число групп:
dialog-rank-type-blom = Нормальные оценки Блома
dialog-rank-type-tukey = Нормальные оценки Тьюки
dialog-rank-type-rankit = Ранкиты
dialog-rank-type-vw = Нормальные оценки Ван дер Вардена
output-rank-executed = Ранжирование наблюдений: создано { $count } переменных для { $rows } наблюдений.

# Recode Variables Dialog & Transform
dialog-recode-diff-title = Перекодировать в другие переменные
dialog-recode-same-title = Перекодировать в те же переменные
dialog-recode-vars-output-vars = Исходная переменная -> Выходная переменная:
dialog-recode-output-var-group = Выходная переменная
dialog-recode-output-name = Имя:
dialog-recode-output-label = Метка:
dialog-recode-btn-change = Изменить
dialog-recode-btn-old-new = Старые и новые значения...
dialog-recode-vars-list = Переменные:
dialog-recode-old-new-title = Перекодирование: старые и новые значения
dialog-recode-old-val-group = Старое значение
dialog-recode-new-val-group = Новое значение
dialog-recode-old-val = Значение:
dialog-recode-old-sysmis = Системный пропуск
dialog-recode-old-missing = Системные или пользовательские пропуски
dialog-recode-old-range = Диапазон:
dialog-recode-through = по
dialog-recode-old-range-lowest = Диапазон: от НАИМЕНЬШЕГО по:
dialog-recode-old-range-highest = Диапазон: от значения по НАИБОЛЬШЕЕ:
dialog-recode-old-all-other = Все остальные значения
dialog-recode-new-val = Значение:
dialog-recode-new-sysmis = Системный пропуск
dialog-recode-new-copy = Скопировать старые значения
dialog-recode-btn-add = Добавить
dialog-recode-btn-remove = Удалить
dialog-recode-old-new-mappings-title = Старое -> Новое:
output-recode-executed = Перекодирование успешно выполнено для { $count } переменных.
output-autorecode-executed = Автоматическое перекодирование успешно выполнено для { $count } переменных.
dialog-autorecode-title = Автоматическое перекодирование
dialog-autorecode-variables = Исходная переменная -> Новое имя:
dialog-autorecode-new-name = Новое имя:
dialog-autorecode-change = Изменить имя
dialog-autorecode-order-group = Порядок перекодирования
dialog-autorecode-order-lowest = От наименьшего значения
dialog-autorecode-order-highest = От наибольшего значения
output-sort-cases-executed = Наблюдения отсортированы по { $count } переменным.
output-select-cases-executed = Выборка наблюдений выполнена ({ $mode }).
output-split-file-executed = Разделение файла выполнено ({ $mode }, переменных: { $count }).
status-filter-on = Фильтр: Включен ({ $var })
status-split-file-on = Файл разделен: ВКЛ ({ $vars })

dialog-select-cases-title = Выбрать наблюдения
dialog-select-cases-select-group = Выбрать
dialog-select-cases-all = Все наблюдения
dialog-select-cases-if = Если удовлетворяется условие
dialog-select-cases-if-btn = Если...
dialog-select-cases-condition-label = Условие
dialog-select-cases-use-filter-var = Использовать переменную-фильтр:
dialog-select-cases-none = нет
dialog-select-cases-unselected-group = Невыбранные наблюдения
dialog-select-cases-action-filter = Отфильтровать
dialog-select-cases-action-delete = Удалить
dialog-select-cases-condition-title = Выбрать наблюдения: Условие
dialog-select-cases-enter-condition = Введите логическое выражение:
dialog-select-cases-condition-hint = напр. age >= 18 & salary > 30000
dialog-select-cases-keypad = Калькулятор и логические операторы:

dialog-split-file-title = Разделить файл
dialog-split-mode-off = Анализировать все наблюдения, не создавать группы
dialog-split-mode-compare = Сравнить группы
dialog-split-mode-separate = Организовать вывод по группам
dialog-split-groups-based-on = Группировать по:
dialog-split-sort-group = Порядок сортировки
dialog-split-sort-by-variables = Сортировать файл по переменным группировки
dialog-split-file-already-sorted = Файл уже отсортирован

# Weight Cases
dialog-weight-cases-title = Взвесить наблюдения
dialog-weight-do-not-weight = Не взвешивать наблюдения
dialog-weight-cases-by = Взвесить наблюдения по
dialog-weight-frequency-var = Переменная веса (частоты):
dialog-weight-current-status = Текущий статус
dialog-weight-status-off = Не взвешивать наблюдения
dialog-weight-status-on = Взвесить наблюдения по { $var }
status-weight-on = Вес: Включен ({ $var })
status-weight-executed = Взвешивание наблюдений: { $var }
status-weight-off-executed = Взвешивание наблюдений отключено


# Linear Regression Dialog
dialog-regression-linear-title = Линейная регрессия
dialog-regression-statistics-title = Линейная регрессия: Статистики
dialog-regression-group-regression-coeffs = Коэффициенты регрессии
dialog-regression-stat-collinearity = Диагностика коллинеарности
dialog-regression-group-confidence-intervals = Доверительные интервалы
dialog-regression-label-ci-level = Уровень:
dialog-regression-label-dependent = Зависимая:
dialog-regression-label-independents = Независимые:
dialog-regression-placeholder-dep = Выберите зависимую переменную
dialog-regression-label-method = Метод:
dialog-regression-method-enter = Ввести (Enter)
dialog-regression-btn-statistics = Статистики...

# Linear Regression Output
output-regression-title = Регрессия
output-regression-table-title = Линейная регрессия
output-col-model = Модель
output-regression-variables-title = Введенные/удаленные переменные
output-regression-vars-entered = Введенные переменные
output-regression-vars-removed = Удаленные переменные
output-regression-method = Метод
output-regression-none = Нет
output-regression-dep-var-note = Зависимая переменная
output-regression-all-vars-note = Все запрошенные переменные введены.
output-regression-model-summary-title = Сводка для модели
output-regression-col-r = R
output-regression-col-r-sq = R-квадрат
output-regression-col-adj-r-sq = Скорректированный R-квадрат
output-regression-col-se-est = Стандартная ошибка оценки
output-regression-predictors-note = Предикторы
output-regression-constant = (Константа)
output-regression-anova-title = Дисперсионный анализ (ANOVA)
output-regression-row-regression = Регрессия
output-regression-row-residual = Остаток
output-regression-coeffs-title = Коэффициенты
output-regression-col-b = B
output-regression-col-se = Станд. ошибка
output-regression-col-beta = Бета
output-regression-col-tol = Толерантность
output-regression-col-vif = VIF
output-regression-executed = Рассчитана линейная регрессия для {$dep} с {$count} предиктором(-ами)


# Reliability Analysis Dialog
dialog-reliability-title = Анализ надежности
dialog-reliability-stats-title = Анализ надежности: Статистики
dialog-reliability-items = Пункты:
dialog-reliability-btn-stats = Статистики...
dialog-reliability-model-label = Модель:
dialog-reliability-model-alpha = Альфа
dialog-reliability-scale-label = Метка шкалы:
dialog-reliability-descriptives-for-group = Описательные для
dialog-reliability-stat-item = Пункта
dialog-reliability-stat-scale = Шкалы
dialog-reliability-stat-scale-if-deleted = Шкалы при исключении пункта

# Reliability Analysis Output
output-reliability-title = Анализ надежности
output-reliability-table-title = Статистика надежности шкалы
output-reliability-case-processing-title = Сводка обработки наблюдений
output-col-cases = Наблюдения
output-col-item = Пункт
output-col-variance = Дисперсия
output-col-std-dev = Станд. отклонение
output-reliability-row-valid = Валидные
output-reliability-row-excluded = Исключенные
output-reliability-listwise-note = Построчное исключение на основе всех переменных в процедуре.
output-reliability-stats-title = Статистика надежности
output-reliability-col-alpha = Альфа Кронбаха
output-reliability-col-alpha-std = Альфа Кронбаха на основе стандартизованных пунктов
output-reliability-col-n-items = Количество пунктов
output-reliability-item-stats-title = Статистика пунктов
output-reliability-item-total-stats-title = Статистика пункта и шкалы
output-reliability-col-mean-if-del = Среднее шкалы при исключении пункта
output-reliability-col-var-if-del = Дисперсия шкалы при исключении пункта
output-reliability-col-corr-r = Скорректированная корреляция пункта с общим баллом
output-reliability-col-alpha-if-del = Альфа Кронбаха при исключении пункта
output-reliability-scale-stats-title = Статистика шкалы
output-reliability-executed = Рассчитан анализ надежности для {$scale} ({$count} пунктов)

# Factor Analysis Dialog
dialog-factor-title = Факторный анализ
dialog-factor-descriptives-title = Факторный анализ: Описательные
dialog-factor-correlation-matrix-group = Матрица корреляций
dialog-factor-kmo-and-bartlett = КМО и критерий сферичности Бартлетта
dialog-factor-extraction-title = Факторный анализ: Метод выделения
dialog-factor-method-group = Метод
dialog-factor-method-pca = Метод главных компонент
dialog-factor-extract-group = Выделение
dialog-factor-based-on-eigen = На основе собственных значений >
dialog-factor-number-of-factors = Фиксированное число факторов:
dialog-factor-variables-label = Переменные:
dialog-factor-descriptives-btn = Описательные...
dialog-factor-extraction-btn = Выделение...

# Factor Analysis Output
output-factor-title = Факторный анализ
output-factor-table-title = Результаты факторного анализа
output-factor-kmo-title = КМО и критерий Бартлетта
output-factor-kmo-measure-col = Мера адекватности выборки Кайзера-Мейера-Олкина.
output-factor-value-col = Значение
output-factor-kmo-stat = Мера адекватности выборки Кайзера-Мейера-Олкина
output-factor-bartlett-approx-chi = Приближенный хи-квадрат критерия Бартлетта
output-factor-bartlett-df = ст.св.
output-factor-bartlett-sig = Знач.
output-factor-communalities-title = Общности
output-factor-variable-col = Переменная
output-factor-initial-col = Начальные
output-factor-extraction-col = Выделенные
output-factor-total-variance-title = Полная объясненная дисперсия
output-factor-component-col = Компонента
output-factor-initial-total-col = Начальный итог
output-factor-initial-pct-col = % дисперсии
output-factor-initial-cum-col = Кумулятивный %
output-factor-extract-total-col = Итог выделения
output-factor-extract-pct-col = % дисперсии
output-factor-extract-cum-col = Кумулятивный %
output-factor-component-matrix-title = Матрица компонент
output-factor-comp-col = Компонента
output-factor-executed = Рассчитан факторный анализ для {$count} наблюдений, выделено {$factors} компонент

# Means Breakdown
menu-analyze-means = Средние...
dialog-means-title = Средние
dialog-means-dependent-list = Зависимые переменные:
dialog-means-independent-list = Слой 1 из 1 / Независимые переменные:
dialog-means-options-title = Средние: Параметры
dialog-means-statistics-group = Статистики
dialog-means-statistics-anova-group = Статистики для первого слоя
dialog-means-stat-mean = Среднее
dialog-means-stat-count = Число наблюдений
dialog-means-stat-stddev = Стандартное отклонение
dialog-means-stat-variance = Дисперсия
dialog-means-stat-median = Медиана
dialog-means-stat-min = Минимум
dialog-means-stat-max = Максимум
dialog-means-stat-anova = Таблица дисперсионного анализа и эта

output-means-title = Средние
output-means-report-title = Отчет
output-means-case-summary-title = Сводка обработки наблюдений
output-means-association-title = Меры связи
output-col-included-n = Включено N
output-col-included-pct = Включено %
output-col-excluded-n = Исключено N
output-col-excluded-pct = Исключено %
output-col-total-n = Всего N
output-col-total-pct = Всего %
output-means-executed = Рассчитано сравнение средних для {$tables} таблиц, {$cases} наблюдений

# Confirm Save Dialog
dialog-confirm-save-title = Несохраненные изменения
dialog-confirm-save-message = Сохранить изменения перед продолжением?
dialog-confirm-save-btn-save = Сохранить
dialog-confirm-save-btn-dont-save = Не сохранять
dialog-confirm-save-btn-cancel = Отмена

# Project and Data Save / Open
status-project-saved = Проект успешно сохранен: { $path }
status-project-opened = Открыт проект { $name } ({ $rows } наблюдений, { $cols } переменных)
status-data-saved = Данные успешно сохранены: { $path }
status-no-dataset-to-save = Нет активного набора данных для сохранения

dialog-save-project-title = Сохранить проект
dialog-open-project-title = Открыть проект StatCraft
dialog-save-data-title = Сохранить данные как
dialog-open-data-title = Открыть файл данных
dialog-filter-all-data = Все поддерживаемые форматы (*.stc, *.sav, *.zsav, *.csv, *.tsv, *.txt, *.xlsx, *.xls)
dialog-filter-spss = Данные SPSS Statistics (*.sav, *.zsav)
dialog-filter-stc = Проект StatCraft (*.stc)
dialog-filter-csv = Текстовые данные (*.csv, *.tsv, *.txt)
dialog-open-syntax-title = Открыть файл синтаксиса SPSS
dialog-save-syntax-title = Сохранить файл синтаксиса SPSS
dialog-filter-sps = Файлы синтаксиса SPSS (*.sps, *.txt)

# Dummy Variables Dialog
dialog-dummy-title = Создать фиктивные (dummy) переменные
dialog-dummy-source-vars = Категориальные переменные:
dialog-dummy-root-prefix = Базовое имя / префикс:
dialog-dummy-omit-reference = Исключить референтную категорию (k - 1)
dialog-dummy-error-empty-vars = Выберите хотя бы одну переменную.
status-dummies-executed = Создано { $count } фиктивных переменных

# Count Values within Cases Dialog & Transform
dialog-count-title = Подсчет значений в строках
dialog-count-target-var = Целевая переменная:
dialog-count-target-label = Метка целевой переменной:
dialog-count-variables = Переменные:
dialog-count-define-values = Значения...
dialog-count-values-title = Подсчет значений в строках: Значения
dialog-count-values-to-count = Значения для подсчета:
dialog-count-val-value = Значение:
dialog-count-val-sysmis = Системно-пропущенные
dialog-count-val-range = Диапазон:
dialog-count-val-range-lowest = Диапазон от НАИМЕНЬШЕГО до:
dialog-count-val-range-highest = Диапазон от значения до НАИБОЛЬШЕГО:
dialog-count-through = по
dialog-count-btn-add = Добавить
dialog-count-btn-change = Изменить
dialog-count-btn-remove = Удалить
output-count-executed = Подсчет значений '{ $target }' успешно выполнен по { $variables } переменным ({ $rows } строк).

# Set Random Number Seed
dialog-seed-title = Установить зерно ДСЧ
dialog-seed-mode-random = Случайная инициализация
dialog-seed-mode-fixed = Задать начальное значение (seed)
dialog-seed-value-label = Значение (1 - 2 000 000 000):
status-random-seed-fixed = Установлено зерно ДСЧ: { $seed }
status-random-seed-random = Включена случайная инициализация ДСЧ

# Create Time Series Dialog & Transform
dialog-timeseries-title = Создать временной ряд
dialog-timeseries-new-vars = Новая переменная -> Функция:
dialog-timeseries-function-group = Функция
dialog-timeseries-fn-diff = Разности (DIFF)
dialog-timeseries-fn-sdiff = Сезонные разности (SDIFF)
dialog-timeseries-fn-sma = Простое скользящее среднее (SMA)
dialog-timeseries-fn-csum = Кумулятивная сумма (CSUM)
dialog-timeseries-order = Порядок / Интервал:
dialog-timeseries-change = Изменить
dialog-timeseries-name = Имя:
dialog-timeseries-label = Метка:
output-timeseries-executed = Создание временных рядов успешно выполнено ({ $count } рядов, { $rows } строк).

# Visual Binning Dialog & Transform
dialog-binning-title = Визуальная группировка
dialog-binning-scanned-vars = Сканированные переменные:
dialog-binning-binned-var = Сгруппированная переменная:
dialog-binning-make-cutpoints = Создать точки среза...
dialog-binning-cutpoints-table = Сетка точек среза
dialog-binning-value-label = Метка
dialog-binning-high-endpoint = Верхняя граница
dialog-binning-included = Включая (<=)
dialog-binning-excluded = Исключая (<)
dialog-binning-name = Имя:
dialog-binning-label = Метка:
dialog-binning-endpoints = Верхние границы:
dialog-binning-endpoints-inclusive = Включая (<=) - нижняя граница исключена
dialog-binning-endpoints-exclusive = Исключая (<) - нижняя граница включена
dialog-binning-add-cutpoint = Добавить срез
dialog-binning-remove-cutpoint = Удалить
dialog-binning-clear-all = Очистить срезы
dialog-binning-grid-bin = Бин
dialog-binning-grid-value = Значение
dialog-binning-grid-high = Верхняя граница
dialog-binning-grid-label = Метка
dialog-binning-make-cutpoints-title = Создание точек среза
dialog-binning-equal-width = Интервалы равной ширины
dialog-binning-equal-percentiles = Равные процентили (на основе наблюдений)
dialog-binning-stddev = Точки среза на основе стандартных отклонений
dialog-binning-stddev-desc = +/- 1 и +/- 2 станд. откл.
dialog-binning-num-intervals = Число точек среза:
dialog-binning-cutpoint-val = Верхняя граница:
dialog-binning-high = Верх
output-binning-executed = Визуальная группировка '{ $target }' успешно создана ({ $rows } строк).

# Replace Missing Values Dialog & Transform
dialog-missing-title = Замена пропущенных значений
dialog-missing-new-vars = Новая переменная -> Метод:
dialog-missing-name = Имя:
dialog-missing-method-group = Метод
dialog-missing-method-smean = Среднее серии (Series mean)
dialog-missing-method-nearby-mean = Среднее соседних точек
dialog-missing-method-nearby-median = Медиана соседних точек
dialog-missing-method-linear = Линейная интерполяция
dialog-missing-method-trend = Линейный тренд в точке
dialog-missing-span = Число соседних точек:
dialog-missing-change = Изменить
output-missing-executed = Замена пропущенных значений успешно выполнена ({ $count } переменных, { $rows } наблюдений).
status-missing-executed = Замена пропущенных значений выполнена для { $count } переменных



# Date and Time Wizard Dialog
dialog-datetime-wizard-title = Мастер даты и времени
dialog-datetime-task-group = Выберите действие:
dialog-datetime-task-extract = Извлечь часть даты или времени
dialog-datetime-task-diff = Рассчитать интервал между двумя датами
dialog-datetime-task-add = Прибавить или вычесть длительность
dialog-datetime-task-construct = Преобразовать в стандартную дату
dialog-datetime-part-label = Извлекаемая часть:
dialog-datetime-unit-label = Единица времени:
dialog-datetime-first-date = Первая дата:
dialog-datetime-second-date = Вторая дата:
dialog-datetime-target-var = Целевая переменная:
dialog-datetime-target-label = Метка переменной:
dialog-datetime-part-year = Год
dialog-datetime-part-month = Месяц
dialog-datetime-part-day = День
dialog-datetime-part-hour = Час
dialog-datetime-part-minute = Минута
dialog-datetime-part-second = Секунда
dialog-datetime-part-dayofweek = День недели
dialog-datetime-part-quarter = Квартал
dialog-datetime-unit-years = Годы
dialog-datetime-unit-months = Месяцы
dialog-datetime-unit-weeks = Недели
dialog-datetime-unit-days = Дни
dialog-datetime-unit-hours = Часы
dialog-datetime-unit-minutes = Минуты
dialog-datetime-unit-seconds = Секунды
dialog-datetime-duration-amount = Значение / Длительность:
status-datetime-executed = Мастер даты и времени: создана переменная {$target}

# Optimal Binning Dialog & Transform
dialog-optbin-title = Оптимальная группировка
dialog-optbin-variables = Переменные для группировки:
dialog-optbin-target = Целевая переменная (категориальная):
dialog-optbin-options = Параметры
dialog-optbin-max-bins = Максимальное число интервалов:
dialog-optbin-prefix = Префикс новых переменных:
dialog-optbin-error-target = Выберите категориальную целевую переменную.
output-optbin-executed = Оптимальная группировка создала { $count } перем.
status-optbin-executed = Оптимальная группировка создала { $count } перем.


# Automated Data Preparation (ADP)
dialog-adp-title = Подготовка данных для моделирования (ADP)
dialog-adp-screening-title = ADP: Скрининг качества и выбросы
dialog-adp-transform-title = ADP: Масштабирование и категории
dialog-adp-partition-title = ADP: Разбиение выборки
dialog-adp-target-label = Целевая переменная:
dialog-adp-inputs-label = Входные признаки (предикторы):
dialog-adp-group-objective = Цель анализа
dialog-adp-group-screening = Фильтры скрининга качества
dialog-adp-max-missing-pct = Максимум пропущенных значений:
dialog-adp-max-category-pct = Максимум преобладающей категории (моды):
dialog-adp-max-unique-pct = Максимум уникальных значений (ID):
dialog-adp-group-outliers = Обработка выбросов
dialog-adp-outliers-none = Без обработки
dialog-adp-outliers-zscore = Стандартное отклонение (Z-Score)
dialog-adp-outliers-iqr = Межквартильный размах (IQR)
dialog-adp-outliers-threshold = Граница отсечения (сигм):
dialog-adp-outliers-multiplier = Множитель IQR:
dialog-adp-outliers-action = Действие с выбросами:
dialog-adp-outliers-winsorize = Винзоризация (обрезка до границ)
dialog-adp-outliers-sysmis = Замена на системные пропуски
dialog-adp-group-rescale = Масштабирование непрерывных признаков
dialog-adp-rescale-none = Без масштабирования
dialog-adp-rescale-zscore = Z-стандартизация
dialog-adp-rescale-minmax = Min-Max [0, 1]
dialog-adp-group-categories = Категориальные признаки
dialog-adp-merge-sparse-label = Объединять редкие категории (< 5%) в 'Other'
dialog-adp-group-advanced = Конструирование признаков
dialog-adp-bin-continuous-label = Супервизорный MDLP-биннинг относительно таргета
dialog-adp-interactions-label = Создавать парные кросс-произведения (взаимодействия)
dialog-adp-partition-enable-label = Разбить выборку для валидации моделей
dialog-adp-partition-train-label = Обучающая выборка:
dialog-adp-partition-test-label = Тестовая выборка:
dialog-adp-partition-validate-label = Валидационная выборка:
dialog-adp-partition-seed-label = Зерно случайных чисел:
dialog-adp-btn-screening = Скрининг...
dialog-adp-btn-transform = Преобразования...
dialog-adp-btn-partition = Разбиение...

# Output Viewer ADP Tables
output-adp-title = Подготовка данных для моделирования
output-adp-summary-title = Сводка автоматической подготовки данных
output-adp-summary-property = Параметр
output-adp-summary-value = Значение
output-adp-obj-balance = Баланс скорости и точности
output-adp-obj-accuracy = Оптимизация точности
output-adp-obj-speed = Оптимизация скорости
output-adp-obj-custom = Пользовательская настройка
output-adp-none = Нет
output-adp-target-label = Целевая переменная
output-adp-objective-label = Цель анализа
output-adp-total-inputs-label = Всего входных полей
output-adp-recommended-label = Рекомендовано к использованию
output-adp-screened-out-label = Отсеяно скринингом
output-adp-action-log-title = Журнал обработки признаков
output-adp-col-field = Поле
output-adp-col-status = Статус
output-adp-col-transformed = Новое поле
output-adp-col-details = Детали обработки
output-adp-status-excluded = Исключено
output-adp-status-recommended = Рекомендовано
output-adp-predictive-power-title = Предиктивная сила признаков
output-adp-col-predictor = Предиктор
output-adp-col-method = Метод связи
output-adp-col-importance = Важность
output-adp-col-sig = Знач.
output-adp-partition-title = Распределение по выборкам
output-adp-col-partition = Выборка
output-adp-col-n = N
output-adp-col-pct = Процент
output-adp-partition-train = Обучающая
output-adp-partition-test = Тестовая
output-adp-partition-validate = Валидационная
output-adp-partition-total = Всего
status-adp-executed = Подготовка данных (ADP): обработано {$total} полей ({$recommended} рекомендовано)

# Programmability Transformations Dialog
dialog-programmability-title = Программируемые преобразования
dialog-programmability-variables = Переменные:
dialog-programmability-script-label = Скрипт преобразований:
dialog-programmability-snippet-formula = + Формула
dialog-programmability-snippet-ifelse = + Условие IF / ELSE
dialog-programmability-snippet-normalize = + Нормализация
dialog-programmability-snippet-math = + Мат. функция
dialog-programmability-status-valid = Синтаксис корректен
dialog-programmability-rules-count = команд(а)
dialog-programmability-status-empty = Скрипт пуст
dialog-programmability-status-error = Ошибка синтаксиса

# Output Viewer Programmability Tables
output-programmability-title = Программируемые преобразования
output-programmability-summary-title = Сводка выполнения скрипта
output-programmability-property = Параметр
output-programmability-value = Значение
output-programmability-prop-statements = Выполнено операторов
output-programmability-prop-created = Созданные переменные
output-programmability-prop-modified = Измененные переменные
output-programmability-prop-rows = Обработано строк
output-programmability-prop-status = Статус выполнения
output-programmability-status-ok = Успешно
output-programmability-none = Нет
output-programmability-logs-title = Журнал действий скрипта
output-programmability-log-num = №
output-programmability-log-detail = Действие
output-programmability-executed = Программируемые преобразования завершены: выполнено операторов: { $statements }, строк: { $rows }.

# AI MCP & Socket Bridge
dialog-mcp-bridge-title = Центр управления AI MCP и сокет
dialog-mcp-bridge-status-active = Активен (127.0.0.1:{ $port })
dialog-mcp-bridge-status-stopped = Остановлен
dialog-mcp-bridge-status-port-busy = Порт { $port } занят первичным экземпляром StatCraft
dialog-mcp-bridge-btn-start = Запустить мост
dialog-mcp-bridge-btn-stop = Остановить мост
dialog-mcp-bridge-btn-retry = Повторить попытку
dialog-mcp-bridge-autostart = Запускать автоматически при старте программы
dialog-mcp-bridge-config-title = Подключение ИИ-агентов (MCP Stdio):
dialog-mcp-bridge-copy-config = Скопировать конфиг MCP
dialog-mcp-bridge-copy-skill = Скопировать инструкцию (Skill)
dialog-mcp-bridge-copied = Скопировано в буфер обмена!
dialog-mcp-bridge-log-title = Журнал активности моста (Live):
dialog-mcp-bridge-clear-log = Очистить журнал
dialog-mcp-bridge-log-empty = Событий пока нет. Мост ожидает подключения агента.

bridge-log-client-connected = { $time } [Подключение] Агент подключился ({ $addr })
bridge-log-client-disconnected = { $time } [Отключение] Сессия агента завершена ({ $addr })
bridge-log-vars-requested = { $time } [Инспекция] Передан словарь переменных ({ $count } колонок)
bridge-log-preview-requested = { $time } [Инспекция] Передан предпросмотр данных ({ $limit } строк)
bridge-log-dataset-loaded = { $time } [Данные] Загружен массив данных ({ $rows } строк, { $cols } колонок)
bridge-log-syntax-executed = { $time } [Синтаксис] Выполнен синтаксис: { $cmd } (команд: { $count })
bridge-log-note-added = { $time } [Отчет] Добавлено аналитическое примечание: "{ $title }"
bridge-log-tab-switched = { $time } [Интерфейс] Переключена вкладка на "{ $tab }"
bridge-log-server-started = { $time } [Сервер] Мост запущен на { $addr }
bridge-log-server-stopped = { $time } [Сервер] Мост остановлен
bridge-log-error = { $time } [Ошибка] { $message }


# Find and Replace Dialog
dialog-find-replace-title = Найти и заменить
dialog-find-tab-find = Найти
dialog-find-tab-replace = Заменить
dialog-find-what = Найти:
dialog-find-replace-with = Заменить на:
dialog-find-match-case = С учетом регистра
dialog-find-match-entire = Точное совпадение ячейки
dialog-find-direction = Направление поиска
dialog-find-dir-down = Вперед
dialog-find-dir-up = Назад
dialog-find-column = Искать в:
dialog-find-col-all = (Все переменные)
dialog-find-btn-find-next = Найти далее
dialog-find-btn-replace = Заменить
dialog-find-btn-replace-all = Заменить все
dialog-find-btn-close = Закрыть
dialog-find-status-not-found = Искомая строка не найдена.
dialog-find-status-found = Найдено в строке { $row }, переменная { $var }.
dialog-find-status-replaced = Заменено совпадений: { $count }.

# Go To Dialog
dialog-goto-title = Переход
dialog-goto-tab-case = Наблюдение
dialog-goto-tab-variable = Переменная
dialog-goto-case-number = Номер наблюдения:
dialog-goto-case-range = (от 1 до { $total })
dialog-goto-var-filter = Фильтр:

# Edit Operations Status
status-case-inserted = Вставлено наблюдение в строке { $row }.
status-var-inserted = Вставлена переменная '{ $name }'.
status-cell-copied = Скопировано в буфер обмена.
status-cell-cut = Вырезано в буфер обмена.
status-cell-pasted = Вставлено из буфера обмена.
status-cell-cleared = Ячейка очищена.


# Explore / EXAMINE
dialog-explore-title = Исследовать
dialog-explore-dependent-list = Зависимые переменные:
dialog-explore-factor-list = Список факторов:
dialog-explore-normality-tests = Графики и критерии нормальности
dialog-explore-display = Вывод
dialog-explore-both = Оба
dialog-explore-statistics = Статистики
dialog-explore-plots = Графики
dialog-explore-statistics-title = Исследовать: Статистики
dialog-explore-descriptives = Описательные статистики
dialog-explore-extremes = Выбросы / Экстремальные значения
dialog-explore-confidence-interval = Доверительный интервал среднего:
dialog-explore-options-title = Исследовать: Параметры
dialog-explore-missing-values = Пропущенные значения
dialog-explore-exclude-listwise = Исключать наблюдения целиком
dialog-explore-exclude-pairwise = Исключать наблюдения попарно

output-explore-title = Исследовать
output-explore-case-processing = Сводка обработки наблюдений
output-explore-descriptives = Описательные статистики
output-explore-normality = Критерии нормальности
output-explore-extremes = Экстремальные значения
output-explore-cases = Наблюдения
output-explore-valid = Допустимые
output-explore-missing = Пропущенные
output-explore-total = Всего
output-explore-percent = Процент
output-explore-statistic = Статистика
output-explore-std-error = Стандартная ошибка
output-explore-mean = Среднее
output-explore-ci-95 = 95% доверительный интервал для среднего
output-explore-lower-bound = Нижняя граница
output-explore-upper-bound = Верхняя граница
output-explore-5-trimmed-mean = 5% усеченное среднее
output-explore-median = Медиана
output-explore-variance = Дисперсия
output-explore-std-deviation = Стандартное отклонение
output-explore-minimum = Минимум
output-explore-maximum = Максимум
output-explore-range = Размах
output-explore-iqr = Межквартильный размах
output-explore-skewness = Асимметрия
output-explore-kurtosis = Эксцесс
output-explore-kolmogorov-smirnov = Колмогорова-Смирнова
output-explore-shapiro-wilk = Шапиро-Уилка
output-explore-df = ст.св.
output-explore-sig = Знач.
output-explore-lilliefors-footnote = *. Это нижняя граница истинной значимости.
output-explore-lilliefors-correction = a. Поправка значимости Лиллиефорса
output-explore-highest = Наибольшие
output-explore-lowest = Наименьшие
output-explore-case-number = Номер наблюдения
output-explore-order = Порядок
output-explore-value = Значение

# Partial Correlations Dialog
dialog-partial-corr-title = Частные корреляции
dialog-partial-corr-variables = Переменные:
dialog-partial-corr-controlling = Контрольные переменные:
dialog-partial-corr-test-sig = Критерий значимости
dialog-partial-corr-two-tailed = Двусторонний
dialog-partial-corr-one-tailed = Односторонний
dialog-partial-corr-zero-order = Показывать корреляции нулевого порядка

# Logistic Regression Dialog
dialog-logistic-regression-title = Бинарная логистическая регрессия
dialog-logistic-dependent = Зависимая:
dialog-logistic-covariates = Ковариаты:
dialog-logistic-placeholder-dep = Выберите бинарную зависимую переменную
dialog-logistic-options = Параметры
dialog-logistic-hosmer-lemeshow = Критерий согласия Хосмера-Лемешова
dialog-logistic-ci = ДИ для Exp(B):
dialog-logistic-cutoff = Порог классификации:
dialog-logistic-dep-binary-ok = (бинарная, 2 значения)
dialog-logistic-dep-not-binary-hint = ({ $count } значений — требуется ровно 2)
dialog-logistic-warning-not-binary = Зависимая переменная имеет { $count } значений (требуется ровно 2). Для 3+ категорий используйте «Мультиномиальную логистическую регрессию».
dialog-logistic-warning-insufficient = Зависимая переменная не содержит данных или является константой (найдено значений: { $count }).

# Диалог мультиномиальной логистической регрессии
dialog-multinomial-logistic-title = Мультиномиальная логистическая регрессия
dialog-multinomial-dependent = Зависимая:
dialog-multinomial-factors = Фактор(ы):
dialog-multinomial-covariates = Ковариата(ы):
dialog-multinomial-dep-categories = ({ $count } категорий)
dialog-multinomial-warning-insufficient = Зависимая переменная должна иметь не менее 2 категорий (найдено: { $count }).
dialog-multinomial-btn-ref-cat = Базовая категория...
dialog-multinomial-btn-statistics = Статистика...
dialog-multinomial-ref-title = Базовая категория
dialog-multinomial-ref-first = Первая категория
dialog-multinomial-ref-last = Последняя категория
dialog-multinomial-ref-custom = Заданное значение
dialog-multinomial-stats-title = Статистика
dialog-multinomial-opt-classification = Таблица классификации
dialog-multinomial-opt-goodness-of-fit = Критерии согласия
dialog-multinomial-opt-lrt = Критерии отношения правдоподобия
dialog-multinomial-opt-constant = Включить константу в уравнение
dialog-multinomial-confidence-level = Доверительный интервал (%):

# Curve Estimation Dialog
dialog-curvefit-title = Оценка кривых
dialog-curvefit-dependent = Зависимая:
dialog-curvefit-independent = Независимая:
dialog-curvefit-placeholder-dep = Выберите зависимую переменную
dialog-curvefit-placeholder-indep = Выберите независимую переменную
dialog-curvefit-models = Модели
dialog-curvefit-linear = Линейная
dialog-curvefit-logarithmic = Логарифмическая
dialog-curvefit-inverse = Обратная
dialog-curvefit-quadratic = Квадратичная
dialog-curvefit-power = Степенная
dialog-curvefit-exponential = Экспоненциальная
dialog-curvefit-include-constant = Включить константу в уравнение

# Partial Correlations Output
output-partial-corr-title = Частные корреляции
output-partial-corr-table-title = Корреляции
output-partial-corr-none = --нет--
output-partial-corr-control-vars = Контрольные переменные
output-partial-corr-correlation = Корреляция
output-partial-corr-sig-2tailed = Знач. (2-сторон)
output-partial-corr-sig-1tailed = Знач. (1-сторон)
output-partial-corr-df = ст.св.

# Logistic Regression Output
output-logistic-title = Логистическая регрессия
output-logistic-table-title = Бинарная логистическая регрессия
output-logistic-omnibus-title = Омнибус-тесты коэффициентов модели
output-logistic-model-summary-title = Сводка для модели
output-logistic-hosmer-title = Критерий Хосмера и Лемешова
output-logistic-classification-title = Таблица классификации
output-logistic-variables-title = Переменные в уравнении
output-logistic-step = Шаг 1
output-logistic-step-row = Шаг
output-logistic-block-row = Блок
output-logistic-model-row = Модель
output-logistic-col-chi-square = Хи-квадрат
output-logistic-col-df = ст.св.
output-logistic-col-sig = Знач.
output-logistic-col-2ll = -2 Log-правдоподобие
output-logistic-col-cox-snell = R-квадрат Кокса и Снелла
output-logistic-col-nagelkerke = R-квадрат Нагелькерке
output-logistic-observed = Наблюденные
output-logistic-predicted = Предсказанные
output-logistic-pct-correct = Процент правильных
output-logistic-overall-pct = Общий процент
output-logistic-col-b = B
output-logistic-col-se = Станд. ошибка
output-logistic-col-wald = Вальд
output-logistic-col-exp-b = Exp(B)
output-logistic-col-ci-lower = Нижняя
output-logistic-col-ci-upper = Верхняя
output-logistic-col-ci-title = 95% Д.И. для EXP(B)
output-logistic-constant = Константа
output-logistic-cutoff-note = a. Пороговое значение: { $cut }

# Вывод мультиномиальной логистической регрессии
output-multinomial-title = Мультиномиальная логистическая регрессия
output-multinomial-table-title = Мультиномиальная логистическая регрессия
output-multinomial-case-processing = Сводка обработки наблюдений
output-multinomial-marginal-percent = Маргинальный процент
output-multinomial-missing = Пропущенные
output-multinomial-model = Модель
output-multinomial-category = Категория
output-multinomial-model-fitting = Информация о подгонке модели
output-multinomial-model-intercept = Только константа
output-multinomial-model-final = Итоговая модель
output-multinomial-goodness-of-fit = Критерии согласия
output-multinomial-pseudo-r2 = Псевдо R-квадрат
output-multinomial-cox-snell = Кокса и Снелла
output-multinomial-nagelkerke = Наджелкерка
output-multinomial-mcfadden = МакФаддена
output-multinomial-lrt = Критерии отношения правдоподобия
output-multinomial-effect = Эффект
output-multinomial-param-estimates = Оценки параметров
output-multinomial-std-error = Станд. ошибка
output-multinomial-wald = Вальд
output-multinomial-exp-b = Exp(B)
output-multinomial-ci-lower = 95% ДИ нижняя
output-multinomial-ci-upper = 95% ДИ верхняя
output-multinomial-classification = Классификация
output-multinomial-observed = Наблюдаемые
output-multinomial-percent-correct = Процент правильных
output-multinomial-overall-percentage = Общий процент

# Curve Estimation Output
output-curvefit-title = Оценка кривых
output-curvefit-table-title = Описание модели и оценки параметров
output-curvefit-model-summary-title = Сводка модели и оценки параметров
output-curvefit-col-equation = Уравнение
output-curvefit-col-rsq = R-квадрат
output-curvefit-col-f = F
output-curvefit-col-df1 = df1
output-curvefit-col-df2 = df2
output-curvefit-col-sig = Знач.
output-curvefit-col-constant = Константа
output-curvefit-col-b1 = b1
output-curvefit-col-b2 = b2
output-curvefit-dep-note = Зависимая переменная: { $dep }

# Status messages
status-partial-corr-executed = Вычислены частные корреляции для { $vars } с контролем { $controls }
status-logistic-executed = Вычислена бинарная логистическая регрессия для { $dep } с { $count } ковариатой(ами)
status-multinomial-executed = Мультиномиальная логистическая регрессия рассчитана для { $dep } с { $count } предиктором(ами)
status-curvefit-executed = Вычислена оценка кривых для { $dep } по { $indep }


# Диалоги общей линейной модели (GLM)
dialog-univariate-glm-title = Одномерный дисперсионный анализ (GLM)
dialog-multivariate-glm-title = Многомерный дисперсионный анализ (MANOVA)
dialog-repeated-glm-title = Дисперсионный анализ повторных измерений (GLM)
dialog-glm-dependent-var = Зависимая переменная:
dialog-glm-dependent-vars = Зависимые переменные:
dialog-glm-fixed-factors = Фиксированные факторы:
dialog-glm-covariates = Ковариаты:
dialog-options-group = Параметры
dialog-glm-intercept = Включить константу в модель
dialog-glm-descriptives = Описательные статистики
dialog-glm-homogeneity = Критерий однородности дисперсий (Ливинь)
dialog-glm-within-factor-name = Имя внутригруппового фактора:
dialog-glm-repeated-measures-list = Переменные повторных измерений:

# Вывод общей линейной модели (GLM)
output-glm-univariate-title = Одномерный дисперсионный анализ (GLM)
output-glm-multivariate-title = Многомерный дисперсионный анализ (MANOVA)
output-glm-repeated-title = Анализ повторных измерений (GLM)
output-glm-between-subjects-title = Критерии межгрупповых эффектов
output-glm-multivariate-tests-title = Многомерные критерии
output-glm-within-subjects-title = Критерии внутригрупповых эффектов
output-glm-mauchly-title = Критерий сферичности Мокли
output-glm-col-source = Источник
output-glm-col-type3-ss = Сумма квадратов типа III
output-glm-col-df = ст.св.
output-glm-col-ms = Средний квадрат
output-glm-col-f = F
output-glm-col-sig = Значимость
output-glm-col-partial-eta = Частное эта-квадрат
output-glm-col-effect = Эффект
output-glm-col-test-stat = Статистика
output-glm-col-value = Значение
output-glm-col-hypo-df = ст.св. гипотезы
output-glm-col-error-df = ст.св. ошибки
output-glm-col-mauchly-w = W Мокли
output-glm-col-chi-square = Приближ. Хи-квадрат
output-glm-col-greenhouse = Гринхаус-Гейссер
output-glm-col-huynh = Хюйнх-Фельдт
output-glm-col-lower-bound = Нижняя граница
output-glm-col-measure = Мера

# Статусные сообщения GLM
status-glm-univariate-executed = Одномерный дисперсионный анализ рассчитан для { $dep }
status-glm-multivariate-executed = Многомерный дисперсионный анализ (MANOVA) рассчитан для { $count } зависимых переменных
status-glm-repeated-executed = Дисперсионный анализ повторных измерений рассчитан для фактора { $factor } ({ $count } замеров)


# Общие ключи диалогов и вывода
dialog-common-variables = Переменные:
output-common-df = ст.св.
output-common-f = F
output-common-sig = Значимость
output-common-se = Станд. ошибка
output-common-time = Время
output-common-status = Статус
output-common-chi-square = Хи-квадрат

# Кластерный анализ методом k-средних (K-Means)
dialog-kmeans-title = Кластерный анализ методом k-средних
dialog-kmeans-selected-variables = Переменные:
dialog-kmeans-criteria = Критерии
dialog-kmeans-clusters-count = Число кластеров:
dialog-kmeans-max-iterations = Максимум итераций:
dialog-kmeans-convergence = Критерий сходимости:
dialog-kmeans-save = Сохранить
dialog-kmeans-save-cluster = Принадлежность к кластеру
dialog-kmeans-save-distance = Расстояние до центра кластера
output-kmeans-title = Быстрый кластерный анализ (Quick Cluster)
output-kmeans-initial-centers = Начальные центры кластеров
output-kmeans-final-centers = Конечные центры кластеров
output-kmeans-distances = Расстояния между конечными центрами
output-kmeans-anova = Дисперсионный анализ (ANOVA)
output-kmeans-cluster-sizes = Число наблюдений в каждом кластере
output-kmeans-col-variable = Переменная
output-kmeans-col-cluster = Кластер { $num }
output-kmeans-col-cluster-num = Кластер
output-kmeans-col-num-cases = Число наблюдений
output-kmeans-col-percent = Процент
output-kmeans-col-cluster-ms = Средний квадрат кластера
output-kmeans-col-error-ms = Средний квадрат ошибки
status-kmeans-executed = Кластеризация k-средних выполнена ({ $clusters } кластеров, { $cases } наблюдений)

# Отчет «Книга кодов» (Codebook)
dialog-codebook-title = Книга кодов
dialog-codebook-selected-variables = Переменные:
dialog-codebook-options = Параметры
dialog-codebook-include-counts = Включить частоты и проценты
dialog-codebook-include-stats = Включить описательные статистики
dialog-codebook-max-categories = Максимум категорий:
output-codebook-title = Книга кодов (Codebook)
output-codebook-var-card = Переменная: { $name }
output-codebook-col-standard-attributes = Стандартные атрибуты
output-codebook-col-value = Значение
output-codebook-col-count = Частота
output-codebook-col-percent = Процент
output-codebook-col-valid-percent = Валидный процент
output-codebook-lbl-name = Имя
output-codebook-lbl-label = Метка
output-codebook-lbl-type = Тип
output-codebook-lbl-measurement = Шкала измерения
output-codebook-lbl-valid = Валидных наблюдений
output-codebook-lbl-missing = Пропущенных наблюдений
output-codebook-lbl-mean = Среднее
output-codebook-lbl-stddev = Станд. отклонение
output-codebook-lbl-median = Медиана
output-codebook-lbl-quartiles = Квартили (25, 50, 75)
status-codebook-executed = Книга кодов сформирована для { $count } переменных

# Отчет «Итоги по наблюдениям» (Summarize)
dialog-summarize-title = Итоги по наблюдениям
dialog-summarize-variables = Переменные:
dialog-summarize-grouping = Группирующие переменные:
dialog-summarize-options = Параметры
dialog-summarize-display-cases = Выводить наблюдения
dialog-summarize-limit-cases = Ограничить первыми наблюдениями:
output-summarize-title = Итоги по наблюдениям (Case Summaries)
output-summarize-table-title = Таблица итогов по наблюдениям
output-summarize-col-case-num = Номер наблюдения
output-summarize-lbl-total = Всего
output-summarize-lbl-group-total = Итого ({ $grp })
status-summarize-executed = Итоги по наблюдениям сформированы для { $count } строк

# Анализ выживаемости Каплана-Мейера
dialog-km-title = Анализ выживаемости Каплана-Мейера
dialog-km-time-variable = Переменная времени:
dialog-km-status-variable = Переменная статуса:
dialog-km-factor-variable = Факторная переменная (необязательно):
dialog-km-event-value-group = Определение события
dialog-km-event-value = Значение события:
output-km-title = Анализ выживаемости Каплана-Мейера
output-km-case-summary = Сводка обработки наблюдений
output-km-means-medians = Средние и медианы времени выживания
output-km-log-rank-test = Общие сравнения (Лог-ранг / Мантель-Кокс)
output-km-event-table = Таблица выживаемости - { $group }
output-km-col-factor = Фактор / Страта
output-km-col-total-n = Всего N
output-km-col-events-n = Число событий
output-km-col-events-pct = % событий
output-km-col-censored-n = Цензурировано N
output-km-col-censored-pct = % цензурированных
output-km-col-estimate = Оценка
output-km-col-mean-estimate = Оценка среднего
output-km-col-mean-se = Станд. ошибка среднего
output-km-col-mean-lower = 95% ДИ нижняя
output-km-col-mean-upper = 95% ДИ верхняя
output-km-test-name = Критерий
output-km-cumulative-survival = Кумулятивная выживаемость
output-km-cum-events = Накопленные события
output-km-remaining = Оставшиеся наблюдения
status-km-executed = Анализ Каплана-Мейера завершен для { $time }

# Таблицы смертности / дожития (Life Tables)
dialog-lifetables-title = Таблицы дожития
dialog-lifetables-time-variable = Переменная времени:
dialog-lifetables-status-variable = Переменная статуса:
dialog-lifetables-intervals-group = Интервалы времени
dialog-lifetables-event-value = Значение события:
dialog-lifetables-end-time = Конечное время:
dialog-lifetables-width = Ширина интервала:
output-lifetables-title = Таблицы дожития (Life Tables)
output-lifetables-table-title = Актуарная таблица дожития
output-lifetables-interval = Интервал
output-lifetables-entering = Вошедшие в интервал
output-lifetables-withdrawn = Выбывшие
output-lifetables-exposed = Подвергшиеся риску
output-lifetables-events = Терминальные события
output-lifetables-q = Доля выбывших (q)
output-lifetables-cum-p = Кумулятивная выживаемость (P)
output-lifetables-hazard = Интенсивность риска (Hazard)
status-lifetables-executed = Анализ таблиц дожития завершен для { $time }

# Регрессия пропорциональных рисков Кокса (COXREG)
dialog-cox-title = Регрессия Кокса
dialog-cox-covariates = Ковариаты:
dialog-cox-strata-variable = Страты (необязательно):
dialog-cox-ties-group = Метод обработки совпадений
dialog-cox-ties-breslow = Бреслоу (Breslow)
dialog-cox-ties-efron = Эфрон (Efron)
output-cox-title = Регрессия Кокса
output-cox-table-title = Модель пропорциональных рисков Кокса
output-cox-case-summary-title = Сводка обработки наблюдений
output-cox-omnibus-title = Омнибус-тесты коэффициентов модели
output-cox-variables-title = Переменные в уравнении
output-cox-baseline-title = Таблица выживаемости при средних значениях ковариат
output-cox-survival-plot-title = График функции выживаемости при средних ковариатах
output-cox-baseline-cum-hazard = Базовый накопленный риск
output-cox-baseline-survival = Базовая выживаемость
output-cox-survival-at-mean = Выживаемость при средних
output-common-stratum = Страта
output-common-overall = Всего
status-cox-executed = Выполнена регрессия Кокса для переменной времени { $time }


# Package 3 Common Data and Table Labels
output-common-variable = Переменная
output-common-valid = Валидные
output-common-n = N
output-common-total = Итого
output-common-value = Значение
output-common-label = Метка
output-common-count = Частота
output-common-percent = Процент
output-common-min = Минимум
output-common-max = Максимум
output-common-mean = Среднее
output-common-stddev = Станд. отклонение
output-common-median = Медиана
output-common-group = Группа
output-common-total-n = Всего N
output-common-ci-lower = 95% ДИ нижняя
output-common-ci-upper = 95% ДИ верхняя

# Package 3 K-Means Titles & Columns
output-kmeans-table-title = Кластерный анализ методом k-средних
output-kmeans-distances-title = Расстояния между конечными центрами кластеров
output-kmeans-anova-title = Таблица дисперсионного анализа (ANOVA)
output-kmeans-cases-title = Число наблюдений в каждом кластере
output-kmeans-cluster = Кластер
output-kmeans-cluster-ms = Средний квадрат кластера
output-kmeans-error-ms = Средний квадрат ошибки

# Package 3 Reports (Codebook & Summarize)
output-codebook-table-title = Таблица книги кодов
output-codebook-position = Позиция
output-codebook-measure = Шкала измерения
output-codebook-type = Тип
output-codebook-stats-title = Описательные статистики
output-codebook-frequencies-title = Таблица частот
output-summarize-case-num = Номер наблюдения

# Package 3 Survival Analysis Titles & Labels
dialog-lifetables-thru-end = По конечное:
dialog-lifetables-by-width = С шагом:
output-km-table-title = Таблица выживаемости Каплана-Мейера
output-km-case-processing-summary = Сводка обработки наблюдений
output-km-means-medians-title = Средние и медианы времени выживания
output-km-logrank-title = Общие сравнения (Лог-ранг / Мантель-Кокс)
output-km-events-n = Число событий
output-km-censored-n = Цензурировано N
output-km-censored-pct = % цензурированных

# Grid AutoFilter & Quick Sort
grid-filter-sort-asc = Сортировать по возрастанию
grid-filter-sort-desc = Сортировать по убыванию
grid-filter-search = Поиск значений...
grid-filter-select-all = (Выбрать все)
grid-filter-blanks = (Пустые)
grid-filter-apply = Применить фильтр
grid-filter-clear = Очистить фильтр
grid-menu-filter-value = Фильтровать по значению этой ячейки
grid-menu-clear-filter = Снять фильтр с этого столбца
grid-menu-copy-range = Копировать диапазон ({ $count } яч.)
grid-menu-clear-contents = Очистить содержимое
grid-menu-insert-case = Вставить наблюдение выше
grid-menu-delete-cases = Удалить выбранные наблюдения ({ $count })
grid-menu-clear-cases = Очистить значения в наблюдениях
grid-menu-insert-variable = Вставить переменную
grid-menu-autofit-col = Автоподбор ширины столбца
grid-integrity-num-stored-as-text = Число сохранено как текст
grid-integrity-convert = Преобразовать в число

# Multi-File Workspace & Dataset Tabs
tab-untitled-dataset = Набор данных { $id }
tab-new-dataset-tooltip = Создать новый набор данных (Ctrl+N)
tab-close-tooltip = Закрыть вкладку (Ctrl+W)
tab-unsaved-badge = Несохраненные изменения
menu-file-save-project = Сохранить проект...
menu-file-save-project-as = Сохранить проект как...
menu-window-datasets = Открытые наборы данных
memory-guard-title = Предупреждение о расходе памяти
memory-guard-warning = Открытие выбранных файлов ({ $count } шт., ~{ $size } МБ) может потребовать значительного объема оперативной памяти. Свободно в системе: ~{ $free } МБ.
memory-guard-continue = Открыть всё равно
memory-guard-cancel = Отмена


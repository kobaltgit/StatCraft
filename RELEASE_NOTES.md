# StatCraft v0.4.0

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.0 is a major milestone release representing the culmination of the **Table Modernization Initiative** alongside enhanced workspace reliability. This release transforms the Data View into a high-performance, virtualized, Excel/SPSS-grade spreadsheet grid capable of handling massive datasets with fluid 60 FPS interactions, deep keyboard ergonomics, multi-column filtering, canonical sorting via the engine command bus, rectangular clipboard exchange, and automated data integrity inspection.

### 📊 Virtualized 4-Quadrant 2D Grid (Feature 006)
* **Spatial 4-Quadrant Architecture:**
  * Pinned top-left corner header (`quad_top_left`);
  * Horizontal-scrolling pinned column variable headers (`quad_top_right`);
  * Vertical-scrolling pinned row/case number headers (`quad_bottom_left`);
  * 2D-scrolling central data cell canvas (`quad_center`).
  * Row number headers remain firmly anchored to the left edge during horizontal scrolling.
* **Prefix-Sum Spatial Indexing (`LineIndex`):**
  * $O(\log N)$ binary search from screen pixels to logical rows and columns with zero heap allocations in `Uniform` mode for hundreds of thousands of cases.
* **Interactive Column Resizing & Auto-Fit:**
  * Visual drag-resizing with dynamic cursor switching (`ResizeHorizontal`) and accent guidelines;
  * Double-click on header separator triggers instant column Auto-fit based on header and cell content;
  * Two-way column width synchronization with SPSS dictionary metadata (`VariableMetadata.columns`).

### 🖱️ Range Selection, Keyboard Navigation & Direct Typing (Feature 007)
* **Multi-Cell Range Selection:**
  * Fluid marquee box selection via mouse dragging or Shift+click / Shift+drag;
  * Vector selection rendering with 1.5 px accent perimeter border, non-blocking transparent cell background highlighting, and active cursor cell differentiation.
* **Full Keyboard Ergonomics:**
  * Navigation via Arrow keys, `Tab` / `Shift+Tab`, `Enter` / `Shift+Enter`, `Home` / `End`, `Ctrl+Home` / `Ctrl+End`;
  * Dynamic range expansion via `Shift` + Arrow keys;
  * **Direct Typing:** Immediately typing alphanumeric characters activates the inline cell editor without requiring pre-clicking or pressing Enter;
  * Cell contents clearance via `Delete` or `Backspace`.
* **Focused Inline Cell Editor:**
  * In-place single-line `TextEdit` with visual focus ring and automatic commit on Enter/Tab or cancellation on Escape.

### 📋 Rectangular TSV/CSV Clipboard & Auto-Fill Series (Feature 007)
* **Native Rectangular Clipboard Transfer:**
  * Bidirectional TSV/CSV clipboard serialization (`Ctrl+C`) and streaming parser (`Ctrl+V`);
  * Multi-cell pasting with automatic row expansion when pasting external matrices.
* **Series Auto-Fill (Fill Handle):**
  * $6 \times 6$ px square Fill Handle marker pinned to the bottom-right corner of selection;
  * Dragging the handle computes linear regression slopes for numeric sequences (`[1, 2, 3] -> [4, 5, 6]`) and cyclic repetition for categorical text.

### 🔍 Column AutoFilter, Dropdowns & Rapid Sorting (Feature 008)
* **Interactive Header AutoFilter:**
  * Vector filter button in each column header (discrete downward chevron in idle state, filled accent funnel icon when filtered);
  * Floating filter popup with live search, "Select All" / "Clear All" bulk toggles, checkbox checklist of unique values, and explicit "(Blanks)" handling;
  * Dynamic evaluation of case eligibility via `DataSet::is_row_matched_by_filter`.
* **Quick Sorting via CommandBus:**
  * Instant Sort Ascending (A–Z) and Sort Descending (Z–A) from column filter and cell context menus;
  * Executes as canonical engine command `data.sort_cases` with 100% reversible Undo/Redo (`Ctrl+Z` / `Ctrl+Y`) and full SPSS syntax logging (`SORT CASES BY <var> (A/D).`).

### 🎯 Cell Context Menu, SPSS FILTER BY Visuals & Data Integrity (Feature 008)
* **Cell Right-Click Context Menu:**
  * "Filter by selected cell value", "Clear column filter", "Sort Ascending / Descending", and quick clipboard operations ("Copy", "Paste").
* **SPSS FILTER BY Visualization Parity:**
  * Filtered / inactive cases are not deleted: row numbers display a diagonal vector strikethrough, and cell values render in muted gray (`Tokens::text_muted()`), automatically excluding them from analytical procedures.
* **Data Integrity Guards ("Numbers Stored as Text"):**
  * Automated detector `is_number_stored_as_text` identifies numbers stored inside text fields;
  * Excel-parity green corner triangle indicator in cell's top-left corner;
  * Single click on badge converts value to `RawValue::Number`, preventing calculation errors.

### 🗂️ Workspace Tab Lifecycle Management (Feature 005)
* Core workspace tabs (**Data View**, **Variable View**, **Output Viewer**) are marked non-closeable (`is_closeable = false`), protecting users from accidental tab loss;
* Auxiliary tabs (Syntax Editor) close cleanly with standard tab close buttons;
* "Window" menu provides quick restoration and single-click switching;
* Running SPSS syntax automatically ensures Output Viewer visibility before executing batches.

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive (silent GUI subsystem launch without terminal window).
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.0 — это масштабное обновление, знаменующее завершение комплексной инициативы **модернизации таблиц данных (Table Modernization Initiative)** и существенное повышение надежности рабочей среды. Данный релиз превращает Data View в высокопроизводительный, виртуализированный табличный процессор уровня Excel и IBM SPSS, способный отображать и плавно прокручивать (60 FPS) массивы из сотен тысяч наблюдений, с поддержкой сквозной клавиатурной навигации, автофильтрации колонок, канонической сортировки через шину команд, прямоугольного буфера обмена и автоматического контроля типов данных.

### 📊 Виртуализированная 4-квадрантная 2D-сетка (Feature 006)
* **Пространственная 4-квадрантная архитектура:**
  * Фиксированный верхний левый заголовок (`quad_top_left`);
  * Колоночные заголовки переменных, закрепленные сверху и скроллящиеся по горизонтали (`quad_top_right`);
  * Номера строк/наблюдений, закрепленные слева и скроллящиеся по вертикали (`quad_bottom_left`);
  * Центральная область ячеек данных со скроллингом в 2D (`quad_center`).
  * Номера строк больше не уплывают за левый край экрана при горизонтальной прокрутке.
* **Пространственная индексация префиксными суммами (`LineIndex`):**
  * $O(\log N)$ бинарный поиск от экранных координат к строкам и столбцам без единой аллокации памяти в режиме `Uniform` для сотен тысяч строк.
* **Интерактивный ресайз столбцов и автоподбор ширины:**
  * Изменение ширины перетаскиванием границы с динамической сменяемостью курсора (`ResizeHorizontal`) и направляющей линией;
  * Двойной клик по разделителю заголовков автоматически подбирает оптимальную ширину (Auto-fit) по содержимому;
  * Двусторонняя синхронизация ширины столбцов со словарем метаданных SPSS (`VariableMetadata.columns`).

### 🖱️ Прямоугольное выделение, клавиатурная навигация и Direct Typing (Feature 007)
* **Прямоугольное выделение диапазона ячеек:**
  * Плавное выделение рамкой мышью, а также с зажатой клавишей `Shift` (Shift-click, Shift-drag);
  * Векторная отрисовка: акцентная рамка 1.5 px по периметру, фоновая подсветка ячеек без перекрытия текста и визуальное выделение активной ячейки-курсора.
* **Сквозная клавиатурная эргономика:**
  * Навигация стрелками, клавишами `Tab` / `Shift+Tab`, `Enter` / `Shift+Enter`, `Home` / `End`, `Ctrl+Home` / `Ctrl+End`;
  * Расширение выделения с удержанием `Shift`;
  * **Direct Typing:** Мгновенный ввод данных в активную ячейку по нажатию буквенно-цифровых клавиш без предварительного двойного клика мышью или нажатия Enter;
  * Быстрая очистка ячеек по `Delete` или `Backspace`.
* **Фокусируемый инлайн-редактор ячеек:**
  * Текстовое поле ввода `TextEdit` с акцентной рамкой фокуса, автосохранением по Enter/Tab и отменой по Escape.

### 📋 Прямоугольный буфер обмена TSV/CSV и автозаполнение (Feature 007)
* **Прямоугольный буфер обмена с поддержкой внешних программ:**
  * Двусторонняя сериализация диапазонов в TSV/CSV (`Ctrl+C`) и потоковый парсер вставки (`Ctrl+V`);
  * Вставка многострочных матриц с авторасширением строк датасета.
* **Серийное автозаполнение (Fill Handle):**
  * Векторный маркер протяжки $6 \times 6$ px в правом нижнем углу выделения;
  * Протяжка вычисляет шаг линейной арифметической прогрессии для чисел (`[1, 2, 3] -> [4, 5, 6]`) и циклическое повторение для текстовых данных.

### 🔍 Колоночный автофильтр, выпадающее меню и быстрая сортировка (Feature 008)
* **Колоночный автофильтр в заголовках:**
  * Векторная кнопка в шапке столбца (шеврон в покое, залитая иконка воронки при активном фильтре);
  * Плавающее окно автофильтра с живым поиском, переключателями «Выбрать все» / «Очистить», списком уникальных значений с чекбоксами и категорией «(Пустые)»;
  * Оценка соответствия строк через `DataSet::is_row_matched_by_filter`.
* **Быстрая сортировка через CommandBus:**
  * Сортировка по возрастанию (A–Z) и убыванию (Z–A) из меню автофильтра и из контекстного меню;
  * Выполняется как каноническая команда движка `data.sort_cases` со 100% обратимым Undo/Redo (`Ctrl+Z` / `Ctrl+Y`) и фиксацией в синтаксисе SPSS (`SORT CASES BY ...`).

### 🎯 Контекстное меню ячеек, визуализация фильтрации и аудит целостности (Feature 008)
* **Контекстное меню ячеек (правый клик):**
  * «Фильтровать по значению ячейки», «Очистить фильтр столбца», «Сортировка по возрастанию / убыванию», «Копировать» / «Вставить».
* **Визуализация фильтрации (Паритет SPSS FILTER BY):**
  * Отфильтрованные строки не удаляются: номера строк перечеркиваются диагональной линией, текст в ячейках отображается приглушенным серым цветом, и строки автоматически исключаются из расчетов аналитических процедур.
* **Аудит целостности данных («Числа, сохраненные как текст»):**
  * Автоматический детектор `is_number_stored_as_text` выявляет числовые данные внутри строковых полей;
  * Зеленый треугольный маркер в верхнем левом углу ячейки в стиле Excel;
  * Одиночный клик по маркеру мгновенно конвертирует значение в `RawValue::Number`.

### 🗂️ Управление жизненным циклом вкладок (Feature 005)
* Базовые представления (**Представление данных**, **Представление переменных**, **Просмотр вывода**) защищены от случайного закрытия (`is_closeable = false`);
* Вспомогательные вкладки (Редактор синтаксиса) закрываются штатной кнопкой с крестиком;
* Меню «Окно» обеспечивает мгновенное открытие и переключение между всеми вкладками;
* Выполнение синтаксиса SPSS гарантирует автоматическое открытие вкладки Просмотра вывода.

### 📦 Релизные сборки под все платформы
* **Windows (x64):** Inno Setup Установщик (`.exe`) + Портативный ZIP-архив (тихий запуск без окна терминала).
* **Linux (x64):** Пакет Debian (`.deb`) + Портативный архив TAR.GZ.
* **macOS (Apple Silicon ARM64 & Intel x64):** Образ диска Apple DMG + Бандл приложения (`.app`) TAR.GZ.

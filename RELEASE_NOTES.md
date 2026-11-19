# StatCraft v0.4.1

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.1 is a feature and stabilization release introducing **Multi-File Workspaces**, **Data Merging (Add Cases & Add Variables)**, **Memory Guard protection**, and comprehensive **Grid Selection & Interaction Modernization** adhering to the strict IBM SPSS Statistics clean-room canon.

### 🗂️ Multi-File Workspace, Tabbed Datasets & Memory Guard (Feature 010 & 011)
* **Concurrent Document Sessions (`WorkspaceManager`):**
  * Open, manipulate, and switch between multiple datasets simultaneously in a unified window;
  * Each dataset session maintains isolated Undo/Redo histories, filter states, split-file contexts, and selection metadata.
* **Lossless In-Memory Compression:**
  * Zero-copy bitmasks for missing values (`NullBitmask`), dictionary encoding for categorical strings (`DictionaryEncodedColumn`), and packed numeric columns (`LosslessCompressedColumn`) retaining full `f64` mathematical precision.
* **SPSS Syntax Commands for Datasets:**
  * Full parser and engine command execution for `DATASET NAME`, `DATASET ACTIVATE`, `DATASET CLOSE`, and `DATASET DISPLAY` with `WINDOW=ASIS` validation.
* **Vector Document Tab Strip (`DocumentTabStrip`):**
  * Horizontal tab bar with custom vector dataset icons, unsaved modification markers (`*`), close buttons (`✕`), smooth wheel scrolling, chevron overflow navigators, and quick dataset creation (`[ ➕ ]`);
  * Full keyboard shortcuts: `Ctrl+Tab` / `Ctrl+Shift+Tab` (next/previous dataset), `Ctrl+W` (close current dataset).
* **Cross-Platform System Memory Guard (`system_memory.rs`):**
  * Native physical RAM probing without heavyweight dependencies: Windows API (`GlobalMemoryStatusEx`), Linux (`/proc/meminfo`), macOS (`sysctl`), and WebAssembly;
  * Safety preflight check: proactively alerts users when opening large files (>250 MB) or batch operations (>10 files) if available RAM is critically low (<500 MB or <15%).

### 🔗 Data Merging: Add Cases & Add Variables (Feature 012)
* **Add Cases (`ADD FILES`):**
  * Concatenate observations vertically from external files (`.sav`, `.zsav`, `.stc`, `.csv`, `.xlsx`) or active workspace tabs;
  * Automated schema alignment, automatic variable type reconciliation, `/IN=var` source indicator tracking, and optional interleaved case ordering via `/BY`.
* **Add Variables (`MATCH FILES`):**
  * Merge variables horizontally across datasets: 1:1 key merges, 1:N lookup tables (`/TABLE`), and sequential merges without keys;
  * Automated resolution of duplicate column collisions with clean rename suggestions and presence indicator flags (`/IN`).
* **Canonical DialogKit Interface:**
  * Clean-room dialog windows (`AddCasesDialog`, `AddVarsDialog`) with dual-pane variable selection, active workspace tab pickers, file browse dialogs, and instant SPSS syntax generation.

### 🛡️ Grid Interaction Modernization & Invariants (Spec-Kit 013)
* **Preserving Rectangular Selection on Right-Click:**
  * Right-clicking inside an active multi-cell selection preserves the highlighted rectangular block rather than collapsing to a single cell.
* **Zero Pollution Invariant for Phantom Rows:**
  * Committing empty or whitespace input on the trailing insertion row (`*`) leaves dataset dimensions intact, eliminating accidental rows with system-missing values (`SYSMIS` / `.`).
* **Physical Case Deletion via Delete Key & Menu Actions:**
  * Differentiated deletion semantics: cell range selections clear cell contents to `Null`; full-row selections (`is_full_rows`) physically delete cases via the engine command `data.delete_row` with reduced sample size $N$ and complete Undo/Redo reversibility.
* **Row and Column Header Context Menus (`ContextMenuTarget`):**
  * Row headers offer "Insert Case Above", "Delete Selected Cases ({count})", and "Clear Cases Values"; column headers offer Ascending/Descending sorts, "Insert Variable", and "Auto-fit Column Width".
* **Click-Outside Dismissal (Eliminating Modal Traps):**
  * Menus and column filter popups measure boundaries and dismiss automatically when clicking outside without requiring `Escape`.

### 🎯 SPSS Canon Range Selection & Fill Handle Removal (Spec-Kit 014 / BUG-014)
* **Pure Read-Only Range Selection (SPSS Canon):**
  * Eliminated the bottom-right fill handle click-interceptor and marker: dragging mouse across cells is guaranteed to perform strictly safe, read-only marquee selection (`SelectRange`);
  * Completely prevents accidental cell value replication and phantom row allocation when selecting from cell `(0, 0)` down across the grid.
* **Zero Mutation Invariant in Auto-Fill (`apply_auto_fill`):**
  * Auto-fill algorithm is strictly capped to existing dataset rows (`target.max_row.min(ds.row_count().saturating_sub(1))`), completely barring `apply_auto_fill` from allocating new empty rows in the dataset.

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive (silent GUI subsystem launch without terminal window).
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.1 — это функциональное обновление и стабилизация, внедряющие **мультифайловое рабочее пространство**, **слияние данных (добавление наблюдений и переменных)**, защиту оперативной памяти **Memory Guard**, а также комплексную **модернизацию интерактивного взаимодействия с сеткой данных** в строгом соответствии с эталонным каноном IBM SPSS Statistics и чистой комнаты.

### 🗂️ Мультифайловый воркспейс, вкладки датасетов и Memory Guard (Feature 010 & 011)
* **Параллельные сессии документов (`WorkspaceManager`):**
  * Одновременное открытие, редактирование и переключение между несколькими наборами данных в едином окне программы;
  * Каждая сессия изолированно хранит свою историю отмены/повтора (Undo/Redo), состояния фильтров, расщепления файлов и выделения ячеек.
* **Сжатие таблиц в памяти без потерь:**
  * Битовые маски пропусков (`NullBitmask`), словарное кодирование текстовых категорий (`DictionaryEncodedColumn`) и плотные числовые колонки (`LosslessCompressedColumn`) со 100% сохранением математической точности `f64`.
* **Команды управления датасетами в синтаксисе SPSS:**
  * Поддержка парсинга и выполнения команд `DATASET NAME`, `DATASET ACTIVATE`, `DATASET CLOSE`, `DATASET DISPLAY` с валидацией модификаторов `WINDOW=ASIS`.
* **Векторная панель вкладок документов (`DocumentTabStrip`):**
  * Вкладки с векторным значком таблицы, индикатором несохраненных изменений `*`, кнопкой закрытия `✕`, плавной прокруткой колесиком, кнопками навигации шевронами и кнопкой быстрого создания нового датасета `[ ➕ ]`;
  * Горячие клавиши: `Ctrl+Tab` / `Ctrl+Shift+Tab` (переключение между датасетами), `Ctrl+W` (закрыть текущий датасет).
* **Кроссплатформенный Memory Guard (`system_memory.rs`):**
  * Нативный опрос доступной физической памяти ОС: Windows API (`GlobalMemoryStatusEx`), Linux (`/proc/meminfo`), macOS (`sysctl`), WebAssembly;
  * Предохранитель: предупреждает пользователя перед открытием тяжелых файлов (>250 МБ) или пакетов (>10 файлов) при критически низком остатке оперативной памяти (<500 МБ или <15%).

### 🔗 Слияние файлов: Добавить наблюдения и Добавить переменные (Feature 012)
* **Добавить наблюдения (`ADD FILES`):**
  * Вертикальное объединение строк из внешних файлов (`.sav`, `.zsav`, `.stc`, `.csv`, `.xlsx`) или соседних открытых вкладок;
  * Автоматическое сопоставление одноименных полей, приведение типов, отслеживание источника `/IN=var` и опциональная сортировка с чередованием по ключам `/BY`.
* **Добавить переменные (`MATCH FILES`):**
  * Горизонтальное объединение колонок: объединение 1:1 по ключам, справочные таблицы 1:N (`/TABLE`), последовательное слияние без ключей;
  * Автоматическое разрешение коллизий дублирующихся имен переменных и создание индикаторов присутствия (`/IN`).
* **Диалоговые окна стандарта DialogKit:**
  * Окна `AddCasesDialog` и `AddVarsDialog` с удобным двухколоночным выбором переменных, селектором открытых вкладок воркспейса, кнопкой обзора файлов на диске и генерацией канонического синтаксиса SPSS.

### 🛡️ Модернизация сетки данных и устранение багов взаимодействия (Spec-Kit 013)
* **Сохранение прямоугольного выделения при правом клике:**
  * Клик правой кнопкой мыши внутри выделенного блока ячеек больше не схлопывает диапазон в одну ячейку, сохраняя блок активным для команд контекстного меню.
* **Защита от загрязнения датасета фантомными строками (Zero Pollution):**
  * Фиксация пустого значения в строке добавления со звездочкой `*` не аллоцирует строк в датасете: появление случайных строк с системными пропусками `SYSMIS` (`.`) исключено.
* **Физическое удаление наблюдений клавишей Delete и через меню:**
  * Разделено поведение: удаление в диапазоне ячеек очищает значения; удаление при выделении строк целиком (`is_full_rows`) физически удаляет наблюдения через команду ядра `data.delete_row` с уменьшением объема выборки $N$ и поддержкой Undo.
* **Контекстные меню заголовков строк и колонок (`ContextMenuTarget`):**
  * Заголовки строк: «Вставить наблюдение выше», «Удалить выбранные наблюдения», «Очистить значения»; заголовки столбцов: сортировка A–Z / Z–A, «Вставить переменную», «Автоподбор ширины столбца».
* **Закрытие меню по клику снаружи (Click-Outside Dismissal):**
  * Контекстные меню и окна автофильтра закрываются при любом клике за пределами их границ без обязательного нажатия `Escape`.

### 🎯 Канон SPSS: выделение ячеек строго Read-Only и удаление Fill Handle (Spec-Kit 014 / BUG-014)
* **Чистое выделение без риска затирания данных:**
  * Маркер автозаполнения и его хит-тест удалены из интерфейса сетки: перетаскивание мыши всегда является безопасной операцией выделения ячеек (`SelectRange`);
  * Полностью устранена проблема случайного тиражирования значения ячейки `(0, 0)` и добавления фантомных строк при выделении таблицы мышью.
* **Инвариант сохранности строк в алгоритме автозаполнения:**
  * Вспомогательная функция `apply_auto_fill` строго ограничена существующими строками датасета без вызова `add_empty_row()`.

### 📦 Мультиплатформенные установочные пакеты
* **Windows (x64):** Инсталлятор Inno Setup (`.exe`) + Портативный ZIP-архив (тихий запуск без консольного окна).
* **Linux (x64):** Пакет Debian (`.deb`) + Портативный TAR.GZ-архив.
* **macOS (Apple Silicon ARM64 & Intel x64):** Образ диска DMG + Пакет приложения (`.app`) TAR.GZ.

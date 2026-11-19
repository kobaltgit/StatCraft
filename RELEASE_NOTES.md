# StatCraft v0.4.2

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.2 is a feature and stabilization release introducing **Dynamic Dataset Colors & Container Persistence** and **Tab Dirty Marker Positioning & Titlebar Normalization**, alongside the multi-dataset capabilities and grid interactions introduced in v0.4.1.

### 🎨 Dynamic Dataset Colors & Container Persistence (Spec-Kit 015 / BUG-015)
* **Collision-Free Random Contrast Colors:**
  * Implemented high-performance 64-bit SplitMix64 generator with continuous golden angle distribution ($137.50777^\circ$).
  * Dynamically scans currently opened document tabs and guarantees a minimum angular distance $\ge 45^\circ$, completely preventing visually identical purple/violet clustering across datasets with similar names (`merge_base.csv`, `merge_add_cases.csv`, `merge_add_vars.csv`).
  * Automatically tunes saturation ($0.65 \dots 0.85$) and optimizes lightness for both dark and light theme palettes.
* **Cross-Pane Visual Identity:**
  * Synchronized colors across the application: DocumentTabStrip active underlines, vector table grid icons, Outline Tree procedure chips, and Notes summary tables query session colors via `workspace.session_color_by_name`.
* **Complete Persistence in `.stc` Containers:**
  * Added `pub tab_color: Option<String>` with `skip_serializing_if = "Option::is_none"` across `StcDatasetRecord`, `StcContainer`, and `DocumentSession`.
  * Saving multi-file workspaces preserves assigned tab colors into `.stc` project files.
  * 100% backward compatibility: legacy v0.4.0 `.stc` project files deserialize seamlessly with automated contrast color assignment.

### 🏷️ Tab Dirty Marker Positioning & Titlebar Normalization (BUG-016)
* **Zero Text Overlap on Document Tabs:**
  * Removed hardcoded coordinate truncation (`min(12) * 6.5`) that previously caused asterisk markers to render directly over filename characters (e.g. `cases` in `merge_add_cases.csv`).
  * Right padding is safely reserved (`32.0 px` when dirty, `20.0 px` when clean).
  * Long filenames cleanly truncate with an ellipsis (`truncate_at_width`), and the amber asterisk marker (`*`) is placed strictly after the title text (`text_x + text_w + 3.0`) with vertical centering (`Align2::LEFT_CENTER`).
* **Titlebar Normalization & Unicode Compliance:**
  * Completely eliminated the hardcoded bullet symbol ` •` (U+2022) from the window titlebar in strict compliance with Vector-First UI standards.
  * Window title now renders cleanly: `file_name` for saved files, or `file_name *` when modified.
* **Semantic Session Isolation in `has_unsaved_changes()`:**
  * Refactored `has_unsaved_changes()` to compare active dataset state against its own session dictionary, dimensions, and delta undo checkpoint, eliminating false dirty triggers when switching between tabs with differing row/column counts.
  * Saving datasets to CSV/TSV/TXT via `save_to_path` properly clears the dirty state via `sess.mark_clean()`.

### 🗂️ Included from v0.4.1: Multi-File Workspaces & Grid Modernization
* **Concurrent Document Sessions (`WorkspaceManager`):** Open, edit, and switch between multiple datasets in a single window with isolated Undo/Redo histories, filter states, and split-file contexts.
* **Cross-Platform Memory Guard:** Real-time RAM monitoring preventing out-of-memory crashes on massive files.
* **Data Merging (`ADD FILES` & `MATCH FILES`):** Vertical case concatenation and horizontal variable merging with key-matching and `/IN` indicator flags.
* **SPSS Canon Range Selection:** 100% safe, read-only grid selection without accidental cell replication or phantom row allocation.

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive (silent GUI launch).
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.2 — это функциональное обновление и стабилизация, внедряющие **динамическую генерацию контрастных цветов датасетов и их сохранение в контейнере проекта `.stc`**, а также **исправление позиционирования маркера несохраненности на вкладках и нормализацию заголовка окна программы**.

### 🎨 Динамические цвета датасетов и сохранение в .stc (Spec-Kit 015 / BUG-015)
* **Контрастные случайные цвета без коллизий:**
  * Внедрен 64-битный хэшер на базе SplitMix64 с непрерывным распределением по золотому сечению ($137.50777^\circ$).
  * Алгоритм сканирует все открытые вкладки и гарантирует минимальное угловое расстояние $\ge 45^\circ$, полностью устраняя визуальное слияние в одинаковый фиолетовый оттенок для файлов со схожими именами (`merge_base.csv`, `merge_add_cases.csv`, `merge_add_vars.csv`).
  * Автоматически варьирует насыщенность ($0.65 \dots 0.85$) и адаптирует светлоту под темную и светлую темы оформления.
* **Сквозная цветовая идентификация:**
  * Синхронизация цветов по всему интерфейсу: активное подчеркивание вкладки DocumentTabStrip, векторный значок таблицы, чипы процедур в дереве навигации (Outline Tree) и таблица «Примечания» (Notes) запрашивают цвет сессии через `workspace.session_color_by_name`.
* **Сохранение в файлах проектов `.stc`:**
  * Добавлено поле `pub tab_color: Option<String>` с атрибутом `skip_serializing_if = "Option::is_none"` в структуры `StcDatasetRecord`, `StcContainer` и `DocumentSession`.
  * При сохранении проекта назначенные цвета вкладок фиксируются в файле `.stc` и восстанавливаются при повторном открытии.
  * 100% обратная совместимость со старыми файлами проектов формата v0.4.0.

### 🏷️ Позиционирование маркера несохраненности и очистка шапки окна (BUG-016)
* **Исключение наложения маркера на текст названия вкладки:**
  * Удален хардкод отсечения `min(12) * 6.5`, приводивший к наложению звездочки на буквы слова `cases` посередине вкладки при длине имени более 12 символов.
  * Справа гарантированно резервируется отступ (`32.0 px` при несохраненных изменениях, `20.0 px` при чистом состоянии).
  * Длинные имена файлов аккуратно усекаются многоточием (`truncate_at_width`), а янтарная звездочка `*` позиционируется строго справа за текстом (`text_x + text_w + 3.0`) с вертикальным центрированием `Align2::LEFT_CENTER`.
* **Нормализация заголовка окна программы:**
  * Полностью удален захардкоженный Unicode-глиф точки ` •` (U+2022) в соответствии со стандартом Vector-First UI (п. 8 AGENTS.md).
  * Заголовок окна отображается строго по стандарту: `file_name` для сохраненного файла или `file_name *` при наличии изменений.
* **Семантическая изоляция сессий в `has_unsaved_changes()`:**
  * Логика проверки несохраненных изменений изолирована для каждой сессии: сравниваются размеры, словарь и чекпоинт дельты активного датасета относительно его собственной сессии, устраняя ложные срабатывания при переключении между вкладками с разным числом строк/колонок.
  * Сохранение набора данных в CSV/TSV/TXT в `save_to_path` корректно сбрасывает dirty-флаг сессии через `sess.mark_clean()`.

### 🗂️ Включено из версии v0.4.1: Мультифайловый воркспейс и слияние данных
* **Параллельные сессии документов (`WorkspaceManager`):** одновременная работа с несколькими датасетами в едином окне с изолированными историями Undo/Redo, фильтрами и расщеплением файлов.
* **Кроссплатформенный Memory Guard:** мониторинг доступной оперативной памяти, предотвращающий сбои при открытии гигантских массивов данных.
* **Слияние данных (`ADD FILES` и `MATCH FILES`):** вертикальное объединение строк и горизонтальное объединение переменных по ключам с созданием индикаторов источника `/IN`.
* **Эталонное выделение диапазона ячеек (канон SPSS):** строго безопасное выделение без риска затирания данных или создания пустых фантомных строк.

### 📦 Дистрибутивы релиза
* **Windows (x64):** Установщик Inno Setup (`.exe`) + Портативный ZIP-архив (тихий запуск без консольного окна).
* **Linux (x64):** Пакет Debian (`.deb`) + Портативный TAR.GZ-архив.
* **macOS (Apple Silicon ARM64 & Intel x64):** Образ диска DMG + Приложение (`.app`) в архиве TAR.GZ.

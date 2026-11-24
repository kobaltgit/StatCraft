# StatCraft v0.4.6

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.6 brings 100% canon parity with IBM SPSS Statistics for pivot table conditional formatting and table styling (**Feature 019 / OUTPUT MODIFY**). It introduces the complete `TableStyleDialog` suite, a high-performance declarative styling engine, a 30-color SPSS matrix palette, 34 data format masks, and robust multi-attribute typography rendering in the Output Viewer.

### 🎨 Table Style & Conditional Formatting Suite (Feature 019)
* **Canonical Modal DialogKit Suite (`table_style_dialog/`):**
  * **Main Dialog:** 5 in-cell ComboBox columns (*Table, Value, Dimension, Condition, Format*), interactive condition and format summaries, rule cloning (`[ Duplicate ]`), rule reordering (`[ Move Up ]`, `[ Move Down ]`), and transactional rollback on cancellation.
  * **Condition Subdialog:** Value selection, 12 predicate operators, modulus comparison (`[ ] Absolute value`), lower/upper range limits, and Top/Bottom N inputs.
  * **Format Subdialog:**
    * Centered live sample preview box displaying `123` reflecting styles, fonts, and colors in real-time.
    * Typography toolbar with active selection toggles: `[ B ]`, `[ I ]`, `[ U ]`.
    * Authentic 5x6 SPSS Color Matrix Palette (`color_palette.rs`) featuring White, Black, No Color (diagonal red slash), 30 canonical SPSS color shades, and custom RGB color picker.
    * 34 Cell Format Masks (`format_masks.rs`): comprehensive formatting for numbers, scientific notation, percentages, quarters, weeks, timestamps, and localized month/day names.
    * Scope selection (*Only cells, Entire column, Entire row*), decimal places spinner, and cell value replacement input.
* **SPSS Syntax & CommandBus Integration:**
  * Canonical SPSS `OUTPUT MODIFY` syntax generation on *Paste*:
    ```sps
    OUTPUT MODIFY
      /SELECT TABLES
      /IF SUBTYPES=['Frequencies']
      /TABLECELLS SELECT=[DIMENSION=COLUMNS] CONDITION=GE(100.0)
       STYLE=[BACKGROUND=RGB(204, 255, 204) FONTSTYLE=BOLD].
    ```
  * Reversible execution with transactional delta logging.

### 📋 Full Changelog for v0.4.6
```markdown
## [0.4.6] - 2026-11-24

### Added
- **Table Style & Conditional Formatting Parity (Feature 019 / IBM SPSS OUTPUT MODIFY):**
  - Typed conditional formatting engine in `statcraft_reporting::styling`.
  - 12 value predicate conditions: All, Equal, NotEqual, LessThan, LessOrEqual, GreaterThan, GreaterOrEqual, Between, NotBetween, TopN, BottomN, SysMis.
  - Dimension and value targeting across table subtypes, columns, rows, and label tokens.
  - 3-way application scope: Only Cells, Entire Column, Entire Row.
  - String value replacement for matching cells.
- **Authentic IBM SPSS Table Style Dialog Suite (DialogKit):**
  - Main Dialog TableStyleDialog with 5 in-cell ComboBox columns and rule ordering.
  - Condition Subdialog with [ ] Absolute Value modulus comparisons.
  - Format Subdialog with live Preview box, B/I/U toolbar, 30-color SPSS matrix palette, 34 format masks, and custom RGB picker.
- **Output Viewer Styled Table Rendering & Reporting:**
  - Vector rendering in Output Viewer with cell background fills and faux bold typography synthesis (`BUG-019`).
  - Auxiliary "Conditional Styles Report" audit table.
  - Canonical SPSS OUTPUT MODIFY syntax generation.
  - HTML report export with inline CSS attributes.
- **Zero-Hardcode Fluent Localization:**
  - 44+ semantic translation keys in locales/ru.ftl and locales/en.ftl.

### Fixed
- **Architectural Decomposition of App Dialog Coordinator (BUG-018):**
  - Decomposed app_dialogs.rs into app_dialogs_stats.rs (284 lines) to maintain < 1000 lines limit.
- **Typography Rendering in Output Viewer & Format Preview (BUG-019):**
  - Resolved egui italic and bold conflict via LayoutJob overstrike synthesis.
```

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.6 обеспечивает 100% паритет с IBM SPSS Statistics в области условного форматирования и стилизации сводных таблиц (**Feature 019 / OUTPUT MODIFY**). Добавлен полный комплекс диалогов `TableStyleDialog`, декларативный движок стилей, каноническая палитра SPSS из 30 оттенков, 34 маски форматов ячеек и надежный рендеринг типографики в Output Viewer.

### 🎨 Комплекс диалогов «Стиль таблицы» (Feature 019)
* **Аутентичный комплекс диалогов DialogKit (`table_style_dialog/`):**
  * **Главное окно:** 5 колонок с выпадающими списками прямо в ячейках (*Таблица, Значение, измерение, Условие, Формат*), сводки правил, дублирование (`[ Дублировать ]`), изменение порядка (`[ Вверх ]`, `[ Вниз ]`) и транзакционный откат по кнопке Отмена.
  * **Поддиалог «Условие»:** выбор значений, 12 операторов сравнения, сравнение по модулю (`[ ] Абсолютное значение`), границы диапазона и ввод Top/Bottom N.
  * **Поддиалог «Формат»:**
    * Центрированный блок предпросмотра с тестовым числом `123`, на лету отражающим выбранный стиль.
    * Тулбар шрифтов с подсветкой активного состояния: `[ B ]`, `[ I ]`, `[ U ]`.
    * Каноническая палитра цветов SPSS 5x6 (`color_palette.rs`): Белый, Черный, Без цвета (красная диагональная черта), 30 оттенков SPSS и кастомный RGB-пикер.
    * 34 маски форматов данных (`format_masks.rs`): числа, экспоненты, проценты, даты, время, дни недели и месяцы.
    * Область применения (*Только ячейки, Весь столбец, Вся строка*), счетчик десятичных знаков и поле замены значения.
* **Генерация синтаксиса SPSS и командная шина:**
  * Генерация канонического синтаксиса `OUTPUT MODIFY` при нажатии *Вставить*:
    ```sps
    OUTPUT MODIFY
      /SELECT TABLES
      /IF SUBTYPES=['Frequencies']
      /TABLECELLS SELECT=[DIMENSION=COLUMNS] CONDITION=GE(100.0)
       STYLE=[BACKGROUND=RGB(204, 255, 204) FONTSTYLE=BOLD].
    ```
  * Обратимое выполнение с сохранением матрицы Undo/Redo.

---

### 📦 Дистрибутивы релиза (Artifacts)
* **Windows (x64):** Автономный инсталлятор (`statcraft-windows-x64-setup.exe`) + Портативный архив (`statcraft-windows-x64.zip`).
* **Linux (x64):** Пакет Debian (`statcraft_0.4.6_amd64.deb`) + Портативный TAR.GZ (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon & Intel):** Подписанный DMG-образ (`statcraft-macos.dmg`).

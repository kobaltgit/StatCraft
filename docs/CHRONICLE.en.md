# StatCraft — Development Chronicle

### Project Kickoff: Architectural Canon and Groundwork
*Time: October 27, 2026, 16:40 (~45 minutes)*

Following an in-depth design interview in `/grill-me` mode and full alignment with the ArtCraft (`storytold`) ecosystem standards, the **StatCraft** project is officially launched — an open-source, high-performance statistical suite written in 100% pure Rust.

The master development regulation `AGENTS.md` is codified:
* Strict compilation limit of `jobs = 1` protecting workstation resources;
* **Never Crash** principle: complete prohibition of `unwrap()`, `expect()`, `panic!()`, and `unsafe`;
* Commands rather than handlers architecture (`CommandBus`, AI-agent schemas);
* Strict L0–L6 layering verified via `cargo xtask layers`;
* Zero string hardcoding in the UI (`locales/*.ftl`, instant runtime RU/EN switching);
* 100% mandatory test coverage per function with precision statistical oracles.

---

### Phase 1: Workspace Scaffolding, L0/L1/L5 Core, and egui_dock UI
*Time: October 27, 2026, 17:05 (~25 minutes)*

Successfully implemented and verified the initial architectural milestone:
1. **L0 Codecs (`statcraft_io`):** Streaming CSV/TSV parser with automatic delimiter detection (comma/semicolon/tab) and type inference (Numeric / Text / Date) with zero panics.
2. **L1 Data Engine (`statcraft_core`):** `DataSet` table structure and `VariableDictionary` (scales `Scale`, `Ordinal`, `Nominal`, `ValueLabels` maps, `MissingValues`). Transactional `DeltaBuffer` with cell-level Undo/Redo.
3. **Zero-Hardcode Localization (`statcraft_i18n`):** Thread-safe Fluent-based catalog engine (`en.ftl` and `ru.ftl`) supporting runtime language switching. `i18n_lint_test` enforces 0% hardcoded literals in UI code.
4. **L5 Command Bus (`statcraft_syntax`):** Strongly-typed `EngineCommand` bus with JSON schemas for AI agents (`data.import_csv`, `var.set_label`, `var.set_measure`, `data.cell_edit`) and Undo/Redo history.
5. **L6 Presentation Layer (`statcraft_app`):** Docking workspace on `egui_dock` featuring **Data View** (virtualized grid for 1M+ rows, value labels toggle), **Variable View** (metadata property grid), and **Output Viewer** (outline tree).
6. **Branding & Vector Icon:** `assets/icon.svg` cleaned of Illustrator export artifacts and rendered natively via the `paint_app_icon` vector widget.
7. **Testing:** 38 out of 38 unit and integration tests passed cleanly (`cargo test --workspace -j 1`). `cargo clippy` and `cargo fmt` passed with zero warnings.

---

### Phase 1.1: Comprehensive End-to-End Icon Integration
*Time: October 27, 2026, 17:30 (~20 minutes)*

Completed end-to-end integration of the official vector brand icon `assets/icon.svg`:
1. **Geometry & Centering:** Removed Adobe Illustrator markup bloat, centered the square glyph viewBox (`26 116 560 560`) with zero letterhead margins.
2. **Multi-resolution Asset Generator (`xtask icon`):** Implemented an automated pipeline based on `resvg` and `ico` generating multi-res PNGs (16, 32, 48, 64, 128, 256 px) and `assets/icon.ico`.
3. **PE `.exe` Resource Embedding:** Configured `winres` in `build.rs` to embed `icon.ico` directly into the `.rsrc` section of `statcraft_app.exe`. Windows Explorer and the Taskbar now render the official icon.
4. **Window & Taskbar Integration:** Supplied the rasterized icon via `eframe::icon_data::from_png_bytes` into `ViewportBuilder::with_icon` for native titlebar and Taskbar presence.
5. **In-App Menu Bar Texture:** Replaced distorted line approximations with a pixel-perfect, crisp texture loaded into egui's context (`egui::TextureHandle`) displayed next to the StatCraft brand name.
6. **Verification:** All 37 tests green, Clippy and Fmt passed cleanly, and resource extraction verified via `System.Drawing.Icon.ExtractAssociatedIcon`.

---

### Phase 1.2: Menu Layout Optimization and Dynamic Localization Fix
*Time: October 27, 2026, 17:48 (~15 minutes)*

Refined top layout ergonomics and solved stale string caching in the localization system:
1. **Menu Bar De-duplication:** Removed duplicate icon and app title from `egui::menu::bar`. The menu items (`File`, `Edit`, `View`, `Data`, `Analyze`) now anchor directly to the top-left client area right below the native Windows titlebar, adhering to desktop UX canon (SPSS, Photoshop).
2. **Elimination of Stale Localized Strings:** Resolved the apparent "hardcoded strings" issue caused by early string materialization in application state (`String` inside `OutputEntry` and `status_message`). When switching languages, old messages lingered in the prior language. Refactored state into semantic types `StatusMessage` and `OutputPayload`, resolving translations dynamically against the active Fluent catalog on each frame.
3. **Refined Russian Catalog:** Cleaned up English parenthetical suffixes in `locales/ru.ftl` (`Представление данных`, `Представление переменных`, `Просмотр вывода`).
4. **Verification:** All 38 workspace tests passing with `-j 1`, Clippy and Fmt clean. Release binary built.

---

### Phase 1.3: Table Grid Alignment Fix and Pinned Column Headers
*Time: October 27, 2026, 18:00 (~15 minutes)*

Resolved table grid column-header misalignment and scrolling defects in both Data View and Variable View:
1. **Root Cause Analysis:** Using `ui.allocate_ui_with_layout` caused cells to shrink-wrap to the bounding box of their text contents (a cell displaying `"1"` took 10 px, a large float took 50 px, headers took 80 px, and `text_edit_singleline` expanded to 180+ px). Consequently, column horizontal positions drifted independently on every row. Furthermore, in Data View the headers were placed inside the virtualized `show_rows` callback, causing them to drift down when scrolling vertically.
2. **Deterministic Geometric Lock:** Every cell in Data View and Variable View now allocates exact width via `ui.allocate_exact_size` with zero item spacing (`item_spacing = (0, 0)`). Backgrounds, crisp borders, and clipped text rendering (`with_clip_rect`) guarantee mathematical sub-pixel alignment between header columns and all row cells across the entire dataset.
3. **Two-Axis Scrolling with Sticky Headers:** Implemented classic spreadsheet ergonomics (SPSS / Excel): an outer `ScrollArea::horizontal()` scrolls both the header row and table body synchronously along the X axis, while an inner `ScrollArea::vertical()` handles vertical virtualization. Column headers stay pinned at the top of the viewport and never scroll out of view.
4. **Constrained Interactive Widgets:** In Variable View, the `TextEdit` single-line widget and the `ComboBox` measure selector are strictly bounded within their cell rects via `allocate_new_ui` with `UiBuilder`, preventing adjacent columns from being displaced.
5. **Verification:** All 38 workspace tests passing with `-j 1`, `cargo clippy --workspace -j 1 -- -D warnings` 100% clean, `xtask layers` verified, and `cargo fmt` applied.

---

### Phase 2: L2 Mathematical Engine statcraft_algo, Descriptives Dialog & Pivot Tables
*Time: October 27, 2026, 18:20 (~20 minutes)*

Established the second computational layer of the StatCraft architecture — the statistical procedure engine:
1. **L2 Mathematical Algorithms (`statcraft_algo`):**
   - Implemented `compute_descriptives`: valid/missing observations $N$, mean $\bar{x}$, standard error of mean $SE_{mean}$, unbiased sample variance $s^2$, sample standard deviation $s$, minimum, maximum, range, sum, unbiased Fisher-Pearson sample skewness $G_1$ (with $SE_{skew}$) and Fisher excess kurtosis $G_2$ (with $SE_{kurt}$).
   - Implemented categorical frequency distributions `compute_frequencies` (frequencies, total percent, valid percent, cumulative percent).
   - **Golden Oracle Testing:** Rigorous mathematical unit tests verified against Python `scipy.stats` reference values with precision $\le 10^{-6}$.
   - **Never Crash Principle:** Comprehensive edge-case handling for empty vectors, single-item samples, NaN sequences, and missing codes without panics.
2. **L5 Command Bus (`statcraft_syntax`):**
   - Implemented typed `DescriptivesCommand` (`stat.descriptives`) with JSON parameter specifications for AI agents.
   - Built SPSS syntax emitter generating `DESCRIPTIVES VARIABLES=... /STATISTICS=MEAN STDDEV MIN MAX.`.
3. **L6 Variable Picker Dialog and Pivot Table (`statcraft_app`):**
   - Implemented `Analyze > Descriptive Statistics > Descriptives...` modal conforming to SPSS desktop UX: source and target variable lists with vector measure badges, vector chevron transfer buttons `[ > ]` and `[ < ]`, statistics selection checkboxes, action buttons (`OK`, `Paste`, `Reset`, `Cancel`).
   - Integrated formatted SPSS-style Pivot Table inside Output Viewer with numerical formatting and authentic grid themes.
4. **DoD Compliance:**
   - Architecture verified with `cargo xtask layers`.
   - All 47 workspace tests passing (`cargo test --workspace -j 1`).
   - Localization linter `i18n_lint_test` verifies zero hardcoded strings across all new dialogs and menus.
   - `cargo clippy --workspace -j 1 -- -D warnings` and `cargo fmt` 100% clean.

---

### Phase 2.1: Dialog Layout and Pivot Table Row Alignment Fix
*Time: October 27, 2026, 18:35 (~15 minutes)*

Fixed two critical UI layout defects discovered during visual testing:
1. **Compact Centered Variable Picker Dialog:**
   - Resolved window width runaway across the screen by constraining `Window` size ($500 \times 420$ px) and anchoring to viewport center (`Align2::CENTER_CENTER`).
   - Fixed variable source/target list dimensions ($196 \times 150$ px) so transfer buttons `[ > ]` and `[ < ]` stay centered between panels.
   - Resolved egui `ScrollArea ID 4DF5` collisions using distinct `id_salt` tokens.
   - Added instant double-click item transfer between variable lists.
2. **Vertical Row Flow in Output Viewer:**
   - Root cause of the endless horizontal line: `ScrollArea` inside the detail panel inherited `Direction::LeftToRight` from parent `ui.horizontal`. Table headers and all observation rows (`age`, `gender`, `education`) were rendered side-by-side on a single endless row.
   - Converted detail panel to strict vertical flow (`ui.vertical`). Each variable is now placed on its own row, stacked vertically under the table header.
   - Added standard SPSS `Valid N (listwise)` footer summary row.
3. **Verification:** All 47 workspace tests green (`cargo test --workspace -j 1`), Clippy and Fmt clean, release executable built.

---

### Phase 2.2: Full Dock Height in Output Viewer & Elimination of Slit Scrolling
*Time: October 27, 2026, 18:45 (~15 minutes)*

Resolved layout height collapse causing the output viewer reports area to shrink to ~80 px:
1. **Defect Root Cause:** `ui.horizontal` was previously used to divide the left outline tree from the right pivot table viewing panel. In `egui`, horizontal rows shrink-wrap their cross-axis bounding box to the smallest child item (the collapsed single-item outline tree header, $\sim 60-80$ px). The inner `ScrollArea` of the report viewer was clamped to this slit, forcing the user to scroll vertically to see each table row.
2. **Explicit Tab Rectangle Splitting:** Replaced `ui.horizontal` with explicit geometric bounding box subdivision of the dock tab (`full_rect = ui.available_rect_before_wrap()`):
   - Left outline navigation panel receives `left_rect` with fixed 180 px width and 100% tab height (`full_rect.height()`);
   - Clean vertical separator stroke between panels;
   - Right detail panel receives `right_rect` claiming all remaining width and 100% tab height via `ui.allocate_new_ui(UiBuilder::new().max_rect(right_rect))`.
3. **Outcome:** Pivot tables and logs now render freely down the entire screen height as a continuous readable statistical document.
4. **Verification:** All 48 workspace tests green (`cargo test --workspace -j 1`), Clippy and Fmt clean, layers verified with `cargo xtask layers`, release binary compiled.

---

### Phase 3: Frequency Analysis, Variable Metadata Dictionary (Value Labels / Missing Values), and SPSS Tables
*Time: October 27, 2026, 19:30 (~25 minutes)*

Implemented the third core milestone of the StatCraft roadmap — the Frequencies procedure and the complete interactive variable metadata lifecycle:
1. **L2 Mathematical Engine (`statcraft_algo`):**
   - Extended the `compute_frequencies_detailed` procedure with comprehensive metadata integration (value labels, discrete missing values, range missing values).
   - Implemented percentiles and quartiles ($Q_1 = 25\%$, $Q_2 = 50\%$ Median, $Q_3 = 75\%$) conforming to the standard SPSS HAVERAGE definition (Weibull / Type 6: $r = p(n+1)$).
   - Deterministic numeric/lexical category sorting and calculation of frequency counts, total %, valid %, and cumulative %.
2. **L5 Command Bus (`statcraft_syntax`):**
   - Created typed `FrequenciesCommand` (`stat.frequencies`) with JSON parameter specs for AI agents and SPSS syntax generator `FREQUENCIES VARIABLES=... /STATISTICS=STDDEV MEAN MEDIAN /ORDER=ANALYSIS.`.
   - Created metadata mutation commands `VarSetValueLabelsCommand` (`var.set_value_labels`) and `VarSetMissingValuesCommand` (`var.set_missing_values`) with full Undo/Redo.
   - Registered all commands in `CommandBus::with_defaults()`.
3. **L6 Interactive Dialogs & Presentation (`statcraft_app`):**
   - **Frequencies Dialog (`FrequenciesDialog`):** Centered modal ($500 \times 420$ px) with source/target variable lists, vector transfer arrows `[ > ]` and `[ < ]`, instant double-click item transfer, "Display frequency tables" toggle, and action buttons (`OK`, `Paste`, `Reset`, `Cancel`).
   - **Value Labels Dialog (`ValueLabelsDialog`):** Modal for adding, updating, and removing `Code -> Label` mappings with scrollable list and selection.
   - **Missing Values Dialog (`MissingValuesDialog`):** Modal supporting "No missing values", "Discrete missing values" (up to 3 values), and "Range plus optional discrete value".
   - **Variable View Integration:** Added authentic SPSS-style `[...]` button inside the "Values" and "Missing" cells, opening the metadata dialogs.
   - **Data View Integration:** Connected `1/A` toggle button on the toolbar for instant switching between raw codes and value labels.
   - **Output Viewer:** Rendering genuine SPSS frequency tables displaying `Value/Label`, `Frequency`, `Percent`, `Valid Percent`, `Cumulative Percent`, summary rows (`Total Valid`, `System Missing`, `Total`), and percentiles block.
4. **Verification & Definition of Done:**
   - Strict Resource Guard (`-j 1` across all tasks).
   - 100% of workspace tests passing (`cargo test --workspace -j 1`).
   - Localization linter `i18n_lint_test` verifies zero hardcoded literals and 100% key parity between `en.ftl` and `ru.ftl`.
   - Architectural layer linter `cargo run --package xtask -j 1 -- layers` passed cleanly.
   - `cargo clippy --workspace -j 1 -- -D warnings` and `cargo fmt --check` clean.

---

### Phase 4: Full Two-Way Reactive Data and Metadata Editor (Variable View & Data View)
*Time: October 27, 2026, 21:05 (~30 minutes)*

Implemented the fourth foundational milestone of StatCraft — a complete two-way reactive data and metadata editor, turning the application from a passive viewer into an interactive statistical spreadsheet:

1. **"Blank State" Out of the Box:**
   - Launching StatCraft without opening a file now initializes an interactive blank dataset (`DataSet::blank(40, 0)`).
   - The user is immediately greeted by an editable spreadsheet grid (40 cases) and can start entering data or defining variables right away without placeholder blocks.

2. **Full Canon of 9 SPSS Variable Types & `VariableTypeDialog`:**
   - On layer L1 `statcraft_core`, the `DataType` enum was expanded to all 9 classical SPSS types: `Numeric`, `Comma`, `Dot`, `Scientific`, `Date`, `Dollar`, `CustomCurrency`, `String`, `RestrictedNumeric`.
   - Core formatting `get_cell_display` now formats numbers according to type parameters (comma/dot thousand groupings, decimal digits, currency symbols, and zero-padded integer widths).
   - Core parser `set_cell_parsed` cleanses currency and formatting symbols and normalizes user input into valid numerical or text values.
   - On L6 `statcraft_app`, built a centered modal dialog `VariableTypeDialog` ($480 \times 360$ px) featuring 9 radio buttons, dynamic width/decimals inputs, date format selectors (`dd-mmm-yyyy`, `yyyy-mm-dd` etc.), and custom currency selectors (`CCA`..`CCE`).

3. **Interactive `Variable View`:**
   - **Name:** In-place borderless `TextEdit` (`.frame(false)`) with identifier validation and uniqueness checks.
   - **Type:** Shows localized type name plus an authentic `[...]` button opening `VariableTypeDialog`.
   - **Width & Decimals:** Compact vector spinners with `▲ / ▼` buttons (step 1); decimals are automatically disabled for string variables.
   - **Label:** In-place singleline text edit for the full human-readable variable label.
   - **Align & Role:** Click to cycle through values (`Left` $\to$ `Right` $\to$ `Center`; `Input` $\to$ `Target` $\to$ `Both` $\to$ `None`).
   - **Measure:** Click to cycle through `Scale` $\to$ `Ordinal` $\to$ `Nominal` with crisp vector icons.
   - **New Variable Ghost Row (`*`):** An empty row is always available at the bottom; typing a name automatically registers the variable and appends a new column to `Data View`.
   - **Row Context Menu:** Right-clicking row numbers brings up "Insert Variable" and "Clear / Delete Variable".

4. **Interactive `Data View`:**
   - **In-Place Cell Editing:** Double-clicking or pressing Enter focuses an inline singleline editor over the cell.
   - **Keyboard Navigation:** Pressing `Enter` commits the value and advances focus to the row below; pressing `Tab` advances to the next column (`Shift+Tab` to previous); pressing `Escape` cancels editing.
   - **Automatic Case Expansion:** Typing into any cell on the bottom row automatically expands `row_count` by 1.
   - **Case Context Menu:** Right-clicking case numbers brings up "Insert Case" and "Delete Case".

5. **L5 Command Bus (`statcraft_syntax`):**
   - Implemented and registered commands: `var.rename`, `var.set_type`, `var.set_decimals`, `var.set_width`, `var.set_align`, `var.add`, `var.delete`, `data.add_row`, `data.delete_row`.

6. **Verification & Definition of Done:**
   - Strict Resource Guard: Compilation and testing strictly executed with `-j 1`.
   - Test suite expanded to 63 unit and integration tests: 100% passing (`cargo test --workspace -j 1`).
   - `cargo clippy --workspace -j 1 -- -D warnings` — 100% clean, 0 warnings.
   - `cargo fmt --check` — 100% clean formatting.
   - Architectural layering verified with `cargo xtask layers`.
   - Localization linter `i18n_lint_test` verifies 0 hardcoded Cyrillic strings and 100% key parity between `ru.ftl` and `en.ftl`.

---

### Phase 5: Output Viewer Architecture, Outline Tree & TSV Export
*Time: October 27, 2026, 22:30 (~20 minutes)*

Completed the fifth core milestone of the StatCraft roadmap — full modernization of the **Output Viewer** to achieve 100% functional parity with IBM SPSS Statistics Viewer:

1. **Hierarchical Document Data Model:**
   - Implemented a structured typed document model on L6 `statcraft_app`: `OutputProcedure`, `OutputNotes` (dataset & sample metadata), `OutputItem`, and `OutputItemKind` (`DescriptivesTable`, `FrequencyTable`, `DatasetSummary`, `Warning`, `CustomText`), managed via `SelectedTarget`.
   - Results now accumulate in an infinite, continuous vertical document scroll — new analyses append below without clearing or replacing previous results.

2. **Outline Tree Pane & Vector-First UI:**
   - The left outline pane includes a quick action toolbar: "Clear All" (`output-btn-clear-all`), "Expand All" (`output-btn-expand-all`), and "Collapse All" (`output-btn-collapse-all`).
   - Collapsible hierarchical tree: document root (`paint_output_root_icon`), procedure folders (`paint_output_folder_icon` + `paint_vector_arrow`), syntax log (`paint_output_log_icon`), analysis notes (`paint_output_notes_icon`), pivot tables (`paint_output_table_icon`), and warnings (`paint_output_warning_icon`).
   - All icons are 100% vector-rendered using `ui.painter()`, preventing "tofu" glyph rendering artifacts.

3. **Synchronized Navigation & Focus Indicator:**
   - Clicking any item in the outline tree smoothly scrolls the canvas directly to that element (`ui.scroll_to_rect`) and highlights it with an accent focus outline (`Tokens::ACCENT`).
   - Clicking an item directly on the canvas synchronizes the selection back to the outline tree.
   - Fast item deletion via `Delete` key and context menus.

4. **TSV Export (Excel / Google Sheets Compatibility):**
   - Implemented tab-separated values (TSV) serialization for descriptives, frequencies, notes, and dataset summaries.
   - Table context menus allow one-click copying directly to the system clipboard for immediate pasting into spreadsheets with column integrity preserved.

5. **Syntax Logging & Analysis Metadata:**
   - The "Paste" button in `Descriptives` and `Frequencies` dialogs appends genuine SPSS syntax to the syntax log.
   - Analyses automatically record an authentic SPSS-style `Notes` block detailing dataset name, case count, variable count, and UTC timestamp.

6. **Verification & Definition of Done:**
   - Strict Resource Guard: compilation and checks run strictly with `-j 1`.
   - 100% workspace tests passing (`cargo test --workspace -j 1`).
   - `cargo clippy --workspace -j 1 -- -D warnings` and `cargo fmt --check` clean.
   - Architectural layering verified with `cargo run --package xtask -j 1 -- layers`.
   - Localization linter `i18n_lint_test` verifies zero hardcoded Cyrillic strings and complete key parity.

---

### Phase 5.1: Elimination of Layout Overflows in i18n (Layout Shield)
*Time: October 27, 2026, 23:20 (~25 minutes)*

Resolved layout overflow and column collision issues when switching to languages with longer word lengths (Russian, German, etc.):
1. **Dynamic Column Width Measurement in Pivot Tables (`output_view.rs`):**
   - Replaced static column widths with dynamic measurement via `painter.layout_no_wrap` considering font and cell padding (`max(base_min_w, measured_text_w + padding)`). This handles compound multi-tier headers ("Valid Percent", "Cumulative Percent") without truncation.
   - Tables wrapped in `ScrollArea::horizontal()`, enabling smooth scrolling across multi-column tables.
   - Each cell strictly clips its rendering area (`with_clip_rect`), preventing text spillover across column borders.
2. **Compact Vector Outline Toolbar:**
   - Replaced overflowing text buttons ("Clear All", "Expand All", "Collapse All") with compact vector icon buttons ($24 \times 22$ px) equipped with hover tooltips:
     - `paint_icon_trash` — trash can for clearing all entries;
     - `paint_icon_expand_all` — expand outline tree;
     - `paint_icon_collapse_all` — collapse outline tree.
   - Enforced hard clipping (`ui.set_clip_rect`) on both outline and canvas panels, eliminating cross-separator spillover.
3. **Auto-Adjusting Header Widths in Variable View (`variable_view.rs`):**
   - Table headers (`Decimals`, `Missing`, `Values`, etc.) automatically adapt to localized text length without truncation.
4. **Verification & Release:**
   - 65 workspace tests passing (`cargo test --workspace -j 1`).
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 errors, 0 warnings.
   - `cargo test -p statcraft_i18n --test i18n_lint_test -j 1` — 0 raw Cyrillic strings in app source.
   - Built production release binary `target\release\statcraft_app.exe` (6,470,144 bytes).

---

### Phase 5.2: Complete Main Menu Architectural Canon & 100% IBM SPSS Statistics Parity
*Time: October 28, 2026, 00:30 (~35 minutes)*

Conducted an exhaustive architectural breakdown and design mapping of all 12 main menus from the canonical IBM SPSS Statistics Data Editor:
1. **File:** Unified project container format `.stc` (StatCraft Project), native binary SPSS `.sav` and compressed `.zsav` codecs (L0 `statcraft_io`), keyboard shortcuts, and report exporting.
2. **Edit:** Bidirectional command Undo/Redo stack on `CommandBus`, specialized clipboard transforms ("Copy with Variable Names" `Ctrl+Shift+C`), case and variable navigation (`Ctrl+G`, `Ctrl+J`), modular Preferences dialog (5 tabs: General, Data Grid, Fonts/Scale, Output, AI Bridge).
3. **View:** Value labels toggle `1/A`, visual themes (Light SPSS Classic, Dark Pro, Auto System), runtime UI language selection, status bar sample flags.
4. **Data:** Multi-level row/variable sorting, relational file merging (*Add Cases / Add Variables*), group split-file processing (*Split File On*), case filtering (*Select Cases*), sampling weights (*Weight Cases*), matrix restructuring (*Wide <-> Long*).
5. **Transform:** 100% parity with SPSS: formula calculator `Compute Variable` with `IF` logic, `Recode` (into same / different variables), automatic string-to-code `ValueLabels` recoding, dummy 0/1 variable generation for regressions, time series shifting (`Shift Values / Lag`), Date & Time wizard, missing value imputation (*Replace Missing Values*).
6. **Analyze:** Full statistical methodology catalog: descriptives, compare means (t-tests, ANOVA), General Linear Model (GLM/ANCOVA/MANOVA), correlations, multiple linear and logistic regressions, non-parametrics, factor analysis (PCA), clustering, scale reliability (Cronbach's $\alpha$), survival analysis (Kaplan-Meier, Cox).
7. **Direct Marketing:** Commercial decision-support extension (RFM analysis, profiling, A/B response testing).
8. **Graphs:** L3 vector plotting engine `statcraft_charts` (`egui_plot`), visual Chart Builder, Gaussian histograms, Boxplots with outlier identification, scatterplot matrices, interactive Brushing & Linking, vector SVG export for scientific journals.
9. **Utilities:** Quick variable inspector modal, variable sets for large-scale questionnaires (500+ items), embedded dataset research comments, study codebook generation.
10. **Extensions:** Plugin ecosystem, declarative `Custom Dialog Builder`, Model Context Protocol (MCP) server hub for agentic AI orchestration, Python/R runtime bridge.
11. **Window:** Emergency layout recovery (*Reset Layout*), split-screen presets, active document switching and protection of permanent workspaces.
12. **Help:** Interactive *Statistics Coach*, open transparent mathematical specifications for L2 algorithms, sample dataset library (Fisher's Iris, clinical trial, survey).

All 12 menus have been documented and synchronized across strategic roadmaps (`docs/ROADMAP.md` and `ROADMAP.en.md`).

---

### Phase 5.3: Canonical 12-Menu Architecture with Readiness Badges (IBM SPSS 12-Menu System)
*Time: October 28, 2026, 01:05 (~35 minutes)*

Integrated the complete canonical ensemble of all 12 menus from desktop IBM SPSS Statistics into the main StatCraft application window (`crates/statcraft_app`):
1. **File**
2. **Edit**
3. **View**
4. **Data**
5. **Transform**
6. **Analyze**
7. **Direct Marketing**
8. **Graphs**
9. **Utilities**
10. **Extensions**
11. **Window**
12. **Help**

Key engineering achievements of this phase:
1. **Reusable Menu Item Widget `menu_item.rs` (`MenuItemStatus`):**
   - Implemented strictly typed menu item status handling:
     - `Ready { shortcut }` — active interactive item with click response and aligned shortcut labels (`Ctrl+O`, `Ctrl+Z`, `Alt+F4`, `Ctrl+1..3`);
     - `ContextDisabled { reason }` — contextual disabling (e.g. analytical procedures when no active dataset is loaded) with informative tooltip *«Requires an open dataset»*;
     - `InDev` — disabled against empty clicks, rendering an aligned semantic badge `[dev]` and tooltip *«In development (planned in Roadmap)»*.
2. **Localization Catalogs `locales/en.ftl` & `locales/ru.ftl`:**
   - Added over 170 new semantic translation keys covering all menu items, flyout submenus (compare means, regression, dimension reduction, file merging, legacy graphs, etc.).
   - `crates/statcraft_i18n/tests/i18n_lint_test.rs` verified 100% key parity between English and Russian catalogs with zero raw Cyrillic characters in the application source code.
3. **Action Bindings in `app.rs`:**
   - **File:** `Open CSV...` (`Ctrl+O`), `Exit` (`Alt+F4`);
   - **Edit:** `Undo` (`Ctrl+Z`), `Redo` (`Ctrl+Y`);
   - **View:** `Value Labels` (toggle between numeric codes and text labels), `Language` (runtime RU/EN switcher);
   - **Analyze:** `Descriptives...` and `Frequencies...` (opening modal analysis dialogs when dataset is present);
   - **Window:** `Reset Workspace Layout` (recovering default docking state: Data View on left, Output Viewer on right), `Analytical Split View`, quick tab focusing for `Data View` (`Ctrl+1`), `Variable View` (`Ctrl+2`), `Output Viewer` (`Ctrl+3`).
4. **Verification & Artifacts:**
   - 100% of workspace tests passing (`cargo test --workspace -j 1`).
   - Architecture layer integrity verified (`cargo xtask layers`).
   - `cargo clippy --workspace -j 1 -- -D warnings` — clean (0 errors, 0 warnings).
   - Produced optimized release binary `target/release/statcraft_app.exe`.

---

### Phase 5.4: Monolithic Custom Titlebar in PhotoCraft / Photoshop Style
*Time: October 28, 2026, 01:30 (~25 minutes)*

Transformed the application's top bar into a unified, modern Client-Side Decoration (CSD) titlebar modeled on the PhotoCraft architectural standard:
1. **Borderless Window Mode (`main.rs`):**
   - Configured `ViewportBuilder::default().with_decorations(false).with_resizable(true)` to eliminate the heavy native OS frame.
2. **Unified Monolithic 34 px Bar (`titlebar.rs` & `app.rs`):**
   - **Left:** Crisp, enlarged StatCraft icon ($24 \times 24$ px) **with no redundant application name text alongside**, immediately preceding all 12 canonical menus;
   - **Center:** Adaptive centering of the active document name (`Untitled-1 •` or the imported dataset name) using `title_x` with collision avoidance and graceful truncation/hiding on narrow viewports;
   - **Right:** Native-sized Windows caption button group (width $46 \times 3 = 138$ px):
     - 100% vector-rendered glyphs (`line_segment`, `rect_stroke`) for Minimize, Maximize/Restore, and Close;
     - Distinctive accent red hover background (`CAPTION_CLOSE_HOVER`) on Close, subtle theme hover on other controls;
     - Localized hover tooltips (`caption-minimize`, `caption-maximize`, `caption-restore`, `caption-close`).
3. **Frameless Window Interactivity:**
   - Entire free gap between menus and caption buttons supports dragging (`ViewportCommand::StartDrag`) and double-click maximize/restore toggle;
   - Invisible perimeter resize zones (`render_resize_zones`) with 4 px edges and 8 px corners with responsive mouse cursor icons, auto-suppressed when maximized.
4. **Verification & DoD:**
   - 100% passing across all 65 workspace tests (`cargo test --workspace -j 1`).
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 warnings.
   - Built optimized production binary `target/release/statcraft_app.exe`.

---

### Phase 5.5: Responsive Menu Overflow, Vertical Centering & Window Drag Restoration
*Time: October 28, 2026, 01:55 (~25 minutes)*

Addressed user feedback on top bar layout alignment and frameless window interactions:
1. **Geometric Vertical Centering:**
   - Switched from `ui.horizontal` (with implicit top `Align::Min`) to `ui.with_layout(Layout::left_to_right(Align::Center))`.
   - The $24 \times 24$ px icon and 13 pt menu buttons are now vertically centered within the 34 px bar with balanced 5 px and 7 px margins.
2. **Restored Dragging and Double-Click Maximize/Restore:**
   - Identified root cause: `egui::menu::bar` internally called `ui.set_min_width(ui.available_width())`, consuming all remaining horizontal width and collapsing `drag_rect` width to `0.0`.
   - Replaced expanding `menu::bar` with direct centered `ui.menu_button` calls, giving an accurate `menus_end_x` measurement.
   - The large `drag_rect` gap between menus and window caption buttons now reliably dispatches `ViewportCommand::StartDrag` on drag and `ViewportCommand::Maximized` on double click.
3. **Responsive Menu Overflow (`»`):**
   - Calculated font-rendered widths for each menu item via `ui.fonts().layout_no_wrap`.
   - If a menu item cannot fit before the right caption button group (`controls_start_x`), it and all subsequent menus gracefully collapse into a compact overflow menu button (`»` / `t!("menu-overflow")`) displaying them as cascading submenus.
   - Menus never overlap the window caption buttons regardless of window width.
4. **Quality & Verification (DoD):**
   - 100% workspace tests passing (`cargo test --workspace -j 1`).
   - Localized via Fluent with zero raw Cyrillic strings in app code (`i18n_lint_test`).
   - Clean `clippy --workspace -j 1 -- -D warnings` (0 errors, 0 warnings).
   - Produced optimized release binary `target/release/statcraft_app.exe`.

---

### Phase 5.6: Canonical Toolstrip with Vector SVG Icons (PhotoCraft Standard)
*Time: October 28, 2026, 02:30 (~25 minutes)*

Implemented a full-featured Toolbar / Toolstrip modeled on IBM SPSS Statistics using PhotoCraft's vector SVG icon architecture:
1. **Vector SVG Icon Infrastructure (`egui_extras` + `icons.rs`):**
   - Integrated official `egui_extras` with `"svg"` feature enabled;
   - Compile-time embedded monochromatic Lucide SVG icon catalog via `include_bytes!` in `icon_data.rs` (zero external asset paths, zero network dependency);
   - Built `SvgIconSet` preprocessor replacing `currentColor` with `#ffffff` and normalizing stroke widths for crisp DPI-scaled hardware rasterization;
   - `crate::icons::icon_image` enables instant runtime tinting to any design system token (`Tokens::TEXT_PRIMARY`, `Tokens::TEXT_MUTED`, `Tokens::ACCENT`, `Color32::WHITE`).
2. **Toolbar Component (`toolbar.rs`):**
   - Flexible placement: **Top Toolbar** (default beneath menu), **Left Toolbar**, and **Hide Toolbar** toggleable in `View -> Toolbars`;
   - Full 16-button canonical SPSS set organized into 6 logical groups with vector separators:
     1. *File Operations:* `Open Data`, `Save Data`, `Print`;
     2. *History & Recall:* `Recall Dialogs`, `Undo` (reactive to command history), `Redo`;
     3. *Navigation & Search:* `Go to Case`, `Go to Variable`, `Variables Info`, `Find`;
     4. *Data Structure:* `Insert Case` (`data.add_row`), `Insert Variable` (`var.add`);
     5. *Data Preparation:* `Split File`, `Select Cases`, `Weight Cases`;
     6. *Display & Analysis:* `Value Labels (1/A)` (interactive toggle state), quick-launch for `Descriptives` and `Frequencies`.
3. **Localization & Interactivity:**
   - 100% of tooltips with keyboard shortcuts localized in `locales/en.ftl` and `locales/ru.ftl`;
   - Real action dispatch to `CommandBus` and analysis modal dialogs.
4. **Quality & DoD:**
   - Localization linter `i18n_lint_test` — 2/2 passed (0% raw Cyrillic);
   - Workspace tests `cargo test --workspace -j 1` — 65/65 passed;
   - Architecture layer integrity `cargo xtask layers` — verified;
   - Linter `cargo clippy --workspace -j 1 -- -D warnings` — 0 warnings;
   - Produced clean release binary `target/release/statcraft_app.exe`.

---

### Phase 6.1: One-Sample Student's T Test (Compare Means)
*Time: October 28, 2026, 09:40 (~35 minutes)*

Implemented the foundational procedure of the **Compare Means** family — Student's One-Sample T-Test (`Analyze -> Compare Means -> One-Sample T Test...`):
1. **Mathematical Engine L2 (`statcraft_algo::ttest_onesample`):**
   - Built Student's t cumulative distribution function, 2-tailed $p$-value, and inverse quantile function (calculating critical $t_{\alpha/2, df}$ for confidence intervals) in 100% pure safe Rust without third-party dependencies;
   - Implemented regularized incomplete beta function $I_x(a, b)$ using modified Lentz continued fractions and Lanczos approximation for $\ln\Gamma(z)$;
   - Golden Oracle verification against Python SciPy (`scipy.stats.ttest_1samp`): t-statistic, $p$-value, degrees of freedom, and 95% difference CI match standard benchmarks to $10^{-8}$;
   - Full Never Crash guarantees for degenerate edge cases ($N < 2$, zero sample variance, missing/NA values).
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::ttest_onesample_cmd`):**
   - Typed `OneSampleTTestCommand` (`stat.ttest.onesample`) with parameter schema for AI agents;
   - Canonical SPSS syntax generator:
     ```text
     T-TEST
       /TESTVAL=100.0
       /CRITERIA=CI(0.95)
       /VARIABLES=salary.
     ```
3. **User Interface L6 (`statcraft_app`):**
   - Dedicated modal dialog `OneSampleTTestDialog` ($500 \times 380$ px) with variable selection lists, vector scale icons, Test Value numeric input, and custom confidence level percentage;
   - Enabled `Analyze -> Compare Means -> One-Sample T Test...` menu action with reactive data availability checks;
   - Rendered dual SPSS Pivot Tables in the Output Viewer:
     1. **One-Sample Statistics:** Variable, N, Mean, Std. Deviation, Std. Error Mean;
     2. **One-Sample Test (Test Value = ...):** Variable, t, df, Sig. (2-tailed), Mean Difference, 95% Confidence Interval of the Difference (Lower, Upper);
   - Context menu TSV clipboard exporter and outline tree integration.
4. **Quality & DoD:**
   - 100% workspace tests passing (`cargo test --workspace -j 1`);
   - Architecture layer rules verified (`cargo run -p xtask -j 1 -- layers`);
   - Command parity tracked (`cargo run -p xtask -j 1 -- parity`);
   - Localization linter `i18n_lint_test` — 2/2 passed (zero unlocalized strings);
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 errors, 0 warnings.
---

### Phase 6.2: Independent-Samples T Test (Compare Means)
*Time: October 28, 2026, 10:20 (~40 minutes)*

Implemented the core statistical procedure for experimental research and A/B testing — **Independent-Samples T-Test** (Analyze -> Compare Means -> Independent-Samples T Test...):
1. **Mathematical Engine L2 (statcraft_algo::ttest_indep):**
   - Full implementation of Levene's Test for Equality of Variances (canonical SPSS $-test centered at the group mean center=\'mean\');
   - Dual-row hypothesis testing calculated simultaneously (IBM SPSS Statistics canon):
     1. *Equal variances assumed* — classic Student\'s t-test with pooled variance ^2$ and degrees of freedom  = N_1 + N_2 - 2$;
     2. *Equal variances not assumed* — Welch\'s t-test with separate variances and Satterthwaite degrees of freedom {welch}$;
   - Descriptive group statistics (, \bar{x}, SD, SE$), exact mean difference $\Delta$, standard error of difference {diff}$, and confidence intervals;
   - **Golden Oracle:** Validated against Python SciPy (scipy.stats.levene and scipy.stats.ttest_ind(equal_var=True/False)) with absolute precision $\le 10^{-6}$;
   - Never Crash guarantees for degenerate edge cases ( < 2$, zero sample variance, and numeric/string group codes such as 1/2, 1.0, M/F).
2. **Command Bus & Syntax L5 (statcraft_syntax::commands::ttest_indep_cmd):**
   - Typed engine command IndependentSamplesTTestCommand (stat.ttest.indep) with parameter schema for AI agents;
   - Canonical SPSS syntax generator:
     `	ext
     T-TEST GROUPS=gender(1 2)
       /CRITERIA=CI(0.95)
       /VARIABLES=salary score.
     `
3. **User Interface L6 (statcraft_app):**
   - Dedicated dialog IndependentSamplesTTestDialog ( \times 420$ px) with source variable list, test variables target, grouping variable slot, and vector transfer arrows;
   - Subdialog Define Groups... for specifying Group 1 and Group 2 labels;
   - Activated menu item Analyze -> Compare Means -> Independent-Samples T Test... with dataset availability validation;
   - Rendered authentic two-tier SPSS Pivot Tables in Output Viewer:
     1. **Group Statistics:** Variable, Group, N, Mean, Std. Deviation, Std. Error Mean;
     2. **Independent Samples Test:** Two-tier header with *Levene\'s Test for Equality of Variances* ($, Sig.) and *t-test for Equality of Means* ($, $, Sig., Mean Diff, Std. Error Diff, 95% CI Lower/Upper) across both variance assumption rows;
   - Context menu TSV clipboard export (Excel/Sheets) and outline navigation.
4. **Quality & DoD:**
   - Workspace unit tests cargo test --workspace -j 1 — 100% passed;
   - Architecture layer rules cargo run -p xtask -j 1 -- layers — 0 violations;
   - Parity catalog cargo run -p xtask -j 1 -- parity — command registered with Done status;
   - Localization linter i18n_lint_test — 2/2 passed (0% raw strings);
   - cargo clippy --workspace -j 1 -- -D warnings — 0 errors, 0 warnings.

---

### Phase 6.3: Paired-Samples T Test (Compare Means)
*Time: October 28, 2026, 11:00 (~35 minutes)*

Implemented the third procedure in the canonical IBM SPSS Statistics **Compare Means** module — **Paired-Samples T Test** (menu `Analyze -> Compare Means -> Paired-Samples T Test...`):
1. **Mathematical Engine L2 (`statcraft_algo::ttest_paired`):**
   - Paired sample descriptive statistics for both variables ($N, \bar{x}_1, \bar{x}_2, s_1, s_2, SE_1, SE_2$);
   - Pearson linear correlation coefficient ($r$), $t$-statistic for correlation and 2-tailed significance ($p_r$) with degrees of freedom $df = N - 2$;
   - Paired differences analysis $d_i = x_i - y_i$: mean difference $\bar{d}$, unbiased sample standard deviation of differences $s_d$, standard error of difference $SE_d$, Student's $t$-statistic, $df = N - 1$, 2-tailed $p$-value, and $100(1-\alpha)\%$ confidence interval of the difference;
   - **Golden Oracle:** Validated against Python `scipy.stats` (`ttest_rel` and `pearsonr`) with precision $\le 10^{-6}$;
   - **Never Crash** guarantees: robust handling of small samples ($N < 2$), degenerate zero-variance differences ($s_d = 0$), and listwise missing value exclusion per pair in 100% safe Rust without `unwrap`/`expect`/`panic`.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::ttest_paired_cmd`):**
   - Strongly-typed `PairedSamplesTTestCommand` (`stat.ttest.paired`) with JSON parameter schema `specs()` for AI agents;
   - Canonical SPSS syntax generator:
     ```text
     T-TEST PAIRS=before WITH after (PAIRED)
       /CRITERIA=CI(0.95).
     ```
3. **User Interface L6 (`statcraft_app`):**
   - Dedicated modal dialog `PairedSamplesTTestDialog` ($520 \times 420$ px) strictly matching the StatCraft canonical desktop dialog layout (source variable list $196 \times 240$ px, vector transfer buttons `>` and `<`, paired variables target list with current selection slots `Variable 1` / `Variable 2`, custom confidence interval %, and canonical bottom horizontal action bar `[ OK ] [ Paste ] [ Reset ] [ Cancel ]`);
   - Activated menu action `Analyze -> Compare Means -> Paired-Samples T Test...` with dataset presence validation;
   - Rendered 3 authentic SPSS Pivot Tables in the Output Viewer:
     1. **Paired Samples Statistics:** Pair $k$, Variable, Mean, N, Std. Deviation, Std. Error Mean;
     2. **Paired Samples Correlations:** Pair $k$ ($v_1$ & $v_2$), N, Correlation, Sig.;
     3. **Paired Samples Test:** Two-tier header with *Paired Differences* super-header (Mean, Std. Deviation, Std. Error Mean, 95% Confidence Interval of the Difference [Lower, Upper]), followed by $t$, $df$, and $Sig. (2\text{-tailed})$;
   - Context menu TSV clipboard export (Excel/Google Sheets) and outline tree integration.
4. **Quality & Definition of Done:**
   - 100% workspace tests passing: `cargo test --workspace -j 1`;
   - Zero hardcoded strings confirmed by localization linter: `i18n_lint_test`;
   - Layer architecture verified: `cargo run -p xtask -j 1 -- layers`;
   - Parity catalog updated: `cargo run -p xtask -j 1 -- parity`;
   - Pure compiler cleanliness: `cargo clippy --workspace -j 1 -- -D warnings` — 0 errors, 0 warnings;
   - Clean formatting: `cargo fmt --check`.

---

### Phase 6.4: One-Way ANOVA and Output Viewer Modular Architecture
*Time: October 28, 2026, 13:40 (~40 minutes)*

Completed the fourth and final component of the canonical IBM SPSS Statistics **Compare Means** module — **One-Way ANOVA** (menu `Analyze -> Compare Means -> One-Way ANOVA...`), coupled with an architectural modularization of the Output Viewer subsystem:
1. **Output Viewer Modular Refactoring (`statcraft_app::views::output_view`):**
   - Eliminated the monolithic 3000+ line `output_view.rs` file that previously caused token saturation and subagent diff failures;
   - Decomposed output presentation into modular components strictly bounded under 350 lines:
     * `model.rs`: data model `OutputViewerState`, `OutputEntry`, and typed `OutputPayload`;
     * `outline.rs`: interactive outline navigation pane, vector icons for folders/tables/root document, and toolbar (Clear All, Expand/Collapse All);
     * `tables_descriptives.rs`: SPSS descriptive statistics Pivot Table + TSV export;
     * `tables_frequencies.rs`: SPSS frequency and quartiles tables + TSV export;
     * `tables_ttest.rs`: Pivot tables for all 3 Student's t-tests (One-Sample, Independent, Paired) + TSV export;
     * `tables_anova.rs`: One-Way ANOVA tables (Descriptives, Levene test, ANOVA table) + TSV export;
     * `mod.rs`: layout coordination, re-exports, and rendering dispatcher.
2. **Mathematical Engine L2 (`statcraft_algo::anova_oneway`):**
   - Partition of variance: between-group ($SS_{between}$), within-group ($SS_{within}$), and total sum of squares ($SS_{total}$);
   - Degrees of freedom: $df_1 = k - 1$, $df_2 = N - k$, $df_{total} = N - 1$;
   - Mean squares ($MS_{between}, MS_{within}$) and Fisher-Snedecor $F$-statistic $F = \frac{MS_{between}}{MS_{within}}$;
   - Exact $p$-value survival function of the $F$-distribution via regularized incomplete beta function $I_x(a, b)$;
   - Levene's Test of Homogeneity of Variances;
   - Factor level descriptive statistics ($N, \bar{y}, s, SE$, 95% CI, Min, Max + Total row);
   - **Golden Oracle:** 100% precision match against Python `scipy.stats` (`scipy.stats.f_oneway`) within $\le 10^{-6}$.
3. **Command Bus & Syntax L5 (`statcraft_syntax::commands::anova_oneway_cmd`):**
   - Strongly-typed `OneWayAnovaCommand` (`stat.anova.oneway`) with parameter schema for external AI agents;
   - Canonical SPSS syntax generator:
     ```text
     ONEWAY salary bonus BY jobcat
       /STATISTICS DESCRIPTIVES HOMOGENEITY
       /MISSING ANALYSIS.
     ```
4. **User Interface L6 (`statcraft_app`):**
   - Dedicated modal dialog `OneWayAnovaDialog` ($520 \times 440$ px) with dependent variables transfer list, categorical factor selector, Descriptives / Homogeneity options, and confidence interval configuration;
   - Activated menu action `Analyze -> Compare Means -> One-Way ANOVA...`;
   - Rendered 3 authentic SPSS Pivot Tables in the Output Viewer («Descriptives», «Test of Homogeneity of Variances», «ANOVA»), clipboard TSV export, and outline tree integration.
5. **Quality & Definition of Done:**
   - 100% workspace tests passing: `cargo test --workspace -j 1`;
   - Zero hardcoded strings confirmed by localization linter: `i18n_lint_test`;
   - Layer architecture verified: `cargo run -p xtask -j 1 -- layers`;
   - Parity catalog updated: `cargo run -p xtask -j 1 -- parity`;
   - Pure compiler cleanliness: `cargo clippy --workspace -j 1 -- -D warnings` — 0 errors, 0 warnings;
   - Clean formatting: `cargo fmt --check`.

---

### Phase 6.5: File Size Guard & Regression Shield
*Time: October 28, 2026, 15:00 (~35 minutes)*

Engineered an automated multi-layer defense against source file bloat, context window degradation, and hidden UI regressions:
1. **Tooling Guard (`xtask/src/file_guard.rs`):**
   - Implemented and integrated the source linter `cargo run -p xtask -j 1 -- file-guard` with differentiated thresholds:
     * Leaf functional modules (`algo/`, `syntax/commands/`, `dialogs/`, `views/output_view/`): strict limit of **500 lines**;
     * Complex interactive editors (`data_view.rs`, `variable_view.rs`, `syntax_editor.rs`): limit of **1000 lines**;
     * Root application coordinator (`app.rs`): ceiling of **1500 lines**;
     * General modules: limit of **1000 lines**;
   - Instructive error message on threshold violation: categorically forbids truncating columns/features or compressing code into one-liners, mandating architectural decomposition instead ("Decompose, Don't Cut" principle).
2. **Architectural Decomposition of Tables and Models (`output_view/`):**
   - Decomposed monolithic `tables_ttest.rs` (571 lines) into three focused modules:
     * `tables_ttest_onesample.rs` (127 lines): One-Sample Statistics and One-Sample Test tables + TSV;
     * `tables_ttest_indep.rs` (180 lines): two-tier Group Statistics and Independent Samples Test (Levene + Welch/Student) tables + TSV;
     * `tables_ttest_paired.rs` (204 lines): Paired Statistics, Correlations, and Paired Differences tables + TSV;
     * `tables_common.rs` (90 lines): reusable layout and cell painting utilities;
   - Decomposed `model.rs` (616 lines) into clean data structures (`model.rs`, 175 lines) and procedure builder methods (`builder.rs`, 456 lines);
   - Extracted subdialog `ttest_indep_define_groups.rs` (46 lines) from `ttest_indep_dialog.rs` (462 lines);
   - Extracted format commands `var_format_cmds.rs` (270 lines) from `var_cmds.rs` (294 lines);
   - Result: 100% of workspace files comply with size limits (76 files scanned).
3. **UI Invariants Test Suite (`crates/statcraft_app/tests/output_viewer_invariants.rs`):**
   - Created headless UI integration tests eliminating the risk of accidental feature erasure:
     1. `test_continuous_history_accumulation`: verifies continuous retention of all analyses (Descriptives, Frequencies, 3 t-tests, ANOVA), strictly monotonic procedure IDs, and preservation of prior Notes/Log blocks;
     2. `test_exhaustive_enum_payloads_render_without_panics`: headless rendering of all 11 `OutputItemKind` variants in `egui` (including edge cases: zero degrees of freedom, empty lists, zero variance) guaranteeing 0 panics;
     3. `test_tsv_export_golden_headers`: verifies exact golden TSV structures (presence of `Mean`, `Std. Deviation`, `Valid N (listwise)`, `Valid Percent`, `Total`, `Levene`, `Equal variances assumed / not assumed`, `Paired Differences`, `Between Groups`, `Sum of Squares`, `F`, `Sig.`);
     4. `test_outline_tree_navigation_integrity`: verifies synchronization between Outline tree focus and procedure scroll, and item deletion integrity.
4. **Normative Grounding (`AGENTS.md`):**
   - Added Section 12 "File Size Guard & Feature Retention" to the general development regulations (`AGENTS.md` and `.agents/rules/AGENTS.md`);
   - Integrated `cargo run -p xtask -j 1 -- file-guard` and `cargo test -p statcraft_app --test output_viewer_invariants -j 1` as mandatory Definition of Done (DoD) criteria.
5. **Quality & Definition of Done:**
   - Source linter `cargo run -p xtask -j 1 -- file-guard` — 76 files scanned, 0 violations;
   - Invariant tests `output_viewer_invariants` — 4/4 passed;
   - Workspace tests `cargo test --workspace -j 1` — 100% passed;
   - Layer architecture verified: `cargo run -p xtask -j 1 -- layers` — 0 violations;
   - Parity catalog verified: `cargo run -p xtask -j 1 -- parity` — clean;
   - Localization linter `i18n_lint_test` — 3/3 passed;
   - Pure compiler cleanliness: `cargo clippy --workspace -j 1 -- -D warnings` — 0 errors, 0 warnings;
   - Clean release build: `cargo build --package statcraft_app --release -j 1` — succeeded (1m 12s).

---

### Phase 6.6: DialogKit Design System Standardization & Design Guard
*Time: October 28, 2026, 16:05 (~40 minutes)*

Eliminated long-standing visual and architectural dialog inconsistencies: mismatched sizes (`500x380`, `500x420`, `520x440`), shifting action buttons lacking a fixed layout grid, and ad-hoc variable transfer widgets were replaced by the canonical IBM SPSS Statistics design system backed by automated static linting:
1. **Canonical Design Specification (`docs/DESIGN_GUIDE.md`):**
   - Formalized 4 core StatCraft dialog archetypes: Type 1 (Statistical Procedure, $560 \times 430$ px), Type 2 (Subdialog/Options, $360 \times 300$ px), Type 3 (Properties, $460 \times 380$ px), Type 4 (Compute, $620 \times 480$ px);
   - Standardized layout grid, spacing (padding $12$ px, gap $8$/$12$ px), and typography;
   - Established the **`i18n Shield`** adaptability rule: uniform action bar button width with padding $\ge 10$ px, preventing layout shifts across language switches (RU $\leftrightarrow$ EN).
2. **`DialogKit` Component Library (`crates/statcraft_app/src/dialogs/common/`):**
   - `DialogShell`: universal modal window wrapper with standardized dimensions, viewport centering, and modal isolation;
   - `TransferButton`: adaptive vector transfer button ($28 \times 28$ px, $3$ px corner radius), automatically orienting the arrow (`->` / `<-`) towards the active target list;
   - `VariablePicker`: two-pane variable list with vector measurement scale icons and canonical `Label [Name]` formatting;
   - `DialogActionBar`: Type 1 bottom action bar (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`) with uniform button widths ($\ge 80 \times 26$ px, $3$ px corner radius);
   - `SubdialogActionBar`: bottom action bar for modal subdialogs (`[ Continue ] [ Cancel ] [ Help ]`);
   - `SubdialogSidebar`: right vertical action strip for SPSS configuration dialogs (`[ Statistics... ]`, `[ Options... ]`, $90 \times 24$ px);
   - `GroupBox`: visual grouping container for checkboxes, radio buttons, and inputs.
3. **Automated Static Tooling Guard (`xtask/src/design_guard.rs`):**
   - Implemented static design linter `cargo run -p xtask -j 1 -- design-guard` analyzing all dialog sources:
     * Check 1: Prohibits raw `ui.button(...)` / `Button::new(...)` invocations for system actions `OK`, `Paste`, `Reset`, `Cancel`, `Continue`;
     * Check 2: Prohibits raw `Window::new(...)` calls and hardcoded arbitrary window dimensions bypassing `DialogShell`;
   - Successfully flagged all 50 initial violations and confirmed complete cleanliness after refactoring.
4. **Headless Invariants Test Suite (`crates/statcraft_app/tests/dialog_invariants.rs`):**
   - Engineered headless UI test suite:
     * Verification of canonical `DialogSize` dimensions and `SubdialogSidebar` constants;
     * Headless rendering of all procedure and properties windows (Descriptives, Frequencies, One-Way ANOVA, 3 T-tests, Value Labels, Missing Values, Variable Type) in `egui::Context` guaranteeing 0 panics;
     * Verification of `can_run` execution gating with empty variable sets;
     * Verification of `Reset` state restoration to SPSS factory defaults.
5. **Full Application-Wide Dialog Unification:**
   - Migrated all analysis and property dialogs to `DialogKit` (`frequencies_dialog.rs`, `descriptives_dialog.rs`, `anova_oneway_dialog.rs`, `ttest_onesample_dialog.rs`, `ttest_indep_dialog.rs`, `ttest_indep_define_groups.rs`, `ttest_paired_dialog.rs`, `missing_values_dialog.rs`, `value_labels_dialog.rs`, `variable_type_dialog.rs`);
   - Extracted Descriptives, ANOVA, and T-Test options into canonical modal subdialogs.
6. **Normative Grounding (`AGENTS.md`):**
   - Added Section 13 "DialogKit Design System Standardization & Design Guard" to the general development regulations;
   - Made passing `cargo run -p xtask -j 1 -- design-guard` and `cargo test -p statcraft_app --test dialog_invariants -j 1` mandatory for Definition of Done (DoD).
7. **Quality & Definition of Done:**
   - `cargo run -p xtask -j 1 -- design-guard` — 0 violations;
   - `cargo run -p xtask -j 1 -- file-guard` — 86 files scanned, 0 file size violations;
   - Invariant tests `dialog_invariants` — 9/9 passed;
   - All workspace tests `cargo test --workspace -j 1` — 93 passed (100%);
   - Architectural layers `cargo run -p xtask -j 1 -- layers` — 0 violations;
   - Parity catalog `cargo run -p xtask -j 1 -- parity` — clean;
   - Localization linter `i18n_lint_test` — 3/3 passed;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 warnings;
   - `cargo fmt --check` — clean;
   - Clean release build `cargo build --package statcraft_app --release -j 1` — succeeded (1m 14s).

---

### Phase 6.7: Hybrid Localization System — Embedded Catalogs and Dynamic External .ftl Discovery
*Time: October 28, 2026, 18:45 (~25 minutes)*

Implemented a hybrid internationalization (i18n) architecture blending the canonical single-binary portability of the ArtCraft ecosystem with the flexibility of user-contributed translations:
1. **Guaranteed Base (Zero-Dependency Single Binary):**
   - Official `ru.ftl` and `en.ftl` catalogs remain embedded in the executable via `include_str!`. StatCraft is 100% self-contained, launches from any location, and never fails when external folders are missing.
2. **Dynamic Locale Discovery (`discover_and_load_locales`):**
   - On startup, the engine scans external paths: `STATCRAFT_LOCALES_DIR` environment variable, `locales/` directory adjacent to the binary (`current_exe()`), and the working `./locales` directory.
   - Any discovered `.ftl` file (e.g. `de.ftl`, `es.ftl`, `fr.ftl`) is registered at runtime and immediately surfaces in the `View -> Language` menu without recompiling the application.
3. **Seamless Key Overriding (Resource Overriding):**
   - When a user supplies an existing language file (e.g. custom `ru.ftl`), external keys override defaults via `add_resource_overriding` without losing untranslated embedded strings.
4. **Intelligent Language Display Name (`language_name`):**
   - Native autonym is read from the language bundle itself (`lang-name-{lang}` or `lang-name`, e.g. `Deutsch`, `Español`);
   - Falls back to active language / English translation;
   - Final fallback displays the uppercase language code (`DE`, `ES`). Completely prevents raw keys such as `[lang-name-xx]`.
5. **Release Workflow Automation (`.github/workflows/release.yml`):**
   - Windows release ZIP `statcraft-windows-x64.zip` now packages the `locales/` directory alongside `statcraft_app.exe`.
6. **Testing & Definition of Done:**
   - Authored end-to-end integration test `crates/statcraft_app/tests/i18n_external_locales_test.rs`;
   - Added 4 new unit tests in `crates/statcraft_i18n/src/lib.rs`;
   - Full test and static analysis pass via `cargo run -p xtask -j 1 -- ci` (`ALL CI QUALITY GATES PASSED!`);
   - Clean release build `cargo build --package statcraft_app --release -j 1` succeeded in 1m 08s.


---

### Phase 7.1: Contingency Tables (Crosstabs) and Pearson's Chi-Square Test of Independence ($\chi^2$)
*Time: October 28, 2026, 20:55 (~35 minutes)*

Implemented the fundamental procedure for analyzing categorical and discrete relationships — **Contingency Tables** (*Crosstabs*), association measures, and Pearson's Chi-Square Test of Independence ($\chi^2$, menu item `Analyze -> Descriptive Statistics -> Crosstabs...`):
1. **Mathematical Core L2 (`statcraft_algo::crosstabs` & `crosstabs_types`):**
   - Construction of categorical $R \times C$ cross-classification grid supporting string categories and numeric value labels;
   - Computation of observed frequencies ($O_{ij}$), expected frequencies ($E_{ij} = \frac{R_i \cdot C_j}{N}$), residuals ($O_{ij} - E_{ij}$), and cell percentage breakdowns (row %, column %, and % of total table);
   - Pearson Chi-Square test: $\chi^2 = \sum \frac{(O_{ij} - E_{ij})^2}{E_{ij}}$ with degrees of freedom $df = (R - 1)(C - 1)$;
   - Likelihood Ratio test ($G^2$): $2 \sum O_{ij} \ln(O_{ij} / E_{ij})$ for cells with $O_{ij} > 0$;
   - Double-precision $p$-value computation of the $\chi^2$ distribution survival function via regularized upper incomplete gamma function $Q(a, x)$ without external dependencies and with zero panics;
   - For $2 \times 2$ fourfold tables:
     * Yates' Continuity Correction: $\chi^2_{Yates} = \frac{N (|ad - bc| - N/2)^2}{(a+b)(c+d)(a+c)(b+d)}$;
     * Two-sided Fisher's Exact Test computing exact hypergeometric probabilities via log-gamma functions and summing extreme outcomes $P(k) \le P(a) + 10^{-11}$;
   - Symmetric measures of association: $\phi = \sqrt{\chi^2 / N}$ and Cramér's $V$ with approximate asymptotic significance;
   - **Golden Oracle:** Unit test verification comparing contingency tables and test statistics against Python `scipy.stats` (`chi2_contingency`, `fisher_exact`) with precision $\le 10^{-6}$.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::crosstabs_cmd`):**
   - Typed engine command `CrosstabsCommand` (`stat.crosstabs`) equipped with JSON parameter specs for external AI agents;
   - Generates authentic IBM SPSS syntax:
     ```text
     CROSSTABS
       /TABLES=gender BY jobcat
       /STATISTICS=CHISQ PHI
       /CELLS=COUNT EXPECTED ROW COLUMN TOTAL RESID.
     ```
3. **User Interface L6 (`statcraft_app::dialogs::crosstabs_dialog`):**
   - Fully unified `CrosstabsDialog` ($560 \times 430$ px) built exclusively with `DialogKit` design primitives:
     * Source variables picker (`VariablePicker`) on the left;
     * Target `Row(s)` and `Column(s)` list pickers in the center with vector `TransferButton`;
     * Right sidebar (`SubdialogSidebar`) hosting `[ Cells... ]` and `[ Statistics... ]` subdialog buttons;
     * Modal subdialogs for `Cells` (Counts: Observed, Expected, Residuals; Percentages: Row, Column, Total) and `Statistics` (Chi-square, Phi and Cramér's V) using `SubdialogActionBar`;
     * Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Architectural Coordinator Decomposition (`app_dialogs.rs`):**
   - Extracted all modal dialog rendering and execution dispatching out of `app.rs` into `app_dialogs.rs`;
   - Reduced `app.rs` from 1485 lines down to 1138 lines, safely below the 1500-line architectural ceiling.
5. **Output Viewer Representation (`tables_crosstabs.rs`):**
   - Four authentic SPSS Pivot Tables:
     1. **Case Processing Summary:** valid, missing, total cases and percentages;
     2. **Crosstabulation Table:** multi-tier $R \times C$ matrix displaying all selected cell layers, row totals, and column totals;
     3. **Chi-Square Tests:** Pearson Chi-Square, Continuity Correction, Likelihood Ratio, Fisher's Exact Test, N of Valid Cases;
     4. **Symmetric Measures:** Phi, Cramér's V, and approximate significance;
   - TSV clipboard export for spreadsheet workflows and Outline Pane navigation.
6. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - All 96 workspace tests completed with 0 failures;
   - Static linters `design-guard`, `file-guard`, `layers`, and `parity` reporting 0 violations;
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Format validation `cargo fmt --check` — completely clean.

---

### Phase 7.3: Nonparametric Tests for Two Samples (Mann-Whitney $U$ and Wilcoxon $W$)
*Time: October 28, 2026, 22:15 (~45 minutes)*

Implemented the fundamental nonparametric framework for comparing two groups on ordinal scales or non-normally distributed continuous variables — **Two-Independent-Samples Tests** (`Analyze -> Nonparametric Tests -> Two Independent Samples...`):
1. **Mathematical Engine L2 (`statcraft_algo::nonparam_twosample` & `nonparam_twosample_types`):**
   - Combined ranking algorithm with fractional mid-ranks and tie sum $T = \sum (t_i^3 - t_i)$;
   - Calculation of rank sums ($W_1, W_2$) and mean ranks ($\bar{R}_1, \bar{R}_2$) for both groups;
   - Mann-Whitney test statistic $U = \min(U_1, U_2)$, where $U_1 = W_1 - \frac{n_1(n_1 + 1)}{2}$;
   - Wilcoxon $W$ statistic definition adhering to the IBM SPSS Statistics standard (rank sum of the group with the smaller mean rank);
   - Expected value $\mu_U = \frac{n_1 n_2}{2}$ and tie-corrected variance $\sigma_U^2 = \frac{n_1 n_2}{12}\left[ (N + 1) - \frac{\sum (t_i^3 - t_i)}{N(N - 1)} \right]$;
   - Asymptotic $Z$-statistic and two-tailed significance derived from the regularized upper incomplete gamma function $Q(0.5, Z^2 / 2)$;
   - Wilcoxon Signed-Rank Test for paired observations with ranking of absolute differences $|d_i|$, exclusion of zero-ties, and $W = \min(W^+, W^-)$;
   - **Never Crash** protection: graceful handling of small sample sizes ($N < 2$), degenerate zero-variance rank distributions, and flexible numeric/string group codes via `matches_group` on 100% safe Rust;
   - **Golden Oracle:** Test suite `nonparam_twosample_tests.rs` validating ranks, $U$, $W$, $Z$, and $p$-value against Python `scipy.stats` (`mannwhitneyu`, `wilcoxon`) and manual SPSS benchmarks with $\le 10^{-6}$ precision.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::nonparam_twosample_cmd`):**
   - Typed engine command `MannWhitneyCommand` (`stat.npar.mann_whitney`) with JSON parameter specs for external AI agents;
   - Canonical IBM SPSS Statistics syntax emission:
     ```text
     NPAR TESTS
       /M-W= salary age BY gender(1 2)
       /MISSING=ANALYSIS.
     ```
3. **User Interface L6 (`statcraft_app::dialogs::nonparam_twosample_dialog`):**
   - Modal dialog `Two-Independent-Samples Tests` ($560 \times 430$ px) built with `DialogKit`:
     * Source variables list (`VariablePicker`) with scale measurement icons;
     * Vector transfer button (`TransferButton`) for test variables and grouping variable;
     * Target variables list `Test Variable List:`;
     * Selected grouping variable display `Grouping Variable:` with `var(val1 val2)` syntax;
     * Right sidebar (`SubdialogSidebar`) with `[ Define Groups... ]` button;
     * Modal subdialog `Define Groups` based on `DialogShell::subdialog` with validation of non-empty, non-identical group values;
     * `Test Type` group with checkboxes `[x] Mann-Whitney U`, `[ ] Wilcoxon W`, `[ ] Kolmogorov-Smirnov Z`;
     * Standard bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Summary Pivot Tables in Output Viewer (`tables_nonparam.rs`):**
   - **Ranks Table:** Variable, Group, $N$, Mean Rank, Sum of Ranks, Total row;
   - **Test Statistics Table:** Canonical multi-column matrix (Mann-Whitney U, Wilcoxon W, Z, Asymp. Sig. 2-tailed) with grouping variable footnote;
   - TSV clipboard export and Outline Pane integration.
5. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 105 workspace unit and integration tests passing with 0 failures;
   - Architectural linters (`design-guard`, `file-guard`, `layers`, `parity`) reporting 0 violations;
   - Strict compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Code formatting `cargo fmt --check` — completely clean.

---

### Phase 7.4: Nonparametric Tests for Two Related Samples (Wilcoxon Signed-Rank Test and Sign Test)
*Time: October 29, 2026, 08:15 (~40 minutes)*

Implemented the fundamental nonparametric framework for paired and repeated-measures analyses when normal distribution of differences is violated — **Two-Related-Samples Tests** (`Analyze -> Nonparametric Tests -> Related Samples (Wilcoxon / Friedman)...`):
1. **Mathematical Engine L2 (`statcraft_algo::nonparam_related` & `nonparam_related_types`):**
   - Paired difference computation $D_i = Y_i - X_i$ (`var2 - var1`);
   - Zero-difference classification as Ties ($D_i = 0$), excluded from ranking per IBM SPSS Statistics standard;
   - Sorting and ranking of absolute differences $|D_i|$ with mid-ranks calculation and tie sum $T = \sum (t_j^3 - t_j)$;
   - Calculation of negative ($D_i < 0$) and positive ($D_i > 0$) rank sums and mean ranks;
   - Tie-corrected variance formula: $\sigma_W^2 = \frac{n(n + 1)(2n + 1)}{24} - \frac{\sum (t_j^3 - t_j)}{48}$;
   - Standardized $Z$-statistic with $\pm 0.5$ continuity correction and two-tailed asymptotic significance via regularized incomplete gamma function $Q(0.5, Z^2 / 2)$;
   - Dynamic determination of test base (negative vs positive ranks) adhering to SPSS footnote convention;
   - **Sign Test:** frequency counts of negative, positive, and zero differences, with exact two-tailed binomial significance calculated via regularized incomplete beta function $I_{0.5}(n - k, k + 1)$;
   - **Never Crash** protection: robust handling of zero variance, identical pairs, and missing values on 100% safe Rust;
   - **Golden Oracle:** Unit and oracle tests in `nonparam_related_tests.rs` validating ranks, $W$, $Z$, and $p$-value against SciPy and SPSS reference tables with $\le 10^{-6}$ precision.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::nonparam_related_cmd`):**
   - Typed engine command `RelatedSamplesTestCommand` (`stat.npar.related2`) with JSON parameter specs for external AI agents;
   - Canonical IBM SPSS Statistics syntax emission:
     ```text
     NPAR TESTS
       /WILCOXON= pre WITH post (PAIRED)
       /SIGN= pre WITH post (PAIRED)
       /MISSING=ANALYSIS.
     ```
3. **User Interface L6 (`statcraft_app::dialogs::nonparam_related_dialog`):**
   - Modal dialog `Two-Related-Samples Tests` ($560 \times 430$ px) built with `DialogKit`:
     * Source variables list (`VariablePicker`) with scale measurement icons;
     * Two-stage slot picker `Current Selections: Variable 1, Variable 2` with adaptive vector transfer buttons;
     * Target pairs list `Test Pairs:`;
     * `Test Type` group with checkboxes `[x] Wilcoxon`, `[ ] Sign`;
     * Standard bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Summary Pivot Tables in Output Viewer (`tables_nonparam_related.rs`):**
   - **Ranks Table (Wilcoxon):** Negative Ranks, Positive Ranks, Ties, Total ($N$, Mean Rank, Sum of Ranks) with difference direction footnotes;
   - **Test Statistics Table (Wilcoxon):** $Z$-score, Asymp. Sig. (2-tailed), method and rank base footnotes;
   - **Frequencies Table (Sign Test):** Negative Differences, Positive Differences, Ties, Total;
   - **Test Statistics Table (Sign Test):** Exact Sig. (2-tailed);
   - TSV clipboard export and Outline Pane integration.
5. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - All 110+ workspace unit and integration tests passing with 0 failures;
   - Differential file size guard verified (`file-guard` $\le 500$ lines for leaf modules);
   - Architectural linters (`design-guard`, `layers`, `parity`) reporting 0 violations;
   - Synchronous dual-catalog localization (`locales/en.ftl`, `locales/ru.ftl`);
   - Strict compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Code formatting `cargo fmt --check` — completely clean.



---

### Phase 7.2: Bivariate Correlations (Pearson $r$, Spearman $\rho$, Kendall $\tau$-b)
*Time: October 28, 2026, 21:30 (~35 minutes)*

Implemented the fundamental bivariate relationship analysis procedure — **Bivariate Correlations** (`Analyze -> Correlate -> Bivariate...`):
1. **Mathematical Engine L2 (`statcraft_algo::correlations` & `correlations_types`):**
   - Pearson linear correlation coefficient calculation ($r = \frac{\sum (x_i - \bar{x})(y_i - \bar{y})}{\sqrt{\sum (x_i - \bar{x})^2 \sum (y_i - \bar{y})^2}}$), cross-product sum of squares ($SS_{xy}$), and unbiased sample covariance ($Cov(x, y)$);
   - Spearman rank correlation ($\rho$) with fractional average ranking for tied observations;
   - Kendall rank correlation ($\tau$-b) with concordant ($P$) and discordant ($Q$) pair computation and analytical tie adjustments $T_x, T_y$;
   - Rigorous $p$-value derivation for two-tailed and one-tailed hypotheses via Student's $t$ survival function (Pearson and Spearman) and asymptotic normal $Z$ statistics (Kendall);
   - Automatic significance flag annotations in matrix cells (`*` at $p < 0.05$, `**` at $p < 0.01$);
   - Missing value handling: pairwise and listwise case exclusions;
   - **Never Crash** protection: safe handling of zero-variance constants, degenerate pairs, and small sample edge cases ($N < 2$) on 100% safe Rust;
   - **Golden Oracle:** Test suite `correlations_tests.rs` verifying correlation matrices and significance values against Python `scipy.stats` (`pearsonr`, `spearmanr`, `kendalltau`) within $\le 10^{-6}$ precision.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::correlations_cmd`):**
   - Typed engine command `CorrelationsCommand` (`stat.correlations`) with JSON parameter specs for external AI agents;
   - Canonical IBM SPSS Statistics syntax emission (`CORRELATIONS /VARIABLES=... /PRINT=TWOTAIL SIG /STATISTICS DESCRIPTIVES XPROD /MISSING=PAIRWISE.` and `NONPAR CORR`).
3. **User Interface L6 (`statcraft_app::dialogs::correlations_dialog`):**
   - Modal dialog `CorrelationsDialog` ($560 \times 430$ px) built with `DialogKit`:
     * Source variables picker (`VariablePicker`) with scale measurement icons;
     * Vector transfer button (`TransferButton`);
     * Target variables list `Variables:`;
     * `Correlation Coefficients` checkboxes (Pearson, Kendall's tau-b, Spearman);
     * `Test of Significance` radio group (Two-tailed / One-tailed) and `Flag significant correlations` checkbox;
     * Right sidebar (`SubdialogSidebar`) with `[ Options... ]` button;
     * Modal subdialog `Options` (Statistics: Means and standard deviations, Cross-product deviations and covariances; Missing Values: Exclude cases pairwise / listwise) with `SubdialogActionBar`;
     * Standard bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Summary Pivot Tables in Output Viewer (`tables_correlations.rs`):**
   - **Descriptive Statistics:** Variable, Mean, Std. Deviation, $N$;
   - **Correlations Matrix Table:** multi-level $K \times K$ grid presenting correlation coefficients with asterisks, exact significance, $N$, optional cross-product deviations, sample covariances, and footnote legends;
   - TSV clipboard export for spreadsheet workflows and Outline Pane integration.
5. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 100 workspace unit and integration tests passing with 0 failures;
   - Architectural linters (`design-guard`, `file-guard`, `layers`, `parity`) reporting 0 violations;
   - Strict compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Code formatting `cargo fmt --check` — completely clean.

---

### Phase 7.3: Two-Independent-Samples Nonparametric Tests (Mann-Whitney $U$ & Wilcoxon $W$)
*Time: October 29, 2026, 07:35 (~35 minutes)*

Implemented the fundamental distribution-free alternative to the independent-samples t-test for comparing two continuous distributions — **Two-Independent-Samples Tests** (`Analyze -> Nonparametric Tests -> Two Independent Samples...`):
1. **Mathematical Engine L2 (`statcraft_algo::nonparam_twosample` & `nonparam_twosample_types`):**
   - Pooled ranking with mid-ranks for tied values across all observations;
   - Sum of ties calculation $\sum (t_i^3 - t_i)$ and tie-corrected variance $\sigma_U^2 = \frac{n_1 n_2}{12} \left( (n + 1) - \frac{\sum (t_i^3 - t_i)}{n(n - 1)} \right)$;
   - Computation of Mann-Whitney $U$, Wilcoxon rank sum $W$, standardized $Z$-score, and asymptotic 2-tailed $p$-value via regularized incomplete gamma function $Q(0.5, Z^2 / 2)$;
   - Support for string or numeric group identifiers (e.g., "1" / "2", "M" / "F");
   - **Never Crash** protection for empty subsets and single-group edge cases on 100% safe Rust;
   - **Golden Oracle:** Unit tests in `nonparam_twosample_tests.rs` verified against Python `scipy.stats.mannwhitneyu` and standard SPSS outputs with precision $\le 10^{-6}$.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::nonparam_twosample_cmd`):**
   - Typed engine command `MannWhitneyCommand` (`stat.npar.mann_whitney`) with JSON parameter specs for external AI agents;
   - Canonical SPSS syntax generator:
     ```text
     NPAR TESTS
       /M-W= salary age BY gender(1 2)
       /MISSING=ANALYSIS.
     ```
3. **User Interface L6 (`statcraft_app::dialogs::nonparam_twosample_dialog` & `nonparam_define_groups`):**
   - Modal dialog `Two-Independent-Samples Tests` ($560 \times 430$ px) built with `DialogKit`:
     * Source variables list `VariablePicker` with measurement scale icons;
     * Adaptive vector transfer buttons `TransferButton`;
     * `Test Variable List:` target list;
     * `Grouping Variable:` display box showing `var(val1 val2)`;
     * Right sidebar `SubdialogSidebar` with `[ Define Groups... ]` button;
     * Modal subdialog `Define Groups` with non-empty distinct value checks;
     * `Test Type` group box with `[x] Mann-Whitney U`, `[ ] Wilcoxon W`, `[ ] Kolmogorov-Smirnov Z` checkboxes;
     * Bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Summary Pivot Tables in Output Viewer (`tables_nonparam.rs`):**
   - **Ranks:** Variable, Group, $N$, Mean Rank, Sum of Ranks, Total row;
   - **Test Statistics:** multi-variable matrix with Mann-Whitney U, Wilcoxon W, Z, Asymp. Sig. (2-tailed), and grouping variable footnote;
   - TSV clipboard export and Outline Pane integration.
5. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 105 workspace unit and integration tests passing;
   - Architectural and design linters (`design-guard`, `file-guard`, `layers`, `parity`) reporting 0 violations;
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Code formatting `cargo fmt --check` — completely clean.

---

### Phase 7.4: Two-Related-Samples Nonparametric Tests (Wilcoxon Signed-Ranks & Sign Test)
*Time: October 29, 2026, 08:15 (~40 minutes)*

Implemented the classical nonparametric toolkit for paired measurements — **Two-Related-Samples Tests** (`Analyze -> Nonparametric Tests -> Related Samples...`):
1. **Mathematical Engine L2 (`statcraft_algo::nonparam_related` & `nonparam_related_types`):**
   - Pairwise difference calculation $D_i = Y_i - X_i$;
   - Classification of zero differences ($D_i = 0$) as ties and their exclusion from ranking per IBM SPSS Statistics canon;
   - Absolute difference ranking $|D_i|$ with mid-ranks for tied values and tie sum computation $\sum (t_j^3 - t_j)$;
   - Calculation of negative ($D_i < 0$) and positive ($D_i > 0$) rank sums and mean ranks;
   - Tie-corrected variance $\sigma_W^2 = \frac{n(n + 1)(2n + 1)}{24} - \frac{\sum (t_j^3 - t_j)}{48}$;
   - Continuous continuity correction $\pm 0.5$, standardized $Z$-score, and asymptotic 2-tailed $p$-value;
   - Methodological rank direction assignment and footnote identification;
   - Implementation of the **Sign Test**: negative, positive, and tie counting, exact two-tailed binomial significance via regularized incomplete beta function $I_{0.5}(n - k, k + 1)$;
   - **Never Crash** resilience for all-tied samples, empty datasets, and zero variances on 100% safe Rust;
   - **Golden Oracle:** Test suite `nonparam_related_tests.rs` verifying results against SciPy and SPSS reference tables with precision $\le 10^{-6}$.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::nonparam_related_cmd`):**
   - Typed engine command `RelatedSamplesTestCommand` (`stat.npar.related2`) with JSON parameter specs for external AI agents;
   - Canonical SPSS syntax generator:
     ```text
     NPAR TESTS
       /WILCOXON= pre WITH post (PAIRED)
       /SIGN= pre WITH post (PAIRED)
       /MISSING=ANALYSIS.
     ```
3. **User Interface L6 (`statcraft_app::dialogs::nonparam_related_dialog`):**
   - Modal dialog `Two-Related-Samples Tests` ($560 \times 430$ px) built with `DialogKit`:
     * Source variables list `VariablePicker` with scale icons;
     * Dual selection panel `Current Selections: Variable 1, Variable 2` with adaptive vector transfer buttons;
     * Formed pairs list `Test Pairs:`;
     * `Test Type` group box with `[x] Wilcoxon`, `[ ] Sign` checkboxes;
     * Bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Summary Pivot Tables in Output Viewer (`tables_nonparam_related.rs`):**
   - **Ranks (Wilcoxon):** Negative Ranks, Positive Ranks, Ties, Total rows ($N$, Mean Rank, Sum of Ranks) with direction footnotes;
   - **Test Statistics (Wilcoxon):** Z-score, Asymp. Sig. (2-tailed), ranking direction footnotes;
   - **Frequencies (Sign Test):** Negative, positive differences, ties, total count;
   - **Test Statistics (Sign Test):** Exact Sig. (2-tailed);
   - TSV clipboard export and Outline Pane integration.
5. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 110+ workspace unit and integration tests passing;
   - Architecture, design, and file-guard linters reporting 0 violations;
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Code formatting `cargo fmt --check` — completely clean.

---

### Phase 7.5: Navigation Panel Overflow Fix & Kruskal-Wallis $H$ Test for $K$ Independent Samples
*Time: October 29, 2026, 08:45 (~35 minutes)*

Completed two major objectives: resolved the navigation title text overflow defect in the Output Viewer Outline Pane and implemented the nonparametric one-way ANOVA counterpart — **Kruskal-Wallis $H$ Test for Several Independent Samples** (`Analyze -> Nonparametric Tests -> K Independent Samples...`):
1. **Outline Navigation Overflow Defect Fix:**
   - Root Cause: low-level `ui.painter().text(...)` calls in `crates/statcraft_app/src/views/output_view/outline.rs` rendered text beyond boundaries without width clipping, and the parent container lacked clipping;
   - Solution:
     1. Increased navigation panel base width `outline_width` from 200.0 to 230.0 px for enhanced readability on high-DPI displays;
     2. Enforced rigid clipping boundary `ui.set_clip_rect(left_rect)` on the tree viewport;
     3. Implemented per-character ellipsis utility `truncate_text_to_width` using `ui.fonts` measurements to append `...` if text exceeds available width;
     4. Enclosed text painting in `ui.painter().with_clip_rect(text_rect)`;
     5. Added `.on_hover_text(...)` tooltips presenting complete untruncated titles on hover.
2. **Mathematical Engine L2 (`statcraft_algo::kruskal_wallis` & `kruskal_wallis_tests`):**
   - Numeric range filtering for grouping variable $[min, max]$ following SPSS canon `BY groupvar(min max)`;
   - Combined ranking across all $N = \sum n_i$ cases with mid-ranks for ties;
   - Group rank sums $R_i$ and mean ranks $\bar{R}_i = R_i / n_i$ across all $k$ groups;
   - Raw Kruskal-Wallis statistic $H = \frac{12}{N(N+1)} \sum \frac{R_i^2}{n_i} - 3(N+1)$;
   - Tie correction factor $C = 1 - \frac{\sum (t_j^3 - t_j)}{N^3 - N}$ and adjusted statistic $H_{adj} = H / C$;
   - Degrees of freedom $df = k - 1$ and asymptotic 2-tailed $p$-value from $\chi^2$ distribution (`chi_square_sf(H_adj, df)`);
   - **Golden Oracle:** Test suite `kruskal_wallis_tests.rs` verified against Python `scipy.stats.kruskal` to $\le 10^{-6}$ precision for both unique and tied distributions;
   - **Never Crash** protection for empty datasets, single groups ($k < 2$), and degenerate cases on 100% safe Rust.
3. **Command Bus & Syntax L5 (`statcraft_syntax::commands::kruskal_wallis_cmd`):**
   - Typed engine command `KruskalWallisCommand` (`stat.npar.kruskal_wallis`) with JSON parameter specs for external AI agents;
   - Canonical SPSS syntax generator:
     ```text
     NPAR TESTS
       /K-W= salary age BY jobcat(1 3)
       /MISSING=ANALYSIS.
     ```
4. **User Interface L6 (`statcraft_app::dialogs::kruskal_wallis_dialog` & `kruskal_define_range`):**
   - Modal dialog `Tests for Several Independent Samples` ($560 \times 430$ px) built with `DialogKit`:
     * Available variables list `VariablePicker` with scale icons;
     * `Test Variable List:` target list and `Grouping Variable:` display box showing range `var(min max)`;
     * Adaptive vector transfer buttons `TransferButton`;
     * Right sidebar `SubdialogSidebar` with `[ Define Range... ]` button;
     * Modal subdialog `Define Range` ($360 \times 300$ px) using `DialogShell::subdialog` with `Minimum:` and `Maximum:` fields;
     * `Test Type` group box with `[x] Kruskal-Wallis H` checkbox;
     * Bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Summary Pivot Tables in Output Viewer (`tables_kruskal.rs`):**
   - **Ranks:** Variable, Group, $N$, Mean Rank, Total row;
   - **Test Statistics:** canonical multi-variable matrix (Kruskal-Wallis H, df, Asymp. Sig.) with footnotes `a. Kruskal Wallis Test` and `b. Grouping Variable: ...`;
   - TSV clipboard export and Outline Pane integration.
6. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 115+ workspace unit and integration tests passing;
   - Architecture, design, and file-guard linters reporting 0 violations;
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Code formatting `cargo fmt --check` — completely clean.


---

### Phase 8.1: Compute Variable (`COMPUTE`)
*Time: October 29, 2026, 09:25 (~35 minutes)*

Implemented the fundamental data transformation mechanism — **Compute Variable** (`COMPUTE`, menu item `Transform -> Compute Variable...`):
1. **Mathematical Engine L2 (`statcraft_algo::expr_eval`):**
   - Architectural decomposition into modular package `expr_eval/` (`types.rs`, `lexer.rs`, `parser.rs`, `eval.rs`, `mod.rs`, `tests.rs`) strictly respecting `file-guard` limits ($\le 500$ lines per file);
   - Robust tokenizer for mathematical expressions: numbers, variable/function identifiers, operators, parentheses, commas, correctly resolving unary `+` and `-`;
   - Dijkstra's Shunting-yard algorithm for infix-to-RPN compilation respecting precedence and right-associativity of power operators (`**`, `^`);
   - Case-by-case row evaluator supporting arithmetic operations (`+`, `-`, `*`, `/`, `**`), single-argument mathematical functions (`ABS`, `EXP`, `LN`, `LOG10`, `RND`, `SQRT`, `TRUNC`), and multi-argument statistical functions (`SUM`, `MEAN`, `MIN`, `MAX`);
   - **Never Crash** resilience: division by zero, negative roots, and missing values safely evaluate to system missing (`NaN` / `SYSMIS`) without halting case processing;
   - 100% unit test coverage for operator precedences, functions, and edge cases.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::compute_cmd`):**
   - Typed engine command `ComputeVariableCommand` (`transform.compute`) with JSON parameter specs for external AI agents;
   - Creating new numeric columns (`DataType::Numeric { decimals: 2 }`, `Measure::Scale`) or overwriting existing variables;
   - Full `Undo` (`Ctrl+Z`) support: automatically deleting created columns or restoring previous values and metadata for overwritten variables;
   - Canonical IBM SPSS Statistics syntax generator:
     ```text
     COMPUTE bmi = weight / ((height / 100) ** 2).
     VARIABLE LABELS bmi "Body Mass Index".
     EXECUTE.
     ```
3. **User Interface L6 (`statcraft_app::dialogs::compute_dialog`):**
   - Canonical calculator dialog $620 \times 480$ px built with `DialogKit` (`DialogShell::compute`, `render_variable_list_panel`, `TransferButton`, `DialogActionBar`);
   - Top section: `Target Variable:` and `Label:` inputs;
   - Left panel: available variables list with measurement scale icons and adaptive `TransferButton`;
   - Center panel: multiline monospace expression editor `Numeric Expression:` and virtual calculator keypad (`+`, `-`, `*`, `/`, `**`, `(`, `)`, `0..9`, `.`, `C`);
   - Right panel: function group selector (*All, Arithmetic, Statistical*) and scrollable function list with syntax snippets;
   - Bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
4. **Integration & Instant Reactivity:**
   - Activated menu item `Transform -> Compute Variable...` with data availability predicate;
   - Upon execution, the target variable immediately reflects in both `Data View` grid and `Variable View` dictionary;
   - Generated syntax is appended to the `Output Viewer` Log.
5. **Quality Control & Definition of Done:**
   - Full automated CI gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 120+ workspace unit and integration tests passing;
   - `file-guard`, `design-guard`, `layers`, and `parity` linters report 0 violations;
   - Full bilingual localization (`locales/en.ftl`, `locales/ru.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings.


---

### Phase 8.2: Recode into Same / Different Variables (`RECODE`)
*Time: October 29, 2026, 10:25 (~35 minutes)*

Implemented the canonical data grouping, discretization, and cleaning tool — **Recode Values** (`RECODE`, menu items `Transform -> Recode into Different Variables...` and `Transform -> Recode into Same Variables...`):
1. **Mathematical Rule Matching Engine L2 (`statcraft_algo::recode`):**
   - Expressive old value specifications `OldValueRule`:
     * Exact numeric value (`Exact(f64)`);
     * System-missing (`SysMissing`);
     * System- or user-missing (`SystemOrUserMissing`);
     * Range from $A$ through $B$ inclusive (`Range(low, high)`);
     * Range from lowest through $A$ (`RangeLowest(high)`);
     * Range from $A$ through highest (`RangeHighest(low)`);
     * All other values (`AllOther` / `ELSE`);
   - Output value specifications `NewValueSpec`:
     * New numeric value (`Value(f64)`);
     * System-missing (`SysMissing`);
     * Copy old value unchanged (`CopyOld`);
   - Sequential rule priority (first matching rule wins), leaving values unchanged if unmatched (IBM SPSS canonical standard);
   - **Never Crash** resilience: safe handling of `NaN`, missing values, and inverted ranges; 100% unit test coverage.
2. **Command Bus L5 (`statcraft_syntax::commands::recode_diff_cmd` & `recode_same_cmd`):**
   - `RecodeIntoDifferentCommand` (`transform.recode.diff`): recoding one or multiple `source -> target` pairs into new or existing columns;
   - `RecodeIntoSameCommand` (`transform.recode.same`): in-place variable recoding;
   - Full `Undo` (`Ctrl+Z`) and `Redo` support: automatic cleanup of newly created columns or restoration of value snapshots and metadata for overwritten columns;
   - Canonical IBM SPSS Statistics syntax generator:
     ```text
     RECODE age (LOWEST THRU 18=1) (19 THRU 35=2) (36 THRU 60=3) (61 THRU HIGHEST=4) (ELSE=SYSMIS) INTO age_group.
     VARIABLE LABELS age_group "Age Group Categories".
     EXECUTE.
     ```
3. **Two-Stage User Interface on DialogKit L6 (`statcraft_app::dialogs`):**
   - Primary dialog `RecodeDialog` ($560 \times 430$ px, `DialogShell::standard`):
     * Dual-mode operation: *Different Variables* and *Same Variables*;
     * Left source variable list `VariablePicker` with scale icons and adaptive `TransferButton`;
     * Central mapping list `Numeric Variable -> Output Variable:`;
     * Output Variable properties box (`Name:`, `Label:`, `Change` button);
     * Modal launcher button `[ Old and New Values... (N) ]`;
     * Bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Modal subdialog `Old and New Values...` (`RecodeOldNewState`):
     * Left `Old Value` group: radio buttons for exact value, system-missing, user-missing, ranges (with `LOWEST` and `HIGHEST`), and `All other values`;
     * Right `New Value` group: radio buttons for value, system-missing, and copying old value;
     * Action buttons `[ Add ]`, `[ Change ]`, `[ Remove ]`;
     * Scrollable mapping list `Old -> New:`;
     * Bottom action bar `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`).
4. **Integration & Reactivity:**
   - Activated menu items `Transform -> Recode into Different Variables...` and `Transform -> Recode into Same Variables...`;
   - Recode results immediately reflect in the `Data View` table grid and `Variable View` dictionary;
   - Syntax logged to `Output Viewer` (Log).
5. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 125+ workspace tests passing with 0 failures;
   - Strict adherence to `file-guard` ($\le 500$ lines per file), `design-guard`, `layers`, and `parity` (31 commands);
   - Synchronous bilingual localization (`locales/en.ftl`, `locales/ru.ftl`);
   - Clean compilation: `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings.

---

### Phase 8.3: Sort Cases Procedure (`SORT CASES`)
*Time: October 29, 2026, 11:15 (~35 minutes)*

Implemented the canonical case ordering procedure in StatCraft — menu action `Data -> Sort Cases...` (`data.sort_cases` command / `SORT CASES BY ... (A|D)` syntax):
1. **Low-Level Row Permutation L0/L1 (`statcraft_io` & `statcraft_core`):**
   - `RawTable::reorder_rows` in `crates/statcraft_io/src/csv_loader.rs` and `DataSet::reorder_cases` in `crates/statcraft_core/src/dataset.rs`;
   - Fast, panic-free rearrangement of all column values according to row permutation index arrays;
   - Strict bounds validation (`IndexOutOfBounds`), preserving `VariableDictionary`, metadata, labels, and missing values intact.
2. **Mathematical Engine L2 (`statcraft_algo::sort_cases`):**
   - `SortKeySpec` specification with `SortDirection` (`Ascending` / `Descending`);
   - Canonical missing value handling adhering to IBM SPSS Statistics standards:
     * System-missing values (`SYSMIS`, `RawValue::Null`, `NaN`) are treated as smallest possible negative values ($-\infty$);
     * In ascending sorts (`Ascending`, `(A)`), missing values appear **first**;
     * In descending sorts (`Descending`, `(D)`), missing values appear **last**;
   - Multi-key stable sorting: ties on primary keys are resolved by subsequent keys while preserving the relative original order of identical cases;
   - `invert_permutation` helper for instantaneous undo calculation;
   - 100% safe Rust without `unwrap()` or `expect()`.
3. **Command Bus L5 (`statcraft_syntax::commands::sort_cases_cmd`):**
   - Strongly-typed `SortCasesCommand` (`data.sort_cases`) with parameter `specs()` for AI agents;
   - Full `Undo` (`Ctrl+Z`) support via an inverse permutation history stack;
   - Canonical SPSS syntax generator:
     ```text
     SORT CASES BY dept (A) salary (D).
     ```
4. **User Interface on DialogKit L6 (`statcraft_app::dialogs`):**
   - Centered `SortCasesDialog` modal ($560 \times 430$ px, `DialogShell::standard`);
   - Left source `VariablePicker` panel with measurement scale icons and `TransferButton` (`->` / `<-`);
   - Target keys list `Sort by:` with directional suffixes (e.g. `dept (A)`, `salary (D)`);
   - Reordering buttons `Move Up` and `Move Down`;
   - `Sort Order` radio group (`(o) Ascending`, `( ) Descending`) updating key directions in real-time;
   - Canonical `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Integration & Reactivity:**
   - Activated menu item `Data -> Sort Cases...`;
   - Instant re-rendering of reordered rows in `Data View`;
   - Syntax logged to `Output Viewer` (Log) and status line notification `StatusMessage::SortCasesExecuted`.
6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 100% workspace tests passing with 0 failures;
   - Strict adherence to `file-guard`, `design-guard`, `layers`, and `parity` (32 commands);
   - Synchronous trilingual localization (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Clean compilation: `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings.

---

### Phase 8.4: Select Cases Procedure (`FILTER BY` / `SELECT IF`)
*Time: October 29, 2026, 12:20 (~50 minutes)*

Implemented the canonical case selection and filtering mechanism in StatCraft — menu action `Data -> Select Cases...` (`data.select_cases` command / `FILTER BY filter_$.` / `FILTER OFF.` / `SELECT IF (...)` syntax):
1. **Filtering Model in DataSet L1 (`statcraft_core::dataset`):**
   - Added `active_filter_var: Option<String>` to `DataSet`;
   - Implemented `set_filter_variable`, `active_filter_var`, `is_row_active`, `is_row_filtered`, and `active_row_indices`;
   - Automatic synchronization on variable removal (`remove_variable`) and renaming (`rename_variable`);
   - Explicit dictionary validation returning typed `CoreError::VariableNotFound`.
2. **Expression Evaluator Extensions L2 (`statcraft_algo::expr_eval`):**
   - Relational comparison operators: `=`, `==`, `<>`, `~=`, `!=`, `<`, `<=`, `>`, `>=`, plus text keywords `EQ`, `NE`, `LT`, `LE`, `GT`, `GE`;
   - Boolean logical operators: `&`, `&&`, `|`, `||`, `~`, `!`, `AND`, `OR`, `NOT`;
   - Row-wise evaluation via RPN yielding $1.0$ (True) or $0.0$ (False), safe missing value handling (`NaN` / `SYSMIS`) without panics.
3. **Command Bus & Undo/Redo L5 (`statcraft_syntax::commands::select_cases_cmd`):**
   - Strongly-typed `SelectCasesCommand` (`data.select_cases`) with parameter `specs()` for AI agents;
   - Supported 3 canonical modes:
     1. *All cases* (`FILTER OFF.`);
     2. *If condition is satisfied* (condition evaluation, auto-generating `filter_$` column with labels `0 'Not Selected'` / `1 'Selected'`, setting `FILTER BY filter_$.`);
     3. *Use filter variable* (`FILTER BY <var>.`);
   - Actions for unselected cases:
     * *Filtered* — temporary exclusion from statistics without data loss;
     * *Deleted* — canonical case deletion (`SELECT IF (<cond>).`);
   - Full `Undo` (`Ctrl+Z`) support restoring filter state, deleted rows, and auto-created columns.
4. **End-to-End Integration Across All Analytical Procedures L2/L5:**
   - All 11 statistical procedures (`descriptives`, `frequencies`, `crosstabs`, `correlations`, `ttest_indep`, `ttest_paired`, `ttest_onesample`, `anova_oneway`, `nonparam_twosample`, `nonparam_related`, `kruskal_wallis`) now iterate rows through `ds.active_row_indices()`;
   - Excluded cases are automatically omitted from computations and reinstated as soon as filters are disabled.
5. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs`):**
   - Centered `SelectCasesDialog` modal ($560 \times 430$ px, `DialogShell::standard`);
   - Left source `VariablePicker` panel with measurement scale icons and adaptive `TransferButton`;
   - `Select` radio group with `[ If... ]` condition button;
   - Modal subdialog `Select Cases: If Condition` ($360 \times 300$ px, `DialogShell::subdialog`) featuring a calculator & operator keypad (`+ - * / ** ( ) = ~= < <= > >= & | ~`);
   - `Unselected Cases Are` radio group (`(o) Filtered`, `( ) Deleted`);
   - Bottom `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
6. **Data View & Status Bar Visualization:**
   - Row numbers of excluded cases in `Data View` are rendered with a diagonal vector slash (`ui.painter().line_segment`), perfectly matching IBM SPSS Statistics behavior;
   - Bottom status bar displays active filter indicator: `Filter: ON (var_name)`;
   - Activated main menu item `Data -> Select Cases...`.
7. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 100% workspace tests passing with 0 failures;
   - Strict adherence to `file-guard`, `design-guard`, `layers`, and `parity` (33 commands);
   - Synchronous trilingual localization (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Clean compilation: `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 9.1: Multiple Linear Regression (`REGRESSION`)
*Time: October 29, 2026, 16:30 (~45 minutes)*

Implemented one of the foundational predictive modeling frameworks in IBM SPSS Statistics — **Multiple Linear Regression** (`Analyze -> Regression -> Linear...`, engine command `stat.regression.linear` / syntax `REGRESSION /DEPENDENT=... /METHOD=ENTER ...`):

1. **Mathematical Core L2 (`statcraft_algo::regression` & `regression_types`):**
   - Ordinary Least Squares (OLS) and Weighted Least Squares (WLS) natively incorporating frequency weights (`row_weight`) and active filters (`active_row_indices`);
   - Numerically stable normal equations matrix solver $(X^T W X)^{-1}$ utilizing Gauss-Jordan elimination with partial pivoting (`invert_matrix`) in 100% safe Rust without external C dependencies;
   - Parameter vector estimation $\mathbf{B} = (X^T W X)^{-1} X^T W \mathbf{Y}$ and standard errors $SE(B_j) = s_e \sqrt{c_{jj}}$;
   - Standardized beta coefficients $\beta_j = B_j \frac{s_{X_j}}{s_Y}$;
   - Student's $t$-statistic $t_j = B_j / SE(B_j)$ and two-tailed significance levels via the regularized incomplete beta function;
   - Configurable confidence intervals for coefficients $B$ with customizable confidence level (default: 95%);
   - Complete Analysis of Variance (ANOVA) table: sums of squares $SS_{reg}, SS_{resid}, SS_{total}$, degrees of freedom $df_{reg} = p, df_{resid} = N - p - 1, df_{total} = N - 1$, mean squares $MS$, $F$-test statistic, and asymptotic $p$-value;
   - Model Summary: $R = \sqrt{R^2}$, $R^2 = SS_{reg} / SS_{total}$, Adjusted $R^2_{adj} = 1 - \frac{MS_{resid}}{MS_{total}}$, standard error of the estimate $s_e = \sqrt{MS_{resid}}$;
   - Collinearity diagnostics: Tolerance ($Tolerance_j = 1 / r^{jj}$) and Variance Inflation Factor ($VIF_j = 1 / Tolerance_j$) computed from the inverted predictor correlation matrix $R_{xx}^{-1}$;
   - **Golden Oracle:** Test suite `regression_tests.rs` verifying regression coefficients, $t$-statistics, $R^2$, ANOVA $F$, Tolerance, and VIF against Python `scipy.stats` and `numpy.linalg.lstsq` benchmarks with $\le 10^{-6}$ precision;
   - **Never Crash** protection: graceful handling of singular matrices, multicollinearity, small samples ($N \le p + 1$), zero predictor variance, and missing values in 100% safe Rust with zero panics.

2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::regression_linear_cmd`):**
   - Strongly-typed `LinearRegressionCommand` (`stat.regression.linear`) with parameter `specs()` for external AI agents;
   - Listwise missing value deletion across all specified dependent and independent variables;
   - Seamless synergy with active sample filtering (`Select Cases`) and frequency weighting (`Weight Cases`);
   - Canonical IBM SPSS Statistics syntax generator:
     ```text
     REGRESSION
       /DEPENDENT=salary
       /METHOD=ENTER educ salbegin
       /STATISTICS=COEFF CI(95) R ANOVA COLLIN.
     ```
   - Full `Undo` / `Redo` lifecycle support.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::regression_dialog`):**
   - Centered canonical modal dialog ($560 \times 430$ px, `DialogShell::standard`);
   - `VariablePicker` panel with measurement scale icons and vector transfer buttons `TransferButton` (`->` / `<-`);
   - Target slots: `Dependent:` (single response variable) and `Independent(s):` (predictor variables list);
   - Method selector dropdown: `Method: Enter`;
   - Right sidebar `SubdialogSidebar` with `[ Statistics... ]` button;
   - Modal statistics subdialog ($360 \times 300$ px, `DialogShell::subdialog`):
     * Checkboxes `[x] Estimates`, `[x] Confidence intervals` (with configurable 95% level), `[x] Model fit`, `[x] Collinearity diagnostics`;
     * Bottom action row `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Summary Pivot Tables in Output Viewer (`tables_regression.rs`):**
   - **Variables Entered/Removed:** Model, Variables Entered, Variables Removed, Method;
   - **Model Summary:** Model, $R$, $R$ Square, Adjusted $R$ Square, Std. Error of the Estimate;
   - **ANOVA:** Model, Regression / Residual / Total ($SS, df, MS, F$, Sig.);
   - **Coefficients:** Model, Unstandardized Coefficients ($B, Std. Error$), Standardized Coefficients ($Beta$), $t$, Sig., 95.0% Confidence Interval for $B$ (Lower / Upper Bound), Collinearity Statistics ($Tolerance, VIF$);
   - TSV clipboard export and Outline Pane integration with procedural icons.

5. **Reactivity, Menu & Status Bar:**
   - Activated main menu item `Analyze -> Regression -> Linear...`;
   - Canonical syntax automatically recorded to `Output Viewer` (Log);
   - Status bar feedback notification (`Linear regression executed: <var>`).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 150+ workspace unit and integration tests passing with 0 failures;
   - Strict adherence to file size limits (`file-guard` $\le 500$ lines on leaf modules, test fixtures modularized in `output_viewer_invariants/fixtures.rs`);
   - Architectural linters (`design-guard`, `layers`, `parity` — 36 documented procedures) reporting 0 violations;
   - Synchronous trilingual localization across all supported languages (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 8.5: Split File Procedure (`SPLIT FILE`)
*Time: October 29, 2026, 13:10 (~45 minutes)*

Implemented the canonical segmented data analysis workflow in StatCraft — menu action `Data -> Split File...` (`data.split_file` command / `SPLIT FILE BY ...` / `SPLIT FILE OFF.` syntax):
1. **Subgroup Model in DataSet L1 (`statcraft_core::split` & `dataset`):**
   - Added core types: `SplitFileMode` (*Off*, *CompareGroups*, *OrganizeOutput*), `SplitFileConfig`, `SplitKeyValue`, and `SplitSubgroup`;
   - Embedded `split_file: Option<SplitFileConfig>` into `DataSet` with accessors `set_split_file()`, `split_file()`, and `split_groups()`;
   - Automatic metadata synchronization when grouping variables are renamed (`rename_variable`) or deleted (`remove_variable`), gracefully reverting to unsplit mode if all grouping keys are removed.
2. **Segmentation Algorithm L2 (`statcraft_algo::split_groups`):**
   - `extract_split_groups` function extracting subgroup partitions across arbitrary combinations of grouping variables (composite keys);
   - Canonical handling of missing values adhering to IBM SPSS Statistics standards (`cat = .`);
   - Seamless integration with `Select Cases`: when an active filter is present, subgroups are computed exclusively over active rows (`active_row_indices`), preventing analytical skew;
   - 100% safe Rust without panics.
3. **Command Bus & Undo/Redo L5 (`statcraft_syntax::commands::split_file_cmd`):**
   - Strongly-typed `SplitFileCommand` (`data.split_file`) with parameter `specs()` for AI agents;
   - Automated pre-sorting of cases by grouping keys (`sort: bool`);
   - Canonical SPSS syntax generator:
     ```text
     SORT CASES BY gender (A) dept (A).
     SPLIT FILE LAYERED BY gender dept.
     ```
     or `SPLIT FILE OFF.` when clearing segmentation;
   - Full `Undo` (`Ctrl+Z`) support restoring both prior split configuration and original case ordering upon undoing auto-sorted splits.
4. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs`):**
   - Centered `SplitFileDialog` modal ($560 \times 430$ px, `DialogShell::standard`);
   - Analysis mode selector:
     1. *(o) Analyze all cases, do not create groups* (`SPLIT FILE OFF.`);
     2. *( ) Compare groups* (`SPLIT FILE LAYERED BY ...`);
     3. *( ) Organize output by groups* (`SPLIT FILE SEPARATE BY ...`);
   - `Groups Based on:` variable selector with `VariablePicker` and adaptive `TransferButton` (`->` / `<-`);
   - Ordering options: `(o) Sort the file by grouping variables` / `( ) File is already sorted`;
   - Standard bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **Architectural Decomposition & File Size Limit Guard:**
   - To uphold the strict `file-guard` $\le 1000$ line threshold in UI coordinators, decoupled `crates/statcraft_app/src/app_dialogs.rs` by extracting data manipulation dialogs into `app_dialogs_data.rs` (455 lines).
6. **Integration, Status Bar & Reactivity:**
   - Status bar displays vector indicator: `Split File: ON (var_name)`;
   - Activated main menu item `Data -> Split File...`;
   - Canonical syntax logged to `Output Viewer` (Log).
7. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 100% workspace tests passing with 0 failures;
   - Strict adherence to `file-guard`, `design-guard`, `layers`, and `parity` (34 commands);
   - Synchronous trilingual localization (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Clean compilation: `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 8.6: Weight Cases Procedure (`WEIGHT BY` / `WEIGHT OFF`)
*Time: October 29, 2026, 14:05 (~40 minutes)*

Implemented one of the most critical analytical mechanisms of IBM SPSS Statistics for sociology, polling, marketing research, and aggregated frequency data — the **Weight Cases** procedure (menu item `Data -> Weight Cases...`, engine command `data.weight_cases` / syntax `WEIGHT BY var.` / `WEIGHT OFF.`):

1. **Weighting Model in DataSet L1 (`statcraft_core::dataset`):**
   - Added `weight_var: Option<String>` to `DataSet` accompanied by `set_weight_variable()`, `weight_var()`, `is_weighted()`, and `row_weight(row_idx)`;
   - Enforced strict canonical SPSS rules: case weights must be non-negative ($w_i \ge 0$), while cases with zero ($w_i = 0$), negative ($w_i < 0$), or missing values (`NaN` / `SYSMIS`) are systematically excluded from computations;
   - Seamless synergy with `Select Cases`: when row filtering is active, weights are applied exclusively to active observations (`active_row_indices`), preventing statistical bias;
   - Metadata synchronization: variable deletion (`remove_variable`) or renaming (`rename_variable`) safely updates or clears the weight state with zero panics.

2. **Mathematical Core L2 (`statcraft_algo`):**
   - **Frequencies (`frequencies.rs`):** weighted category frequencies ($f_w = \sum w_i$), weighted valid and cumulative percentages, and weighted percentile/quartile distribution analysis;
   - **Descriptive Statistics (`descriptives.rs`):** `compute_descriptives_weighted` routine computing weighted mean $\bar{x}_w = \frac{\sum w_i x_i}{\sum w_i}$, weighted sample variance $s_w^2 = \frac{\sum w_i (x_i - \bar{x}_w)^2}{W - 1}$ (where $W = \sum w_i$), standard error of the mean, weighted skewness, and weighted kurtosis;
   - **Crosstabs (`crosstabs.rs`):** weighted $R \times C$ contingency tables, marginal totals, and Pearson's Chi-Square test of independence incorporating frequency weights;
   - 100% safe Rust, zero runtime overhead when weighting is disabled.

3. **Command Bus & Syntax L5 (`statcraft_syntax::commands::weight_cases_cmd`):**
   - Strongly-typed `WeightCasesCommand` (`data.weight_cases`) with parameter `specs()` for AI agents;
   - Canonical SPSS syntax generator:
     ```text
     WEIGHT BY count.
     ```
     or `WEIGHT OFF.` upon deactivation;
   - Full `Undo` / `Redo` (`Ctrl+Z` / `Ctrl+Y`) lifecycle restoring previous weighting state.

4. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs`):**
   - Centered `WeightCasesDialog` modal ($560 \times 430$ px, `DialogShell::standard`);
   - Left `VariablePicker` panel displaying measurement scale icons and vector `TransferButton` (`->` / `<-`);
   - Mode selection radio group:
     1. *(o) Do not weight cases* (`WEIGHT OFF.`);
     2. *( ) Weight cases by:* with frequency variable target field;
   - Informative sample status display (`Current Status: Weight Off` / `Weight on by <var>`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

5. **Status Bar, Menu & Reactivity:**
   - Dedicated vector status indicator: `Weight: ON (var)`;
   - Activated main menu item `Data -> Weight Cases...`;
   - Canonical syntax logged to `Output Viewer` (Log).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 149 Rust files verified in `file-guard`, `design-guard`, `layers`, and `parity` — 35 commands);
   - 100% workspace tests passing with 0 failures;
   - Synchronous trilingual localization (`locales/ru.ftl`, `locales/en.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 11.1: Factor Analysis / Principal Component Analysis (Factor Analysis / PCA: `FACTOR`)
*Time: October 29, 2026, 18:25 (~35 minutes)*

Implemented the fundamental dimension reduction procedure of IBM SPSS Statistics — **Factor Analysis / Principal Component Analysis** (`Analyze -> Dimension Reduction -> Factor...`), complementing Reliability Analysis (Cronbach's Alpha) for comprehensive psychometric and multivariate validation:

1. **Mathematical Engine L2 (`statcraft_algo::factor`, `factor_types`, `factor_tests`):**
   - Listwise missing case deletion and observation weighting (`row_weight`) support;
   - Computation of pairwise Pearson correlations and weighted correlation matrix $R$ ($p \times p$);
   - Factorability and sampling adequacy diagnostics:
     * **Kaiser-Meyer-Olkin (KMO) Measure of Sampling Adequacy**: anti-image partial correlation calculation $a_{ij} = -s_{ij} / \sqrt{s_{ii} s_{jj}}$ via numerical inversion of $S = R^{-1}$ and $\text{KMO} = \frac{\sum_{i \ne j} r_{ij}^2}{\sum_{i \ne j} r_{ij}^2 + \sum_{i \ne j} a_{ij}^2}$;
     * **Bartlett's Test of Sphericity**: $\chi^2 = -(n - 1 - (2p + 5)/6) \ln |R|$ at $df = p(p - 1)/2$ with exact $p$-value from Chi-square survival function;
   - Principal Component Analysis (**PCA**) Extraction:
     * Spectral decomposition of real symmetric matrix $R$ via cyclic Jacobi rotations on 100% safe Rust without external linear algebra dependencies (`jacobi_eigen_symmetric`);
     * Kaiser criterion ($\lambda \ge 1.0$) or fixed factor count extraction;
     * Deterministic sign convention for positive dominant loadings;
     * Factor loadings $l_{ik} = \sqrt{\lambda_k} \cdot v_{ik}$;
     * Communalities (*Communalities*): initial 1.000 for PCA, extraction $h_i^2 = \sum_{k=1}^m l_{ik}^2$;
     * Total Variance Explained (*Total Variance Explained*): initial eigenvalues, % of variance, cumulative %, and extraction sums of squared loadings;
     * Component Matrix (*Component Matrix*);
   - **Golden Oracle:** `factor_tests.rs` unit tests verified against Python `scipy.linalg.eigh` and `factor_analyzer` with precision $\le 10^{-6}$;
   - **Never Crash** protection against singular matrices, $n < 2$, zero-variance variables, and missing cases.

2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::factor_cmd`):**
   - Typed engine command `FactorCommand` (`stat.factor`) with JSON parameter specs for external AI agents;
   - Canonical IBM SPSS Statistics syntax generator and parser:
     ```text
     FACTOR
       /VARIABLES q1 q2 q3 q4 q5
       /MISSING LISTWISE
       /ANALYSIS q1 q2 q3 q4 q5
       /PRINT INITIAL EXTRACTION KMO
       /CRITERIA MINEIGEN(1)
       /EXTRACTION PC
       /ROTATION NOROTATE.
     ```
   - Full `Undo` / `Redo` lifecycle support.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::factor_dialog`):**
   - Centered canonical modal dialog ($560 \times 430$ px, `DialogShell::standard`);
   - Two-panel `VariablePicker` with measurement scale icons and vector transfer buttons `TransferButton` (`->` / `<-`);
   - `Variables:` target list (requires at least 2 items to enable `[ OK ]`);
   - Right sidebar `SubdialogSidebar` with modal configuration triggers:
     * `[ Descriptives... ]`: subdialog with `[x] KMO and Bartlett's test of sphericity`;
     * `[ Extraction... ]`: subdialog with PCA method and factor retention criteria (Eigenvalues over 1.0 / Fixed number of factors);
     * Action row `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Summary Pivot Tables in Output Viewer (`tables_factor.rs`):**
   - **KMO and Bartlett's Test:** Kaiser-Meyer-Olkin Measure of Sampling Adequacy, Bartlett's Test of Sphericity (Approx. Chi-Square, df, Sig.);
   - **Communalities:** Variable, Initial (1.000), Extraction;
   - **Total Variance Explained:** Component, Initial Eigenvalues (Total, % of Variance, Cumulative %), Extraction Sums of Squared Loadings;
   - **Component Matrix:** variables $\times$ components;
   - TSV clipboard export and Outline Tree integration.

5. **Reactivity, Menu & Status Bar:**
   - Activated main menu item `Analyze -> Dimension Reduction -> Factor...`;
   - Canonical syntax automatically recorded to Output Viewer (Log);
   - Status bar feedback notification (`Factor analysis computed for <n> case(s), <m> component(s) extracted`).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - Strict adherence to file size limits (`file-guard` $\le 500$ lines on leaf modules);
   - Strict adherence to DialogKit standards (`design-guard` — 100% compliant);
   - Architectural and parity linters (`layers`, `parity` — 38 documented procedures) reporting 0 violations;
   - Synchronous localization without string hardcoding (`locales/en.ftl`, `locales/ru.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 9.1: Multiple Linear Regression (`REGRESSION`)
*Time: October 29, 2026, 16:30 (~45 minutes)*

Implemented one of the foundational predictive modeling frameworks in IBM SPSS Statistics — **Multiple Linear Regression** (`Analyze -> Regression -> Linear...`, engine command `stat.regression.linear` / syntax `REGRESSION /DEPENDENT=... /METHOD=ENTER ...`):

1. **Mathematical Core L2 (`statcraft_algo::regression` & `regression_types`):**
   - Ordinary Least Squares (OLS) and Weighted Least Squares (WLS) natively incorporating frequency weights (`row_weight`) and active filters (`active_row_indices`);
   - Numerically stable normal equations matrix solver $(X^T W X)^{-1}$ utilizing Gauss-Jordan elimination with partial pivoting (`invert_matrix`) in 100% safe Rust without external C dependencies;
   - Parameter vector estimation $\mathbf{B} = (X^T W X)^{-1} X^T W \mathbf{Y}$ and standard errors $SE(B_j) = s_e \sqrt{c_{jj}}$;
   - Standardized beta coefficients $\beta_j = B_j \frac{s_{X_j}}{s_Y}$;
   - Student's $t$-statistic $t_j = B_j / SE(B_j)$ and two-tailed significance levels via the regularized incomplete beta function;
   - Configurable confidence intervals for coefficients $B$ with customizable confidence level (default: 95%);
   - Complete Analysis of Variance (ANOVA) table: sums of squares $SS_{reg}, SS_{resid}, SS_{total}$, degrees of freedom $df_{reg} = p, df_{resid} = N - p - 1, df_{total} = N - 1$, mean squares $MS$, $F$-test statistic, and asymptotic $p$-value;
   - Model Summary: $R = \sqrt{R^2}$, $R^2 = SS_{reg} / SS_{total}$, Adjusted $R^2_{adj} = 1 - \frac{MS_{resid}}{MS_{total}}$, standard error of the estimate $s_e = \sqrt{MS_{resid}}$;
   - Collinearity diagnostics: Tolerance ($Tolerance_j = 1 / r^{jj}$) and Variance Inflation Factor ($VIF_j = 1 / Tolerance_j$) computed from the inverted predictor correlation matrix $R_{xx}^{-1}$;
   - **Golden Oracle:** Test suite `regression_tests.rs` verifying regression coefficients, $t$-statistics, $R^2$, ANOVA $F$, Tolerance, and VIF against Python `scipy.stats` and `numpy.linalg.lstsq` benchmarks with $\le 10^{-6}$ precision;
   - **Never Crash** protection: graceful handling of singular matrices, multicollinearity, small samples ($N \le p + 1$), zero predictor variance, and missing values in 100% safe Rust with zero panics.

2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::regression_linear_cmd`):**
   - Strongly-typed `LinearRegressionCommand` (`stat.regression.linear`) with parameter `specs()` for external AI agents;
   - Listwise missing value deletion across all specified dependent and independent variables;
   - Seamless synergy with active sample filtering (`Select Cases`) and frequency weighting (`Weight Cases`);
   - Canonical IBM SPSS Statistics syntax generator:
     ```text
     REGRESSION
       /DEPENDENT=salary
       /METHOD=ENTER educ salbegin
       /STATISTICS=COEFF CI(95) R ANOVA COLLIN.
     ```
   - Full `Undo` / `Redo` lifecycle support.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::regression_dialog`):**
   - Centered canonical modal dialog ($560 \times 430$ px, `DialogShell::standard`);
   - `VariablePicker` panel with measurement scale icons and vector transfer buttons `TransferButton` (`->` / `<-`);
   - Target slots: `Dependent:` (single response variable) and `Independent(s):` (predictor variables list);
   - Method selector dropdown: `Method: Enter`;
   - Right sidebar `SubdialogSidebar` with `[ Statistics... ]` button;
   - Modal statistics subdialog ($360 \times 300$ px, `DialogShell::subdialog`):
     * Checkboxes `[x] Estimates`, `[x] Confidence intervals` (with configurable 95% level), `[x] Model fit`, `[x] Collinearity diagnostics`;
     * Bottom action row `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Summary Pivot Tables in Output Viewer (`tables_regression.rs`):**
   - **Variables Entered/Removed:** Model, Variables Entered, Variables Removed, Method;
   - **Model Summary:** Model, $R$, $R$ Square, Adjusted $R$ Square, Std. Error of the Estimate;
   - **ANOVA:** Model, Regression / Residual / Total ($SS, df, MS, F$, Sig.);
   - **Coefficients:** Model, Unstandardized Coefficients ($B, Std. Error$), Standardized Coefficients ($Beta$), $t$, Sig., 95.0% Confidence Interval for $B$ (Lower / Upper Bound), Collinearity Statistics ($Tolerance, VIF$);
   - TSV clipboard export and Outline Pane integration with procedural icons.

5. **Reactivity, Menu & Status Bar:**
   - Activated main menu item `Analyze -> Regression -> Linear...`;
   - Canonical syntax automatically recorded to `Output Viewer` (Log);
   - Status bar feedback notification (`Linear regression executed: <var>`).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 150+ workspace unit and integration tests passing with 0 failures;
   - Strict adherence to file size limits (`file-guard` $\le 500$ lines on leaf modules, test fixtures modularized in `output_viewer_invariants/fixtures.rs`);
   - Architectural linters (`design-guard`, `layers`, `parity` — 36 documented procedures) reporting 0 violations;
   - Synchronous trilingual localization across all supported languages (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 11.1: Factor Analysis / Principal Component Analysis (Factor Analysis / PCA: `FACTOR`)
*Time: October 29, 2026, 18:25 (~35 minutes)*

Implemented the fundamental dimension reduction procedure of IBM SPSS Statistics — **Factor Analysis / Principal Component Analysis** (`Analyze -> Dimension Reduction -> Factor...`), complementing Reliability Analysis (Cronbach's Alpha) for comprehensive psychometric and multivariate validation:

1. **Mathematical Engine L2 (`statcraft_algo::factor`, `factor_types`, `factor_tests`):**
   - Listwise missing case deletion and observation weighting (`row_weight`) support;
   - Computation of pairwise Pearson correlations and weighted correlation matrix $R$ ($p \times p$);
   - Factorability and sampling adequacy diagnostics:
     * **Kaiser-Meyer-Olkin (KMO) Measure of Sampling Adequacy**: anti-image partial correlation calculation $a_{ij} = -s_{ij} / \sqrt{s_{ii} s_{jj}}$ via numerical inversion of $S = R^{-1}$ and $\text{KMO} = \frac{\sum_{i \ne j} r_{ij}^2}{\sum_{i \ne j} r_{ij}^2 + \sum_{i \ne j} a_{ij}^2}$;
     * **Bartlett's Test of Sphericity**: $\chi^2 = -(n - 1 - (2p + 5)/6) \ln |R|$ at $df = p(p - 1)/2$ with exact $p$-value from Chi-square survival function;
   - Principal Component Analysis (**PCA**) Extraction:
     * Spectral decomposition of real symmetric matrix $R$ via cyclic Jacobi rotations on 100% safe Rust without external linear algebra dependencies (`jacobi_eigen_symmetric`);
     * Kaiser criterion ($\lambda \ge 1.0$) or fixed factor count extraction;
     * Deterministic sign convention for positive dominant loadings;
     * Factor loadings $l_{ik} = \sqrt{\lambda_k} \cdot v_{ik}$;
     * Communalities (*Communalities*): initial 1.000 for PCA, extraction $h_i^2 = \sum_{k=1}^m l_{ik}^2$;
     * Total Variance Explained (*Total Variance Explained*): initial eigenvalues, % of variance, cumulative %, and extraction sums of squared loadings;
     * Component Matrix (*Component Matrix*);
   - **Golden Oracle:** `factor_tests.rs` unit tests verified against Python `scipy.linalg.eigh` and `factor_analyzer` with precision $\le 10^{-6}$;
   - **Never Crash** protection against singular matrices, $n < 2$, zero-variance variables, and missing cases.

2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::factor_cmd`):**
   - Typed engine command `FactorCommand` (`stat.factor`) with JSON parameter specs for external AI agents;
   - Canonical IBM SPSS Statistics syntax generator and parser:
     ```text
     FACTOR
       /VARIABLES q1 q2 q3 q4 q5
       /MISSING LISTWISE
       /ANALYSIS q1 q2 q3 q4 q5
       /PRINT INITIAL EXTRACTION KMO
       /CRITERIA MINEIGEN(1)
       /EXTRACTION PC
       /ROTATION NOROTATE.
     ```
   - Full `Undo` / `Redo` lifecycle support.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::factor_dialog`):**
   - Centered canonical modal dialog ($560 \times 430$ px, `DialogShell::standard`);
   - Two-panel `VariablePicker` with measurement scale icons and vector transfer buttons `TransferButton` (`->` / `<-`);
   - `Variables:` target list (requires at least 2 items to enable `[ OK ]`);
   - Right sidebar `SubdialogSidebar` with modal configuration triggers:
     * `[ Descriptives... ]`: subdialog with `[x] KMO and Bartlett's test of sphericity`;
     * `[ Extraction... ]`: subdialog with PCA method and factor retention criteria (Eigenvalues over 1.0 / Fixed number of factors);
     * Action row `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Summary Pivot Tables in Output Viewer (`tables_factor.rs`):**
   - **KMO and Bartlett's Test:** Kaiser-Meyer-Olkin Measure of Sampling Adequacy, Bartlett's Test of Sphericity (Approx. Chi-Square, df, Sig.);
   - **Communalities:** Variable, Initial (1.000), Extraction;
   - **Total Variance Explained:** Component, Initial Eigenvalues (Total, % of Variance, Cumulative %), Extraction Sums of Squared Loadings;
   - **Component Matrix:** variables $\times$ components;
   - TSV clipboard export and Outline Tree integration.

5. **Reactivity, Menu & Status Bar:**
   - Activated main menu item `Analyze -> Dimension Reduction -> Factor...`;
   - Canonical syntax automatically recorded to Output Viewer (Log);
   - Status bar feedback notification (`Factor analysis computed for <n> case(s), <m> component(s) extracted`).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - Strict adherence to file size limits (`file-guard` $\le 500$ lines on leaf modules);
   - Strict adherence to DialogKit standards (`design-guard` — 100% compliant);
   - Architectural and parity linters (`layers`, `parity` — 38 documented procedures) reporting 0 violations;
   - Synchronous localization without string hardcoding (`locales/en.ftl`, `locales/ru.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.






---

### Phase 10.1: Scale Reliability Analysis (Reliability Analysis: `RELIABILITY`)
*Time: October 29, 2026, 17:35 (~40 minutes)*

Implemented the fundamental psychometric, survey, and applied statistical measurement procedure — **Reliability Analysis** (`Analyze -> Scale -> Reliability Analysis...`, engine command `stat.scale.reliability` / syntax `RELIABILITY /VARIABLES=... /SCALE('ALL VARIABLES') ALL /MODEL=ALPHA`):

1. **Mathematical Engine L2 (`statcraft_algo::reliability` & `reliability_types`):**
   - Internal consistency estimation via Cronbach's Alpha: $\alpha = \frac{k}{k - 1} \left( 1 - \frac{\sum s_i^2}{s_{total}^2} \right)$;
   - Standardized Alpha based on average inter-item correlation $\bar{r}$: $\alpha_{std} = \frac{k \bar{r}}{1 + (k - 1) \bar{r}}$;
   - Listwise missing data deletion across all analyzed scale items;
   - Case Processing Summary: Valid ($N, \%$), Excluded ($N, \%$), and Total ($N, 100.0\%$);
   - Item Statistics: Mean ($\bar{x}_i$), Standard Deviation ($s_i$), Valid Sample Count ($N$);
   - Item-Total Statistics:
     * Scale Mean if Item Deleted: $\mu_{-i} = \mu_{total} - \bar{x}_i$;
     * Scale Variance if Item Deleted: $s_{-i}^2 = s_{total}^2 + s_i^2 - 2 \text{Cov}(X_i, X_{total})$;
     * Corrected Item-Total Correlation: $r_{i(total - i)} = \frac{\text{Cov}(X_i, X_{total}) - s_i^2}{\sqrt{s_i^2 \cdot s_{-i}^2}}$;
     * Cronbach's Alpha if Item Deleted: $\alpha_{-i} = \frac{k - 1}{k - 2} \left( 1 - \frac{\sum_{j \neq i} s_j^2}{s_{-i}^2} \right)$ for $k > 2$;
   - Summary Scale Statistics: Mean, Variance, Standard Deviation, Item Count;
   - Seamless synergy with active sample filtering (`Select Cases`) and frequency weighting (`Weight Cases`);
   - **Golden Oracle:** Unit and validation tests in `reliability_tests.rs` with analytical precision $\le 10^{-6}$;
   - **Never Crash** protection: safe handling of $k < 2$, zero-variance degenerate scales, identical survey responses, and missing values on 100% safe Rust.

2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::reliability_cmd`):**
   - Strongly-typed `ReliabilityCommand` (`stat.scale.reliability`) with parameter `specs()` for external AI agents;
   - Canonical IBM SPSS Statistics syntax generator:
     ```text
     RELIABILITY
       /VARIABLES=q1 q2 q3 q4
       /SCALE('ALL VARIABLES') ALL
       /MODEL=ALPHA
       /STATISTICS=DESCRIPTIVES SCALE
       /SUMMARY=TOTAL.
     ```
   - Full `Undo` / `Redo` lifecycle support.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::reliability_dialog`):**
   - Centered canonical modal dialog ($560 \times 430$ px, `DialogShell::standard`);
   - Two-panel `VariablePicker` with measurement scale icons and vector transfer buttons `TransferButton` (`->` / `<-`);
   - `Items:` target list (requires at least 2 items to enable `[ OK ]`);
   - Model selector dropdown: `Model: Alpha`;
   - Text input for scale identifier: `Scale label:` (default `ALL VARIABLES`);
   - Right sidebar `SubdialogSidebar` with `[ Statistics... ]` button;
   - Modal statistics subdialog ($360 \times 300$ px, `DialogShell::subdialog`):
     * Checkboxes `[x] Item`, `[x] Scale`, `[x] Scale if item deleted`;
     * Bottom action row `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Summary Pivot Tables in Output Viewer (`tables_reliability.rs`):**
   - **Case Processing Summary:** Cases (Valid, Excluded, Total) with $N$ and $\%$, listwise note footnote;
   - **Reliability Statistics:** Cronbach's Alpha, Cronbach's Alpha Based on Standardized Items, N of Items;
   - **Item Statistics:** Item, Mean, Std. Deviation, N;
   - **Item-Total Statistics:** Item, Scale Mean if Item Deleted, Scale Variance if Item Deleted, Corrected Item-Total Correlation, Cronbach's Alpha if Item Deleted;
   - **Scale Statistics:** Mean, Variance, Std. Deviation, N of Items;
   - TSV clipboard export and Outline Tree integration.

5. **Reactivity, Menu & Status Bar:**
   - Activated main menu item `Analyze -> Scale -> Reliability Analysis...`;
   - Canonical syntax automatically recorded to `Output Viewer` (Log);
   - Status bar feedback notification (`Reliability analysis computed for <scale> (<n> items)`).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 160+ workspace unit and integration tests passing with 0 failures;
   - Strict adherence to file size limits (`file-guard` $\le 500$ lines on leaf modules);
   - Strict adherence to DialogKit standards (`design-guard` — 100% compliant);
   - Architectural and parity linters (`layers`, `parity` — 37 documented procedures) reporting 0 violations;
   - Synchronous trilingual localization across all supported languages (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 11.1: Factor Analysis / Principal Component Analysis (Factor Analysis / PCA: `FACTOR`)
*Time: October 29, 2026, 18:25 (~35 minutes)*

Implemented the fundamental dimension reduction procedure of IBM SPSS Statistics — **Factor Analysis / Principal Component Analysis** (`Analyze -> Dimension Reduction -> Factor...`), complementing Reliability Analysis (Cronbach's Alpha) for comprehensive psychometric and multivariate validation:

1. **Mathematical Engine L2 (`statcraft_algo::factor`, `factor_types`, `factor_tests`):**
   - Listwise missing case deletion and observation weighting (`row_weight`) support;
   - Computation of pairwise Pearson correlations and weighted correlation matrix $R$ ($p \times p$);
   - Factorability and sampling adequacy diagnostics:
     * **Kaiser-Meyer-Olkin (KMO) Measure of Sampling Adequacy**: anti-image partial correlation calculation $a_{ij} = -s_{ij} / \sqrt{s_{ii} s_{jj}}$ via numerical inversion of $S = R^{-1}$ and $\text{KMO} = \frac{\sum_{i \ne j} r_{ij}^2}{\sum_{i \ne j} r_{ij}^2 + \sum_{i \ne j} a_{ij}^2}$;
     * **Bartlett's Test of Sphericity**: $\chi^2 = -(n - 1 - (2p + 5)/6) \ln |R|$ at $df = p(p - 1)/2$ with exact $p$-value from Chi-square survival function;
   - Principal Component Analysis (**PCA**) Extraction:
     * Spectral decomposition of real symmetric matrix $R$ via cyclic Jacobi rotations on 100% safe Rust without external linear algebra dependencies (`jacobi_eigen_symmetric`);
     * Kaiser criterion ($\lambda \ge 1.0$) or fixed factor count extraction;
     * Deterministic sign convention for positive dominant loadings;
     * Factor loadings $l_{ik} = \sqrt{\lambda_k} \cdot v_{ik}$;
     * Communalities (*Communalities*): initial 1.000 for PCA, extraction $h_i^2 = \sum_{k=1}^m l_{ik}^2$;
     * Total Variance Explained (*Total Variance Explained*): initial eigenvalues, % of variance, cumulative %, and extraction sums of squared loadings;
     * Component Matrix (*Component Matrix*);
   - **Golden Oracle:** `factor_tests.rs` unit tests verified against Python `scipy.linalg.eigh` and `factor_analyzer` with precision $\le 10^{-6}$;
   - **Never Crash** protection against singular matrices, $n < 2$, zero-variance variables, and missing cases.

2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::factor_cmd`):**
   - Typed engine command `FactorCommand` (`stat.factor`) with JSON parameter specs for external AI agents;
   - Canonical IBM SPSS Statistics syntax generator and parser:
     ```text
     FACTOR
       /VARIABLES q1 q2 q3 q4 q5
       /MISSING LISTWISE
       /ANALYSIS q1 q2 q3 q4 q5
       /PRINT INITIAL EXTRACTION KMO
       /CRITERIA MINEIGEN(1)
       /EXTRACTION PC
       /ROTATION NOROTATE.
     ```
   - Full `Undo` / `Redo` lifecycle support.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::factor_dialog`):**
   - Centered canonical modal dialog ($560 \times 430$ px, `DialogShell::standard`);
   - Two-panel `VariablePicker` with measurement scale icons and vector transfer buttons `TransferButton` (`->` / `<-`);
   - `Variables:` target list (requires at least 2 items to enable `[ OK ]`);
   - Right sidebar `SubdialogSidebar` with modal configuration triggers:
     * `[ Descriptives... ]`: subdialog with `[x] KMO and Bartlett's test of sphericity`;
     * `[ Extraction... ]`: subdialog with PCA method and factor retention criteria (Eigenvalues over 1.0 / Fixed number of factors);
     * Action row `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Summary Pivot Tables in Output Viewer (`tables_factor.rs`):**
   - **KMO and Bartlett's Test:** Kaiser-Meyer-Olkin Measure of Sampling Adequacy, Bartlett's Test of Sphericity (Approx. Chi-Square, df, Sig.);
   - **Communalities:** Variable, Initial (1.000), Extraction;
   - **Total Variance Explained:** Component, Initial Eigenvalues (Total, % of Variance, Cumulative %), Extraction Sums of Squared Loadings;
   - **Component Matrix:** variables $\times$ components;
   - TSV clipboard export and Outline Tree integration.

5. **Reactivity, Menu & Status Bar:**
   - Activated main menu item `Analyze -> Dimension Reduction -> Factor...`;
   - Canonical syntax automatically recorded to Output Viewer (Log);
   - Status bar feedback notification (`Factor analysis computed for <n> case(s), <m> component(s) extracted`).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - Strict adherence to file size limits (`file-guard` $\le 500$ lines on leaf modules);
   - Strict adherence to DialogKit standards (`design-guard` — 100% compliant);
   - Architectural and parity linters (`layers`, `parity` — 38 documented procedures) reporting 0 violations;
   - Synchronous localization without string hardcoding (`locales/en.ftl`, `locales/ru.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase 12.1: Subgroup Means Breakdown and One-Way ANOVA (Subgroup Means Breakdown: `MEANS`)
*Time: October 29, 2026, 19:40 (~45 minutes)*

Implemented the classical exploratory procedure for subgroup analysis of IBM SPSS Statistics — **Subgroup Means Breakdown** (`Analyze -> Compare Means -> Means...`), combining detailed cell-level summary statistics across factor levels, one-way analysis of variance (ANOVA), and measures of association:

1. **Mathematical Engine L2 (`statcraft_algo::means`, `means_types`, `means_tests`):**
   - Listwise missing case deletion per table and full support for observation weights (`row_weight`);
   - Natural sorting of group categories (numeric ordering for numeric groups, lexicographic for strings);
   - Complete set of cell statistics per group and for the overall Total row:
     * Case count ($N$), weighted mean ($\bar{x}$), unbiased standard deviation ($s$), sample variance ($s^2$);
     * Median (with proper linear interpolation), minimum ($\min$), maximum ($\max$), weighted sum of values ($\sum x$);
   - **One-Way ANOVA Table (ANOVA Table):**
     * Between Groups variation: $SS_{between} = \sum n_j (\bar{x}_j - \bar{x})^2$, $df = k - 1$;
     * Within Groups variation: $SS_{within} = \sum \sum (x_{ij} - \bar{x}_j)^2$, $df = N - k$;
     * Total variation: $SS_{total} = SS_{between} + SS_{within}$, $df = N - 1$;
     * Mean Squares ($MS$), Fisher's $F$-test ($F = MS_{between} / MS_{within}$), and exact $p$-value from Fisher distribution survival function (`f_distribution_sf`);
   - **Measures of Association:**
     * Coefficient of determination $\eta^2 = SS_{between} / SS_{total}$ (proportion of variance explained) and correlation ratio $\eta = \sqrt{\eta^2}$;
   - **Golden Oracle:** `means_tests.rs` unit tests verified against analytical standards and Python `scipy.stats.f_oneway` with precision $\le 10^{-6}$;
   - **Never Crash** resilience against empty samples, single-case groups ($N < 2$), zero within-group variance, and missing data on 100% safe Rust.

2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::means_cmd`):**
   - Typed engine command `MeansCommand` (`stat.means`) with JSON parameter specs for external AI agents;
   - Canonical IBM SPSS Statistics syntax generator:
     ```text
     MEANS TABLES=salary score BY educ
       /CELLS=MEAN COUNT STDDEV VARIANCE MEDIAN MIN MAX SUM
       /STATISTICS ANOVA.
     ```
   - Full `Undo` / `Redo` lifecycle support.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::means_dialog`):**
   - Centered canonical modal dialog ($560 \times 430$ px, `DialogShell::standard`);
   - Two-panel `VariablePicker` with measurement scale icons and adaptive vector transfer buttons `TransferButton` (`->` / `<-`);
   - Target lists `Dependent List:` and `Layer 1 of 1:` (independent factor variables);
   - Right sidebar `SubdialogSidebar` with `[ Options... ]` button:
     * Modal subdialog for selecting statistics (Mean, Number of Cases, Std. Deviation, Variance, Median, Minimum, Maximum, Sum) and checkbox `[x] Anova table and eta`;
     * Action row `SubdialogActionBar` (`[ Continue ] [ Cancel ] [ Help ]`);
   - Canonical bottom action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).

4. **Summary Pivot Tables in Output Viewer (`tables_means.rs`):**
   - **Case Processing Summary:** Dependent $\times$ Grouping, Included ($N$, %), Excluded ($N$, %), Total ($N$, %);
   - **Report:** factor level breakdown with statistics (Mean, N, Std. Deviation, Variance, Median, Minimum, Maximum, Sum) and summary Total row;
   - **ANOVA Table:** Sum of Squares, df, Mean Square, F, Sig. for Between Groups, Within Groups, and Total;
   - **Measures of Association:** Eta, Eta Squared;
   - TSV clipboard export and Outline Tree integration.

5. **Reactivity, Menu & Status Bar:**
   - Activated main menu item `Analyze -> Compare Means -> Means...`;
   - Canonical syntax automatically recorded to Output Viewer (Log);
   - Status bar feedback notification (`Means breakdown computed for <n> case(s)`).

6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - Strict adherence to file size limits (`file-guard` $\le 500$ lines on leaf modules);
   - Strict adherence to DialogKit standards (`design-guard` — 100% compliant);
   - Architectural and parity linters (`layers`, `parity` — 39 documented procedures) reporting 0 violations;
   - Synchronous trilingual localization without string hardcoding (`locales/en.ftl`, `locales/ru.ftl`, `C:\StatCraft\locales\fr.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings;
   - Release binary built and deployed to `C:\StatCraft\statcraft_app.exe`.

---

### Phase MP-1: Session Lifecycle & File Menu Navigation
*Time: October 29, 2026, 21:30 (~25 minutes)*

Activated core session lifecycle and navigation items in the main `File` menu of StatCraft (`crates/statcraft_app`), transitioning them from `MenuItemStatus::InDev` to fully operational `MenuItemStatus::Ready`:
1. **Recent Files Manager L6 (`recent_files.rs`):**
   - `RecentFiles` queue data structure maintaining up to 10 recently opened files;
   - Automatic rotation: accessed files are prepended to the top, duplicates promoted, older entries beyond 10 truncated;
   - Dual persistence: saving to `.statcraft/recent_files.json` and integrating with `eframe::Storage` upon shutdown;
   - Interactive submenu `File -> Recent Files`: path tooltips on hover, disabled placeholder `(No recent files)` when empty, and separator with `Clear Recent Files`.
2. **Session Lifecycle Coordination L6 (`app_session.rs`):**
   - `File -> New -> Data` (`Ctrl+N`): clears current dataset to a blank sheet, resets `current_file_path`, resets `CommandBus` history and active filter/weight/split states, activates `Data View` tab, reports to status bar, and appends `NEW DATASET.` to the `Output Viewer` Log;
   - `File -> New -> Syntax`: shifts focus to the `Syntax Editor` tab, initializes canonical header `* StatCraft Syntax Editor. / * Created: <timestamp>.`, reports status;
   - `File -> New -> Project` (`Ctrl+Shift+N`): full state reset (dataset, output viewer log, syntax editor, active path);
   - `File -> Close Project`: resets active project state, restores status bar to `Ready`;
   - Unsaved changes guard `has_unsaved_changes()`: modal confirmation `ConfirmSaveDialog` (`[ Save ] [ Don't Save ] [ Cancel ]`) built on `DialogShell::subdialog` prevents accidental loss of data.
3. **Syntax Editor Interactive Screen L6 (`views/syntax_editor.rs`):**
   - Interactive view `SyntaxEditorState` and `render_syntax_editor`;
   - Monospace multiline code editor tab docked in the workspace (`Tab::SyntaxEditor`);
   - Shortcut navigation via `Window -> Syntax Editor` (`Ctrl+4`).
4. **Quality Control & Definition of Done:**
   - Dedicated unit and integration test suite `crates/statcraft_app/tests/session_lifecycle_tests.rs` (queue rotation, change detection, session reset flows, headless rendering);
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - File guard compliance: coordinator `app.rs` stays strictly under limit (1407 of 1500 lines);
   - DialogKit design compliance `design-guard` — 100% OK;
   - Synchronous bilingual localization (`locales/en.ftl`, `locales/ru.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings.

---

### Phase MP-3: Export Data & Export Output Reports
*Time: October 29, 2026, 22:15 (~35 minutes)*

Implemented complete export workflows for data and analysis outputs in the `File -> Export` menu:
1. **Data Export L0/L1 (`statcraft_io::csv_writer`, `statcraft_core::csv_export`):**
   - L0 module `csv_writer.rs` supporting `CsvExportOptions` (comma, semicolon, tab delimiters, quote escaping, newline/carriage return handling, decimal comma conversion, and configurable missing value `sysmis_repr`);
   - L1 module `csv_export.rs` implementing `export_dataset_to_csv` for streaming `DataSet` export with value labels substitution (`ValueLabels`) or raw numeric codes, empty dataset safety (0 rows/cols), and streaming `std::io::Write`;
   - Save file dialog filters: `Comma-Separated Values (*.csv)`, `Tab-Delimited Values (*.tsv, *.txt)`, `Semicolon-Separated Values (*.csv)` with automatic delimiter resolution from path.
2. **Reporting Architectural Layer L4 (`crates/statcraft_reporting`):**
   - Created new standalone L4 layer crate `statcraft_reporting` (registered in `xtask/src/layers.rs` maintaining strict top-to-bottom hierarchy L4 $\to$ Utility, L6 $\to$ L4);
   - Universal reporting model: `ReportDocument`, `ReportSection`, `ReportItem` (Log, Notes, Table, Text, Warning), `ReportTable`, `ReportNotes`, `CellAlignment`;
   - HTML report generator `html_export.rs`: standalone single-file HTML5 document with authentic SPSS table styling (sans-serif typography, double header borders, zebra striping, accent procedure headings, italic footnotes, monospace syntax logs, and HTML entity escaping);
   - Monospace plain text generator `txt_export.rs`: ASCII border tables with dynamic column width calculation and text alignment;
   - TSV bundle generator `tsv_export.rs`: joins all session tables separated by double newlines for direct spreadsheet pasting (Excel, Google Sheets).
3. **User Interface & Integration L6 (`crates/statcraft_app`):**
   - Module `views/output_view/export.rs`: converts `OutputViewerState` into `ReportDocument` across all statistical procedures;
   - Module `export_handlers.rs`: activates `File -> Export -> Export Output...` and `File -> Export -> Export Data...` (with `requires_dataset_status` availability guard);
   - Native OS save file dialogs via `rfd::FileDialog::save_file()`;
   - Status bar feedback notifications (`status-data-exported`, `status-output-exported`);
   - Syntax logging in `Output Viewer`: `SAVE TRANSLATE OUTFILE='...' /TYPE=... /REPLACE /FIELDNAMES.` and `OUTPUT EXPORT /CONTENTS=ALL /TYPE=... /DESTINATION='...'`.
4. **Quality Control & Definition of Done:**
   - 100% pass on full CI pipeline: `cargo run -p xtask -j 1 -- ci` (`fmt`, `clippy -D warnings`, `cargo test --workspace -j 1`, `parity`);
   - Strict layer validation `cargo run -p xtask -j 1 -- layers` (L4Reporting verification);
   - File size guard compliance `file-guard` ($\le 500$ lines on all new modules);
   - DialogKit design compliance `design-guard`;
   - Synchronous bilingual localization without string hardcoding (`locales/en.ftl`, `locales/ru.ftl`);
   - Test suites added for `statcraft_io::csv_writer`, `statcraft_core::csv_export`, `statcraft_reporting` (HTML, TXT, TSV), `statcraft_app::export_handlers`, and `output_view::export`.

---

### Phase MP-5: SPSS Binary Data Files Import (.sav / .zsav)
*Time: October 29, 2026, 22:20 (~30 minutes)*

Implemented full-featured import of original binary IBM SPSS Statistics files (`.sav` and compressed `.zsav`) with complete extraction of data matrices and dictionary metadata without relying on proprietary third-party libraries:
1. **Low-Level Binary Decoder L0 (`crates/statcraft_io::spss_loader`):**
   - Modular architecture: `header.rs`, `records.rs`, `decompress.rs`, `mod.rs` (each strictly $\le 500$ lines compliant with `file-guard`);
   - System Header Record 0 parsing (`$FL2` / `$FL3`, automatic endianness detection via `layout_code`, compression bias `bias = 100.0`, case weight index `weight_index`, case count `case_cnt`);
   - Variable Records Record 2 handling: numeric & string variables, labels, user missing value specifications (discrete values, ranges);
   - Value Labels Records Record 3 & 4: pairs `(f64, label)` mapped to target variable slots;
   - Extension Records Record 7: subtype 4 (Display parameters: Measure Scale/Ordinal/Nominal, alignment, column widths), subtype 13 (Long Variable Names mapping short 8-char names to UTF-8 names), subtype 20 (Character Encoding), subtype 24 (ZLIB chunk metadata for `.zsav`);
   - Bytecode streaming decompressor Record 999: opcode decoding 1..251 (`code - bias`), 252 (EOF), 253 (8-byte IEEE 754 floats or string literals), 254 (whitespace), 255 (system missing `SYSMIS` / `Null`);
   - Compressed `.zsav` support: reads `ztrailer`, streams block decompression via `flate2::read::ZlibDecoder` into uniform bytecode reader.
2. **Core Layer Integration L1 (`crates/statcraft_core::spss`):**
   - Module `spss.rs`: `DataSet::from_spss_data`, `DataSet::load_spss_from_path`, `DataSet::load_spss_from_reader`;
   - Converts `SpssVariableInfo` into `VariableMetadata`: 9 canonical SPSS types, `Measure`, `Alignment`, `ValueLabels`, `MissingValues`, and `weight_var`;
   - Typed error integration `CoreError::Io(#[from] statcraft_io::IoError)`.
3. **Command Bus L5 (`crates/statcraft_syntax`):**
   - Typed engine command `ImportSpssCommand` (`data.import_spss`) with JSON schema `specs()`, execution and Undo support;
   - Registered in `CommandBus::with_defaults()`;
   - SPSS syntax equivalent generator `GET FILE='<path>'.`;
   - Transparent `.sav` and `.zsav` support added to `OpenFileCommand` (`data.open_file`).
4. **Desktop UI L6 (`crates/statcraft_app`):**
   - Activated menu item `File -> Import Data -> SPSS Data...` (`menu-file-import-spss`) with `SPSS Statistics Data (*.sav, *.zsav)` file dialog;
   - Added SPSS file filters to general `File -> Open Data...` dialog (`open_data_dialog`);
   - `load_file_from_path` in `app_session.rs` automatically detects `.sav`/`.zsav`, loads dataset via command bus, logs `GET FILE='<path>'.` into Output Viewer, updates `RecentFiles`, and focuses `Data View`.
5. **Quality Control & Definition of Done:**
   - 100% pass on CI quality gate: `cargo run -p xtask -j 1 -- ci` (`fmt`, `clippy -D warnings`, `cargo test --workspace -j 1`, `parity`, `layers`, `file-guard`, `design-guard`);
   - Unit tests in `statcraft_io::spss_loader::tests`: bytecode compression, uncompressed streams, corrupt header handling;
   - Session lifecycle integration test `crates/statcraft_app/tests/session_lifecycle_tests.rs::test_load_spss_sav_session_lifecycle`;
   - 100% bilingual localization (`locales/en.ftl`, `locales/ru.ftl`);
   - 100% Safe Rust without `unwrap`/`expect`/`panic` in non-test code.


---

### Phase MP-2: Native Project Format (.stc) & Save Commands
*Time: October 29, 2026, 22:25 (~30 minutes)*

Implemented full native project format (`.stc`) persistence and file operations across the main `File` menu:
1. **Native Project Format Codec L0 (`statcraft_io::stc_codec`):**
   - Project container structure `StcContainer` (`format_version = 1`, `dataset_name`, `metadata_json`, `table: RawTable`, `active_filter_var`, `weight_var`, `split_file_json`);
   - Streaming Gzip compression via `flate2::write::GzEncoder` with automatic detection of the GZIP header (`0x1f 0x8b`) and fallback JSON deserialization on read;
   - Clean codec APIs: `save_stc_container`, `load_stc_container`, `save_stc_container_to_path`, `load_stc_container_from_path`;
   - CSV/TSV table export `crates/statcraft_io/src/csv_writer.rs` (`export_raw_table_to_csv_path`) with localized decimal separator and escaping support.
2. **Comprehensive Metadata Dictionary Serialization L1 (`statcraft_core::stc_codec`):**
   - 100% loss-free serialization and restoration of dataset state:
     * All 9 canonical SPSS variable data types (`Numeric`, `Comma`, `Dot`, `Scientific`, `Date`, `Dollar`, `CustomCurrency`, `String`, `RestrictedNumeric`);
     * Measurement scales (`Scale`, `Ordinal`, `Nominal`), text alignments (`Left`, `Right`, `Center`), column widths, and decimal digits;
     * Complete value label dictionaries `ValueLabels` (`BTreeMap<String, String>`);
     * Discrete and range missing values `MissingValues`;
     * Active filter configuration (`active_filter_var`), frequency weight variable (`weight_var`), and split-file settings (`split_file`);
   - Unified methods: `save_stc_to_writer`, `load_stc_from_reader`, `save_stc_to_path`, `load_stc_from_path`, and `save_csv_to_path`.
3. **Command Bus & Canonical Syntax L5 (`statcraft_syntax::commands::file_cmds`):**
   - Save command `SaveFileCommand` (`data.save_file`) with parameter specs `specs()` and canonical syntax generator `SAVE OUTFILE='<path>'.\nEXECUTE.`;
   - Open command `OpenFileCommand` (`data.open_file`) with syntax generator `GET FILE='<path>'.\nEXECUTE.`;
   - Default registration in `CommandBus::with_defaults()`.
4. **Main Menu & Native Dialog Integration L6 (`statcraft_app::app_session` & `app.rs`):**
   - Transitioned menu items `File -> Save` (`Ctrl+S`), `File -> Save As...` (`Ctrl+Shift+S`), `File -> Save Data As...`, `File -> Open Project...` (`Ctrl+Shift+O`), `File -> Open Data...` (`Ctrl+O`) to `MenuItemStatus::Ready`;
   - Native system file dialog integration via `rfd::FileDialog` with file extensions, filters, and smart default filenames;
   - Responsive keyboard shortcuts in `update()` (`Ctrl+S` -> `save_project`, `Ctrl+Shift+S` -> `save_project_as`, `Ctrl+Shift+O` -> `open_project_dialog`);
   - Dynamic window title displaying the active file name and dirty state indicator `*`;
   - Automatic `RecentFiles` queue tracking and Output Viewer log recording (`SAVE OUTFILE=...` / `GET FILE=...`);
   - Safe session transition guard: prompt to save project before creating/opening new projects via `ConfirmSaveDialog`.
5. **Architectural Decomposition ("Divide, don't prune"):**
   - Extracted status message coordination into `crates/statcraft_app/src/status_message.rs`, slimming `app.rs` from 1453 down to ~1300 lines, ensuring healthy safety margin under the 1500 line limit.
6. **Quality Control & Definition of Done:**
   - Full automated CI quality gate passed (`cargo run -p xtask -j 1 -- ci` — 100% OK);
   - 202 Rust files verified within strict size limits (`file-guard` — 100% OK);
   - Architecture layer rules `layers` (L0-L6) and design rules `design-guard` — 100% compliant;
   - 125+ workspace unit and integration tests passing;
   - 100% synchronous localization (`locales/en.ftl`, `locales/ru.ftl`);
   - Compiler linter `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — 0 warnings.

---

### Phase MP-4: Microsoft Excel Import (.xlsx / .xls / .ods)
*Time: October 29, 2026, 23:05 (~35 minutes)*

Implemented direct tabular data reading and importing from Microsoft Excel and OpenDocument spreadsheets without needing prior conversion to CSV:
1. **I/O Layer L0 (`statcraft_io::excel_loader`):**
   - Integrated modern crate `calamine = "0.37"` (100% safe Rust, memory-efficient streaming parser without COM or Office dependencies);
   - Functions `load_excel_from_path`, `load_excel_from_reader`, and `load_excel_from_range` supporting `.xlsx`, `.xlsm`, `.xls`, `.ods`;
   - Worksheet parsing engine:
     * Default selection of the first sheet (`workbook.sheet_names()[0]`) or by explicit sheet name;
     * Canonical SPSS variable name sanitization and deduplication (`sanitize_and_dedup_headers`): invalid character conversion to `_`, `V_` prefix for leading digits, `VAR00001` generation for empty headers, and sequential `_1`, `_2` disambiguation;
     * High-fidelity cell type mapping from `calamine::Data` (integers `Int`, floats `Float`, booleans `Bool` -> `1.0`/`0.0`, dates/times `DateTime`/`DateTimeIso`, strings `String`, empty/error cells `Empty`/`Error`/`"NA"`/`"."` -> `RawValue::Null`);
     * Automated column type inference: columns where all non-null values parse as numbers are typed as `RawDataType::Numeric`, while mixed columns cast numbers to formatted text as `RawDataType::Text`;
   - **Never Crash** guarantees: robust handling of empty workbooks, 0-row sheets, and header-only sheets without panics.
2. **Command Bus & Syntax L5 (`statcraft_syntax::commands::import_excel`):**
   - Strongly-typed `ImportExcelCommand` (`data.import_excel`) with parameter specification `specs()` (`file_path`, `sheet_name`, `dataset_name`) for external AI agents;
   - Direct population of active `DataSet` with full `Undo` support (dataset reset);
   - Registered in default command bus `CommandBus::with_defaults()`.
3. **User Interface & Integration L6 (`crates/statcraft_app`):**
   - Activated menu items `File -> Import Data -> Excel...` (`menu-file-import-excel`) and `File -> Import Data -> CSV...` (`menu-file-import-csv`);
   - Added `open_excel_dialog` using `rfd::FileDialog` with filter `Excel Spreadsheets (*.xlsx, *.xls, *.ods, *.xlsm)`;
   - Enhanced `load_file_from_path`: automated detection of spreadsheet extensions, execution of `data.import_excel`, addition to `RecentFiles` queue;
   - Canonical syntax logging in Output Viewer:
     ```text
     GET DATA /TYPE=XLSX /FILE='<path>' /FIRSTROW=YES.
     ```
   - Real-time status bar notification: `Dataset imported: <name> (<n> cases, <m> variables)`.
4. **Quality Control & Definition of Done:**
   - 100% pass on full CI pipeline: `cargo run -p xtask -j 1 -- ci` (`cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace -j 1`, `parity`);
   - Architecture layer integrity verified via `xtask layers` and `xtask parity` (40 documented commands);
   - File guard compliance: `excel_loader.rs` (344 of 500 lines), coordinator `app.rs` (1471 of 1500 lines);
   - DialogKit design compliance `design-guard`;
   - Synchronous bilingual localization without string hardcoding (`locales/en.ftl`, `locales/ru.ftl`);
   - Comprehensive test coverage for Excel parsing in `statcraft_io`, command execution in `statcraft_syntax`, and session lifecycle integration.

---

### Final Phase: Seamless File Menu Integration, Save Dialog Guard & Global Shortcuts
*Time: October 29, 2026, 22:45 (~20 minutes)*

Completed comprehensive end-to-end integration for the 12 File Menu operations within StatCraft (`crates/statcraft_app`):
1. **Window Close Interception & Data Loss Prevention (Window Close Guard):**
   - Intercepted window close requests via `ctx.input(|i| i.viewport().close_requested())` when unsaved changes exist (`has_unsaved_changes() == true`);
   - Instantly canceled unprompted exit via `ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose)`;
   - Triggered canonical modal `ConfirmSaveDialog` while capturing the requested deferred action in `pending_session_action`;
   - Full support for actions: `[ Save ]` (persists changes to `.stc` / `.csv` before proceeding), `[ Don't Save ]` (discards changes and executes deferred action), and `[ Cancel ]` (retains changes and aborts transition).
2. **Global Keyboard Accelerators:**
   - Universal shortcuts accessible across all active application panes:
     * `Ctrl+Shift+N` — New Project (`NewProject`);
     * `Ctrl+N` — New Dataset (`NewDataset`);
     * `Ctrl+O` — Open Data (`open_data_dialog` supporting `.stc`, `.sav`, `.zsav`, `.csv`, `.xlsx`);
     * `Ctrl+Shift+O` — Open StatCraft Project (`open_project_dialog`);
     * `Ctrl+S` — Save Project (`save_project`);
     * `Ctrl+Shift+S` — Save Project As (`save_project_as`).
3. **Repository Cleanliness & Dialog Invariant Testing (DoD):**
   - Added headless regression test `test_headless_render_confirm_save_dialog` in `crates/statcraft_app/tests/dialog_invariants.rs` ensuring panic-free rendering and state compliance;
   - Ignored runtime `.statcraft/` session folders in `.gitignore`;
   - 100% pass across all 130+ workspace unit and integration tests with `-j 1` and complete pass on `cargo run -p xtask -j 1 -- ci` (`cargo fmt --check`, `clippy -D warnings`, `file-guard`, `design-guard`, `layers`, `parity`).

---

### Phase 41: Full Project Workspace Persistence & Restoration (.stc)
*Time: October 29, 2026, 23:40 (~35 minutes)*

Implemented monolithic persistence of the complete StatCraft analytical workspace in the native `.stc` container format (analogous to `.psd` in Adobe Photoshop):
1. **L0 Project Container Extension (`statcraft_io::stc_codec`):**
   - Extended `StcContainer` with optional fields `output_viewer_json` and `syntax_text` preserving full backward compatibility (`Format Version 1`, `#[serde(default, skip_serializing_if = "Option::is_none")]`);
   - Added constructor `StcContainer::with_project_data` for unified packaging of all session artifacts;
   - Guaranteed backward compatibility: legacy `.stc` files without viewer or syntax data load seamlessly without error.
2. **Output Viewer Serialization L6 (`statcraft_app::views::output_view`):**
   - Implemented `static_str_serde` module for safe deserialization of static localization keys (`&'static str` via `Box::leak`);
   - Full serialization support across all 19 `OutputItemKind` variants (descriptive statistics, frequency tables, t-tests, ANOVA, crosstabs, correlations, non-parametric tests, linear regression, reliability analysis, factor analysis, means, execution logs, notes, warnings, and dataset summaries);
   - Added `OutputViewerState::to_json()` and `OutputViewerState::load_from_json()` with automated `next_id = max(id) + 1` recalculation preventing identifier collisions during subsequent computations.
3. **L1 Core Bridge (`statcraft_core::stc_codec`):**
   - Implemented `project_to_stc`, `save_project_stc_to_path`, and `load_project_stc_from_path` coordinating dataset raw tables, metadata dictionary, and application project components.
4. **Session Lifecycle Integration L6 (`statcraft_app::app_session`):**
   - In `save_to_path`: saving `.stc` captures the right-hand Output Viewer procedures and active syntax script;
   - In `load_file_from_path`: opening an `.stc` restores 100% of the analytical space — raw data table, variable dictionary, active case filter, weights, split file configuration, all output procedures/pivot tables, and syntax editor script;
   - Strict menu decoupling: `File -> Save Data As...` excludes `.stc` and exports only tabular data (`.csv` / `.tsv`), while `File -> Save Project` (`Ctrl+S`) and `File -> Save Project As...` (`Ctrl+Shift+S`) persist the complete monolithic workspace.
5. **Quality Control & Definition of Done:**
   - Added end-to-end integration test `test_save_and_load_full_project_with_output_and_syntax` in `session_lifecycle_tests.rs`;
   - Full workspace test suite `cargo test --workspace -j 1` passing with 0 failures;
   - Complete CI quality gate passed: `cargo run -p xtask -j 1 -- ci` (`file-guard`, `design-guard`, `layers`, `parity`, `clippy -D warnings`, `cargo fmt --check`).

---

### Phase 42: Toolbar Save Action, Saved State Checkpoint (SavedCheckpoint) & Status Indication
*Time: October 30, 2026, 05:35 (~25 minutes)*

Resolved lifecycle defects in project persistence, dirty state synchronization, and user feedback:
1. **Toolbar Action Dispatching L6 (`crates/statcraft_app/src/app.rs`):**
   - Implemented `ToolbarAction::SaveData`: clicking the diskette icon on the main toolbar now triggers `self.save_project()`, presenting a save dialog or updating the active `.stc` project file;
   - Implemented `ToolbarAction::Redo` dispatching to `ds.redo()`, dynamically synchronizing `can_redo` availability with `ds.delta.redo_stack`.
2. **Saved State Checkpoint `SavedCheckpoint` (`crates/statcraft_app/src/app_session.rs`):**
   - Designed a lightweight `SavedCheckpoint` tracking document state (column and row counts, cell undo stack length `delta_undo_len`, command history length `command_history_len`, syntax text length `syntax_len`, and output viewer procedure count);
   - Added `saved_checkpoint: Option<SavedCheckpoint>` to `StatCraftApp`;
   - Refactored `has_unsaved_changes(&self)` to evaluate current document deviation from `saved_checkpoint`: the dirty marker (`*`) in the window caption bar turns off immediately after saving and also when reverting changes via `Undo` (`Ctrl+Z`);
   - Automated checkpoint capture via `update_saved_checkpoint()` across `save_to_path`, `load_file_from_path` (supporting `.stc`, `.sav`, `.xlsx`, `.csv`), `execute_new_dataset`, `execute_new_project`, and `execute_close_project`.
3. **Status Bar Visual Feedback (`crates/statcraft_app/src/status_message.rs`):**
   - Added `StatusMessage::color(&self) -> egui::Color32`: save and export confirmations (`ProjectSaved`, `DataSaved`, `DataExported`, `OutputExported`) are rendered in `Tokens::MEASURE_NOMINAL` emerald green; error alerts in `Color32::from_rgb(235, 80, 80)`; standard operational status in `Tokens::TEXT_MUTED`;
   - Used `ui.colored_label(...)` in the bottom status panel for immediate peripheral recognition.
4. **Quality Control & Definition of Done:**
   - Extended `test_has_unsaved_changes_detection` and added `test_toolbar_save_action_execution` in `crates/statcraft_app/tests/session_lifecycle_tests.rs`;
   - Added unit test `test_status_message_colors` in `status_message.rs`;
   - 100% test pass on workspace suite: `cargo test --workspace -j 1`;
   - Full pass across all CI quality gates: `cargo run -p xtask -j 1 -- ci` (`cargo fmt --check`, `clippy -D warnings`, `file-guard`, `design-guard`, `layers`, `parity`).

---

### Phase 43: Shift Values Procedure (Lead / Lag) — `transform.shift_values`
*Time: October 30, 2026, 06:05 (~30 minutes)*

Implemented the canonical time-series shift procedure from IBM SPSS Statistics — `Transform -> Shift Values (Lead / Lag)...` (`transform.shift_values` command / `SHIFT VALUES` syntax):
1. **L2 Algorithmic Foundation (`crates/statcraft_algo/src/shift.rs`):**
   - Defined `ShiftDirection` enum (`Lag` — shift down referencing past cases, `Lead` — shift up referencing future cases);
   - Core function `compute_shift_raw(values, direction, step)` over `RawValue` column vectors with arbitrary step $K \ge 1$;
   - Full edge-case handling: step $K = 0$ or empty vectors return identical clones; $K \ge N$ fills the vector with `RawValue::Null`;
   - For `Lag(step)`, the first $K$ observations are padded with `RawValue::Null`, followed by `values[i - step]`;
   - For `Lead(step)`, observations are shifted up by $K$ positions, with the trailing $K$ cells filled with `RawValue::Null`;
   - 100% safe Rust without `unwrap`/`expect`/`panic`, unit tested across empty cases, nulls, and string fields.
2. **L5 Command Bus & Syntax Translation (`crates/statcraft_syntax/src/commands/shift_values_cmd.rs`):**
   - Strongly-typed `ShiftValuesCommand` (`transform.shift_values`) registered in `CommandBus`;
   - JSON parameter `specs()` for AI agents (`source_var`, `target_var`, `direction`, `step`, `target_label`);
   - Canonical SPSS syntax generator:
     ```text
     SHIFT VALUES /VARIABLE=sales /RESULT=sales_lag1 /LAG=1.
     VARIABLE LABELS sales_lag1 "Previous Sales".
     EXECUTE.
     ```
   - Safe Undo/Redo history: new variables inherit data type and measurement scale from the source column, while existing variables save old metadata and column values for complete restoration (`ShiftUndoEntry::Created` / `ShiftUndoEntry::Overwritten`);
   - Integration tests covering command execution, syntax generation, and clean undo reversal.
3. **Zero-Hardcode Localization (`locales/en.ftl`, `locales/ru.ftl`):**
   - Declarative translation keys for dialog title, labels, direction radios, steps counter, and log outputs.
4. **L6 DialogKit Desktop User Interface (`crates/statcraft_app/src/dialogs/shift_values_dialog.rs`):**
   - Standard modal dialog window adhering to `DialogKit` (`DialogShell::standard` 560 × 430 px);
   - Available variables list via `render_variable_list_panel` and vector `TransferButton`;
   - Target variable name and optional label inputs with automatic suffix suggestion;
   - Shift direction `GroupBox` (`Lag` / `Lead`) and step `DragValue` counter;
   - Canonical Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **L6 Application Integration (`app.rs`, `app_dialogs_data.rs`):**
   - Activated menu item `menu-transform-shift` under `Transform` (removed `[dev]` badge);
   - Executing the dialog triggers `transform.shift_values`, appends syntax log to `Output Viewer`, and sets `StatusMessage::ShiftValuesExecuted` in the status bar;
   - Added headless regression test `test_headless_render_shift_values_dialog` in `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Quality Control & Definition of Done:**
   - 100% pass across workspace unit and integration tests;
   - Passed all architectural and style linters: `layers`, `file-guard`, `design-guard`, `parity`.

---

### Phase 44: Count Values within Cases Procedure — `transform.count_values`
*Time: October 30, 2026, 06:15 (~35 minutes)*

Implemented the canonical case-level value occurrence aggregation procedure from IBM SPSS Statistics — `Transform -> Count Values within Cases...` (`transform.count_values` command / `COUNT` syntax):
1. **L2 Algorithmic Foundation (`crates/statcraft_algo/src/count_values.rs`):**
   - Defined `ValueCriterion` enum (`ExactNumber`, `ExactString`, `Range { low, high }`, `LowestThru(high)`, `ThruHighest(low)`, `SystemMissing`);
   - Criterion matching via `ValueCriterion::matches(&self, val: &RawValue) -> bool` with floating-point epsilon $10^{-9}$, string support, numeric ranges, and missing value checks (`SYSMIS` / `NaN` / `Null`);
   - Core routine `compute_count_across_columns(rows, columns_data, criteria)`: calculates the number of variables satisfying any of the specified criteria for each case;
   - Returns a non-missing numeric vector `RawValue::Number(count as f64)` following SPSS standards;
   - 100% safe Rust without `unwrap`/`expect`/`panic`, with comprehensive unit tests.
2. **L5 Command Bus & Syntax Translation (`crates/statcraft_syntax/src/commands/count_values_cmd.rs`):**
   - Strongly-typed `CountValuesCommand` (`transform.count_values`) registered in `CommandBus`;
   - JSON parameter `specs()` for AI agents (`target_var`, `target_label`, `variables`, `criteria`);
   - Canonical SPSS syntax generator:
     ```text
     COUNT high_scores = q1 q2 (5) (8 THRU 10).
     VARIABLE LABELS high_scores "High score count".
     EXECUTE.
     ```
   - Safe Undo/Redo history: new variables receive `DataType::Numeric { decimals: 0 }` and `Measure::Scale`, while overwriting existing columns saves previous metadata and values (`CountUndoEntry::Created` / `CountUndoEntry::Overwritten`);
   - Unit tests covering execution, syntax formatting, and undo lifecycle in `statcraft_syntax`.
3. **Zero-Hardcode Localization (`locales/en.ftl`, `locales/ru.ftl`):**
   - Declarative keys for dialog titles, target variable inputs, lists, range radio groups, `Add`/`Change`/`Remove` buttons, and `output-count-executed` status reporting.
4. **L6 DialogKit Desktop User Interface (`crates/statcraft_app/src/dialogs/`):**
   - Main dialog `CountValuesDialog` (`count_values_dialog.rs`, 560 × 430 px, `DialogShell::standard`) with target variable & label fields, available variables list via `render_variable_list_panel`, vector `TransferButton`, selected variables list, and right sidebar with `Define Values...` button and criteria preview;
   - Dedicated modal subdialog `CountValuesDefineState` (`count_values_define_dialog.rs`, `DialogShell::standard`) to configure exact values, system-missing markers, and ranges (`Range`, `LOWEST THRU`, `THRU HIGHEST`) with `SubdialogActionBar`;
   - Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **L6 Application Integration (`app.rs`, `app_dialogs_data.rs`):**
   - Main menu item `Transform -> Count Values within Cases...` activated (removed `InDev` status);
   - Execution updates the active dataset, appends canonical syntax to `Output Viewer`, and displays `StatusMessage::CountValuesExecuted` in the status bar;
   - Added headless regression test `test_headless_render_count_values_dialog` in `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Quality Control & Definition of Done:**
   - 100% workspace tests passing;
   - Full pass across all architectural guards: `layers`, `file-guard`, `design-guard`, `parity`.

---

### Phase 45: Set Random Number Seed Procedure — `transform.random_seed`
*Time: October 30, 2026, 06:16 (~30 minutes)*

Implemented the canonical pseudorandom number generator seed configuration procedure from IBM SPSS Statistics — `Transform -> Set Random Number Seed...` (`transform.random_seed` command / `SET SEED` syntax):
1. **L1 RNG State Engine (`crates/statcraft_core/src/rng.rs`):**
   - Thread-safe atomic global seed storage (`AtomicBool`, `AtomicU64`, `Ordering::SeqCst`);
   - Functions `set_global_random_seed(seed: u64)`, `clear_global_random_seed()`, and `get_global_random_seed() -> Option<u64>`;
   - Helper function `get_effective_random_seed() -> u64`: returns the fixed seed or generates a clock-derived value in the canonical range [1, 2,000,000,000];
   - Full support for both "Random initialization" and "Fixed starting point (seed)" modes.
2. **L5 Command Bus & Syntax Translation (`crates/statcraft_syntax/src/commands/random_seed_cmd.rs`):**
   - Strongly-typed `RandomSeedCommand` (`transform.random_seed`) registered in `CommandBus`;
   - JSON parameter `specs()` (`seed` integer or null, range 1 to 2,000,000,000);
   - Canonical SPSS syntax generator:
     ```text
     SET SEED = 2000000.
     ```
     or `SET SEED = RANDOM.` upon random initialization;
   - Full `undo` support restoring the previous seed state via `RandomSeedUndoEntry`;
   - Unit tests for parameter validation, syntax generation, and undo lifecycle.
3. **Zero-Hardcode Localization (`locales/en.ftl`, `locales/ru.ftl`):**
   - Declarative keys for `dialog-seed-title`, `dialog-seed-mode-random`, `dialog-seed-mode-fixed`, `dialog-seed-value-label`, `status-random-seed-fixed`, and `status-random-seed-random`.
4. **L6 DialogKit Desktop User Interface (`crates/statcraft_app/src/dialogs/random_seed_dialog.rs`):**
   - Modal window `RandomSeedDialog` built with `DialogKit` (`DialogShell::properties`, `GroupBox`, mode radio buttons, validated single-line text edit);
   - Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **L6 Application Integration (`app.rs`, `app_dialogs_data.rs`):**
   - Activated main menu item `Transform -> Set Random Number Seed...` (removed `InDev` status);
   - Executing the dialog updates global RNG state, logs syntax to `Output Viewer`, and sets `StatusMessage::RandomSeedExecuted` in the status bar;
   - Added headless regression test `test_headless_render_random_seed_dialog` in `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Quality Control & Definition of Done:**
   - 100% workspace tests passing;
   - Full pass across all architectural guards: `layers`, `file-guard`, `design-guard`, `parity`, and `clippy -D warnings`.

---

### Phase 46: Create Dummy Variables Procedure (`SPSSINC CREATE DUMMIES`) — `transform.create_dummies`
*Time: October 30, 2026, 06:45 (~30 minutes)*

Implemented the canonical indicator (0/1) binary dummy variables generation procedure (One-Hot Encoding) from IBM SPSS Statistics — `Transform -> Create Dummy Variables...` (`transform.create_dummies` command / `SPSSINC CREATE DUMMIES` syntax):
1. **L2 Algorithmic Foundation (`crates/statcraft_algo/src/dummies.rs`):**
   - Engine module `generate_dummies_for_column` with `DummySpec` and `GeneratedDummyColumn` types;
   - Unique category values detection and ascending order sorting for numeric and textual inputs, safely skipping missing data (`Null`, `NaN`, empty strings);
   - Support for reference category omission (`omit_first == true`) producing $k - 1$ indicator variables for linear regression modeling;
   - Value labels integration via `ValueLabels`: creates readable column labels such as `gender = Female`, `jobcat = Clerical`;
   - Variable name sanitization (alphanumerics and underscores), missing value propagation (`RawValue::Null`);
   - 100% Safe Rust without panics (`unwrap`, `expect`) with comprehensive unit tests for numeric, text, and empty datasets.
2. **L5 Command Bus & Syntax Translation (`crates/statcraft_syntax/src/commands/dummy_cmd.rs`):**
   - Strongly-typed `CreateDummiesCommand` (`DummyCommand`, ID `transform.create_dummies`) registered in `CommandBus`;
   - JSON parameter `specs()` (`variables`, `root_prefix`, `omit_reference`);
   - Appending newly created variables to dataset (`Numeric`, 0 decimals, `Measure::Nominal`) with automatic collision resolution;
   - Full `Undo` (`Ctrl+Z`) support via reverse index removal history (`DummyUndoEntry`);
   - Canonical SPSS extension syntax generator:
     ```text
     SPSSINC CREATE DUMMIES VARIABLES=gender jobcat
       /ROOTNAMES=d_gender d_jobcat
       /OMITFIRST=NO.
     EXECUTE.
     ```
3. **Zero-Hardcode Localization (`locales/en.ftl`, `locales/ru.ftl`):**
   - Declarative translation keys: `dialog-dummy-title`, `dialog-dummy-source-vars`, `dialog-dummy-root-prefix`, `dialog-dummy-omit-reference`, `dialog-dummy-error-empty-vars`, and `status-dummies-executed`.
4. **L6 DialogKit Desktop User Interface (`crates/statcraft_app/src/dialogs/dummy_dialog.rs`):**
   - Modal window `DummyDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Available variables list via `render_variable_list_panel` on the left and vector `TransferButton` in the center;
   - Right column: selected categorical variables list, `GroupBox` with prefix `TextEdit`, and `omit_reference` checkbox;
   - Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **L6 Application Integration (`app.rs`, `app_dialogs_data.rs`):**
   - Main menu item `Transform -> Create Dummy Variables...` activated (removed `InDev` status);
   - Execution handled in `app_dialogs_data.rs`, logging syntax to `Output Viewer` and emitting `StatusMessage::CreateDummiesExecuted` in the status bar;
   - Added headless regression test `test_headless_render_dummy_dialog` in `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Quality Control & Definition of Done:**
   - 100% workspace tests passing;
   - Full pass across all architectural guards: `layers`, `file-guard`, `design-guard`, `parity`, and `clippy -D warnings`.

---

### Phase 47: Automatic Recode Procedure (`AUTORECODE`) — `transform.auto_recode`
*Time: October 30, 2026, 07:15 (~30 minutes)*

Implemented the canonical transformation of string or irregular numeric categories into consecutive integer codes ($1, 2, 3, \dots, K$) with automatic assignment of original values to value labels — `Transform -> Recode -> Automatic Recode...` (`transform.auto_recode` command / `AUTORECODE` syntax):
1. **L2 Algorithmic Foundation (`crates/statcraft_algo/src/auto_recode.rs`):**
   - Core function `compute_auto_recode` and ordering enumeration `AutoRecodeOrder` (`LowestValue`, `HighestValue`);
   - Detection of unique non-missing values (`RawValue::Number` and `RawValue::Text`), safely skipping missing data (`Null`, `NaN`, empty strings);
   - Sorting unique categories: ascending (`LowestValue`) or descending (`HighestValue`);
   - Sequential assignment of integer codes ($1.0, 2.0, \dots, K$);
   - Automatic generation of `ValueLabels` dictionary (`"1"` $\to$ original value, `"2"` $\to$ original value...);
   - 100% Safe Rust without panics (`unwrap`, `expect`) with comprehensive unit tests for string, numeric, and missing datasets.
2. **L5 Command Bus & Syntax Translation (`crates/statcraft_syntax/src/commands/auto_recode_cmd.rs`):**
   - Strongly-typed `AutoRecodeCommand` (ID `transform.auto_recode`) registered in `CommandBus`;
   - JSON parameter `specs()` (array of `pairs` with `source_var` and `target_var`, `order` option);
   - Creating new numeric variables (`Numeric`, `decimals: 0`, `Measure::Nominal`) with generated `ValueLabels` dictionary or overwriting existing variables;
   - Full `Undo` (`Ctrl+Z`) support via `AutoRecodeUndoItem` history (`Created` / `Overwritten`);
   - Canonical IBM SPSS Statistics syntax generator:
     ```text
     AUTORECODE VARIABLES=country /INTO country_code
       /PRINT.
     EXECUTE.
     ```
     (with `/DESCENDING` subclause for `HighestValue` order).
3. **Zero-Hardcode Localization (`locales/en.ftl`, `locales/ru.ftl`):**
   - Declarative translation keys: `dialog-autorecode-title`, `dialog-autorecode-variables`, `dialog-autorecode-new-name`, `dialog-autorecode-change`, `dialog-autorecode-order-group`, `dialog-autorecode-order-lowest`, `dialog-autorecode-order-highest`, `output-autorecode-executed`.
4. **L6 DialogKit Desktop User Interface (`crates/statcraft_app/src/dialogs/auto_recode_dialog.rs`):**
   - Modal window `AutoRecodeDialog` ($560 \times 430$ px, `DialogShell::standard`);
   - Available variables list via `render_variable_list_panel` on the left and vector `TransferButton` in the center;
   - Right column: `source -> target` pairs list, `GroupBox` with new variable name input and `Change Name` button, `GroupBox` with ordering radio buttons (*Lowest value / Highest value*);
   - Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`).
5. **L6 Application Integration (`app.rs`, `app_dialogs_data.rs`):**
   - Main menu item `Transform -> Recode -> Automatic Recode...` activated (removed `InDev` status);
   - Execution handled in `app_dialogs_data.rs`, logging syntax to `Output Viewer` and emitting `StatusMessage::AutoRecodeExecuted` in the status bar;
   - Added headless regression test `test_headless_render_auto_recode_dialog` in `crates/statcraft_app/tests/dialog_invariants.rs`.
6. **Quality Control & Definition of Done:**
   - 100% workspace tests passing (`cargo test --workspace -j 1`);
   - Full pass across all architectural guards: `layers`, `file-guard`, `design-guard`, `parity`, and `clippy -D warnings`.

---

### Microplan #8: Create Time Series — `transform.create_time_series`
*Time: October 30, 2026, 07:45 (~35 minutes)*

Implemented the canonical time-series dynamic series generator from IBM SPSS Statistics — **Create Time Series...** (`Transform -> Create Time Series...`, command ID `transform.create_time_series`):
1. **Mathematical Core L2 (`statcraft_algo::time_series`):**
   - Implemented `compute_time_series_transform` supporting four primary transformations:
     - `DIFF`: successive differences of order $K$ (degree of differencing $(1-L)^k$);
     - `SDIFF`: seasonal differences with period/lag $S$ ($(1-L^s)$);
     - `SMA`: simple moving average of span $W$ (trailing moving average requiring full window);
     - `CSUM`: cumulative sum with standard SPSS missing value propagation;
   - Safe missing value handling for `RawValue::Null`, non-numbers, and NaNs with 100% Safe Rust without panics;
   - Comprehensive unit tests covering all 4 functions, edge cases, and missing value propagation.
2. **Command Bus L5 (`statcraft_syntax::commands::time_series_cmd`):**
   - Engine command `CreateTimeSeriesCommand` (`transform.create_time_series`) with JSON Schema `specs()`;
   - Generation of authentic SPSS syntax `CREATE diff_sales = DIFF(sales, 1) / sma_sales = SMA(sales, 3) / csum_sales = CSUM(sales). EXECUTE.`;
   - Batch `Undo` support (`SingleVarUndoEntry::Created` and `SingleVarUndoEntry::Overwritten`).
3. **Desktop User Interface L6 (`statcraft_app::dialogs::time_series_dialog`):**
   - Built on canonical DialogKit (`DialogShell::standard`);
   - Left panel for available variables with measure icons, center `TransferButton`, right list of generated series `target = FN(source, order)`, target variable name and label fields, function radio buttons, order/span field, and `Change` button;
   - Bottom Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Main menu item `menu-transform-time-series` updated from `InDev` to `Ready`;
   - Fluent bilingual localization (`dialog-timeseries-*` in `locales/ru.ftl` and `locales/en.ftl`).
4. **Quality Control & Definition of Done:**
   - 100% Safe Rust, 0 panics;
   - `cargo run -p xtask -j 1 -- layers` — 0 violations;
   - `cargo run -p xtask -j 1 -- file-guard` — all 232 files within size limits;
   - `cargo run -p xtask -j 1 -- design-guard` — all dialogs strictly conform to DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — 47 commands registered;
   - `cargo test --workspace -j 1` — 100% passing tests across all crates;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 warnings.

---

### Microplan #6: Rank Cases — `transform.rank_cases`
*Time: October 30, 2026, 07:50 (~35 minutes)*

Implemented the canonical case ranking and normal scores procedure from IBM SPSS Statistics — **Rank Cases...** (`Transform -> Rank Cases...`, command ID `transform.rank_cases`):
1. **Mathematical Engine L2 (`statcraft_algo::rank_cases` and `rank_cases_tests`):**
   - Implemented `compute_ranks` function with complete tie resolution support (`TiesHandling`):
     - `Mean`: average rank of tied cases (SPSS default);
     - `Low`: lowest rank in the tie cluster;
     - `High`: highest rank in the tie cluster;
     - `Seq`: sequential ranks without tie formation;
   - Direction selection `RankDirection` (`Ascending` assigning rank 1 to the smallest value, `Descending` assigning rank 1 to the largest value);
   - Comprehensive spectrum of rank transformation types `RankType`:
     - `SimpleRank`: standard integers / midranks;
     - `FractionalRank`: fractional ranks $r / N$;
     - `FractionalPercent` / `Percentiles`: percentile ranks $(r / N) \times 100$;
     - `Ntiles(k)`: quantiles calculated via $\lceil (r \cdot k) / N \rceil$ (default 4 for quartiles);
     - Normal scores: Blom $((r - 3/8) / (w + 1/4))$, Tukey $((r - 1/3) / (w + 1/3))$, Rankit $((r - 1/2) / w)$, Van der Waerden $(r / (w + 1))$;
   - Clean implementation of standard normal inverse CDF `inv_normal_cdf` (Peter J. Acklam rational approximation with error $< 10^{-9}$);
   - 100% Safe Rust, `RawValue::Null` missing cases preserved at original indices.
2. **Command Bus L5 (`statcraft_syntax::commands::rank_cases_cmd`):**
   - Engine command `RankCasesCommand` (`transform.rank_cases`) with parameter `specs()`;
   - Batch creation and overwriting of numeric target variables with `Measure::Ordinal` and prefix (default `R_`);
   - Authentic SPSS syntax generation `RANK VARIABLES=... /TIES=... /RANK ... \n EXECUTE.`;
   - Comprehensive batch `Undo` support (`Ctrl+Z`) restoring previous columns and metadata.
3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::rank_cases_dialog`):**
   - Canonical DialogKit standard window (`DialogShell::standard`);
   - Left `Variables:` list, center `TransferButton`, target variables list;
   - `GroupBox` containers for rank direction (*Smallest value / Largest value*) and ties handling (*Mean / Low / High*);
   - Right `SubdialogSidebar` with `Rank Types...` trigger opening a dedicated subdialog (`DialogShell::subdialog`) for rank types and Ntiles groups configuration;
   - Canonical Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Main menu item `menu-transform-rank-cases` updated from `InDev` to `Ready`;
   - Bilingual Fluent localization (`dialog-rank-*` and `output-rank-executed` in `ru.ftl` and `en.ftl`).
4. **Quality Control & Definition of Done:**
   - 100% Safe Rust, 0 panics;
   - `cargo run -p xtask -j 1 -- layers` — 0 violations;
   - `cargo run -p xtask -j 1 -- file-guard` — all 232 files within size limits;
   - `cargo run -p xtask -j 1 -- design-guard` — all dialogs strictly conform to DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — parity logged;
   - `cargo test --workspace -j 1` — 100% passing tests across all crates.

---

### Microplan #9: Visual Binning — `transform.visual_binning`
*Time: October 30, 2026, 08:45 (~35 minutes)*

Implemented a fundamental IBM SPSS Statistics data preparation procedure — **Visual Binning...** (`Transform -> Visual Binning...`, command ID `transform.visual_binning`):
1. **Mathematical Engine L2 (`statcraft_algo::binning`):**
   - Continuous variable discretization algorithm `compute_visual_binning` and `compute_visual_binning_with_labels` based on a list of `Cutpoint` specifications (upper bound `upper_bound` and inclusion boundary `inclusive`);
   - Automated value labels generation according to SPSS conventions (`"<= 25000"`, `"25000 - 50000"`, `"> 50000"`);
   - Cutpoint generators for three canonical discretization strategies:
     - `generate_equal_width_cutpoints`: equal-width intervals with width $W = (max - min) / N$;
     - `generate_equal_percentile_cutpoints`: equal-percentile quantile binning (equal number of observations per bin) using the SPSS HAVERAGE percentile method;
     - `generate_stddev_cutpoints`: standard deviation intervals at $\pm 1\sigma, \pm 2\sigma$ around sample mean $\mu$;
   - 100% Safe Rust without `unwrap` in runtime code, safe preservation of missing cases `RawValue::Null`, and unit tests covering all methods and generators.
2. **Command Bus L5 (`statcraft_syntax::commands::binning_cmd`):**
   - Engine command `VisualBinningCommand` (`transform.visual_binning`) with JSON Schema `specs()`;
   - Creation and overwriting of numeric target variables with `Measure::Ordinal`, `DataType::Numeric { decimals: 0 }`, and attached `ValueLabels` dictionary;
   - Canonical SPSS syntax generation:
     ```text
     RECODE salary (MISSING=COPY) (LO THRU 25000 = 1) (LO THRU 50000 = 2) (ELSE = 3)
       INTO salary_binned.
     VARIABLE LABELS salary_binned 'Salary (Binned)'.
     VALUE LABELS salary_binned 1 '<= 25000' 2 '25000 - 50000' 3 '> 50000'.
     EXECUTE.
     ```
   - Comprehensive `Undo` support (`Ctrl+Z`).
3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::binning_dialog` and `binning_cutpoints_dialog`):**
   - Canonical DialogKit modal dialog `VisualBinningDialog` (`DialogShell::standard`);
   - Left-hand variable picker `render_variable_list_panel`;
   - Right-hand control area: target variable name and label, upper endpoint inclusion radio buttons (*Included (<=) / Excluded (<)*), `Make Cutpoints...` subdialog trigger, manual cutpoint addition, and interactive `Cutpoint Grid` table with editable labels;
   - Subdialog `binning_cutpoints_dialog` (`DialogShell::subdialog`, `SubdialogActionBar`) for method selection and number of cutpoints;
   - Canonical Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Activated main menu item `menu-transform-visual-binning` (status `InDev` removed), integration in `app_dialogs_data.rs`, status bar feedback, and syntax logging to `Output Viewer`.
4. **Quality Control & Definition of Done:**
   - 100% Safe Rust, 0 panics;
   - `cargo run -p xtask -j 1 -- layers` — 0 violations;
   - `cargo run -p xtask -j 1 -- file-guard` — all 232 files within size limits;
   - `cargo run -p xtask -j 1 -- design-guard` — all dialogs strictly conform to DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — command registered;
   - `cargo test --workspace -j 1` — 100% passing tests across all crates;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 warnings.

---

### Microplan #7: Replace Missing Values (RMV) — `transform.replace_missing`
*Time: October 30, 2026, 09:15 (~30 minutes)*

Implemented the canonical IBM SPSS Statistics missing data imputation procedure — **Replace Missing Values...** (`Transform -> Replace Missing Values...`, command ID `transform.replace_missing`):
1. **Mathematical Engine L2 (`statcraft_algo::replace_missing`):**
   - Implemented `impute_missing_values` supporting all five canonical SPSS imputation methods (`ImputationMethod`):
     - `SeriesMean` (SMEAN): imputes missing values with the mean of all valid cases in the series;
     - `MeanOfNearby(span)` (MEAN): mean of valid nearby values within window $[i - \text{span} .. i + \text{span}]$;
     - `MedianOfNearby(span)` (MEDIAN): median of valid nearby values within window $[i - \text{span} .. i + \text{span}]$;
     - `LinearInterpolation` (LINEAR): linear interpolation between preceding valid $(i_0, y_0)$ and succeeding valid $(i_1, y_1)$ values, with endpoint nearest-neighbor carry for edge missing cases;
     - `LinearTrendAtPoint` (TREND): linear regression trend $y = a + b \cdot x$ using case observation index $x_i = i + 1$;
   - Robust Never Crash protection: safe handling of edge missing cases, series consisting entirely of missing values, empty or single-element series using 100% safe Rust.
2. **Command Bus L5 (`statcraft_syntax::commands::replace_missing_cmd`):**
   - Engine command `ReplaceMissingCommand` (`transform.replace_missing`) with JSON Schema `specs()` for AI agents;
   - Appending new numeric columns (`DataType::Numeric { decimals: 2 }`, `Measure::Scale`) or replacing existing columns;
   - Canonical SPSS syntax generation:
     ```sps
     RMV new_salary = SMEAN(salary)
       /new_sales = MEDIAN(sales, 2)
       /new_temp = LINEAR(temp).
     EXECUTE.
     ```
   - Comprehensive batch `Undo` support (`Ctrl+Z`) restoring original columns, metadata, and values.
3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::replace_missing_dialog`):**
   - Modal window designed according to DialogKit standards (`DialogShell::standard`, $560 \times 430$ px);
   - Left-hand source variable selection panel, central `TransferButton`, right-hand list of replacement rules `target -> METHOD(source)`;
   - Variable name editing field with `Change` button, `GroupBox` method selection radio buttons, and integer `span` input for nearby point methods;
   - Canonical Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Main menu item `Transform -> Replace Missing Values...` activated (status `[dev]` removed);
   - Bilingual Fluent localization (`dialog-missing-*`, `output-missing-executed`, `status-missing-executed` in `ru.ftl` and `en.ftl`).
4. **Quality Control & Definition of Done:**
   - 100% Safe Rust, 0 panics, 0 `unwrap` in production code;
   - `cargo run -p xtask -j 1 -- layers` — 0 violations;
   - `cargo run -p xtask -j 1 -- file-guard` — all 232 files strictly within line limits;
   - `cargo run -p xtask -j 1 -- design-guard` — all dialogs strictly conform to DialogKit;
   - `cargo run -p xtask -j 1 -- parity` — command registered in parity registry;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 warnings;
   - `cargo test --workspace -j 1` — 100% passing tests across the entire workspace.






### Microplan #10: Date and Time Wizard — `transform.date_time_wizard`
*Time: October 30, 2026, 10:00 (~30 minutes)*

Implemented the canonical IBM SPSS Statistics date and time manipulation facility — **Date and Time Wizard...** (`Transform -> Date and Time Wizard...`, command ID `transform.date_time_wizard`):

1. **Mathematical Core L2 (`statcraft_algo::date_time`):**
   - Module `crates/statcraft_algo/src/date_time.rs` (393 lines, strictly within the $\le 500$ lines limit);
   - Safe date and timestamp parsing without panics: supports ISO 8601, RFC 3339, `YYYY-MM-DD`, `DD.MM.YYYY`, `MM/DD/YYYY`, `YYYY/MM/DD`, formats with timestamps `HH:MM:SS`, and numeric unix timestamps or epochs;
   - Operation enum `DateTimeOperation`:
     - `ExtractPart(DatePart)`: extracts year (`Year`), month (`Month`), day (`Day`), hour (`Hour`), minute (`Minute`), second (`Second`), day of week (`DayOfWeek`, 1 = Sunday per SPSS standard), and quarter (`Quarter`);
     - `Difference { unit: DateUnit }`: computes exact calendar intervals between two dates in years, months, weeks, days, hours, minutes, or seconds;
     - `AddDuration { amount, unit }`: adds or subtracts time durations with automated leap-year adjustment (e.g., Jan 31 + 1 month = Feb 29 in a leap year);
     - `ConstructDate`: normalizes string or numeric data into standard canonical date format;
   - Full support for missing values (`RawValue::Null`) and unparseable input with null preservation and zero crashes.

2. **Command Bus L5 (`statcraft_syntax::commands::date_time_cmd`):**
   - Engine command `DateTimeWizardCommand` (`transform.date_time_wizard`) with JSON Schema `specs()` for MCP and AI agents;
   - Adds new columns with `Measure::Scale` and `DataType::Numeric` (for extracted parts and differences) or `DataType::Date` (for date outcomes);
   - Canonical SPSS syntax generation:
     ```sps
     COMPUTE birth_year = XDATE.YEAR(birth_date).
     COMPUTE age_years = DATEDIFF(date2, date1, "years").
     COMPUTE due_date = DATESUM(start_date, 30, "days").
     VARIABLE LABELS target "Result Label".
     EXECUTE.
     ```
   - Comprehensive batch `Undo` support (`Ctrl+Z`) restoring original columns and dictionary metadata.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::date_time_dialog`):**
   - Modal dialog constructed with `DialogKit` (`DialogShell::standard`, $560 \times 430$ px);
   - Header `GroupBox` for selecting the wizard task (Extract part / Calculate interval between two dates / Add or subtract duration / Convert to standard date);
   - Left-hand available variable panel `render_variable_list_panel`, adaptive `TransferButton`, slots for primary and secondary dates;
   - Dropdown selectors `ComboBox::from_id_salt` for extracted parts and time units, `DragValue` counter for duration amount, text inputs for target variable name and label;
   - Bottom Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Main menu item `Transform -> Date and Time Wizard...` activated (status `[dev]` removed);
   - Synchronous bilingual Fluent localization (`dialog-datetime-*`, `status-datetime-executed` in `locales/en.ftl` and `locales/ru.ftl`).

4. **Quality Control & Definition of Done:**
   - 100% Safe Rust, 0 panics, 0 `unwrap()` calls in production code;
   - `cargo run -p xtask -j 1 -- layers` — 0 architectural violations;
   - `cargo run -p xtask -j 1 -- file-guard` — all 237 files strictly within line limits;
   - `cargo run -p xtask -j 1 -- design-guard` — 100% DialogKit compliance;
   - `cargo run -p xtask -j 1 -- parity` — command registered in parity checklist;
   - `cargo test --workspace -j 1` — 100% passing test suite across all workspace crates and invariant tests.

---

### Microplan #11: Optimal Binning — `transform.optimal_binning`
*Time: October 30, 2026, 10:35 (~35 minutes)*

Implemented the supervised discretization procedure of continuous attributes from IBM SPSS Statistics — **Optimal Binning...** (`Transform -> Optimal Binning...`, command ID `transform.optimal_binning`):

1. **Algorithmic Core L2 (`statcraft_algo::optimal_binning`):**
   - New module `crates/statcraft_algo/src/optimal_binning.rs` (289 lines, strictly $\le 500$ lines);
   - MDLP Discretization Algorithm (Minimum Description Length Principle, Fayyad & Irani 1993):
     * Sorting of $(x_i, y_i)$ pairs by continuous feature $x$;
     * Entropy and Information Gain optimization: $\Delta Ent = Ent(S) - \frac{|S_1|}{|S|}Ent(S_1) - \frac{|S_2|}{|S|}Ent(S_2)$;
     * Minimum Description Length stopping rule: candidate cutpoint is accepted only when Information Gain exceeds model complexity threshold $\frac{\log_2(N - 1) + \Delta}{N}$;
     * Global Priority Queue to iteratively split intervals with maximal global gain while strictly enforcing `max_bins` (default 5, min 2);
     * Robust Never Crash handling of constant features, single-class targets, and missing values (`Null`);
     * Automated interval boundary labeling (`<= bound`, `prev - curr`, `> bound`) and `ValueLabels` dictionary generation.

2. **Engine Command Bus L5 (`statcraft_syntax::commands::optimal_binning_cmd`):**
   - Typed engine command `OptimalBinningCommand` (`transform.optimal_binning`) with JSON Schema `specs()` for MCP and AI agents;
   - Creation of new numeric columns with `Measure::Ordinal` and 0 decimals, attaching generated category labels `ValueLabels`;
   - Canonical IBM SPSS Statistics syntax generator:
     ```sps
     OPTIMAL BINNING
       /VARIABLES = income age BY default_status
       /CRITERIA METHOD = MDLP.
     EXECUTE.
     ```
   - Full batch `Undo` support (`Ctrl+Z`) removing generated columns in reverse index order.

3. **DialogKit Desktop User Interface L6 (`statcraft_app::dialogs::optimal_binning_dialog`):**
   - Modal dialog constructed with `DialogKit` (`DialogShell::standard`, $560 \times 430$ px);
   - Left-hand source variable picker `render_variable_list_panel`;
   - Vector adaptive `TransferButton` buttons for variables to bin and the categorical guide variable;
   - Right-hand panel: variables list, guide variable slot with clear button, settings `GroupBox` with `max_bins` counter (2..20) and target prefix input (`OB_`);
   - Canonical Type 1 action bar `DialogActionBar` (`[ OK ] [ Paste ] [ Reset ] [ Cancel ] [ Help ]`);
   - Main menu item `Transform -> Optimal Binning...` activated (status `[dev]` removed);
   - Handled in `app_dialogs_transform.rs`, status message `StatusMessage::OptimalBinningExecuted`, and log recording in `Output Viewer`.

4. **Architectural Decomposition for File Size Guard:**
   - Decomposed `StatCraftApp` menu rendering into an isolated package `crates/statcraft_app/src/app_menus/` (`mod.rs`, `file_edit_view.rs`, `data_transform.rs`, `analyze_etc.rs`), reducing `app.rs` from 1557 to 585 lines (below the 1500 ceiling);
   - Decomposed `date_time_dialog` into `date_time_dialog/` (`mod.rs`, `view.rs`), satisfying the $\le 500$ lines limit.

5. **Quality Control & Definition of Done:**
   - 100% Safe Rust, 0 panics and 0 unwraps in application code;
   - `cargo run -p xtask -j 1 -- layers` — 0 violations;
   - `cargo run -p xtask -j 1 -- file-guard` — all 245 files strictly within line limits;
   - `cargo run -p xtask -j 1 -- design-guard` — 100% DialogKit compliance;
   - `cargo run -p xtask -j 1 -- parity` — 51 documented commands;
   - `cargo clippy --workspace -j 1 -- -D warnings` — 0 warnings;
   - `cargo test --workspace -j 1` — 100% passing test suite across the entire workspace.

---

### Version 0.2.0: Multi-Platform Binary Builds Across All OSes (Windows, Linux, macOS)
*Time: October 30, 2026, 14:15 (~15 minutes)*

Released version **0.2.0** and configured an automated CI/CD pipeline for building desktop release distributions across all key platforms:

1. **Ecosystem Version Bump to 0.2.0:**
   - Updated workspace root `Cargo.toml`: `version = "0.2.0"`;
   - Updated crates `statcraft_reporting` and `xtask` to inherit `version.workspace = true`;
   - Synchronized package metadata across all workspace members.

2. **Cross-Platform GitHub Actions Release Workflow (`.github/workflows/release.yml`):**
   - Configured matrix builds for both portable archives and native system installers across all target environments:
     * **Windows x64** (`windows-latest`, `x86_64-pc-windows-msvc`): portable archive `statcraft-windows-x64.zip` and full Inno Setup installer `statcraft-windows-x64-installer.exe` (`packaging/windows/installer.iss`);
     * **Linux x64** (`ubuntu-22.04`, `x86_64-unknown-linux-gnu`): portable archive `statcraft-linux-x64.tar.gz` and Debian/Ubuntu installer package `statcraft-linux-x64.deb` with application menu integration (`packaging/linux/statcraft.desktop`);
     * **macOS Apple Silicon** (`macos-latest`, `aarch64-apple-darwin`): portable archive `statcraft-macos-arm64.tar.gz` and mountable DMG image `statcraft-macos-arm64.dmg` with `StatCraft.app` bundle (`packaging/macos/Info.plist`);
     * **macOS Intel** (`macos-13`, `x86_64-apple-darwin`): portable archive `statcraft-macos-x64.tar.gz` and mountable DMG image `statcraft-macos-x64.dmg`;
   - All bundles are packaged with external Fluent translation catalogs (`locales/`), documentation (`README.md`, `README.ru.md`), and license `LICENSE-MIT`;
   - Enhanced `statcraft_i18n` with automatic runtime locale discovery for Linux system paths (`/usr/share/statcraft/locales`) and macOS app bundles (`Contents/Resources/locales`);
   - Automated GitHub Release publication using `softprops/action-gh-release@v2` triggered by `v*` tag push or manual `workflow_dispatch`, attaching all 8 distribution packages in a single step without race conditions.

3. **Quality Control (Definition of Done):**
   - Strict adherence to Resource Guard (`-j 1` across all compilation steps);
   - 100% passing local CI quality gate (`cargo run -p xtask -j 1 -- ci`).


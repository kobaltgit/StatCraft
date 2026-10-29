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

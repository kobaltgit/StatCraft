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

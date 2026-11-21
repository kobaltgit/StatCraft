# StatCraft v0.4.4

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.4 is a dedicated stability and user experience release addressing OS-level window frame composition on Windows 10 and safeguarding statistical dialog layouts against label bloating.

### 🪟 Windows 10 Borderless Window Artifacts & Cursor Precision (BUG-017)
* **Zero Black Bars & Accurate Hit-Testing:**
  * Fixed an issue on Windows 10 where the desktop compositor (DWM) reserved unpainted non-client caption and border areas (`WS_CAPTION | WS_BORDER`), causing a ~26 px black bar across the top of the window and vertically offsetting mouse interaction coordinates.
  * Integrated safe Win32 API wrapper `winsafe = "0.0.29"` in `crates/statcraft_app` (100% Safe Rust without `unsafe` blocks in application code).
  * Implemented dedicated window frame guard `crates/statcraft_app/src/window_frame.rs` based on the proven PhotoCraft canon:
    * `strip(hwnd)`: dynamically removes unwanted caption and border styles on each frame and triggers `SWP_FRAMECHANGED` recalculation.
    * `keep_clear_of_the_taskbar(hwnd)`: ensures maximized custom-frame windows properly respect desktop work areas, supporting both standard and auto-hiding taskbars without under-clipping.
  * Preserves 100% of StatCraft's authentic Client-Side Decoration (CSD) appearance while restoring pixel-perfect 1:1 mouse tracking.
  * Verified through 7 automated unit tests.

### 🛡️ Modal Dialog Layout Protection against Long Variable Labels (BUG-017)
* **Strict Dialog Boundary Guard:**
  * Updated `DialogShell::show` in `crates/statcraft_app/src/dialogs/common/dialog_shell.rs` to enforce `.max_size(size)` and `.constrain_to(ctx.screen_rect())`.
  * Prevents statistical procedure dialogs from stretching beyond canonical dimensions ($560 \times 430$ px) regardless of variable metadata lengths.
* **Smart Variable Picker Truncation:**
  * In `crates/statcraft_app/src/dialogs/common/variable_picker.rs`, constrained `ScrollArea::vertical().max_width(inner_size.x)`.
  * Selectable variable items now cleanly truncate with ellipsis (`.truncate()`), while displaying the complete survey question text in an on-hover tooltip (`.on_hover_text(...)`).
  * Action buttons (`[ OK ]`, `[ Paste ]`, `[ Reset ]`, `[ Cancel ]`) remain firmly anchored on-screen even on survey datasets with 70–90+ character question labels.

### 📋 Full Changelog for v0.4.4
```markdown
## [0.4.4] - 2026-11-22

### Fixed
- **Windows 10 Borderless Window Artifacts & Mouse Cursor Offset (BUG-017):**
  - Integrated safe Win32 API wrapper `winsafe = "0.0.29"` in `statcraft_app` (100% Safe Rust without `unsafe` blocks in application code).
  - Implemented window frame guard `crates/statcraft_app/src/window_frame.rs` following the PhotoCraft canon.
  - Dynamically strips `WS_CAPTION | WS_BORDER` styles forced by `winit` on Windows 10 and forces frame recalculation (`SWP_FRAMECHANGED`) on every frame, eliminating the 26 px black bar at the top and side border artifacts.
  - Restored 1:1 mouse hit-test precision across menus, tabs, and controls without vertical offset.
  - Added `keep_clear_of_the_taskbar` to prevent maximized custom-frame windows from clipping behind or over the Windows taskbar, with support for both standard and auto-hiding taskbars.
- **Modal Dialog Bloating Protection on Long Variable Labels (BUG-017):**
  - Added `.max_size(size)` and `.constrain_to(ctx.screen_rect())` to `DialogShell::show`, strictly preventing procedure dialogs from stretching beyond screen boundaries.
  - Constrained `ScrollArea::vertical().max_width(inner_size.x)` in `variable_picker.rs`.
  - Added `.truncate()` to selectable list item buttons and attached full variable descriptions via `.on_hover_text(...)` tooltips so long survey questions (70–90+ characters) never push action buttons off-screen.

### Added
- **Window Frame Unit Tests (`window_frame.rs`):**
  - Added 7 unit tests covering borderless style bitmasks, taskbar clamping on all 4 monitor edges, auto-hiding taskbar 1-pixel reservation, and monitor edge detection.
```

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`statcraft-windows-x64-installer.exe`) + Portable ZIP archive (`statcraft-windows-x64.zip`).
* **Linux (x64):** Debian package (`statcraft_0.4.4_amd64.deb`) + Portable TAR.GZ archive (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon ARM64 & Intel x64):** DMG Disk Images + Portable TAR.GZ Application Bundles (`.app`).

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.4 посвящён повышению стабильности и эргономики интерфейса: устранению системных дефектов отрисовки безрамочного окна на Windows 10 и защите модальных диалогов от раздутия длинными метками переменных.

### 🪟 Устранение артефактов безрамочного окна и точность курсора на Windows 10 (BUG-017)
* **Ликвидация черных полос и смещения кликов:**
  * Устранена проблема Windows 10, при которой оконный менеджер DWM резервировал неокрашиваемую неклиентскую область `WS_CAPTION | WS_BORDER`, создавая черную полосу ~26 px сверху окна и сдвигая вертикальные координаты курсора мыши.
  * Подключена безопасная типобезопасная обертка системных вызовов `winsafe = "0.0.29"` в `crates/statcraft_app` (100% Safe Rust без использования ключевого слова `unsafe`).
  * Реализован модуль стража окна `crates/statcraft_app/src/window_frame.rs` по канону PhotoCraft:
    * `strip(hwnd)`: динамически снимает стили заголовка и границы при каждом кадре и инициирует пересчет области окна `SWP_FRAMECHANGED`.
    * `keep_clear_of_the_taskbar(hwnd)`: предотвращает заезд развернутого окна на панель задач Windows с корректной поддержкой обычных и автоматически скрываемых таскбаров.
  * Сохранено 100% оригинального внешнего вида Client-Side Decoration (CSD) и восстановлена точность хит-теста мыши 1:1.
  * Покрыто 7 модульными тестами геометрии и стилей окна.

### 🛡️ Защита модальных окон DialogKit от раздутия длинными строками (BUG-017)
* **Жесткий контроль границ диалогов:**
  * В `DialogShell::show` добавлены ограничения `.max_size(size)` и `.constrain_to(ctx.screen_rect())`.
  * Диалоговые окна статистических процедур гарантированно не растягиваются шире установленных канонических размеров ($560 \times 430$ px) и никогда не выходят за границы монитора.
* **Умное усечение в списках переменных:**
  * В `variable_picker.rs` контейнеру `ScrollArea::vertical()` задано ограничение `.max_width(inner_size.x)`.
  * Элементы списков переменных автоматически усекаются с многоточием (`.truncate()`), а полный текст вопроса анкеты выводится во всплывающем тултипе при наведении (`.on_hover_text(...)`).
  * Кнопки действий (`[ OK ]`, `[ Вставить ]`, `[ Сброс ]`, `[ Отмена ]`) всегда остаются на виду даже на анкетах с вопросами по 70–90+ символов.

### 📋 Полный журнал изменений (Changelog v0.4.4)
```markdown
## [0.4.4] - 2026-11-22

### Исправлено
- **Артефакты безрамочного окна и сдвиг курсора мыши на Windows 10 (BUG-017):**
  - Интегрирована безопасная обертка Win32 API `winsafe = "0.0.29"` (100% Safe Rust без `unsafe`).
  - Реализован страж окна `window_frame.rs` со снятием `WS_CAPTION | WS_BORDER` и защитой от перекрытия панели задач при максимизации.
  - Устранена черная полоса 26 px сверху и восстановлен точный хит-тест мыши 1:1.
- **Защита модальных диалогов от раздутия длинными именами (BUG-017):**
  - Ограничения `max_size` и `constrain_to` в `DialogShell`.
  - Усечение текста элементов списков `VariablePicker` с тултипом полного названия при наведении.
```

### 📦 Артефакты сборки
* **Windows (x64):** Установщик Inno Setup (`statcraft-windows-x64-installer.exe`) + Портативный ZIP-архив (`statcraft-windows-x64.zip`).
* **Linux (x64):** Пакет Debian (`statcraft_0.4.4_amd64.deb`) + Портативный TAR.GZ (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon ARM64 & Intel x64):** Образы дисков DMG + Пакеты приложений (`.app`) в TAR.GZ.

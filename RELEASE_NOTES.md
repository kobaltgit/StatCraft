# StatCraft v0.4.10

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.10 introduces **TURF (Total Unduplicated Reach and Frequency) Analysis** (`SPSSINC TURF`, Features 034–036) for market research and assortment optimization, along with a global static **`i18n-guard`** CI linter and Output Viewer variable label improvements.

### 📊 TURF Analysis (`SPSSINC TURF`, Features 034–036)
* **Combinatorial Reach & Frequency Engine:**
  * Packed `Vec<u64>` respondent bitset evaluation for $k = 1 \dots K_{\max}$ combinations (`BESTN`), computing **Unduplicated Reach** (count & `% of Cases`), **Frequency** (count, `% of Responses`, and **Mean Responses** per reached case).
  * Respects active dataset case weights (`WEIGHT BY`) and case filters (`SELECT IF` / `filter_$`).
  * Supports positive response criteria (`CRITERIA`), minimum positive percentage screening (`MINPERCENT`), always required items (`FORCE`), iterative `REMOVALCYCLES`, `MAXSETS` checked `u128` operation guard, **Importance Weights** (`/IMPORTANCE STRICT`), and **Tournament Heats** (`/HEATS USEHEATS`, `HEATSIZE`, `NUMWINNERS`) for large variable pools.
* **DialogKit UI & Canonical `.sps` Syntax:**
  * Added **Analyze > Descriptive Statistics > TURF Analysis...** (`TurfDialog`) with **Importance Weights...** and **Advanced Selection...** subdialogs, plus full `SPSSINC TURF` / `TURF` syntax round-trip.
* **Output Viewer & Export:**
  * Case & Parameter Summary table, Reach & Frequency Pivot Tables per cycle and size with human-readable variable `Label`s, vector Reach & Frequency curves, and TSV/HTML/TXT export.

### 🛡️ Global i18n Guard & Output Viewer Polish
* **Global `xtask i18n-guard`:** Integrated a static UI localization linter into `cargo xtask ci` to permanently prevent hardcoded strings in dialog widgets and Output Viewer table headers.
* **Fixes:** Corrected TURF `% of Responses` calculation (ratio to total positive responses), localized all TURF/Explore table headers, and fixed `TurfDialog` sidebar button wrapping.

### 📋 Full Changelog for v0.4.10
```markdown
## [0.4.10] - 2026-11-26

### Added
- **TURF Analysis (Features 034–036):** Clean-room combinatorial TURF engine (`stat.turf`), Importance Weights, Tournament Heats, Removal Cycles, DialogKit UI + subdialogs, `SPSSINC TURF` .sps round-trip, Pivot Tables with variable labels, Reach & Frequency chart, and TSV/HTML/TXT export.
- **Global Static i18n Linter (`xtask i18n-guard`):** Enforces zero hardcoded UI strings and table headers across all dialogs and views in CI.

### Fixed
- **TURF & Output Viewer:** Fixed `% of Responses` denominator, localized all TURF and Explore report table headers, and fixed `TurfDialog` right sidebar layout.
```

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.10 добавляет **TURF-анализ** (*Total Unduplicated Reach and Frequency*, `SPSSINC TURF`, Features 034–036) для маркетинговых исследований и оптимизации продуктовых линеек, а также глобальный статический линтер локализации **`i18n-guard`** и отображение меток переменных в выводе TURF.

### 📊 TURF-анализ (`SPSSINC TURF`, Features 034–036)
* **Комбинаторный движок охвата и частоты:**
  * Быстрый битовый расчёт на `Vec<u64>` для комбинаций размера $k = 1 \dots K_{\max}$ (`BESTN`): **Недублированный охват** (количество и `% наблюдений`), **Частота** (количество, `% ответов` и **Среднее ответов** на достигнутого респондента).
  * Учёт весов наблюдений (`WEIGHT BY`) и фильтра (`filter_$`), критериев положительных ответов (`CRITERIA`), порога отсечения (`MINPERCENT`), обязательных переменных (`FORCE`), циклов исключения (`REMOVALCYCLES`), лимита операций (`MAXSETS`), **Весов важности** (`/IMPORTANCE STRICT`) и эвристики **Расширенного выбора / Заездов** (`/HEATS`).
* **Интерфейс `DialogKit` и синтаксис `.sps`:**
  * Пункт меню **Анализ > Описательные статистики > TURF-анализ...** с поддиалогами **Веса...** и **Расширенный выбор...**, а также полный round-trip синтаксиса `SPSSINC TURF` / `TURF`.
* **Просмотр вывода и экспорт:**
  * Таблица сводки параметров, сводные таблицы «Охват и частота» по циклам и размерам с подстановкой текстовых меток переменных из словаря, векторный график охвата и частоты, экспорт в TSV/HTML/TXT.

### 🛡️ Глобальный запрет сырых строк (`i18n-guard`) и исправления
* **Статический линтер `xtask i18n-guard`:** Встроен в `cargo xtask ci` и pre-push hook — автоматически блокирует любые нелокализованные заголовки таблиц и строки интерфейса в `dialogs/` и `views/`.
* **Исправления:** Исправлен расчёт `% ответов` (доля от общего числа положительных ответов), локализованы все заголовки таблиц TURF и Explore, исправлена ширина правой колонки кнопок в диалоге `TURF-анализ`.

### 📋 Полный список изменений v0.4.10
```markdown
## [0.4.10] - 2026-11-26

### Added
- **TURF Analysis (Features 034–036):** Чистый комбинаторный движок TURF (`stat.turf`), веса важности, заезды (Heats), циклы исключения, диалог и поддиалоги на `DialogKit`, синтаксис `SPSSINC TURF`, сводные таблицы с метками переменных, векторный график и экспорт TSV/HTML/TXT.
- **Global Static i18n Linter (`xtask i18n-guard`):** Автоматический запрет сырых строк интерфейса и заголовков таблиц в `ci`.

### Fixed
- **TURF & Output Viewer:** Исправлен знаменатель `% ответов`, локализованы заголовки таблиц TURF и Explore, исправлена верстка правой колонки в `TurfDialog`.
```

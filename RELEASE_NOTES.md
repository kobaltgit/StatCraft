# StatCraft v0.4.9

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.4.9 adds two new robust/errors-in-variables regression procedures requested by the community (GitHub Issue #4), **Linux AppImage** packaging, and important Output Viewer & i18n polish.

### 📈 New Regression Procedures (Features 032–033)
* **Theil–Sen Robust Linear Regression (`THEIL SEN`):**
  * Non-parametric outlier-resistant simple linear regression (~29.3% breakdown point) with Sen (1968) tie-corrected confidence intervals, Mann–Kendall trend test ($\tau_b, Z, p$), MAD of residuals, and `MAX_EXACT_PAIRS_N = 5000` safety guard.
  * Full `DialogKit` UI with 5 applicable subdialogs (`Options...`, `Plots...`, `Save...`, `Style...`, `Bootstrap...`), `PRE_n`/`RES_n` dataset saving with `Undo`, vector scatter & residual plots, and `.sps` syntax round-trip.
* **Orthogonal / Deming Regression (`ORTHOGONAL REGRESSION`):**
  * Closed-form errors-in-variables regression supporting **Orthogonal (Total Least Squares, $\lambda = 1.0$)** and general **Deming Regression** ($\lambda = \sigma_y^2 / \sigma_x^2 > 0$).
  * Leave-One-Out **Jackknife** standard errors and Student-$t$ confidence intervals ($df = N - 2$), orthogonal residual SD, Pearson's $r$, `PRE_n`/`RES_n` saving with `Undo`, scatter plot with clipped $Y = X$ identity line, `Bootstrap`, and `.sps` syntax round-trip.

### 🐧 Linux AppImage & Output Viewer Polish
* **Linux AppImage:** Added `statcraft-linux-x64.AppImage` to the automated release pipeline alongside `.deb` and `.tar.gz`.
* **Output Viewer & i18n:**
  * Localized the `Notes` metadata table (`Output Created`, `Active Dataset`, `N of Active Cases`, `N of Variables`, `Session`).
  * Display human-readable variable `Label`s in regression tables and plot axes when defined.
  * Fixed $Y = X$ identity line scaling in Deming scatter plots, added axis padding & Y-axis numeric ticks, normalized `-0.0000` formatting, and auto-sized table columns to fit long variable labels.

### 📋 Full Changelog for v0.4.9
```markdown
## [0.4.9] - 2026-11-25

### Added
- **Theil–Sen Estimator (Feature 032):** Robust simple linear regression, Mann-Kendall trend test, Bootstrap, PRE/RES saving, scatter/residual plots, and THEIL SEN .sps syntax.
- **Orthogonal / Deming Regression (Feature 033):** Closed-form Deming (lambda = 1.0 & custom lambda > 0), Jackknife SE & t-CI, Bootstrap, PRE/RES saving, scatter plot with Y=X line, and ORTHOGONAL REGRESSION .sps syntax.
- **Linux AppImage:** Automated `statcraft-linux-x64.AppImage` build in GitHub Releases.

### Fixed
- **Output Viewer & i18n:** Translated hardcoded Notes labels, fixed Deming Y=X plot scaling, added axis padding & Y-ticks, eliminated `-0.0000`, auto-fitted table column widths, and rendered variable Labels.
```

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.4.9 добавляет две новые регрессионные процедуры по запросу сообщества (GitHub Issue #4), сборку **Linux AppImage**, а также важные улучшения Output Viewer и локализации.

### 📈 Новые регрессионные процедуры (Features 032–033)
* **Робастная регрессия Тейла — Сена (`THEIL SEN`):**
  * Непараметрическая, устойчивая к выбросам простая линейная регрессия (точка срыва ~29.3%) с ДИ по методу Sen (1968) с поправкой на связки, критерием тренда Манна — Кендалла ($\tau_b, Z, p$), MAD остатков и защитой `MAX_EXACT_PAIRS_N = 5000`.
  * Интерфейс на `DialogKit` с 5 применимыми поддиалогами (`Параметры...`, `Графики...`, `Сохранить...`, `Стиль...`, `Бутстреп...`), сохранение `PRE_n`/`RES_n` в датасет с `Undo`, векторные графики и полный `.sps` round-trip.
* **Ортогональная регрессия и регрессия Деминга (`ORTHOGONAL REGRESSION`):**
  * Аналитическая регрессия с ошибками в обеих переменных для **ортогональной регрессии (Total Least Squares, $\lambda = 1.0$)** и общей **регрессии Деминга** ($\lambda = \sigma_y^2 / \sigma_x^2 > 0$).
  * Стандартные ошибки методом **Jackknife** и $t$-доверительные интервалы ($df = N - 2$), СКО ортогональных остатков, $r$ Пирсона, сохранение `PRE_n`/`RES_n` с `Undo`, график рассеяния с линией $Y = X$, `Бутстреп...` и `.sps` round-trip.

### 🐧 Linux AppImage и исправления Output Viewer
* **Linux AppImage:** в пайплайн авторелизов добавлен универсальный пакет `statcraft-linux-x64.AppImage` в дополнение к `.deb` и `.tar.gz`.
* **Output Viewer и локализация:**
  * Локализована таблица `Примечания` (`Вывод создан`, `Активный массив данных`, `Число активных наблюдений`, `Число переменных`, `Сессия`).
  * В таблицах и на осях графиков новых регрессий отображаются человекочитаемые метки переменных (`Label`).
  * Исправлено масштабирование оси Y при включённой линии $Y = X$ в Deming, добавлены отступы и числовые деления оси Y, убран `-0.0000`, настроена автоширина колонок под длинные названия.

---

### 📦 Дистрибутивы релиза (Artifacts)
* **Windows (x64):** Автономный инсталлятор (`statcraft-windows-x64-installer.exe`) + Портативный архив (`statcraft-windows-x64.zip`).
* **Linux (x64):** AppImage (`statcraft-linux-x64.AppImage`) + Пакет Debian (`statcraft-linux-x64.deb`) + Портативный TAR.GZ (`statcraft-linux-x64.tar.gz`).
* **macOS (Apple Silicon & Intel):** Подписанный DMG-образ (`statcraft-macos-arm64.dmg`, `statcraft-macos-x64.dmg`).

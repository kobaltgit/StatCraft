# StatCraft v0.3.0

[English](#english) | [Русский](#русский)

---

<a name="english"></a>
## English

StatCraft v0.3.0 marks a monumental milestone in statistical computing: the official release of the **StatCraft Network Bridge & Autonomous Code Mode via Model Context Protocol (MCP)**, alongside a brand-new **Universal Theme System (Dark Pro, SPSS Classic Light, System Auto)** and **Automatic System Language Detection**.

### 🤖 StatCraft Network Bridge & Autonomous AI Copilot (MCP / Code Mode)
* **Zero SPSS Experience Required:** Users can now perform complex statistical analyses using natural language in their favorite AI assistants (Antigravity, Claude, ChatGPT, Ollama). The agent queries dataset schemas, formulates statistical pipelines, executes computations through StatCraft, and interprets $p$-values and effect sizes.
* **Deterministic Math vs. LLM Hallucinations:** Large language models guide the investigation, while StatCraft's verified native Rust calculation engine performs the mathematics with $\le 10^{-6}$ precision verified against SciPy and SPSS.
* **Dual Bridge Interfaces:**
  * **Headless stdio MCP Server:** Run `statcraft.exe --mcp` inside any MCP client (Claude Desktop, Cursor, Antigravity).
  * **Local IPC Socket Server:** Background TCP socket (`127.0.0.1:18493`) enabling live two-way control of the running StatCraft graphical window.
* **Rich MCP Toolset:**
  * `statcraft_run_syntax` — execute batch SPSS syntax (`.sps`) and stream back ASCII tables;
  * `statcraft_execute_command` — universal dynamic invocation of any registered core engine command;
  * `statcraft_list_commands` & `statcraft_list_variables` — live introspection of command schemas and variable metadata;
  * `statcraft_load_dataset` — directly import raw CSV/TSV data into the active session;
  * `statcraft_get_data_preview` & `statcraft_insert_output_note` — live data slicing and markdown notes insertion.
* **Desktop UI (`Extensions -> MCP Bridge / Copilot`):** In-app bridge management dialog built with DialogKit, featuring vector status indicators, real-time activity log, and one-click JSON configuration snippets.

### 🎨 Universal Theme System (Dark, Light, System)
* **Three Complete Visual Modes:**
  * **Dark Pro:** Modern ArtCraft deep dark theme;
  * **SPSS Classic Light:** Academic high-contrast light theme (`#FFFFFF` app canvas, `#F7F8FA` dialogs, `#E9ECEF` matrix headers, `#DDF4FF` soft-blue selection);
  * **System Auto:** Seamless real-time tracking of OS light/dark mode preference.
* **Strict WCAG Compliance:** Contrast ratio of primary text exceeds AAA ($15.6:1$ vs $7.0:1$ standard); muted text exceeds AA ($5.1:1$ vs $4.5:1$ standard).
* **Lock-Free Atomic Architecture:** 100% Safe Rust global atomic switch (`AtomicBool` with `Ordering::Relaxed`) delivering static reference palette lookups with zero allocation overhead.
* **Autonomous DialogKit Theming:** All existing 16 dialogs and all future extensions automatically adopt the active theme out of the box without engine changes.
* **FPS Guard & Persistence:** Visual style updates cached to protect render loop FPS; theme choice saved to `eframe::Storage` (`STORAGE_KEY_THEME = "statcraft_theme"`).

### 🌐 Automatic System Language Detection & Session Persistence
* **Smart Auto-Locale:** Automatically detects user operating system language (Windows API, macOS preferences, POSIX environment variables) with seamless English fallback for international users.
* **Persistent Preferences:** User language choice and bridge autostart flag reliably persist across sessions in `eframe::Storage`.

### 📦 Multi-Platform Release Artifacts
* **Windows (x64):** Inno Setup Installer (`.exe`) + Portable ZIP archive.
* **Linux (x64):** Debian package (`.deb`) + Portable TAR.GZ archive.
* **macOS (Apple Silicon ARM64 & Intel x64):** Apple DMG Disk Image + Application Bundle (`.app`) TAR.GZ.

---

<a name="русский"></a>
## Русский

Релиз StatCraft v0.3.0 представляет важнейшую веху в развитии проекта: официальный запуск **сетевого моста StatCraft Bridge и автономного режима Code Mode на базе Model Context Protocol (MCP)**, внедрение **универсальной системы тем оформления (Dark Pro, SPSS Classic Light, System Auto)** и **автоматического определения языка операционной системы**.

### 🤖 Сетевой мост StatCraft Bridge и ИИ-ассистенты (MCP / Code Mode)
* **Статистика без знания SPSS:** Пользователи могут проводить полный статистический анализ на простом человеческом языке через ИИ-агентов (Antigravity, Claude, ChatGPT, Ollama). Агент сам исследует метаданные, отправляет команды, считывает таблицы вывода и профессионально интерпретирует уровни значимости ($p$-value).
* **Математическая строгость против галлюцинаций LLM:** Языковая модель формирует логику исследования, а вычисления выполняет нативный Rust-движок StatCraft со строгой погрешностью $\le 10^{-6}$ (верифицирован против SciPy и канонических алгоритмов SPSS).
* **Два режима подключения:**
  * **Консольный stdio MCP-сервер:** Запуск `statcraft.exe --mcp` в конфигурации любого MCP-клиента (Claude Desktop, Cursor, Antigravity).
  * **Локальный IPC-сокет:** Фоновый TCP-сервер (`127.0.0.1:18493`), позволяющий ИИ-агенту управлять открытым окном StatCraft в реальном времени.
* **Набор инструментов MCP:**
  * `statcraft_run_syntax` — выполнение пакетного синтаксиса SPSS (`.sps`) с получением текстовых таблиц вывода;
  * `statcraft_execute_command` — универсальный вызов любой зарегистрированной команды ядра;
  * `statcraft_list_commands` и `statcraft_list_variables` — интроспекция схем команд и метаданных переменных;
  * `statcraft_load_dataset` — прямая загрузка сырых массивов данных в память активной сессии;
  * `statcraft_get_data_preview` и `statcraft_insert_output_note` — срез наблюдений и вставка аналитических заметок в Output Viewer.
* **Интерфейс в программе (`Расширения -> Мост MCP / Copilot`):** Модальное окно управления мостом на базе DialogKit с векторным индикатором статуса, журналом активности и кнопками копирования готовых настроек подключения.

### 🎨 Универсальная система тем оформления (Dark, Light, System)
* **Три полноценных режима:**
  * **Dark Pro:** Современная тёмная тема ArtCraft;
  * **SPSS Classic Light:** Академическая высококонтрастная светлая тема (`#FFFFFF` фон холста, `#F7F8FA` фон диалогов, `#E9ECEF` заголовки матриц, `#DDF4FF` выделение);
  * **System Auto:** Автоматическое следование за системной темой операционной системы.
* **Высокая контрастность по WCAG:** Контрастность основного текста превосходит стандарт AAA ($15.6:1$ при норме $7.0:1$); вторичного текста — превосходит AA ($5.1:1$ при норме $4.5:1$).
* **Атомарная архитектура без блокировок:** 100% Safe Rust глобальный переключатель (`AtomicBool` с `Ordering::Relaxed`), обеспечивающий мгновенный доступ к палитрам без аллокаций и оверхеда на горячем пути кадра egui.
* **Автономность для всех диалогов:** Все существующие 16 диалогов и любые будущие расширения автоматически поддерживают активную тему без модификации ядра.
* **Защита FPS и сохранение настроек:** Обновление стилей кэшируется в `applied_dark_mode` для защиты плавности интерфейса; выбор темы сохраняется в `eframe::Storage` (`STORAGE_KEY_THEME = "statcraft_theme"`).

### 🌐 Автоопределение языка системы и персистентность настроек
* **Умный автодетект:** Автоматическое определение языка ОС (Windows API, macOS preferences, переменные окружения POSIX) с корректным фоллбэком на английский язык для международных пользователей.
* **Надёжная персистентность:** Выбранный пользователем язык и статус автозапуска моста MCP сохраняются между перезапусками приложения в `eframe::Storage`.

### 📦 Дистрибутивы для всех операционных систем
* **Windows (x64):** Инсталлятор Inno Setup (`.exe`) + Портативный ZIP-архив.
* **Linux (x64):** Пакет Debian (`.deb`) + Портативный архив TAR.GZ.
* **macOS (Apple Silicon ARM64 и Intel x64):** Образ диска DMG + Приложение (`.app`) в архиве TAR.GZ.

---

## Предыдущие релизы / Previous Releases

### StatCraft v0.2.2
* Исчерпывающий двуязычный каталог функциональных возможностей (`docs/FEATURES.md` и `docs/FEATURES_RU.md`).
* Синхронизация версий в 4 манифестах пакетирования и релизные сборки.

### StatCraft v0.2.1
* Доказательный Clean-Room математический аудит вычислительного ядра против *IBM SPSS Statistics 24 Algorithms Guide* (1271 стр.) с точностью «Золотого оракула» ($\le 10^{-6}$).
* Финализация блоков `adp` (Automated Data Preparation) и `transform.programmability`.

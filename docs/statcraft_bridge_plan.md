# План реализации: Сетевой мост StatCraft (Code Mode, Семантический лог & Agent Kit)

## Описание цели
Реализовать крейт **`statcraft_bridge`** и интеграцию с графическим интерфейсом StatCraft в парадигме **Code Mode (не классический раздутый MCP, а сверхкомпактный исполняемый интерфейс)**:
1. **Философия Code Mode вместо классического MCP:**
   - *Классический MCP* плодит десятки узких микро-инструментов (`set_var_name`, `set_var_type`, `set_var_decimals`, `set_var_labels`, `run_ttest`, `run_anova`...), что забивает контекстное окно LLM сотнями токенов схемы, путает модель и требует многократных сетевых вызовов туда-обратно.
   - *Code Mode в StatCraft* предоставляет компактный набор из **6 высокоуровневых инструментов**, где ядром является пакетный синтаксис SPSS (`statcraft_run_syntax`). Один вызов синтаксиса способен на лету создать датасет с нуля, настроить любые шкалы/метки переменных, провести сложную серию статистических тестов и построить таблицы в отчете.
2. **Полный цикл управления данными для агента:**
   - Построение датасета и словаря переменных с нуля по текстовому запросу пользователя;
   - Мгновенное внедрение готовых таблиц пользователя (Excel/CSV/Markdown) в память активного окна без промежуточных файлов на диске;
   - Интеллектуальный аудит переменных: совет по исправлению шкал/меток/пропусков и пакетное применение по согласованию с пользователем.
3. **Интерактивный семантический журнал активности агента (Live Activity Log):**
   - Накопление действий агента в памяти приложения в фоновом режиме (даже когда окно настроек закрыто);
   - При открытии окна отображение полной хронологии с поддержкой мгновенной смены языка (RU/EN) «на лету» через каталоги Fluent без потери истории.
4. **Безопасный и деликатный жизненный цикл:**
   - Сервер внутри процесса StatCraft на `127.0.0.1:18493`, гарантированное освобождение порта и потока при закрытии программы (`Drop`);
   - Запуск через единый бинарник `statcraft.exe --mcp`;
   - Деликатность: агент не срывает фокус и не переключает вкладки у пользователя во время расчетов;
   - Адаптивный таймаут (дефолт 45с, опционально до 300с для тяжелых массивов).

---

## Архитектура: Согласованные решения (по итогам /grill-me)

### 1. Парадигма Code Mode (Инструменты MCP)
Схема `tools/list` оптимизирована под минимальное потребление контекста (всего 6 инструментов):

| Инструмент | Назначение в парадигме Code Mode |
|---|---|
| **`statcraft_run_syntax`** | **Главное ядро Code Mode.** Пакетное исполнение SPSS-синтаксиса (`.sps`). Управляет данными (`DATA LIST`, `COMPUTE`, `RECODE`), переменными (`VARIABLE LEVEL`, `VALUE LABELS`, `MISSING VALUES`, `VARIABLE LABELS`) и расчетами (`FREQUENCIES`, `DESCRIPTIVES`, `CROSSTABS`, `T-TEST`, `ONEWAY`, `REGRESSION`, `RELIABILITY`). Поддерживает параметр `timeout_seconds` (дефолт 45с, до 300с). |
| **`statcraft_load_dataset`** | Прямой импорт сырых данных пользователя: принимает текст таблицы (`csv_content: String`, разделители запятая/точка с запятой/табуляция) и опциональное имя. Создает `DataSet` в памяти активного окна в 1 клик. |
| **`statcraft_list_variables`** | Сверхкомпактная инспекция словаря переменных: имена, типы данных, шкалы (`Scale`/`Nominal`/`Ordinal`), текстовые метки, метки значений (Value Labels) и фильтры пропусков (Missing Values). |
| **`statcraft_get_data_preview`** | Инспекция среза данных: первые $N$ строк (по умолчанию 10) для экспресс-анализа распределений, пропусков и типов данных. |
| **`statcraft_insert_output_note`** | Добавление аналитических выводов агента в дерево Output Viewer в виде канонического блока `Notes` с форматированной карточкой. |
| **`statcraft_set_active_tab`** | Деликатная навигация: переключение вкладок (`data`, `variable`, `output`, `syntax`) **только** по осознанному решению агента при презентации итогов анализа (фокус не срывается автоматически). |

---

### 2. Сценарии работы агента через Code Mode

#### Сценарий А: Создание данных и переменных с нуля
Пользователь просит: *«Сделай мне учебный датасет для исследования стресса на 30 сотрудников»*.
1. Агент формирует и запускает `statcraft_run_syntax`:
   ```sps
   DATA LIST FREE / id (F3.0) age (F3.0) gender (F1.0) department (F1.0) stress (F2.0).
   BEGIN DATA
   1 24 1 1 7
   2 35 2 2 4
   ...
   30 42 1 3 8
   END DATA.
   VARIABLE LEVEL age (SCALE) / stress (ORDINAL) / gender, department (NOMINAL).
   VALUE LABELS gender 1 "Мужской" 2 "Женский"
     / department 1 "IT" 2 "HR" 3 "Sales".
   VARIABLE LABELS stress "Уровень стресса (1-10)".
   ```
2. В окне StatCraft сразу заполняются Data View и Variable View со всеми шкалами и метками.

#### Сценарий Б: Внедрение готовых таблиц пользователя
Пользователь вставляет Markdown или CSV из Excel прямо в диалог: *«Вот выгрузка, залей в программу»*.
1. Агент парсит таблицу в CSV-формат и вызывает `statcraft_load_dataset(csv_content = "id,name,score\n1,Alpha,85\n2,Beta,92")`.
2. Мост передает данные в `DataSet::from_raw_table()`, и таблица мгновенно открывается в Data View.

#### Сценарий В: Аудит словаря переменных и консультирование
1. Агент инспектирует структуру через `statcraft_list_variables` и `statcraft_get_data_preview`.
2. Находит аномалии: числовая переменная `status` имеет шкалу Scale, а пропуски в `salary` забиты кодом `-999`.
3. Агент пишет пользователю:
   > *«Я обнаружил, что `status` — это номинальный код, а в `salary` значение `-999` исказит расчет средней. Предлагаю:*
   > 1. *Сменить шкалу `status` на Nominal и добавить метки (1='Active', 2='Pending');*
   > 2. *Объявить `-999` в `salary` как Missing Value.*
   > *Применить изменения?»*
4. После согласия пользователя агент запускает `statcraft_run_syntax`:
   ```sps
   VARIABLE LEVEL status (NOMINAL).
   VALUE LABELS status 1 "Active" 2 "Pending".
   MISSING VALUES salary (-999).
   ```
5. В Variable View мгновенно обновляются иконки шкал и свойства ячеек.

---

### 3. Семантический журнал активности агента (Semantic Live Activity Log)
* **Принцип долгоживущего хранения:**
  Журнал живет в фоновом сервисе `BridgeService` внутри `StatCraftApp`. Он активен и пополняется **всегда**, даже когда окно настроек моста закрыто.
* **Структура в памяти:**
  ```rust
  pub struct BridgeLogEntry {
      pub timestamp: String,
      pub event: BridgeLogEvent,
  }

  pub enum BridgeLogEvent {
      ClientConnected { address: String },
      ClientDisconnected { address: String },
      VariablesRequested { count: usize },
      DataPreviewRequested { limit: usize },
      DatasetLoaded { rows: usize, cols: usize },
      SyntaxExecuted { preview: String, count: usize },
      CommandExecuted { command_id: String },
      OutputNoteAdded { title: String },
      TabSwitched { tab: String },
      ServerStarted { address: String },
      ServerStopped,
      Error { message: String },
  }
  ```
* **Мгновенный перевод на лету (Fluent i18n):**
  Хранятся не захардкоженные строки, а перечисление `BridgeLogEvent`. Рендеринг в UI происходит динамически через `t!("bridge-log-...", time = ..., ...)`. При смене языка `Вид -> Язык -> English` **вся накопленная история журнала на экране мгновенно переводится на английский язык**.
* **Кольцевой буфер:**
  Хранятся последние 100 записей (`VecDeque<BridgeLogEntry>`) строго в оперативной памяти (не оставляя мусора на диске и обеспечивая приватность). Кнопка `[ Очистить журнал ]` сбрасывает буфер.

---

### 4. Жизненный цикл, сеть и предохранители
* **Транспорт и бинарник:**
  - Запуск через единый исполняемый файл: `statcraft.exe --mcp`. Процесс мгновенно запускает stdio-проксирование без инициализации UI/eframe.
  - Локальный TCP-сокет: `127.0.0.1:18493` (Loopback, полная сетевая изоляция от внешней сети без необходимости токенов).
* **Предохранитель занятого порта:**
  - Первый запущенный экземпляр StatCraft занимает порт 18493 и является активным.
  - При случайном запуске второго экземпляра он не перехватывает команды, а отображает статус: *«Порт 18493 занят первичным экземпляром StatCraft»* с кнопкой *«Повторить»*.
* **Гарантия чистого выхода (Zero Zombie Processes):**
  - Сервер работает в легковесном потоке `std::thread` внутри процесса.
  - Сокет настраивается в неблокирующем режиме (`set_nonblocking(true)`) или с прерыванием по флагу `AtomicBool`.
  - В `Drop` сервиса моста сокет закрывается, поток завершается (`join`), порт освобождается.
* **Поведение при закрытом GUI:**
  - Если агент обращается через MCP, а окно StatCraft не запущено, агент получает понятный вежливый ответ:
    `"StatCraft GUI не запущен или мост выключен. Пожалуйста, запустите StatCraft для выполнения команд."`

---

## Архитектура и поток данных

```mermaid
sequenceDiagram
    autonumber
    actor Agent as ИИ-агент (Antigravity / Claude)
    participant MCP as MCP Stdio (statcraft.exe --mcp)
    participant Socket as TCP Мост (127.0.0.1:18493)
    participant GUI as StatCraftApp (UI-поток egui)
    participant Log as Семантический лог (BridgeLogRing)
    participant Output as Окно вывода (Output Viewer)

    Agent->>MCP: tools/call ("statcraft_run_syntax", {syntax: "FREQUENCIES..."})
    MCP->>Socket: JSON-RPC 2.0 {"method": "syntax.execute", "params": {...}}
    Socket->>GUI: BridgeAppRequest через std::sync::mpsc + ctx.request_repaint()
    Note over GUI: egui update() выполняет синтаксис
    GUI->>Log: Запись BridgeLogEvent::SyntaxExecuted в память
    GUI->>Output: Добавление команды в лог + Генерация Pivot Tables
    GUI-->>Socket: BridgeAppResponse {status: "ok", count: 1}
    Socket-->>MCP: JSON-RPC ответ {result: {...}}
    Agent<<--MCP: Результат выполнения (таблицы сгенерированы в окне)
```

---

## Предлагаемые изменения по компонентам

### 1. Архитектура слоев и манифесты (`Cargo.toml`, `xtask`)

#### [MODIFY] [Cargo.toml](file:///d:/Projects/active/StatCraft/Cargo.toml)
* Добавить `"crates/statcraft_bridge"` в `workspace.members`.

#### [MODIFY] [xtask/src/layers.rs](file:///d:/Projects/active/StatCraft/xtask/src/layers.rs)
* Зарегистрировать крейт `statcraft_bridge` в реестре `CRATE_REGISTRY`:
  ```rust
  CrateSpec {
      name: "statcraft_bridge",
      level: LayerLevel::Utility,
      allowed_dependencies: &["statcraft_core", "statcraft_syntax"],
  },
  ```
* Добавить `"statcraft_bridge"` в `allowed_dependencies` для `statcraft_app`.

---

### 2. Новый крейт: `statcraft_bridge` (`crates/statcraft_bridge/`)

#### [NEW] `crates/statcraft_bridge/Cargo.toml`
* `license.workspace = true` (MIT OR Apache-2.0).
* Зависимости: `statcraft_core`, `statcraft_syntax`, `serde`, `serde_json`, `thiserror`.

#### [NEW] `crates/statcraft_bridge/src/lib.rs`
* Экспорт модулей: `protocol`, `server`, `client`, `channel`, `mcp`, `events`.

#### [NEW] `crates/statcraft_bridge/src/events.rs`
* Типизированный enum `BridgeLogEvent` и структура `BridgeLogEntry`.
* Ограниченный кольцевой буфер `BridgeLogRing` (`VecDeque`, емкость 100 элементов).

#### [NEW] `crates/statcraft_bridge/src/protocol.rs`
* Структуры JSON-RPC 2.0 (`RpcRequest`, `RpcResponse`, `RpcError`).
* Методы протокола Code Mode:
  - `dataset.get_info`: статус датасета, строки/колонки;
  - `dataset.get_variables`: словарь переменных со шкалами, метками и пропусками;
  - `dataset.get_data_preview`: первые строки данных;
  - `dataset.load_csv`: прямой импорт таблицы из CSV-текста;
  - `syntax.execute`: пакетное выполнение SPSS-синтаксиса (Code Mode);
  - `output.add_note`: вставка блока Notes в дерево отчета;
  - `ui.set_active_tab`: переключение вкладок.

#### [NEW] `crates/statcraft_bridge/src/channel.rs`
* Типизированный межпоточный канал передачи запросов `BridgeAppRequest` и ответов `oneshot` между сокетом и UI-потоком.

#### [NEW] `crates/statcraft_bridge/src/server.rs`
* `BridgeServer`:
  - Прослушивание `127.0.0.1:18493`;
  - Неблокирующая обработка с мягким выходом по флагу `Arc<AtomicBool>`;
  - Потокобезопасная остановка сокета и освобождение порта при `stop()` и `Drop`.

#### [NEW] `crates/statcraft_bridge/src/client.rs`
* `BridgeClient`: локальный клиент для интеграционных тестов и stdio-проксирования.

#### [NEW] `crates/statcraft_bridge/src/mcp.rs`
* Реализация stdio MCP сервера:
  - Обработка `initialize`, `tools/list`, `tools/call`;
  - Экспорт 6 инструментов Code Mode;
  - Адаптивный таймаут (дефолт 45с, опционально до 300с);
  - Вежливое текстовое уведомление при недоступности GUI.

---

### 3. Графический интерфейс и сервис (`crates/statcraft_app/`)

#### [MODIFY] [crates/statcraft_app/Cargo.toml](file:///d:/Projects/active/StatCraft/crates/statcraft_app/Cargo.toml)
* Добавить `statcraft_bridge = { path = "../statcraft_bridge" }`.

#### [NEW] `crates/statcraft_app/src/bridge_service.rs`
* Долгоживущий сервис в приложении:
  - Управление потоком сервера и реализация `Drop`;
  - Хранение `BridgeLogRing` в памяти сессии;
  - Обработка запросов `process_bridge_requests()` в кадре `update()`:
    * Запуск синтаксиса через `syntax_runner::run_syntax_batch`;
    * Прямой импорт таблицы через `DataSet::from_raw_table`;
    * Чтение словаря из `DataSet`;
    * Добавление заметок в `OutputViewerState`;
    * Переключение вкладок в `dock_state`.

#### [MODIFY] [crates/statcraft_app/src/app.rs](file:///d:/Projects/active/StatCraft/crates/statcraft_app/src/app.rs)
* Добавить поля:
  - `bridge_service: BridgeService`;
  - `mcp_bridge_dialog_state: McpBridgeDialogState`.
* В `update()` вызывать обработку запросов моста.
* Загрузка и сохранение настройки автозапуска (`autostart: true` по умолчанию) через `eframe::Storage` и `.statcraft/`.

#### [NEW] `crates/statcraft_app/src/dialogs/mcp_bridge_dialog.rs`
* Модульный диалог на базе `DialogKit` (**строго $\le 500$ строк** для `xtask file-guard`):
  - Обертка `DialogShell::standard(...)`;
  - Векторный круговой индикатор статуса (`ui.painter().circle_filled`);
  - Кнопки управления `[ Остановить мост ]` / `[ Запустить мост ]`;
  - Чекбокс `[x] Запускать автоматически при старте программы`;
  - Выбор формата конфигурации (Claude Desktop / Antigravity / Python) с автоопределением пути к `statcraft.exe`;
  - Кнопки `[ Скопировать конфигурацию MCP ]` и `[ Скопировать инструкцию (Skill) ]`;
  - **Прокручиваемый семантический журнал активности агента** с локализацией на лету через `t!(...)`;
  - Кнопка `[ Очистить журнал ]`.

#### [MODIFY] [crates/statcraft_app/src/app_menus/analyze_etc.rs](file:///d:/Projects/active/StatCraft/crates/statcraft_app/src/app_menus/analyze_etc.rs)
* Активировать пункт меню `menu-extensions-mcp-bridge` (статус `Ready`, открытие диалога).

#### [MODIFY] [crates/statcraft_app/src/main.rs](file:///d:/Projects/active/StatCraft/crates/statcraft_app/src/main.rs)
* Перехват флага `--mcp`: вызов `statcraft_bridge::mcp::run_stdio()` без запуска eframe/DirectX.

---

### 4. Навык агента и локализация (`.agents/skills/`, `locales/`)

#### [NEW] `.agents/skills/statcraft-assistant/SKILL.md`
* Документированный системный навык для ИИ-аналитика данных:
  - Роль эксперта-статистика в StatCraft;
  - Правила составления SPSS-синтаксиса (структура команд, подкоманды через слеш `/`, обязательная точка `.` в конце);
  - Каталог процедур: `DATA LIST`, `VARIABLE LEVEL`, `VALUE LABELS`, `MISSING VALUES`, `FREQUENCIES`, `DESCRIPTIVES`, `CROSSTABS`, `CORRELATIONS`, `T-TEST`, `ONEWAY`, `REGRESSION`, `RELIABILITY`;
  - Рекомендации по формулированию ясных и емких выводов в блоках Notes по стандартам научной отчетности (APA).

#### [MODIFY] [locales/en.ftl](file:///d:/Projects/active/StatCraft/locales/en.ftl) & [locales/ru.ftl](file:///d:/Projects/active/StatCraft/locales/ru.ftl)
* Ключи интерфейса и семантического журнала:
  - `dialog-mcp-bridge-title`
  - `dialog-mcp-bridge-status-active`
  - `dialog-mcp-bridge-status-stopped`
  - `dialog-mcp-bridge-status-port-busy`
  - `dialog-mcp-bridge-btn-start`
  - `dialog-mcp-bridge-btn-stop`
  - `dialog-mcp-bridge-btn-retry`
  - `dialog-mcp-bridge-autostart`
  - `dialog-mcp-bridge-copy-config`
  - `dialog-mcp-bridge-copy-skill`
  - `dialog-mcp-bridge-copied`
  - `dialog-mcp-bridge-log-title`
  - `dialog-mcp-bridge-clear-log`
  - `bridge-log-client-connected = { $time } [Подключение] Агент подключился ({ $addr })`
  - `bridge-log-client-disconnected = { $time } [Отключение] Сессия агента завершена ({ $addr })`
  - `bridge-log-vars-requested = { $time } [Инспекция] Передан словарь переменных ({ $count } колонок)`
  - `bridge-log-preview-requested = { $time } [Инспекция] Передан предпросмотр данных ({ $limit } строк)`
  - `bridge-log-dataset-loaded = { $time } [Данные] Загружен массив данных ({ $rows } строк, { $cols } колонок)`
  - `bridge-log-syntax-executed = { $time } [Синтаксис] Выполнен синтаксис: { $cmd } (команд: { $count })`
  - `bridge-log-note-added = { $time } [Отчет] Добавлено аналитическое примечание: "{ $title }"`
  - `bridge-log-tab-switched = { $time } [Интерфейс] Переключена вкладка на "{ $tab }"`
  - `bridge-log-server-started = { $time } [Сервер] Мост запущен на { $addr }`
  - `bridge-log-server-stopped = { $time } [Сервер] Мост остановлен`

---

## План проверки и критерии готовности (Definition of Done)

### Автоматические тесты
1. **Тесты крейта `statcraft_bridge`:**
   ```powershell
   cargo test -p statcraft_bridge -j 1
   ```
   - Сериализация и десериализация протокола JSON-RPC 2.0;
   - Запуск и остановка сервера: освобождение сокета, выход потока без паник;
   - Сквозной тест через `BridgeClient`: загрузка CSV, выполнение синтаксиса, добавление заметок, проверка ответа;
   - Проверка форматирования семантического лога при переключении языков RU/EN.
2. **Инварианты архитектуры и кода:**
   ```powershell
   cargo run -p xtask -j 1 -- layers
   cargo run -p xtask -j 1 -- file-guard
   cargo run -p xtask -j 1 -- design-guard
   cargo run -p xtask -j 1 -- parity
   cargo test -p statcraft_app --test output_viewer_invariants -j 1
   cargo test --workspace -j 1
   cargo clippy --workspace -j 1 -- -D warnings
   ```

### Ручная верификация
1. Запуск StatCraft GUI: `cargo run -p statcraft_app -j 1`.
2. Меню `Расширения -> Мост для ИИ-агентов и MCP...`:
   - Векторный зеленый индикатор, статус «Активен», порт 18493;
   - Нажатие `[ Остановить мост ]` $\to$ статус «Остановлен», запись в логе;
   - Нажатие `[ Запустить мост ]` $\to$ статус «Активен»;
   - Закрытие диалога;
3. Внешний запуск `statcraft.exe --mcp`:
   - Вызов `statcraft_load_dataset` с сырым CSV $\to$ проверка появления данных в Data View;
   - Вызов `statcraft_run_syntax` с настройкой шкал и расчетом $\to$ проверка появления таблиц в Output Viewer;
   - Открытие диалога моста $\to$ вся история действий отображается в журнале;
   - Переключение `Вид -> Язык -> English` $\to$ вся история в журнале мгновенно переведена на английский язык.
4. Проверка жизненного цикла: закрытие окна StatCraft $\to$ проверка в PowerShell, что порт 18493 освобожден и зомби-процессов нет.

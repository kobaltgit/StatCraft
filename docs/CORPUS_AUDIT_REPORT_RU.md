# StatCraft — Протокол корпусного аудита форматов данных (Real-World Corpora & Codecs Audit)

## Вводная часть
Настоящий документ содержит официальный доказательный протокол Clean-Room аудита подсистемы ввода-вывода `crates/statcraft_io` (кодеки SPSS `.sav`/`.zsav`, Excel `.xlsx`/`.xls`/`.ods`, CSV/TSV RFC 4180 и контейнер проекта StatCraft `.stc`) на реальных корпусах массивов данных, синтетических краевых тестах и поврежденных потоках фаззинга.

Аудит проведен с использованием **Craft Anything MCP** в режиме чистой комнаты (`clean-room: true`) на соответствие спецификациям *GNU PSPP System File Format Specification*, *RFC 4180* и инварианту надежности экосистемы ArtCraft *Never Crash*.

---

## 1. Сводная матрица аудита и результаты (Микропланы C-01 — C-05)

| Микроплан | Подсистема / Цель | Целевой инвариант | Результат и статус верификации |
|---|---|---|---|
| **C-01** | SPSS `.sav` / `.zsav` Bytecode Stream | Полная матрица байт-кодов (0..255), ZLIB декомпрессия блоков | **ИСПРАВЛЕНО (BUG-003):** Исправлено декодирование опкода 254 в числовой ноль `0.0` (ранее ошибочно декодировался как `Null`). Подтверждена обработка опкода 255 (`SYSMIS`), смещения $val = opcode - bias$ (коды 1..251), несжатых литералов 253 и конца файла 252. Декомпрессия блоков `.zsav` по смещениям zheader/ztrailer. |
| **C-02** | SPSS Record 7, метаданные и кодировки | Подтипы Record 7 (3, 11, 14, 20); транскодинг Windows-1251 | **ИСПРАВЛЕНО (BUG-004):** Исправлена проверка расширенных записей на канонический подтип 11 для параметров отображения (шкала Nominal/Ordinal/Scale, ширина, выравнивание). Реализована поддержка подтипа 14 для очень длинных строк Very Long Strings ($> 255$ байт) через слоты продолжения. Создан безопасный модуль транскодирования `encoding.rs` без сторонних зависимостей (Windows-1251 для кириллицы, Windows-1252, KOI8-R). |
| **C-03** | Электронные таблицы Excel (`.xlsx`, `.xls`, `.ods`) | Приведение типов «грязных» колонок, формульные ошибки, экстремальные даты | **ПОДТВЕРЖДЕНО:** Колонки со смешанными типами (числа и текст) безопасно приводятся к `RawDataType::Text`. Формульные ошибки (`#VALUE!`, `#REF!`, `#DIV/0!`, `#N/A`, `#NAME?`) и текстовые маркеры пропусков (`"-"`, `"none"`, `"NA"`) транслируются в `RawValue::Null`. Заголовки санируются и дедуплицируются по правилам SPSS. |
| **C-04** | Кодек CSV / TSV RFC 4180 | Многострочные поля, экранирование кавычек, UTF-8 BOM, обратимость | **ПОДТВЕРЖДЕНО:** Корректное чтение многострочных ячеек в кавычках (`"line 1\nline 2"`) и экранированных кавычек (`""`). Автоопределение разделителей (`,`, `;`, `\t`, `|`). Отсечение сигнатуры UTF-8 BOM (`\u{FEFF}`) в начале файла. Математический инвариант обратимости $\text{RawTable} \equiv \text{reload}(\text{export}(\text{RawTable}))$ полностью подтвержден. |
| **C-05** | Фаззинг поврежденных данных и корпусный раннер | Инвариант Never Crash: 0 паник на поврежденных входах | **ПОДТВЕРЖДЕНО:** Реализована утилита сборщика `cargo xtask test-corpus` и встроенный набор фаззинга (11 аномальных потоков: усеченные заголовки, невалидный magic, отрицательный bias, битый gzip, поврежденный json). 100% поврежденных файлов безопасно возвращают типизированную ошибку `Err(IoError)` с нулевым числом паник. |

---

## 2. Результаты запуска корпусного тестирования (`cargo xtask test-corpus`)

```text
=======================================================
  StatCraft Real-World Corpora & Codecs Audit Runner  
  Clean-Room Standards & Never Crash Verification     
=======================================================

File                                | Format   | Status           | Dim (R x C)  | Time    
------------------------------------+----------+------------------+--------------+---------
corrupt_bad_magic.sav               | sav      | [PASS] Safe Err  | Malformed... |     2 ms
corrupt_truncated.sav               | sav      | [PASS] Safe Err  | Io("Faile... |     0 ms
multiline_bom.csv                   | csv      | [PASS] Loaded OK | 2 x 3        |     2 ms
pipe_delimited.csv                  | csv      | [PASS] Loaded OK | 2 x 3        |     0 ms
semicolon_european.csv              | csv      | [PASS] Loaded OK | 3 x 3        |     0 ms
synthetic_clean.sav                 | sav      | [PASS] Loaded OK | 1 x 1        |     1 ms
synthetic_compressed.sav            | sav      | [PASS] Loaded OK | 3 x 1        |     0 ms
synthetic_cyrillic.sav              | sav      | [PASS] Loaded OK | 1 x 1        |     0 ms
valid_survey.csv                    | csv      | [PASS] Loaded OK | 4 x 4        |     0 ms
valid_survey.stc                    | stc      | [PASS] Loaded OK | 2 x 2        |     4 ms

[INFO] Scanning supplementary dataset files in 'test_data/'...
File                                | Format   | Status           | Dim (R x C)  | Time    
------------------------------------+----------+------------------+--------------+---------
sample_survey.csv                   | csv      | [PASS] Loaded OK | 20 x 7       |     0 ms
sample_survey.stc                   | stc      | [PASS] Loaded OK | 20 x 7       |     0 ms

=== Running Malformed Input Fuzzing Suite (Never Crash Invariant) ===
  [FUZZ OK] Zero bytes                     -> Safe typed Err (0 panics)
  [FUZZ OK] Single null byte               -> Safe typed Err (0 panics)
  [FUZZ OK] Truncated SPSS magic           -> Safe typed Err (0 panics)
  [FUZZ OK] Random garbage 16B             -> Safe typed Err (0 panics)
  [FUZZ OK] Truncated SPSS header 50B      -> Safe typed Err (0 panics)
  [FUZZ OK] SPSS bad magic 176B            -> Safe typed Err (0 panics)
  [FUZZ OK] SPSS negative bias 176B        -> Safe typed Err (0 panics)
  [FUZZ OK] Corrupt GZIP header 4B         -> Safe typed Err (0 panics)
  [FUZZ OK] Truncated JSON STC             -> Safe typed Err (0 panics)
  [FUZZ OK] CSV unclosed quotes            -> Safe typed Err (0 panics)
  [FUZZ OK] CSV null characters            -> Safe typed Err (0 panics)

=======================================================
  CORPUS AUDIT SUMMARY
  Total Scanned Files : 12
  Valid Files OK      : 10
  Corrupt Safe Errors : 2
  Fuzzed Probes OK    : 11
  Panics Detected     : 0
=======================================================
>>> [SUCCESS] All corpora and codecs passed with ZERO panics!
```

---

## 3. Зарегистрированные дефекты в реестре `docs/BUGS_AND_ISSUES.md`
- **BUG-003:** В сжатых байт-кодовых потоках SPSS `.sav` опкод 254 ошибочно декодировался как пропуск (`Null`), превращая нулевые значения в `SYSMIS`. Исправлено в `crates/statcraft_io/src/spss_loader/decompress.rs`.
- **BUG-004:** В расширенных записях SPSS Record 7 параметры отображения переменных (шкала измерения, ширина, выравнивание) проверялись по ошибочному подтипу 4 вместо канонического подтипа 11. Исправлено в `crates/statcraft_io/src/spss_loader/records.rs` с поддержкой подтипа 11, подтипа 14 (Very Long Strings), подтипа 3 (codepage) и подтипа 20 (encoding).

---

## 4. Контроль критериев завершения задач (Definition of Done)
1. `cargo test --workspace -j 1` — 100% тестов воркспейса завершились успешно (код 0).
2. `cargo fmt --all -- --check` — форматирование исходного кода полностью валидно.
3. `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — ноль предупреждений компилятора.
4. `cargo run -p xtask -j 1 -- layers` — строгое соблюдение слоистой архитектуры L0–L6.
5. `cargo run -p xtask -j 1 -- file-guard` — все 269 файлов укладываются в лимиты строк.
6. `cargo run -p xtask -j 1 -- design-guard` — 100% соответствие дизайн-системе DialogKit.
7. `cargo run -p xtask -j 1 -- test-corpus` — 100% успешный прогон корпусных тестов, 0 паник.
8. `cargo run -p xtask -j 1 -- ci` — полный шлюз контроля качества пройден.

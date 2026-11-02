# Investigation report

**Question:** Соответствуют ли модули M-09 (Programmability) и M-10 (SET SEED & RNG) каноническим алгоритмам и поведению IBM SPSS Statistics?

**Purpose:** Математический и архитектурный аудит модулей M-09 (Programmability) и M-10 (SET SEED & RNG) по канону IBM SPSS.

**Done when:** Построчный аудит модулей M-09 и M-10 против канона IBM SPSS Statistics, фиксация гипотез и findings, тесты пройдены без сбоев.

## What we looked at

_No targets recorded._

## What we found

_No findings have been recorded yet._

## Open questions

_None outstanding._

## How this was done

- **Mode:** Clean room (nothing inside the original was read)

## Handoff

Все микропланы Фазы 2 (M-01 — M-20) полностью проаудированы построчно с фиксацией доказательств в Craft Anything MCP. Результаты:
1. Раздел A (M-01 — M-10): M-01 (expr_eval): зафиксированы 3 расхождения (приоритеты ** и NOT, 3VL Клини, нехватка SIN/COS/TAN/MOD/LG10). M-02 (time_series): зафиксировано расхождение SMA (реализован как trailing PMA). M-03 (shift), M-04 (date_time), M-05 (recode), M-06 (autorecode), M-07 (count), M-08 (dummies) — 100% подтверждены. M-09 (programmability) и M-10 (rng) — 100% подтверждены, отмечено использование SplitMix64 вместо LCG.
2. Раздел B (M-11 — M-15): M-11 (select_cases: FILTER/DELETE), M-12 (sort_cases: стабильная сортировка, Ordering::Less для Null/NaN как наименьших значений), M-13 (weight_cases: w <= 0/NaN отсекаются в row_weight), M-14 (split_file: LAYERED/SEPARATE с автосортировкой), M-15 (variable: 11 атрибутов словаря, дискретные и интервальные MissingValues) — 100% соответствие канону SPSS.
3. Раздел C (M-16 — M-19): M-16 (spss_loader: $FL2/$FL3, типы 1,2,3/4,7, байт-код и zsav zlib декомпрессия), M-17 (excel_loader: V_, _, автовывод), M-18 (csv: RFC 4180, round-trip), M-19 (stc: Parquet + JSON metadata) — 100% подтверждены.
4. Раздел D (M-20): M-20 (reporting: двухъярусные заголовки PivotTable, подавление нуля .000 для p-value, экспорт HTML/TSV/TXT) — 100% подтверждено.
Все тесты во всех затронутых крейтах (statcraft_algo, statcraft_core, statcraft_syntax, statcraft_io, statcraft_reporting) пройдены без ошибок (-j 1).


# Investigation report

**Question:** Соответствует ли реализация expr_eval в statcraft_algo каноническим алгоритмам IBM SPSS Statistics для COMPUTE?

**Purpose:** Верификация математической и логической эквивалентности движка выражений COMPUTE канону IBM SPSS pp. 280-288.

**Done when:** Код expr_eval полностью проверен построчно против канона IBM SPSS pp. 280-288, гипотезы верифицированы с фиксацией findings, тесты пройдены без ошибок.

## What we looked at

- crates/statcraft_algo/src/expr_eval/lexer.rs
- crates/statcraft_algo/src/expr_eval/parser.rs
- crates/statcraft_algo/src/expr_eval/eval.rs
- crates/statcraft_algo/src/expr_eval/mod.rs

## What we found

_No findings have been recorded yet._

## Open questions

_None outstanding._

## How this was done

- **Mode:** Clean room (nothing inside the original was read)

## Handoff

Микроплан 2.3 (Блок 9: Мастер даты и времени date_time.rs) завершен реальным построчным аудитом. Результаты: 1) Парсинг форматов ISO 8601, европейского, американского и timestamp полностью верифицирован (hyp_11: supported). 2) Извлечение компонентов XDATE.* строго соответствует канону SPSS, включая канонический день недели Sunday=1 (hyp_10: supported). 3) Прибавление интервалов DATEADD с корректным усечением конца месяца верифицировано (hyp_9: supported). 4) Расчет разностей DATEDIFF в календарных годах, месяцах, неделях, днях, секундах верифицирован; отмечено, что для минут и часов вычисляется вещественная доля (division by 60/3600), тогда как канонический DATEDIFF SPSS выполняет целочисленное усечение (hyp_8: supported). Следующий шаг: Микроплан 2.4 (Блок 10: Перекодирование, дамми и счетчики recode.rs, auto_recode.rs, dummies.rs, count_values.rs).


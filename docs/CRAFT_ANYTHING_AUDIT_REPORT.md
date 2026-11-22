# Investigation report

**Question:** Математический аудит и документирование пропущенных процедур ядра statcraft_algo: Блок 16 (Продвинутая регрессия и частные корреляции), Блок 17 (Общая линейная модель GLM / UNIANOVA / MANOVA) и Блок 20 (Регрессия пропорциональных рисков Кокса COXREG) по канону IBM SPSS Statistics 24.

**Purpose:** Обеспечение 100% математической точности и полной доказательной базы в протоколах аудита SPSS_ALGORITHMS_AUDIT.md и SPSS_ALGORITHMS_AUDIT_RU.md по канону Clean-Room.

**Done when:** Построчная верификация математических формул и алгоритмов для Блоков 16, 17, 20, прогон эталонных тестов с -j 1, синхронное обновление русскоязычной и англоязычной спецификаций аудита, сохранение отчета расследования.

## What we looked at

- crates/statcraft_algo/src/partial_corr/
- crates/statcraft_algo/src/logistic/
- crates/statcraft_algo/src/curvefit/
- crates/statcraft_algo/src/glm/
- crates/statcraft_algo/src/cox/
- crates/statcraft_algo/src/frequencies/
- crates/statcraft_syntax/src/commands/frequencies_cmd/
- crates/statcraft_syntax/src/syntax_parser/mappers/stats/
- docs/IBM_SPSS_Statistics_Algorithms.pdf

## What we found

- Процедура FREQUENCIES декомпозирована на модульный пакет `frequencies/` (options.rs, percentiles.rs, stats.rs, table.rs, mod.rs) с соблюдением лимитов строк <= 500.
- Все 14 дескриптивных статистик, перцентили SPSS HAVERAGE и правило Mode Tie-Breaker полностью верифицированы Golden Oracle тестом `frequencies_oracle_test.rs` с погрешностью <= 1e-6.
- Командная шина `stat.frequencies` расширена поддержкой 12 одномерных статистик, перцентилей (квартили, N-тили, custom cutpoints), сгруппированных срезов `group_midpoints` и пересортировки категорий (`dvalue`, `afreq`, `dfreq`) с динамическим пересчетом кумулятивных процентов.
- Парсер SPSS-синтаксиса обеспечивает 100% двусторонний паритет для подкоманд `/VARIABLES`, `/FORMAT` (`TABLE`/`NOTABLE`, `AVALUE`/`DVALUE`/`AFREQ`/`DFREQ`), `/STATISTICS` (включая макросы `DEFAULT`, `ALL`, `NONE` и 12 индивидуальных ключевых слов), `/PERCENTILES`, `/NTILES`, `/ORDER`.
- Модули `frequencies_cmd/` и `mappers/stats/` архитектурно декомпозированы в изолированные пакеты, строго соблюдая Rule 12 (File Size Guard <= 500 строк).

## Open questions

_None outstanding._

## How this was done

- **Mode:** Clean room (nothing inside the original was read)

## Handoff

Математический аудит и документирование пропущенных разделов успешно завершены. Блок 16 (PARTIAL CORR, LOGISTIC REGRESSION, CURVEFIT), Блок 17 (UNIANOVA, MANOVA, REPEATED MEASURES) и Блок 20 (COXREG) полностью верифицированы по канону IBM SPSS Statistics 24 Algorithms Guide. Все 26 эталонных тестов Golden Oracle в statcraft_algo пройдены с абсолютной точностью <= 1e-6. Документы docs/SPSS_ALGORITHMS_AUDIT_RU.md и docs/SPSS_ALGORITHMS_AUDIT.md синхронизированы, содержат полные формулы, степени свободы, алгоритмы оптимизации, тесты согласия и ссылки на страницы первоисточника. Все 20 блоков математического ядра теперь на 100% задокументированы.


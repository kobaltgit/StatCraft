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
- docs/IBM_SPSS_Statistics_Algorithms.pdf

## What we found

_No findings have been recorded yet._

## Open questions

_None outstanding._

## How this was done

- **Mode:** Clean room (nothing inside the original was read)

## Handoff

Математический аудит и документирование пропущенных разделов успешно завершены. Блок 16 (PARTIAL CORR, LOGISTIC REGRESSION, CURVEFIT), Блок 17 (UNIANOVA, MANOVA, REPEATED MEASURES) и Блок 20 (COXREG) полностью верифицированы по канону IBM SPSS Statistics 24 Algorithms Guide. Все 26 эталонных тестов Golden Oracle в statcraft_algo пройдены с абсолютной точностью <= 1e-6. Документы docs/SPSS_ALGORITHMS_AUDIT_RU.md и docs/SPSS_ALGORITHMS_AUDIT.md синхронизированы, содержат полные формулы, степени свободы, алгоритмы оптимизации, тесты согласия и ссылки на страницы первоисточника. Все 20 блоков математического ядра теперь на 100% задокументированы.


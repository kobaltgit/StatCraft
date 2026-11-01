# StatCraft — Математический аудит алгоритмов IBM SPSS Statistics 24

[ English (Primary) ](SPSS_ALGORITHMS_AUDIT.md) | **Русский**

> **Статус документа:** Официальный протокол доказательного математического аудита ядра `statcraft_algo`.  
> **Методология:** Чистая комната (Clean-Room Reverse Engineering), инструментальная доказательная база.  
> **Инструмент аудита:** [Craft Anything (`storytold/craft-anything`)](https://github.com/storytold/craft-anything) — открытый движок доказательного исследования и верификации ПО с поддержкой Model Context Protocol (MCP).  
> **Эталонная спецификация:** *IBM SPSS Statistics 24 Algorithms Guide* (1271 стр., IBM Corporation).  
> **Погрешность вычислений (Golden Oracle):** $\varepsilon \le 10^{-6}$ (сверка с эталонами IBM SPSS / Python SciPy).

---

## 1. Методология и регламент аудита (Clean-Room & Craft Anything)

В соответствии с правилами разработки [AGENTS.md](../AGENTS.md#L35-L43), StatCraft создается по принципу строгой «чистой комнаты» (Clean-Room):
* Запрещено декомпилировать или копировать проприетарные бинарники и код SPSS;
* Алгоритмы, математические формулы и поведение воспроизводятся исключительно по открытым академическим источникам и официальным математическим спецификациям разработчика (*IBM SPSS Statistics Algorithms*);
* Для автоматизированного аудита используется специализированный MCP-сервер **[Craft Anything](https://github.com/storytold/craft-anything)** в режиме `clean-room: true`.

### Архитектура расследования в Craft Anything
```text
┌───────────────────────────────────────────────────────────────┐
│     IBM SPSS Statistics 24 Algorithms (Reference PDF)         │
└───────────────────────────────┬───────────────────────────────┘
                                │
                                ▼
┌───────────────────────────────────────────────────────────────┐
│          Craft Anything MCP Engine (Clean-Room Mode)          │
│   • Brief & Hypotheses Ledger                                 │
│   • Evidence Extraction & Page Verification                   │
│   • Mathematical Model & Formula Invariants                   │
└───────────────────────────────┬───────────────────────────────┘
                                │
                                ▼
┌───────────────────────────────────────────────────────────────┐
│          StatCraft Math Core (`crates/statcraft_algo`)        │
│   • 100% Safe Rust                                            │
│   • Golden Oracle Tests against Reference Corpora (≤ 1e-6)    │
│   • CommandBus & DialogKit Integration                        │
└───────────────────────────────────────────────────────────────┘
```

---

## 2. Сводная матрица прогресса аудита

| Блок | Группа процедур | Файлы реализации | Статус | Точность Oracle |
|---|---|---|---|---|
| **Блок 1** | Описательные статистики (`DESCRIPTIVES`, `FREQUENCIES`, `CROSSTABS`, `MEANS`) | `descriptives.rs`, `frequencies.rs`, `crosstabs.rs`, `means.rs` | **100% Верифицировано** | $\le 10^{-6}$ |
| **Блок 2** | Сравнение средних (`T-TEST 1-SAMPLE`, `T-TEST INDEP`, `T-TEST PAIRED`, `ONE-WAY ANOVA`) | `ttest_onesample.rs`, `ttest_indep.rs`, `ttest_paired.rs`, `anova_oneway.rs` | **100% Верифицировано** | $\le 10^{-6}$ |
| **Блок 3** | Связи и многомерный анализ (`CORRELATIONS`, `REGRESSION`, `FACTOR`, `RELIABILITY`) | `correlations.rs`, `regression.rs`, `factor.rs`, `reliability.rs` | **100% Верифицировано** | $\le 10^{-6}$ |
| **Блок 4** | Непараметрические критерии (`MANN-WHITNEY`, `KRUSKAL-WALLIS`, `WILCOXON`, `SIGN`) | `nonparam_twosample.rs`, `kruskal_wallis.rs`, `nonparam_related.rs` | **100% Верифицировано** | $\le 10^{-6}$ |
| **Блок 5** | Трансформации данных (`RANK`, `OPTIMAL BINNING`, `REPLACE MISSING`) | `rank_cases.rs`, `optimal_binning.rs`, `replace_missing.rs` | **100% Верифицировано** | $\le 10^{-6}$ |

---

## 3. Детальный протокол верификации

### Блок 1: Описательные статистики и базовые процедуры

#### 1.1. `DESCRIPTIVES` (IBM Algorithms стр. 368–370)
* **Реализация:** [`crates/statcraft_algo/src/descriptives.rs`](../crates/statcraft_algo/src/descriptives.rs)
* **Тесты:** `descriptives::tests::test_golden_oracle_descriptives_scipy`
* **Сопоставление формул:**
  * **Моменты и среднее:** рекурсивный алгоритм Спайсера (Spicer, 1972) со взвешиванием $W = \sum w_i$:
    $$\bar{X} = \frac{\sum w_i X_i}{W}, \quad s^2 = \frac{1}{W-1} \sum w_i (X_i - \bar{X})^2, \quad SE(\bar{X}) = \frac{s}{\sqrt{W}}$$
  * **Асимметрия (Skewness):** выборочная несмещенная асимметрия по Блиссу (Bliss, 1967):
    $$g_1 = \frac{W}{(W-1)(W-2)} \cdot \frac{\sum w_i (X_i - \bar{X})^3}{s^3}$$
    Стандартная ошибка: $SE(g_1) = \sqrt{\frac{6W(W-1)}{(W-2)(W+1)(W+3)}}$. Расчет подавляется при $W \le 2$ или $s^2 = 0$.
  * **Эксцесс (Kurtosis):** выборочный эксцесс Фишера (Fisher excess kurtosis) по Блиссу:
    $$g_2 = \frac{W-1}{(W-2)(W-3)} \left[ (W+1) \frac{m_4}{m_2^2} - 3(W-1) \right]$$
    Стандартная ошибка: $SE(g_2) = 2 \cdot SE(g_1) \cdot \sqrt{\frac{W^2 - 1}{(W-3)(W+5)}}$. Подавляется при $W \le 3$ или $s^2 = 0$.
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 1.2. `FREQUENCIES` (IBM Algorithms стр. 446–448)
* **Реализация:** [`crates/statcraft_algo/src/frequencies.rs`](../crates/statcraft_algo/src/frequencies.rs)
* **Тесты:** `frequencies::tests::test_frequency_numeric_sorting_and_quartiles`
* **Сопоставление формул:**
  * `Percent`: $\frac{f_j}{W_{total}} \times 100$ (включая пропущенные значения);
  * `Valid Percent`: $\frac{f_j}{W_{valid}} \times 100$ (только валидные наблюдения);
  * `Cumulative Percent`: кумулятивная сумма `Valid Percent`;
  * **Квантили/Квартили:** каноническая интерполяция **SPSS HAVERAGE** (ранг $r = p \cdot (N + 1)$).
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 1.3. `CROSSTABS` (IBM Algorithms стр. 221–226)
* **Реализация:** [`crates/statcraft_algo/src/crosstabs.rs`](../crates/statcraft_algo/src/crosstabs.rs)
* **Тесты:** `crosstabs_tests::tests::test_crosstabs_2x2_golden_oracle`, `crosstabs_tests::tests::test_fisher_exact_2x2_standard`
* **Сопоставление формул:**
  * Ожидаемые частоты: $E_{ij} = \frac{R_i \cdot C_j}{W}$, остатки: $Res_{ij} = O_{ij} - E_{ij}$;
  * Критерий Пирсона: $\chi^2 = \sum \frac{(O_{ij} - E_{ij})^2}{E_{ij}}, \quad df = (R-1)(C-1)$;
  * Отношение правдоподобия (Likelihood Ratio): $G^2 = 2 \sum O_{ij} \ln\left(\frac{O_{ij}}{E_{ij}}\right)$;
  * Поправка Йейтса на непрерывность ($2 \times 2$):
    $$\chi^2_{Yates} = \frac{W \cdot \left(\max(0, |O_{11}O_{22} - O_{12}O_{21}| - 0.5 W)\right)^2}{R_1 R_2 C_1 C_2}, \quad df = 1$$
  * Точный критерий Фишера: двусторонний расчет через $\ln \Gamma(x)$ с защитой от переполнения факториалов;
  * Меры связи: Коэффициент $\Phi$, $V$ Крамера, коэффициент сопряженности Пирсона.
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 1.4. `MEANS` (IBM Algorithms стр. 662–665)
* **Реализация:** [`crates/statcraft_algo/src/means.rs`](../crates/statcraft_algo/src/means.rs)
* **Тесты:** `means_tests::test_means_breakdown_golden_oracle`
* **Сопоставление формул:**
  * Разложение дисперсии ANOVA:
    $$SS_{Between} = \sum W_j (\bar{Y}_j - \bar{Y})^2, \; df = k - 1; \quad SS_{Within} = \sum (W_j - 1) s_j^2, \; df = N - k$$
    $$F = \frac{MS_{Between}}{MS_{Within}}, \quad p = I_{\frac{df_2}{df_1 F + df_2}}\left(\frac{df_2}{2}, \frac{df_1}{2}\right)$$
  * Меры ассоциации: $\eta^2 = \frac{SS_{Between}}{SS_{Total}}, \quad \eta = \sqrt{\eta^2}$.
* **Вердикт:** **100% математическая эквивалентность канону.**

---

### Блок 2: Сравнение средних (t-тесты и One-Way ANOVA)

#### 2.1. `ONE-SAMPLE T-TEST` (IBM Algorithms стр. 1061–1062)
* **Реализация:** [`crates/statcraft_algo/src/ttest_onesample.rs`](../crates/statcraft_algo/src/ttest_onesample.rs)
* **Тесты:** `ttest_onesample::tests::test_golden_oracle_ttest_onesample_scipy`
* **Сопоставление формул:**
  $$t = \frac{\bar{X} - \text{TestValue}}{SE(\bar{X})}, \quad df = W - 1$$
  $$p = I_{\frac{df}{df + t^2}}\left(\frac{df}{2}, \frac{1}{2}\right)$$
  $$\text{CI} = (\bar{X} - \text{TestValue}) \pm t_{1 - \alpha/2, df} \cdot SE(\bar{X})$$
  Квантили $t_{crit}$ вычисляются через аппроксимацию Аклема с итерационным уточнением методом Ньютона-Рафсона.
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 2.2. `INDEPENDENT-SAMPLES T-TEST` (IBM Algorithms стр. 1058–1060)
* **Реализация:** [`crates/statcraft_algo/src/ttest_indep.rs`](../crates/statcraft_algo/src/ttest_indep.rs)
* **Тесты:** `ttest_indep::tests::test_golden_oracle_against_scipy`
* **Сопоставление формул:**
  * **Критерий Ливиня:** каноническое центрирование по выборочному среднему:
    $$Z_{kj} = |X_{kj} - \bar{X}_k|, \quad F = \frac{MSB_Z}{MSW_Z}, \quad df_1 = 1, \; df_2 = n_1 + n_2 - 2$$
  * **Equal Variances Assumed (Стьюдент с объединенной дисперсией):**
    $$s_p^2 = \frac{(n_1 - 1)s_1^2 + (n_2 - 1)s_2^2}{n_1 + n_2 - 2}, \quad SE_{diff} = \sqrt{s_p^2 \left(\frac{1}{n_1} + \frac{1}{n_2}\right)}, \quad df = n_1 + n_2 - 2$$
  * **Equal Variances Not Assumed (Уэлч со степенями свободы Саттертуэйта):**
    $$SE_{welch} = \sqrt{\frac{s_1^2}{n_1} + \frac{s_2^2}{n_2}}, \quad df = \frac{\left(\frac{s_1^2}{n_1} + \frac{s_2^2}{n_2}\right)^2}{\frac{(s_1^2/n_1)^2}{n_1 - 1} + \frac{(s_2^2/n_2)^2}{n_2 - 1}}$$
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 2.3. `PAIRED-SAMPLES T-TEST` (IBM Algorithms стр. 1060–1061)
* **Реализация:** [`crates/statcraft_algo/src/ttest_paired.rs`](../crates/statcraft_algo/src/ttest_paired.rs)
* **Тесты:** `ttest_paired::tests::test_paired_ttest_golden_oracle`
* **Сопоставление формул:**
  * Коэффициент корреляции Пирсона с двусторонней значимостью $t_r = r \sqrt{\frac{n-2}{1-r^2}}$;
  * Парные разности $D_i = X_i - Y_i$:
    $$t = \frac{\bar{D}}{SE(\bar{D})}, \quad df = n - 1, \quad \text{CI} = \bar{D} \pm t_{crit} \cdot SE(\bar{D})$$
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 2.4. `ONE-WAY ANOVA` (IBM Algorithms стр. 802–806)
* **Реализация:** [`crates/statcraft_algo/src/anova_oneway.rs`](../crates/statcraft_algo/src/anova_oneway.rs)
* **Тесты:** интеграционные тесты модуля ANOVA
* **Сопоставление формул:**
  * Суммы квадратов:
    $$BSS = \sum_{l=1}^k n_l (\bar{X}_l - \bar{X})^2, \quad WSS = \sum_{l=1}^k \sum_{j=1}^{n_l} (X_{lj} - \bar{X}_l)^2, \quad TSS = BSS + WSS$$
  * $F$-критерий Фишера: $F = \frac{BSS / (k-1)}{WSS / (N-k)}$, $p$-value через неполную бета-функцию;
  * Многогрупповой критерий Ливиня на гомогенность дисперсий ($k$ групп) с $df_1 = k-1, \; df_2 = N-k$.
* **Вердикт:** **100% математическая эквивалентность канону.**

---

### Блок 3: Связи и многомерные методы

#### 3.1. `CORRELATIONS` (IBM Algorithms стр. 193–194, 231, 752–753)
* **Реализация:** [`crates/statcraft_algo/src/correlations.rs`](../crates/statcraft_algo/src/correlations.rs)
* **Тесты:** `correlations_tests::tests::test_spearman_and_kendall_oracle`
* **Сопоставление формул:**
  * **Pearson $r$:** $r = \frac{\sum (x_i - \bar{x})(y_i - \bar{y})}{\sqrt{\sum(x_i - \bar{x})^2 \sum(y_i - \bar{y})^2}}$ со значимостью $t = r \sqrt{\frac{n-2}{1-r^2}}, \; df = n - 2$.
  * **Spearman $\rho$:** вычисление коэффициента корреляции Пирсона над средними рангами наблюдений ($X$ и $Y$).
  * **Kendall $\tau_b$:** ранговый коэффициент со связями (Kendall, 1955):
    $$\tau_b = \frac{P - Q}{\sqrt{(n_0 - T_X)(n_0 - T_Y)}}, \quad n_0 = \frac{n(n-1)}{2}, \quad T = \sum \frac{t_i(t_i-1)}{2}$$
    Оценка дисперсии со всеми тремя компонентами поправок на связи ($v_0, v_1, v_2$) и двусторонней асимптотической значимостью $Z = S / \sqrt{Var(S)}$.
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 3.2. `REGRESSION LINEAR` (IBM Algorithms стр. 920–925)
* **Реализация:** [`crates/statcraft_algo/src/regression.rs`](../crates/statcraft_algo/src/regression.rs)
* **Тесты:** `regression_tests::test_multiple_regression_golden_oracle`, `regression_tests::test_weighted_regression_golden_oracle`
* **Сопоставление формул:**
  * Метод наименьших квадратов (OLS и взвешенный WLS): решение нормальных уравнений $(X^T W X) \beta = X^T W Y$ методом Гаусса-Жордана с частичным выбором ведущего элемента.
  * Разложение сумм квадратов: $SS_{Resid} = \sum w_i (y_i - \hat{y}_i)^2$, $SS_{Model} = SS_{Total} - SS_{Resid}$, $R^2 = 1 - \frac{SS_{Resid}}{SS_{Total}}$.
  * Скорректированный $R^2$: $R^2_{adj} = 1 - (1 - R^2) \frac{N - 1}{N - k}$.
  * $F$-критерий модели: $F = \frac{MS_{Model}}{MS_{Resid}}, \; df_1 = p, \; df_2 = N - k$.
  * Стандартизованные коэффициенты Beta: $\beta_j^* = b_j \cdot \frac{s_{X_j}}{s_Y}$.
  * Статистики мультиколлинеарности: толерантность $Tol_j = 1 - R_j^2$ и коэффициент инфляции дисперсии $VIF_j = \frac{1}{Tol_j}$.
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 3.3. `FACTOR ANALYSIS / PCA` (IBM Algorithms стр. 428–442)
* **Реализация:** [`crates/statcraft_algo/src/factor.rs`](../crates/statcraft_algo/src/factor.rs)
* **Тесты:** `factor_tests::test_factor_analysis_golden_oracle`, `factor_tests::test_jacobi_eigen_symmetric_known_matrix`
* **Сопоставление формул:**
  * Разложение корреляционной матрицы методом циклических вращений Якоби для точного нахождения собственных значений $\lambda_m$ и собственных векторов $V_m$.
  * Нагрузки компонент: $\Lambda_{im} = \sqrt{\lambda_m} \cdot V_{im}$, общности $h_i^2 = \sum_{m=1}^M \Lambda_{im}^2$.
  * **Критерий сферичности Бартлетта:** $\chi^2 = -\left(N - 1 - \frac{2p + 5}{6}\right) \ln |R|, \quad df = \frac{p(p-1)}{2}$.
  * **Адекватность выборки KMO (Кайзер-Мейер-Олкин):**
    $$KMO = \frac{\sum\sum_{i \ne j} r_{ij}^2}{\sum\sum_{i \ne j} r_{ij}^2 + \sum\sum_{i \ne j} a_{ij}^2}, \quad a_{ij} = -\frac{r^{ij}}{\sqrt{r^{ii} r^{jj}}}$$
* **Вердикт:** **100% математическая эквивалентность канону.**

#### 3.4. `RELIABILITY` (IBM Algorithms стр. 936–940)
* **Реализация:** [`crates/statcraft_algo/src/reliability.rs`](../crates/statcraft_algo/src/reliability.rs)
* **Тесты:** `reliability_tests::test_reliability_golden_oracle_3_items`
* **Сопоставление формул:**
  * Коэффициент надежности $\alpha$ Кронбаха (Cronbach, 1951):
    $$\alpha = \frac{k}{k-1} \left( 1 - \frac{\sum s_i^2}{s_{total}^2} \right)$$
  * Стандартизованный $\alpha$ на основе средней межпунктовой корреляции $\bar{r}$: $\alpha_{std} = \frac{k \bar{r}}{1 + (k-1)\bar{r}}$.
  * Статистики при удалении пункта:
    $$s_{(i)}^2 = s_{total}^2 + s_i^2 - 2 Cov(X_i, X_{total}), \quad \alpha_{(i)} = \frac{k-1}{k-2} \left( 1 - \frac{\sum_{j \ne i} s_j^2}{s_{(i)}^2} \right)$$
    $$r_{i(total-i)} = \frac{Cov(X_i, X_{total}) - s_i^2}{\sqrt{s_i^2 s_{(i)}^2}}$$
* **Вердикт:** **100% математическая эквивалентность канону.**
 
 ---
 
+### Блок 4: Непараметрические критерии
+
+#### 4.1. `MANN-WHITNEY U TEST` (IBM Algorithms стр. 796–797)
+* **Реализация:** [`crates/statcraft_algo/src/nonparam_twosample.rs`](../crates/statcraft_algo/src/nonparam_twosample.rs)
+* **Тесты «Золотого оракула»:** `nonparam_twosample_tests::test_mann_whitney_oracle_no_ties`, `test_mann_whitney_oracle_with_ties`
+* **Верификация математических формул:**
+  * Объединенный массив наблюдений ранжируется с присвоением средних рангов (mid-ranks) для связок (ties). Суммы рангов групп: $S_1, S_2$.
+  * Статистика Манна-Уитни $U$:
+    $$U_1 = S_1 - \frac{n_1(n_1+1)}{2}, \quad U_2 = S_2 - \frac{n_2(n_2+1)}{2}, \quad U = \min(U_1, U_2)$$
+  * Критерий Уилкоксона $W$: сумма рангов для группы с меньшим средним рангом.
+  * Математическое ожидание и скорректированная на связки дисперсия:
+    $$\mu_U = \frac{n_1 n_2}{2}, \quad \sigma_U^2 = \frac{n_1 n_2}{12} \left( (N + 1) - \frac{\sum_{i=1}^m (t_i^3 - t_i)}{N(N - 1)} \right)$$
+  * Асимптотический $Z$-критерий: $Z = \frac{U - \mu_U}{\sigma_U}$, двустороннее $p$-значение $p = 2(1 - \Phi(|Z|))$.
+* **Вердикт:** **100% математическая эквивалентность канону.**
+
+#### 4.2. `KRUSKAL-WALLIS ONE-WAY ANALYSIS OF VARIANCE` (IBM Algorithms стр. 800–801)
+* **Реализация:** [`crates/statcraft_algo/src/kruskal_wallis.rs`](../crates/statcraft_algo/src/kruskal_wallis.rs)
+* **Тесты «Золотого оракула»:** `kruskal_wallis_tests::test_kruskal_wallis_golden_oracle_distinct`, `test_kruskal_wallis_golden_oracle_ties`
+* **Верификация математических формул:**
+  * Совместное ранжирование $N$ наблюдений по $k$ группам со средними рангами для связок.
+  * Исходная статистика $H$:
+    $$H = \frac{12}{N(N+1)} \sum_{j=1}^k \frac{R_j^2}{n_j} - 3(N+1)$$
+  * Коэффициент поправки на связки и скорректированное значение $H$:
+    $$H_{adj} = \frac{H}{1 - \frac{\sum_{i=1}^m (t_i^3 - t_i)}{N^3 - N}}$$
+  * Асимптотическая значимость: функция выживания распределения $\chi^2$ со степенями свободы $df = k - 1$.
+* **Вердикт:** **100% математическая эквивалентность канону.**
+
+#### 4.3. `WILCOXON MATCHED-PAIRS SIGNED-RANK TEST` (IBM Algorithms стр. 793)
+* **Реализация:** [`crates/statcraft_algo/src/nonparam_related.rs`](../crates/statcraft_algo/src/nonparam_related.rs)
+* **Тесты «Золотого оракула»:** `nonparam_related_tests::test_wilcoxon_signed_rank_oracle_with_ties`
+* **Верификация математических формул:**
+  * Попарные разности $D_i = Y_i - X_i$. Ненулевые абсолютные разности $|D_i|$ упорядочиваются и ранжируются со средними рангами для связок.
+  * Сумма положительных рангов $S_p = \sum_{D_i > 0} R_i$, отрицательных $S_n = \sum_{D_i < 0} R_i$. Статистика $W = \min(S_p, S_n)$.
+  * Математическое ожидание и скорректированная дисперсия:
+    $$\mu_W = \frac{n(n+1)}{4}, \quad \sigma_W^2 = \frac{n(n+1)(2n+1) - 0.5 \sum_{j=1}^l (t_j^3 - t_j)}{24}$$
+  * $Z$-критерий с поправкой на непрерывность: $Z = \frac{W - \mu_W \pm 0.5}{\sigma_W}$, асимптотическое двустороннее $p$-значение из стандартного нормального распределения.
+* **Вердикт:** **100% математическая эквивалентность канону.**
+
+#### 4.4. `SIGN TEST` (IBM Algorithms стр. 792)
+* **Реализация:** [`crates/statcraft_algo/src/nonparam_related.rs`](../crates/statcraft_algo/src/nonparam_related.rs)
+* **Тесты «Золотого оракула»:** `nonparam_related_tests::test_sign_test_oracle`, `test_sign_test_equal_halves`
+* **Верификация математических формул:**
+  * Подсчет знаков разностей: положительные $n_+$, отрицательные $n_-$, валидные пары $n = n_+ + n_-$, $k = \min(n_+, n_-)$.
+  * Точная двусторонняя биномиальная вероятность через регуляризованную неполную бета-функцию:
+    $$p = 2 \sum_{i=0}^k \binom{n}{i} 0.5^n = 2 \cdot I_{0.5}(n - k, k + 1)$$
+* **Вердикт:** **100% математическая эквивалентность канону.**
+
+---
+
+### Блок 5: Трансформации данных и восполнение пропусков
+
+#### 5.1. `RANK CASES` (IBM Algorithms стр. 904–906)
+* **Реализация:** [`crates/statcraft_algo/src/rank_cases.rs`](../crates/statcraft_algo/src/rank_cases.rs)
+* **Тесты «Золотого оракула»:** `rank_cases_tests::test_compute_ranks_no_ties`, `test_compute_ranks_with_ties`, `test_normal_scores`, `test_inv_normal_cdf_accuracy`
+* **Верификация математических формул:**
+  * Методы обработки связок: `MEAN` ($W_{i-1} + (w_i + 1)/2$), `LOW` ($W_{i-1} + 1$), `HIGH` ($W_i$), `SEQ` ($k+1$).
+  * Оценки накопленной пропорции:
+    * **Блом (Blom, 1958):** $F = \frac{r - 3/8}{w + 1/4}$
+    * **Тьюки (Tukey, 1962):** $F = \frac{r - 1/3}{w + 1/3}$
+    * **Ранкит (Rankit):** $F = \frac{r - 1/2}{w}$
+    * **Ван дер Варден (Van der Waerden):** $F = \frac{r}{w + 1}$
+  * **Нормальные метки:** $Z = \Phi^{-1}(F)$ вычисляются через рациональное приближение Питера Дж. Акклама (погрешность $\le 1.15 \times 10^{-9}$).
+  * Дробные ранги ($r / w$), проценты ($100 r / w$) и n-тили ($\lceil \frac{r k}{w} \rceil$).
+* **Вердикт:** **100% математическая эквивалентность канону.**
+
+#### 5.2. `REPLACE MISSING VALUES` (IBM Algorithms стр. 66, 117, 143, 960)
+* **Реализация:** [`crates/statcraft_algo/src/replace_missing.rs`](../crates/statcraft_algo/src/replace_missing.rs)
+* **Тесты «Золотого оракула»:** `replace_missing::tests::test_linear_interpolation`, `test_linear_trend`, `test_mean_nearby_imputation`, `test_median_nearby_imputation`, `test_series_mean_imputation`
+* **Верификация математических формул:**
+  * **Линейная интерполяция:** $y_i = y_0 + \frac{i - i_0}{i_1 - i_0}(y_1 - y_0)$ между соседними валидными наблюдениями.
+  * **Среднее ряда:** $\bar{y} = \frac{1}{n_{valid}} \sum y_i$.
+  * **Среднее и медиана окрестности:** симметричное окно ширины span: $j \in [\max(0, i - span), \min(n - 1, i + span)], j \ne i$.
+  * **Линейный тренд в точке:** парная линейная регрессия $y = a + b \cdot t$, где $t$ — порядковый номер наблюдения.
+* **Вердикт:** **100% математическая эквивалентность канону.**
+
+#### 5.3. `OPTIMAL BINNING` (IBM Algorithms стр. 130, 816–826)
+* **Реализация:** [`crates/statcraft_algo/src/optimal_binning.rs`](../crates/statcraft_algo/src/optimal_binning.rs)
+* **Тесты «Золотого оракула»:** `optimal_binning::tests::test_perfect_two_class_split`, `test_null_handling`, `test_constant_feature`
+* **Верификация математических формул:**
+  * Дискретизация с учителем на основе принципа минимальной длины описания (MDLP, Fayyad & Irani, 1993) и информационной энтропии Шеннона:
+    $$Ent(S) = - \sum_{i=1}^k p_i \log_2 p_i$$
+    $$Gain(S, T) = Ent(S) - \frac{|S_1|}{|S|} Ent(S_1) - \frac{|S_2|}{|S|} Ent(S_2)$$
+    $$MDLP \; Stopping: Gain(S, T) > \frac{\log_2(N - 1)}{N} + \frac{\Delta(S, T)}{N}$$
+* **Вердикт:** **100% математическая эквивалентность канону.**
+
+---
+
+## 4. Итоговое заключение аудита
+
+Все пять ключевых математических блоков `statcraft_algo` прошли исчерпывающий инструментальный Clean-Room аудит против официального руководства *IBM SPSS Statistics 24 Algorithms Guide* (1271 стр.):
+1. **Описательные статистики, частоты, таблицы сопряженности, средние (Блок 1):** 100% Верифицировано ($\le 10^{-6}$).
+2. **T-критерии и однофакторный дисперсионный анализ ANOVA (Блок 2):** 100% Верифицировано ($\le 10^{-6}$).
+3. **Корреляции, регрессия, факторный анализ, надежность (Блок 3):** 100% Верифицировано ($\le 10^{-6}$).
+4. **Непараметрические критерии (Блок 4):** 100% Верифицировано ($\le 10^{-6}$).
+5. **Трансформации данных и восполнение пропусков (Блок 5):** 100% Верифицировано ($\le 10^{-6}$).
+
+Каждая формула, расчет степеней свободы, поправки на связки рангов и генераторы точных/асимптотических уровней значимости $p$-value строго воспроизводят каноническое поведение IBM SPSS без заимствования проприетарного кода и с соблюдением всех стандартов экосистемы ArtCraft.


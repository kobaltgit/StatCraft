export interface AuditBlock {
  id: string;
  number: number;
  titleEn: string;
  titleRu: string;
  procedures: string[];
  referencePage: string;
  implementationFiles: string[];
  status: "verified" | "review";
  oraclePrecision: string;
  formulas: {
    name: string;
    latex: string;
    descriptionEn: string;
    descriptionRu: string;
  }[];
  notesEn: string;
  notesRu: string;
}

export const AUDIT_BLOCKS: AuditBlock[] = [
  {
    id: "block-1",
    number: 1,
    titleEn: "Descriptives, Frequencies & Crosstabs",
    titleRu: "Описательные статистики, частоты и таблицы сопряженности",
    procedures: ["DESCRIPTIVES", "FREQUENCIES", "CROSSTABS", "MEANS"],
    referencePage: "pp. 368–370, 446–448, 221–226",
    implementationFiles: ["descriptives.rs", "frequencies.rs", "crosstabs.rs", "means.rs"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶ (SciPy & IBM Canon)",
    formulas: [
      {
        name: "Spicer Moments",
        latex: "\\bar{X} = \\frac{\\sum w_i X_i}{W}, \\quad s^2 = \\frac{1}{W-1}\\sum w_i (X_i - \\bar{X})^2",
        descriptionEn: "Recursive moments weighting via Spicer (1972)",
        descriptionRu: "Рекуррентные моменты с весами по Спайсеру (1972)",
      },
      {
        name: "Bliss Skewness & Kurtosis",
        latex: "g_1 = \\frac{W}{(W-1)(W-2)} \\cdot \\frac{\\sum w_i (X_i - \\bar{X})^3}{s^3}",
        descriptionEn: "Unbiased sample skewness and Fisher excess kurtosis with SE",
        descriptionRu: "Несмещенная выборочная асимметрия и эксцесс Блисса со стандартными ошибками",
      },
      {
        name: "Yates Continuity & Fisher Exact",
        latex: "\\chi^2_{Yates} = \\frac{W (|O_{11}O_{22} - O_{12}O_{21}| - 0.5W)^2}{R_1 R_2 C_1 C_2}",
        descriptionEn: "2x2 table continuity correction & exact hypergeometric test",
        descriptionRu: "Поправка Йейтса на непрерывность и точный критерий Фишера",
      },
    ],
    notesEn: "HAVERAGE canonical quantile interpolation r = p*(N+1) verified.",
    notesRu: "Квартили HAVERAGE r = p*(N+1) верифицированы.",
  },
  {
    id: "block-2",
    number: 2,
    titleEn: "Compare Means & T-Tests",
    titleRu: "Сравнение средних и Т-критерии",
    procedures: ["T-TEST 1-SAMPLE", "T-TEST INDEP", "T-TEST PAIRED", "ONE-WAY ANOVA"],
    referencePage: "pp. 1061–1065, 804–812",
    implementationFiles: ["ttest_onesample.rs", "ttest_indep.rs", "ttest_paired.rs", "anova_oneway.rs"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶ (SciPy / Statsmodels)",
    formulas: [
      {
        name: "Welch-Satterthwaite df",
        latex: "df = \\frac{(s_1^2/n_1 + s_2^2/n_2)^2}{\\frac{(s_1^2/n_1)^2}{n_1 - 1} + \\frac{(s_2^2/n_2)^2}{n_2 - 1}}",
        descriptionEn: "Degrees of freedom under unequal variances",
        descriptionRu: "Степени свободы Саттертуэйта при нарушении равенства дисперсий",
      },
      {
        name: "Levene Homogeneity Test",
        latex: "F = \\frac{(N - k) \\sum n_i (\\bar{z}_{i\\cdot} - \\bar{z}_{\\cdot\\cdot})^2}{(k - 1) \\sum (z_{ij} - \\bar{z}_{i\\cdot})^2}",
        descriptionEn: "Absolute residual ANOVA for homogeneity of variance",
        descriptionRu: "Критерий однородности дисперсий Левена",
      },
    ],
    notesEn: "Dual output: Equal Variances Assumed & Satterthwaite robust test.",
    notesRu: "Двойной вывод: критерий Стьюдента и робастный критерий Уэлча.",
  },
  {
    id: "block-3",
    number: 3,
    titleEn: "Bivariate Correlations & Linear Regression",
    titleRu: "Парные корреляции и линейная регрессия",
    procedures: ["CORRELATIONS", "REGRESSION OLS", "FACTOR", "RELIABILITY"],
    referencePage: "pp. 196–203, 891–904, 395–404",
    implementationFiles: ["correlations.rs", "regression.rs", "factor.rs", "reliability.rs"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶",
    formulas: [
      {
        name: "Kendall Tau-b with Ties",
        latex: "\\tau_b = \\frac{P - Q}{\\sqrt{(v_0 - v_1)(v_0 - v_2)}}",
        descriptionEn: "Rank correlation accounting for all tie corrections",
        descriptionRu: "Ранговая корреляция Кендалла с тремя поправками на связки рангов",
      },
      {
        name: "OLS Matrix Solution",
        latex: "\\hat{\\beta} = (X^T X)^{-1} X^T y, \\quad s_e^2 = \\frac{e^T e}{N - p}",
        descriptionEn: "Ordinary least squares coefficients, tolerance & VIF diagnostics",
        descriptionRu: "Оценки коэффициентов МНК, толерантность и фактор инфляции дисперсии VIF",
      },
    ],
    notesEn: "Cronbach's alpha and principal axis factor analysis verified.",
    notesRu: "Альфа Кронбаха и факторный анализ верифицированы.",
  },
  {
    id: "block-4",
    number: 4,
    titleEn: "Nonparametric Tests",
    titleRu: "Непараметрические критерии",
    procedures: ["MANN-WHITNEY U", "KRUSKAL-WALLIS H", "WILCOXON", "SIGN"],
    referencePage: "pp. 627–634, 555–559, 793–795",
    implementationFiles: ["nonparam_twosample.rs", "kruskal_wallis.rs", "nonparam_related.rs"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶",
    formulas: [
      {
        name: "Mann-Whitney U",
        latex: "U = n_1 n_2 + \\frac{n_1(n_1 + 1)}{2} - R_1, \\quad Z = \\frac{U - E(U)}{\\sigma_U}",
        descriptionEn: "Mid-rank tie adjustments T = sum(t_i^3 - t_i)",
        descriptionRu: "Критерий Манна-Уитни со средними рангами и поправками связок",
      },
      {
        name: "Kruskal-Wallis Adjusted H",
        latex: "H_{adj} = \\frac{H}{1 - \\sum (t_i^3 - t_i) / (N^3 - N)}",
        descriptionEn: "K-group rank variance ratio with tie correction factor",
        descriptionRu: "Критерий Краскела-Уоллиса со скорректированной дисперсией",
      },
    ],
    notesEn: "Wilcoxon signed-rank and exact binomial sign tests verified.",
    notesRu: "Знаково-ранговый критерий Вилкоксона и критерий знаков подтверждены.",
  },
  {
    id: "block-16",
    number: 16,
    titleEn: "Advanced Regression & Partial Correlations",
    titleRu: "Продвинутая регрессия и частные корреляции",
    procedures: ["PARTIAL CORR", "LOGISTIC REGRESSION", "CURVEFIT"],
    referencePage: "pp. 797–801, 586–598, 248–254",
    implementationFiles: ["partial_corr/", "logistic/", "curvefit/"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶ (Statsmodels / SciPy)",
    formulas: [
      {
        name: "Schur Complement Partial Corr",
        latex: "S = R_{XX} - R_{XZ} R_{ZZ}^{-1} R_{ZX}, \\quad r_{ij.Z} = \\frac{S_{ij}}{\\sqrt{S_{ii} S_{jj}}}",
        descriptionEn: "Partial correlations via inverted controlling block and t-test df = N - k - 2",
        descriptionRu: "Частные корреляции через обращение блока ковариат и t-тест df = N - k - 2",
      },
      {
        name: "Binary Logistic IRLS",
        latex: "\\beta^{(t+1)} = \\beta^{(t)} + (X^T W X)^{-1} X^T (y - \\pi)",
        descriptionEn: "Newton-Raphson scoring with Hosmer-Lemeshow H (df = 8) and classification table",
        descriptionRu: "IRLS оптимизация с тестом согласия Хосмера-Лемешова и матрицей классификации",
      },
    ],
    notesEn: "6 curve estimation models: linear, log, inverse, quadratic, power, exponential.",
    notesRu: "6 криволинейных моделей CURVEFIT подтверждены.",
  },
  {
    id: "block-17",
    number: 17,
    titleEn: "General Linear Model (GLM)",
    titleRu: "Общая линейная модель (GLM)",
    procedures: ["UNIANOVA", "GLM MULTIVARIATE", "GLM REPEATED"],
    referencePage: "pp. 476–505, 506–528, 529–548",
    implementationFiles: ["glm/univariate.rs", "glm/multivariate.rs", "glm/repeated.rs"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶",
    formulas: [
      {
        name: "Type III Sum of Squares",
        latex: "SS(\\text{Type III}) = R(E \\mid \\text{all other effects}), \\quad \\text{Partial } \\eta^2 = \\frac{SS_E}{SS_E + SS_{\\text{Error}}}",
        descriptionEn: "Marginal reduction of Yates' weighted squares of means",
        descriptionRu: "Суммы квадратов типа III через маргинальную редукцию моделей Йейтса",
      },
      {
        name: "MANOVA Spectral Analysis",
        latex: "\\Lambda = \\prod_{i=1}^s \\frac{1}{1 + \\lambda_i}, \\quad V = \\sum \\frac{\\lambda_i}{1 + \\lambda_i}",
        descriptionEn: "Wilks' Lambda, Pillai, Hotelling-Lawley, and Roy's Largest Root via E^-1/2 H E^-1/2",
        descriptionRu: "4 канонических критерия MANOVA через обобщенные собственные значения",
      },
      {
        name: "Mauchly's Sphericity Test",
        latex: "W = \\frac{\\det(M S M^T)}{[\\frac{1}{k-1} \\text{tr}(M S M^T)]^{k-1}}",
        descriptionEn: "Helmert orthonormal contrast transform with Greenhouse-Geisser epsilon",
        descriptionRu: "Критерий сферичности Мокли с поправкой Гринхауса-Гейссера",
      },
    ],
    notesEn: "Orthogonal deviation contrasts and Jacobi SVD solver ensure stability.",
    notesRu: "Ортогональные контрасты отклонений и Jacobi SVD солвер.",
  },
  {
    id: "block-18",
    number: 18,
    titleEn: "Clustering & Survival Analysis",
    titleRu: "Кластерный анализ и анализ выживаемости",
    procedures: ["QUICK CLUSTER", "CODEBOOK", "SUMMARIZE", "KM SURVIVAL", "LIFE TABLES"],
    referencePage: "pp. 838–844, 574–585, 608–618",
    implementationFiles: ["cluster_kmeans.rs", "survival_km.rs", "survival_lifetables.rs"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶ (Lifelines / SciPy)",
    formulas: [
      {
        name: "Kaplan-Meier Survival & Greenwood",
        latex: "\\hat{S}(t) = \\prod_{t_i \\le t} \\left(1 - \\frac{d_i}{n_i}\\right), \\quad \\widehat{\\text{Var}}(\\hat{S}(t)) = [\\hat{S}(t)]^2 \\sum_{t_i \\le t} \\frac{d_i}{n_i (n_i - d_i)}",
        descriptionEn: "Product-limit survival function with Greenwood's variance and Mantel-Cox log-rank test",
        descriptionRu: "Функция дожития Каплана-Мейера с дисперсией Гринвуда и лог-ранговым критерием",
      },
    ],
    notesEn: "K-Means deterministic Maximin cluster seed initialization.",
    notesRu: "Детерминированная инициализация центров Maximin в K-Means.",
  },
  {
    id: "block-19",
    number: 19,
    titleEn: "Multinomial Logistic Regression",
    titleRu: "Мультиномиальная логистическая регрессия",
    procedures: ["NOMREG"],
    referencePage: "pp. 675–692",
    implementationFiles: ["multinomial_logistic/"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶ (Statsmodels MNLogit)",
    formulas: [
      {
        name: "Generalized Logit Model",
        latex: "\\pi_{ij} = \\frac{\\exp(\\beta_j^T x_i)}{1 + \\sum_{k=1}^{J-1} \\exp(\\beta_k^T x_i)}, \\quad \\pi_{iJ} = \\frac{1}{1 + \\sum_{k=1}^{J-1} \\exp(\\beta_k^T x_i)}",
        descriptionEn: "Newton-Raphson / Fisher Scoring on faer with Ridge Hessian damping lambda = 1e-6",
        descriptionRu: "Обобщенные логиты с базовой категорией и Ридж-регуляризацией Never Crash",
      },
    ],
    notesEn: "Likelihood Ratio Tests per effect, subpopulation Pearson/Deviance goodness-of-fit.",
    notesRu: "LRT тесты эффектов, критерии Пирсона и девианса по субпопуляциям.",
  },
  {
    id: "block-20",
    number: 20,
    titleEn: "Cox Proportional Hazards Regression",
    titleRu: "Регрессия пропорциональных рисков Кокса",
    procedures: ["COXREG"],
    referencePage: "pp. 265–275",
    implementationFiles: ["cox/baseline.rs", "cox/engine.rs", "cox/mod.rs"],
    status: "verified",
    oraclePrecision: "≤ 10⁻⁶ (Lifelines CoxPHFitter)",
    formulas: [
      {
        name: "Partial Likelihood with Ties",
        latex: "\\ln L_p(\\beta) = \\sum_{s=1}^S \\sum_{k=1}^{K_s} [\\beta^T s_{sk} - d_{sk} \\ln S^{(0)}_{sk}(\\beta)]",
        descriptionEn: "Newton-Raphson partial likelihood optimization with Breslow & Efron tie handling",
        descriptionRu: "Оптимизация частичного правдоподобия с обработкой связок по Бреслоу и Эфрону",
      },
      {
        name: "Breslow Baseline Hazard",
        latex: "\\hat{H}_{0s}(t) = \\sum_{t_{(k)} \\le t} \\frac{d_k}{\\sum_{j \\in R_s(t_{(k)})} \\exp(\\hat{\\beta}^T x_j)}",
        descriptionEn: "Cumulative baseline hazard function and step survival curves across clinical strata",
        descriptionRu: "Кумулятивный базовый риск по Бреслоу и ступенчатая функция выживаемости",
      },
    ],
    notesEn: "Stratification across clinical groups, Wald test 95% CI for hazard ratios Exp(B).",
    notesRu: "Стратификация клинических групп и 95% ДИ Вальда для отношения рисков Exp(B).",
  },
];

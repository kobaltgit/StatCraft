---
name: statcraft-assistant
description: Interact with the StatCraft statistical computing application via the Code Mode MCP bridge for SPSS syntax execution, dataset manipulation, variable metadata auditing, and reporting.
---

# StatCraft Assistant Skill (Autonomous Code Mode)

This skill enables AI agents to control and inspect the active **StatCraft** desktop application session using an autonomous **Code Mode (SPSS-syntax REPL)**.

---

## 1. Code Mode REPL Architecture

Instead of issuing dozens of slow, fragmented micro-tools, agents use batch SPSS syntax (`statcraft_run_syntax`) or direct tabular loading (`statcraft_load_dataset`).

* **Instant Output Return**: `statcraft_run_syntax` directly returns the ASCII/Unicode tabular output (Pivot Tables) of all generated procedures (ANOVA, Descriptives, Frequencies, T-Tests, Regression, Reliability) in the tool response! The agent immediately sees means, degrees of freedom, test statistics, and exact $p$-values in its context.
* **Self-Correction & Dynamic Self-Documentation**: The compiled StatCraft binary is the single source of truth:
  - Run `HELP.` to see the entire directory of supported categories and commands.
  - Run `HELP <COMMAND>.` (e.g. `HELP ONEWAY.`, `HELP T-TEST.`) to get the exact syntax template and available subcommands.
  - Or inspect the dynamic MCP resource `statcraft://syntax-reference`.
  - If a command is misspelled, the syntax parser returns a `"Did you mean ...?"` suggestion.

---

## 2. Tools Reference

### 1. `statcraft_run_syntax` (Primary Batch Execution Engine)
Executes a batch SPSS syntax script (`.sps`) in StatCraft and returns formatted result tables.
- **Parameters**:
  - `syntax` (string, required): Full SPSS syntax script. Each command MUST end with a period (`.`).
  - `timeout_seconds` (integer, optional): Timeout in seconds (default: 45, max: 300).
- **Universal Code Mode Directive**:
  - Any core command can be directly invoked in syntax via:
    `RUN COMMAND "<command_id>" { <json_params> }.`
    Example: `RUN COMMAND "transform.shift_values" { "source_var": "salary", "target_var": "salary_lag", "direction": "lag", "step": 1 }.`
- **Core Procedures**:
  - **Self-Documentation**: `HELP.` or `HELP <COMMAND>.` (e.g. `HELP ONEWAY.`).
  - **Variable Properties**:
    - `VARIABLE LEVEL varlist (SCALE | ORDINAL | NOMINAL).`
    - `VARIABLE LABELS var 'Descriptive Label'.`
    - `VALUE LABELS var 1 'Male' 2 'Female'.`
    - `MISSING VALUES var (99, 999).`
  - **Transformations**: `COMPUTE target = expr.`, `SORT CASES BY var (A).`, `SPLIT FILE BY var.`, `FILTER BY var.`, `WEIGHT BY var.`
  - **Statistical Procedures**: `DESCRIPTIVES`, `FREQUENCIES`, `CROSSTABS`, `MEANS`, `T-TEST`, `ONEWAY`, `CORRELATIONS`, `REGRESSION`, `RELIABILITY`, `FACTOR`.

### 2. `statcraft_list_commands` (Dynamic Introspection)
Inspects all available statistical procedures, transformations, and utilities compiled into StatCraft.
- Returns command IDs, menu hierarchy, syntax keywords, and complete JSON parameter schemas.
- Supports optional `"category"` filter (e.g. `"Statistics"`, `"Transform"`, `"Data"`).
- Dynamic resource equivalent: read `statcraft://commands` for full Markdown catalog.

### 3. `statcraft_execute_command` (Universal Direct Command Execution)
Directly executes any StatCraft engine command by its `command_id` and JSON arguments.
- Automatically validates parameters according to the command's JSON schema.
- Records output into Output Viewer and returns structured JSON plus formatted output text.
- Can invoke any current or future procedure without code changes in the bridge!

### 4. `statcraft_load_dataset`
Directly loads raw tabular text (CSV, TSV) into the active session memory without writing temporary files to disk.
- **Parameters**: `csv_content` (string, required), `name` (string, optional).

### 5. `statcraft_list_variables`
Inspects the variable dictionary of the active dataset: names, types, measures, labels, value labels, and missing value rules.

### 6. `statcraft_get_data_preview`
Fetches the first $N$ rows (default 10) to inspect value distributions before running analysis.
- **Parameters**: `limit` (integer, optional).

### 7. `statcraft_insert_output_note`
Inserts an analytical card or APA-style conclusion into the StatCraft Output Viewer tree.
- **Parameters**: `title` (string, required), `content` (string, required).

### 8. `statcraft_set_active_tab`
Switches active view tab in StatCraft (`"data"`, `"variable"`, `"output"`, `"syntax"`).

---

## 3. Recommended Workflow

1. **Inspect Data**: Call `statcraft_list_variables` or `statcraft_get_data_preview`.
2. **Consult Syntax if Needed**: Run `HELP <COMMAND>.` via `statcraft_run_syntax` to verify available subcommands.
3. **Execute Analysis in Batch**:
   ```sps
   VARIABLE LEVEL salary (SCALE) /jobcat (NOMINAL).
   ONEWAY salary BY jobcat /STATISTICS DESCRIPTIVES HOMOGENEITY.
   ```
4. **Interpret Output**: Read the returned ASCII ANOVA and Descriptives tables directly from the tool response.
5. **Summarize Findings**: Insert an analytical note into the report tree via `statcraft_insert_output_note`.

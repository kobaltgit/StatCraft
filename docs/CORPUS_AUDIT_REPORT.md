# StatCraft — Real-World Corpora & Codecs Audit Report (Phase 4 / Codecs Audit)

## Executive Summary
This document provides the formal clean-room evidence report of the data ingestion and export subsystem `crates/statcraft_io` (SPSS `.sav`/`.zsav`, Excel `.xlsx`/`.xls`/`.ods`, CSV/TSV RFC 4180, and StatCraft `.stc` container) across real-world datasets, synthetically generated edge cases, and malformed fuzzing streams.

The audit was conducted using the **Craft Anything MCP** in clean-room mode (`clean-room: true`), evaluating conformance against the *GNU PSPP System File Format Specification*, *RFC 4180*, and the *ArtCraft Never Crash Invariant*.

---

## 1. Audit Matrix & Findings (Microplans C-01 — C-05)

| Microplan | Subsystem / Target | Target Invariant | Finding & Verification Status |
|---|---|---|---|
| **C-01** | SPSS `.sav` / `.zsav` Bytecode Stream | Full opcode matrix (0..255), ZLIB block decompression | **RESOLVED (BUG-003):** Fixed opcode 254 decoding to numeric `0.0` (previously decoded to `Null`). Verified opcode 255 as `SYSMIS`, bias subtraction for codes 1..251, literal 253, and EOF 252. Decompressed `.zsav` blocks via zheader/ztrailer offsets. |
| **C-02** | SPSS Extended Record 7 & Encodings | Record 7 subtypes 3, 11, 14, 20; CP1251 transcoding | **RESOLVED (BUG-004):** Corrected Record 7 subtype matching to canonical Subtype 11 for variable display parameters (measurement level, column width, alignment). Implemented Subtype 14 for Very Long Strings ($> 255$ bytes) across continuation slots. Added zero-dependency safe transcoders in `encoding.rs` for Windows-1251 (Cyrillic), Windows-1252, and KOI8-R. |
| **C-03** | Excel Worksheets Ingestion (`.xlsx`, `.xls`, `.ods`) | Dirty columns casting, formula error cells, extreme dates | **VERIFIED:** Mixed columns with numbers and text safely cast to `RawDataType::Text`. Formula error cells (`#VALUE!`, `#REF!`, `#DIV/0!`, `#N/A`, `#NAME?`) and missing markers (`"-"`, `"none"`, `"NA"`) map to `RawValue::Null`. Column headers sanitized and deduplicated per SPSS rules. |
| **C-04** | CSV / TSV Ingestion & Export | RFC 4180 multiline, quote escapes, BOM, roundtrip | **VERIFIED:** Multiline quoted fields (`"line 1\nline 2"`) and escaped quotes (`""`) preserved. Automatic delimiter detection (`,`, `;`, `\t`, `|`). Stripping of UTF-8 BOM (`\u{FEFF}`) from headers. Mathematical roundtrip invariant $\text{RawTable} \equiv \text{reload}(\text{export}(\text{RawTable}))$ verified. |
| **C-05** | Malformed Input Fuzzing & Corpus Runner | Never Crash invariant: 0 panics on corrupted inputs | **VERIFIED:** Implemented `cargo xtask test-corpus` runner and malformed fuzzing suite (11 fuzzed probe streams: truncated headers, bad magic, negative bias, corrupt gzip, invalid json). 100% of malformed inputs safely return typed `Err(IoError)` with 0 panics. |

---

## 2. Automated Corpus Execution Results (`cargo xtask test-corpus`)

The automated corpus runner executed against `fixtures/corpus/` and `test_data/`:

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

## 3. Registered Bugs & Defect Corrections
- **BUG-003:** In compressed SPSS `.sav` files, opcode 254 was decoded as `None` (Null) instead of `Some(0.0)`. Corrected in `crates/statcraft_io/src/spss_loader/decompress.rs`.
- **BUG-004:** In Record Type 7, Variable Display Parameters were erroneously matched on subtype 4 instead of canonical subtype 11. Corrected in `crates/statcraft_io/src/spss_loader/records.rs` with full support for Subtype 11, Subtype 14 (Very Long Strings), Subtype 3 (codepage), and Subtype 20 (encoding).

---

## 4. Definition of Done (DoD) Verification
1. `cargo test --workspace -j 1` — 100% tests passed, 0 failed.
2. `cargo fmt --all -- --check` — Clean formatting across all crates.
3. `cargo clippy --workspace --all-targets -j 1 -- -D warnings` — Clean, zero warnings.
4. `cargo run -p xtask -j 1 -- layers` — Strictly adheres to L0-L6 layering.
5. `cargo run -p xtask -j 1 -- file-guard` — All 269 files within size limits.
6. `cargo run -p xtask -j 1 -- design-guard` — 100% DialogKit compliance.
7. `cargo run -p xtask -j 1 -- test-corpus` — 100% passed, 0 panics.
8. `cargo run -p xtask -j 1 -- ci` — Full CI quality gate passed.

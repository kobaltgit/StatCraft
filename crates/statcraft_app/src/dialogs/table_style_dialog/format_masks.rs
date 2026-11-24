use statcraft_i18n::t;

pub const FORMAT_MASK_KEYS: [&str; 34] = [
    "dialog-format-mask-as-is",
    "dialog-format-mask-num-1",
    "dialog-format-mask-num-sci-1",
    "dialog-format-mask-num-comma-dot",
    "dialog-format-mask-num-dot-comma",
    "dialog-format-mask-num-sci-2",
    "dialog-format-mask-num-pct",
    "dialog-format-mask-date-d-mmm-yy",
    "dialog-format-mask-date-d-mmm-yyyy",
    "dialog-format-mask-date-dd-mm-yy",
    "dialog-format-mask-date-mm-dd-yyyy",
    "dialog-format-mask-date-dd-dot-mm-dot-yy",
    "dialog-format-mask-date-dd-dot-mm-dot-yyyy",
    "dialog-format-mask-date-yy-mm-dd",
    "dialog-format-mask-date-yyyy-mm-dd",
    "dialog-format-mask-date-yyddd",
    "dialog-format-mask-date-yyyyddd",
    "dialog-format-mask-date-q-q-yy",
    "dialog-format-mask-date-q-q-yyyy",
    "dialog-format-mask-date-mmm-yy",
    "dialog-format-mask-date-mmm-yyyy",
    "dialog-format-mask-date-w-wk-yy",
    "dialog-format-mask-date-w-wk-yyyy",
    "dialog-format-mask-datetime-d-m-y-hm",
    "dialog-format-mask-datetime-d-mmm-y-hms",
    "dialog-format-mask-datetime-y-m-d-hm",
    "dialog-format-mask-datetime-y-m-d-hms",
    "dialog-format-mask-duration-d-h-m",
    "dialog-format-mask-duration-d-h-m-s",
    "dialog-format-mask-named-days",
    "dialog-format-mask-named-months",
    "dialog-format-mask-time-m-s",
    "dialog-format-mask-time-hm",
    "dialog-format-mask-time-hms",
];

pub fn get_format_mask_label(idx: usize) -> String {
    let key = FORMAT_MASK_KEYS
        .get(idx)
        .copied()
        .unwrap_or(FORMAT_MASK_KEYS[0]);
    t!(key)
}

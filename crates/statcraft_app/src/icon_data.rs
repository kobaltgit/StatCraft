//! Compile-time embedded SVG icons for StatCraft.
//! All icons are monochromatic SVGs from Lucide (ISC/MIT) or custom vector craft assets.

pub static ICONS: &[(&str, &[u8])] = &[
    (
        "arrow-down-to-line",
        include_bytes!("../assets/icons/arrow-down-to-line.svg"),
    ),
    (
        "arrow-right-to-line",
        include_bytes!("../assets/icons/arrow-right-to-line.svg"),
    ),
    (
        "calculator",
        include_bytes!("../assets/icons/calculator.svg"),
    ),
    ("chart-bar", include_bytes!("../assets/icons/chart-bar.svg")),
    ("clock", include_bytes!("../assets/icons/clock.svg")),
    (
        "columns-plus",
        include_bytes!("../assets/icons/columns-plus.svg"),
    ),
    ("filter", include_bytes!("../assets/icons/filter.svg")),
    (
        "folder-open",
        include_bytes!("../assets/icons/folder-open.svg"),
    ),
    ("info", include_bytes!("../assets/icons/info.svg")),
    ("printer", include_bytes!("../assets/icons/printer.svg")),
    ("redo-2", include_bytes!("../assets/icons/redo-2.svg")),
    ("rows-plus", include_bytes!("../assets/icons/rows-plus.svg")),
    ("save", include_bytes!("../assets/icons/save.svg")),
    ("scale", include_bytes!("../assets/icons/scale.svg")),
    ("search", include_bytes!("../assets/icons/search.svg")),
    ("split", include_bytes!("../assets/icons/split.svg")),
    ("undo-2", include_bytes!("../assets/icons/undo-2.svg")),
    (
        "value-labels",
        include_bytes!("../assets/icons/value-labels.svg"),
    ),
];

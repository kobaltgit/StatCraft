//! String encoding transcoder for SPSS files.
//!
//! Handles UTF-8, Windows-1251 (Russian), Windows-1252 (Western European),
//! and KOI8-R without panics or external dependencies.

/// Maps a Windows-1251 byte (0x80..=0xFF) to a Unicode char.
pub fn cp1251_byte_to_char(b: u8) -> char {
    match b {
        0x80 => '\u{0402}', // Ђ
        0x81 => '\u{0403}', // Ѓ
        0x82 => '\u{201A}', // ‚
        0x83 => '\u{0453}', // ѓ
        0x84 => '\u{201E}', // „
        0x85 => '\u{2026}', // …
        0x86 => '\u{2020}', // †
        0x87 => '\u{2021}', // ‡
        0x88 => '\u{20AC}', // €
        0x89 => '\u{2030}', // ‰
        0x8A => '\u{0409}', // Љ
        0x8B => '\u{2039}', // ‹
        0x8C => '\u{040A}', // Њ
        0x8D => '\u{040C}', // Ќ
        0x8E => '\u{040B}', // Ћ
        0x8F => '\u{040F}', // Џ
        0x90 => '\u{0452}', // ђ
        0x91 => '\u{2018}', // ‘
        0x92 => '\u{2019}', // ’
        0x93 => '\u{201C}', // “
        0x94 => '\u{201D}', // ”
        0x95 => '\u{2022}', // •
        0x96 => '\u{2013}', // –
        0x97 => '\u{2014}', // —
        0x98 => '\u{0098}',
        0x99 => '\u{2122}', // ™
        0x9A => '\u{0459}', // љ
        0x9B => '\u{203A}', // ›
        0x9C => '\u{045A}', // њ
        0x9D => '\u{045C}', // ќ
        0x9E => '\u{045B}', // ћ
        0x9F => '\u{045F}', // џ
        0xA0 => '\u{00A0}', // NBSP
        0xA1 => '\u{040E}', // Ў
        0xA2 => '\u{045E}', // ў
        0xA3 => '\u{0408}', // Ј
        0xA4 => '\u{00A4}', // ¤
        0xA5 => '\u{0490}', // Ґ
        0xA6 => '\u{00A6}', // ¦
        0xA7 => '\u{00A7}', // §
        0xA8 => '\u{0401}', // Ё
        0xA9 => '\u{00A9}', // ©
        0xAA => '\u{0404}', // Є
        0xAB => '\u{00AB}', // «
        0xAC => '\u{00AC}', // ¬
        0xAD => '\u{00AD}', // SHY
        0xAE => '\u{00AE}', // ®
        0xAF => '\u{0407}', // Ї
        0xB0 => '\u{00B0}', // °
        0xB1 => '\u{00B1}', // ±
        0xB2 => '\u{0406}', // І
        0xB3 => '\u{0456}', // і
        0xB4 => '\u{0491}', // ґ
        0xB5 => '\u{00B5}', // µ
        0xB6 => '\u{00B6}', // ¶
        0xB7 => '\u{00B7}', // ·
        0xB8 => '\u{0451}', // ё
        0xB9 => '\u{2116}', // №
        0xBA => '\u{0454}', // є
        0xBB => '\u{00BB}', // »
        0xBC => '\u{0458}', // ј
        0xBD => '\u{0405}', // Ѕ
        0xBE => '\u{0455}', // ѕ
        0xBF => '\u{0457}', // ї
        0xC0..=0xDF => {
            // Russian Capital Letters 'А'..='Я'
            let offset = (b - 0xC0) as u32;
            char::from_u32(0x0410 + offset).unwrap_or('?')
        }
        0xE0..=0xFF => {
            // Russian Small Letters 'а'..='я'
            let offset = (b - 0xE0) as u32;
            char::from_u32(0x0430 + offset).unwrap_or('?')
        }
        _ => b as char,
    }
}

/// Decodes bytes into a String according to Windows-1251.
pub fn decode_cp1251(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if b < 0x80 {
            out.push(b as char);
        } else {
            out.push(cp1251_byte_to_char(b));
        }
    }
    out
}

/// Decodes bytes into a String according to Windows-1252.
pub fn decode_cp1252(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        match b {
            0x80 => out.push('\u{20AC}'),
            0x82 => out.push('\u{201A}'),
            0x83 => out.push('\u{0192}'),
            0x84 => out.push('\u{201E}'),
            0x85 => out.push('\u{2026}'),
            0x86 => out.push('\u{2020}'),
            0x87 => out.push('\u{2021}'),
            0x88 => out.push('\u{02C6}'),
            0x89 => out.push('\u{2030}'),
            0x8A => out.push('\u{0160}'),
            0x8B => out.push('\u{2039}'),
            0x8C => out.push('\u{0152}'),
            0x8E => out.push('\u{017D}'),
            0x91 => out.push('\u{2018}'),
            0x92 => out.push('\u{2019}'),
            0x93 => out.push('\u{201C}'),
            0x94 => out.push('\u{201D}'),
            0x95 => out.push('\u{2022}'),
            0x96 => out.push('\u{2013}'),
            0x97 => out.push('\u{2014}'),
            0x98 => out.push('\u{02DC}'),
            0x99 => out.push('\u{2122}'),
            0x9A => out.push('\u{0161}'),
            0x9B => out.push('\u{203A}'),
            0x9C => out.push('\u{0153}'),
            0x9E => out.push('\u{017E}'),
            0x9F => out.push('\u{0178}'),
            _ => out.push(b as char),
        }
    }
    out
}

/// Decodes bytes into a String according to KOI8-R.
pub fn decode_koi8r(bytes: &[u8]) -> String {
    const KOI8R_TABLE: [char; 128] = [
        '─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '▀', '▄', '█', '▌', '▐', '░', '▒',
        '▓', '⌠', '■', '∙', '√', '≈', '≤', '≥', ' ', '⌡', '°', '²', '·', '÷', '═', '║', '╒', 'ё',
        '╓', '╔', '╕', '╖', '╗', '╘', '╙', '╚', '╛', '╜', '╝', '╞', '╟', '╠', '╡', 'Ё', '╢', '╣',
        '╤', '╥', '╦', '╧', '╨', '╩', '╪', '╫', '╬', '©', 'ю', 'а', 'б', 'ц', 'д', 'е', 'ф', 'г',
        'х', 'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'я', 'р', 'с', 'т', 'у', 'ж', 'в', 'ь', 'ы',
        'з', 'ш', 'э', 'щ', 'ч', 'ъ', 'Ю', 'А', 'Б', 'Ц', 'Д', 'Е', 'Ф', 'Г', 'Х', 'И', 'Й', 'К',
        'Л', 'М', 'Н', 'О', 'П', 'Я', 'Р', 'С', 'Т', 'У', 'Ж', 'В', 'Ь', 'Ы', 'З', 'Ш', 'Э', 'Щ',
        'Ч', 'Ъ',
    ];

    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if b < 0x80 {
            out.push(b as char);
        } else {
            out.push(KOI8R_TABLE[(b - 0x80) as usize]);
        }
    }
    out
}

/// Decodes an SPSS byte buffer using declared encoding and codepage.
pub fn decode_spss_string(bytes: &[u8], encoding: Option<&str>, codepage: Option<u32>) -> String {
    let enc_upper = encoding.map(|s| s.trim().to_uppercase());
    let enc_ref = enc_upper.as_deref().unwrap_or("");

    // Check if explicitly CP1251
    if codepage == Some(1251)
        || enc_ref.contains("1251")
        || enc_ref.contains("CP1251")
        || enc_ref.contains("WINDOWS-1251")
    {
        return decode_cp1251(bytes);
    }

    // Check if explicitly KOI8
    if enc_ref.contains("KOI8") {
        return decode_koi8r(bytes);
    }

    // Check if explicitly CP1252 / Latin-1
    if codepage == Some(1252) || enc_ref.contains("1252") || enc_ref.contains("ISO-8859-1") {
        return decode_cp1252(bytes);
    }

    // Check if valid UTF-8
    if let Ok(valid_str) = std::str::from_utf8(bytes) {
        return valid_str.to_string();
    }

    // Fallback detection: if non-ASCII bytes and codepage is likely 1251
    if bytes.iter().any(|&b| b >= 0xC0) {
        return decode_cp1251(bytes);
    }

    // Default safe fallback
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp1251_decoding() {
        // "Возраст" in CP1251: C2 EE E7 F0 E0 F1 F2
        let raw = [0xC2, 0xEE, 0xE7, 0xF0, 0xE0, 0xF1, 0xF2];
        let decoded = decode_spss_string(&raw, Some("WINDOWS-1251"), None);
        assert_eq!(decoded, "Возраст");

        let decoded_cp = decode_spss_string(&raw, None, Some(1251));
        assert_eq!(decoded_cp, "Возраст");
    }

    #[test]
    fn test_utf8_decoding() {
        let text = "Hello World / Привет";
        let bytes = text.as_bytes();
        let decoded = decode_spss_string(bytes, Some("UTF-8"), Some(65001));
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_cp1252_decoding() {
        // "Café" in CP1252: C a f 0xE9
        let raw = [b'C', b'a', b'f', 0xE9];
        let decoded = decode_spss_string(&raw, Some("WINDOWS-1252"), None);
        assert_eq!(decoded, "Café");
    }
}

//! Normalization to clean UTF-8 (doc 2 §2.3) — strict decode, no guessing:
//! a faulty normalization is surfaced, never buried (Law II.2 at the floor).

/// The initial supported set (doc 2 §2.4, finalized in SLICE_02 §4):
/// everything that degrades to UTF-8 text deterministically. Text-layer
/// PDFs and RTF are deferred — both need parsing beyond a decode.
pub const SUPPORTED_TYPES: &[&str] = &[
    "txt", "md", "json", "py", "html", "htm", "css", "js", "mjs", "ts", "rs", "c", "h", "cpp",
    "hpp", "java", "sh", "ps1", "toml", "yaml", "yml", "xml", "csv", "tsv", "log", "ini", "cfg",
    "sql",
];

/// The outcome of the decode floor. `Failed` and `Unsupported` are data,
/// not errors — the pipe stores and surfaces them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeResult {
    Text(String),
    Failed(String),
    Unsupported,
}

pub fn is_supported(filetype: &str) -> bool {
    SUPPORTED_TYPES.contains(&filetype)
}

/// Strict decode: UTF-8, or UTF-16 LE/BE by BOM. Whitespace
/// standardization v1 = BOM strip + line endings to LF (SLICE_02 §4);
/// structure in the text is retained, not stripped — it is signal.
pub fn normalize(filetype: &str, bytes: &[u8]) -> DecodeResult {
    if !is_supported(filetype) {
        return DecodeResult::Unsupported;
    }
    let decoded = if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        std::str::from_utf8(rest)
            .map(str::to_owned)
            .map_err(|e| format!("invalid UTF-8 after BOM: {e}"))
    } else if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        decode_utf16(rest, false).ok_or_else(|| "invalid UTF-16 LE".to_string())
    } else if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        decode_utf16(rest, true).ok_or_else(|| "invalid UTF-16 BE".to_string())
    } else {
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|e| format!("invalid UTF-8: {e}"))
    };
    match decoded {
        Ok(text) => DecodeResult::Text(text.replace("\r\n", "\n").replace('\r', "\n")),
        Err(reason) => DecodeResult::Failed(reason),
    }
}

fn decode_utf16(bytes: &[u8], big_endian: bool) -> Option<String> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| {
            if big_endian {
                u16::from_be_bytes([pair[0], pair[1]])
            } else {
                u16::from_le_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    String::from_utf16(&units).ok()
}

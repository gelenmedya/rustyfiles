//! Legacy text encodings: text that is not UTF-8 is read in the Windows code page of the
//! user's language (old Turkish files are Windows-1254), guessed from `LANG`.

use std::sync::OnceLock;

fn lang() -> &'static str {
    static L: OnceLock<String> = OnceLock::new();
    L.get_or_init(|| {
        ["LC_ALL", "LC_CTYPE", "LANG"].iter().find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty())).unwrap_or_default().to_ascii_lowercase()
    })
}

pub fn legacy_ansi_codepage() -> u32 {
    let l = lang();
    match () {
        _ if l.starts_with("tr") => 1254,
        _ if l.starts_with("ru") || l.starts_with("uk") || l.starts_with("bg") || l.starts_with("be") || l.starts_with("sr") => 1251,
        _ if l.starts_with("pl") || l.starts_with("cs") || l.starts_with("sk") || l.starts_with("hu") || l.starts_with("hr") || l.starts_with("ro") => 1250,
        _ if l.starts_with("el") => 1253,
        _ if l.starts_with("he") => 1255,
        _ if l.starts_with("ar") || l.starts_with("fa") => 1256,
        _ if l.starts_with("lt") || l.starts_with("lv") || l.starts_with("et") => 1257,
        _ if l.starts_with("vi") => 1258,
        _ => 1252,
    }
}

fn decode(raw: &[u8], cp: u32) -> Option<String> {
    let enc = match cp {
        1250 => encoding_rs::WINDOWS_1250,
        1251 => encoding_rs::WINDOWS_1251,
        1253 => encoding_rs::WINDOWS_1253,
        1254 => encoding_rs::WINDOWS_1254,
        1255 => encoding_rs::WINDOWS_1255,
        1256 => encoding_rs::WINDOWS_1256,
        1257 => encoding_rs::WINDOWS_1257,
        1258 => encoding_rs::WINDOWS_1258,
        _ => encoding_rs::WINDOWS_1252,
    };
    let (s, _, _) = enc.decode(raw);
    Some(s.into_owned())
}

/// Text from bytes of unknown encoding: UTF-8 if valid (a truncated trailing sequence is
/// tolerated, since previews read only the head of a file), otherwise the legacy page.
pub fn decode_text(raw: &[u8]) -> String {
    match std::str::from_utf8(raw) {
        Ok(s) => s.to_string(),
        Err(e) if e.error_len().is_none() => String::from_utf8_lossy(&raw[..e.valid_up_to()]).into_owned(),
        Err(_) => decode(raw, legacy_ansi_codepage()).unwrap_or_else(|| String::from_utf8_lossy(raw).into_owned()),
    }
}

/// True when the user's locale is Turkish (sorting then treats `I` as capital `ı`).
pub fn user_locale_is_turkish() -> bool {
    lang().starts_with("tr")
}

#[cfg(test)]
mod tests {
    #[test]
    fn windows_1254() {
        let raw = [0xDE, b'i', b'r', b'k', b'e', b't', b' ', 0xFD, 0xF0, 0xFC, 0xFE, 0xF6, 0xE7];
        assert_eq!(super::decode(&raw, 1254).as_deref(), Some("Şirket ığüşöç"));
    }
}

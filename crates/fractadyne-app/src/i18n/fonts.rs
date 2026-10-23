//! Fonts for the writing systems the interface fonts lack (design/multilingual.md §9).
//!
//! Latin and Cyrillic draw with the bundled fonts. Japanese, Chinese and Korean need a font of
//! their own, and a whole one is 5–30 MB, so none is bundled: the system's is found and read when
//! the language is chosen, and kept for the session. They are never one "CJK" font: the same Han
//! character is drawn differently in Japanese, Simplified and Traditional Chinese typography (Han
//! unification), so each language gets a font of its own. (The language menu's own names, 日本語,
//! 简体中文, 繁體中文 and 한국어, come from tiny bundled subsets instead: see `theme::install_fonts`.)

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

/// A system font a writing system needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Need {
    Japanese,
    ChineseSimplified,
    ChineseTraditional,
    Korean,
}

impl Need {
    /// The font a language written in `script` (ISO 15924) needs beyond the bundled ones.
    pub(crate) fn for_script(script: &str) -> Option<Need> {
        match script {
            "Jpan" | "Hira" | "Kana" => Some(Need::Japanese),
            "Hans" | "Hani" => Some(Need::ChineseSimplified),
            "Hant" => Some(Need::ChineseTraditional),
            "Kore" | "Hang" => Some(Need::Korean),
            _ => None,
        }
    }

    /// The name egui knows the font by.
    pub(crate) fn key(self) -> &'static str {
        match self {
            Need::Japanese => "system-ja",
            Need::ChineseSimplified => "system-zh-Hans",
            Need::ChineseTraditional => "system-zh-Hant",
            Need::Korean => "system-ko",
        }
    }

    /// A character the font must hold to draw the language.
    fn probe(self) -> char {
        match self {
            Need::Japanese => 'あ',
            Need::ChineseSimplified => '简',
            Need::ChineseTraditional => '體',
            Need::Korean => '한',
        }
    }

    /// Where to look, best first: a file and the faces in it to prefer (a collection holds several).
    fn candidates(self) -> Vec<(PathBuf, &'static [&'static str])> {
        let mut out = Vec::new();
        if cfg!(windows) {
            let fonts = std::env::var_os("WINDIR").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from).join("Fonts");
            let list: &[(&str, &[&str])] = match self {
                // Regular weights only: Yu Gothic's UI face at 400 is in the MEDIUM file (YuGothR.ttc
                // holds the UI face at Semilight, 350, too thin beside the interface's own font).
                Need::Japanese => &[
                    ("YuGothM.ttc", &["Yu Gothic UI Regular"]),
                    ("YuGothR.ttc", &["Yu Gothic Regular"]),
                    ("meiryo.ttc", &["Meiryo UI", "Meiryo"]),
                    ("msgothic.ttc", &["MS UI Gothic", "MS Gothic"]),
                ],
                Need::ChineseSimplified => &[
                    ("msyh.ttc", &["Microsoft YaHei UI", "Microsoft YaHei"]),
                    ("Deng.ttf", &["DengXian"]),
                    ("simsun.ttc", &["SimSun"]),
                ],
                Need::ChineseTraditional => &[
                    ("msjh.ttc", &["Microsoft JhengHei UI", "Microsoft JhengHei"]),
                    ("mingliu.ttc", &["PMingLiU", "MingLiU"]),
                ],
                Need::Korean => &[
                    ("malgun.ttf", &["Malgun Gothic"]),
                    ("gulim.ttc", &["Gulim", "Dotum"]),
                ],
            };
            out.extend(list.iter().map(|(f, faces)| (fonts.join(f), *faces)));
        } else {
            let faces: &'static [&'static str] = match self {
                Need::Japanese => &["Noto Sans CJK JP", "Noto Sans JP", "Source Han Sans JP", "IPAexGothic", "IPAGothic"],
                Need::ChineseSimplified => &["Noto Sans CJK SC", "Noto Sans SC", "Source Han Sans SC", "WenQuanYi Micro Hei"],
                Need::ChineseTraditional => &["Noto Sans CJK TC", "Noto Sans TC", "Source Han Sans TC", "AR PL UMing TW"],
                Need::Korean => &["Noto Sans CJK KR", "Noto Sans KR", "Source Han Sans K", "NanumGothic", "UnDotum"],
            };
            for dir in ["/usr/share/fonts", "/usr/local/share/fonts"] {
                for f in [
                    "opentype/noto/NotoSansCJK-Regular.ttc",
                    "noto-cjk/NotoSansCJK-Regular.ttc",
                    "google-noto-cjk/NotoSansCJK-Regular.ttc",
                    "opentype/noto-cjk/NotoSansCJK-Regular.ttc",
                    "truetype/noto/NotoSansCJK-Regular.ttc",
                    "OTF/NotoSansCJK-Regular.ttc",
                ] {
                    out.push((PathBuf::from(dir).join(f), faces));
                }
            }
            // Anything else fontconfig knows for the language.
            let lang = match self {
                Need::Japanese => "ja",
                Need::ChineseSimplified => "zh-cn",
                Need::ChineseTraditional => "zh-tw",
                Need::Korean => "ko",
            };
            // (Any face of it that draws the language: fontconfig chose the file for the language.)
            if let Ok(o) = std::process::Command::new("fc-match").args(["-f", "%{file}", &format!("sans:lang={lang}")]).output() {
                let file = String::from_utf8_lossy(&o.stdout).trim().to_owned();
                if !file.is_empty() {
                    out.push((PathBuf::from(file), &[]));
                }
            }
        }
        out
    }
}

/// A system font, read.
pub(crate) struct SystemFont {
    /// Its face's name, for the log.
    pub(crate) name: String,
    pub(crate) data: Arc<egui::FontData>,
}

/// The system font `need` asks for, found and read once (`None` when the system has none that can
/// draw it). Reading one costs its size in memory (Microsoft YaHei: 19 MB) for the session.
pub(crate) fn system_font(need: Need) -> Option<&'static SystemFont> {
    static FOUND: [OnceLock<Option<SystemFont>>; 4] = [OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new()];
    let cell = &FOUND[need as usize];
    cell.get_or_init(|| {
        let found = need.candidates().into_iter().find_map(|(path, faces)| read_face(&path, faces, need));
        match &found {
            Some(f) => crate::diag::log_line("language", &format!("font for {need:?}: {}", f.name)),
            None => crate::diag::log_line("language", &format!("no system font for {need:?}")),
        }
        found
    })
    .as_ref()
}

/// The face of `path` named first in `faces`, if it can draw the language (a probe character must
/// be in it). A file with none of those faces is passed over, so a light or a bold face is never
/// taken for want of the regular one; an empty `faces` takes any face that draws the language.
fn read_face(path: &std::path::Path, faces: &[&str], need: Need) -> Option<SystemFont> {
    let bytes = std::fs::read(path).ok()?;
    let count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1);
    let probe = need.probe();
    let mut named: Vec<(usize, u32, String)> = Vec::new();
    for i in 0..count {
        let Ok(face) = ttf_parser::Face::parse(&bytes, i) else { continue };
        if face.glyph_index(probe).is_none() {
            continue;
        }
        // Every name it gives, in every language: a Japanese font names itself in Japanese first
        // (游ゴシック), and in English in another record.
        let names: Vec<String> = face
            .names()
            .into_iter()
            .filter(|n| n.name_id == ttf_parser::name_id::FULL_NAME || n.name_id == ttf_parser::name_id::FAMILY)
            .filter_map(|n| n.to_string())
            .collect();
        let rank = faces.iter().position(|f| names.iter().any(|n| n.starts_with(f)));
        if rank.is_none() && !faces.is_empty() {
            continue;
        }
        let name = rank
            .and_then(|r| names.iter().find(|n| n.starts_with(faces[r])))
            .or_else(|| names.first())
            .cloned()
            .unwrap_or_default();
        named.push((rank.unwrap_or(0), i, name));
    }
    named.sort();
    let (_, index, name) = named.into_iter().next()?;
    let mut data = egui::FontData::from_owned(bytes);
    data.index = index;
    Some(SystemFont { name, data: Arc::new(data) })
}

#[cfg(test)]
#[path = "../../tests/unit/i18n/fonts/tests.rs"]
mod tests;

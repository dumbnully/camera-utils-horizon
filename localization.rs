use std::{collections::BTreeMap, sync::OnceLock};

type Catalogs = BTreeMap<String, BTreeMap<String, String>>;

fn catalogs() -> &'static Catalogs {
    static CATALOGS: OnceLock<Catalogs> = OnceLock::new();
    CATALOGS.get_or_init(|| {
        serde_json::from_slice(crate::native_data!("camerautils", "locales.json"))
            .expect("invalid bundled localization data")
    })
}

fn catalog(code: &str) -> &'static BTreeMap<String, String> {
    let normalized = code.to_ascii_lowercase().replace('-', "_");
    let alias = match normalized.as_str() {
        "zh_hk" | "zh_hant" | "zh_hant_tw" | "zh_hant_hk" => "zh_tw",
        "zh" | "zh_hans" | "zh_hans_cn" | "zh_sg" => "zh_cn",
        "pt_br" => "pt",
        "sr_cs" | "sr_latn_rs" | "sr_latn" => "sr_latn",
        "no" | "no_no" => "nb",
        "iw_il" | "iw" => "he",
        "tl_ph" | "tl" => "fil",
        "in_id" | "in" => "id",
        other => other,
    };
    let all = catalogs();
    all.get(alias)
        .or_else(|| all.get(alias.split('_').next().unwrap_or("en")))
        .unwrap_or_else(|| &all["en"])
}

pub(super) fn tr<'a>(code: &str, source: &'a str) -> &'a str {
    catalog(code)
        .get(source)
        .map(String::as_str)
        .unwrap_or(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_catalogs() {
        let all = catalogs();
        assert_eq!(all.len(), 57);
        let expected: Vec<_> = all["en"].keys().collect();
        assert_eq!(expected.len(), 47);
        for values in all.values() {
            assert_eq!(values.keys().collect::<Vec<_>>(), expected);
            assert!(values.values().all(|value| !value.trim().is_empty()));
        }
    }

    #[test]
    fn game_regions_and_scripts() {
        for (code, expected) in [
            ("PT-BR", "pt"),
            ("pt_PT", "pt_pt"),
            ("fr_CA", "fr"),
            ("es_MX", "es"),
            ("de_DE", "de"),
            ("zh_CN", "zh_cn"),
            ("zh_HK", "zh_tw"),
            ("sr_SP", "sr"),
            ("sr_CS", "sr_latn"),
            ("ja_JP", "ja"),
            ("xx_XX", "en"),
        ] {
            assert_eq!(tr(code, "Back"), catalogs()[expected]["Back"]);
        }
    }

    #[test]
    fn live_language_changes() {
        for code in ["ja_JP", "en_US", "pt_BR", "ja_JP"] {
            assert_eq!(tr(code, "Back"), catalog(code)["Back"]);
        }
        assert_ne!(tr("pt_BR", "Back"), tr("en_US", "Back"));
        assert_eq!(tr("en_US", "Unknown message"), "Unknown message");
    }
}

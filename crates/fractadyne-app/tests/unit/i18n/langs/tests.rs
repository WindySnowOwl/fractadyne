use super::*;

fn installed(tags: &[&str]) -> Vec<Installed> {
    tags.iter()
        .map(|t| Installed { tag: (*t).into(), name: (*t).into(), english_name: String::new(), revision: 0, draft: false, source: Source::Builtin })
        .collect()
}

#[test]
fn the_system_language_finds_the_nearest_installed_one() {
    let have = installed(&["en", "es", "fr", "de", "ja", "zh-Hans", "km", "uk"]);
    let best = |req: &[&str]| best_match(&req.iter().map(|s| s.to_string()).collect::<Vec<_>>(), &have);
    assert_eq!(best(&["fr-CA"]).as_deref(), Some("fr"));
    assert_eq!(best(&["de-AT", "en-US"]).as_deref(), Some("de"));
    assert_eq!(best(&["ja-JP"]).as_deref(), Some("ja"));
    // A Chinese system says where it is, not which script: mainland and Singapore read Simplified.
    assert_eq!(best(&["zh-CN"]).as_deref(), Some("zh-Hans"));
    assert_eq!(best(&["zh-SG"]).as_deref(), Some("zh-Hans"));
    // Taiwan reads Traditional, which is not installed: the next preference decides.
    assert_eq!(best(&["zh-TW", "en-US"]).as_deref(), Some("en"));
    assert_eq!(best(&["km-KH"]).as_deref(), Some("km"));
    assert_eq!(best(&["uk-UA"]).as_deref(), Some("uk"));
    // A language the app does not have matches nothing (the caller falls back to English).
    assert_eq!(best(&["ko-KR"]), None);
    assert_eq!(best(&[]), None);
}

#[test]
fn the_pseudo_language_is_never_matched_from_the_system() {
    // Even a system that asks for it gets English: the pseudo-language is chosen only by name.
    let have = installed(&["en", pseudo::TAG]);
    assert_eq!(best_match(&["en-XA".to_string()], &have).as_deref(), Some("en"));
}

/// The second batch's languages: the regions a system names find them, Portugal's Portuguese takes
/// Brazil's (the one there is), Taiwan and Hong Kong take Traditional Chinese, mainland Simplified.
#[test]
fn the_second_batch_is_found_from_the_systems_regions() {
    let have = installed(&["en", "es", "pt-BR", "ru", "ko", "id", "tr", "it", "pl", "zh-Hans", "zh-Hant"]);
    let best = |req: &[&str]| best_match(&req.iter().map(|s| s.to_string()).collect::<Vec<_>>(), &have);
    assert_eq!(best(&["pt-BR"]).as_deref(), Some("pt-BR"));
    assert_eq!(best(&["pt-PT"]).as_deref(), Some("pt-BR"));
    assert_eq!(best(&["ru-RU"]).as_deref(), Some("ru"));
    assert_eq!(best(&["ko-KR"]).as_deref(), Some("ko"));
    assert_eq!(best(&["id-ID"]).as_deref(), Some("id"));
    assert_eq!(best(&["tr-TR"]).as_deref(), Some("tr"));
    assert_eq!(best(&["it-IT"]).as_deref(), Some("it"));
    assert_eq!(best(&["pl-PL"]).as_deref(), Some("pl"));
    assert_eq!(best(&["zh-TW"]).as_deref(), Some("zh-Hant"));
    assert_eq!(best(&["zh-HK"]).as_deref(), Some("zh-Hant"));
    assert_eq!(best(&["zh-CN"]).as_deref(), Some("zh-Hans"));
}

#[test]
fn tags_are_canonical() {
    assert_eq!(canonical_tag("zh-hans").as_deref(), Some("zh-Hans"));
    assert_eq!(canonical_tag("pt_br").as_deref(), Some("pt-BR"));
    assert_eq!(canonical_tag("not a tag"), None);
}

fn write(dir: &Path, name: &str, body: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(name), body).unwrap();
}

#[test]
fn the_highest_revision_of_a_language_wins_and_a_working_folder_always() {
    let root = std::env::temp_dir().join(format!("fd-langs-{}-{}", std::process::id(), line!()));
    let (pkg, user, dev) = (root.join("pkg"), root.join("user"), root.join("dev"));
    write(&pkg, "fractadyne-de.ftl", "### revision: 5\nlanguage-name = Deutsch\nlanguage-english-name = German\n");
    write(&user, "fractadyne-de.ftl", "### revision: 9\nlanguage-name = Deutsch\n");
    write(&pkg, "fractadyne-fr.ftl", "### revision: 3\nlanguage-name = Français\nlanguage-english-name = French\n");
    write(&user, "fractadyne-fr.ftl", "### revision: 2\nlanguage-name = Français\n");
    // The repository's layout in a translator's folder: <tag>/fractadyne.ftl, unstamped.
    write(&dev.join("es"), "fractadyne.ftl", "language-name = Español\nlanguage-english-name = Spanish\n");
    write(&pkg, "fractadyne-es.ftl", "### revision: 50\nlanguage-name = Español\n");
    write(&pkg, "notes.txt", "not a language");
    let got = discover_in(Some(&pkg), Some(&user), Some(&dev));
    let find = |t: &str| got.iter().find(|i| i.tag == t).unwrap();
    assert_eq!(got[0].tag, "en");
    assert_eq!(got.len(), 4);
    assert_eq!(find("de").revision, 9);
    assert!(matches!(find("de").source, Source::User(_)));
    assert_eq!(find("fr").revision, 3);
    assert!(matches!(find("fr").source, Source::Package(_)));
    assert!(matches!(find("es").source, Source::Dev(_)));
    assert_eq!(find("es").menu_label(), "Español — Spanish");
    assert_eq!(find("de").menu_label(), "Deutsch");
    let _ = std::fs::remove_dir_all(&root);
}

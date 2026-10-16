use super::*;

#[test]
fn the_setting_offers_every_card_but_the_window_s_own() {
    let cards = vec![(1, "NVIDIA GeForce RTX 3080".to_string()), (2, "AMD Radeon RX 6800 XT".to_string())];
    assert_eq!(second_card_choices(&cards, "NVIDIA GeForce RTX 3080"), vec![(2, "AMD Radeon RX 6800 XT".to_string())]);
    // One card: nothing to offer.
    assert!(second_card_choices(&cards[..1], "NVIDIA GeForce RTX 3080").is_empty());
    // Two identical cards: neither can be told apart from the window's, so both are offered.
    let twins = vec![(1, "NVIDIA GeForce RTX 3080".to_string()), (2, "NVIDIA GeForce RTX 3080".to_string())];
    assert_eq!(second_card_choices(&twins, "NVIDIA GeForce RTX 3080"), twins);
    // A window on a card the listing does not name (another API): every card is offered.
    assert_eq!(second_card_choices(&cards, "llvmpipe"), cards);
}

#[test]
fn an_export_renders_on_the_window_s_card_and_the_ones_named_beside_it() {
    use crate::farm::local::Gpus;
    let cards = vec![(1, "AMD Radeon RX 6800 XT".to_string()), (2, "NVIDIA GeForce RTX 3070".to_string())];
    // The window on the 6800 XT: `all` adds the 3070; naming both adds the 3070 alone.
    assert_eq!(export_extra_cards(&Gpus::All, &cards, "AMD Radeon RX 6800 XT"), vec![2]);
    assert_eq!(export_extra_cards(&Gpus::List(vec![1, 2]), &cards, "AMD Radeon RX 6800 XT"), vec![2]);
    assert_eq!(export_extra_cards(&Gpus::List(vec![2]), &cards, "AMD Radeon RX 6800 XT"), vec![2]);
    // One card named twice: the window plus a second device on the same card.
    let one = vec![(1, "NVIDIA GeForce RTX 3080".to_string())];
    assert_eq!(export_extra_cards(&Gpus::List(vec![1, 1]), &one, "NVIDIA GeForce RTX 3080"), vec![1]);
    assert!(export_extra_cards(&Gpus::All, &one, "NVIDIA GeForce RTX 3080").is_empty());
}

#[test]
fn a_second_device_copies_a_reference_only_within_its_card_s_budget() {
    const GIB: u64 = 1 << 30;
    // Sharing the window's card: both copies live there, so a quarter each at most.
    assert_eq!(reference_budget(8 * GIB, true), 2 * GIB);
    assert_eq!(reference_budget(2 * GIB, true), GIB / 2);
    // A card of its own holds only its copy.
    assert_eq!(reference_budget(16 * GIB, false), 8 * GIB);
    // Unknown memory: a fixed budget, above any moving reference (~37 MB).
    assert_eq!(reference_budget(0, true), UNKNOWN_VRAM_BUDGET);
    assert!(UNKNOWN_VRAM_BUDGET > 37 << 20);
    let p = fractadyne_gpu::MandelbrotParams {
        orbit: Arc::new(vec![[0.0; 4]; 1000]),
        bla: Arc::new(vec![[0.0; 4]; 500]),
        ..Default::default()
    };
    assert_eq!(reference_bytes(&p), 16 * 1500);
}

#[test]
fn the_setting_is_same_card_unless_chosen_otherwise() {
    // Never chosen: Same card. Chosen: what was chosen.
    assert_eq!(setting_spec(""), SAME_CARD);
    assert_eq!(setting_spec(OFF), "");
    assert_eq!(setting_spec(" 2 "), "2");
    assert_eq!(setting_spec(" same "), SAME_CARD);
    // An edited session file reads as off, never as a name to search for.
    assert_eq!(setting_spec("0"), "");
    assert_eq!(setting_spec("RTX"), "");
    assert_eq!(setting_spec("6800"), ""); // a model number is a name, not a position
    assert_eq!(clean_setting("RTX"), OFF);
    // The stored value keeps "never chosen" apart from a choice...
    assert_eq!(clean_setting(" "), "");
    assert_eq!(clean_setting(" same "), SAME_CARD);
    // ...so a harness takes only a choice.
    assert_eq!(explicit_spec(""), "");
    assert_eq!(explicit_spec(SAME_CARD), SAME_CARD);
    assert_eq!(explicit_spec("2"), "2");
    assert_eq!(explicit_spec(OFF), "");
}

#[test]
fn the_status_line_says_where_the_card_stands() {
    assert!(WorkerState::Off.text(false).starts_with("Off"));
    assert_eq!(WorkerState::Running("X · Vulkan".into()).text(false), "In use: X · Vulkan");
    assert!(WorkerState::Lost("X".into()).text(false).contains("choose it again"));
    assert!(WorkerState::Running("X".into()).text(true).ends_with("(set by --worker-gpu)"));
}

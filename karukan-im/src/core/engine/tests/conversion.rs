use super::*;

#[test]
fn test_conversion_char_commits_and_continues() {
    let mut engine = InputMethodEngine::new();

    // Type "あい" and enter conversion
    engine.process_key(&press('a'));
    engine.process_key(&press('i'));
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Type 'k' during conversion → should commit candidate and start new input
    let result = engine.process_key(&press('k'));
    assert!(result.consumed);

    // Should have committed the conversion
    let has_commit = result
        .actions
        .iter()
        .any(|a| matches!(a, EngineAction::Commit(_)));
    assert!(has_commit, "Should have a commit action");

    // Should now be in Composing with 'k' in preedit
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.preedit().unwrap().text(), "k");
}

#[test]
fn test_conversion_char_commits_and_continues_romaji() {
    let mut engine = InputMethodEngine::new();

    // Type "あ" and enter conversion
    engine.process_key(&press('a'));
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Type 'k', 'a' → commits conversion, then starts "か"
    engine.process_key(&press('k'));
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.preedit().unwrap().text(), "k");

    engine.process_key(&press('a'));
    assert_eq!(engine.preedit().unwrap().text(), "か");
}

#[test]
fn test_alphabet_mode_space_inserts_literal_space() {
    let mut engine = InputMethodEngine::new();

    // Enter alphabet mode via Shift+N
    engine.process_key(&press_shift('N'));
    assert!(engine.input_mode == InputMode::Alphabet);

    // Type "ew" (no shift needed, alphabet mode persists)
    engine.process_key(&press('e'));
    engine.process_key(&press('w'));
    assert_eq!(engine.preedit().unwrap().text(), "New");

    // Space → should insert literal space, NOT start conversion
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.preedit().unwrap().text(), "New ");

    // Type "york" (alphabet mode persists)
    engine.process_key(&press('y'));
    engine.process_key(&press('o'));
    engine.process_key(&press('r'));
    engine.process_key(&press('k'));
    assert_eq!(engine.preedit().unwrap().text(), "New york");
}

/// Helper: press a key with Shift modifier
fn press_shift_key(keysym: Keysym) -> KeyEvent {
    KeyEvent::new(keysym, KeyModifiers::new().with_shift(true), true)
}

#[test]
fn test_partial_conversion_cancel_restores_full_text() {
    let mut engine = InputMethodEngine::new();
    // Disable auto_suggest to avoid model inference
    engine.config.auto_suggest = false;

    // Type "あいうえお" (a i u e o)
    for ch in "aiueo".chars() {
        engine.process_key(&press(ch));
    }
    assert_eq!(engine.preedit().unwrap().text(), "あいうえお");
    assert_eq!(engine.input_buf.text, "あいうえお");

    // Select first 3 chars with Shift+Right×3 from beginning
    // First, move cursor to beginning
    engine.process_key(&press_key(Keysym::HOME));
    assert_eq!(engine.input_buf.cursor_pos, 0);

    // Shift+Right ×3 → select "あいう"
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    assert_eq!(engine.input_buf.selection_range(), Some((0, 3)));

    // Space → start partial conversion of "あいう"
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Verify input_buf.text still has full text
    assert_eq!(engine.input_buf.text, "あいうえお");
    // Verify remaining_after_conversion is set
    assert!(engine.remaining_after_conversion.is_some());
    let (before, after) = engine.remaining_after_conversion.as_ref().unwrap();
    assert_eq!(before, "");
    assert_eq!(after, "えお");

    // Navigate candidates (Down)
    engine.process_key(&press_key(Keysym::DOWN));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // input_buf.text should still be intact
    assert_eq!(engine.input_buf.text, "あいうえお");

    // Cancel with Escape
    let result = engine.process_key(&press_key(Keysym::ESCAPE));
    assert!(result.consumed);

    // Should be back in Composing with full text
    assert!(
        matches!(engine.state(), InputState::Composing { .. }),
        "Expected Composing, got {:?}",
        std::mem::discriminant(engine.state())
    );
    assert_eq!(engine.preedit().unwrap().text(), "あいうえお");
    assert_eq!(engine.input_buf.text, "あいうえお");

    // No commit action should have been produced
    let has_commit = result
        .actions
        .iter()
        .any(|a| matches!(a, EngineAction::Commit(_)));
    assert!(!has_commit, "Cancel should not produce a Commit action");
}

#[test]
fn test_partial_conversion_cancel_middle_selection() {
    let mut engine = InputMethodEngine::new();
    engine.config.auto_suggest = false;

    // Type "あいうえおかきく"
    for ch in "aiueokakiku".chars() {
        engine.process_key(&press(ch));
    }
    assert_eq!(engine.input_buf.text, "あいうえおかきく");

    // Move cursor to position 2 (after "あい")
    engine.process_key(&press_key(Keysym::HOME));
    engine.process_key(&press_key(Keysym::RIGHT));
    engine.process_key(&press_key(Keysym::RIGHT));
    assert_eq!(engine.input_buf.cursor_pos, 2);

    // Shift+Right ×3 → select "うえお"
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    assert_eq!(engine.input_buf.selection_range(), Some((2, 5)));

    // Space → partial conversion of "うえお"
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Check remaining
    let (before, after) = engine.remaining_after_conversion.as_ref().unwrap();
    assert_eq!(before, "あい");
    assert_eq!(after, "かきく");
    assert_eq!(engine.input_buf.text, "あいうえおかきく");

    // Cancel
    let result = engine.process_key(&press_key(Keysym::ESCAPE));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.preedit().unwrap().text(), "あいうえおかきく");
    assert_eq!(engine.input_buf.text, "あいうえおかきく");
}

#[test]
fn test_partial_conversion_cancel_with_key_release() {
    let mut engine = InputMethodEngine::new();
    engine.config.auto_suggest = false;

    // Type "あいうえお"
    for ch in "aiueo".chars() {
        engine.process_key(&press(ch));
    }
    assert_eq!(engine.input_buf.text, "あいうえお");

    // Select first 3 chars
    engine.process_key(&press_key(Keysym::HOME));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));

    // Space → partial conversion
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Navigate candidate
    engine.process_key(&press_key(Keysym::DOWN));

    // Esc press → cancel
    let result = engine.process_key(&press_key(Keysym::ESCAPE));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.preedit().unwrap().text(), "あいうえお");

    // Esc RELEASE → consumed (prevents app from seeing it) but no state change
    let result = engine.process_key(&release_key(Keysym::ESCAPE));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.preedit().unwrap().text(), "あいうえお");
    assert_eq!(engine.input_buf.text, "あいうえお");

    // Simulate what happens if reset() is called after Esc
    // (e.g., fcitx5 calls reset externally)
    engine.reset();
    // After reset, state is Empty, text is gone
    assert!(matches!(engine.state(), InputState::Empty));
    // This is expected behavior for reset() after cancel — the text IS lost
    // but that's because reset() is designed to discard Composing state.
}

#[test]
fn test_shift_arrow_in_conversion_cancels_and_selects() {
    let mut engine = InputMethodEngine::new();
    engine.config.auto_suggest = false;

    // Type "あいうえお" (a i u e o)
    for ch in "aiueo".chars() {
        engine.process_key(&press(ch));
    }
    assert_eq!(engine.input_buf.text, "あいうえお");

    // Space → full conversion (Conversion state)
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Shift+Right×3 → should cancel conversion, return to Composing with selection "あいう"
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));

    // Should be in Composing with selection (0, 3)
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.input_buf.text, "あいうえお");
    assert_eq!(engine.input_buf.selection_range(), Some((0, 3)));

    // remaining_after_conversion should be cleared
    assert!(engine.remaining_after_conversion.is_none());
}

#[test]
fn test_shift_left_in_conversion_cancels_and_selects_from_end() {
    let mut engine = InputMethodEngine::new();
    engine.config.auto_suggest = false;

    // Type "あいうえお"
    for ch in "aiueo".chars() {
        engine.process_key(&press(ch));
    }

    // Space → Conversion
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Shift+Left×2 → cancel, cursor at end, select left 2 chars → "えお" selected
    engine.process_key(&press_shift_key(Keysym::LEFT));
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    engine.process_key(&press_shift_key(Keysym::LEFT));

    assert_eq!(engine.input_buf.text, "あいうえお");
    assert_eq!(engine.input_buf.selection_range(), Some((3, 5)));
}

#[test]
fn test_consecutive_partial_conversion() {
    let mut engine = InputMethodEngine::new();
    engine.config.auto_suggest = false;

    // Type "あいうえお" (a i u e o)
    for ch in "aiueo".chars() {
        engine.process_key(&press(ch));
    }
    assert_eq!(engine.input_buf.text, "あいうえお");

    // Space → full conversion
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Shift+Right×3 → cancel conversion, select "あいう"
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.input_buf.selection_range(), Some((0, 3)));

    // Space → partial conversion of "あいう"
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));
    // remaining should be ("", "えお")
    let (before, after) = engine.remaining_after_conversion.as_ref().unwrap();
    assert_eq!(before, "");
    assert_eq!(after, "えお");

    // Enter → bake partial conversion result into composing buffer
    let result = engine.process_key(&press_key(Keysym::RETURN));
    assert!(result.consumed);

    // Bake approach: no Commit action — result is baked into composing buffer
    let has_commit = result
        .actions
        .iter()
        .any(|a| matches!(a, EngineAction::Commit(_)));
    assert!(!has_commit, "Partial conversion should bake, not commit");

    // Should be in Composing with baked text (fallback "あいう" + "えお")
    assert!(
        matches!(engine.state(), InputState::Composing { .. }),
        "Expected Composing after bake, got {:?}",
        std::mem::discriminant(engine.state())
    );
    // Without model inference, the fallback candidate is hiragana itself
    assert_eq!(engine.input_buf.text, "あいうえお");

    // Now select "えお" and convert it too
    engine.process_key(&press_key(Keysym::HOME));
    // Skip past the baked portion (3 chars)
    engine.process_key(&press_key(Keysym::RIGHT));
    engine.process_key(&press_key(Keysym::RIGHT));
    engine.process_key(&press_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    assert_eq!(engine.input_buf.selection_range(), Some((3, 5)));

    // Space → partial conversion of "えお"
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Enter → bake again
    let result = engine.process_key(&press_key(Keysym::RETURN));
    assert!(result.consumed);
    assert!(matches!(engine.state(), InputState::Composing { .. }));

    // Final Enter in Composing → commit everything
    let result = engine.process_key(&press_key(Keysym::RETURN));
    assert!(result.consumed);
    let has_commit = result
        .actions
        .iter()
        .any(|a| matches!(a, EngineAction::Commit(_)));
    assert!(has_commit, "Final Enter in Composing should commit");
    assert!(matches!(engine.state(), InputState::Empty));
}

#[test]
fn test_partial_conversion_from_end_bakes_and_keeps_before() {
    let mut engine = InputMethodEngine::new();
    engine.config.auto_suggest = false;

    // Type "あいうえお"
    for ch in "aiueo".chars() {
        engine.process_key(&press(ch));
    }

    // Space → full conversion
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));

    // Shift+Left×2 → cancel conversion, select "えお" from end
    engine.process_key(&press_shift_key(Keysym::LEFT));
    engine.process_key(&press_shift_key(Keysym::LEFT));
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    assert_eq!(engine.input_buf.selection_range(), Some((3, 5)));

    // Space → partial conversion of "えお"
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));
    // remaining should be ("あいう", "")
    let (before, after) = engine.remaining_after_conversion.as_ref().unwrap();
    assert_eq!(before, "あいう");
    assert_eq!(after, "");

    // Enter → bake: "あいう" + converted "えお" stays in composing
    let result = engine.process_key(&press_key(Keysym::RETURN));
    assert!(result.consumed);

    // No Commit — result baked into composing
    let has_commit = result
        .actions
        .iter()
        .any(|a| matches!(a, EngineAction::Commit(_)));
    assert!(!has_commit, "Partial conversion should bake, not commit");

    // Should be in Composing with full baked text
    assert!(matches!(engine.state(), InputState::Composing { .. }));
    // Fallback candidate for "えお" is "えお", so baked text = "あいう" + "えお"
    assert_eq!(engine.input_buf.text, "あいうえお");

    // User can now select "あいう" and convert it
    engine.process_key(&press_key(Keysym::HOME));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    engine.process_key(&press_shift_key(Keysym::RIGHT));
    assert_eq!(engine.input_buf.selection_range(), Some((0, 3)));

    // Space → partial conversion of "あいう"
    engine.process_key(&press_key(Keysym::SPACE));
    assert!(matches!(engine.state(), InputState::Conversion { .. }));
}

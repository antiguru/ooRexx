use rexx_inventory::errors;

#[test]
fn every_message_from_the_catalogue_is_present() {
    // 56 <Message> + 648 <SubMessage> = 704, as of 8c880bdd. If this changes,
    // the C++ tree gained or lost an error and the Rust side must follow.
    assert_eq!(errors::MESSAGES.len(), 704);
    assert_eq!(errors::MESSAGES.iter().filter(|m| m.sub == 0).count(), 56);
}

#[test]
fn a_major_carries_its_own_text_with_no_substitutions() {
    let m = errors::lookup(3, 0).expect("error 3 exists");
    assert_eq!(m.text, "Failure during initialization.");
    assert_eq!(m.symbol, "Error_Program_unreadable");
}

#[test]
fn a_submessage_is_keyed_by_the_pair_and_renders_markup_like_the_oracle() {
    let m = errors::lookup(3, 1).expect("error 3.001 exists");
    assert_eq!(m.number, 200, "MessageNumber is independent of the code");
    // <q> keeps its quotes; <Sub position="1"/> becomes &1.
    // Compare against RexxErrorMessages.h:62.
    assert_eq!(m.text, "Failure during initialization: File \"&1\" is unreadable.");
}

#[test]
fn q_markup_around_literal_text_still_renders_its_quotes() {
    // 36 messages wrap literal text in <q> with no substitution at all.
    // Dropping the wrapper would diverge from the oracle on every one.
    let m = errors::lookup(6, 0).expect("the unmatched-quote error exists");
    assert_eq!(m.text, "Unmatched \"/*\" or quote.");
}

#[test]
fn error_13_is_invalid_character_in_program() {
    let m = errors::lookup(13, 0).expect("error 13 exists");
    assert!(m.text.to_ascii_lowercase().contains("invalid character"));
}

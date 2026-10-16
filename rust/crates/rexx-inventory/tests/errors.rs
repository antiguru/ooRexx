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
    assert_eq!(
        m.text,
        "Failure during initialization: File \"&1\" is unreadable."
    );
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

/// The `N` a message constant's type carries.
fn arity<const N: usize>(_: errors::Msg<N>) -> usize {
    N
}

/// The highest `&k` in the catalogue text of `(major, sub)`.
fn highest_placeholder(code: (u16, u16)) -> usize {
    let text = errors::lookup(code.0, code.1)
        .expect("the constant names a row")
        .text;
    text.split('&')
        .skip(1)
        .filter_map(|rest| rest.chars().next()?.to_digit(10))
        .max()
        .map_or(0, |k| k as usize)
}

#[test]
fn a_constants_arity_is_its_texts_highest_placeholder() {
    let sample = [
        (errors::E35_1.code(), arity(errors::E35_1)),
        (errors::E13_1.code(), arity(errors::E13_1)),
        (errors::E10_2.code(), arity(errors::E10_2)),
        (errors::E99_943.code(), arity(errors::E99_943)),
        (errors::E6_1.code(), arity(errors::E6_1)),
    ];
    for (code, n) in sample {
        assert_eq!(n, highest_placeholder(code), "{code:?}");
    }
    assert_eq!(arity(errors::E13_1), 2);
    assert_eq!(arity(errors::E10_2), 3);
}

#[test]
fn a_message_with_no_placeholder_takes_no_inserts() {
    let _: errors::Msg<0> = errors::E8_2;
    assert_eq!(highest_placeholder(errors::E8_2.code()), 0);
    assert_eq!(errors::E8_2.code(), (8, 2));
}

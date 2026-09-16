/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

use super::*;

// ---- ADDRESS, the environment-naming forms ----

/// The running activation's `(current, alternate)` pair as text, `None`
/// staying `None` because it is not a name -- it is "the platform's
/// default environment", which this crate does not yet have a spelling
/// for.
fn address_pair(interp: &Interp) -> (Option<String>, Option<String>) {
    let show = |name: &Option<Rc<[u8]>>| {
        name.as_ref()
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
    };
    let state = &interp.activation().address;
    (show(&state.current), show(&state.alternate))
}

/// Measured on the oracle through `ADDRESS()`, four spellings of one
/// intent: `address envC` -> `ENVC`, `address 'LiTeRaL'` -> `LiTeRaL`,
/// and both computed forms -> the value unchanged.
#[test]
fn only_the_symbol_form_upcases_the_environment_name() {
    for (source, want) in [
        (&b"address envC\n"[..], "ENVC"),
        (&b"address 'LiTeRaL'\n"[..], "LiTeRaL"),
        (&b"nm = 'mIxEd'\naddress value nm\n"[..], "mIxEd"),
        (&b"nm = 'mIxEd'\naddress (nm)\n"[..], "mIxEd"),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, source).expect("test program runs");
        assert_eq!(
            address_pair(&interp).0.as_deref(),
            Some(want),
            "{}",
            String::from_utf8_lossy(source)
        );
    }
}

/// Bare `ADDRESS` swaps the pair, forever.
#[test]
fn bare_address_is_a_toggle_and_not_a_stack() {
    for (toggles, current, alternate) in [
        (0, "ENVB", "ENVA"),
        (1, "ENVA", "ENVB"),
        (2, "ENVB", "ENVA"),
        (3, "ENVA", "ENVB"),
        (4, "ENVB", "ENVA"),
    ] {
        let mut source = b"address envA\naddress envB\n".to_vec();
        for _ in 0..toggles {
            source.extend_from_slice(b"address\n");
        }
        let mut interp = Interp::new();
        run_source(&mut interp, &source).expect("test program runs");
        assert_eq!(
            address_pair(&interp),
            (Some(current.to_string()), Some(alternate.to_string())),
            "after {toggles} bare ADDRESS"
        );
    }
}

/// A bare `ADDRESS` before any other `ADDRESS` changes nothing, and the
/// default is a name the pair can toggle back **to**.
#[test]
fn a_bare_address_with_no_prior_environment_changes_nothing() {
    const LEAD: &str = "address\naddress\naddress\n";
    for (tail, current, alternate, what) in [
        (
            "",
            None,
            None,
            "three bare ADDRESSes before any other ADDRESS",
        ),
        (
            "address envA\n",
            Some("ENVA"),
            None,
            "the default is still what the first set displaces",
        ),
        (
            "address envA\naddress\n",
            None,
            Some("ENVA"),
            "a bare ADDRESS must toggle back to the default, not decline \
             to move because the alternate is spelled None",
        ),
    ] {
        let mut interp = Interp::new();
        run_source(&mut interp, format!("{LEAD}{tail}").as_bytes()).expect("test program runs");
        assert_eq!(
            address_pair(&interp),
            (current.map(str::to_string), alternate.map(str::to_string)),
            "{what}"
        );
    }
}

/// Setting the same name twice leaves the toggle with nothing to swap:
/// `set` discards the *old* alternate rather than keeping a history.
/// Measured on the oracle, `address envA; address envA` then three bare
/// `ADDRESS`es, all five `ADDRESS()` readings `ENVA`.
#[test]
fn setting_the_same_environment_twice_leaves_the_toggle_nothing_to_do() {
    for toggles in 0..=3 {
        let mut source = b"address envA\naddress envA\n".to_vec();
        for _ in 0..toggles {
            source.extend_from_slice(b"address\n");
        }
        let mut interp = Interp::new();
        run_source(&mut interp, &source).expect("test program runs");
        assert_eq!(
            address_pair(&interp),
            (Some("ENVA".to_string()), Some("ENVA".to_string())),
            "after {toggles} bare ADDRESS"
        );
    }
}

/// The environment is **per activation**: a callee's own `ADDRESS` dies
/// with it, and so does its own toggle.
#[test]
fn a_callees_own_environment_does_not_survive_the_return() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"address outer\ncall sub\nexit\nsub:\naddress inner\naddress\nreturn\n",
    )
    .expect("test program runs");
    assert_eq!(address_pair(&interp), (Some("OUTER".to_string()), None));
}

/// 250 bytes is accepted and 251 raises 29.1, on both setting forms.
#[test]
fn an_environment_name_over_two_hundred_and_fifty_bytes_raises_29_1() {
    for length in [250usize, 251] {
        let name = "z".repeat(length);
        for source in [
            format!("address '{name}'\n"),
            format!("nm = '{name}'\naddress value nm\n"),
        ] {
            let mut interp = Interp::new();
            let outcome = run_source(&mut interp, source.as_bytes());
            if length == 250 {
                outcome.expect("a 250-byte name is accepted");
                assert_eq!(address_pair(&interp).0.as_deref(), Some(name.as_str()));
                continue;
            }
            let Err(Failure::Raised(raised)) = outcome else {
                panic!("expected 29.1 for a {length}-byte name, got {outcome:?}");
            };
            assert_eq!((raised.number, raised.sub), (29, 1));
            assert_eq!(
                raised.additional,
                vec![b"250".to_vec(), name.clone().into_bytes()],
                "the limit and the whole rejected name, untruncated"
            );
            assert_eq!(
                address_pair(&interp),
                (None, None),
                "a rejected name must not have been installed"
            );
        }
    }
}

/// `address with output stem o.` is **not** a `WITH` form. `WITH` is a
/// keyword only after an environment or a `VALUE` expression (`addressNew`),
/// so that clause names an environment called `WITH` and runs `OUTPUT STEM
/// O.` as a command against it -- measured, the oracle answers `>>> "OUTPUT
/// STEM O."` and `+++ "RC(30)"` on stderr at exit 0.
///
/// **`RC` is what parts the two readings.** A command reached an environment
/// nothing is registered for, so it is 30; a `WITH` form would have issued no
/// command at all and left `rc` deriving its own name.
#[test]
fn with_after_no_environment_names_an_environment_called_with() {
    let mut interp = Interp::new();
    let printed = say_output(&mut interp, b"address with output stem o.\nsay rc\n");
    assert_eq!(String::from_utf8_lossy(&printed), "30\n");
}

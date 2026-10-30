#[test]
fn registered_executable_survives_dictionary_mutation_and_collection() {
    for mutation in [
        ".k~define('FOO', .k~method('OTHER'))",
        ".k~define('FOO')",
        ".k~define('FOO', .nil)",
    ] {
        for prefetch in [false, true] {
            let source = format!(
                "m=.k~method('BODY')~copy~~setPrivate~~setUnguarded\n.k~define('FOO',m)\n.local~installed=.k~method('FOO')\ndrop m\n.s~new~go\nexit\n::class k\n::method go\nself~foo\n::method body protected\n{}\n{mutation}\nm=.context~executable\nsay (m == .installed) m~isGuarded m~isPrivate m~isProtected\n::method other\nreturn 17\n::class s subclass k\n",
                if prefetch {
                    "before=.context~executable"
                } else {
                    "nop"
                }
            );
            let outcome = crate::run_program_collect_every_alloc(
                "/identity.rex",
                source.into_bytes(),
                crate::Invocation::none(),
            );
            assert_eq!(outcome.exit_code, 0, "{:?}", outcome);
            assert_eq!(
                outcome.stdout, b"1 0 1 1\n",
                "{mutation}, prefetch={prefetch}"
            );
        }
    }
}

#[test]
fn executing_copies_and_enhanced_metaclass_entries_survive_collection() {
    for (name, source, expected) in [
        (
            "identity",
            include_str!("../../../../../corpus/lang/executing_method_after_redefinition.rex"),
            "0 replace 1 0 1 1\nsame 1\n0 remove 1 0 1 1\nsame 1\n0 hide 1 0 1 1\nsame 1\n1 replace 1 0 1 1\nsame 1\n1 remove 1 0 1 1\nsame 1\n1 hide 1 0 1 1\nsame 1\nBAR 1 1 0 1\nFOO 1 0 1 1\n",
        ),
        (
            "enhanced",
            include_str!("../../../../../corpus/lang/enhanced_metaclass_method_identity.rex"),
            "entry pong 1\ninherited pong 1\nheld 1 1\nlisted 1 1\nfound 1\nheld 1 1\nlisted 1 1\nfound 1\nheld 1 1\nlisted 1 1\nfound 1\nobject own 1\nordinary ordinary 1\n",
        ),
    ] {
        let outcome = crate::run_program_collect_every_alloc(
            name,
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 0, "{:?}", outcome);
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), expected, "{name}");
    }
}

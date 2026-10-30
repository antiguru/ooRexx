#[test]
fn registered_executable_survives_dictionary_mutation_and_collection() {
    for mutation in [".k~define('FOO', .k~method('OTHER'))", ".k~define('FOO')", ".k~define('FOO', .nil)"] {
        for prefetch in [false, true] {
            let source = format!("m=.k~method('BODY')~copy~~setPrivate~~setUnguarded\n.k~define('FOO',m)\n.local~installed=.k~method('FOO')\ndrop m\n.s~new~go\nexit\n::class k\n::method go\nself~foo\n::method body protected\n{}\n{mutation}\nm=.context~executable\nsay (m == .installed) m~isGuarded m~isPrivate m~isProtected\n::method other\nreturn 17\n::class s subclass k\n", if prefetch { "before=.context~executable" } else { "nop" });
            let outcome = crate::run_program_collect_every_alloc("/identity.rex", source.into_bytes(), crate::Invocation::none());
            assert_eq!(outcome.exit_code, 0, "{:?}", outcome);
            assert_eq!(outcome.stdout, b"1 0 1 1\n", "{mutation}, prefetch={prefetch}");
        }
    }
}

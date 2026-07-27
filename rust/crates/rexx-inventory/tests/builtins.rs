#[test]
fn the_builtin_table_has_81_entries() {
    // The table at BuiltinFunctions.cpp:3042 holds 81 entries plus a leading
    // NULL dummy, which the extractor skips.
    assert_eq!(rexx_inventory::builtins::NAMES.len(), 81);
}

#[test]
fn table_order_is_preserved_because_the_parser_indexes_by_position() {
    // NOT alphabetical: the table is mostly sorted but has an appended tail
    // (...X2D, XRANGE, USERID, LOWER, UPPER, RXFUNCADD, RXFUNCDROP,
    // RXFUNCQUERY, ENDLOCAL, SETLOCAL, QUALIFY, GC). Sorting it would break
    // the index the parser resolves builtins through, so the test pins the
    // ends rather than asserting an ordering.
    assert_eq!(rexx_inventory::builtins::NAMES[0], "ABBREV");
    assert_eq!(rexx_inventory::builtins::NAMES[80], "GC");
}

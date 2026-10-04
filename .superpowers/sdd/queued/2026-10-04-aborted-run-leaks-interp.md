# An aborted run leaks its Interp

Since Task 18 (2fd6b05b3), a run that ends with a native call or command still in flight (deadline,
loud refusal) leaks its `Interp` in `execute_on`, and its pool threads stay blocked on a lend that
never comes. Harmless for the rexx binary, which exits. The interpreter API (embedding) needs the
run to cancel or wait out in-flight calls and free the Interp. Ruling P53.

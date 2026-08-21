/* A real shared library, not LIBRARY REXX. The two forms share one row --
   a row is (directive, keyword, position) -- and only this one has a probe,
   so if the LIBRARY REXX form is ever moved to a different phase it has no
   row here. See owning_phase's doc in tests/gate_table_d.rs. */
say 'main'
::routine r external 'LIBRARY zzznolib zzzr'

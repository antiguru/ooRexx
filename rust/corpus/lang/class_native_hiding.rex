/* Task 21: `Setup.cpp`'s native hiding and removal, read back through
   `~method`, which is the class's own instance dictionary and so sees both.

   `.Stem` and `.VariableReference` hide the comparison operators: the entry
   is present and holds `.nil`. `.Queue` takes names back off the set it was
   donated from `.Array`: `APPEND` survives the donation and is the decoy the
   pair needs, since without it a build that donated nothing to `.Queue` at
   all would pass the removal rows. `.Array`'s own answers are what the
   removed names are read against, name for name. */
say .Stem~method("==")
say .Stem~method("\==")
say .VariableReference~method("==")
say .VariableReference~method("><")
say .Queue~method("APPEND")
say .Array~method("SORT")
say .Array~method("MAKESTRING")

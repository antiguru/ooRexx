-- A library procedure's Routine object that no directive binds still has an
-- annotation table, an empty one.
r = .Routine~loadExternalRoutine('yy', 'LIBRARY orxfunction TestGetRoutine')
say r~class~id r~annotation('k')

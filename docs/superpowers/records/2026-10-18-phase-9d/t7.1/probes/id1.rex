lr = .Routine~loadExternalRoutine('TestRaiseException0', 'LIBRARY orxfunction TestRaiseException0')
say (lr == .routines~r) (lr == .routines~r2) (.routines~r == .routines~r2)
::routine r external 'LIBRARY orxfunction TestRaiseException0'
::routine r2 external 'LIBRARY orxfunction TestRaiseException0'

/* ::REQUIRES LIBRARY loads the library once however often it is named, so a
   program naming it twice, and naming it again through
   Package~loadLibrary, sees one load and no error. loadExternalMethod and
   loadExternalRoutine answer .nil for a library that is not there and for a
   procedure it does not export. */
say 'requires ran'
say 'load' .context~package~loadLibrary('rxregexp')
say 'again' .context~package~loadLibrary('rxregexp')
say 'absent' .context~package~loadLibrary('zorkolib')
say 'method' .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Parse')~class~id
say 'nomethod' .Method~loadExternalMethod('m', 'LIBRARY rxregexp NoSuchEntry')
say 'nolib' .Method~loadExternalMethod('m', 'LIBRARY zorkolib RegExp_Parse')
say 'noroutine' .Routine~loadExternalRoutine('r', 'LIBRARY rxregexp NoSuchRoutine')
::requires 'rxregexp' LIBRARY

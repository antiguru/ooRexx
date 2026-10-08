/* define with an ::ATTRIBUTE getter's Method object, which reads the variable
 * in the defining class's pool, where it is unset. */
c = .object~subclass('K')
c~define('AA', .u~method('A'))
say c~new~aa
::class u
::attribute a

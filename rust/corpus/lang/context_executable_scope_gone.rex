/* .context~executable in a method whose class took the name back while it
 * ran. */
o = .t~new
say o~m
::class t
::method m
.t~define('M')
return .context~executable~class~id

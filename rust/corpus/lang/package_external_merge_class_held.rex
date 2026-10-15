/* An external routine's public class, merged into its caller's package
   without importing the callee, outlives the callee's package. */
call lineout 'extclass.rex', "return 'ext'"
call lineout 'extclass.rex', '::class kx public'
call lineout 'extclass.rex', '::method v'
call lineout 'extclass.rex', "  return 'kxv' copies('x', 2)"
call stream 'extclass.rex', 'c', 'close'
call sub
do zi = 1 to 1000
  zz = copies('x', 100000)
end
say 'a' .context~package~findClass('KX')
say 'b' .context~package~importedClasses~hasIndex('KX') .context~package~importedPackages~items
c = .context~package~findClass('KX')
if c \== .nil then say 'c' c~new~v
exit
sub: procedure
  call extclass
  return

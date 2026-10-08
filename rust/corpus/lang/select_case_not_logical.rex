/* A SELECT CASE value's == answer that is not 0 or 1 is Error 34.905. */
select case .c~new
  when 'x' then say 'x'
  otherwise say 'otherwise'
end
exit
::class c
::method '=='; return 'banana'

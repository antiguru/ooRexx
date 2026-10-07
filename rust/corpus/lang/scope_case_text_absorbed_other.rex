/* The other half of scope_case_text_absorbed.rex: the callee's value would
   match the absorbed WHEN and its own SELECT CASE's value does not. */
select case 'a'
  when g() then when 'zz' then say 'absorbed matched'
  otherwise say 'other'
end
say 'done'
exit
g:
  select case 'zz'
    when 'zz' then return 'a'
    otherwise return 'y'
  end

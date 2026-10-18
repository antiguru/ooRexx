call (.probe~new~run)
::class probe
::method run
  cond = 3
  start:
  signal on any
  cond += 1
  drop rc
  select case cond
    when 4 then a = 1 / 9999999999
    when 5 then "object"~no_method
    when 6 then call datatype .RexxInfo
    when 7 then call charin "/"
    when 8 then call datatype no_value
    when 9 then call no_target
    otherwise return
  end
  say 'cond' cond "didn't raise"
  any:
  object = condition("object")
  say cond condition('c') '|' condition('i') '|' condition('d') '|' var('RC') '|' object~allIndexes~makearray~sort~toString(,',')
  a = condition('a')
  if a~isA(.array) then say '  A' a~items a~toString(,',')
  else say '  A' a
  signal start

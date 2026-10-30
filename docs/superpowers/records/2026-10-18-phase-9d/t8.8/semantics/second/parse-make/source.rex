k = .key~new
parse value 'left KM right' with before (k) after
say '['before']['after']'
parse caseless value 'left km right' with before (k) after
say '['before']['after']'
parse value 'abcdef' with before +(2) after
say before after
exit
::class key
::method makeString
  say 'make'
  return 'KM'

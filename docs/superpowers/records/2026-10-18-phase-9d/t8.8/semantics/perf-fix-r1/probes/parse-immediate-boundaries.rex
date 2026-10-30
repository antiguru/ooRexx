/* Immediate and heap patterns keep the same search and positional rules. */
short = 'xy'
whole = 12
long = 'long-pattern-more-than-inline'
parse value 'leftxyright' with before (short) after
say '['before']['after']'
parse value 'left12right' with before (whole) after
say '['before']['after']'
parse value 'leftlong-pattern-more-than-inlineright' with before (long) after
say '['before']['after']'
parse caseless value 'leftXYright' with before (short) after
say '['before']['after']'
parse caseless value 'left12right' with before (whole) after
say '['before']['after']'
parse caseless value 'leftLONG-PATTERN-MORE-THAN-INLINEright' with before (long) after
say '['before']['after']'
parse value 'abcdef' with before +(2) after
say '['before']['after']'

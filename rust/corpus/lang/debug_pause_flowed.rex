/* Interactive debug pauses after NUMERIC as after SAY, so each line typed at
   a pause runs before the next clause. */
trace ?a
say 'c1'
numeric digits 9
say 'c3'
say 'c4'

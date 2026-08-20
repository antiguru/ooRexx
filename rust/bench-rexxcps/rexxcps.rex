/* REXXCPS 2.2, canonicalised: the loop counts are fixed and the trial loop  */
/* that scaled them to an elapsed-time target is gone. The 1000-clause timed */
/* body below is REXXCPS 2.2's, verbatim. See README.md for what changed and */
/* why this file is not in bench-programs/.                                  */

rexxcps=2.2    /* REXXCPS version; quotable only if code unchanged */

/* Fixed. Both interpreters therefore do identical work, which is what makes
   their wall times comparable; the counts are the ones the oracle's own
   calibration settled on for this machine. */
count=200      /* Repetition count */
averaging=100  /* Averaging-over count */

tracevar='Off' /* Trace setting (for development use) */

signal on novalue
parse source  source 1 system .
parse version version

say '----- REXXCPS' rexxcps '-- Measuring REXX clauses/second -----'
say ' REXX version is:' version
say '       System is:' system

/* Calibrate for the empty do-loop */
empty=0
do i=1 to averaging
  call time 'R'
  do count; end
  empty=time('R')+empty
  end
empty=empty/averaging

/* One trial. The original ran a second one at a scaled count when the first
   came in under a second; a count that depends on how fast the machine was
   that day is what this file exists to remove. */
full=0
do i=1 to averaging
  trace value tracevar
  call time 'R'
  do count;
    /* -----  This is first of the 1000 clauses ----- */
    flag=0; p0='b'
    do loop=1 to 14
      /* This is the "block" comment in loop */
      key1='Key Bee'
      acompound.key1.loop=substr(1234"5678",6,2)
      if flag=acompound.key1.loop then say 'Failed1'
      do j=1.1 to 2.2 by 1.1   /* executed 28 times */
        if j>acompound.key1.loop then say 'Failed2'
        if 17<length(j)-1        then say 'Failed3'
        if j='foobar'            then say 'Failed4'
        if substr(1234,1,1)=9    then say 'Failed5'
        if word(key1,1)='?'      then say 'Failed6'
        if j<5 then do   /* This path taken */
          acompound.key1.loop=acompound.key1.loop+1
          if j=2 then leave
          end
        iterate
        end /* j */
      avar.=1.0''loop
      select
        when flag='string' then say 'FailedS1'
        when avar.flag.2=0 then say 'FailedS2'
        when flag=5+99.7   then say 'FailedS3'
        when flag          then avar.1.2=avar.1.2*1.1
        when flag==0       then flag=0
        end
      if 1 then flag=1
      select
        when flag=='ring'  then say 'FailedT1'
        when avar.flag.3=0 then say 'FailedT2'
        when flag          then avar.1.2=avar.1.2*1.1
        when flag==0       then flag=1
        end
      parse value 'Foo Bar' with v1 +5 v2 .
      trace value trace(); address value address()
      call subroutine 'with' 2 'args', '(This is the second)'1''1
      rc='This is an awfully boring program'; parse var rc p1 (p0) p5
      rc='is an awfully boring program This'; parse var rc p2 (p0) p6
      rc='an awfully boring program This is'; parse var rc p3 (p0) p7
      rc='awfully boring program This is an'; parse var rc p4 (p0) p8
      end loop
    /* -----  This is last of the 1000 clauses ----- */
    end
  full=time('R')+full
  trace off
  end
total=full /* total time */

/* show the counts. The elapsed time is deliberately not on this line: the
   counts are fixed, so this line is the same on both sides and in every run,
   and a reader comparing two reports is comparing the work and not the clock. */
say '        Averaged:' count 'x' averaging 'iterations of 1000 clauses'

/* Now display the statistics */
innertime=total/averaging-empty
thousand=innertime/count
/* Developer's statistics: */
if left(tracevar,1)='O' then nop
 else do
  say
  say 'Total (full DOs):' total 'secs (average of' averaging ,
    'measures of' count 'iterations)'
  say 'Time for one iteration (1000 clauses) was:' thousand 'seconds'
  end

/* And finally, the Result... */
say
say'     Performance:' format(1000/thousand,,0) 'REXX clauses per second'
say

exit


/* Target routine for the timed CALL - called 14 times */
subroutine:
  parse upper arg a1 a2 a3 ., a4
  parse var a3 b1 b2 b3 .
  do 1; rc=a1 a2 a3; parse var rc c1 c2 c3; end
  return

novalue:
  say 'NoValue raised'

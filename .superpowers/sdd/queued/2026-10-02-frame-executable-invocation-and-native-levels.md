# Frame values: an internal call's executable, an INTERPRET frame's invocation, native levels

Found by the Phase 6 S2-S5 Task 9 review (x3, x4, y1); present at a4bde5677 and visible since
Task 9 through TraceObject STACKFRAME and CALLERSTACKFRAME tables.

- An internal call inside a method: `.context~executable` (and the frame's EXECUTABLE) is the
  `Method` on the oracle, a `Routine` here.

```rexx
.w~new~one
::class w
::method one
  call inner
  return
inner:
  say .context~executable~class~id
```

Oracle `Method`, this crate `Routine`, rc 0 both.

- An INTERPRET frame's INVOCATION is minted again on each ask here; the oracle keeps one per
  fragment.

```rexx
interpret 'do i = 1 to 2; say .context~stackframes[1]~invocation .context~invocation; end'
```

Oracle `1 2` twice; this crate `1 2` then `3 2`.

- `.context~stackFrames` leaves out the frame of a primitive method that entered the method
  (`InternalActivationFrame`, `concurrency/ActivationFrame.cpp:100`): `live_levels` holds library
  natives only. Task 9 fix round 1 names that frame as a `>I>` line's CALLERSTACKFRAME
  (`Interp::entering_native_frame`), not in `stackFrames`.

```rexx
.message~new(.w~new, 'two')~send
::class w
::method two
  do f over .context~stackframes
    say f~type f~name
  end
```

Oracle `METHOD TWO`, `METHOD SEND`, `PROGRAM <path>`; this crate omits `METHOD SEND`.

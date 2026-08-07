# ANSI X3.274 compliance: design

**Status:** design, approved 2026-08-07 for the two decisions in section 3.
S0 is specified here; S1 and later are named but deliberately not specified, because deciding their contents is S0's job.

## 1. What this is

A new work axis: read ANSI X3.274-1996 and check this project against it.
It is **not** a build phase and it does not sit after Phase 10.
It becomes possible incrementally as build phases land, so it runs beside them.

**The series is `S0`, `S1`, ... -- `S` for *standard*.**
It is deliberately not called *conformance*: this project already uses "conformance" for the ooTest rungs L0 to L3-full, and `phase-9-exclusions.txt`, `conformance-baseline.md` and every gate document use it in that sense.
Two meanings of one word across a plan this size is how a criterion gets read as satisfied by the wrong evidence.

## 2. What the standard actually is, measured 2026-08-07

The published ANSI X3.274-1996 is sold by ANSI.
The **pre-publication approved final draft** is free at `https://www.rexxla.org/rexxlang/standards/j18pub.pdf`, 414,740 bytes, and rexxla states it is functionally equivalent to the published standard.

**It is machine-readable and it is the whole document.**
`pdfinfo` reports `10 page(s)`, which is wrong -- it is a PDF 1.1 written by PageMaker 5.0 in 1998 and the page objects do not map to logical pages.
`pdftotext` yields **11,786 lines**, and the document's own index runs to page **167**.

**The standard defines each built-in function as executable Rexx, not as prose.**
This is the finding that decides the method. Section 9.3.21, verbatim:

```rexx
call CheckArgs 'rANY rWHOLE>0 oWHOLE>=0 oPAD'
String = #Bif_Arg.1
Num    = #Bif_Arg.2
if #Bif_ArgExists.3 then Length = #Bif_Arg.3
else Length = length(String)+1-Num
...
        call #Raise 'SYNTAX', 23.1, b2x(#Outcome)
```

So a compliance check can be **differential**, the same method the rest of this project uses, rather than an exercise in reading prose and forming opinions.
The `Config_*` interfaces those definitions call (`Config_Substr`, `Config_C2B`, `Config_Xrange`, ...) are themselves specified in section 5.

**One divergence is already visible in that snippet.** ANSI raises **23.1** where Phase 4c measured ooRexx raising **40.23** for a bad `SUBSTR` pad argument.

### What is not usable

**The "ANSI BIF Tester" on rexxla.org is a web form, not a downloadable tool.**
It is valuable as a description -- it "executes the actual Rexx code that defines the BIFs in the ANSI Standard", which independently confirms the executable-definitions finding -- but it is somebody's server.
**It is not an oracle and nothing in this series may script against it.**
Its own page also excludes nine BIFs (`STREAM LINES LINEIN LINEOUT CHARS CHARIN CHAROUT VALUE`, and no nesting), so it could not serve as one regardless.

Also on that page and not yet read: the **errata** (`stan_err`), which modifies the standard and therefore binds any compliance claim, and **Date and Time Conversions in the Rexx Standard** (`stan_dt`).

## 3. The two decisions taken

### D-S1 -- a finding is a three-way map, never a two-way comparison

This project is defined as byte-for-byte agreement with ooRexx.
**Where ooRexx diverges from ANSI, this crate diverges too, by design.**
A compliance check that compares the Rust crate directly against the standard therefore reports our deliberate choices as our defects.

So every finding records three values -- what ANSI specifies, what ooRexx does, what this crate does -- and is classified by which pair disagrees:

| ANSI | ooRexx | us | classification |
|---|---|---|---|
| X | X | **Y** | **our defect.** Fix it. Also a hole in the differential corpus, since the oracle would have caught it. |
| X | **Y** | Y | **ooRexx divergence.** Not our bug. Record it; filing upstream is Moritz's call. |
| X | **Y** | **Z** | **two defects stacked.** Rarest and worst: we match neither. |
| X | X | X | compliant. Record it; a checked row is evidence. |

The third row is the one a two-way check cannot see at all, and it is the reason for the three-way rule rather than a preference for thoroughness.

### D-S2 -- whole language, several phases, S0 decides the split

The reach is the whole language, not just the built-ins.
But the decomposition is **not** guessed at now: S0 reads the document and proposes the split, because the standard's own structure is the only sane basis for it and nobody has read all 167 pages yet.

## 4. What S0 does

S0 is setup. It writes no compliance findings and fixes no code.

1. **Vendor the sources.** `j18pub.pdf` plus its extracted text, the errata, and the date/time document, committed under `docs/standards/` with their retrieval date and URL. The standard is the input to every later phase and a phase whose input can change under it cannot be re-run. Record that the draft is *stated* to be functionally equivalent to the published standard and that nobody here has compared them -- that is an assumption, not a measurement.
2. **Settle the BIF population, which is not obvious.** Two independent counts of the same document disagree: `/bin/grep -c "call CheckArgs"` gives **66**, and the index's `^[A-Z]+ function [0-9]+$` entries give **63** distinct names -- the regex misses BIFs whose index entry carries more than one page number. Neither is yet a population. Produce the actual set, by a method stated in the file, and cross-check it two ways.
   **This is the phase's first opportunity to make the error the phase exists to catch**: a count over a document is not a population, and this project has shipped that mistake four times.
3. **Map ANSI's BIF set against ours, three ways.** Ours is 81 known names, 66 in scope, 15 excluded (`phase-4-exclusions.txt`). ANSI's set is not ours: it contains `QUALIFY`, `STREAM`, `CHARIN`, `CHAROUT`, `CHARS`, `LINEIN`, `LINEOUT` and `LINES`, **several of which are in our excluded 15** -- so we are non-compliant there by construction, and that is a legitimate finding rather than a surprise. Produce the three-way set map: ANSI-only, ours-only, both.
   Note also that our in-scope count is 66 and `call CheckArgs` occurs 66 times. **This is arithmetic coincidence, not correspondence** -- the sets differ. Say so in the file, because the numbers invite the wrong inference.
4. **Build the extractor and prove it is not vacuous.** Turn each ANSI BIF definition into a runnable reference implementation, with the section 5 `Config_*` scaffolding. It must run under the oracle and produce answers. The negative control: a deliberately corrupted extraction must fail loudly rather than silently producing an empty program -- this project already shipped an L1 harness whose extracted programs executed **nothing at all** while a differential over them reported success, and that is the exact failure mode here.
5. **Propose the S1+ decomposition** as a written plan, based on the document's own structure, with each phase's entry condition given in terms of which build phase must have landed first.

## 5. What this closes

**Phase 2's exit gate has an open criterion this series discharges.**
`phase-2-gate.md` section 3 records "ANSI X3.274 vectors -- **CANNOT ASSESS**", because "no vector files exist anywhere in the tree, and the session was offline".
The blocker was availability, not applicability, and it no longer holds.

That gate's own reasoning needs one correction when S0 cites it.
It argues the criterion "partly answers itself", since where the standard and ooRexx disagree the interpreter wins, making the vectors "a secondary check by construction".
The first half is right and is D-S1's premise.
The conclusion does not follow: the criterion asks for **an enumeration of the deviations**, and the gate says so in the next sentence. A rule for resolving disagreements is not a list of them.

## 6. Two hazards specific to this series

**A prose standard invites interpretation, and interpretation is not measurement.**
Where the standard is executable, run it. Where it is prose, a finding must quote the clause verbatim and state the reading it depends on. A compliance claim resting on a paraphrase is worth nothing, and this project's measured failure rate on restating a result in different words is high enough that the rule has to be mechanical.

**The standard is a 1996 document describing classic Rexx; ooRexx is a superset.**
Many divergences will be deliberate extensions rather than defects. The three-way map handles that, but only if "ooRexx does more than ANSI requires" is distinguished from "ooRexx does something ANSI forbids". Those are different findings and only the second is interesting.

# Task 23 controller notes

Measured by the controller against the shipped oracle, 2026-08-24, before dispatch. **Every premise
in the brief that I could check held.** Unlike Task 21, this brief needs no corrections; what follows
is confirmation plus the two file facts a reader would otherwise have to hunt for.

## The five checks, run as one program, oracle rc 0

    say .TraceObject~option                             ->  N
    say .Supplier~superClasses                          ->  The Object class
    say .array~of(1,2)~supplier~hasMethod("ALLITEMS")   ->  1
    say .String~hasMethod("DEFINECLASSMETHOD")          ->  0
    say .Supplier~hasMethod("INHERITINSTANCEMETHODS")   ->  0
    say .Class~hasMethod("DEFINE")                      ->  1

The `.Supplier` pair is the brief's point made in two lines: the class graph says `Object` alone
while an instance answers `ALLITEMS`, so **no class-graph assertion can see the donation and only a
method set can**.

## `CoreClasses.orx` run directly

Confirmed rc **159**, and the frame names the line:

        47 *-* rexxPackage~addClass('LOCALSERVER', .LocalServer)
    Error 97 running <path> line 47:  Object method not found.
    Error 97.1:  Object "REXXPACKAGE" does not understand message "ADDCLASS".

`use arg rexxPackage` with no argument leaves the symbol as its own uppercased name, and the first
send to it fails. The bootstrap must **call** the file with a Package object.

## Where the three files actually are

* `interpreter/RexxClasses/CoreClasses.orx`
* `interpreter/RexxClasses/StreamClasses.orx`
* `interpreter/platform/unix/PlatformObjects.orx`

The first two are the only `.orx` files in `RexxClasses/`. The unix `PlatformObjects.orx` is **one
line**, `-- Nothing to do currently`, confirming the brief. The windows file is two lines and is
neither read nor embedded by this task, which the task must say rather than leave a CI platform to
discover.

## Read-only

These three files are upstream sources. **Never edit them.** The brief's sha256-at-build-time rule
is what makes an accidental edit a build failure rather than a silent divergence.

---

# BLOCKER FOUND BEFORE DISPATCH, AND THE RULING ON IT

## The bootstrap cannot run without the message scope override send, which is unbuilt

Measured at `5b054d4b5`. Running `CoreClasses.orx` through the crate is **rc 120**:

    rexx-exec: a message scope override on "The StringTable class" is not implemented (Phase 5)

with no traceback frame. The other two files get further: `PlatformObjects.orx` is rc 0 and
`StreamClasses.orx` reaches `98.909 Class "COMPARABLE" not found` at its own line 506, which is only
because a standalone run has not loaded `CoreClasses.orx` first.

**The cause, located exactly.** `CoreClasses.orx:3993` is `::class "TraceObject" subclass StringTable
public`, and its **class-side** `::method activate class` runs `self~activate:super` at `:3998`
before setting `option = 'N'`. Installing a class runs its `ACTIVATE`, so the send is reached at
install time, not at translation time and not by any later call.

**That is the very thing the brief's check 3 exists to detect.** `.TraceObject~option` is `N`
separates a bootstrap that ran `ACTIVATE` from one that did not, and `N` is assigned on the line
after the scope override. The check cannot pass while the send refuses.

**It is not a translation-time gap.** Measured: a program whose *uncalled* method body contains
`self~init:super` is rc 0 on the crate and byte-identical to the oracle. So only the executed path
matters, and the bootstrap executes this one.

**The minimal differential**, oracle rc 0 printing `base+k`, crate rc 120 on both engines:

    say .k~tag
    ::class base
    ::method tag class
      return "base"
    ::class k subclass base
    ::method tag class
      return self~tag:super || "+k"

## The plan contradicts itself here

* Line 58 lists **"the scope-override send (`~m:super`)"** among what old Task 5 delivered. **That is
  false**, measured above.
* Lines 150 and 1699 of the same plan say the opposite and are right: old Task 5 landed
  `Interp::resolve`'s `start_scope` parameter, **not the expression that reaches it**, and a message
  scope override is refused rc 120 on both engines.
* Line 2131 assigns `self~init:super` to 5b, but as **instance construction chaining** -- what INIT
  chaining means once `~new` exists. That is a different question from whether the send form exists
  at all.

## Ruling

**Task 23 builds the message scope override send.** Scope: the expression form `receiver~name:super`
and `receiver~name:ClassName` reaching the `start_scope` parameter `Interp::resolve` already carries,
on both engines, with the oracle's own errors for a scope that is not a superclass. **5b keeps what
line 2131 actually assigns it**: `~new`, `init` chaining semantics and `UNINIT`, none of which this
ruling touches.

Reason: Task 23's "Done when" requires the three files to install and the prologue to reach its
`exit`, and no path to that exists without this send. Deferring it would make the task unachievable
rather than smaller.

**Cost if wrong:** Task 23 grows by one expression form. If it turns out materially larger than that
-- if the send needs the whole `FORWARD`/superclass-search machinery rather than the parameter that
is already there -- report BLOCKED with the measurement and I will re-rule rather than let the task
absorb an unbounded amount of 5b.

**Also correct the plan's line 58** as part of this task: it claims a delivery that does not exist,
and it is the line a later reader would trust.

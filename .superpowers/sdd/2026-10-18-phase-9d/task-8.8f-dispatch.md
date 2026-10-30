# Task 8.8f: Semantic residuals and interrupted stream reads

## Scope

Read `task-8.8f-brief.md` and the common dispatch before implementing its assigned semantic items. This slice also owns the now-demonstrated FIFO EINTR retry defect, which is an original acceptance subpart of the FIFO queued item. Executable identity and worker-spawn propagation belong to the preceding e2 slice. Preserve their error and ownership contracts while using the resulting input helpers.

## Interrupted named-stream reads

The read-only `task-8.8-fifo-eintr.md` and its durable evidence show a real interrupted read on a verified owned reading thread. On the frozen e1 binary, `take(...).read_to_end` retries EINTR and needs a controlled writer unblock before reporting HALT with READY state. The oracle reports NOTREADY and ERROR:4, returns no data, then reports HALT with ERROR:4 without an unblock. Its partial-data control consumes a known prefix but still returns an empty value on the interrupted operation.

* Fix the narrow named-stream read layer so EINTR is returned rather than retried. Preserve the existing HALT policy, Deviation 12 and P60 scope.
* Preserve the measured partial-error behavior, EOF, short-read and byte-count behavior. Avoid an allocation proportional to an unchecked requested count when little or no input exists.
* Reuse the captured readiness/signal/trace driver and add only the meaningful regression controls needed for the new read path. Do not signal unrelated processes, introduce unsafe Rust or broaden into a general cancellation framework.
* The separate closing-LINEOUT oracle byte-consumption defect awaits Moritz's ruling. Preserve the crate's bytes and leave the new Deviation entry pending until that ruling arrives.

## Other semantic contracts

Follow the f brief's local-reference derivations for collection messages, TRACE rendering, regular-file stdout queries, Stem index strings, Number max/min arguments and PARSE conversion failures. Omitted and leading-empty Stem index cases are adjacent boundary witnesses, with a narrow fix only if the oracle confirms the same derived concatenation contract. Preserve Deviations 77 through 81, including TRACE's object-name behavior, rather than using required STRING conversion for every trace rendering.

RAISE DESCRIPTION conversion timing also awaits the explicit requested ruling. Prepare the concrete evidence or correction without writing a numbered Deviation or claiming that item closed before the ruling. Continue independent items while the choice is pending. Root retains any new oracle-departure judgement.

## Proof and handoff

Follow commit-before-verification, exact probe restoration and the common capped foreground workflow. Run meaningful focused semantic regressions, relevant collection/source/coverage checks, formatting and clippy, with all changed descriptors named. Preserve a release runtime pin and its source/build hash for own-BASE performance comparisons. Write `task-8.8f-report.md` and retain exact commands/statuses and semantic rulings under the records tree.

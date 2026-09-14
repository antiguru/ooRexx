# Review packages not copied here

The review packages `scripts/review-package` generated for this plan's task reviews are not in
this record, for the reason `docs/superpowers/records/README.md` gives: each is the output of
`git log --oneline`, `git diff --stat` and `git diff -U10` over one commit range. The ranges, read
from the first and last commit line of each package, with both endpoints on `plan/rust-rewrite`:

| package | oldest commit | newest commit |
|---|---|---|
| `task-1-review-package.md` | `36e1f5b99` | `36e1f5b99` |
| `task-2-fix1-package.md` | `3978a2964` | `152740fbe` |
| `task-2-review-package.md` | `85fd2f136` | `85fd2f136` |
| `task-3-review-package.md` | `b975c534a` | `b975c534a` |
| `task-4-review-package.md` | `6dc03a009` | `6dc03a009` |
| `task-5-review-package.md` | `06fd862d8` | `06fd862d8` |
| `task-6-review-package.md` | `9f3227058` | `9f3227058` |
| `task-7-review-package.md` | `15e7c2797` | `c4128d31d` |
| `task-8-review-package.md` | `ff03676bc` | `f07592242` |
| `task-9-review-package.md` | `0b586d0c7` | `0b586d0c7` |

Reconstruct one with `git log --oneline <oldest>~1..<newest>`, `git diff --stat` and `git diff -U10`
over the same range.

# passwd_home reads /etc/passwd only

Source: Phase 9b Task 6 re-review M2 (.superpowers/sdd/2026-10-13-phase-9b/task-6-rereview.md).
The oracle's `~user` lookups use getpwnam, which consults NSS (LDAP, systemd-homed, sssd). The crate's
passwd_home parses /etc/passwd, so an NSS-only account misses. For cd the shell hand-back usually hides
it. For QUALIFY/.File/streams (qualify_name) an NSS-only ~user diverges. Fix without unsafe: e.g.
`getent passwd NAME` as a fallback subprocess, or a safe NSS-free crate if one exists. Needs a witness
with an account that is not in /etc/passwd, which may not be available on this machine.

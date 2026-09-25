# usage: . mkmsg9.sh; mk N "subject" "body"
mk ()
{
    cat > $S/msg$1.txt <<EOM
$2

$3

\`git blame -w -C -C -C\` recovers the moved lines' history. Instruments
and artifacts: docs/superpowers/records/2026-09-15-file-split/task-10-files/c$1/.

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD
EOM
}

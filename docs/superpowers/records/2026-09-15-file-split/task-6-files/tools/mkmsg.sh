mk () 
{ 
    cat > $S/msg$1.txt <<EOF
$2

$3

\`git blame -w -C -C -C\` recovers the moved lines' history. Instruments
and artifacts: docs/superpowers/records/2026-09-15-file-split/task-6-files/c$1/.

Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_0157HT3JqnRRbgiMDb84iGSD
EOF

}

#!/bin/sh
# Build ooRexx and run its test suite inside a BSD VM.
#
# Shared by the FreeBSD and OpenBSD jobs. It lives in a file rather than inline
# in each job because GitHub Actions does not support YAML anchors, and the
# `uses:` of a VM action cannot be selected from a matrix, so the two jobs
# cannot be collapsed into one.
#
# Everything is written into the workspace, which the VM action rsyncs back to
# the host afterwards. The results are therefore judged on the host by the same
# script every other platform uses, rather than reimplemented here.

set -eux

cmake --version
uname -a

# The OpenBSD run segfaulted in the suite, and an exit status of 139 says only
# that it happened, not where. Cores are enabled so the backtrace below has
# something to work from, the same way the Windows job keeps crash dumps.
ulimit -c unlimited || true

cmake -S . -B build -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build build --parallel 4

WS=`pwd`

build/bin/rexx -v
echo 'say .rexxinfo~version' > hello.rex
build/bin/rexx hello.rex

# How the interpreter is invoked matters on OpenBSD and nowhere else.
# SysProcess::getExecutableFullPath() has no procfs to fall back on there, so it
# reads argv[0] out of sysctl and then, deliberately, only accepts it if it is
# absolute. Started any other way it gives up and .rexxInfo~executable is .nil,
# which the test framework dereferences at startup:
#
#   Error 88.909: Argument 2 must have a string value.
#   64 *-* executableLocation=filespec('location', .rexxInfo~executable)
#
# Both forms are printed rather than assumed, so the log says what each one
# actually produced on this platform.
echo "--- .rexxInfo~executable, started with an absolute path ---"
"$WS/build/bin/rexx" -e 'say .rexxinfo~executable' || true
echo "--- .rexxInfo~executable, found on PATH ---"
( PATH="$WS/build/bin:$PATH"; export PATH; rexx -e 'say .rexxinfo~executable' ) || true

svn checkout --non-interactive --trust-server-cert \
    https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest

# The VM logs in as root, and the suite is not meant to be run that way: its
# ReadMe.first says so, and root defeats every test that asserts something
# cannot be read, written or deleted, because root can do all three regardless
# of the mode bits.
#
# Measured in a local OpenBSD VM, same build and same machine, root against an
# ordinary user: File 2 failures against 0, Stream 1 against 0, SysFileXXX 4
# against 0, ProcessInvocation 1 against 0. Nine failures that are entirely an
# artefact of the login.
#
# So the suite runs as an unprivileged user, which is also what the Linux, macOS
# and Windows runners do, and makes the platforms comparable rather than each
# carrying its own private list of excuses.
TESTUSER=oorexxtest
case `uname -s` in
FreeBSD) pw useradd -n "$TESTUSER" -m -s /bin/sh || true ;;
*)       useradd -m -s /bin/sh "$TESTUSER" || true ;;
esac

# The build tree stays owned by root and is only read; the suite directory has
# to be writable because the tests create files inside it.
chmod -R a+rX "$WS/build"
chown -R "$TESTUSER" ootest

# The test user also has to be able to walk down to the workspace. On the
# runner those directories are world-executable already, so this changes
# nothing there; run the same script in a VM with the tree staged somewhere
# private and it fails with a bare "Permission denied" that says nothing about
# which directory refused. Only the traverse bit is added, never read or write.
dir=$WS
while [ "$dir" != "/" ] && [ -n "$dir" ]; do
    chmod a+x "$dir" 2>/dev/null || true
    dir=`dirname "$dir"`
done

# Run out of the build tree rather than an install, so the suite picks up the
# interpreter, the runtime libraries and the compiled native API test binaries
# together and the native API tests actually run.
PATH="$WS/build/bin:$PATH"
LD_LIBRARY_PATH="$WS/build/lib:$WS/build/bin:${LD_LIBRARY_PATH:-}"
export PATH LD_LIBRARY_PATH

# These have to be writable by the test user, and are created here rather than
# left for the redirection to make as root.
: > "$WS/testresults.txt"
: > "$WS/testexitcode.txt"
chown "$TESTUSER" "$WS/testresults.txt" "$WS/testexitcode.txt"

# The output is teed rather than only redirected. The FreeBSD VM died partway
# through a run once, and because the file only existed inside the VM the
# copyback never happened and there was nothing at all to look at afterwards.
# Going through the job log too means a VM that disappears still leaves the
# evidence behind.
#
# The interpreter is named by absolute path for the reason given above. The
# suite's exit code is recorded, not acted on: a non-zero code is expected
# whenever an environmental test fails, and telling those apart from real
# failures is the host-side check step's job.
#
# stdin comes from /dev/null because in a VM the suite's stdin is the ssh
# session itself. The FreeBSD run died with the connection closed by the remote
# host immediately after ADDRESS.testGroup, which starts child processes that
# read stdin; a child consuming the session's stdin would end it exactly that
# way. Handing the suite an stdin of its own removes that possibility, so if it
# still dies there the cause is in the interpreter rather than in the plumbing.
#
# su -l starts a login shell and discards the environment, so the paths have to
# be set again on the far side. That goes in a script rather than inline in
# su -c: quoting a command that itself contains quoted variable references is
# how the first attempt ended up passing a literal $PATH through, which emptied
# the search path and made the framework's own "id -u" fail with 127.
#
# The heredoc is unquoted so $WS expands as the file is written, while \$PATH
# and \$? are left for the script to evaluate when it runs.
cat > "$WS/run-suite.sh" <<SCRIPT
#!/bin/sh
cd "$WS/ootest" || exit 1

# FreeBSD gives this login a 512MB stack limit, and CALL.testGroup's
# test_stacksize deliberately recurses until the interpreter's stack check
# stops it. Every Rexx activation carries heap with it, so the deeper the
# limit lets it go, the more memory it takes. Measured on FreeBSD 14.2, the
# same test group either way:
#
#   ulimit -s 524288   25 tests, 0 failures, 1m54s, peak RSS 5.6GB
#   ulimit -s 8192     25 tests, 0 failures, 2.3s,  peak RSS 126MB
#
# 5.6GB in the 6144MB VM the runner gives us is most of the machine, and the
# kernel starts killing things, sshd included. That is why the FreeBSD job kept
# ending with the connection closed by the remote host rather than with a test
# failure, and why it never reproduced on a workstation with room to spare.
#
# 8MB is what the Linux and macOS runners use, so this makes the platforms
# comparable rather than leaving BSD the outlier. The test still passes.
ulimit -s 8192 2>/dev/null || true

PATH="$WS/build/bin:\$PATH"
LD_LIBRARY_PATH="$WS/build/lib:$WS/build/bin"
export PATH LD_LIBRARY_PATH
"$WS/build/bin/rexx" testOORexx.rex -s < /dev/null
echo \$? > "$WS/testexitcode.txt"
SCRIPT
chmod 755 "$WS/run-suite.sh"

set +e
su -l "$TESTUSER" -c "$WS/run-suite.sh" 2>&1 | tee "$WS/testresults.txt"
set -e

# Any core left behind gets a backtrace printed into the job log. The suite
# starts child interpreters, so the core that matters may well not be the one
# from the process that was started here, and the search is deliberately wide.
# Nothing in here is allowed to fail the run: this is diagnosis, and a missing
# debugger should not turn a crash into a different error.
set +e
cores=`find "$WS" /var/crash -name '*.core' -o -name 'core' 2>/dev/null | head -n 5`
if [ -n "$cores" ]; then
    debugger=`command -v egdb || command -v gdb`
    for core in $cores; do
        echo "================ core: $core"
        file "$core" || true
        if [ -n "$debugger" ]; then
            "$debugger" -batch \
                -ex 'bt full' \
                -ex 'thread apply all bt' \
                "$WS/build/bin/rexx" "$core" 2>&1 | head -n 200
        else
            echo "no gdb or egdb available to read it"
        fi
    done
else
    echo "no core files found"
fi
set -e

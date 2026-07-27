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

# Run out of the build tree rather than an install, so the suite picks up the
# interpreter, the runtime libraries and the compiled native API test binaries
# together and the native API tests actually run.
PATH="$WS/build/bin:$PATH"
LD_LIBRARY_PATH="$WS/build/lib:$WS/build/bin:${LD_LIBRARY_PATH:-}"
export PATH LD_LIBRARY_PATH

cd ootest
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
set +e
{ "$WS/build/bin/rexx" testOORexx.rex -s < /dev/null; echo $? > "$WS/testexitcode.txt"; } 2>&1 \
    | tee "$WS/testresults.txt"
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

// A native routine whose callback runs on a thread it starts and joins.
// Built from this directory, ORACLE the oracle's source tree:
// g++ -shared -fPIC -O1 -I$ORACLE/api -I$ORACLE/api/platform/unix -I$ORACLE/build -o libforeign.so foreign.cpp -pthread
#include "oorexxapi.h"
#include <thread>
RexxRoutine1(int, fsend, RexxObjectPtr, obj)
{
    RexxThreadContext *tc = context->threadContext;
    std::thread t([tc, obj] { tc->SendMessage0(obj, "SPEAK"); });
    t.join();
    return 7;
}
RexxRoutineEntry foreign_routines[] = { REXX_TYPED_ROUTINE(fsend, fsend), REXX_LAST_ROUTINE() };
RexxPackageEntry foreign_package_entry = { STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0,
    "foreign", "1.0.0", NULL, NULL, foreign_routines, NULL };
OOREXX_GET_PACKAGE(foreign);

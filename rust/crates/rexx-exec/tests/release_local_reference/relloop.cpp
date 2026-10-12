#include "oorexxapi.h"
RexxRoutine1(RexxObjectPtr, RelLoop, int, n)
{
    for (int i = 0; i < n; i++) {
        RexxObjectPtr s = context->String("some string that is long enough to be boxed on the heap");
        context->ReleaseLocalReference(s);
    }
    return context->String("done");
}
RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(RelLoop, RelLoop),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry relloop_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "relloop", "1.0.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(relloop);

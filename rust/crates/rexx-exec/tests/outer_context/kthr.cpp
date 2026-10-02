#include "oorexxapi.h"
static RexxThreadContext *kept = NULL;
RexxRoutine1(RexxObjectPtr, KOuter, RexxObjectPtr, o)
{
    kept = context->threadContext;
    return context->SendMessage0(o, "RUN");
}
RexxRoutine2(RexxObjectPtr, KM, CSTRING, which, RexxObjectPtr, obj)
{
    switch (which[0]) {
    case 'C': kept->ClearCondition(); return context->String("cleared");
    case 'R': kept->RaiseException0(93900); return context->String("raised");
    case 'M': { RexxObjectPtr r = kept->SendMessage0(obj, "BUMP"); return r == NULL ? context->String("nullM") : r; }
    case 'D': kept->DirectoryPut((RexxDirectoryObject)obj, context->String("v"), "K"); return context->String("put");
    case 'S': { RexxObjectPtr s = kept->String("hello"); return s == NULL ? context->String("nullS") : s; }
    case 'H': { context->RaiseException0(93900); kept->ClearCondition(); return context->String("hc"); }
    }
    return context->String("?");
}
RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(KOuter, KOuter),
    REXX_TYPED_ROUTINE(KM, KM),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry kthr_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "kthr", "1.0.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(kthr);

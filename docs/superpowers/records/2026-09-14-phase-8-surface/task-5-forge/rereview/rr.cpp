#include <oorexxapi.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
extern "C" void c_call_uw(void (*)(void *, size_t), void *, size_t);
extern "C" void c_call_nouw(void (*)(void *, size_t), void *, size_t);
static char dlog[512];
static void L(const char *s) { strncat(dlog, s, sizeof(dlog) - strlen(dlog) - 2); strcat(dlog, ";"); }
struct Noisy { const char *n; ~Noisy() { L(n); if (getenv("RR_DTOR_STDERR")) fprintf(stderr, "dtor %s\n", n); } };
typedef void (*T0)(void *, size_t);

#ifndef SECOND
static RexxCallContext *stashed;
RexxRoutine0(RexxStringObject, Dtors) { RexxStringObject s = context->String(dlog); dlog[0] = 0; return s; }
// C++ catch (...) around a Throw, swallowed
RexxRoutine0(int, CatchAll) {
    try { Noisy n = {"inner"}; context->ThrowException0(40001); L("after-throw"); }
    catch (...) { L("caught"); }
    L("after-catch");
    return 5;
}
// catch (...) and rethrow
RexxRoutine0(int, Rethrow) {
    try { Noisy n = {"inner"}; context->ThrowException0(40001); }
    catch (...) { L("caught-rethrow"); throw; }
    return 5;
}
// A foreign C++ exception escaping the entry point
RexxRoutine0(int, Foreign) { Noisy n = {"foreign"}; throw 42; return 1; }
// A Throw through a C frame with unwind tables
RexxRoutine0(int, CThrowUw) { Noisy n = {"cuw"}; c_call_uw((T0)context->functions->ThrowException0, context, 40001); L("after-c"); return 1; }
// ... and through one without
RexxRoutine0(int, CThrowNoUw) { Noisy n = {"cnouw"}; c_call_nouw((T0)context->functions->ThrowException0, context, 40001); L("after-c"); return 1; }
// Plain routine used after a Throw
RexxRoutine1(RexxStringObject, Again, CSTRING, s) {
    char b[128]; snprintf(b, sizeof b, "again %s digits=%u", s, (unsigned)context->GetContextDigits());
    return context->String(b);
}
// A method using its context, and a send, after a Throw
RexxMethod1(RexxObjectPtr, MAgain, RexxObjectPtr, o) {
    context->SetObjectVariable("V", context->String("set"));
    RexxObjectPtr r = context->SendMessage0(o, "RUN");
    return r ? r : context->String("null");
}
// stash this call's context, send RUN (whose native Throws through the stash)
RexxRoutine1(RexxObjectPtr, Stash, RexxObjectPtr, o) {
    stashed = context; Noisy n = {"stasher"};
    RexxObjectPtr r = context->SendMessage0(o, "RUN");
    L(context->CheckCondition() ? "stasher held" : "stasher clear");
    stashed = NULL;
    return r ? r : context->String("null");
}
RexxRoutine0(int, ThrowStashed) { Noisy n = {"inner-stashed"}; stashed->ThrowException0(40001); return 1; }
// Load a second library whose loader throws through this call's context
RexxRoutine0(int, LoadThrow) {
    char b[64]; snprintf(b, sizeof b, "%p", (void *)context); setenv("RR_CTX", b, 1);
    Noisy n = {"loadthrow"};
    logical_t p = context->LoadLibrary("rr2");
    L(p ? "loaded" : "load-null");
    return 3;
}
RexxRoutine1(size_t, CStr, CSTRING, s) { return strlen(s); }
RexxRoutine1(RexxStringObject, SData2, RexxObjectPtr, o) {
    const char *a = context->StringData((RexxStringObject)o);
    char first[64]; snprintf(first, sizeof first, "%s", a ? a : "(null)");
    context->SendMessage1(o, "APPEND", context->String("def"));
    const char *b = context->StringData((RexxStringObject)o);
    char buf[200]; snprintf(buf, sizeof buf, "first=%s second=%s same=%d", first, b ? b : "(null)", a == b);
    return context->String(buf);
}
RexxRoutine1(RexxStringObject, OTS2, RexxObjectPtr, o) {
    const char *a = context->ObjectToStringValue(o);
    char first[64]; snprintf(first, sizeof first, "%s", a ? a : "(null)");
    context->SendMessage1(o, "APPEND", context->String("def"));
    const char *b = context->ObjectToStringValue(o);
    char buf[200]; snprintf(buf, sizeof buf, "first=%s second=%s", first, b ? b : "(null)");
    return context->String(buf);
}
static RexxObjectPtr keptObj; static const char *keptPtr;
RexxRoutine1(int, KeepOTS, RexxObjectPtr, o) { keptObj = context->RequestGlobalReference(o); keptPtr = context->ObjectToStringValue(o); return 0; }
RexxRoutine0(RexxStringObject, ReadOTS) { return context->String(keptPtr); }
RexxMethod0(RexxStringObject, Align) {
    char b[400]; int n = 0; size_t sizes[] = {0, 1, 3, 15, 16, 17, 100, 4096};
    for (size_t i = 0; i < sizeof sizes / sizeof sizes[0]; i++) {
        void *p = context->AllocateObjectMemory(sizes[i]);
        void *q = context->ReallocateObjectMemory(p, sizes[i] * 3 + 5);
        void *r = context->ReallocateObjectMemory(q, 2);
        n += snprintf(b + n, sizeof b - n, "%zu:%d/%d/%d ", sizes[i], (int)((uintptr_t)p % 16), (int)((uintptr_t)q % 16), (int)((uintptr_t)r % 16));
    }
    for (size_t sz = 0; sz < 40; sz += 13) {
        RexxBufferObject bo = context->NewBuffer(sz);
        n += snprintf(b + n, sizeof b - n, "buf%zu:%d ", sz, (int)((uintptr_t)context->BufferData(bo) % 16));
    }
    return context->String(b);
}
static void RexxEntry unloader(RexxThreadContext *) { fprintf(stderr, "rr unloader ran\n"); }
RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(Dtors, Dtors), REXX_TYPED_ROUTINE(CatchAll, CatchAll), REXX_TYPED_ROUTINE(Rethrow, Rethrow),
    REXX_TYPED_ROUTINE(Foreign, Foreign), REXX_TYPED_ROUTINE(CThrowUw, CThrowUw), REXX_TYPED_ROUTINE(CThrowNoUw, CThrowNoUw),
    REXX_TYPED_ROUTINE(Again, Again), REXX_TYPED_ROUTINE(Stash, Stash), REXX_TYPED_ROUTINE(ThrowStashed, ThrowStashed),
    REXX_TYPED_ROUTINE(LoadThrow, LoadThrow), REXX_TYPED_ROUTINE(CStr, CStr), REXX_TYPED_ROUTINE(SData2, SData2), REXX_TYPED_ROUTINE(OTS2, OTS2), REXX_TYPED_ROUTINE(KeepOTS, KeepOTS), REXX_TYPED_ROUTINE(ReadOTS, ReadOTS), REXX_LAST_ROUTINE() };
RexxMethodEntry methods[] = { REXX_METHOD(MAgain, MAgain), REXX_METHOD(Align, Align), REXX_LAST_METHOD() };
RexxPackageEntry rr_package_entry = { STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "rr", "1.0", NULL, unloader, routines, methods };
OOREXX_GET_PACKAGE(rr);
#else
static void RexxEntry loader(RexxThreadContext *) {
    const char *e = getenv("RR_CTX"); if (!e) return;
    RexxCallContext *c = (RexxCallContext *)(uintptr_t)strtoull(e, NULL, 16);
    Noisy n = {"loader"};
    fprintf(stderr, "loader throwing\n");
    c->ThrowException0(40001);
    fprintf(stderr, "loader after throw\n");
}
RexxPackageEntry rr2_package_entry = { STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "rr2", "1.0", loader, NULL, NULL, NULL };
OOREXX_GET_PACKAGE(rr2);
#endif

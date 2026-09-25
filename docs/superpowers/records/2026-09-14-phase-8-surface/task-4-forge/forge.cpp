#include "oorexxapi.h"
#include <stdio.h>

#ifndef TAG
#define TAG "forge"
#endif
#ifndef REQUIRED
#define REQUIRED REXX_INTERPRETER_4_0_0
#endif

static RexxThreadContext *stash = NULL;

static void say(const char *what)
{
    printf("%s %s\n", TAG, what);
    fflush(stdout);
}

static void RexxEntry loader(RexxThreadContext *c)
{
    say("loader");
    if (stash == NULL) stash = c;
    RexxObjectPtr n = c->WholeNumberToObject(7);
    printf("%s loader answer null=%d\n", TAG, n == NULL);
    fflush(stdout);
#ifdef LOADER_REFUSES
    c->NewStringFromAsciiz("x");
    say("loader after refusal");
#endif
#ifdef LOADER_RAISES
    c->RaiseException0(Rexx_Error_Incorrect_call);
    say("loader after raise");
#endif
}

static void RexxEntry unloader(RexxThreadContext *c)
{
    say("unloader");
    RexxObjectPtr n = c->WholeNumberToObject(8);
    printf("%s unloader answer null=%d same=%d\n", TAG, n == NULL, c == stash);
    fflush(stdout);
#ifdef UNLOADER_REFUSES
    c->NewStringFromAsciiz("x");
    say("unloader after refusal");
#endif
#ifdef UNLOADER_RAISES
    c->RaiseException0(Rexx_Error_Incorrect_call);
    say("unloader after raise");
#endif
}

__attribute__((destructor)) static void gone(void)
{
    say("destructor");
#ifdef DESTRUCTOR_CALLS
    if (stash != NULL)
    {
        RexxObjectPtr n = stash->WholeNumberToObject(9);
        printf("%s destructor answer null=%d\n", TAG, n == NULL);
        fflush(stdout);
    }
#endif
}

RexxRoutine0(int, Stash)
{
    stash = context->threadContext;
    return 1;
}

RexxRoutine0(RexxObjectPtr, UseStash)
{
    return stash->WholeNumberToObject(42);
}

RexxRoutine0(int, Same)
{
    return context->threadContext == stash;
}

RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(Stash, Stash),
    REXX_TYPED_ROUTINE(UseStash, UseStash),
    REXX_TYPED_ROUTINE(Same, Same),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry forge_package_entry = {
    STANDARD_PACKAGE_HEADER
    REQUIRED,
    TAG,
    "1.0",
#ifdef NO_HOOKS
    NULL, NULL,
#else
    loader,
    unloader,
#endif
    routines,
    NULL
};

OOREXX_GET_PACKAGE(forge);

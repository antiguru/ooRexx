/* A C frame between the extension and a Throw slot. */
#include <stddef.h>
static volatile int c_after = 0;
void FN(void (*f)(void *, size_t), void *ctx, size_t n) { f(ctx, n); c_after++; }

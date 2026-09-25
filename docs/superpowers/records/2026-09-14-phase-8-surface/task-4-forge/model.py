M = (1 << 64) - 1
def h(s):
    x = 0
    for b in s.encode():
        x = (31 * x + (b if b < 128 else b - 256)) & M
    return x
def bucket_size(c):
    if c < 17: return 17
    return c + 1 if c % 2 == 0 else c
class T:
    def __init__(self, cap=17):
        self.b = bucket_size(cap); n = self.b * 2
        self.idx = [None]*n; self.nxt = [None]*n
        for i in range(n):
            self.nxt[i] = None if (i < self.b or i + 1 == n) else i + 1
        self.free = self.b
    def full(self): return self.free is None
    def put(self, k):
        if self.full(): self.expand()
        self._put(k)
    def _put(self, k):
        p = h(k) % self.b
        if self.idx[p] is None: self.idx[p] = k; self.nxt[p] = None; return
        prev = None
        while p is not None:
            if self.idx[p] == k: return
            prev = p; p = self.nxt[p]
        e = self.free; self.free = self.nxt[e]
        self.idx[e] = k; self.nxt[prev] = e; self.nxt[e] = None
    def expand(self):
        old = list(self.order())
        new = T(2 * self.b * 2)
        for k in old: new._put(k)
        self.__dict__ = new.__dict__
    def remove(self, k):
        p = h(k) % self.b; prev = None
        while p is not None and self.idx[p] is not None:
            if self.idx[p] == k: break
            prev = p; p = self.nxt[p]
        else: return
        if prev is None:
            n = self.nxt[p]
            if n is None: self.idx[p] = None; return
            self.idx[p] = self.idx[n]; self.nxt[p] = self.nxt[n]
            self.idx[n] = None; self.nxt[n] = self.free; self.free = n
        else:
            self.nxt[prev] = self.nxt[p]
            self.idx[p] = None; self.nxt[p] = self.free; self.free = p
    def order(self):
        for i in range(self.b):
            p = i
            while p is not None and self.idx[p] is not None:
                yield self.idx[p]; p = self.nxt[p]
def unload_order(ops):
    t = T(); t.put('REXX'); t.put('REXXUTIL')
    for op in ops:
        if op.startswith('-'): t.remove(op[1:])
        else: t.put(op)
    return [k for k in t.order() if k not in ('REXX','REXXUTIL')]
if __name__ == '__main__':
    import sys
    print(' '.join(unload_order(sys.argv[1:])))

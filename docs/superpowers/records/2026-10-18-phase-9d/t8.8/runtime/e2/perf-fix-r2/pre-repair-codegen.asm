
/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx:     file format elf64-x86-64


Disassembly of section .text:

000000000026c340 <<rexx_exec::Interp>::begin_invoke>:
<rexx_exec::Interp>::begin_invoke:
  26c340:	push   %rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2720
  26c341:	push   %r15
  26c343:	push   %r14
  26c345:	push   %r13
  26c347:	push   %r12
  26c349:	push   %rbx
  26c34a:	sub    $0x508,%rsp
  26c351:	mov    %r9,0x20(%rsp)
  26c356:	mov    %ecx,%r15d
  26c359:	mov    %rdx,0x30(%rsp)
  26c35e:	mov    %rsi,%r14
  26c361:	mov    %rdi,%rbx
<core::option::Option<rexx_exec::dispatch::ObjectModel>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:634
  26c364:	cmpq   $0xffffffffffffffff,0x1170(%rsi)
<rexx_exec::Interp>::object_model:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:1832
  26c36c:	je     26dee9 <<rexx_exec::Interp>::begin_invoke+0x1ba9>
<rexx_exec::Interp>::invocable:
  26c372:	mov    %r8,0x28(%rsp)
  26c377:	mov    0x548(%rsp),%r12
  26c37f:	mov    0x540(%rsp),%r13
<rexx_exec::method_rows::MethodRows<rexx_exec::dispatch::NativeEntry>>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/method_rows.rs:31
  26c387:	mov    %r15d,%ebp
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::dispatch::NativeEntry>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26c38a:	cmp    %rbp,0x12d0(%r14)
  26c391:	mov    %r12,0x78(%rsp)
  26c396:	mov    %r15d,0x80(%rsp)
  26c39e:	jbe    26c3d9 <<rexx_exec::Interp>::begin_invoke+0x99>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_exec::dispatch::NativeEntry>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26c3a0:	mov    0x12c8(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::dispatch::NativeEntry>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26c3a7:	mov    %rbp,%rcx
  26c3aa:	shl    $0x5,%rcx
<core::option::Option<rexx_exec::dispatch::NativeEntry>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26c3ae:	mov    (%rax,%rcx,1),%rsi
  26c3b2:	cmp    $0x2,%rsi
  26c3b6:	je     26c481 <<rexx_exec::Interp>::begin_invoke+0x141>
<rexx_exec::Interp>::invocable:
  26c3bc:	add    %rcx,%rax
  26c3bf:	mov    %rsi,%rcx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2676
  26c3c2:	mov    0x8(%rax),%r13
  26c3c6:	mov    0x10(%rax),%r12
<core::result::Result<rexx_exec::dispatch::Invocable, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26c3ca:	cmp    $0xffffffffffffffff,%rcx
  26c3ce:	jne    26c6dc <<rexx_exec::Interp>::begin_invoke+0x39c>
  26c3d4:	jmp    26cc3f <<rexx_exec::Interp>::begin_invoke+0x8ff>
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry, rustc_hash::FxBuildHasher>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/map.rs:825
  26c3d9:	cmpq   $0x0,0x12f0(%r14)
<rexx_exec::method_rows::MethodRows<rexx_exec::dispatch::NativeEntry>>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/method_rows.rs:33
  26c3e1:	je     26c481 <<rexx_exec::Interp>::begin_invoke+0x141>
<std::collections::hash::map::HashMap<rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/map.rs:1039
  26c3e7:	movabs $0xf1357aea2e62a9c5,%rax
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26c3f1:	imul   %rbp,%rax
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26c3f5:	rol    $0x1a,%rax
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26c3f9:	mov    %rax,%rsi
  26c3fc:	shr    $0x39,%rsi
  26c400:	mov    0x12d8(%r14),%rcx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26c407:	mov    0x12e0(%r14),%rdx
  26c40e:	movd   %esi,%xmm0
  26c412:	punpcklbw %xmm0,%xmm0
  26c416:	pshuflw $0x0,%xmm0,%xmm0
  26c41b:	pshufd $0x44,%xmm0,%xmm0
  26c420:	xor    %esi,%esi
  26c422:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26c426:	and    %rdx,%rax
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26c429:	movdqu (%rcx,%rax,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c42e:	movdqa %xmm2,%xmm3
  26c432:	pcmpeqb %xmm0,%xmm3
  26c436:	pmovmskb %xmm3,%edi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c43a:	test   %edi,%edi
  26c43c:	je     26c468 <<rexx_exec::Interp>::begin_invoke+0x128>
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26c43e:	tzcnt  %edi,%r8d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26c443:	add    %rax,%r8
  26c446:	and    %rdx,%r8
<*mut (rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26c449:	neg    %r8
  26c44c:	lea    (%r8,%r8,4),%r8
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26c450:	cmp    -0x28(%rcx,%r8,8),%r15d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c455:	je     26c6be <<rexx_exec::Interp>::begin_invoke+0x37e>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26c45b:	lea    -0x1(%rdi),%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c45f:	and    %di,%r8w
  26c463:	mov    %r8d,%edi
  26c466:	jne    26c43e <<rexx_exec::Interp>::begin_invoke+0xfe>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c468:	pcmpeqb %xmm1,%xmm2
  26c46c:	pmovmskb %xmm2,%edi
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c470:	test   %edi,%edi
  26c472:	jne    26c481 <<rexx_exec::Interp>::begin_invoke+0x141>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26c474:	add    %rsi,%rax
  26c477:	add    $0x10,%rax
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26c47b:	add    $0x10,%rsi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26c47f:	jmp    26c426 <<rexx_exec::Interp>::begin_invoke+0xe6>
<rexx_exec::Interp>::invocable:
  26c481:	movabs $0xf1357aea2e62a9c5,%rax
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::InstalledMethodBody)>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1337
  26c48b:	cmpq   $0x0,0x1bd8(%r14)
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_exec::InstalledMethodBody, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/rust/deps/hashbrown-0.17.1/src/map.rs:1284
  26c493:	je     26c530 <<rexx_exec::Interp>::begin_invoke+0x1f0>
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26c499:	mov    %rbp,%rcx
  26c49c:	imul   %rax,%rcx
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26c4a0:	rol    $0x1a,%rcx
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26c4a4:	mov    %rcx,%rdi
  26c4a7:	shr    $0x39,%rdi
  26c4ab:	mov    0x1bc0(%r14),%rdx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26c4b2:	mov    0x1bc8(%r14),%rsi
  26c4b9:	movd   %edi,%xmm0
  26c4bd:	punpcklbw %xmm0,%xmm0
  26c4c1:	pshuflw $0x0,%xmm0,%xmm0
  26c4c6:	pshufd $0x44,%xmm0,%xmm0
  26c4cb:	xor    %edi,%edi
  26c4cd:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26c4d1:	and    %rsi,%rcx
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26c4d4:	movdqu (%rdx,%rcx,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c4d9:	movdqa %xmm2,%xmm3
  26c4dd:	pcmpeqb %xmm0,%xmm3
  26c4e1:	pmovmskb %xmm3,%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c4e6:	test   %r8d,%r8d
  26c4e9:	je     26c515 <<rexx_exec::Interp>::begin_invoke+0x1d5>
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26c4eb:	tzcnt  %r8d,%r9d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26c4f0:	add    %rcx,%r9
  26c4f3:	and    %rsi,%r9
<*mut (rexx_core::body::MethodId, rexx_exec::InstalledMethodBody)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26c4f6:	neg    %r9
  26c4f9:	lea    (%r9,%r9,2),%r9
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26c4fd:	cmp    -0x18(%rdx,%r9,8),%r15d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c502:	je     26c5d7 <<rexx_exec::Interp>::begin_invoke+0x297>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26c508:	lea    -0x1(%r8),%r9d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c50c:	and    %r8w,%r9w
  26c510:	mov    %r9d,%r8d
  26c513:	jne    26c4eb <<rexx_exec::Interp>::begin_invoke+0x1ab>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c515:	pcmpeqb %xmm1,%xmm2
  26c519:	pmovmskb %xmm2,%r8d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c51e:	test   %r8d,%r8d
  26c521:	jne    26c530 <<rexx_exec::Interp>::begin_invoke+0x1f0>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26c523:	add    %rdi,%rcx
  26c526:	add    $0x10,%rcx
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26c52a:	add    $0x10,%rdi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26c52e:	jmp    26c4d1 <<rexx_exec::Interp>::begin_invoke+0x191>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::GeneratedMethod)>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1337
  26c530:	cmpq   $0x0,0x1cd8(%r14)
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_exec::GeneratedMethod, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/rust/deps/hashbrown-0.17.1/src/map.rs:1284
  26c538:	je     26c5f2 <<rexx_exec::Interp>::begin_invoke+0x2b2>
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26c53e:	mov    %rbp,%rcx
  26c541:	imul   %rax,%rcx
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26c545:	rol    $0x1a,%rcx
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26c549:	mov    %rcx,%rdi
  26c54c:	shr    $0x39,%rdi
  26c550:	mov    0x1cc0(%r14),%rdx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26c557:	mov    0x1cc8(%r14),%rsi
  26c55e:	movd   %edi,%xmm0
  26c562:	punpcklbw %xmm0,%xmm0
  26c566:	pshuflw $0x0,%xmm0,%xmm0
  26c56b:	pshufd $0x44,%xmm0,%xmm0
  26c570:	xor    %edi,%edi
  26c572:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26c576:	and    %rsi,%rcx
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26c579:	movdqu (%rdx,%rcx,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c57e:	movdqa %xmm2,%xmm3
  26c582:	pcmpeqb %xmm0,%xmm3
  26c586:	pmovmskb %xmm3,%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c58b:	test   %r8d,%r8d
  26c58e:	je     26c5bc <<rexx_exec::Interp>::begin_invoke+0x27c>
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26c590:	tzcnt  %r8d,%r10d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26c595:	add    %rcx,%r10
  26c598:	and    %rsi,%r10
<*mut (rexx_core::body::MethodId, rexx_exec::GeneratedMethod)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26c59b:	shl    $0x5,%r10
  26c59f:	mov    %rdx,%r9
  26c5a2:	sub    %r10,%r9
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26c5a5:	cmp    -0x20(%r9),%r15d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c5a9:	je     26c6ed <<rexx_exec::Interp>::begin_invoke+0x3ad>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26c5af:	lea    -0x1(%r8),%r9d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c5b3:	and    %r8w,%r9w
  26c5b7:	mov    %r9d,%r8d
  26c5ba:	jne    26c590 <<rexx_exec::Interp>::begin_invoke+0x250>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c5bc:	pcmpeqb %xmm1,%xmm2
  26c5c0:	pmovmskb %xmm2,%r8d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c5c5:	test   %r8d,%r8d
  26c5c8:	jne    26c5f2 <<rexx_exec::Interp>::begin_invoke+0x2b2>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26c5ca:	add    %rdi,%rcx
  26c5cd:	add    $0x10,%rcx
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26c5d1:	add    $0x10,%rdi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26c5d5:	jmp    26c576 <<rexx_exec::Interp>::begin_invoke+0x236>
<rexx_exec::Interp>::invocable:
  26c5d7:	mov    %rbx,0x8(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2679
  26c5dc:	lea    (%rdx,%r9,8),%rax
<core::option::Option<&rexx_exec::InstalledMethodBody>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26c5e0:	mov    -0x10(%rax),%r13
  26c5e4:	mov    -0x8(%rax),%r12
  26c5e8:	mov    $0x2,%eax
  26c5ed:	jmp    26c7bf <<rexx_exec::Interp>::begin_invoke+0x47f>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, &rexx_exec::dispatch::native::NativeExternal)>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1337
  26c5f2:	cmpq   $0x0,0x1cf8(%r14)
<hashbrown::map::HashMap<rexx_core::body::MethodId, &rexx_exec::dispatch::native::NativeExternal, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/rust/deps/hashbrown-0.17.1/src/map.rs:1284
  26c5fa:	je     26c708 <<rexx_exec::Interp>::begin_invoke+0x3c8>
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26c600:	mov    %rbp,%rcx
  26c603:	imul   %rax,%rcx
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26c607:	rol    $0x1a,%rcx
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26c60b:	mov    %rcx,%rdi
  26c60e:	shr    $0x39,%rdi
  26c612:	mov    0x1ce0(%r14),%rdx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26c619:	mov    0x1ce8(%r14),%rsi
  26c620:	movd   %edi,%xmm0
  26c624:	punpcklbw %xmm0,%xmm0
  26c628:	pshuflw $0x0,%xmm0,%xmm0
  26c62d:	pshufd $0x44,%xmm0,%xmm0
  26c632:	xor    %edi,%edi
  26c634:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26c638:	and    %rsi,%rcx
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26c63b:	movdqu (%rdx,%rcx,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c640:	movdqa %xmm2,%xmm3
  26c644:	pcmpeqb %xmm0,%xmm3
  26c648:	pmovmskb %xmm3,%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c64d:	test   %r8d,%r8d
  26c650:	je     26c6a0 <<rexx_exec::Interp>::begin_invoke+0x360>
  26c652:	mov    %rbx,0x8(%rsp)
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26c657:	tzcnt  %r8d,%r10d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26c65c:	add    %rcx,%r10
  26c65f:	and    %rsi,%r10
<*mut (rexx_core::body::MethodId, &rexx_exec::dispatch::native::NativeExternal)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26c662:	shl    $0x4,%r10
  26c666:	mov    %rdx,%r9
  26c669:	sub    %r10,%r9
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26c66c:	cmp    -0x10(%r9),%r15d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c670:	je     26c7b6 <<rexx_exec::Interp>::begin_invoke+0x476>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26c676:	lea    -0x1(%r8),%r9d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c67a:	and    %r8w,%r9w
  26c67e:	mov    %r9d,%r8d
  26c681:	mov    0x8(%rsp),%rbx
  26c686:	mov    0x548(%rsp),%r12
  26c68e:	mov    0x80(%rsp),%r15d
  26c696:	mov    0x540(%rsp),%r13
  26c69e:	jne    26c657 <<rexx_exec::Interp>::begin_invoke+0x317>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c6a0:	pcmpeqb %xmm1,%xmm2
  26c6a4:	pmovmskb %xmm2,%r8d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c6a9:	test   %r8d,%r8d
  26c6ac:	jne    26c708 <<rexx_exec::Interp>::begin_invoke+0x3c8>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26c6ae:	add    %rdi,%rcx
  26c6b1:	add    $0x10,%rcx
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26c6b5:	add    $0x10,%rdi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26c6b9:	jmp    26c638 <<rexx_exec::Interp>::begin_invoke+0x2f8>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry)>>::get::<hashbrown::map::equivalent_key<rexx_core::body::MethodId, rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry>::{closure#0}>:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1217
  26c6be:	lea    (%rcx,%r8,8),%rax
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2676
  26c6c2:	mov    -0x20(%rax),%rcx
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry)>>::get::<hashbrown::map::equivalent_key<rexx_core::body::MethodId, rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry>::{closure#0}>:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1217
  26c6c6:	add    $0xffffffffffffffe0,%rax
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2676
  26c6ca:	mov    0x8(%rax),%r13
  26c6ce:	mov    0x10(%rax),%r12
<core::result::Result<rexx_exec::dispatch::Invocable, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26c6d2:	cmp    $0xffffffffffffffff,%rcx
  26c6d6:	je     26cc3f <<rexx_exec::Interp>::begin_invoke+0x8ff>
<rexx_exec::Interp>::begin_invoke:
  26c6dc:	mov    %rbx,0x8(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2728
  26c6e1:	mov    0x18(%rax),%rbx
  26c6e5:	mov    %rcx,%rax
<core::result::Result<rexx_exec::dispatch::Invocable, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26c6e8:	jmp    26c7bf <<rexx_exec::Interp>::begin_invoke+0x47f>
<rexx_exec::Interp>::begin_invoke:
  26c6ed:	mov    %rbx,0x8(%rsp)
<core::option::Option<&rexx_exec::GeneratedMethod>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26c6f2:	mov    -0x18(%r9),%r13
  26c6f6:	mov    -0x10(%r9),%r12
  26c6fa:	mov    -0x8(%r9),%rbx
  26c6fe:	mov    $0x3,%eax
<rexx_exec::Interp>::invocable:
  26c703:	jmp    26c7bf <<rexx_exec::Interp>::begin_invoke+0x47f>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::LibraryBinding)>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1337
  26c708:	cmpq   $0x0,0x1de0(%r14)
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_exec::LibraryBinding, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/rust/deps/hashbrown-0.17.1/src/map.rs:1284
  26c710:	je     26cbb1 <<rexx_exec::Interp>::begin_invoke+0x871>
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26c716:	imul   %rbp,%rax
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26c71a:	rol    $0x1a,%rax
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26c71e:	mov    %rax,%rsi
  26c721:	shr    $0x39,%rsi
  26c725:	mov    0x1dc8(%r14),%rcx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26c72c:	mov    0x1dd0(%r14),%rdx
  26c733:	movd   %esi,%xmm0
  26c737:	punpcklbw %xmm0,%xmm0
  26c73b:	pshuflw $0x0,%xmm0,%xmm0
  26c740:	pshufd $0x44,%xmm0,%xmm0
  26c745:	xor    %esi,%esi
  26c747:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26c74b:	and    %rdx,%rax
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26c74e:	movdqu (%rcx,%rax,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c753:	movdqa %xmm2,%xmm3
  26c757:	pcmpeqb %xmm0,%xmm3
  26c75b:	pmovmskb %xmm3,%edi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c75f:	test   %edi,%edi
  26c761:	je     26c799 <<rexx_exec::Interp>::begin_invoke+0x459>
  26c763:	mov    %rbx,0x8(%rsp)
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26c768:	tzcnt  %edi,%r9d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26c76d:	add    %rax,%r9
  26c770:	and    %rdx,%r9
<*mut (rexx_core::body::MethodId, rexx_exec::LibraryBinding)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26c773:	shl    $0x5,%r9
  26c777:	mov    %rcx,%r8
  26c77a:	sub    %r9,%r8
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26c77d:	cmp    -0x20(%r8),%r15d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c781:	je     26cc53 <<rexx_exec::Interp>::begin_invoke+0x913>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26c787:	lea    -0x1(%rdi),%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c78b:	and    %di,%r8w
  26c78f:	mov    %r8d,%edi
  26c792:	mov    0x8(%rsp),%rbx
  26c797:	jne    26c768 <<rexx_exec::Interp>::begin_invoke+0x428>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c799:	pcmpeqb %xmm1,%xmm2
  26c79d:	pmovmskb %xmm2,%edi
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c7a1:	test   %edi,%edi
  26c7a3:	jne    26cbb1 <<rexx_exec::Interp>::begin_invoke+0x871>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26c7a9:	add    %rsi,%rax
  26c7ac:	add    $0x10,%rax
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26c7b0:	add    $0x10,%rsi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26c7b4:	jmp    26c74b <<rexx_exec::Interp>::begin_invoke+0x40b>
<core::option::Option<&&rexx_exec::dispatch::native::NativeExternal>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26c7b6:	mov    -0x8(%r9),%r13
  26c7ba:	mov    $0x4,%eax
<rexx_exec::Interp>::begin_invoke:
  26c7bf:	mov    0x558(%rsp),%r10
  26c7c7:	mov    %rax,0x40(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2728
  26c7cc:	mov    %rax,0x2e0(%rsp)
  26c7d4:	mov    %r13,0x2e8(%rsp)
  26c7dc:	mov    %r12,0x2f0(%rsp)
  26c7e4:	mov    %rbx,0x2f8(%rsp)
rexx_exec::dispatch::seam::clear:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:66
  26c7ec:	cmpb   $0x1,0x1ff4(%r14)
  26c7f4:	jne    26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:67
  26c7f6:	cmpb   $0x0,0x1ff5(%r14)
core::mem::replace::<bool>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26c7fe:	movb   $0x0,0x1ff5(%r14)
rexx_exec::dispatch::seam::clear:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:67
  26c806:	je     26ca0a <<rexx_exec::Interp>::begin_invoke+0x6ca>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2735
  26c80c:	cmpq   $0x2,0x40(%rsp)
  26c812:	jne    26c8d7 <<rexx_exec::Interp>::begin_invoke+0x597>
<rexx_exec::plan::Programs>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:135
  26c818:	mov    %r13d,%ebx
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::ProgramSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26c81b:	cmp    0xc58(%r14),%rbx
  26c822:	jae    26c9ae <<rexx_exec::Interp>::begin_invoke+0x66e>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:33
  26c828:	mov    0xc50(%r14),%rax
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::ProgramSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26c82f:	mov    %rbx,%r15
  26c832:	shl    $0x4,%r15
<rexx_exec::plan::ProgramId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:32
  26c836:	cmp    %r13,(%rax,%r15,1)
<rexx_exec::plan::Programs>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:136
  26c83a:	jne    26c9ae <<rexx_exec::Interp>::begin_invoke+0x66e>
  26c840:	add    %r15,%rax
<core::option::Option<alloc::rc::Rc<rexx_parse::Program>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26c843:	mov    0x8(%rax),%rcx
  26c847:	test   %rcx,%rcx
  26c84a:	je     26c9ae <<rexx_exec::Interp>::begin_invoke+0x66e>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26c850:	incq   (%rcx)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26c853:	je     26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<alloc::rc::Rc<rexx_parse::Program> as core::clone::Clone>::clone:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2527
  26c859:	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[rexx_parse::ast::Directive]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26c85d:	cmp    0xa0(%rax),%r12
  26c864:	mov    %rax,0x18(%rsp)
  26c869:	jae    26dff2 <<rexx_exec::Interp>::begin_invoke+0x1cb2>
<rexx_exec::Interp>::begin_method:
  26c86f:	movabs $0x8000000000000000,%rsi
<alloc::raw_vec::RawVecInner>::non_null::<rexx_parse::ast::Directive>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26c879:	mov    0x98(%rax),%rcx
<usize as core::slice::index::SliceIndex<[rexx_parse::ast::Directive]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26c880:	lea    (%r12,%r12,4),%rdx
  26c884:	lea    (%rcx,%rdx,8),%rax
rexx_exec::activation::body_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1127
  26c888:	mov    (%rcx,%rdx,8),%rcx
  26c88c:	mov    %rcx,%rdx
  26c88f:	xor    %rsi,%rdx
  26c892:	test   %rcx,%rcx
  26c895:	mov    $0x5,%ecx
  26c89a:	cmovs  %rdx,%rcx
  26c89e:	cmp    $0x8,%rcx
  26c8a2:	je     26cfb9 <<rexx_exec::Interp>::begin_invoke+0xc79>
  26c8a8:	cmp    $0x4,%rcx
  26c8ac:	je     26c8b8 <<rexx_exec::Interp>::begin_invoke+0x578>
  26c8ae:	cmp    $0x1,%rcx
  26c8b2:	jne    26d15e <<rexx_exec::Interp>::begin_invoke+0xe1e>
  26c8b8:	mov    0x8(%rax),%rsi
<core::option::Option<rexx_parse::ast::CodeBody>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26c8bc:	cmpq   $0xffffffffffffffff,(%rsi)
  26c8c0:	je     26d15e <<rexx_exec::Interp>::begin_invoke+0xe1e>
<rexx_parse::ast::GuardOption as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/ast.rs:1102
  26c8c6:	cmpb   $0x2,0x74(%rsi)
  26c8ca:	setne  %al
  26c8cd:	mov    %rax,0x50(%rsp)
rexx_exec::guards::directive_guarded:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/guards.rs:384
  26c8d2:	jmp    26cfd0 <<rexx_exec::Interp>::begin_invoke+0xc90>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2757
  26c8d7:	jae    26ca43 <<rexx_exec::Interp>::begin_invoke+0x703>
  26c8dd:	cmp    $0x2,%r12
  26c8e1:	je     26ca43 <<rexx_exec::Interp>::begin_invoke+0x703>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2760
  26c8e7:	testb  $0x1,0x40(%rsp)
  26c8ec:	jne    26cb40 <<rexx_exec::Interp>::begin_invoke+0x800>
  26c8f2:	cmp    %r13,%r10
  26c8f5:	jbe    26cb40 <<rexx_exec::Interp>::begin_invoke+0x800>
  26c8fb:	mov    %r10,%rbp
  26c8fe:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2762
  26c906:	mov    %r13,%rsi
  26c909:	call   26efe0 <<rexx_exec::error::Raised>::too_many_method_arguments>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26c90e:	mov    $0x70,%edi
  26c913:	call   *0x26b69f(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26c919:	test   %rax,%rax
  26c91c:	je     26dfbd <<rexx_exec::Interp>::begin_invoke+0x1c7d>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26c922:	movups 0x160(%rsp),%xmm0
  26c92a:	movups %xmm0,0x60(%rax)
  26c92e:	movups 0x150(%rsp),%xmm0
  26c936:	movups %xmm0,0x50(%rax)
  26c93a:	movups 0x140(%rsp),%xmm0
  26c942:	movups %xmm0,0x40(%rax)
  26c946:	movdqu 0x100(%rsp),%xmm0
  26c94f:	movdqu 0x110(%rsp),%xmm1
  26c958:	movdqu 0x120(%rsp),%xmm2
  26c961:	movdqu 0x130(%rsp),%xmm3
  26c96a:	movdqu %xmm3,0x30(%rax)
  26c96f:	movdqu %xmm2,0x20(%rax)
  26c974:	movdqu %xmm1,0x10(%rax)
  26c979:	movdqu %xmm0,(%rax)
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2762
  26c97d:	movq   $0x3,0x68(%rsp)
  26c986:	mov    %rax,0x70(%rsp)
  26c98b:	movq   $0x2,0x60(%rsp)
  26c994:	mov    0x8(%rsp),%r15
  26c999:	mov    0x548(%rsp),%r12
  26c9a1:	mov    0x540(%rsp),%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2766
  26c9a9:	jmp    26cead <<rexx_exec::Interp>::begin_invoke+0xb6d>
  26c9ae:	lea    0x100(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:34
  26c9b6:	call   1cede0 <<rexx_exec::Loud>::missing_body>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26c9bb:	mov    $0x18,%edi
  26c9c0:	call   *0x26b5f2(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26c9c6:	test   %rax,%rax
  26c9c9:	je     26df1d <<rexx_exec::Interp>::begin_invoke+0x1bdd>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26c9cf:	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26c9d2:	mov    0x110(%rsp),%rax
  26c9da:	mov    %rax,0x10(%rbx)
  26c9de:	movdqu 0x100(%rsp),%xmm0
  26c9e7:	movdqu %xmm0,(%rbx)
  26c9eb:	mov    $0x2,%r14d
<rexx_exec::Interp>::begin_invoke:
  26c9f1:	mov    0x8(%rsp),%rax
<core::result::Result<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  26c9f6:	mov    %r14,0x8(%rax)
  26c9fa:	mov    %rbx,0x10(%rax)
  26c9fe:	movq   $0x1,(%rax)
<rexx_exec::Interp>::begin_invoke:
  26ca05:	jmp    26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::dispatch::AccessScope>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26ca0a:	cmp    %rbp,0x1090(%r14)
  26ca11:	jbe    26ca8a <<rexx_exec::Interp>::begin_invoke+0x74a>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_exec::dispatch::AccessScope>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26ca13:	mov    0x1088(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::dispatch::AccessScope>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26ca1a:	lea    0x0(,%rbp,2),%rcx
  26ca22:	add    %rbp,%rcx
<core::option::Option<rexx_exec::dispatch::AccessScope>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26ca25:	cmpq   $0x2,(%rax,%rcx,8)
  26ca2a:	je     26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
rexx_exec::dispatch::seam::clear:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:68
  26ca30:	lea    (%rax,%rcx,8),%rax
  26ca34:	testb  $0x1,0x10(%rax)
  26ca38:	je     26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
  26ca3e:	jmp    26cc88 <<rexx_exec::Interp>::begin_invoke+0x948>
<rexx_exec::Interp>::begin_invoke:
  26ca43:	lea    0x2e0(%rsp),%rdx
  26ca4b:	mov    0x8(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2804
  26ca50:	mov    %r14,%rsi
  26ca53:	mov    0x30(%rsp),%rcx
  26ca58:	mov    0x80(%rsp),%r8d
  26ca60:	mov    0x20(%rsp),%r9
  26ca65:	push   %r10
  26ca67:	push   0x558(%rsp)
  26ca6e:	push   0x558(%rsp)
  26ca75:	push   0x558(%rsp)
  26ca7c:	call   26ebb0 <<rexx_exec::Interp>::begin_invoke_other>
  26ca81:	add    $0x20,%rsp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2808
  26ca85:	jmp    26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_exec::dispatch::AccessScope, rustc_hash::FxBuildHasher>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/map.rs:825
  26ca8a:	cmpq   $0x0,0x10b0(%r14)
<rexx_exec::method_rows::MethodRows<rexx_exec::dispatch::AccessScope>>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/method_rows.rs:33
  26ca92:	je     26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
<std::collections::hash::map::HashMap<rexx_core::body::MethodId, rexx_exec::dispatch::AccessScope, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/map.rs:1039
  26ca98:	movabs $0xf1357aea2e62a9c5,%rax
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26caa2:	imul   %rbp,%rax
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26caa6:	rol    $0x1a,%rax
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26caaa:	mov    %rax,%rsi
  26caad:	shr    $0x39,%rsi
  26cab1:	mov    0x1098(%r14),%rcx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26cab8:	mov    0x10a0(%r14),%rdx
  26cabf:	movd   %esi,%xmm0
  26cac3:	punpcklbw %xmm0,%xmm0
  26cac7:	pshuflw $0x0,%xmm0,%xmm0
  26cacc:	pshufd $0x44,%xmm0,%xmm0
  26cad1:	xor    %esi,%esi
  26cad3:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26cad7:	and    %rdx,%rax
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26cada:	movdqu (%rcx,%rax,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26cadf:	movdqa %xmm2,%xmm3
  26cae3:	pcmpeqb %xmm0,%xmm3
  26cae7:	pmovmskb %xmm3,%edi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26caeb:	test   %edi,%edi
  26caed:	je     26cb23 <<rexx_exec::Interp>::begin_invoke+0x7e3>
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26caef:	tzcnt  %edi,%r9d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26caf4:	add    %rax,%r9
  26caf7:	and    %rdx,%r9
<*mut (rexx_core::body::MethodId, rexx_exec::dispatch::AccessScope)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26cafa:	shl    $0x5,%r9
  26cafe:	mov    %rcx,%r8
  26cb01:	sub    %r9,%r8
  26cb04:	mov    0x80(%rsp),%r9d
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26cb0c:	cmp    -0x20(%r8),%r9d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26cb10:	je     26cc72 <<rexx_exec::Interp>::begin_invoke+0x932>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26cb16:	lea    -0x1(%rdi),%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26cb1a:	and    %di,%r8w
  26cb1e:	mov    %r8d,%edi
  26cb21:	jne    26caef <<rexx_exec::Interp>::begin_invoke+0x7af>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26cb23:	pcmpeqb %xmm1,%xmm2
  26cb27:	pmovmskb %xmm2,%edi
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26cb2b:	test   %edi,%edi
  26cb2d:	jne    26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26cb33:	add    %rsi,%rax
  26cb36:	add    $0x10,%rax
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26cb3a:	add    $0x10,%rsi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26cb3e:	jmp    26cad7 <<rexx_exec::Interp>::begin_invoke+0x797>
<rexx_exec::Interp>::begin_invoke:
  26cb40:	mov    %r10,%rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2760
  26cb43:	test   %r12,%r12
  26cb46:	mov    0x550(%rsp),%r13
  26cb4e:	je     26ce17 <<rexx_exec::Interp>::begin_invoke+0xad7>
  26cb54:	cmp    $0x1,%r12
  26cb58:	jne    26cd2f <<rexx_exec::Interp>::begin_invoke+0x9ef>
  26cb5e:	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2779
  26cb63:	mov    %r14,%rsi
  26cb66:	mov    0x20(%rsp),%rdx
  26cb6b:	mov    %r13,%rcx
  26cb6e:	mov    %rbp,%r8
  26cb71:	call   *%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2781
  26cb73:	mov    0x60(%rsp),%rax
  26cb78:	cmp    $0x2,%rax
  26cb7c:	mov    0x8(%rsp),%r15
  26cb81:	mov    0x548(%rsp),%r12
  26cb89:	mov    0x540(%rsp),%r13
  26cb91:	je     26cead <<rexx_exec::Interp>::begin_invoke+0xb6d>
  26cb97:	test   %rax,%rax
  26cb9a:	jne    26d212 <<rexx_exec::Interp>::begin_invoke+0xed2>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2782
  26cba0:	movdqu 0x68(%rsp),%xmm0
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>::Ran>:
  26cba6:	movdqu %xmm0,0x8(%r15)
  26cbac:	jmp    26d5dd <<rexx_exec::Interp>::begin_invoke+0x129d>
<rexx_exec::Interp>::begin_invoke:
  26cbb1:	mov    %rbx,%r15
  26cbb4:	lea    0x330(%rsp),%rdi
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2702
  26cbbc:	mov    %r14,%rsi
  26cbbf:	mov    0x30(%rsp),%rdx
  26cbc4:	call   1c8650 <<rexx_exec::Interp>::scope_id>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26cbc9:	mov    0x338(%rsp),%rbx
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  26cbd1:	mov    0x340(%rsp),%r8
  26cbd9:	lea    0x100(%rsp),%rdi
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2703
  26cbe1:	mov    %r13,%rsi
  26cbe4:	mov    %r12,%rdx
  26cbe7:	mov    %rbx,%rcx
  26cbea:	call   26e710 <<rexx_exec::Loud>::unbound_method>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26cbef:	mov    $0x18,%edi
  26cbf4:	call   *0x26b3be(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26cbfa:	test   %rax,%rax
  26cbfd:	je     26dfd1 <<rexx_exec::Interp>::begin_invoke+0x1c91>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26cc03:	mov    %rax,%r12
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26cc06:	mov    0x110(%rsp),%rax
  26cc0e:	mov    %rax,0x10(%r12)
  26cc13:	movdqu 0x100(%rsp),%xmm0
  26cc1c:	movdqu %xmm0,(%r12)
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26cc22:	cmpq   $0x0,0x330(%rsp)
  26cc2b:	je     26cc36 <<rexx_exec::Interp>::begin_invoke+0x8f6>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26cc2d:	mov    %rbx,%rdi
  26cc30:	call   *0x26b362(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_invoke:
  26cc36:	mov    $0x2,%r13d
  26cc3c:	mov    %r15,%rbx
<core::result::Result<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  26cc3f:	mov    %r13,0x8(%rbx)
  26cc43:	mov    %r12,0x10(%rbx)
  26cc47:	movq   $0x1,(%rbx)
<rexx_exec::Interp>::begin_invoke:
  26cc4e:	jmp    26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
<core::ptr::non_null::NonNull<alloc::rc::RcInner<rexx_api::load::Library>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:454
  26cc53:	mov    -0x8(%r8),%rbx
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26cc57:	incq   (%rbx)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26cc5a:	je     26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<core::option::Option<usize> as core::clone::Clone>::clone:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2274
  26cc60:	mov    -0x18(%r8),%r13
  26cc64:	mov    -0x10(%r8),%r12
  26cc68:	mov    $0x5,%eax
<rexx_exec::Interp>::invocable:
  26cc6d:	jmp    26c7bf <<rexx_exec::Interp>::begin_invoke+0x47f>
<core::option::Option<rexx_exec::dispatch::AccessScope>>::is_some_and::<<rexx_exec::Interp>::method_is_protected::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:660
  26cc72:	cmpq   $0x2,-0x18(%r8)
  26cc77:	je     26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
  26cc7d:	testb  $0x1,-0x8(%r8)
  26cc82:	je     26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
<rexx_exec::Interp>::effective_security_manager:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:115
  26cc88:	mov    %r14,%rdi
  26cc8b:	call   26ae50 <<rexx_exec::Interp>::armed_security_manager>
<core::option::Option<rexx_core::handle::ObjRef>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/macros/mod.rs:434
  26cc90:	cmp    $0x1,%rax
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2494
  26cc94:	jne    26de8d <<rexx_exec::Interp>::begin_invoke+0x1b4d>
  26cc9a:	mov    0x548(%rsp),%r15
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:131
  26cca2:	cmp    $0x7,%r15
  26cca6:	jbe    26d1d3 <<rexx_exec::Interp>::begin_invoke+0xe93>
<rexx_exec::Interp>::text_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:230
  26ccac:	lea    0x108(%rsp),%rdi
<rexx_exec::Interp>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:96
  26ccb4:	mov    0x540(%rsp),%rsi
  26ccbc:	mov    %r15,%rdx
  26ccbf:	call   15ae90 <<rexx_core::bytes::Bytes>::from_slice>
<rexx_exec::Interp>::text_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:230
  26ccc4:	movq   $0x0,0x140(%rsp)
  26ccd0:	movb   $0x0,0x100(%rsp)
  26ccd8:	lea    0x100(%rsp),%rdx
  26cce0:	mov    %r14,%rdi
  26cce3:	xor    %esi,%esi
  26cce5:	call   18c260 <<rexx_exec::Interp>::alloc_with>
  26ccea:	mov    %rax,0x18(%rsp)
<rexx_exec::Interp>::text:
  26ccef:	mov    0x548(%rsp),%rcx
<rexx_exec::Interp>::charge_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:104
  26ccf7:	cmp    $0x37,%rcx
  26ccfb:	jb     26d3e5 <<rexx_exec::Interp>::begin_invoke+0x10a5>
<rexx_core::heap::Heap>::hold_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:183
  26cd01:	add    %rcx,0xaa0(%r14)
  26cd08:	mov    0xa90(%r14),%rax
<rexx_core::heap::Heap>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:176
  26cd0f:	add    %rcx,%rax
  26cd12:	mov    %rax,0xa90(%r14)
<rexx_exec::Interp>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:3249
  26cd19:	cmp    0x1fc8(%r14),%rax
  26cd20:	setae  %al
  26cd23:	or     %al,0x1fff(%r14)
  26cd2a:	jmp    26d3e5 <<rexx_exec::Interp>::begin_invoke+0x10a5>
<core::ptr::non_null::NonNull<core::option::Option<rexx_core::handle::ObjRef>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  26cd2f:	test   %rbp,%rbp
<core::slice::iter::Iter<core::option::Option<rexx_core::handle::ObjRef>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  26cd32:	je     26ce17 <<rexx_exec::Interp>::begin_invoke+0xad7>
  26cd38:	mov    %rbp,%r15
  26cd3b:	shl    $0x4,%r15
  26cd3f:	xor    %r12d,%r12d
  26cd42:	jmp    26cd5d <<rexx_exec::Interp>::begin_invoke+0xa1d>
<core::ptr::non_null::NonNull<core::option::Option<rexx_core::handle::ObjRef>> as core::cmp::PartialEq>::eq:
  26cd44:	data16 data16 cs nopw 0x0(%rax,%rax,1)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  26cd50:	add    $0x10,%r12
  26cd54:	cmp    %r12,%r15
<core::slice::iter::Iter<core::option::Option<rexx_core::handle::ObjRef>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  26cd57:	je     26ce17 <<rexx_exec::Interp>::begin_invoke+0xad7>
core::iter::adapters::flatten::try_flatten_one::<&core::option::Option<rexx_core::handle::ObjRef>, (), core::ops::control_flow::ControlFlow<()>, core::iter::traits::iterator::Iterator::any::check<&rexx_core::handle::ObjRef, <rexx_exec::Interp>::argument_may_run_rexx::{closure#0}>::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/flatten.rs:819
  26cd5d:	cmpl   $0x1,0x0(%r13,%r12,1)
  26cd63:	jne    26cd50 <<rexx_exec::Interp>::begin_invoke+0xa10>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/flatten.rs:820
  26cd65:	mov    0x8(%r13,%r12,1),%rsi
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  26cd6a:	test   $0x3,%sil
  26cd6e:	jne    26cd50 <<rexx_exec::Interp>::begin_invoke+0xa10>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  26cd70:	mov    %rsi,%rax
  26cd73:	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  26cd77:	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26cd79:	cmp    %rcx,0xa08(%r14)
  26cd80:	jbe    26cdac <<rexx_exec::Interp>::begin_invoke+0xa6c>
<rexx_exec::Interp>::may_run_rexx:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:3907
  26cd82:	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  26cd89:	mov    %rsi,%rdx
  26cd8c:	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26cd90:	lea    (%rcx,%rcx,2),%rcx
  26cd94:	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  26cd98:	cmp    %edx,0x4(%rax,%rcx,1)
  26cd9c:	jne    26cdac <<rexx_exec::Interp>::begin_invoke+0xa6c>
  26cd9e:	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  26cda1:	cmpl   $0x1,(%rax)
  26cda4:	jne    26cdac <<rexx_exec::Interp>::begin_invoke+0xa6c>
<rexx_exec::Interp>::may_run_rexx:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:3906
  26cda6:	cmpb   $0x2,0x8(%rax)
  26cdaa:	jb     26cd50 <<rexx_exec::Interp>::begin_invoke+0xa10>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:3909
  26cdac:	mov    %r14,%rdi
  26cdaf:	call   1dfe50 <<rexx_exec::Interp>::is_base_class>
<core::slice::iter::Iter<core::option::Option<rexx_core::handle::ObjRef>> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::adapters::flatten::try_flatten_one<&core::option::Option<rexx_core::handle::ObjRef>, (), core::ops::control_flow::ControlFlow<()>, core::iter::traits::iterator::Iterator::any::check<&rexx_core::handle::ObjRef, <rexx_exec::Interp>::argument_may_run_rexx::{closure#0}>::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<()>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  26cdb4:	test   %al,%al
  26cdb6:	jne    26cd50 <<rexx_exec::Interp>::begin_invoke+0xa10>
<rexx_exec::Interp>::begin_invoke:
  26cdb8:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2765
  26cdc0:	mov    %r14,%rsi
  26cdc3:	mov    %rbx,%rdx
  26cdc6:	mov    0x30(%rsp),%rcx
  26cdcb:	mov    0x80(%rsp),%r8d
  26cdd3:	mov    0x20(%rsp),%r9
  26cdd8:	push   %rbp
  26cdd9:	push   %r13
  26cddb:	push   0x558(%rsp)
  26cde2:	push   0x558(%rsp)
  26cde9:	call   26f350 <<rexx_exec::Interp>::run_leveled>
  26cdee:	add    $0x20,%rsp
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::dispatch::NativeStarted, rexx_exec::dispatch::NativeStarted::Ran>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:836
  26cdf2:	cmpl   $0x1,0x100(%rsp)
  26cdfa:	jne    26e029 <<rexx_exec::Interp>::begin_invoke+0x1ce9>
  26ce00:	lea    0x108(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:838
  26ce08:	movdqu (%rax),%xmm0
  26ce0c:	movdqu %xmm0,0x68(%rsp)
  26ce12:	jmp    26c98b <<rexx_exec::Interp>::begin_invoke+0x64b>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/pinning.rs:33
  26ce17:	incl   0x520(%r14)
  26ce1e:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2774
  26ce26:	mov    %r14,%rsi
  26ce29:	mov    0x20(%rsp),%rdx
  26ce2e:	mov    %r13,%rcx
  26ce31:	mov    %rbp,%r8
  26ce34:	call   *%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/pinning.rs:35
  26ce36:	decl   0x520(%r14)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/pinning.rs:36
  26ce3d:	mov    0x110(%rsp),%rax
  26ce45:	mov    %rax,0x340(%rsp)
  26ce4d:	movdqu 0x100(%rsp),%xmm0
  26ce56:	movdqa %xmm0,0x330(%rsp)
<core::ptr::non_null::NonNull<alloc::sync::ArcInner<rexx_exec::timer::Inbox<rexx_exec::scheduler::Posted>>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:454
  26ce5f:	mov    0x13d0(%r14),%rax
core::sync::atomic::atomic_load::<u32>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/sync/atomic.rs:3995
  26ce66:	mov    0x40(%rax),%eax
<rexx_exec::timer::Requests>::pending:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/timer.rs:79
  26ce69:	test   $0x4,%al
<rexx_exec::Interp>::serve_posted_halt:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:2664
  26ce6b:	jne    26dfe5 <<rexx_exec::Interp>::begin_invoke+0x1ca5>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2777
  26ce71:	movdqu 0x338(%rsp),%xmm0
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::dispatch::NativeStarted, rexx_exec::dispatch::NativeStarted::Ran>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:836
  26ce7a:	cmpq   $0x1,0x330(%rsp)
  26ce83:	mov    0x8(%rsp),%r15
  26ce88:	mov    0x548(%rsp),%r12
  26ce90:	mov    0x540(%rsp),%r13
  26ce98:	jne    26cfb5 <<rexx_exec::Interp>::begin_invoke+0xc75>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:838
  26ce9e:	movdqu %xmm0,0x68(%rsp)
  26cea4:	movq   $0x2,0x60(%rsp)
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2796
  26cead:	movdqu 0x68(%rsp),%xmm0
  26ceb3:	movdqa %xmm0,0x80(%rsp)
  26cebc:	lea    0xa0(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2797
  26cec4:	mov    %r14,%rsi
  26cec7:	mov    0x30(%rsp),%rdx
  26cecc:	call   1c8650 <<rexx_exec::Interp>::scope_id>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26ced1:	mov    0xa8(%rsp),%rbx
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  26ced9:	mov    0xb0(%rsp),%r8
<rexx_exec::Interp>::blame_native_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:4045
  26cee1:	movb   $0x1,0x150(%rsp)
  26cee9:	mov    %r13,0x128(%rsp)
  26cef1:	mov    %r12,0x130(%rsp)
  26cef9:	mov    0x20(%rsp),%rax
  26cefe:	mov    %rax,0x148(%rsp)
  26cf06:	mov    0x550(%rsp),%rax
  26cf0e:	mov    %rax,0x138(%rsp)
  26cf16:	mov    %rbp,0x140(%rsp)
  26cf1e:	movq   $0x0,0x100(%rsp)
  26cf2a:	movq   $0xffffffffffffffff,0x110(%rsp)
  26cf36:	lea    0x300(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:4053
  26cf3e:	mov    %r13,%rsi
  26cf41:	mov    %r12,%rdx
  26cf44:	mov    %rbx,%rcx
  26cf47:	call   1c8800 <<rexx_exec::error::Raised>::compiled_method_line>
  26cf4c:	movq   $0xffffffffffffffff,0x330(%rsp)
  26cf58:	lea    0x300(%rsp),%rsi
  26cf60:	lea    0x330(%rsp),%rdx
  26cf68:	lea    0x100(%rsp),%rcx
  26cf70:	mov    %r14,%rdi
  26cf73:	call   1d7900 <<rexx_exec::Interp>::blame_native_level>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26cf78:	cmpq   $0x0,0xa0(%rsp)
  26cf81:	je     26cf8c <<rexx_exec::Interp>::begin_invoke+0xc4c>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26cf83:	mov    %rbx,%rdi
  26cf86:	call   *0x26b00c(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_invoke:
  26cf8c:	mov    $0x1,%eax
  26cf91:	movdqa 0x80(%rsp),%xmm0
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>::Ran>:
  26cf9a:	movdqu %xmm0,0x8(%r15)
  26cfa0:	mov    %rax,(%r15)
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2808
  26cfa3:	add    $0x508,%rsp
  26cfaa:	pop    %rbx
  26cfab:	pop    %r12
  26cfad:	pop    %r13
  26cfaf:	pop    %r14
  26cfb1:	pop    %r15
  26cfb3:	pop    %rbp
  26cfb4:	ret
  26cfb5:	xor    %eax,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2778
  26cfb7:	jmp    26cf9a <<rexx_exec::Interp>::begin_invoke+0xc5a>
rexx_exec::activation::body_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1128
  26cfb9:	mov    0x8(%rax),%rsi
<core::option::Option<rexx_parse::ast::CodeBody>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26cfbd:	cmpq   $0xffffffffffffffff,(%rsi)
  26cfc1:	je     26d15e <<rexx_exec::Interp>::begin_invoke+0xe1e>
<rexx_exec::Interp>::begin_method:
  26cfc7:	movq   $0x0,0x50(%rsp)
<hashbrown::map::HashMap<rexx_core::handle::ObjRef, rexx_exec::dispatch::executable::MethodFlagWrites, rustc_hash::FxBuildHasher>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/map.rs:825
  26cfd0:	cmpq   $0x0,0x1c58(%r14)
<rexx_exec::Interp>::sends_guarded:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/executable.rs:82
  26cfd8:	jne    26e060 <<rexx_exec::Interp>::begin_invoke+0x1d20>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:54
  26cfde:	movb   $0x0,0x60(%rsp)
<rexx_exec::Interp>::activation_depth:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1341
  26cfe3:	cmpq   $0x1,0x470(%r14)
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:55
  26cfeb:	mov    0x1b8(%r14),%rax
<rexx_exec::Interp>::activation_depth:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1341
  26cff2:	sbb    $0xffffffffffffffff,%rax
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:55
  26cff6:	cmp    $0x270f,%rax
  26cffc:	ja     26d024 <<rexx_exec::Interp>::begin_invoke+0xce4>
<rexx_exec::Interp>::stack_exhausted::<u8>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:1339
  26cffe:	mov    0x1f58(%r14),%rax
  26d005:	lea    0x60(%rsp),%rcx
<usize>::abs_diff:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:3181
  26d00a:	mov    %rax,%rdx
  26d00d:	sub    %rcx,%rdx
  26d010:	sub    %rax,%rcx
  26d013:	cmovb  %rdx,%rcx
<rexx_exec::Interp>::stack_exhausted::<u8>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:1339
  26d017:	cmp    0x1f60(%r14),%rcx
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:55
  26d01e:	jbe    26d2c4 <<rexx_exec::Interp>::begin_invoke+0xf84>
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  26d024:	mov    $0x2,%edi
  26d029:	mov    $0x1,%esi
  26d02e:	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  26d033:	test   %rax,%rax
  26d036:	je     26e272 <<rexx_exec::Interp>::begin_invoke+0x1f32>
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26d03c:	movw   $0x3131,(%rax)
<rexx_exec::error::Raised>::syntax:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:139
  26d041:	movq   $0xffffffffffffffff,0x118(%rsp)
  26d04d:	lea    -0x1c643d(%rip),%rcx        # a6c17 <panic_unwind::imp::CANARY+0x41eba>
  26d054:	mov    %rcx,0x120(%rsp)
  26d05c:	movq   $0x6,0x128(%rsp)
  26d068:	movq   $0x0,0x100(%rsp)
  26d074:	movq   $0x8,0x108(%rsp)
  26d080:	movq   $0x0,0x110(%rsp)
  26d08c:	movq   $0x2,0x130(%rsp)
  26d098:	mov    %rax,0x138(%rsp)
  26d0a0:	movq   $0x2,0x140(%rsp)
  26d0ac:	movq   $0xffffffffffffffff,0x148(%rsp)
  26d0b8:	movabs $0x1000b00000000,%rax
  26d0c2:	mov    %rax,0x160(%rsp)
  26d0ca:	movl   $0x0,0x168(%rsp)
  26d0d5:	movw   $0x0,0x16c(%rsp)
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26d0df:	mov    $0x70,%edi
  26d0e4:	call   *0x26aece(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26d0ea:	test   %rax,%rax
  26d0ed:	mov    0x18(%rsp),%rdi
  26d0f2:	je     26e04f <<rexx_exec::Interp>::begin_invoke+0x1d0f>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
  26d0f8:	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26d0fb:	movups 0x160(%rsp),%xmm0
  26d103:	movups %xmm0,0x60(%rax)
  26d107:	movups 0x150(%rsp),%xmm0
  26d10f:	movups %xmm0,0x50(%rax)
  26d113:	movups 0x140(%rsp),%xmm0
  26d11b:	movups %xmm0,0x40(%rax)
  26d11f:	movdqu 0x100(%rsp),%xmm0
  26d128:	movdqu 0x110(%rsp),%xmm1
  26d131:	movdqu 0x120(%rsp),%xmm2
  26d13a:	movdqu 0x130(%rsp),%xmm3
  26d143:	movdqu %xmm3,0x30(%rax)
  26d148:	movdqu %xmm2,0x20(%rax)
  26d14d:	movdqu %xmm1,0x10(%rax)
  26d152:	movdqu %xmm0,(%rax)
  26d156:	mov    $0x3,%r14d
<rexx_exec::Interp>::begin_method:
  26d15c:	jmp    26d1a6 <<rexx_exec::Interp>::begin_invoke+0xe66>
  26d15e:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:44
  26d166:	call   1cede0 <<rexx_exec::Loud>::missing_body>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26d16b:	mov    $0x18,%edi
  26d170:	call   *0x26ae42(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26d176:	test   %rax,%rax
  26d179:	je     26e03e <<rexx_exec::Interp>::begin_invoke+0x1cfe>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26d17f:	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26d182:	mov    0x110(%rsp),%rax
  26d18a:	mov    %rax,0x10(%rbx)
  26d18e:	movdqu 0x100(%rsp),%xmm0
  26d197:	movdqu %xmm0,(%rbx)
  26d19b:	mov    $0x2,%r14d
<rexx_exec::Interp>::begin_method:
  26d1a1:	mov    0x18(%rsp),%rdi
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d1a6:	decq   (%rdi)
<alloc::rc::Rc<rexx_parse::Program> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26d1a9:	jne    26d1b0 <<rexx_exec::Interp>::begin_invoke+0xe70>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26d1ab:	call   191f90 <<alloc::rc::Rc<rexx_parse::Program>>::drop_slow>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26d1b0:	cmp    $0xffffffffffffffff,%r14
  26d1b4:	jne    26c9f1 <<rexx_exec::Interp>::begin_invoke+0x6b1>
<rexx_exec::Interp>::begin_invoke:
  26d1ba:	mov    0x8(%rsp),%rax
  26d1bf:	movq   $0x2,0x8(%rax)
  26d1c7:	movq   $0x0,(%rax)
core::ptr::drop_glue::<rexx_exec::dispatch::Invocable>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  26d1ce:	jmp    26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:134
  26d1d3:	movq   $0x0,0x100(%rsp)
<core::iter::adapters::zip::Zip<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>> as core::iter::adapters::zip::ZipImpl<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>>>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/zip.rs:300
  26d1df:	test   %r15,%r15
  26d1e2:	je     26d3d6 <<rexx_exec::Interp>::begin_invoke+0x1096>
<rexx_core::handle::ObjRef>::inline_text:
  26d1e8:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:136
  26d1f0:	mov    0x540(%rsp),%rsi
  26d1f8:	mov    %r15,%rdx
  26d1fb:	call   *0x26adc7(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:138
  26d201:	mov    0x100(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:140
  26d209:	shl    $0x8,%rax
  26d20d:	jmp    26d3d8 <<rexx_exec::Interp>::begin_invoke+0x1098>
<rexx_exec::Interp>::begin_invoke:
  26d212:	mov    %r15,0x8(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2783
  26d217:	mov    0x68(%rsp),%r15
  26d21c:	mov    %r13,%rdi
  26d21f:	mov    0x70(%rsp),%r13
<alloc::boxed::Box<[u8]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:845
  26d224:	mov    %r12,%rsi
  26d227:	call   1ac620 <<alloc::boxed::Box<[u8]>>::try_clone_from_ref_in>
  26d22c:	test   %rax,%rax
  26d22f:	je     26e0f1 <<rexx_exec::Interp>::begin_invoke+0x1db1>
<rexx_exec::Interp>::begin_invoke:
  26d235:	mov    %rax,%rbx
  26d238:	mov    %rdx,%r12
<alloc::boxed::Box<[core::option::Option<rexx_core::handle::ObjRef>]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:845
  26d23b:	mov    0x550(%rsp),%rdi
  26d243:	mov    %rbp,%rsi
  26d246:	call   26f190 <<alloc::boxed::Box<[core::option::Option<rexx_core::handle::ObjRef>]>>::try_clone_from_ref_in>
  26d24b:	test   %rax,%rax
  26d24e:	je     26e100 <<rexx_exec::Interp>::begin_invoke+0x1dc0>
<core::option::Option<alloc::boxed::Box<rexx_exec::dispatch::NativePark>>>::as_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:765
  26d254:	mov    0x500(%r14),%rdi
  26d25b:	test   %rdi,%rdi
  26d25e:	je     26d580 <<rexx_exec::Interp>::begin_invoke+0x1240>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2791
  26d264:	add    $0x28,%rdi
  26d268:	mov    %r15,0x100(%rsp)
  26d270:	mov    %r13,0x108(%rsp)
  26d278:	mov    %rbx,0x110(%rsp)
  26d280:	mov    %r12,0x118(%rsp)
  26d288:	mov    %rax,0x120(%rsp)
  26d290:	mov    %rdx,0x128(%rsp)
  26d298:	mov    0x30(%rsp),%rax
  26d29d:	mov    %rax,0x130(%rsp)
  26d2a5:	mov    0x20(%rsp),%rax
  26d2aa:	mov    %rax,0x138(%rsp)
  26d2b2:	lea    0x100(%rsp),%rsi
<alloc::vec::Vec<(rexx_exec::dispatch::Then, core::option::Option<rexx_exec::dispatch::NativeBlame>)>>::push:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:996
  26d2ba:	call   26f1f0 <<alloc::vec::Vec<(rexx_exec::dispatch::Then, core::option::Option<rexx_exec::dispatch::NativeBlame>)>>::push_mut>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2791
  26d2bf:	jmp    26d5d0 <<rexx_exec::Interp>::begin_invoke+0x1290>
  26d2c4:	mov    0x18(%rsp),%rax
<rexx_exec::Interp>::begin_method:
  26d2c9:	lea    0x10(%rax),%r9
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:60
  26d2cd:	mov    %r13,0x110(%rsp)
  26d2d5:	movq   $0x1,0x100(%rsp)
  26d2e1:	mov    %r12,0x108(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:65
  26d2e9:	lea    0xa8(%rax),%r8
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:59
  26d2f0:	mov    0x8(%rsi),%rdx
  26d2f4:	mov    0x10(%rsi),%rcx
  26d2f8:	lea    0x100(%rsp),%rsi
  26d300:	mov    %r14,%rdi
  26d303:	call   25d7b0 <<rexx_exec::Interp>::plan_for>
  26d308:	mov    %rax,0x38(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:68
  26d30d:	mov    0x80(%rax),%rsi
  26d314:	lea    0xab0(%r14),%rdi
  26d31b:	call   15f920 <<rexx_core::roots::RootSet>::push_slots>
  26d320:	mov    %rax,0x48(%rsp)
  26d325:	mov    %rdx,0xf8(%rsp)
  26d32d:	movb   $0x1,0x7(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:71
  26d332:	cmpb   $0x0,0x50(%rsp)
  26d337:	je     26d353 <<rexx_exec::Interp>::begin_invoke+0x1013>
  26d339:	mov    0x38(%rsp),%rax
  26d33e:	cmpb   $0x0,0xf9(%rax)
  26d345:	jne    26d358 <<rexx_exec::Interp>::begin_invoke+0x1018>
  26d347:	movzbl 0xe4c(%r14),%eax
  26d34f:	mov    %al,0x7(%rsp)
  26d353:	mov    0x38(%rsp),%rax
  26d358:	mov    %rax,%rcx
  26d35b:	mov    0x30(%rax),%rax
  26d35f:	mov    %rax,0xe0(%rsp)
  26d367:	mov    0x38(%rcx),%rax
  26d36b:	mov    %rax,0xe8(%rsp)
  26d373:	mov    0x40(%rcx),%rax
  26d377:	mov    %rax,0xd8(%rsp)
  26d37f:	mov    0x48(%rcx),%rax
  26d383:	mov    %rax,0xf0(%rsp)
<rexx_exec::Interp>::next_activation_id:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1142
  26d38b:	mov    0x1f50(%r14),%rax
  26d392:	mov    %rax,0x10(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1143
  26d397:	inc    %rax
  26d39a:	mov    %rax,0x1f50(%r14)
  26d3a1:	mov    0x28(%rsp),%rsi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:73
  26d3a6:	cmpl   $0x2,(%rsi)
  26d3a9:	jne    26d5e9 <<rexx_exec::Interp>::begin_invoke+0x12a9>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:75
  26d3af:	mov    %r14,%rdi
  26d3b2:	mov    0x20(%rsp),%rsi
  26d3b7:	mov    0x30(%rsp),%rdx
  26d3bc:	call   26ea40 <<rexx_exec::Interp>::super_scope_for>
  26d3c1:	mov    %rax,0xc0(%rsp)
  26d3c9:	mov    %rdx,0xc8(%rsp)
  26d3d1:	jmp    26d606 <<rexx_exec::Interp>::begin_invoke+0x12c6>
<rexx_exec::Interp>::begin_invoke:
  26d3d6:	xor    %eax,%eax
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:140
  26d3d8:	lea    (%rax,%r15,4),%rax
  26d3dc:	add    $0x3,%rax
  26d3e0:	mov    %rax,0x18(%rsp)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  26d3e5:	mov    0xaf0(%r14),%r15
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26d3ec:	cmp    0xae0(%r14),%r15
  26d3f3:	jne    26d401 <<rexx_exec::Interp>::begin_invoke+0x10c1>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26d3f5:	lea    0xae0(%r14),%rdi
  26d3fc:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26d401:	mov    0xae8(%r14),%rax
  26d408:	mov    0x18(%rsp),%rcx
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26d40d:	mov    %rcx,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26d411:	inc    %r15
  26d414:	mov    %r15,0xaf0(%r14)
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2499
  26d41b:	mov    %r14,%rdi
  26d41e:	mov    0x550(%rsp),%rsi
  26d426:	mov    0x558(%rsp),%rdx
  26d42e:	call   26afc0 <<rexx_exec::Interp>::security_arguments_array>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2500
  26d433:	lea    -0x20339a(%rip),%rcx        # 6a0a0 <panic_unwind::imp::CANARY+0x5343>
  26d43a:	mov    %rcx,0x100(%rsp)
  26d442:	movq   $0x6,0x108(%rsp)
  26d44e:	mov    0x20(%rsp),%rcx
  26d453:	mov    %rcx,0x110(%rsp)
  26d45b:	lea    -0x21a9ea(%rip),%rcx        # 52a78 <GCC_except_table5014+0x3094>
  26d462:	mov    %rcx,0x118(%rsp)
  26d46a:	movq   $0x4,0x120(%rsp)
  26d476:	mov    0x18(%rsp),%rcx
  26d47b:	mov    %rcx,0x128(%rsp)
  26d483:	lea    -0x1c6922(%rip),%rcx        # a6b68 <panic_unwind::imp::CANARY+0x41e0b>
  26d48a:	mov    %rcx,0x130(%rsp)
  26d492:	movq   $0x9,0x138(%rsp)
  26d49e:	mov    %rax,0x140(%rsp)
<rexx_exec::Interp>::effective_security_manager:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:112
  26d4a6:	cmpb   $0x1,0x1ff4(%r14)
  26d4ae:	jne    26de8d <<rexx_exec::Interp>::begin_invoke+0x1b4d>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:115
  26d4b4:	mov    %r14,%rdi
  26d4b7:	call   26ae50 <<rexx_exec::Interp>::armed_security_manager>
<rexx_exec::Interp>::security_check_by:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:202
  26d4bc:	cmp    $0x1,%rax
  26d4c0:	jne    26de8d <<rexx_exec::Interp>::begin_invoke+0x1b4d>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  26d4c6:	mov    0xaf0(%r14),%rax
  26d4cd:	mov    %rax,0x18(%rsp)
<rexx_exec::Interp>::security_check_by:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:206
  26d4d2:	sub    $0x8,%rsp
  26d4d6:	lea    -0x1c6a58(%rip),%rcx        # a6a85 <panic_unwind::imp::CANARY+0x41d28>
  26d4dd:	lea    0x338(%rsp),%rdi
  26d4e5:	lea    0x108(%rsp),%r9
  26d4ed:	mov    $0x6,%r8d
  26d4f3:	mov    %r14,%rsi
  26d4f6:	push   $0x3
  26d4f8:	call   26b0f0 <<rexx_exec::Interp>::security_send>
  26d4fd:	add    $0x10,%rsp
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::truncate:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1823
  26d501:	mov    0xaf0(%r14),%r15
  26d508:	mov    0x18(%rsp),%rax
  26d50d:	cmp    %r15,%rax
  26d510:	ja     26d51c <<rexx_exec::Interp>::begin_invoke+0x11dc>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1828
  26d512:	mov    %rax,0xaf0(%r14)
  26d519:	mov    %rax,%r15
<rexx_exec::Interp>::security_check_by:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:208
  26d51c:	movzbl 0x330(%rsp),%edx
  26d524:	mov    %edx,%ecx
  26d526:	xor    $0x1,%cl
  26d529:	mov    0x338(%rsp),%rax
  26d531:	and    %al,%cl
  26d533:	mov    0x340(%rsp),%rsi
  26d53b:	mov    %rsi,0x18(%rsp)
  26d540:	cmp    $0x1,%cl
  26d543:	jne    26da72 <<rexx_exec::Interp>::begin_invoke+0x1732>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26d549:	lea    0xae0(%r14),%rax
  26d550:	cmp    (%rax),%r15
  26d553:	jne    26d561 <<rexx_exec::Interp>::begin_invoke+0x1221>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26d555:	lea    0xae0(%r14),%rdi
  26d55c:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26d561:	mov    0xae8(%r14),%rax
  26d568:	mov    0x18(%rsp),%rcx
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26d56d:	mov    %rcx,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26d571:	inc    %r15
  26d574:	mov    %r15,0xaf0(%r14)
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2505
  26d57b:	jmp    26dde1 <<rexx_exec::Interp>::begin_invoke+0x1aa1>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2792
  26d580:	mov    %rbx,0x100(%rsp)
  26d588:	mov    %r12,0x108(%rsp)
  26d590:	mov    %rax,0x110(%rsp)
  26d598:	mov    %rdx,0x118(%rsp)
  26d5a0:	mov    0x30(%rsp),%rax
  26d5a5:	mov    %rax,0x120(%rsp)
  26d5ad:	mov    0x20(%rsp),%rax
  26d5b2:	mov    %rax,0x128(%rsp)
  26d5ba:	lea    0x100(%rsp),%rcx
  26d5c2:	mov    %r14,%rdi
  26d5c5:	mov    %r15,%rsi
  26d5c8:	mov    %r13,%rdx
  26d5cb:	call   26f260 <<rexx_exec::Interp>::push_native_tail>
  26d5d0:	mov    0x8(%rsp),%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2794
  26d5d5:	movq   $0x2,0x8(%r15)
  26d5dd:	movq   $0x0,(%r15)
  26d5e4:	jmp    26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:74
  26d5e9:	mov    %r14,%rdi
  26d5ec:	mov    0x30(%rsp),%rdx
  26d5f1:	call   2328e0 <<rexx_exec::Interp>::super_scope_in>
  26d5f6:	mov    %rax,0xc0(%rsp)
  26d5fe:	mov    %rdx,0xc8(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:82
  26d606:	mov    %r14,%rdi
  26d609:	mov    0x550(%rsp),%rsi
  26d611:	mov    0x558(%rsp),%rdx
  26d619:	call   25e000 <<rexx_exec::Interp>::shared_arguments>
  26d61e:	mov    %rax,0x98(%rsp)
  26d626:	mov    %rdx,0x28(%rsp)
<rexx_exec::activity::Activity>::invocation_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:434
  26d62b:	mov    0x4f0(%r14),%rcx
  26d632:	mov    0x548(%rsp),%rax
  26d63a:	cmp    %rax,0x4f8(%r14)
  26d641:	sete   %al
  26d644:	test   %rcx,%rcx
  26d647:	je     26e088 <<rexx_exec::Interp>::begin_invoke+0x1d48>
  26d64d:	test   %al,%al
  26d64f:	je     26e092 <<rexx_exec::Interp>::begin_invoke+0x1d52>
<[u8]>::as_ptr:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:729
  26d655:	lea    0x10(%rcx),%rdi
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  26d659:	mov    0x540(%rsp),%rsi
  26d661:	mov    %rcx,0xd0(%rsp)
  26d669:	mov    0x548(%rsp),%rdx
  26d671:	call   *0x26a961(%rip)        # 4d7fd8 <bcmp@GLIBC_2.2.5>
  26d677:	mov    0xd0(%rsp),%rdx
  26d67f:	test   %eax,%eax
<rexx_exec::activity::Activity>::invocation_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:435
  26d681:	jne    26e092 <<rexx_exec::Interp>::begin_invoke+0x1d52>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d687:	incq   (%rdx)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26d68a:	je     26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d690:	incq   (%rdx)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26d693:	je     26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
core::mem::replace::<rexx_exec::CallContext>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  26d699:	movdqu 0x70(%r14),%xmm0
  26d69f:	movdqu 0x80(%r14),%xmm1
  26d6a8:	movdqu 0x90(%r14),%xmm2
  26d6b1:	movdqa %xmm2,0x320(%rsp)
  26d6ba:	movdqa %xmm1,0x310(%rsp)
  26d6c3:	movdqa %xmm0,0x300(%rsp)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d6cc:	movq   $0x1,0x70(%r14)
  26d6d4:	mov    0x20(%rsp),%rax
  26d6d9:	mov    %rax,0x78(%r14)
  26d6dd:	mov    %rdx,0x80(%r14)
  26d6e4:	mov    0x78(%rsp),%rax
  26d6e9:	mov    %rax,0x88(%r14)
  26d6f0:	mov    0x98(%rsp),%rax
  26d6f8:	mov    %rax,0x90(%r14)
  26d6ff:	mov    0x28(%rsp),%rax
  26d704:	mov    %rax,0x98(%r14)
  26d70b:	mov    $0x2,%dil
<rexx_exec::Interp>::options_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2679
  26d70e:	cmpb   $0x1,0x1ff6(%r14)
  26d716:	jne    26d763 <<rexx_exec::Interp>::begin_invoke+0x1423>
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::ProgramSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26d718:	cmp    0xc58(%r14),%rbx
  26d71f:	jae    26d763 <<rexx_exec::Interp>::begin_invoke+0x1423>
<rexx_exec::plan::Programs>::package_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:150
  26d721:	mov    0xc50(%r14),%rax
<rexx_exec::plan::ProgramId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:32
  26d728:	cmp    %r13,(%rax,%r15,1)
<rexx_exec::plan::Programs>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:136
  26d72c:	jne    26d763 <<rexx_exec::Interp>::begin_invoke+0x1423>
  26d72e:	add    %r15,%rax
<core::option::Option<alloc::rc::Rc<rexx_parse::Program>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26d731:	cmpq   $0x0,0x8(%rax)
  26d736:	je     26d763 <<rexx_exec::Interp>::begin_invoke+0x1423>
  26d738:	cmp    0xc70(%r14),%rbx
  26d73f:	jae    26d763 <<rexx_exec::Interp>::begin_invoke+0x1423>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_exec::plan::PackageSlot>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26d741:	mov    0xc68(%r14),%rax
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::PackageSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26d748:	lea    (%rbx,%rbx,2),%rcx
<core::option::Option<rexx_core::handle::ObjRef> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  26d74c:	cmpl   $0x1,(%rax,%rcx,8)
  26d750:	jne    26d763 <<rexx_exec::Interp>::begin_invoke+0x1423>
<rexx_exec::plan::Programs>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:158
  26d752:	lea    (%rax,%rcx,8),%rax
  26d756:	mov    0x8(%rax),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  26d75a:	test   $0x3,%cl
  26d75d:	je     26df31 <<rexx_exec::Interp>::begin_invoke+0x1bf1>
<rexx_exec::Interp>::begin_method:
  26d763:	mov    0x10(%rsp),%rax
<rexx_exec::activation::Activation>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:880
  26d768:	mov    %rax,0x418(%rsp)
  26d770:	mov    0x18(%rsp),%rax
  26d775:	mov    %rax,0x3c8(%rsp)
  26d77d:	mov    %r13,0x420(%rsp)
  26d785:	movq   $0x1,0x330(%rsp)
  26d791:	mov    %r12,0x338(%rsp)
  26d799:	mov    0x38(%rsp),%rax
  26d79e:	mov    %rax,0x3d0(%rsp)
  26d7a6:	movdqu 0x265a2a(%rip),%xmm0        # 4d31d8 <rexx_api::ffi::CALL_CONTEXT+0x17e48>
  26d7ae:	movdqu %xmm0,0x3d8(%rsp)
  26d7b7:	movdqu 0x265a29(%rip),%xmm1        # 4d31e8 <rexx_api::ffi::CALL_CONTEXT+0x17e58>
  26d7bf:	movdqu %xmm1,0x3e8(%rsp)
  26d7c8:	mov    0x48(%rsp),%rax
  26d7cd:	mov    %rax,0x428(%rsp)
  26d7d5:	mov    0xf8(%rsp),%r13
  26d7dd:	mov    %r13,0x430(%rsp)
  26d7e5:	movw   $0x1,0x4f6(%rsp)
  26d7ef:	movb   $0x3,0x506(%rsp)
  26d7f7:	mov    %rdx,0x438(%rsp)
  26d7ff:	mov    0x78(%rsp),%rax
  26d804:	mov    %rax,0x440(%rsp)
  26d80c:	mov    0x30(%rsp),%rax
  26d811:	mov    %rax,0x448(%rsp)
  26d819:	mov    0x20(%rsp),%rax
  26d81e:	mov    %rax,0x450(%rsp)
  26d826:	movq   $0x0,0x370(%rsp)
  26d832:	movq   $0x8,0x378(%rsp)
  26d83e:	movq   $0x0,0x380(%rsp)
  26d84a:	pxor   %xmm2,%xmm2
  26d84e:	movdqu %xmm2,0x458(%rsp)
  26d857:	movb   $0x0,0x507(%rsp)
  26d85f:	movb   $0x1,0x4f8(%rsp)
  26d867:	movq   $0x9,0x398(%rsp)
  26d873:	movq   $0x0,0x3a0(%rsp)
  26d87f:	movb   $0x0,0x3a8(%rsp)
  26d887:	movl   $0x0,0x388(%rsp)
  26d892:	movw   $0x0,0x4f4(%rsp)
  26d89c:	movl   $0x0,0x4f0(%rsp)
  26d8a7:	movabs $0x1000000000000,%rax
  26d8b1:	mov    %rax,0x4f9(%rsp)
  26d8b9:	movb   $0x4e,0x501(%rsp)
  26d8c1:	movq   $0x0,0x3b0(%rsp)
  26d8cd:	movb   $0x2,0x3b8(%rsp)
  26d8d5:	movw   $0x0,0x3c1(%rsp)
  26d8df:	movb   $0x0,0x3c3(%rsp)
  26d8e7:	movq   $0x0,0x468(%rsp)
  26d8f3:	movq   $0x0,0x478(%rsp)
  26d8ff:	movq   $0x0,0x340(%rsp)
  26d90b:	movdqu %xmm2,0x488(%rsp)
  26d914:	movdqu %xmm2,0x498(%rsp)
  26d91d:	movdqu %xmm2,0x4a8(%rsp)
  26d926:	movl   $0x30001,0x502(%rsp)
  26d931:	movq   $0x0,0x350(%rsp)
  26d93d:	movq   $0x0,0x360(%rsp)
  26d949:	movq   $0x1,0x4b8(%rsp)
  26d955:	movl   $0x0,0x390(%rsp)
  26d960:	movq   $0x0,0x4e0(%rsp)
  26d96c:	movdqu %xmm2,0x4c0(%rsp)
  26d975:	movq   $0x0,0x4d0(%rsp)
  26d981:	movdqu %xmm1,0x408(%rsp)
  26d98a:	movdqu %xmm0,0x3f8(%rsp)
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26d993:	cmp    %rbp,0xfd8(%r14)
  26d99a:	jbe    26d9c1 <<rexx_exec::Interp>::begin_invoke+0x1681>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26d99c:	mov    0xfd0(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26d9a3:	shl    $0x4,%rbp
<core::option::Option<rexx_core::handle::ObjRef>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26d9a7:	cmpb   $0x0,(%rax,%rbp,1)
  26d9ab:	je     26dab6 <<rexx_exec::Interp>::begin_invoke+0x1776>
  26d9b1:	mov    %edi,%r15d
  26d9b4:	lea    (%rax,%rbp,1),%rdi
  26d9b8:	add    $0x8,%rdi
  26d9bc:	jmp    26da97 <<rexx_exec::Interp>::begin_invoke+0x1757>
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_core::handle::ObjRef, rustc_hash::FxBuildHasher>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/map.rs:825
  26d9c1:	cmpq   $0x0,0xff8(%r14)
<rexx_exec::method_rows::MethodRows<rexx_core::handle::ObjRef>>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/method_rows.rs:33
  26d9c9:	je     26dab6 <<rexx_exec::Interp>::begin_invoke+0x1776>
<std::collections::hash::map::HashMap<rexx_core::body::MethodId, rexx_core::handle::ObjRef, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/map.rs:1039
  26d9cf:	movabs $0xf1357aea2e62a9c5,%rax
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26d9d9:	imul   %rax,%rbp
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26d9dd:	rol    $0x1a,%rbp
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26d9e1:	mov    %rbp,%rdx
  26d9e4:	shr    $0x39,%rdx
  26d9e8:	mov    0xfe0(%r14),%rax
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26d9ef:	mov    0xfe8(%r14),%rcx
  26d9f6:	movd   %edx,%xmm0
  26d9fa:	punpcklbw %xmm0,%xmm0
  26d9fe:	pshuflw $0x0,%xmm0,%xmm0
  26da03:	pshufd $0x44,%xmm0,%xmm0
  26da08:	xor    %edx,%edx
  26da0a:	pcmpeqd %xmm1,%xmm1
  26da0e:	mov    0x80(%rsp),%r9d
<hashbrown::raw::RawTableInner>::find_inner:
  26da16:	and    %rcx,%rbp
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26da19:	movdqu (%rax,%rbp,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26da1e:	movdqa %xmm2,%xmm3
  26da22:	pcmpeqb %xmm0,%xmm3
  26da26:	pmovmskb %xmm3,%esi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26da2a:	test   %esi,%esi
  26da2c:	je     26da59 <<rexx_exec::Interp>::begin_invoke+0x1719>
  26da2e:	mov    %edi,%r15d
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26da31:	tzcnt  %esi,%r8d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26da36:	add    %rbp,%r8
  26da39:	and    %rcx,%r8
<*mut (rexx_core::body::MethodId, rexx_core::handle::ObjRef)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26da3c:	shl    $0x4,%r8
  26da40:	mov    %rax,%rdi
  26da43:	sub    %r8,%rdi
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26da46:	cmp    -0x10(%rdi),%r9d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26da4a:	je     26da93 <<rexx_exec::Interp>::begin_invoke+0x1753>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26da4c:	lea    -0x1(%rsi),%edi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26da4f:	and    %si,%di
  26da52:	mov    %edi,%esi
  26da54:	mov    %r15d,%edi
  26da57:	jne    26da2e <<rexx_exec::Interp>::begin_invoke+0x16ee>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26da59:	pcmpeqb %xmm1,%xmm2
  26da5d:	pmovmskb %xmm2,%esi
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26da61:	test   %esi,%esi
  26da63:	jne    26dab6 <<rexx_exec::Interp>::begin_invoke+0x1776>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26da65:	add    %rdx,%rbp
  26da68:	add    $0x10,%rbp
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26da6c:	add    $0x10,%rdx
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26da70:	jmp    26da16 <<rexx_exec::Interp>::begin_invoke+0x16d6>
<rexx_exec::Interp>::begin_invoke:
  26da72:	mov    $0x1,%ecx
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26da77:	test   %dl,%dl
  26da79:	je     26ddd9 <<rexx_exec::Interp>::begin_invoke+0x1a99>
  26da7f:	mov    0x8(%rsp),%rdx
  26da84:	mov    0x40(%rsp),%rsi
  26da89:	mov    0x18(%rsp),%r15
  26da8e:	jmp    26deb4 <<rexx_exec::Interp>::begin_invoke+0x1b74>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_core::handle::ObjRef)>>::get::<hashbrown::map::equivalent_key<rexx_core::body::MethodId, rexx_core::body::MethodId, rexx_core::handle::ObjRef>::{closure#0}>:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1217
  26da93:	add    $0xfffffffffffffff8,%rdi
<core::option::Option<&rexx_core::handle::ObjRef>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26da97:	mov    (%rdi),%r12
  26da9a:	lea    0x330(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:125
  26daa2:	call   1d3620 <<rexx_exec::activation::Activation>::cold_mut>
  26daa7:	movq   $0x1,0x10(%rax)
  26daaf:	mov    %r12,0x18(%rax)
  26dab3:	mov    %r15d,%edi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:127
  26dab6:	cmp    $0x2,%dil
  26daba:	je     26db15 <<rexx_exec::Interp>::begin_invoke+0x17d5>
  26dabc:	lea    0x398(%rsp),%rax
  26dac4:	mov    0xb0(%rsp),%rcx
  26dacc:	mov    %rcx,0x10(%rax)
  26dad0:	movdqa 0xa0(%rsp),%xmm0
  26dad9:	movdqu %xmm0,(%rax)
<rexx_exec::PackageStart>::apply:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1896
  26dadd:	mov    %dil,0x4f0(%rsp)
  26dae5:	mov    0x58(%rsp),%eax
  26dae9:	mov    %eax,0x4f1(%rsp)
  26daf0:	movzbl 0x5c(%rsp),%eax
  26daf5:	mov    %al,0x4f5(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1897
  26dafc:	cmp    $0x2,%bl
  26daff:	je     26db15 <<rexx_exec::Interp>::begin_invoke+0x17d5>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1898
  26db01:	mov    %bl,0x4f9(%rsp)
  26db08:	mov    0x28(%rsp),%rax
  26db0d:	mov    %rax,0x4fa(%rsp)
<rexx_exec::Interp>::begin_method:
  26db15:	movzbl 0x7(%rsp),%ebp
<rexx_exec::activation::ActivationFlags>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:516
  26db1a:	or     0x4fc(%rsp),%bpl
  26db22:	mov    0x50(%rsp),%rax
  26db27:	add    %eax,%eax
  26db29:	mov    %ebp,%ecx
  26db2b:	not    %cl
  26db2d:	and    $0x1,%cl
  26db30:	shl    $0x2,%cl
  26db33:	or     %al,%cl
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:133
  26db35:	mov    %cl,0x507(%rsp)
  26db3c:	lea    0x100(%rsp),%rbx
  26db44:	lea    0x330(%rsp),%rsi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:134
  26db4c:	mov    $0x1d8,%edx
  26db51:	mov    %rbx,%rdi
  26db54:	call   *0x26a46e(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
  26db5a:	mov    %r14,%rdi
  26db5d:	mov    %rbx,%rsi
  26db60:	call   25d5a0 <<rexx_exec::Interp>::push_activation>
  26db65:	mov    0xf0(%rsp),%rbx
  26db6d:	mov    0xe8(%rsp),%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:135
  26db75:	mov    %r14,%rdi
  26db78:	call   25de30 <<rexx_exec::Interp>::trace_package_invocation_entry>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:136
  26db7d:	test   %bpl,0x50(%rsp)
  26db82:	jne    26e1b0 <<rexx_exec::Interp>::begin_invoke+0x1e70>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:148
  26db88:	cmpq   $0x0,0xe0(%rsp)
  26db91:	jne    26dbaa <<rexx_exec::Interp>::begin_invoke+0x186a>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:150
  26db93:	lea    -0x21b182(%rip),%rsi        # 52a18 <GCC_except_table5014+0x3034>
  26db9a:	mov    $0x4,%edx
  26db9f:	mov    %r14,%rdi
  26dba2:	call   21f570 <<rexx_exec::Interp>::slot_of>
  26dba7:	mov    %rax,%r15
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:86
  26dbaa:	mov    0x470(%r14),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26dbb1:	test   %rax,%rax
  26dbb4:	je     26e0bd <<rexx_exec::Interp>::begin_invoke+0x1d7d>
<alloc::vec::Vec<(usize, rexx_exec::activation::InstanceVar)>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3126
  26dbba:	cmpq   $0x0,0x50(%rax)
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:87
  26dbbf:	je     26dbdc <<rexx_exec::Interp>::begin_invoke+0x189c>
<rexx_core::roots::SlotFrame as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:31
  26dbc1:	cmp    %r13,0x100(%rax)
  26dbc8:	jne    26dbdc <<rexx_exec::Interp>::begin_invoke+0x189c>
  26dbca:	mov    0x48(%rsp),%rcx
  26dbcf:	cmp    %rcx,0xf8(%rax)
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:39
  26dbd6:	je     26e1c7 <<rexx_exec::Interp>::begin_invoke+0x1e87>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:300
  26dbdc:	cmp    0xb80(%r14),%r13
  26dbe3:	jne    26e113 <<rexx_exec::Interp>::begin_invoke+0x1dd3>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:302
  26dbe9:	add    0xb78(%r14),%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  26dbf0:	cmp    0xb08(%r14),%r15
  26dbf7:	jae    26e0d7 <<rexx_exec::Interp>::begin_invoke+0x1d97>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26dbfd:	mov    0xb00(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:245
  26dc04:	shl    $0x4,%r15
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:308
  26dc08:	movq   $0x1,(%rax,%r15,1)
  26dc10:	mov    0x20(%rsp),%rcx
  26dc15:	mov    %rcx,0x8(%rax,%r15,1)
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:153
  26dc1a:	cmpq   $0x0,0xd8(%rsp)
  26dc23:	jne    26dc3c <<rexx_exec::Interp>::begin_invoke+0x18fc>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:155
  26dc25:	lea    -0x1c7356(%rip),%rsi        # a68d6 <panic_unwind::imp::CANARY+0x41b79>
  26dc2c:	mov    $0x5,%edx
  26dc31:	mov    %r14,%rdi
  26dc34:	call   21f570 <<rexx_exec::Interp>::slot_of>
  26dc39:	mov    %rax,%rbx
<core::option::Option<rexx_core::handle::ObjRef>>::unwrap_or:
  26dc3c:	testb  $0x1,0xc0(%rsp)
  26dc44:	mov    $0x2,%r8d
  26dc4a:	cmovne 0xc8(%rsp),%r8
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:86
  26dc53:	mov    0x470(%r14),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26dc5a:	test   %rax,%rax
  26dc5d:	je     26e0bd <<rexx_exec::Interp>::begin_invoke+0x1d7d>
<alloc::vec::Vec<(usize, rexx_exec::activation::InstanceVar)>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3126
  26dc63:	cmpq   $0x0,0x50(%rax)
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:87
  26dc68:	je     26dc85 <<rexx_exec::Interp>::begin_invoke+0x1945>
<rexx_core::roots::SlotFrame as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:31
  26dc6a:	cmp    %r13,0x100(%rax)
  26dc71:	jne    26dc85 <<rexx_exec::Interp>::begin_invoke+0x1945>
  26dc73:	mov    0x48(%rsp),%rcx
  26dc78:	cmp    %rcx,0xf8(%rax)
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:39
  26dc7f:	je     26e1e4 <<rexx_exec::Interp>::begin_invoke+0x1ea4>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:300
  26dc85:	cmp    0xb80(%r14),%r13
  26dc8c:	jne    26e134 <<rexx_exec::Interp>::begin_invoke+0x1df4>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:302
  26dc92:	add    0xb78(%r14),%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  26dc99:	cmp    0xb08(%r14),%rbx
  26dca0:	jae    26e0d7 <<rexx_exec::Interp>::begin_invoke+0x1d97>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26dca6:	mov    0xb00(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:245
  26dcad:	shl    $0x4,%rbx
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:308
  26dcb1:	movq   $0x1,(%rax,%rbx,1)
  26dcb9:	mov    %r8,0x8(%rax,%rbx,1)
<rexx_exec::Interp>::save_clause_state:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/clause.rs:456
  26dcbe:	mov    0x460(%r14),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/clause.rs:457
  26dcc5:	movzbl 0x468(%r14),%ecx
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:172
  26dccd:	movaps 0x300(%rsp),%xmm0
  26dcd5:	movdqa 0x310(%rsp),%xmm1
  26dcde:	movdqa 0x320(%rsp),%xmm2
  26dce7:	movdqa %xmm2,0x130(%rsp)
  26dcf0:	movdqa %xmm1,0x120(%rsp)
  26dcf9:	movaps %xmm0,0x110(%rsp)
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dd01:	movb   $0x2,0x188(%rsp)
<rexx_exec::Interp>::save_clause_state:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/clause.rs:454
  26dd09:	movups 0x450(%r14),%xmm0
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dd11:	movups %xmm0,0x158(%rsp)
  26dd19:	mov    %rax,0x168(%rsp)
  26dd21:	mov    %cl,0x170(%rsp)
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  26dd28:	movdqu 0x4a8(%r14),%xmm0
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26dd31:	pxor   %xmm1,%xmm1
  26dd35:	movdqu %xmm1,0x4a8(%r14)
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dd3e:	pshufd $0x4e,%xmm0,%xmm0
  26dd43:	movdqu %xmm0,0x178(%rsp)
core::mem::replace::<core::option::Option<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  26dd4c:	movdqu 0x60(%r14),%xmm0
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26dd52:	movq   $0x0,0x60(%r14)
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dd5a:	movdqa %xmm0,0x100(%rsp)
  26dd63:	movabs $0x8000000000000000,%rax
  26dd6d:	mov    %rax,0x140(%rsp)
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  26dd75:	mov    0x2a8(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26dd7c:	cmp    0x298(%r14),%rbx
  26dd83:	jne    26dd91 <<rexx_exec::Interp>::begin_invoke+0x1a51>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:171
  26dd85:	lea    0x298(%r14),%rdi
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26dd8c:	call   25e1e0 <<alloc::raw_vec::RawVec<rexx_exec::run::call::CallTail>>::grow_one>
<*mut rexx_exec::run::call::CallTail>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  26dd91:	lea    (%rbx,%rbx,8),%rdi
  26dd95:	shl    $0x4,%rdi
  26dd99:	add    0x2a0(%r14),%rdi
  26dda0:	lea    0x100(%rsp),%rsi
core::ptr::write::<rexx_exec::run::call::CallTail>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26dda8:	mov    $0x90,%edx
  26ddad:	call   *0x26a215(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26ddb3:	inc    %rbx
  26ddb6:	mov    %rbx,0x2a8(%r14)
  26ddbd:	mov    $0x2,%r15d
  26ddc3:	mov    $0x8,%eax
  26ddc8:	xor    %ecx,%ecx
  26ddca:	mov    0x8(%rsp),%rdx
  26ddcf:	mov    0x40(%rsp),%rsi
  26ddd4:	jmp    26debd <<rexx_exec::Interp>::begin_invoke+0x1b7d>
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2505
  26ddd9:	test   $0x1,%al
  26dddb:	je     26de8d <<rexx_exec::Interp>::begin_invoke+0x1b4d>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26dde1:	lea    0xae0(%r14),%rax
  26dde8:	cmp    (%rax),%r15
  26ddeb:	mov    0x18(%rsp),%rbx
  26ddf0:	jne    26ddfe <<rexx_exec::Interp>::begin_invoke+0x1abe>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26ddf2:	lea    0xae0(%r14),%rdi
  26ddf9:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26ddfe:	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26de05:	movabs $0x544c555345521b,%rcx
  26de0f:	mov    %rcx,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26de13:	inc    %r15
  26de16:	mov    %r15,0xaf0(%r14)
  26de1d:	lea    0x330(%rsp),%rdi
rexx_exec::dispatch::hash::store_get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/hash.rs:989
  26de25:	mov    %r14,%rsi
  26de28:	mov    %rbx,%rdx
  26de2b:	call   26b690 <rexx_exec::dispatch::hash::merged_get>
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
  26de30:	mov    0x338(%rsp),%rax
  26de38:	mov    0x340(%rsp),%r15
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26de40:	cmpb   $0x0,0x330(%rsp)
  26de48:	mov    $0x1,%ecx
  26de4d:	jne    26deaa <<rexx_exec::Interp>::begin_invoke+0x1b6a>
<rexx_exec::Interp>::security_entry:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:275
  26de4f:	cmp    $0x1,%rax
  26de53:	jne    26dea6 <<rexx_exec::Interp>::begin_invoke+0x1b66>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  26de55:	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26de5c:	cmp    0xae0(%r14),%rbx
  26de63:	jne    26de71 <<rexx_exec::Interp>::begin_invoke+0x1b31>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26de65:	lea    0xae0(%r14),%rdi
  26de6c:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26de71:	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26de78:	mov    %r15,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26de7c:	inc    %rbx
  26de7f:	mov    %rbx,0xaf0(%r14)
  26de86:	mov    $0x1,%eax
  26de8b:	jmp    26dea8 <<rexx_exec::Interp>::begin_invoke+0x1b68>
<rexx_exec::Interp>::begin_invoke:
  26de8d:	mov    0x28(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2732
  26de92:	movq   $0x2,(%rax)
  26de99:	mov    0x558(%rsp),%r10
  26dea1:	jmp    26c80c <<rexx_exec::Interp>::begin_invoke+0x4cc>
  26dea6:	xor    %eax,%eax
  26dea8:	xor    %ecx,%ecx
  26deaa:	mov    0x8(%rsp),%rdx
  26deaf:	mov    0x40(%rsp),%rsi
  26deb4:	mov    %rax,0x8(%rdx)
  26deb8:	mov    $0x10,%eax
  26debd:	mov    %r15,(%rdx,%rax,1)
  26dec1:	mov    %rcx,(%rdx)
core::ptr::drop_glue::<rexx_exec::dispatch::Invocable>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  26dec4:	cmp    $0x5,%rsi
  26dec8:	jb     26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
<core::ptr::non_null::NonNull<alloc::rc::RcInner<rexx_api::load::Library>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:454
  26dece:	mov    0x2f8(%rsp),%rdi
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26ded6:	decq   (%rdi)
<alloc::rc::Rc<rexx_api::load::Library> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26ded9:	jne    26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26dedf:	call   162d60 <<alloc::rc::Rc<rexx_api::load::Library>>::drop_slow>
  26dee4:	jmp    26cfa3 <<rexx_exec::Interp>::begin_invoke+0xc63>
<rexx_exec::Interp>::object_model:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:1833
  26dee9:	mov    %r14,%rdi
  26deec:	mov    %r8,%r12
  26deef:	call   18a340 <<rexx_exec::Interp>::build_object_model>
  26def4:	mov    %r12,%r8
<core::option::Option<rexx_exec::dispatch::ObjectModel>>::as_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:765
  26def7:	cmpq   $0xffffffffffffffff,0x1170(%r14)
  26deff:	jne    26c372 <<rexx_exec::Interp>::begin_invoke+0x32>
<core::option::Option<&mut rexx_exec::dispatch::ObjectModel>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  26df05:	lea    -0x203f8a(%rip),%rdi        # 69f82 <panic_unwind::imp::CANARY+0x5225>
  26df0c:	lea    0x2523cd(%rip),%rdx        # 4c02e0 <rexx_api::ffi::CALL_CONTEXT+0x4f50>
  26df13:	mov    $0x23,%esi
  26df18:	call   11f010 <core::option::expect_failed>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  26df1d:	mov    $0x8,%edi
  26df22:	mov    $0x18,%esi
  26df27:	call   111b17 <alloc::alloc::handle_alloc_error>
  26df2c:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<rexx_exec::Interp>::begin_method:
  26df31:	mov    %rdx,%r15
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  26df34:	mov    %rcx,%rax
  26df37:	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  26df3b:	mov    %eax,%edx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26df3d:	cmp    %rdx,0xa08(%r14)
  26df44:	jbe    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
<rexx_exec::Interp>::package_data:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/package_state.rs:171
  26df4a:	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  26df51:	shr    $0x22,%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26df55:	lea    (%rdx,%rdx,2),%rdx
  26df59:	shl    $0x5,%rdx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  26df5d:	cmp    %ecx,0x4(%rax,%rdx,1)
  26df61:	jne    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
  26df67:	add    %rdx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  26df6a:	cmpl   $0x1,(%rax)
  26df6d:	jne    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
<rexx_exec::Interp>::package_data:
  26df73:	mov    0x470(%r14),%rbx
<rexx_core::body::Body>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1439
  26df7a:	movzbl 0x8(%rax),%ecx
  26df7e:	cmp    $0x5,%ecx
  26df81:	je     26e150 <<rexx_exec::Interp>::begin_invoke+0x1e10>
  26df87:	cmp    $0x4,%ecx
  26df8a:	jne    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
  26df90:	mov    0x38(%rax),%rax
  26df94:	test   %rax,%rax
  26df97:	je     26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
<rexx_core::body::NativeState>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:433
  26df9d:	movabs $0x8000000000000000,%rcx
  26dfa7:	add    $0x4,%rcx
  26dfab:	cmp    %rcx,(%rax)
  26dfae:	jne    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:434
  26dfb4:	add    $0x8,%rax
  26dfb8:	jmp    26e15d <<rexx_exec::Interp>::begin_invoke+0x1e1d>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  26dfbd:	mov    $0x8,%edi
  26dfc2:	mov    $0x70,%esi
  26dfc7:	call   111b17 <alloc::alloc::handle_alloc_error>
  26dfcc:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
  26dfd1:	mov    $0x8,%edi
  26dfd6:	mov    $0x18,%esi
  26dfdb:	call   111b17 <alloc::alloc::handle_alloc_error>
  26dfe0:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<rexx_exec::Interp>::serve_posted_halt:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:2665
  26dfe5:	mov    %r14,%rdi
  26dfe8:	call   26f610 <<rexx_exec::Interp>::serve_halt_now>
  26dfed:	jmp    26ce71 <<rexx_exec::Interp>::begin_invoke+0xb31>
<rexx_exec::Interp>::begin_invoke:
  26dff2:	lea    0x100(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:41
  26dffa:	call   1cede0 <<rexx_exec::Loud>::missing_body>
  26dfff:	lea    0x330(%rsp),%rdi
  26e007:	lea    0x100(%rsp),%rsi
<rexx_exec::Loud as core::convert::Into<rexx_exec::error::Failure>>::into:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/convert/mod.rs:781
  26e00f:	call   1bf3c0 <<rexx_exec::error::Failure as core::convert::From<rexx_exec::Loud>>::from>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:41
  26e014:	mov    0x330(%rsp),%r14
  26e01c:	mov    0x338(%rsp),%rbx
  26e024:	jmp    26d1a1 <<rexx_exec::Interp>::begin_invoke+0xe61>
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::dispatch::NativeStarted, rexx_exec::dispatch::NativeStarted::Ran>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:837
  26e029:	movdqu 0x108(%rsp),%xmm0
  26e032:	xor    %eax,%eax
  26e034:	mov    0x8(%rsp),%r15
  26e039:	jmp    26cf9a <<rexx_exec::Interp>::begin_invoke+0xc5a>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  26e03e:	mov    $0x8,%edi
  26e043:	mov    $0x18,%esi
  26e048:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e04d:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
  26e04f:	mov    $0x8,%edi
  26e054:	mov    $0x70,%esi
  26e059:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e05e:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<rexx_exec::Interp>::sends_guarded:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/executable.rs:85
  26e060:	movzbl 0x50(%rsp),%edx
  26e065:	mov    %r14,%rdi
  26e068:	mov    %rsi,0x38(%rsp)
  26e06d:	mov    0x80(%rsp),%esi
  26e074:	call   26e820 <<rexx_exec::Interp>::written_guard>
  26e079:	mov    0x38(%rsp),%rsi
  26e07e:	mov    %rax,0x50(%rsp)
  26e083:	jmp    26cfde <<rexx_exec::Interp>::begin_invoke+0xc9e>
<rexx_exec::Interp>::begin_method:
  26e088:	xor    %eax,%eax
<rexx_exec::activity::Activity>::invocation_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:434
  26e08a:	test   %al,%al
  26e08c:	jne    26d655 <<rexx_exec::Interp>::begin_invoke+0x1315>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:436
  26e092:	mov    %r14,%rdi
  26e095:	mov    0x540(%rsp),%rsi
  26e09d:	mov    0x548(%rsp),%rdx
  26e0a5:	call   25e0c0 <<rexx_exec::activity::Activity>::new_invocation_name>
  26e0aa:	mov    %rdx,0x78(%rsp)
<rexx_exec::Interp>::begin_method:
  26e0af:	mov    %rax,%rdx
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e0b2:	incq   (%rdx)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26e0b5:	jne    26d699 <<rexx_exec::Interp>::begin_invoke+0x1359>
<rexx_exec::Interp>::begin_method:
  26e0bb:	ud2
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  26e0bd:	lea    -0x20426a(%rip),%rdi        # 69e5a <panic_unwind::imp::CANARY+0x50fd>
  26e0c4:	lea    0x2520dd(%rip),%rdx        # 4c01a8 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  26e0cb:	mov    $0x11,%esi
  26e0d0:	call   11f010 <core::option::expect_failed>
  26e0d5:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  26e0d7:	lea    -0x203460(%rip),%rdi        # 6ac7e <panic_unwind::imp::CANARY+0x5f21>
  26e0de:	lea    0x2527e3(%rip),%rdx        # 4c08c8 <rexx_api::ffi::CALL_CONTEXT+0x5538>
  26e0e5:	mov    $0x31,%esi
  26e0ea:	call   11c7f0 <core::panicking::panic>
  26e0ef:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<rexx_exec::Interp>::begin_invoke:
  26e0f1:	mov    %r12,%rsi
<alloc::boxed::Box<[u8]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:847
  26e0f4:	mov    $0x1,%edi
  26e0f9:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e0fe:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
core::mem::size_of_val::<[core::option::Option<rexx_core::handle::ObjRef>]>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:412
  26e100:	shl    $0x4,%rbp
<alloc::boxed::Box<[core::option::Option<rexx_core::handle::ObjRef>]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:847
  26e104:	mov    $0x8,%edi
  26e109:	mov    %rbp,%rsi
  26e10c:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e111:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:311
  26e113:	lea    0xab0(%r14),%rdi
  26e11a:	mov    0x48(%rsp),%rsi
  26e11f:	mov    %r13,%rdx
  26e122:	mov    %r15,%rcx
  26e125:	mov    0x20(%rsp),%r8
  26e12a:	call   1607c0 <<rexx_core::roots::RootSet>::set_frame_slot_of>
  26e12f:	jmp    26dc1a <<rexx_exec::Interp>::begin_invoke+0x18da>
  26e134:	lea    0xab0(%r14),%rdi
  26e13b:	mov    0x48(%rsp),%rsi
  26e140:	mov    %r13,%rdx
  26e143:	mov    %rbx,%rcx
  26e146:	call   1607c0 <<rexx_core::roots::RootSet>::set_frame_slot_of>
  26e14b:	jmp    26dcbe <<rexx_exec::Interp>::begin_invoke+0x197e>
<rexx_core::body::Body>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1440
  26e150:	mov    0x10(%rax),%rax
<rexx_core::body::NativeObject>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1350
  26e154:	cmpl   $0x5,(%rax)
  26e157:	jne    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1351
  26e159:	mov    0x8(%rax),%rax
<rexx_core::body::PackageState>::downcast_ref::<rexx_exec::package_state::PackageData>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:229
  26e15d:	mov    (%rax),%rsi
  26e160:	mov    0x8(%rax),%rax
  26e164:	lea    0x100(%rsp),%rdi
  26e16c:	mov    %rsi,0x28(%rsp)
<dyn core::any::Any>::is::<rexx_exec::package_state::PackageData>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/any.rs:201
  26e171:	call   *0x18(%rax)
core::intrinsics::type_id_eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:3128
  26e174:	movdqa 0x100(%rsp),%xmm0
  26e17d:	pcmpeqb -0x21de25(%rip),%xmm0        # 50360 <GCC_except_table5014+0x97c>
  26e185:	pmovmskb %xmm0,%eax
  26e189:	cmp    $0xffff,%eax
<rexx_exec::Interp>::package_data:
  26e18e:	jne    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
<rexx_exec::Interp>::options_of:
  26e190:	mov    0x28(%rsp),%rax
<core::option::Option<rexx_exec::options::PackageOptions>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26e195:	cmpb   $0x2,0xcd(%rax)
<rexx_exec::Interp>::options_of:
  26e19c:	jne    26e1fc <<rexx_exec::Interp>::begin_invoke+0x1ebc>
<rexx_exec::Interp>::begin_method:
  26e19e:	mov    $0x2,%dil
  26e1a1:	jmp    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
  26e1a3:	mov    0x10(%rsp),%rax
  26e1a8:	mov    %r15,%rdx
  26e1ab:	jmp    26d768 <<rexx_exec::Interp>::begin_invoke+0x1428>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:137
  26e1b0:	mov    %r14,%rdi
  26e1b3:	mov    0x20(%rsp),%rsi
  26e1b8:	mov    0x30(%rsp),%rdx
  26e1bd:	call   26eaa0 <<rexx_exec::Interp>::reserve_for_method>
  26e1c2:	jmp    26db88 <<rexx_exec::Interp>::begin_invoke+0x1848>
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:40
  26e1c7:	mov    %r14,%rdi
  26e1ca:	mov    0x48(%rsp),%rsi
  26e1cf:	mov    %r13,%rdx
  26e1d2:	mov    %r15,%rcx
  26e1d5:	mov    0x20(%rsp),%r8
  26e1da:	call   220300 <<rexx_exec::Interp>::set_exposed_variable>
  26e1df:	jmp    26dc1a <<rexx_exec::Interp>::begin_invoke+0x18da>
  26e1e4:	mov    %r14,%rdi
  26e1e7:	mov    0x48(%rsp),%rsi
  26e1ec:	mov    %r13,%rdx
  26e1ef:	mov    %rbx,%rcx
  26e1f2:	call   220300 <<rexx_exec::Interp>::set_exposed_variable>
  26e1f7:	jmp    26dcbe <<rexx_exec::Interp>::begin_invoke+0x197e>
<rexx_exec::Interp>::begin_method:
  26e1fc:	mov    0x28(%rsp),%rax
  26e201:	add    $0x98,%rax
<core::option::Option<&rexx_exec::activation::Activation>>::map::<&rexx_num::settings::Settings, <rexx_exec::Interp>::begin_method::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  26e207:	test   %rbx,%rbx
<rexx_exec::Interp>::package_start:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2700
  26e20a:	je     26e220 <<rexx_exec::Interp>::begin_invoke+0x1ee0>
  26e20c:	add    $0x68,%rbx
  26e210:	mov    0x28(%rsp),%rcx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2701
  26e215:	cmpb   $0x0,0xcc(%rcx)
  26e21c:	cmovne %rbx,%rax
<rexx_num::settings::Settings as core::clone::Clone>::clone:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-num/src/settings.rs:106
  26e220:	mov    0x10(%rax),%rcx
  26e224:	mov    %rcx,0xb0(%rsp)
  26e22c:	movdqu (%rax),%xmm0
  26e230:	movdqa %xmm0,0xa0(%rsp)
  26e239:	mov    0x28(%rsp),%rcx
<rexx_exec::Interp>::package_start:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2704
  26e23e:	movzbl 0xc6(%rcx),%edi
  26e245:	movzbl 0xcb(%rcx),%eax
  26e24c:	mov    %al,0x5c(%rsp)
  26e250:	mov    0xc7(%rcx),%eax
  26e256:	mov    %eax,0x58(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2705
  26e25a:	movzbl 0xb0(%rcx),%ebx
  26e261:	mov    0xb1(%rcx),%rax
  26e268:	mov    %rax,0x28(%rsp)
  26e26d:	jmp    26e1a3 <<rexx_exec::Interp>::begin_invoke+0x1e63>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  26e272:	mov    $0x1,%edi
  26e277:	mov    $0x2,%esi
  26e27c:	call   111b04 <alloc::raw_vec::handle_error>
  26e281:	jmp    26e0bb <<rexx_exec::Interp>::begin_invoke+0x1d7b>
<rexx_exec::Interp>::begin_method:
  26e286:	mov    %rax,%r14
  26e289:	lea    0x300(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:179
  26e291:	call   258b80 <core::ptr::drop_glue::<rexx_exec::CallContext>>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e296:	decq   (%r15)
<alloc::rc::Rc<[u8]> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e299:	jne    26e312 <<rexx_exec::Interp>::begin_invoke+0x1fd2>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e29b:	mov    %r15,%rdi
  26e29e:	mov    0x78(%rsp),%rsi
  26e2a3:	call   1660e0 <<alloc::rc::Rc<[u8]>>::drop_slow>
  26e2a8:	jmp    26e312 <<rexx_exec::Interp>::begin_invoke+0x1fd2>
<rexx_exec::Interp>::begin_method:
  26e2aa:	mov    %rax,%r14
  26e2ad:	mov    0x98(%rsp),%rax
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e2b5:	decq   (%rax)
<alloc::rc::Rc<[core::option::Option<rexx_core::handle::ObjRef>]> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e2b8:	jne    26e312 <<rexx_exec::Interp>::begin_invoke+0x1fd2>
  26e2ba:	mov    0x98(%rsp),%rdi
  26e2c2:	mov    0x28(%rsp),%rsi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e2c7:	call   1848b0 <<alloc::rc::Rc<[core::option::Option<rexx_core::handle::ObjRef>]>>::drop_slow>
  26e2cc:	jmp    26e312 <<rexx_exec::Interp>::begin_invoke+0x1fd2>
<rexx_exec::Interp>::begin_invoke:
  26e2ce:	jmp    26e2d0 <<rexx_exec::Interp>::begin_invoke+0x1f90>
  26e2d0:	mov    %rax,%r14
  26e2d3:	lea    0x330(%rsp),%rdi
  26e2db:	call   2eb2d0 <core::ptr::drop_glue::<core::result::Result<core::option::Option<rexx_exec::clause::HandlerExit>, rexx_exec::error::Failure>>>
  26e2e0:	jmp    26e39d <<rexx_exec::Interp>::begin_invoke+0x205d>
  26e2e5:	mov    %rax,%r14
  26e2e8:	lea    0x330(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:179
  26e2f0:	call   1a3570 <core::ptr::drop_glue::<rexx_exec::activation::Activation>>
  26e2f5:	jmp    26e390 <<rexx_exec::Interp>::begin_invoke+0x2050>
  26e2fa:	mov    %rax,%r14
  26e2fd:	lea    0x100(%rsp),%rdi
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1042
  26e305:	call   25e190 <core::ptr::drop_glue::<rexx_exec::run::call::CallTail>>
  26e30a:	jmp    26e39d <<rexx_exec::Interp>::begin_invoke+0x205d>
<rexx_exec::Interp>::begin_method:
  26e30f:	mov    %rax,%r14
  26e312:	mov    0x38(%rsp),%rax
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e317:	decq   (%rax)
<alloc::rc::Rc<rexx_exec::plan::Plan> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e31a:	jne    26e3f7 <<rexx_exec::Interp>::begin_invoke+0x20b7>
  26e320:	mov    0x38(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e325:	call   168c30 <<alloc::rc::Rc<rexx_exec::plan::Plan>>::drop_slow>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  26e32a:	jmp    26e3f7 <<rexx_exec::Interp>::begin_invoke+0x20b7>
<rexx_exec::Interp>::begin_invoke:
  26e32f:	jmp    26e331 <<rexx_exec::Interp>::begin_invoke+0x1ff1>
  26e331:	mov    %rax,%r14
  26e334:	jmp    26e3f7 <<rexx_exec::Interp>::begin_invoke+0x20b7>
  26e339:	mov    %rax,%r14
  26e33c:	jmp    26e42e <<rexx_exec::Interp>::begin_invoke+0x20ee>
  26e341:	jmp    26e388 <<rexx_exec::Interp>::begin_invoke+0x2048>
  26e343:	mov    %rax,%r14
  26e346:	jmp    26e35f <<rexx_exec::Interp>::begin_invoke+0x201f>
  26e348:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e34b:	cmpq   $0x0,0xa0(%rsp)
  26e354:	je     26e35f <<rexx_exec::Interp>::begin_invoke+0x201f>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e356:	mov    %rbx,%rdi
  26e359:	call   *0x269c39(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2800
  26e35f:	lea    0x68(%rsp),%rdi
  26e364:	call   1ac670 <core::ptr::drop_glue::<rexx_exec::error::Failure>>
  26e369:	jmp    26e39d <<rexx_exec::Interp>::begin_invoke+0x205d>
  26e36b:	mov    %rax,%r14
<alloc::boxed::Box<[u8]> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:1999
  26e36e:	test   %r12,%r12
  26e371:	je     26e470 <<rexx_exec::Interp>::begin_invoke+0x2130>
<rexx_exec::Interp>::begin_invoke:
  26e377:	mov    %rbx,%rdi
  26e37a:	call   *0x269c18(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  26e380:	mov    %r14,%rdi
  26e383:	call   4b1640 <_Unwind_Resume@plt>
  26e388:	mov    %rax,%r14
  26e38b:	jmp    26e39d <<rexx_exec::Interp>::begin_invoke+0x205d>
  26e38d:	mov    %rax,%r14
  26e390:	lea    0x300(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:179
  26e398:	call   258b80 <core::ptr::drop_glue::<rexx_exec::CallContext>>
core::ptr::drop_glue::<rexx_exec::dispatch::Invocable>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  26e39d:	cmpq   $0x5,0x40(%rsp)
  26e3a3:	jb     26e470 <<rexx_exec::Interp>::begin_invoke+0x2130>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2808
  26e3a9:	mov    0x2f8(%rsp),%rdi
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e3b1:	decq   (%rdi)
<alloc::rc::Rc<rexx_api::load::Library> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e3b4:	jne    26e470 <<rexx_exec::Interp>::begin_invoke+0x2130>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e3ba:	call   162d60 <<alloc::rc::Rc<rexx_api::load::Library>>::drop_slow>
  26e3bf:	jmp    26e470 <<rexx_exec::Interp>::begin_invoke+0x2130>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2720
  26e3c4:	call   11f113 <core::panicking::panic_in_cleanup>
  26e3c9:	mov    %rax,%r14
  26e3cc:	lea    0x100(%rsp),%rdi
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e3d4:	call   16ce90 <core::ptr::drop_glue::<rexx_exec::error::Raised>>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  26e3d9:	jmp    26e3f7 <<rexx_exec::Interp>::begin_invoke+0x20b7>
<rexx_exec::Interp>::begin_method:
  26e3db:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e3de:	cmpq   $0x0,0x100(%rsp)
  26e3e7:	je     26e3f7 <<rexx_exec::Interp>::begin_invoke+0x20b7>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e3e9:	mov    0x108(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e3f1:	call   *0x269ba1(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_method:
  26e3f7:	mov    0x18(%rsp),%rax
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e3fc:	decq   (%rax)
<alloc::rc::Rc<rexx_parse::Program> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e3ff:	jne    26e470 <<rexx_exec::Interp>::begin_invoke+0x2130>
  26e401:	mov    0x18(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e406:	call   191f90 <<alloc::rc::Rc<rexx_parse::Program>>::drop_slow>
  26e40b:	jmp    26e470 <<rexx_exec::Interp>::begin_invoke+0x2130>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:23
  26e40d:	call   11f113 <core::panicking::panic_in_cleanup>
<rexx_exec::Interp>::begin_invoke:
  26e412:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e415:	cmpq   $0x0,0x100(%rsp)
  26e41e:	je     26e42e <<rexx_exec::Interp>::begin_invoke+0x20ee>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e420:	mov    0x108(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e428:	call   *0x269b6a(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e42e:	cmpq   $0x0,0x330(%rsp)
  26e437:	je     26e470 <<rexx_exec::Interp>::begin_invoke+0x2130>
<rexx_exec::Interp>::begin_invoke:
  26e439:	mov    %rbx,%rdi
  26e43c:	call   *0x269b56(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  26e442:	mov    %r14,%rdi
  26e445:	call   4b1640 <_Unwind_Resume@plt>
  26e44a:	mov    %rax,%r14
  26e44d:	lea    0x100(%rsp),%rdi
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e455:	call   16ce90 <core::ptr::drop_glue::<rexx_exec::error::Raised>>
  26e45a:	mov    %r14,%rdi
  26e45d:	call   4b1640 <_Unwind_Resume@plt>
<rexx_exec::Interp>::begin_invoke:
  26e462:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e465:	cmpq   $0x0,0x100(%rsp)
  26e46e:	jne    26e478 <<rexx_exec::Interp>::begin_invoke+0x2138>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26e470:	mov    %r14,%rdi
  26e473:	call   4b1640 <_Unwind_Resume@plt>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e478:	mov    0x108(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e480:	call   *0x269b12(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  26e486:	mov    %r14,%rdi
  26e489:	call   4b1640 <_Unwind_Resume@plt>

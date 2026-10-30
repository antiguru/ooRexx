
/tmp/codex-p9d-t88-target/release/rexx:     file format elf64-x86-64


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
  26c36c:	je     26dee2 <<rexx_exec::Interp>::begin_invoke+0x1ba2>
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
  26c391:	mov    %r12,0x80(%rsp)
  26c399:	mov    %r15d,0x70(%rsp)
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
  26c3ce:	jne    26c6d6 <<rexx_exec::Interp>::begin_invoke+0x396>
  26c3d4:	jmp    26cc33 <<rexx_exec::Interp>::begin_invoke+0x8f3>
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
  26c455:	je     26c6b8 <<rexx_exec::Interp>::begin_invoke+0x378>
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
  26c5a9:	je     26c6e7 <<rexx_exec::Interp>::begin_invoke+0x3a7>
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
  26c5d7:	mov    %rbx,0x10(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2679
  26c5dc:	lea    (%rdx,%r9,8),%rax
<core::option::Option<&rexx_exec::InstalledMethodBody>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26c5e0:	mov    -0x10(%rax),%r13
  26c5e4:	mov    -0x8(%rax),%r12
  26c5e8:	mov    $0x2,%eax
  26c5ed:	jmp    26c7b9 <<rexx_exec::Interp>::begin_invoke+0x479>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, &rexx_exec::dispatch::native::NativeExternal)>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1337
  26c5f2:	cmpq   $0x0,0x1cf8(%r14)
<hashbrown::map::HashMap<rexx_core::body::MethodId, &rexx_exec::dispatch::native::NativeExternal, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/rust/deps/hashbrown-0.17.1/src/map.rs:1284
  26c5fa:	je     26c702 <<rexx_exec::Interp>::begin_invoke+0x3c2>
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
  26c650:	je     26c69d <<rexx_exec::Interp>::begin_invoke+0x35d>
  26c652:	mov    %rbx,0x10(%rsp)
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
  26c670:	je     26c7b0 <<rexx_exec::Interp>::begin_invoke+0x470>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26c676:	lea    -0x1(%r8),%r9d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c67a:	and    %r8w,%r9w
  26c67e:	mov    %r9d,%r8d
  26c681:	mov    0x10(%rsp),%rbx
  26c686:	mov    0x548(%rsp),%r12
  26c68e:	mov    0x70(%rsp),%r15d
  26c693:	mov    0x540(%rsp),%r13
  26c69b:	jne    26c657 <<rexx_exec::Interp>::begin_invoke+0x317>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c69d:	pcmpeqb %xmm1,%xmm2
  26c6a1:	pmovmskb %xmm2,%r8d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c6a6:	test   %r8d,%r8d
  26c6a9:	jne    26c702 <<rexx_exec::Interp>::begin_invoke+0x3c2>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26c6ab:	add    %rdi,%rcx
  26c6ae:	add    $0x10,%rcx
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26c6b2:	add    $0x10,%rdi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26c6b6:	jmp    26c638 <<rexx_exec::Interp>::begin_invoke+0x2f8>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry)>>::get::<hashbrown::map::equivalent_key<rexx_core::body::MethodId, rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry>::{closure#0}>:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1217
  26c6b8:	lea    (%rcx,%r8,8),%rax
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2676
  26c6bc:	mov    -0x20(%rax),%rcx
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry)>>::get::<hashbrown::map::equivalent_key<rexx_core::body::MethodId, rexx_core::body::MethodId, rexx_exec::dispatch::NativeEntry>::{closure#0}>:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1217
  26c6c0:	add    $0xffffffffffffffe0,%rax
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2676
  26c6c4:	mov    0x8(%rax),%r13
  26c6c8:	mov    0x10(%rax),%r12
<core::result::Result<rexx_exec::dispatch::Invocable, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26c6cc:	cmp    $0xffffffffffffffff,%rcx
  26c6d0:	je     26cc33 <<rexx_exec::Interp>::begin_invoke+0x8f3>
<rexx_exec::Interp>::begin_invoke:
  26c6d6:	mov    %rbx,0x10(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2728
  26c6db:	mov    0x18(%rax),%rbx
  26c6df:	mov    %rcx,%rax
<core::result::Result<rexx_exec::dispatch::Invocable, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26c6e2:	jmp    26c7b9 <<rexx_exec::Interp>::begin_invoke+0x479>
<rexx_exec::Interp>::begin_invoke:
  26c6e7:	mov    %rbx,0x10(%rsp)
<core::option::Option<&rexx_exec::GeneratedMethod>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26c6ec:	mov    -0x18(%r9),%r13
  26c6f0:	mov    -0x10(%r9),%r12
  26c6f4:	mov    -0x8(%r9),%rbx
  26c6f8:	mov    $0x3,%eax
<rexx_exec::Interp>::invocable:
  26c6fd:	jmp    26c7b9 <<rexx_exec::Interp>::begin_invoke+0x479>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_exec::LibraryBinding)>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1337
  26c702:	cmpq   $0x0,0x1de0(%r14)
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_exec::LibraryBinding, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/rust/deps/hashbrown-0.17.1/src/map.rs:1284
  26c70a:	je     26cba5 <<rexx_exec::Interp>::begin_invoke+0x865>
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26c710:	imul   %rbp,%rax
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26c714:	rol    $0x1a,%rax
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26c718:	mov    %rax,%rsi
  26c71b:	shr    $0x39,%rsi
  26c71f:	mov    0x1dc8(%r14),%rcx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26c726:	mov    0x1dd0(%r14),%rdx
  26c72d:	movd   %esi,%xmm0
  26c731:	punpcklbw %xmm0,%xmm0
  26c735:	pshuflw $0x0,%xmm0,%xmm0
  26c73a:	pshufd $0x44,%xmm0,%xmm0
  26c73f:	xor    %esi,%esi
  26c741:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26c745:	and    %rdx,%rax
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26c748:	movdqu (%rcx,%rax,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c74d:	movdqa %xmm2,%xmm3
  26c751:	pcmpeqb %xmm0,%xmm3
  26c755:	pmovmskb %xmm3,%edi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c759:	test   %edi,%edi
  26c75b:	je     26c793 <<rexx_exec::Interp>::begin_invoke+0x453>
  26c75d:	mov    %rbx,0x10(%rsp)
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26c762:	tzcnt  %edi,%r9d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26c767:	add    %rax,%r9
  26c76a:	and    %rdx,%r9
<*mut (rexx_core::body::MethodId, rexx_exec::LibraryBinding)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26c76d:	shl    $0x5,%r9
  26c771:	mov    %rcx,%r8
  26c774:	sub    %r9,%r8
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26c777:	cmp    -0x20(%r8),%r15d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c77b:	je     26cc47 <<rexx_exec::Interp>::begin_invoke+0x907>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26c781:	lea    -0x1(%rdi),%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26c785:	and    %di,%r8w
  26c789:	mov    %r8d,%edi
  26c78c:	mov    0x10(%rsp),%rbx
  26c791:	jne    26c762 <<rexx_exec::Interp>::begin_invoke+0x422>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26c793:	pcmpeqb %xmm1,%xmm2
  26c797:	pmovmskb %xmm2,%edi
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26c79b:	test   %edi,%edi
  26c79d:	jne    26cba5 <<rexx_exec::Interp>::begin_invoke+0x865>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26c7a3:	add    %rsi,%rax
  26c7a6:	add    $0x10,%rax
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26c7aa:	add    $0x10,%rsi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26c7ae:	jmp    26c745 <<rexx_exec::Interp>::begin_invoke+0x405>
<core::option::Option<&&rexx_exec::dispatch::native::NativeExternal>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26c7b0:	mov    -0x8(%r9),%r13
  26c7b4:	mov    $0x4,%eax
<rexx_exec::Interp>::begin_invoke:
  26c7b9:	mov    0x558(%rsp),%r10
  26c7c1:	mov    %rax,0x40(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2728
  26c7c6:	mov    %rax,0x2e0(%rsp)
  26c7ce:	mov    %r13,0x2e8(%rsp)
  26c7d6:	mov    %r12,0x2f0(%rsp)
  26c7de:	mov    %rbx,0x2f8(%rsp)
rexx_exec::dispatch::seam::clear:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:66
  26c7e6:	cmpb   $0x1,0x1ff4(%r14)
  26c7ee:	jne    26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:67
  26c7f0:	cmpb   $0x0,0x1ff5(%r14)
core::mem::replace::<bool>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26c7f8:	movb   $0x0,0x1ff5(%r14)
rexx_exec::dispatch::seam::clear:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:67
  26c800:	je     26ca04 <<rexx_exec::Interp>::begin_invoke+0x6c4>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2735
  26c806:	cmpq   $0x2,0x40(%rsp)
  26c80c:	jne    26c8d1 <<rexx_exec::Interp>::begin_invoke+0x591>
<rexx_exec::plan::Programs>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:135
  26c812:	mov    %r13d,%ebx
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::ProgramSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26c815:	cmp    0xc58(%r14),%rbx
  26c81c:	jae    26c9a8 <<rexx_exec::Interp>::begin_invoke+0x668>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:33
  26c822:	mov    0xc50(%r14),%rax
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::ProgramSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26c829:	mov    %rbx,%r15
  26c82c:	shl    $0x4,%r15
<rexx_exec::plan::ProgramId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:32
  26c830:	cmp    %r13,(%rax,%r15,1)
<rexx_exec::plan::Programs>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:136
  26c834:	jne    26c9a8 <<rexx_exec::Interp>::begin_invoke+0x668>
  26c83a:	add    %r15,%rax
<core::option::Option<alloc::rc::Rc<rexx_parse::Program>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26c83d:	mov    0x8(%rax),%rcx
  26c841:	test   %rcx,%rcx
  26c844:	je     26c9a8 <<rexx_exec::Interp>::begin_invoke+0x668>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26c84a:	incq   (%rcx)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26c84d:	je     26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<alloc::rc::Rc<rexx_parse::Program> as core::clone::Clone>::clone:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2527
  26c853:	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[rexx_parse::ast::Directive]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26c857:	cmp    0xa0(%rax),%r12
  26c85e:	mov    %rax,0x18(%rsp)
  26c863:	jae    26dfeb <<rexx_exec::Interp>::begin_invoke+0x1cab>
<rexx_exec::Interp>::begin_method:
  26c869:	movabs $0x8000000000000000,%rsi
<alloc::raw_vec::RawVecInner>::non_null::<rexx_parse::ast::Directive>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26c873:	mov    0x98(%rax),%rcx
<usize as core::slice::index::SliceIndex<[rexx_parse::ast::Directive]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26c87a:	lea    (%r12,%r12,4),%rdx
  26c87e:	lea    (%rcx,%rdx,8),%rax
rexx_exec::activation::body_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1127
  26c882:	mov    (%rcx,%rdx,8),%rcx
  26c886:	mov    %rcx,%rdx
  26c889:	xor    %rsi,%rdx
  26c88c:	test   %rcx,%rcx
  26c88f:	mov    $0x5,%ecx
  26c894:	cmovs  %rdx,%rcx
  26c898:	cmp    $0x8,%rcx
  26c89c:	je     26cfa0 <<rexx_exec::Interp>::begin_invoke+0xc60>
  26c8a2:	cmp    $0x4,%rcx
  26c8a6:	je     26c8b2 <<rexx_exec::Interp>::begin_invoke+0x572>
  26c8a8:	cmp    $0x1,%rcx
  26c8ac:	jne    26d145 <<rexx_exec::Interp>::begin_invoke+0xe05>
  26c8b2:	mov    0x8(%rax),%rsi
<core::option::Option<rexx_parse::ast::CodeBody>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26c8b6:	cmpq   $0xffffffffffffffff,(%rsi)
  26c8ba:	je     26d145 <<rexx_exec::Interp>::begin_invoke+0xe05>
<rexx_parse::ast::GuardOption as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/ast.rs:1102
  26c8c0:	cmpb   $0x2,0x74(%rsi)
  26c8c4:	setne  %al
  26c8c7:	mov    %rax,0x48(%rsp)
rexx_exec::guards::directive_guarded:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/guards.rs:384
  26c8cc:	jmp    26cfb7 <<rexx_exec::Interp>::begin_invoke+0xc77>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2757
  26c8d1:	jae    26ca3d <<rexx_exec::Interp>::begin_invoke+0x6fd>
  26c8d7:	cmp    $0x2,%r12
  26c8db:	je     26ca3d <<rexx_exec::Interp>::begin_invoke+0x6fd>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2760
  26c8e1:	testb  $0x1,0x40(%rsp)
  26c8e6:	jne    26cb34 <<rexx_exec::Interp>::begin_invoke+0x7f4>
  26c8ec:	cmp    %r13,%r10
  26c8ef:	jbe    26cb34 <<rexx_exec::Interp>::begin_invoke+0x7f4>
  26c8f5:	mov    %r10,%rbp
  26c8f8:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2762
  26c900:	mov    %r13,%rsi
  26c903:	call   26efe0 <<rexx_exec::error::Raised>::too_many_method_arguments>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26c908:	mov    $0x70,%edi
  26c90d:	call   *0x26b6a5(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26c913:	test   %rax,%rax
  26c916:	je     26dfb6 <<rexx_exec::Interp>::begin_invoke+0x1c76>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26c91c:	movups 0x160(%rsp),%xmm0
  26c924:	movups %xmm0,0x60(%rax)
  26c928:	movups 0x150(%rsp),%xmm0
  26c930:	movups %xmm0,0x50(%rax)
  26c934:	movups 0x140(%rsp),%xmm0
  26c93c:	movups %xmm0,0x40(%rax)
  26c940:	movdqu 0x100(%rsp),%xmm0
  26c949:	movdqu 0x110(%rsp),%xmm1
  26c952:	movdqu 0x120(%rsp),%xmm2
  26c95b:	movdqu 0x130(%rsp),%xmm3
  26c964:	movdqu %xmm3,0x30(%rax)
  26c969:	movdqu %xmm2,0x20(%rax)
  26c96e:	movdqu %xmm1,0x10(%rax)
  26c973:	movdqu %xmm0,(%rax)
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2762
  26c977:	movq   $0x3,0x60(%rsp)
  26c980:	mov    %rax,0x68(%rsp)
  26c985:	movq   $0x2,0x58(%rsp)
  26c98e:	mov    0x10(%rsp),%r15
  26c993:	mov    0x548(%rsp),%r12
  26c99b:	mov    0x540(%rsp),%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2766
  26c9a3:	jmp    26ce9a <<rexx_exec::Interp>::begin_invoke+0xb5a>
  26c9a8:	lea    0x100(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:34
  26c9b0:	call   1cede0 <<rexx_exec::Loud>::missing_body>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26c9b5:	mov    $0x18,%edi
  26c9ba:	call   *0x26b5f8(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26c9c0:	test   %rax,%rax
  26c9c3:	je     26df16 <<rexx_exec::Interp>::begin_invoke+0x1bd6>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26c9c9:	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26c9cc:	mov    0x110(%rsp),%rax
  26c9d4:	mov    %rax,0x10(%rbx)
  26c9d8:	movdqu 0x100(%rsp),%xmm0
  26c9e1:	movdqu %xmm0,(%rbx)
  26c9e5:	mov    $0x2,%r14d
<rexx_exec::Interp>::begin_invoke:
  26c9eb:	mov    0x10(%rsp),%rax
<core::result::Result<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  26c9f0:	mov    %r14,0x8(%rax)
  26c9f4:	mov    %rbx,0x10(%rax)
  26c9f8:	movq   $0x1,(%rax)
<rexx_exec::Interp>::begin_invoke:
  26c9ff:	jmp    26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::dispatch::AccessScope>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26ca04:	cmp    %rbp,0x1090(%r14)
  26ca0b:	jbe    26ca81 <<rexx_exec::Interp>::begin_invoke+0x741>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_exec::dispatch::AccessScope>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26ca0d:	mov    0x1088(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::dispatch::AccessScope>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26ca14:	lea    0x0(,%rbp,2),%rcx
  26ca1c:	add    %rbp,%rcx
<core::option::Option<rexx_exec::dispatch::AccessScope>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26ca1f:	cmpq   $0x2,(%rax,%rcx,8)
  26ca24:	je     26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
rexx_exec::dispatch::seam::clear:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:68
  26ca2a:	lea    (%rax,%rcx,8),%rax
  26ca2e:	testb  $0x1,0x10(%rax)
  26ca32:	je     26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
  26ca38:	jmp    26cc7c <<rexx_exec::Interp>::begin_invoke+0x93c>
<rexx_exec::Interp>::begin_invoke:
  26ca3d:	lea    0x2e0(%rsp),%rdx
  26ca45:	mov    0x10(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2804
  26ca4a:	mov    %r14,%rsi
  26ca4d:	mov    0x30(%rsp),%rcx
  26ca52:	mov    0x70(%rsp),%r8d
  26ca57:	mov    0x20(%rsp),%r9
  26ca5c:	push   %r10
  26ca5e:	push   0x558(%rsp)
  26ca65:	push   0x558(%rsp)
  26ca6c:	push   0x558(%rsp)
  26ca73:	call   26ebb0 <<rexx_exec::Interp>::begin_invoke_other>
  26ca78:	add    $0x20,%rsp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2808
  26ca7c:	jmp    26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_exec::dispatch::AccessScope, rustc_hash::FxBuildHasher>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/map.rs:825
  26ca81:	cmpq   $0x0,0x10b0(%r14)
<rexx_exec::method_rows::MethodRows<rexx_exec::dispatch::AccessScope>>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/method_rows.rs:33
  26ca89:	je     26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
<std::collections::hash::map::HashMap<rexx_core::body::MethodId, rexx_exec::dispatch::AccessScope, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/map.rs:1039
  26ca8f:	movabs $0xf1357aea2e62a9c5,%rax
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26ca99:	imul   %rbp,%rax
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26ca9d:	rol    $0x1a,%rax
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26caa1:	mov    %rax,%rsi
  26caa4:	shr    $0x39,%rsi
  26caa8:	mov    0x1098(%r14),%rcx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26caaf:	mov    0x10a0(%r14),%rdx
  26cab6:	movd   %esi,%xmm0
  26caba:	punpcklbw %xmm0,%xmm0
  26cabe:	pshuflw $0x0,%xmm0,%xmm0
  26cac3:	pshufd $0x44,%xmm0,%xmm0
  26cac8:	xor    %esi,%esi
  26caca:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26cace:	and    %rdx,%rax
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26cad1:	movdqu (%rcx,%rax,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26cad6:	movdqa %xmm2,%xmm3
  26cada:	pcmpeqb %xmm0,%xmm3
  26cade:	pmovmskb %xmm3,%edi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26cae2:	test   %edi,%edi
  26cae4:	je     26cb17 <<rexx_exec::Interp>::begin_invoke+0x7d7>
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26cae6:	tzcnt  %edi,%r9d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26caeb:	add    %rax,%r9
  26caee:	and    %rdx,%r9
<*mut (rexx_core::body::MethodId, rexx_exec::dispatch::AccessScope)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26caf1:	shl    $0x5,%r9
  26caf5:	mov    %rcx,%r8
  26caf8:	sub    %r9,%r8
  26cafb:	mov    0x70(%rsp),%r9d
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26cb00:	cmp    -0x20(%r8),%r9d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26cb04:	je     26cc66 <<rexx_exec::Interp>::begin_invoke+0x926>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26cb0a:	lea    -0x1(%rdi),%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26cb0e:	and    %di,%r8w
  26cb12:	mov    %r8d,%edi
  26cb15:	jne    26cae6 <<rexx_exec::Interp>::begin_invoke+0x7a6>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26cb17:	pcmpeqb %xmm1,%xmm2
  26cb1b:	pmovmskb %xmm2,%edi
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26cb1f:	test   %edi,%edi
  26cb21:	jne    26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26cb27:	add    %rsi,%rax
  26cb2a:	add    $0x10,%rax
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26cb2e:	add    $0x10,%rsi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26cb32:	jmp    26cace <<rexx_exec::Interp>::begin_invoke+0x78e>
<rexx_exec::Interp>::begin_invoke:
  26cb34:	mov    %r10,%rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2760
  26cb37:	test   %r12,%r12
  26cb3a:	mov    0x550(%rsp),%r13
  26cb42:	je     26ce04 <<rexx_exec::Interp>::begin_invoke+0xac4>
  26cb48:	cmp    $0x1,%r12
  26cb4c:	jne    26cd23 <<rexx_exec::Interp>::begin_invoke+0x9e3>
  26cb52:	lea    0x58(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2779
  26cb57:	mov    %r14,%rsi
  26cb5a:	mov    0x20(%rsp),%rdx
  26cb5f:	mov    %r13,%rcx
  26cb62:	mov    %rbp,%r8
  26cb65:	call   *%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2781
  26cb67:	mov    0x58(%rsp),%rax
  26cb6c:	cmp    $0x2,%rax
  26cb70:	mov    0x10(%rsp),%r15
  26cb75:	mov    0x548(%rsp),%r12
  26cb7d:	mov    0x540(%rsp),%r13
  26cb85:	je     26ce9a <<rexx_exec::Interp>::begin_invoke+0xb5a>
  26cb8b:	test   %rax,%rax
  26cb8e:	jne    26d1f9 <<rexx_exec::Interp>::begin_invoke+0xeb9>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2782
  26cb94:	movdqu 0x60(%rsp),%xmm0
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>::Ran>:
  26cb9a:	movdqu %xmm0,0x8(%r15)
  26cba0:	jmp    26d5ca <<rexx_exec::Interp>::begin_invoke+0x128a>
<rexx_exec::Interp>::begin_invoke:
  26cba5:	mov    %rbx,%r15
  26cba8:	lea    0x330(%rsp),%rdi
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2702
  26cbb0:	mov    %r14,%rsi
  26cbb3:	mov    0x30(%rsp),%rdx
  26cbb8:	call   1c8650 <<rexx_exec::Interp>::scope_id>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26cbbd:	mov    0x338(%rsp),%rbx
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  26cbc5:	mov    0x340(%rsp),%r8
  26cbcd:	lea    0x100(%rsp),%rdi
<rexx_exec::Interp>::invocable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2703
  26cbd5:	mov    %r13,%rsi
  26cbd8:	mov    %r12,%rdx
  26cbdb:	mov    %rbx,%rcx
  26cbde:	call   26e710 <<rexx_exec::Loud>::unbound_method>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26cbe3:	mov    $0x18,%edi
  26cbe8:	call   *0x26b3ca(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26cbee:	test   %rax,%rax
  26cbf1:	je     26dfca <<rexx_exec::Interp>::begin_invoke+0x1c8a>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26cbf7:	mov    %rax,%r12
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26cbfa:	mov    0x110(%rsp),%rax
  26cc02:	mov    %rax,0x10(%r12)
  26cc07:	movdqu 0x100(%rsp),%xmm0
  26cc10:	movdqu %xmm0,(%r12)
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26cc16:	cmpq   $0x0,0x330(%rsp)
  26cc1f:	je     26cc2a <<rexx_exec::Interp>::begin_invoke+0x8ea>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26cc21:	mov    %rbx,%rdi
  26cc24:	call   *0x26b36e(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_invoke:
  26cc2a:	mov    $0x2,%r13d
  26cc30:	mov    %r15,%rbx
<core::result::Result<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  26cc33:	mov    %r13,0x8(%rbx)
  26cc37:	mov    %r12,0x10(%rbx)
  26cc3b:	movq   $0x1,(%rbx)
<rexx_exec::Interp>::begin_invoke:
  26cc42:	jmp    26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
<core::ptr::non_null::NonNull<alloc::rc::RcInner<rexx_api::load::Library>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:454
  26cc47:	mov    -0x8(%r8),%rbx
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26cc4b:	incq   (%rbx)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26cc4e:	je     26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<core::option::Option<usize> as core::clone::Clone>::clone:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2274
  26cc54:	mov    -0x18(%r8),%r13
  26cc58:	mov    -0x10(%r8),%r12
  26cc5c:	mov    $0x5,%eax
<rexx_exec::Interp>::invocable:
  26cc61:	jmp    26c7b9 <<rexx_exec::Interp>::begin_invoke+0x479>
<core::option::Option<rexx_exec::dispatch::AccessScope>>::is_some_and::<<rexx_exec::Interp>::method_is_protected::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:660
  26cc66:	cmpq   $0x2,-0x18(%r8)
  26cc6b:	je     26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
  26cc71:	testb  $0x1,-0x8(%r8)
  26cc76:	je     26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
<rexx_exec::Interp>::effective_security_manager:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:115
  26cc7c:	mov    %r14,%rdi
  26cc7f:	call   26ae50 <<rexx_exec::Interp>::armed_security_manager>
<core::option::Option<rexx_core::handle::ObjRef>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/macros/mod.rs:434
  26cc84:	cmp    $0x1,%rax
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2494
  26cc88:	jne    26de86 <<rexx_exec::Interp>::begin_invoke+0x1b46>
  26cc8e:	mov    0x548(%rsp),%r15
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:131
  26cc96:	cmp    $0x7,%r15
  26cc9a:	jbe    26d1ba <<rexx_exec::Interp>::begin_invoke+0xe7a>
<rexx_exec::Interp>::text_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:230
  26cca0:	lea    0x108(%rsp),%rdi
<rexx_exec::Interp>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:96
  26cca8:	mov    0x540(%rsp),%rsi
  26ccb0:	mov    %r15,%rdx
  26ccb3:	call   15ae90 <<rexx_core::bytes::Bytes>::from_slice>
<rexx_exec::Interp>::text_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:230
  26ccb8:	movq   $0x0,0x140(%rsp)
  26ccc4:	movb   $0x0,0x100(%rsp)
  26cccc:	lea    0x100(%rsp),%rdx
  26ccd4:	mov    %r14,%rdi
  26ccd7:	xor    %esi,%esi
  26ccd9:	call   18c260 <<rexx_exec::Interp>::alloc_with>
  26ccde:	mov    %rax,0x18(%rsp)
<rexx_exec::Interp>::text:
  26cce3:	mov    0x548(%rsp),%rcx
<rexx_exec::Interp>::charge_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:104
  26cceb:	cmp    $0x37,%rcx
  26ccef:	jb     26d3d2 <<rexx_exec::Interp>::begin_invoke+0x1092>
<rexx_core::heap::Heap>::hold_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:183
  26ccf5:	add    %rcx,0xaa0(%r14)
  26ccfc:	mov    0xa90(%r14),%rax
<rexx_core::heap::Heap>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:176
  26cd03:	add    %rcx,%rax
  26cd06:	mov    %rax,0xa90(%r14)
<rexx_exec::Interp>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:3249
  26cd0d:	cmp    0x1fc8(%r14),%rax
  26cd14:	setae  %al
  26cd17:	or     %al,0x1fff(%r14)
  26cd1e:	jmp    26d3d2 <<rexx_exec::Interp>::begin_invoke+0x1092>
<core::ptr::non_null::NonNull<core::option::Option<rexx_core::handle::ObjRef>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  26cd23:	test   %rbp,%rbp
<core::slice::iter::Iter<core::option::Option<rexx_core::handle::ObjRef>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  26cd26:	je     26ce04 <<rexx_exec::Interp>::begin_invoke+0xac4>
  26cd2c:	mov    %rbp,%r15
  26cd2f:	shl    $0x4,%r15
  26cd33:	xor    %r12d,%r12d
  26cd36:	jmp    26cd4d <<rexx_exec::Interp>::begin_invoke+0xa0d>
<core::ptr::non_null::NonNull<core::option::Option<rexx_core::handle::ObjRef>> as core::cmp::PartialEq>::eq:
  26cd38:	nopl   0x0(%rax,%rax,1)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  26cd40:	add    $0x10,%r12
  26cd44:	cmp    %r12,%r15
<core::slice::iter::Iter<core::option::Option<rexx_core::handle::ObjRef>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  26cd47:	je     26ce04 <<rexx_exec::Interp>::begin_invoke+0xac4>
core::iter::adapters::flatten::try_flatten_one::<&core::option::Option<rexx_core::handle::ObjRef>, (), core::ops::control_flow::ControlFlow<()>, core::iter::traits::iterator::Iterator::any::check<&rexx_core::handle::ObjRef, <rexx_exec::Interp>::argument_may_run_rexx::{closure#0}>::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/flatten.rs:819
  26cd4d:	cmpl   $0x1,0x0(%r13,%r12,1)
  26cd53:	jne    26cd40 <<rexx_exec::Interp>::begin_invoke+0xa00>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/flatten.rs:820
  26cd55:	mov    0x8(%r13,%r12,1),%rsi
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  26cd5a:	test   $0x3,%sil
  26cd5e:	jne    26cd40 <<rexx_exec::Interp>::begin_invoke+0xa00>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  26cd60:	mov    %rsi,%rax
  26cd63:	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  26cd67:	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26cd69:	cmp    %rcx,0xa08(%r14)
  26cd70:	jbe    26cd9c <<rexx_exec::Interp>::begin_invoke+0xa5c>
<rexx_exec::Interp>::may_run_rexx:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:3907
  26cd72:	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  26cd79:	mov    %rsi,%rdx
  26cd7c:	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26cd80:	lea    (%rcx,%rcx,2),%rcx
  26cd84:	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  26cd88:	cmp    %edx,0x4(%rax,%rcx,1)
  26cd8c:	jne    26cd9c <<rexx_exec::Interp>::begin_invoke+0xa5c>
  26cd8e:	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  26cd91:	cmpl   $0x1,(%rax)
  26cd94:	jne    26cd9c <<rexx_exec::Interp>::begin_invoke+0xa5c>
<rexx_exec::Interp>::may_run_rexx:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:3906
  26cd96:	cmpb   $0x2,0x8(%rax)
  26cd9a:	jb     26cd40 <<rexx_exec::Interp>::begin_invoke+0xa00>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:3909
  26cd9c:	mov    %r14,%rdi
  26cd9f:	call   1dfe50 <<rexx_exec::Interp>::is_base_class>
<core::slice::iter::Iter<core::option::Option<rexx_core::handle::ObjRef>> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::adapters::flatten::try_flatten_one<&core::option::Option<rexx_core::handle::ObjRef>, (), core::ops::control_flow::ControlFlow<()>, core::iter::traits::iterator::Iterator::any::check<&rexx_core::handle::ObjRef, <rexx_exec::Interp>::argument_may_run_rexx::{closure#0}>::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<()>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  26cda4:	test   %al,%al
  26cda6:	jne    26cd40 <<rexx_exec::Interp>::begin_invoke+0xa00>
<rexx_exec::Interp>::begin_invoke:
  26cda8:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2765
  26cdb0:	mov    %r14,%rsi
  26cdb3:	mov    %rbx,%rdx
  26cdb6:	mov    0x30(%rsp),%rcx
  26cdbb:	mov    0x70(%rsp),%r8d
  26cdc0:	mov    0x20(%rsp),%r9
  26cdc5:	push   %rbp
  26cdc6:	push   %r13
  26cdc8:	push   0x558(%rsp)
  26cdcf:	push   0x558(%rsp)
  26cdd6:	call   26f350 <<rexx_exec::Interp>::run_leveled>
  26cddb:	add    $0x20,%rsp
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::dispatch::NativeStarted, rexx_exec::dispatch::NativeStarted::Ran>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:836
  26cddf:	cmpl   $0x1,0x100(%rsp)
  26cde7:	jne    26e022 <<rexx_exec::Interp>::begin_invoke+0x1ce2>
  26cded:	lea    0x108(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:838
  26cdf5:	movdqu (%rax),%xmm0
  26cdf9:	movdqu %xmm0,0x60(%rsp)
  26cdff:	jmp    26c985 <<rexx_exec::Interp>::begin_invoke+0x645>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/pinning.rs:33
  26ce04:	incl   0x520(%r14)
  26ce0b:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2774
  26ce13:	mov    %r14,%rsi
  26ce16:	mov    0x20(%rsp),%rdx
  26ce1b:	mov    %r13,%rcx
  26ce1e:	mov    %rbp,%r8
  26ce21:	call   *%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/pinning.rs:35
  26ce23:	decl   0x520(%r14)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/pinning.rs:36
  26ce2a:	mov    0x110(%rsp),%rax
  26ce32:	mov    %rax,0x340(%rsp)
  26ce3a:	movdqu 0x100(%rsp),%xmm0
  26ce43:	movdqa %xmm0,0x330(%rsp)
<core::ptr::non_null::NonNull<alloc::sync::ArcInner<rexx_exec::timer::Inbox<rexx_exec::scheduler::Posted>>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:454
  26ce4c:	mov    0x13d0(%r14),%rax
core::sync::atomic::atomic_load::<u32>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/sync/atomic.rs:3995
  26ce53:	mov    0x40(%rax),%eax
<rexx_exec::timer::Requests>::pending:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/timer.rs:79
  26ce56:	test   $0x4,%al
<rexx_exec::Interp>::serve_posted_halt:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:2664
  26ce58:	jne    26dfde <<rexx_exec::Interp>::begin_invoke+0x1c9e>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2777
  26ce5e:	movdqu 0x338(%rsp),%xmm0
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::dispatch::NativeStarted, rexx_exec::dispatch::NativeStarted::Ran>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:836
  26ce67:	cmpq   $0x1,0x330(%rsp)
  26ce70:	mov    0x10(%rsp),%r15
  26ce75:	mov    0x548(%rsp),%r12
  26ce7d:	mov    0x540(%rsp),%r13
  26ce85:	jne    26cf9c <<rexx_exec::Interp>::begin_invoke+0xc5c>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:838
  26ce8b:	movdqu %xmm0,0x60(%rsp)
  26ce91:	movq   $0x2,0x58(%rsp)
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2796
  26ce9a:	movdqu 0x60(%rsp),%xmm0
  26cea0:	movdqa %xmm0,0x70(%rsp)
  26cea6:	lea    0x90(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2797
  26ceae:	mov    %r14,%rsi
  26ceb1:	mov    0x30(%rsp),%rdx
  26ceb6:	call   1c8650 <<rexx_exec::Interp>::scope_id>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26cebb:	mov    0x98(%rsp),%rbx
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  26cec3:	mov    0xa0(%rsp),%r8
<rexx_exec::Interp>::blame_native_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:4045
  26cecb:	movb   $0x1,0x150(%rsp)
  26ced3:	mov    %r13,0x128(%rsp)
  26cedb:	mov    %r12,0x130(%rsp)
  26cee3:	mov    0x20(%rsp),%rax
  26cee8:	mov    %rax,0x148(%rsp)
  26cef0:	mov    0x550(%rsp),%rax
  26cef8:	mov    %rax,0x138(%rsp)
  26cf00:	mov    %rbp,0x140(%rsp)
  26cf08:	movq   $0x0,0x100(%rsp)
  26cf14:	movq   $0xffffffffffffffff,0x110(%rsp)
  26cf20:	lea    0x300(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:4053
  26cf28:	mov    %r13,%rsi
  26cf2b:	mov    %r12,%rdx
  26cf2e:	mov    %rbx,%rcx
  26cf31:	call   1c8800 <<rexx_exec::error::Raised>::compiled_method_line>
  26cf36:	movq   $0xffffffffffffffff,0x330(%rsp)
  26cf42:	lea    0x300(%rsp),%rsi
  26cf4a:	lea    0x330(%rsp),%rdx
  26cf52:	lea    0x100(%rsp),%rcx
  26cf5a:	mov    %r14,%rdi
  26cf5d:	call   1d7900 <<rexx_exec::Interp>::blame_native_level>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26cf62:	cmpq   $0x0,0x90(%rsp)
  26cf6b:	je     26cf76 <<rexx_exec::Interp>::begin_invoke+0xc36>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26cf6d:	mov    %rbx,%rdi
  26cf70:	call   *0x26b022(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_invoke:
  26cf76:	mov    $0x1,%eax
  26cf7b:	movdqa 0x70(%rsp),%xmm0
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>, rexx_exec::run::call::Started<core::option::Option<rexx_core::handle::ObjRef>>::Ran>:
  26cf81:	movdqu %xmm0,0x8(%r15)
  26cf87:	mov    %rax,(%r15)
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2808
  26cf8a:	add    $0x508,%rsp
  26cf91:	pop    %rbx
  26cf92:	pop    %r12
  26cf94:	pop    %r13
  26cf96:	pop    %r14
  26cf98:	pop    %r15
  26cf9a:	pop    %rbp
  26cf9b:	ret
  26cf9c:	xor    %eax,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2778
  26cf9e:	jmp    26cf81 <<rexx_exec::Interp>::begin_invoke+0xc41>
rexx_exec::activation::body_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1128
  26cfa0:	mov    0x8(%rax),%rsi
<core::option::Option<rexx_parse::ast::CodeBody>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26cfa4:	cmpq   $0xffffffffffffffff,(%rsi)
  26cfa8:	je     26d145 <<rexx_exec::Interp>::begin_invoke+0xe05>
<rexx_exec::Interp>::begin_method:
  26cfae:	movq   $0x0,0x48(%rsp)
<hashbrown::map::HashMap<rexx_core::handle::ObjRef, rexx_exec::dispatch::executable::MethodFlagWrites, rustc_hash::FxBuildHasher>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/map.rs:825
  26cfb7:	cmpq   $0x0,0x1c58(%r14)
<rexx_exec::Interp>::sends_guarded:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/executable.rs:82
  26cfbf:	jne    26e059 <<rexx_exec::Interp>::begin_invoke+0x1d19>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:54
  26cfc5:	movb   $0x0,0x58(%rsp)
<rexx_exec::Interp>::activation_depth:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1341
  26cfca:	cmpq   $0x1,0x470(%r14)
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:55
  26cfd2:	mov    0x1b8(%r14),%rax
<rexx_exec::Interp>::activation_depth:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1341
  26cfd9:	sbb    $0xffffffffffffffff,%rax
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:55
  26cfdd:	cmp    $0x270f,%rax
  26cfe3:	ja     26d00b <<rexx_exec::Interp>::begin_invoke+0xccb>
<rexx_exec::Interp>::stack_exhausted::<u8>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:1339
  26cfe5:	mov    0x1f58(%r14),%rax
  26cfec:	lea    0x58(%rsp),%rcx
<usize>::abs_diff:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:3181
  26cff1:	mov    %rax,%rdx
  26cff4:	sub    %rcx,%rdx
  26cff7:	sub    %rax,%rcx
  26cffa:	cmovb  %rdx,%rcx
<rexx_exec::Interp>::stack_exhausted::<u8>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:1339
  26cffe:	cmp    0x1f60(%r14),%rcx
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:55
  26d005:	jbe    26d2ab <<rexx_exec::Interp>::begin_invoke+0xf6b>
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  26d00b:	mov    $0x2,%edi
  26d010:	mov    $0x1,%esi
  26d015:	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  26d01a:	test   %rax,%rax
  26d01d:	je     26e25f <<rexx_exec::Interp>::begin_invoke+0x1f1f>
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26d023:	movw   $0x3131,(%rax)
<rexx_exec::error::Raised>::syntax:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:139
  26d028:	movq   $0xffffffffffffffff,0x118(%rsp)
  26d034:	lea    -0x1c6424(%rip),%rcx        # a6c17 <panic_unwind::imp::CANARY+0x41eba>
  26d03b:	mov    %rcx,0x120(%rsp)
  26d043:	movq   $0x6,0x128(%rsp)
  26d04f:	movq   $0x0,0x100(%rsp)
  26d05b:	movq   $0x8,0x108(%rsp)
  26d067:	movq   $0x0,0x110(%rsp)
  26d073:	movq   $0x2,0x130(%rsp)
  26d07f:	mov    %rax,0x138(%rsp)
  26d087:	movq   $0x2,0x140(%rsp)
  26d093:	movq   $0xffffffffffffffff,0x148(%rsp)
  26d09f:	movabs $0x1000b00000000,%rax
  26d0a9:	mov    %rax,0x160(%rsp)
  26d0b1:	movl   $0x0,0x168(%rsp)
  26d0bc:	movw   $0x0,0x16c(%rsp)
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26d0c6:	mov    $0x70,%edi
  26d0cb:	call   *0x26aee7(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26d0d1:	test   %rax,%rax
  26d0d4:	mov    0x18(%rsp),%rdi
  26d0d9:	je     26e048 <<rexx_exec::Interp>::begin_invoke+0x1d08>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
  26d0df:	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26d0e2:	movups 0x160(%rsp),%xmm0
  26d0ea:	movups %xmm0,0x60(%rax)
  26d0ee:	movups 0x150(%rsp),%xmm0
  26d0f6:	movups %xmm0,0x50(%rax)
  26d0fa:	movups 0x140(%rsp),%xmm0
  26d102:	movups %xmm0,0x40(%rax)
  26d106:	movdqu 0x100(%rsp),%xmm0
  26d10f:	movdqu 0x110(%rsp),%xmm1
  26d118:	movdqu 0x120(%rsp),%xmm2
  26d121:	movdqu 0x130(%rsp),%xmm3
  26d12a:	movdqu %xmm3,0x30(%rax)
  26d12f:	movdqu %xmm2,0x20(%rax)
  26d134:	movdqu %xmm1,0x10(%rax)
  26d139:	movdqu %xmm0,(%rax)
  26d13d:	mov    $0x3,%r14d
<rexx_exec::Interp>::begin_method:
  26d143:	jmp    26d18d <<rexx_exec::Interp>::begin_invoke+0xe4d>
  26d145:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:44
  26d14d:	call   1cede0 <<rexx_exec::Loud>::missing_body>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  26d152:	mov    $0x18,%edi
  26d157:	call   *0x26ae5b(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  26d15d:	test   %rax,%rax
  26d160:	je     26e037 <<rexx_exec::Interp>::begin_invoke+0x1cf7>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26d166:	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  26d169:	mov    0x110(%rsp),%rax
  26d171:	mov    %rax,0x10(%rbx)
  26d175:	movdqu 0x100(%rsp),%xmm0
  26d17e:	movdqu %xmm0,(%rbx)
  26d182:	mov    $0x2,%r14d
<rexx_exec::Interp>::begin_method:
  26d188:	mov    0x18(%rsp),%rdi
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d18d:	decq   (%rdi)
<alloc::rc::Rc<rexx_parse::Program> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26d190:	jne    26d197 <<rexx_exec::Interp>::begin_invoke+0xe57>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26d192:	call   191f90 <<alloc::rc::Rc<rexx_parse::Program>>::drop_slow>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26d197:	cmp    $0xffffffffffffffff,%r14
  26d19b:	jne    26c9eb <<rexx_exec::Interp>::begin_invoke+0x6ab>
<rexx_exec::Interp>::begin_invoke:
  26d1a1:	mov    0x10(%rsp),%rax
  26d1a6:	movq   $0x2,0x8(%rax)
  26d1ae:	movq   $0x0,(%rax)
core::ptr::drop_glue::<rexx_exec::dispatch::Invocable>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  26d1b5:	jmp    26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:134
  26d1ba:	movq   $0x0,0x100(%rsp)
<core::iter::adapters::zip::Zip<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>> as core::iter::adapters::zip::ZipImpl<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>>>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/zip.rs:300
  26d1c6:	test   %r15,%r15
  26d1c9:	je     26d3c3 <<rexx_exec::Interp>::begin_invoke+0x1083>
<rexx_core::handle::ObjRef>::inline_text:
  26d1cf:	lea    0x100(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:136
  26d1d7:	mov    0x540(%rsp),%rsi
  26d1df:	mov    %r15,%rdx
  26d1e2:	call   *0x26ade0(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:138
  26d1e8:	mov    0x100(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:140
  26d1f0:	shl    $0x8,%rax
  26d1f4:	jmp    26d3c5 <<rexx_exec::Interp>::begin_invoke+0x1085>
<rexx_exec::Interp>::begin_invoke:
  26d1f9:	mov    %r15,0x10(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2783
  26d1fe:	mov    0x60(%rsp),%r15
  26d203:	mov    %r13,%rdi
  26d206:	mov    0x68(%rsp),%r13
<alloc::boxed::Box<[u8]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:845
  26d20b:	mov    %r12,%rsi
  26d20e:	call   1ac620 <<alloc::boxed::Box<[u8]>>::try_clone_from_ref_in>
  26d213:	test   %rax,%rax
  26d216:	je     26e0ea <<rexx_exec::Interp>::begin_invoke+0x1daa>
<rexx_exec::Interp>::begin_invoke:
  26d21c:	mov    %rax,%rbx
  26d21f:	mov    %rdx,%r12
<alloc::boxed::Box<[core::option::Option<rexx_core::handle::ObjRef>]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:845
  26d222:	mov    0x550(%rsp),%rdi
  26d22a:	mov    %rbp,%rsi
  26d22d:	call   26f190 <<alloc::boxed::Box<[core::option::Option<rexx_core::handle::ObjRef>]>>::try_clone_from_ref_in>
  26d232:	test   %rax,%rax
  26d235:	je     26e0f9 <<rexx_exec::Interp>::begin_invoke+0x1db9>
<core::option::Option<alloc::boxed::Box<rexx_exec::dispatch::NativePark>>>::as_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:765
  26d23b:	mov    0x500(%r14),%rdi
  26d242:	test   %rdi,%rdi
  26d245:	je     26d56d <<rexx_exec::Interp>::begin_invoke+0x122d>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2791
  26d24b:	add    $0x28,%rdi
  26d24f:	mov    %r15,0x100(%rsp)
  26d257:	mov    %r13,0x108(%rsp)
  26d25f:	mov    %rbx,0x110(%rsp)
  26d267:	mov    %r12,0x118(%rsp)
  26d26f:	mov    %rax,0x120(%rsp)
  26d277:	mov    %rdx,0x128(%rsp)
  26d27f:	mov    0x30(%rsp),%rax
  26d284:	mov    %rax,0x130(%rsp)
  26d28c:	mov    0x20(%rsp),%rax
  26d291:	mov    %rax,0x138(%rsp)
  26d299:	lea    0x100(%rsp),%rsi
<alloc::vec::Vec<(rexx_exec::dispatch::Then, core::option::Option<rexx_exec::dispatch::NativeBlame>)>>::push:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:996
  26d2a1:	call   26f1f0 <<alloc::vec::Vec<(rexx_exec::dispatch::Then, core::option::Option<rexx_exec::dispatch::NativeBlame>)>>::push_mut>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2791
  26d2a6:	jmp    26d5bd <<rexx_exec::Interp>::begin_invoke+0x127d>
  26d2ab:	mov    0x18(%rsp),%rax
<rexx_exec::Interp>::begin_method:
  26d2b0:	lea    0x10(%rax),%r9
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:60
  26d2b4:	mov    %r13,0x110(%rsp)
  26d2bc:	movq   $0x1,0x100(%rsp)
  26d2c8:	mov    %r12,0x108(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:65
  26d2d0:	lea    0xa8(%rax),%r8
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:59
  26d2d7:	mov    0x8(%rsi),%rdx
  26d2db:	mov    0x10(%rsi),%rcx
  26d2df:	lea    0x100(%rsp),%rsi
  26d2e7:	mov    %r14,%rdi
  26d2ea:	call   25d7b0 <<rexx_exec::Interp>::plan_for>
  26d2ef:	mov    %rax,0x38(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:68
  26d2f4:	mov    0x80(%rax),%rsi
  26d2fb:	lea    0xab0(%r14),%rdi
  26d302:	call   15f920 <<rexx_core::roots::RootSet>::push_slots>
  26d307:	mov    %rax,0xf0(%rsp)
  26d30f:	mov    %rdx,0xf8(%rsp)
  26d317:	movb   $0x1,0xf(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:71
  26d31c:	cmpb   $0x0,0x48(%rsp)
  26d321:	je     26d33d <<rexx_exec::Interp>::begin_invoke+0xffd>
  26d323:	mov    0x38(%rsp),%rax
  26d328:	cmpb   $0x0,0xf9(%rax)
  26d32f:	jne    26d342 <<rexx_exec::Interp>::begin_invoke+0x1002>
  26d331:	movzbl 0xe4c(%r14),%eax
  26d339:	mov    %al,0xf(%rsp)
  26d33d:	mov    0x38(%rsp),%rax
  26d342:	mov    %rax,%rcx
  26d345:	mov    0x30(%rax),%rax
  26d349:	mov    %rax,0xd0(%rsp)
  26d351:	mov    0x38(%rcx),%rax
  26d355:	mov    %rax,0xd8(%rsp)
  26d35d:	mov    0x40(%rcx),%rax
  26d361:	mov    %rax,0xc8(%rsp)
  26d369:	mov    0x48(%rcx),%rax
  26d36d:	mov    %rax,0xe0(%rsp)
<rexx_exec::Interp>::next_activation_id:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1142
  26d375:	mov    0x1f50(%r14),%rax
  26d37c:	mov    %rax,0xe8(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:1143
  26d384:	inc    %rax
  26d387:	mov    %rax,0x1f50(%r14)
  26d38e:	mov    0x28(%rsp),%rsi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:73
  26d393:	cmpl   $0x2,(%rsi)
  26d396:	jne    26d5d6 <<rexx_exec::Interp>::begin_invoke+0x1296>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:75
  26d39c:	mov    %r14,%rdi
  26d39f:	mov    0x20(%rsp),%rsi
  26d3a4:	mov    0x30(%rsp),%rdx
  26d3a9:	call   26ea40 <<rexx_exec::Interp>::super_scope_for>
  26d3ae:	mov    %rax,0xb0(%rsp)
  26d3b6:	mov    %rdx,0xb8(%rsp)
  26d3be:	jmp    26d5f3 <<rexx_exec::Interp>::begin_invoke+0x12b3>
<rexx_exec::Interp>::begin_invoke:
  26d3c3:	xor    %eax,%eax
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:140
  26d3c5:	lea    (%rax,%r15,4),%rax
  26d3c9:	add    $0x3,%rax
  26d3cd:	mov    %rax,0x18(%rsp)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  26d3d2:	mov    0xaf0(%r14),%r15
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26d3d9:	cmp    0xae0(%r14),%r15
  26d3e0:	jne    26d3ee <<rexx_exec::Interp>::begin_invoke+0x10ae>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26d3e2:	lea    0xae0(%r14),%rdi
  26d3e9:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26d3ee:	mov    0xae8(%r14),%rax
  26d3f5:	mov    0x18(%rsp),%rcx
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26d3fa:	mov    %rcx,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26d3fe:	inc    %r15
  26d401:	mov    %r15,0xaf0(%r14)
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2499
  26d408:	mov    %r14,%rdi
  26d40b:	mov    0x550(%rsp),%rsi
  26d413:	mov    0x558(%rsp),%rdx
  26d41b:	call   26afc0 <<rexx_exec::Interp>::security_arguments_array>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2500
  26d420:	lea    -0x203387(%rip),%rcx        # 6a0a0 <panic_unwind::imp::CANARY+0x5343>
  26d427:	mov    %rcx,0x100(%rsp)
  26d42f:	movq   $0x6,0x108(%rsp)
  26d43b:	mov    0x20(%rsp),%rcx
  26d440:	mov    %rcx,0x110(%rsp)
  26d448:	lea    -0x21a9d7(%rip),%rcx        # 52a78 <GCC_except_table5014+0x3094>
  26d44f:	mov    %rcx,0x118(%rsp)
  26d457:	movq   $0x4,0x120(%rsp)
  26d463:	mov    0x18(%rsp),%rcx
  26d468:	mov    %rcx,0x128(%rsp)
  26d470:	lea    -0x1c690f(%rip),%rcx        # a6b68 <panic_unwind::imp::CANARY+0x41e0b>
  26d477:	mov    %rcx,0x130(%rsp)
  26d47f:	movq   $0x9,0x138(%rsp)
  26d48b:	mov    %rax,0x140(%rsp)
<rexx_exec::Interp>::effective_security_manager:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:112
  26d493:	cmpb   $0x1,0x1ff4(%r14)
  26d49b:	jne    26de86 <<rexx_exec::Interp>::begin_invoke+0x1b46>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:115
  26d4a1:	mov    %r14,%rdi
  26d4a4:	call   26ae50 <<rexx_exec::Interp>::armed_security_manager>
<rexx_exec::Interp>::security_check_by:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:202
  26d4a9:	cmp    $0x1,%rax
  26d4ad:	jne    26de86 <<rexx_exec::Interp>::begin_invoke+0x1b46>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  26d4b3:	mov    0xaf0(%r14),%rax
  26d4ba:	mov    %rax,0x18(%rsp)
<rexx_exec::Interp>::security_check_by:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:206
  26d4bf:	sub    $0x8,%rsp
  26d4c3:	lea    -0x1c6a45(%rip),%rcx        # a6a85 <panic_unwind::imp::CANARY+0x41d28>
  26d4ca:	lea    0x338(%rsp),%rdi
  26d4d2:	lea    0x108(%rsp),%r9
  26d4da:	mov    $0x6,%r8d
  26d4e0:	mov    %r14,%rsi
  26d4e3:	push   $0x3
  26d4e5:	call   26b0f0 <<rexx_exec::Interp>::security_send>
  26d4ea:	add    $0x10,%rsp
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::truncate:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1823
  26d4ee:	mov    0xaf0(%r14),%r15
  26d4f5:	mov    0x18(%rsp),%rax
  26d4fa:	cmp    %r15,%rax
  26d4fd:	ja     26d509 <<rexx_exec::Interp>::begin_invoke+0x11c9>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1828
  26d4ff:	mov    %rax,0xaf0(%r14)
  26d506:	mov    %rax,%r15
<rexx_exec::Interp>::security_check_by:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:208
  26d509:	movzbl 0x330(%rsp),%edx
  26d511:	mov    %edx,%ecx
  26d513:	xor    $0x1,%cl
  26d516:	mov    0x338(%rsp),%rax
  26d51e:	and    %al,%cl
  26d520:	mov    0x340(%rsp),%rsi
  26d528:	mov    %rsi,0x18(%rsp)
  26d52d:	cmp    $0x1,%cl
  26d530:	jne    26d9f1 <<rexx_exec::Interp>::begin_invoke+0x16b1>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26d536:	lea    0xae0(%r14),%rax
  26d53d:	cmp    (%rax),%r15
  26d540:	jne    26d54e <<rexx_exec::Interp>::begin_invoke+0x120e>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26d542:	lea    0xae0(%r14),%rdi
  26d549:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26d54e:	mov    0xae8(%r14),%rax
  26d555:	mov    0x18(%rsp),%rcx
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26d55a:	mov    %rcx,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26d55e:	inc    %r15
  26d561:	mov    %r15,0xaf0(%r14)
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2505
  26d568:	jmp    26ddda <<rexx_exec::Interp>::begin_invoke+0x1a9a>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2792
  26d56d:	mov    %rbx,0x100(%rsp)
  26d575:	mov    %r12,0x108(%rsp)
  26d57d:	mov    %rax,0x110(%rsp)
  26d585:	mov    %rdx,0x118(%rsp)
  26d58d:	mov    0x30(%rsp),%rax
  26d592:	mov    %rax,0x120(%rsp)
  26d59a:	mov    0x20(%rsp),%rax
  26d59f:	mov    %rax,0x128(%rsp)
  26d5a7:	lea    0x100(%rsp),%rcx
  26d5af:	mov    %r14,%rdi
  26d5b2:	mov    %r15,%rsi
  26d5b5:	mov    %r13,%rdx
  26d5b8:	call   26f260 <<rexx_exec::Interp>::push_native_tail>
  26d5bd:	mov    0x10(%rsp),%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2794
  26d5c2:	movq   $0x2,0x8(%r15)
  26d5ca:	movq   $0x0,(%r15)
  26d5d1:	jmp    26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:74
  26d5d6:	mov    %r14,%rdi
  26d5d9:	mov    0x30(%rsp),%rdx
  26d5de:	call   2328e0 <<rexx_exec::Interp>::super_scope_in>
  26d5e3:	mov    %rax,0xb0(%rsp)
  26d5eb:	mov    %rdx,0xb8(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:82
  26d5f3:	mov    %r14,%rdi
  26d5f6:	mov    0x550(%rsp),%rsi
  26d5fe:	mov    0x558(%rsp),%rdx
  26d606:	call   25e000 <<rexx_exec::Interp>::shared_arguments>
  26d60b:	mov    %rax,0x88(%rsp)
  26d613:	mov    %rdx,0xc0(%rsp)
<rexx_exec::activity::Activity>::invocation_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:434
  26d61b:	mov    0x4f0(%r14),%rcx
  26d622:	mov    0x548(%rsp),%rax
  26d62a:	cmp    %rax,0x4f8(%r14)
  26d631:	sete   %al
  26d634:	test   %rcx,%rcx
  26d637:	je     26e07e <<rexx_exec::Interp>::begin_invoke+0x1d3e>
  26d63d:	test   %al,%al
  26d63f:	je     26e088 <<rexx_exec::Interp>::begin_invoke+0x1d48>
  26d645:	mov    %rcx,0x28(%rsp)
  26d64a:	mov    0x28(%rsp),%rax
<[u8]>::as_ptr:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:729
  26d64f:	lea    0x10(%rax),%rdi
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  26d653:	mov    0x540(%rsp),%rsi
  26d65b:	mov    0x548(%rsp),%rdx
  26d663:	call   *0x26a96f(%rip)        # 4d7fd8 <bcmp@GLIBC_2.2.5>
  26d669:	mov    0x28(%rsp),%rsi
  26d66e:	test   %eax,%eax
<rexx_exec::activity::Activity>::invocation_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:435
  26d670:	jne    26e088 <<rexx_exec::Interp>::begin_invoke+0x1d48>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d676:	incq   (%rsi)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26d679:	je     26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d67f:	incq   (%rsi)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26d682:	je     26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
core::mem::replace::<rexx_exec::CallContext>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  26d688:	movdqu 0x70(%r14),%xmm0
  26d68e:	movdqu 0x80(%r14),%xmm1
  26d697:	movdqu 0x90(%r14),%xmm2
  26d6a0:	movdqa %xmm2,0x320(%rsp)
  26d6a9:	movdqa %xmm1,0x310(%rsp)
  26d6b2:	movdqa %xmm0,0x300(%rsp)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26d6bb:	movq   $0x1,0x70(%r14)
  26d6c3:	mov    0x20(%rsp),%rax
  26d6c8:	mov    %rax,0x78(%r14)
  26d6cc:	mov    %rsi,0x80(%r14)
  26d6d3:	mov    0x80(%rsp),%rax
  26d6db:	mov    %rax,0x88(%r14)
  26d6e2:	mov    0x88(%rsp),%rax
  26d6ea:	mov    %rax,0x90(%r14)
  26d6f1:	mov    0xc0(%rsp),%rax
  26d6f9:	mov    %rax,0x98(%r14)
  26d700:	mov    $0x2,%dil
<rexx_exec::Interp>::options_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2679
  26d703:	cmpb   $0x1,0x1ff6(%r14)
  26d70b:	jne    26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::ProgramSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26d70d:	cmp    0xc58(%r14),%rbx
  26d714:	jae    26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
<rexx_exec::plan::Programs>::package_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:150
  26d716:	mov    0xc50(%r14),%rax
<rexx_exec::plan::ProgramId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:32
  26d71d:	cmp    %r13,(%rax,%r15,1)
<rexx_exec::plan::Programs>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:136
  26d721:	jne    26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
  26d723:	add    %r15,%rax
<core::option::Option<alloc::rc::Rc<rexx_parse::Program>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26d726:	cmpq   $0x0,0x8(%rax)
  26d72b:	je     26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
  26d72d:	cmp    0xc70(%r14),%rbx
  26d734:	jae    26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_exec::plan::PackageSlot>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26d736:	mov    0xc68(%r14),%rax
<usize as core::slice::index::SliceIndex<[rexx_exec::plan::PackageSlot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26d73d:	lea    (%rbx,%rbx,2),%rcx
<core::option::Option<rexx_core::handle::ObjRef> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  26d741:	cmpl   $0x1,(%rax,%rcx,8)
  26d745:	jne    26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
<rexx_exec::plan::Programs>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:158
  26d747:	lea    (%rax,%rcx,8),%rax
  26d74b:	mov    0x8(%rax),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  26d74f:	test   $0x3,%cl
  26d752:	je     26df2a <<rexx_exec::Interp>::begin_invoke+0x1bea>
<rexx_exec::Interp>::begin_method:
  26d758:	mov    0xe8(%rsp),%rdx
<rexx_exec::activation::Activation>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:880
  26d760:	mov    %rdx,0x418(%rsp)
  26d768:	mov    0x18(%rsp),%rdx
  26d76d:	mov    %rdx,0x3c8(%rsp)
  26d775:	mov    %r13,0x420(%rsp)
  26d77d:	movq   $0x1,0x330(%rsp)
  26d789:	mov    %r12,0x338(%rsp)
  26d791:	mov    0x38(%rsp),%rdx
  26d796:	mov    %rdx,0x3d0(%rsp)
  26d79e:	movdqu 0x265a32(%rip),%xmm0        # 4d31d8 <rexx_api::ffi::CALL_CONTEXT+0x17e48>
  26d7a6:	movdqu %xmm0,0x3d8(%rsp)
  26d7af:	movdqu 0x265a31(%rip),%xmm1        # 4d31e8 <rexx_api::ffi::CALL_CONTEXT+0x17e58>
  26d7b7:	movdqu %xmm1,0x3e8(%rsp)
  26d7c0:	mov    0xf0(%rsp),%r12
  26d7c8:	mov    %r12,0x428(%rsp)
  26d7d0:	mov    0xf8(%rsp),%r15
  26d7d8:	mov    %r15,0x430(%rsp)
  26d7e0:	movw   $0x1,0x4f6(%rsp)
  26d7ea:	movb   $0x3,0x506(%rsp)
  26d7f2:	mov    %rsi,0x438(%rsp)
  26d7fa:	mov    0x80(%rsp),%rdx
  26d802:	mov    %rdx,0x440(%rsp)
  26d80a:	mov    0x30(%rsp),%rdx
  26d80f:	mov    %rdx,0x448(%rsp)
  26d817:	mov    0x20(%rsp),%rdx
  26d81c:	mov    %rdx,0x450(%rsp)
  26d824:	movq   $0x0,0x370(%rsp)
  26d830:	movq   $0x8,0x378(%rsp)
  26d83c:	movq   $0x0,0x380(%rsp)
  26d848:	pxor   %xmm2,%xmm2
  26d84c:	movdqu %xmm2,0x458(%rsp)
  26d855:	movb   $0x0,0x507(%rsp)
  26d85d:	movb   $0x1,0x4f8(%rsp)
  26d865:	movq   $0x9,0x398(%rsp)
  26d871:	movq   $0x0,0x3a0(%rsp)
  26d87d:	movb   $0x0,0x3a8(%rsp)
  26d885:	movl   $0x0,0x388(%rsp)
  26d890:	movw   $0x0,0x4f4(%rsp)
  26d89a:	movl   $0x0,0x4f0(%rsp)
  26d8a5:	movabs $0x1000000000000,%rdx
  26d8af:	mov    %rdx,0x4f9(%rsp)
  26d8b7:	movb   $0x4e,0x501(%rsp)
  26d8bf:	movq   $0x0,0x3b0(%rsp)
  26d8cb:	movb   $0x2,0x3b8(%rsp)
  26d8d3:	movw   $0x0,0x3c1(%rsp)
  26d8dd:	movb   $0x0,0x3c3(%rsp)
  26d8e5:	movq   $0x0,0x468(%rsp)
  26d8f1:	movq   $0x0,0x478(%rsp)
  26d8fd:	movq   $0x0,0x340(%rsp)
  26d909:	movdqu %xmm2,0x488(%rsp)
  26d912:	movdqu %xmm2,0x498(%rsp)
  26d91b:	movdqu %xmm2,0x4a8(%rsp)
  26d924:	movl   $0x30001,0x502(%rsp)
  26d92f:	movq   $0x0,0x350(%rsp)
  26d93b:	movq   $0x0,0x360(%rsp)
  26d947:	movq   $0x1,0x4b8(%rsp)
  26d953:	movl   $0x0,0x390(%rsp)
  26d95e:	movq   $0x0,0x4e0(%rsp)
  26d96a:	movdqu %xmm2,0x4c0(%rsp)
  26d973:	movq   $0x0,0x4d0(%rsp)
  26d97f:	movdqu %xmm1,0x408(%rsp)
  26d988:	movdqu %xmm0,0x3f8(%rsp)
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:122
  26d991:	cmp    $0x2,%dil
  26d995:	jne    26d9a3 <<rexx_exec::Interp>::begin_invoke+0x1663>
  26d997:	xor    %eax,%eax
  26d999:	mov    0x70(%rsp),%r10d
  26d99e:	jmp    26da24 <<rexx_exec::Interp>::begin_invoke+0x16e4>
  26d9a3:	lea    0x398(%rsp),%rdx
  26d9ab:	mov    0xa0(%rsp),%rsi
  26d9b3:	mov    %rsi,0x10(%rdx)
  26d9b7:	movdqa 0x90(%rsp),%xmm0
  26d9c0:	movdqu %xmm0,(%rdx)
<rexx_exec::PackageStart>::apply:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1896
  26d9c4:	mov    %dil,0x4f0(%rsp)
  26d9cc:	mov    0x50(%rsp),%edx
  26d9d0:	mov    %edx,0x4f1(%rsp)
  26d9d7:	movzbl 0x54(%rsp),%edx
  26d9dc:	mov    %dl,0x4f5(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1897
  26d9e3:	cmp    $0x2,%cl
  26d9e6:	mov    0x70(%rsp),%r10d
  26d9eb:	jne    26da12 <<rexx_exec::Interp>::begin_invoke+0x16d2>
  26d9ed:	xor    %eax,%eax
  26d9ef:	jmp    26da24 <<rexx_exec::Interp>::begin_invoke+0x16e4>
<rexx_exec::Interp>::begin_invoke:
  26d9f1:	mov    $0x1,%ecx
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26d9f6:	test   %dl,%dl
  26d9f8:	je     26ddd2 <<rexx_exec::Interp>::begin_invoke+0x1a92>
  26d9fe:	mov    0x10(%rsp),%rdx
  26da03:	mov    0x40(%rsp),%rsi
  26da08:	mov    0x18(%rsp),%r15
  26da0d:	jmp    26dead <<rexx_exec::Interp>::begin_invoke+0x1b6d>
<rexx_exec::PackageStart>::apply:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1898
  26da12:	mov    %cl,0x4f9(%rsp)
  26da19:	mov    %rax,0x4fa(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1897
  26da21:	shr    $0x10,%eax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26da24:	cmp    %rbp,0xfd8(%r14)
  26da2b:	jbe    26da4f <<rexx_exec::Interp>::begin_invoke+0x170f>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26da2d:	mov    0xfd0(%r14),%rcx
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26da34:	shl    $0x4,%rbp
<core::option::Option<rexx_core::handle::ObjRef>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26da38:	cmpb   $0x0,(%rcx,%rbp,1)
  26da3c:	je     26db1d <<rexx_exec::Interp>::begin_invoke+0x17dd>
  26da42:	lea    (%rcx,%rbp,1),%r8
  26da46:	add    $0x8,%r8
  26da4a:	jmp    26daf9 <<rexx_exec::Interp>::begin_invoke+0x17b9>
<hashbrown::map::HashMap<rexx_core::body::MethodId, rexx_core::handle::ObjRef, rustc_hash::FxBuildHasher>>::is_empty:
/rust/deps/hashbrown-0.17.1/src/map.rs:825
  26da4f:	cmpq   $0x0,0xff8(%r14)
<rexx_exec::method_rows::MethodRows<rexx_core::handle::ObjRef>>::get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/method_rows.rs:33
  26da57:	je     26db1d <<rexx_exec::Interp>::begin_invoke+0x17dd>
<std::collections::hash::map::HashMap<rexx_core::body::MethodId, rexx_core::handle::ObjRef, rustc_hash::FxBuildHasher>>::get::<rexx_core::body::MethodId>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/std/src/collections/hash/map.rs:1039
  26da5d:	movabs $0xf1357aea2e62a9c5,%rcx
<usize>::wrapping_mul:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2728
  26da67:	imul   %rcx,%rbp
core::intrinsics::rotate_left::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:2215
  26da6b:	rol    $0x1a,%rbp
<hashbrown::control::tag::Tag>::full:
/rust/deps/hashbrown-0.17.1/src/control/tag.rs:47
  26da6f:	mov    %rbp,%rsi
  26da72:	shr    $0x39,%rsi
  26da76:	mov    0xfe0(%r14),%rcx
<hashbrown::raw::RawTableInner>::probe_seq:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2452
  26da7d:	mov    0xfe8(%r14),%rdx
  26da84:	movd   %esi,%xmm0
  26da88:	punpcklbw %xmm0,%xmm0
  26da8c:	pshuflw $0x0,%xmm0,%xmm0
  26da91:	pshufd $0x44,%xmm0,%xmm0
  26da96:	xor    %esi,%esi
  26da98:	pcmpeqd %xmm1,%xmm1
<hashbrown::raw::RawTableInner>::find_inner:
  26da9c:	and    %rdx,%rbp
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  26da9f:	movdqu (%rcx,%rbp,1),%xmm2
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26daa4:	movdqa %xmm2,%xmm3
  26daa8:	pcmpeqb %xmm0,%xmm3
  26daac:	pmovmskb %xmm3,%edi
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26dab0:	test   %edi,%edi
  26dab2:	je     26dadc <<rexx_exec::Interp>::begin_invoke+0x179c>
<core::num::nonzero::NonZero<u16>>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:628
  26dab4:	tzcnt  %edi,%r9d
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2032
  26dab9:	add    %rbp,%r9
  26dabc:	and    %rdx,%r9
<*mut (rexx_core::body::MethodId, rexx_core::handle::ObjRef)>::sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:1054
  26dabf:	shl    $0x4,%r9
  26dac3:	mov    %rcx,%r8
  26dac6:	sub    %r9,%r8
<rexx_core::body::MethodId as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1070
  26dac9:	cmp    -0x10(%r8),%r10d
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26dacd:	je     26daf5 <<rexx_exec::Interp>::begin_invoke+0x17b5>
<core::num::nonzero::NonZero<u16>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/nonzero.rs:486
  26dacf:	lea    -0x1(%rdi),%r8d
<hashbrown::control::bitmask::BitMask>::lowest_set_bit:
/rust/deps/hashbrown-0.17.1/src/control/bitmask.rs:40
  26dad3:	and    %di,%r8w
  26dad7:	mov    %r8d,%edi
  26dada:	jne    26dab4 <<rexx_exec::Interp>::begin_invoke+0x1774>
core::core_arch::x86::sse2::_mm_movemask_epi8:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/../../stdarch/crates/core_arch/src/x86/sse2.rs:1569
  26dadc:	pcmpeqb %xmm1,%xmm2
  26dae0:	pmovmskb %xmm2,%edi
core::intrinsics::likely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:455
  26dae4:	test   %edi,%edi
  26dae6:	jne    26db1d <<rexx_exec::Interp>::begin_invoke+0x17dd>
<hashbrown::raw::ProbeSeq>::move_next:
/rust/deps/hashbrown-0.17.1/src/raw.rs:90
  26dae8:	add    %rsi,%rbp
  26daeb:	add    $0x10,%rbp
/rust/deps/hashbrown-0.17.1/src/raw.rs:89
  26daef:	add    $0x10,%rsi
<hashbrown::raw::RawTableInner>::find_inner:
/rust/deps/hashbrown-0.17.1/src/raw.rs:2012
  26daf3:	jmp    26da9c <<rexx_exec::Interp>::begin_invoke+0x175c>
<hashbrown::raw::RawTable<(rexx_core::body::MethodId, rexx_core::handle::ObjRef)>>::get::<hashbrown::map::equivalent_key<rexx_core::body::MethodId, rexx_core::body::MethodId, rexx_core::handle::ObjRef>::{closure#0}>:
/rust/deps/hashbrown-0.17.1/src/raw.rs:1217
  26daf5:	add    $0xfffffffffffffff8,%r8
<core::option::Option<&rexx_core::handle::ObjRef>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  26daf9:	mov    (%r8),%rbx
  26dafc:	lea    0x330(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:128
  26db04:	call   1d3620 <<rexx_exec::activation::Activation>::cold_mut>
  26db09:	movq   $0x1,0x10(%rax)
  26db11:	mov    %rbx,0x18(%rax)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:132
  26db15:	movzbl 0x4fc(%rsp),%eax
  26db1d:	movzbl 0xf(%rsp),%ebp
<rexx_exec::activation::ActivationFlags>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:516
  26db22:	or     %al,%bpl
  26db25:	mov    0x48(%rsp),%rax
  26db2a:	add    %eax,%eax
  26db2c:	mov    %ebp,%ecx
  26db2e:	not    %cl
  26db30:	and    $0x1,%cl
  26db33:	shl    $0x2,%cl
  26db36:	or     %al,%cl
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:133
  26db38:	mov    %cl,0x507(%rsp)
  26db3f:	lea    0x100(%rsp),%rbx
  26db47:	lea    0x330(%rsp),%rsi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:134
  26db4f:	mov    $0x1d8,%edx
  26db54:	mov    %rbx,%rdi
  26db57:	call   *0x26a46b(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
  26db5d:	mov    %r14,%rdi
  26db60:	mov    %rbx,%rsi
  26db63:	call   25d5a0 <<rexx_exec::Interp>::push_activation>
  26db68:	mov    0xe0(%rsp),%rbx
  26db70:	mov    0xd8(%rsp),%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:135
  26db78:	mov    %r14,%rdi
  26db7b:	call   25de30 <<rexx_exec::Interp>::trace_package_invocation_entry>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:136
  26db80:	test   %bpl,0x48(%rsp)
  26db85:	jne    26e1a6 <<rexx_exec::Interp>::begin_invoke+0x1e66>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:148
  26db8b:	cmpq   $0x0,0xd0(%rsp)
  26db94:	jne    26dbad <<rexx_exec::Interp>::begin_invoke+0x186d>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:150
  26db96:	lea    -0x21b185(%rip),%rsi        # 52a18 <GCC_except_table5014+0x3034>
  26db9d:	mov    $0x4,%edx
  26dba2:	mov    %r14,%rdi
  26dba5:	call   21f570 <<rexx_exec::Interp>::slot_of>
  26dbaa:	mov    %rax,%r13
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:86
  26dbad:	mov    0x470(%r14),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26dbb4:	test   %rax,%rax
  26dbb7:	je     26e0b6 <<rexx_exec::Interp>::begin_invoke+0x1d76>
<alloc::vec::Vec<(usize, rexx_exec::activation::InstanceVar)>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3126
  26dbbd:	cmpq   $0x0,0x50(%rax)
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:87
  26dbc2:	je     26dbda <<rexx_exec::Interp>::begin_invoke+0x189a>
<rexx_core::roots::SlotFrame as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:31
  26dbc4:	cmp    %r15,0x100(%rax)
  26dbcb:	jne    26dbda <<rexx_exec::Interp>::begin_invoke+0x189a>
  26dbcd:	cmp    %r12,0xf8(%rax)
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:39
  26dbd4:	je     26e1bd <<rexx_exec::Interp>::begin_invoke+0x1e7d>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:300
  26dbda:	cmp    0xb80(%r14),%r15
  26dbe1:	jne    26e10c <<rexx_exec::Interp>::begin_invoke+0x1dcc>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:302
  26dbe7:	add    0xb78(%r14),%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  26dbee:	cmp    0xb08(%r14),%r13
  26dbf5:	jae    26e0d0 <<rexx_exec::Interp>::begin_invoke+0x1d90>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26dbfb:	mov    0xb00(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:245
  26dc02:	shl    $0x4,%r13
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:308
  26dc06:	movq   $0x1,(%rax,%r13,1)
  26dc0e:	mov    0x20(%rsp),%rcx
  26dc13:	mov    %rcx,0x8(%rax,%r13,1)
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:153
  26dc18:	cmpq   $0x0,0xc8(%rsp)
  26dc21:	jne    26dc3a <<rexx_exec::Interp>::begin_invoke+0x18fa>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:155
  26dc23:	lea    -0x1c7354(%rip),%rsi        # a68d6 <panic_unwind::imp::CANARY+0x41b79>
  26dc2a:	mov    $0x5,%edx
  26dc2f:	mov    %r14,%rdi
  26dc32:	call   21f570 <<rexx_exec::Interp>::slot_of>
  26dc37:	mov    %rax,%rbx
<core::option::Option<rexx_core::handle::ObjRef>>::unwrap_or:
  26dc3a:	testb  $0x1,0xb0(%rsp)
  26dc42:	mov    $0x2,%r8d
  26dc48:	cmovne 0xb8(%rsp),%r8
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:86
  26dc51:	mov    0x470(%r14),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26dc58:	test   %rax,%rax
  26dc5b:	je     26e0b6 <<rexx_exec::Interp>::begin_invoke+0x1d76>
<alloc::vec::Vec<(usize, rexx_exec::activation::InstanceVar)>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3126
  26dc61:	cmpq   $0x0,0x50(%rax)
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:87
  26dc66:	je     26dc7e <<rexx_exec::Interp>::begin_invoke+0x193e>
<rexx_core::roots::SlotFrame as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:31
  26dc68:	cmp    %r15,0x100(%rax)
  26dc6f:	jne    26dc7e <<rexx_exec::Interp>::begin_invoke+0x193e>
  26dc71:	cmp    %r12,0xf8(%rax)
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:39
  26dc78:	je     26e1d8 <<rexx_exec::Interp>::begin_invoke+0x1e98>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:300
  26dc7e:	cmp    0xb80(%r14),%r15
  26dc85:	jne    26e12b <<rexx_exec::Interp>::begin_invoke+0x1deb>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:302
  26dc8b:	add    0xb78(%r14),%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  26dc92:	cmp    0xb08(%r14),%rbx
  26dc99:	jae    26e0d0 <<rexx_exec::Interp>::begin_invoke+0x1d90>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26dc9f:	mov    0xb00(%r14),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:245
  26dca6:	shl    $0x4,%rbx
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:308
  26dcaa:	movq   $0x1,(%rax,%rbx,1)
  26dcb2:	mov    %r8,0x8(%rax,%rbx,1)
<rexx_exec::Interp>::save_clause_state:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/clause.rs:456
  26dcb7:	mov    0x460(%r14),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/clause.rs:457
  26dcbe:	movzbl 0x468(%r14),%ecx
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:172
  26dcc6:	movaps 0x300(%rsp),%xmm0
  26dcce:	movdqa 0x310(%rsp),%xmm1
  26dcd7:	movdqa 0x320(%rsp),%xmm2
  26dce0:	movdqa %xmm2,0x130(%rsp)
  26dce9:	movdqa %xmm1,0x120(%rsp)
  26dcf2:	movaps %xmm0,0x110(%rsp)
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dcfa:	movb   $0x2,0x188(%rsp)
<rexx_exec::Interp>::save_clause_state:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/clause.rs:454
  26dd02:	movups 0x450(%r14),%xmm0
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dd0a:	movups %xmm0,0x158(%rsp)
  26dd12:	mov    %rax,0x168(%rsp)
  26dd1a:	mov    %cl,0x170(%rsp)
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  26dd21:	movdqu 0x4a8(%r14),%xmm0
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26dd2a:	pxor   %xmm1,%xmm1
  26dd2e:	movdqu %xmm1,0x4a8(%r14)
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dd37:	pshufd $0x4e,%xmm0,%xmm0
  26dd3c:	movdqu %xmm0,0x178(%rsp)
core::mem::replace::<core::option::Option<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  26dd45:	movdqu 0x60(%r14),%xmm0
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26dd4b:	movq   $0x0,0x60(%r14)
<rexx_exec::run::call::CallTail>::method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/call.rs:183
  26dd53:	movdqa %xmm0,0x100(%rsp)
  26dd5c:	movabs $0x8000000000000000,%rax
  26dd66:	mov    %rax,0x140(%rsp)
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  26dd6e:	mov    0x2a8(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26dd75:	cmp    0x298(%r14),%rbx
  26dd7c:	jne    26dd8a <<rexx_exec::Interp>::begin_invoke+0x1a4a>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:171
  26dd7e:	lea    0x298(%r14),%rdi
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26dd85:	call   25e1e0 <<alloc::raw_vec::RawVec<rexx_exec::run::call::CallTail>>::grow_one>
<*mut rexx_exec::run::call::CallTail>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  26dd8a:	lea    (%rbx,%rbx,8),%rdi
  26dd8e:	shl    $0x4,%rdi
  26dd92:	add    0x2a0(%r14),%rdi
  26dd99:	lea    0x100(%rsp),%rsi
core::ptr::write::<rexx_exec::run::call::CallTail>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26dda1:	mov    $0x90,%edx
  26dda6:	call   *0x26a21c(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26ddac:	inc    %rbx
  26ddaf:	mov    %rbx,0x2a8(%r14)
  26ddb6:	mov    $0x2,%r15d
  26ddbc:	mov    $0x8,%eax
  26ddc1:	xor    %ecx,%ecx
  26ddc3:	mov    0x10(%rsp),%rdx
  26ddc8:	mov    0x40(%rsp),%rsi
  26ddcd:	jmp    26deb6 <<rexx_exec::Interp>::begin_invoke+0x1b76>
<rexx_exec::Interp>::check_protected_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2505
  26ddd2:	test   $0x1,%al
  26ddd4:	je     26de86 <<rexx_exec::Interp>::begin_invoke+0x1b46>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26ddda:	lea    0xae0(%r14),%rax
  26dde1:	cmp    (%rax),%r15
  26dde4:	mov    0x18(%rsp),%rbx
  26dde9:	jne    26ddf7 <<rexx_exec::Interp>::begin_invoke+0x1ab7>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26ddeb:	lea    0xae0(%r14),%rdi
  26ddf2:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26ddf7:	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26ddfe:	movabs $0x544c555345521b,%rcx
  26de08:	mov    %rcx,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26de0c:	inc    %r15
  26de0f:	mov    %r15,0xaf0(%r14)
  26de16:	lea    0x330(%rsp),%rdi
rexx_exec::dispatch::hash::store_get:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/hash.rs:989
  26de1e:	mov    %r14,%rsi
  26de21:	mov    %rbx,%rdx
  26de24:	call   26b690 <rexx_exec::dispatch::hash::merged_get>
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
  26de29:	mov    0x338(%rsp),%rax
  26de31:	mov    0x340(%rsp),%r15
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  26de39:	cmpb   $0x0,0x330(%rsp)
  26de41:	mov    $0x1,%ecx
  26de46:	jne    26dea3 <<rexx_exec::Interp>::begin_invoke+0x1b63>
<rexx_exec::Interp>::security_entry:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/security.rs:275
  26de48:	cmp    $0x1,%rax
  26de4c:	jne    26de9f <<rexx_exec::Interp>::begin_invoke+0x1b5f>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  26de4e:	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  26de55:	cmp    0xae0(%r14),%rbx
  26de5c:	jne    26de6a <<rexx_exec::Interp>::begin_invoke+0x1b2a>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  26de5e:	lea    0xae0(%r14),%rdi
  26de65:	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  26de6a:	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  26de71:	mov    %r15,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  26de75:	inc    %rbx
  26de78:	mov    %rbx,0xaf0(%r14)
  26de7f:	mov    $0x1,%eax
  26de84:	jmp    26dea1 <<rexx_exec::Interp>::begin_invoke+0x1b61>
<rexx_exec::Interp>::begin_invoke:
  26de86:	mov    0x28(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2732
  26de8b:	movq   $0x2,(%rax)
  26de92:	mov    0x558(%rsp),%r10
  26de9a:	jmp    26c806 <<rexx_exec::Interp>::begin_invoke+0x4c6>
  26de9f:	xor    %eax,%eax
  26dea1:	xor    %ecx,%ecx
  26dea3:	mov    0x10(%rsp),%rdx
  26dea8:	mov    0x40(%rsp),%rsi
  26dead:	mov    %rax,0x8(%rdx)
  26deb1:	mov    $0x10,%eax
  26deb6:	mov    %r15,(%rdx,%rax,1)
  26deba:	mov    %rcx,(%rdx)
core::ptr::drop_glue::<rexx_exec::dispatch::Invocable>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  26debd:	cmp    $0x5,%rsi
  26dec1:	jb     26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
<core::ptr::non_null::NonNull<alloc::rc::RcInner<rexx_api::load::Library>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:454
  26dec7:	mov    0x2f8(%rsp),%rdi
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26decf:	decq   (%rdi)
<alloc::rc::Rc<rexx_api::load::Library> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26ded2:	jne    26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26ded8:	call   162d60 <<alloc::rc::Rc<rexx_api::load::Library>>::drop_slow>
  26dedd:	jmp    26cf8a <<rexx_exec::Interp>::begin_invoke+0xc4a>
<rexx_exec::Interp>::object_model:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:1833
  26dee2:	mov    %r14,%rdi
  26dee5:	mov    %r8,%r12
  26dee8:	call   18a340 <<rexx_exec::Interp>::build_object_model>
  26deed:	mov    %r12,%r8
<core::option::Option<rexx_exec::dispatch::ObjectModel>>::as_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:765
  26def0:	cmpq   $0xffffffffffffffff,0x1170(%r14)
  26def8:	jne    26c372 <<rexx_exec::Interp>::begin_invoke+0x32>
<core::option::Option<&mut rexx_exec::dispatch::ObjectModel>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  26defe:	lea    -0x203f83(%rip),%rdi        # 69f82 <panic_unwind::imp::CANARY+0x5225>
  26df05:	lea    0x2523d4(%rip),%rdx        # 4c02e0 <rexx_api::ffi::CALL_CONTEXT+0x4f50>
  26df0c:	mov    $0x23,%esi
  26df11:	call   11f010 <core::option::expect_failed>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  26df16:	mov    $0x8,%edi
  26df1b:	mov    $0x18,%esi
  26df20:	call   111b17 <alloc::alloc::handle_alloc_error>
  26df25:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<rexx_exec::Interp>::begin_method:
  26df2a:	mov    %rsi,%r15
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  26df2d:	mov    %rcx,%rax
  26df30:	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  26df34:	mov    %eax,%edx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  26df36:	cmp    %rdx,0xa08(%r14)
  26df3d:	jbe    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
<rexx_exec::Interp>::package_data:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/package_state.rs:171
  26df43:	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  26df4a:	shr    $0x22,%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  26df4e:	lea    (%rdx,%rdx,2),%rdx
  26df52:	shl    $0x5,%rdx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  26df56:	cmp    %ecx,0x4(%rax,%rdx,1)
  26df5a:	jne    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
  26df60:	add    %rdx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  26df63:	cmpl   $0x1,(%rax)
  26df66:	jne    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
<rexx_exec::Interp>::package_data:
  26df6c:	mov    0x470(%r14),%rbx
<rexx_core::body::Body>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1439
  26df73:	movzbl 0x8(%rax),%ecx
  26df77:	cmp    $0x5,%ecx
  26df7a:	je     26e145 <<rexx_exec::Interp>::begin_invoke+0x1e05>
  26df80:	cmp    $0x4,%ecx
  26df83:	jne    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
  26df89:	mov    0x38(%rax),%rax
  26df8d:	test   %rax,%rax
  26df90:	je     26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
<rexx_core::body::NativeState>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:433
  26df96:	movabs $0x8000000000000000,%rcx
  26dfa0:	add    $0x4,%rcx
  26dfa4:	cmp    %rcx,(%rax)
  26dfa7:	jne    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:434
  26dfad:	add    $0x8,%rax
  26dfb1:	jmp    26e152 <<rexx_exec::Interp>::begin_invoke+0x1e12>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  26dfb6:	mov    $0x8,%edi
  26dfbb:	mov    $0x70,%esi
  26dfc0:	call   111b17 <alloc::alloc::handle_alloc_error>
  26dfc5:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
  26dfca:	mov    $0x8,%edi
  26dfcf:	mov    $0x18,%esi
  26dfd4:	call   111b17 <alloc::alloc::handle_alloc_error>
  26dfd9:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<rexx_exec::Interp>::serve_posted_halt:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/scheduler.rs:2665
  26dfde:	mov    %r14,%rdi
  26dfe1:	call   26f610 <<rexx_exec::Interp>::serve_halt_now>
  26dfe6:	jmp    26ce5e <<rexx_exec::Interp>::begin_invoke+0xb1e>
<rexx_exec::Interp>::begin_invoke:
  26dfeb:	lea    0x100(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:41
  26dff3:	call   1cede0 <<rexx_exec::Loud>::missing_body>
  26dff8:	lea    0x330(%rsp),%rdi
  26e000:	lea    0x100(%rsp),%rsi
<rexx_exec::Loud as core::convert::Into<rexx_exec::error::Failure>>::into:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/convert/mod.rs:781
  26e008:	call   1bf3c0 <<rexx_exec::error::Failure as core::convert::From<rexx_exec::Loud>>::from>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:41
  26e00d:	mov    0x330(%rsp),%r14
  26e015:	mov    0x338(%rsp),%rbx
  26e01d:	jmp    26d188 <<rexx_exec::Interp>::begin_invoke+0xe48>
<core::result::Result<core::option::Option<rexx_core::handle::ObjRef>, rexx_exec::error::Failure>>::map::<rexx_exec::dispatch::NativeStarted, rexx_exec::dispatch::NativeStarted::Ran>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:837
  26e022:	movdqu 0x108(%rsp),%xmm0
  26e02b:	xor    %eax,%eax
  26e02d:	mov    0x10(%rsp),%r15
  26e032:	jmp    26cf81 <<rexx_exec::Interp>::begin_invoke+0xc41>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  26e037:	mov    $0x8,%edi
  26e03c:	mov    $0x18,%esi
  26e041:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e046:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
  26e048:	mov    $0x8,%edi
  26e04d:	mov    $0x70,%esi
  26e052:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e057:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<rexx_exec::Interp>::sends_guarded:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/executable.rs:85
  26e059:	movzbl 0x48(%rsp),%edx
  26e05e:	mov    %r14,%rdi
  26e061:	mov    %rsi,0x38(%rsp)
  26e066:	mov    0x70(%rsp),%esi
  26e06a:	call   26e820 <<rexx_exec::Interp>::written_guard>
  26e06f:	mov    0x38(%rsp),%rsi
  26e074:	mov    %rax,0x48(%rsp)
  26e079:	jmp    26cfc5 <<rexx_exec::Interp>::begin_invoke+0xc85>
<rexx_exec::Interp>::begin_method:
  26e07e:	xor    %eax,%eax
<rexx_exec::activity::Activity>::invocation_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:434
  26e080:	test   %al,%al
  26e082:	jne    26d645 <<rexx_exec::Interp>::begin_invoke+0x1305>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activity.rs:436
  26e088:	mov    %r14,%rdi
  26e08b:	mov    0x540(%rsp),%rsi
  26e093:	mov    0x548(%rsp),%rdx
  26e09b:	call   25e0c0 <<rexx_exec::activity::Activity>::new_invocation_name>
  26e0a0:	mov    %rdx,0x80(%rsp)
<rexx_exec::Interp>::begin_method:
  26e0a8:	mov    %rax,%rsi
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e0ab:	incq   (%rsi)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  26e0ae:	jne    26d688 <<rexx_exec::Interp>::begin_invoke+0x1348>
<rexx_exec::Interp>::begin_method:
  26e0b4:	ud2
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  26e0b6:	lea    -0x204263(%rip),%rdi        # 69e5a <panic_unwind::imp::CANARY+0x50fd>
  26e0bd:	lea    0x2520e4(%rip),%rdx        # 4c01a8 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  26e0c4:	mov    $0x11,%esi
  26e0c9:	call   11f010 <core::option::expect_failed>
  26e0ce:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  26e0d0:	lea    -0x203459(%rip),%rdi        # 6ac7e <panic_unwind::imp::CANARY+0x5f21>
  26e0d7:	lea    0x2527ea(%rip),%rdx        # 4c08c8 <rexx_api::ffi::CALL_CONTEXT+0x5538>
  26e0de:	mov    $0x31,%esi
  26e0e3:	call   11c7f0 <core::panicking::panic>
  26e0e8:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<rexx_exec::Interp>::begin_invoke:
  26e0ea:	mov    %r12,%rsi
<alloc::boxed::Box<[u8]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:847
  26e0ed:	mov    $0x1,%edi
  26e0f2:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e0f7:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
core::mem::size_of_val::<[core::option::Option<rexx_core::handle::ObjRef>]>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:412
  26e0f9:	shl    $0x4,%rbp
<alloc::boxed::Box<[core::option::Option<rexx_core::handle::ObjRef>]>>::clone_from_ref_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:847
  26e0fd:	mov    $0x8,%edi
  26e102:	mov    %rbp,%rsi
  26e105:	call   111b17 <alloc::alloc::handle_alloc_error>
  26e10a:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:311
  26e10c:	lea    0xab0(%r14),%rdi
  26e113:	mov    %r12,%rsi
  26e116:	mov    %r15,%rdx
  26e119:	mov    %r13,%rcx
  26e11c:	mov    0x20(%rsp),%r8
  26e121:	call   1607c0 <<rexx_core::roots::RootSet>::set_frame_slot_of>
  26e126:	jmp    26dc18 <<rexx_exec::Interp>::begin_invoke+0x18d8>
  26e12b:	lea    0xab0(%r14),%rdi
  26e132:	mov    %r12,%rsi
  26e135:	mov    %r15,%rdx
  26e138:	mov    %rbx,%rcx
  26e13b:	call   1607c0 <<rexx_core::roots::RootSet>::set_frame_slot_of>
  26e140:	jmp    26dcb7 <<rexx_exec::Interp>::begin_invoke+0x1977>
<rexx_core::body::Body>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1440
  26e145:	mov    0x10(%rax),%rax
<rexx_core::body::NativeObject>::package:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1350
  26e149:	cmpl   $0x5,(%rax)
  26e14c:	jne    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:1351
  26e14e:	mov    0x8(%rax),%rax
<rexx_core::body::PackageState>::downcast_ref::<rexx_exec::package_state::PackageData>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/body.rs:229
  26e152:	mov    (%rax),%rsi
  26e155:	mov    0x8(%rax),%rax
  26e159:	lea    0x100(%rsp),%rdi
  26e161:	mov    %rsi,0x28(%rsp)
<dyn core::any::Any>::is::<rexx_exec::package_state::PackageData>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/any.rs:201
  26e166:	call   *0x18(%rax)
core::intrinsics::type_id_eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:3128
  26e169:	movdqa 0x100(%rsp),%xmm0
  26e172:	pcmpeqb -0x21de1a(%rip),%xmm0        # 50360 <GCC_except_table5014+0x97c>
  26e17a:	pmovmskb %xmm0,%eax
  26e17e:	cmp    $0xffff,%eax
<rexx_exec::Interp>::package_data:
  26e183:	jne    26e193 <<rexx_exec::Interp>::begin_invoke+0x1e53>
<rexx_exec::Interp>::options_of:
  26e185:	mov    0x28(%rsp),%rax
<core::option::Option<rexx_exec::options::PackageOptions>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  26e18a:	cmpb   $0x2,0xcd(%rax)
<rexx_exec::Interp>::options_of:
  26e191:	jne    26e1ee <<rexx_exec::Interp>::begin_invoke+0x1eae>
<rexx_exec::Interp>::begin_method:
  26e193:	mov    %r15,%rsi
  26e196:	mov    $0x2,%dil
  26e199:	jmp    26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
  26e19e:	mov    %r15,%rsi
  26e1a1:	jmp    26d758 <<rexx_exec::Interp>::begin_invoke+0x1418>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:137
  26e1a6:	mov    %r14,%rdi
  26e1a9:	mov    0x20(%rsp),%rsi
  26e1ae:	mov    0x30(%rsp),%rdx
  26e1b3:	call   26eaa0 <<rexx_exec::Interp>::reserve_for_method>
  26e1b8:	jmp    26db8b <<rexx_exec::Interp>::begin_invoke+0x184b>
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:40
  26e1bd:	mov    %r14,%rdi
  26e1c0:	mov    %r12,%rsi
  26e1c3:	mov    %r15,%rdx
  26e1c6:	mov    %r13,%rcx
  26e1c9:	mov    0x20(%rsp),%r8
  26e1ce:	call   220300 <<rexx_exec::Interp>::set_exposed_variable>
  26e1d3:	jmp    26dc18 <<rexx_exec::Interp>::begin_invoke+0x18d8>
  26e1d8:	mov    %r14,%rdi
  26e1db:	mov    %r12,%rsi
  26e1de:	mov    %r15,%rdx
  26e1e1:	mov    %rbx,%rcx
  26e1e4:	call   220300 <<rexx_exec::Interp>::set_exposed_variable>
  26e1e9:	jmp    26dcb7 <<rexx_exec::Interp>::begin_invoke+0x1977>
<rexx_exec::Interp>::begin_method:
  26e1ee:	mov    0x28(%rsp),%rax
  26e1f3:	add    $0x98,%rax
<core::option::Option<&rexx_exec::activation::Activation>>::map::<&rexx_num::settings::Settings, <rexx_exec::Interp>::begin_method::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  26e1f9:	test   %rbx,%rbx
<rexx_exec::Interp>::package_start:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2700
  26e1fc:	je     26e212 <<rexx_exec::Interp>::begin_invoke+0x1ed2>
  26e1fe:	add    $0x68,%rbx
  26e202:	mov    0x28(%rsp),%rcx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2701
  26e207:	cmpb   $0x0,0xcc(%rcx)
  26e20e:	cmovne %rbx,%rax
<rexx_num::settings::Settings as core::clone::Clone>::clone:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-num/src/settings.rs:106
  26e212:	mov    0x10(%rax),%rcx
  26e216:	mov    %rcx,0xa0(%rsp)
  26e21e:	movdqu (%rax),%xmm0
  26e222:	movdqa %xmm0,0x90(%rsp)
  26e22b:	mov    0x28(%rsp),%rdx
<rexx_exec::Interp>::package_start:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2704
  26e230:	movzbl 0xc6(%rdx),%edi
  26e237:	movzbl 0xcb(%rdx),%eax
  26e23e:	mov    %al,0x54(%rsp)
  26e242:	mov    0xc7(%rdx),%eax
  26e248:	mov    %eax,0x50(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:2705
  26e24c:	movzbl 0xb0(%rdx),%ecx
  26e253:	mov    0xb1(%rdx),%rax
  26e25a:	jmp    26e19e <<rexx_exec::Interp>::begin_invoke+0x1e5e>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  26e25f:	mov    $0x1,%edi
  26e264:	mov    $0x2,%esi
  26e269:	call   111b04 <alloc::raw_vec::handle_error>
  26e26e:	jmp    26e0b4 <<rexx_exec::Interp>::begin_invoke+0x1d74>
<rexx_exec::Interp>::begin_method:
  26e273:	mov    %rax,%r14
  26e276:	lea    0x300(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:179
  26e27e:	call   258b80 <core::ptr::drop_glue::<rexx_exec::CallContext>>
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e283:	decq   (%r15)
<alloc::rc::Rc<[u8]> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e286:	jne    26e305 <<rexx_exec::Interp>::begin_invoke+0x1fc5>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e288:	mov    %r15,%rdi
  26e28b:	mov    0x80(%rsp),%rsi
  26e293:	call   1660e0 <<alloc::rc::Rc<[u8]>>::drop_slow>
  26e298:	jmp    26e305 <<rexx_exec::Interp>::begin_invoke+0x1fc5>
<rexx_exec::Interp>::begin_method:
  26e29a:	mov    %rax,%r14
  26e29d:	mov    0x88(%rsp),%rax
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e2a5:	decq   (%rax)
<alloc::rc::Rc<[core::option::Option<rexx_core::handle::ObjRef>]> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e2a8:	jne    26e305 <<rexx_exec::Interp>::begin_invoke+0x1fc5>
  26e2aa:	mov    0x88(%rsp),%rdi
  26e2b2:	mov    0xc0(%rsp),%rsi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e2ba:	call   1848b0 <<alloc::rc::Rc<[core::option::Option<rexx_core::handle::ObjRef>]>>::drop_slow>
  26e2bf:	jmp    26e305 <<rexx_exec::Interp>::begin_invoke+0x1fc5>
<rexx_exec::Interp>::begin_invoke:
  26e2c1:	jmp    26e2c3 <<rexx_exec::Interp>::begin_invoke+0x1f83>
  26e2c3:	mov    %rax,%r14
  26e2c6:	lea    0x330(%rsp),%rdi
  26e2ce:	call   2eb2d0 <core::ptr::drop_glue::<core::result::Result<core::option::Option<rexx_exec::clause::HandlerExit>, rexx_exec::error::Failure>>>
  26e2d3:	jmp    26e390 <<rexx_exec::Interp>::begin_invoke+0x2050>
  26e2d8:	mov    %rax,%r14
  26e2db:	lea    0x330(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:179
  26e2e3:	call   1a3570 <core::ptr::drop_glue::<rexx_exec::activation::Activation>>
  26e2e8:	jmp    26e383 <<rexx_exec::Interp>::begin_invoke+0x2043>
  26e2ed:	mov    %rax,%r14
  26e2f0:	lea    0x100(%rsp),%rdi
<alloc::vec::Vec<rexx_exec::run::call::CallTail>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1042
  26e2f8:	call   25e190 <core::ptr::drop_glue::<rexx_exec::run::call::CallTail>>
  26e2fd:	jmp    26e390 <<rexx_exec::Interp>::begin_invoke+0x2050>
<rexx_exec::Interp>::begin_method:
  26e302:	mov    %rax,%r14
  26e305:	mov    0x38(%rsp),%rax
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e30a:	decq   (%rax)
<alloc::rc::Rc<rexx_exec::plan::Plan> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e30d:	jne    26e3ea <<rexx_exec::Interp>::begin_invoke+0x20aa>
  26e313:	mov    0x38(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e318:	call   168c30 <<alloc::rc::Rc<rexx_exec::plan::Plan>>::drop_slow>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  26e31d:	jmp    26e3ea <<rexx_exec::Interp>::begin_invoke+0x20aa>
<rexx_exec::Interp>::begin_invoke:
  26e322:	jmp    26e324 <<rexx_exec::Interp>::begin_invoke+0x1fe4>
  26e324:	mov    %rax,%r14
  26e327:	jmp    26e3ea <<rexx_exec::Interp>::begin_invoke+0x20aa>
  26e32c:	mov    %rax,%r14
  26e32f:	jmp    26e421 <<rexx_exec::Interp>::begin_invoke+0x20e1>
  26e334:	jmp    26e37b <<rexx_exec::Interp>::begin_invoke+0x203b>
  26e336:	mov    %rax,%r14
  26e339:	jmp    26e352 <<rexx_exec::Interp>::begin_invoke+0x2012>
  26e33b:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e33e:	cmpq   $0x0,0x90(%rsp)
  26e347:	je     26e352 <<rexx_exec::Interp>::begin_invoke+0x2012>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e349:	mov    %rbx,%rdi
  26e34c:	call   *0x269c46(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2800
  26e352:	lea    0x60(%rsp),%rdi
  26e357:	call   1ac670 <core::ptr::drop_glue::<rexx_exec::error::Failure>>
  26e35c:	jmp    26e390 <<rexx_exec::Interp>::begin_invoke+0x2050>
  26e35e:	mov    %rax,%r14
<alloc::boxed::Box<[u8]> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:1999
  26e361:	test   %r12,%r12
  26e364:	je     26e463 <<rexx_exec::Interp>::begin_invoke+0x2123>
<rexx_exec::Interp>::begin_invoke:
  26e36a:	mov    %rbx,%rdi
  26e36d:	call   *0x269c25(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  26e373:	mov    %r14,%rdi
  26e376:	call   4b1640 <_Unwind_Resume@plt>
  26e37b:	mov    %rax,%r14
  26e37e:	jmp    26e390 <<rexx_exec::Interp>::begin_invoke+0x2050>
  26e380:	mov    %rax,%r14
  26e383:	lea    0x300(%rsp),%rdi
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:179
  26e38b:	call   258b80 <core::ptr::drop_glue::<rexx_exec::CallContext>>
core::ptr::drop_glue::<rexx_exec::dispatch::Invocable>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  26e390:	cmpq   $0x5,0x40(%rsp)
  26e396:	jb     26e463 <<rexx_exec::Interp>::begin_invoke+0x2123>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2808
  26e39c:	mov    0x2f8(%rsp),%rdi
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e3a4:	decq   (%rdi)
<alloc::rc::Rc<rexx_api::load::Library> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e3a7:	jne    26e463 <<rexx_exec::Interp>::begin_invoke+0x2123>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e3ad:	call   162d60 <<alloc::rc::Rc<rexx_api::load::Library>>::drop_slow>
  26e3b2:	jmp    26e463 <<rexx_exec::Interp>::begin_invoke+0x2123>
<rexx_exec::Interp>::begin_invoke:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch.rs:2720
  26e3b7:	call   11f113 <core::panicking::panic_in_cleanup>
  26e3bc:	mov    %rax,%r14
  26e3bf:	lea    0x100(%rsp),%rdi
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e3c7:	call   16ce90 <core::ptr::drop_glue::<rexx_exec::error::Raised>>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  26e3cc:	jmp    26e3ea <<rexx_exec::Interp>::begin_invoke+0x20aa>
<rexx_exec::Interp>::begin_method:
  26e3ce:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e3d1:	cmpq   $0x0,0x100(%rsp)
  26e3da:	je     26e3ea <<rexx_exec::Interp>::begin_invoke+0x20aa>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e3dc:	mov    0x108(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e3e4:	call   *0x269bae(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::begin_method:
  26e3ea:	mov    0x18(%rsp),%rax
core::mem::replace::<usize>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  26e3ef:	decq   (%rax)
<alloc::rc::Rc<rexx_parse::Program> as core::ops::drop::Drop>::drop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2500
  26e3f2:	jne    26e463 <<rexx_exec::Interp>::begin_invoke+0x2123>
  26e3f4:	mov    0x18(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/rc.rs:2501
  26e3f9:	call   191f90 <<alloc::rc::Rc<rexx_parse::Program>>::drop_slow>
  26e3fe:	jmp    26e463 <<rexx_exec::Interp>::begin_invoke+0x2123>
<rexx_exec::Interp>::begin_method:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/method_entry.rs:23
  26e400:	call   11f113 <core::panicking::panic_in_cleanup>
<rexx_exec::Interp>::begin_invoke:
  26e405:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e408:	cmpq   $0x0,0x100(%rsp)
  26e411:	je     26e421 <<rexx_exec::Interp>::begin_invoke+0x20e1>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e413:	mov    0x108(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e41b:	call   *0x269b77(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e421:	cmpq   $0x0,0x330(%rsp)
  26e42a:	je     26e463 <<rexx_exec::Interp>::begin_invoke+0x2123>
<rexx_exec::Interp>::begin_invoke:
  26e42c:	mov    %rbx,%rdi
  26e42f:	call   *0x269b63(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  26e435:	mov    %r14,%rdi
  26e438:	call   4b1640 <_Unwind_Resume@plt>
  26e43d:	mov    %rax,%r14
  26e440:	lea    0x100(%rsp),%rdi
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e448:	call   16ce90 <core::ptr::drop_glue::<rexx_exec::error::Raised>>
  26e44d:	mov    %r14,%rdi
  26e450:	call   4b1640 <_Unwind_Resume@plt>
<rexx_exec::Interp>::begin_invoke:
  26e455:	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  26e458:	cmpq   $0x0,0x100(%rsp)
  26e461:	jne    26e46b <<rexx_exec::Interp>::begin_invoke+0x212b>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  26e463:	mov    %r14,%rdi
  26e466:	call   4b1640 <_Unwind_Resume@plt>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  26e46b:	mov    0x108(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  26e473:	call   *0x269b1f(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  26e479:	mov    %r14,%rdi
  26e47c:	call   4b1640 <_Unwind_Resume@plt>


/tmp/codex-p9d-t88-pins/8.8a-perf-r1-rexx:     file format elf64-x86-64


Disassembly of section .text:

000000000030f420 <<rexx_exec::Interp>::exec_parse>:
<rexx_exec::Interp>::exec_parse:
  30f420:	55                   	push   %rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:474
  30f421:	41 57                	push   %r15
  30f423:	41 56                	push   %r14
  30f425:	41 55                	push   %r13
  30f427:	41 54                	push   %r12
  30f429:	53                   	push   %rbx
  30f42a:	48 81 ec 48 02 00 00 	sub    $0x248,%rsp
  30f431:	48 89 bc 24 50 01 00 	mov    %rdi,0x150(%rsp)
  30f438:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:480
  30f439:	48 8b 86 50 04 00 00 	mov    0x450(%rsi),%rax
  30f440:	48 89 84 24 c0 00 00 	mov    %rax,0xc0(%rsp)
  30f447:	00 
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:564
  30f448:	48 8b 41 18          	mov    0x18(%rcx),%rax
  30f44c:	48 bf f1 ff ff ff ff 	movabs $0x7ffffffffffffff1,%rdi
  30f453:	ff ff 7f 
  30f456:	48 01 c7             	add    %rax,%rdi
  30f459:	48 83 ff 05          	cmp    $0x5,%rdi
  30f45d:	48 89 74 24 18       	mov    %rsi,0x18(%rsp)
  30f462:	48 89 94 24 f0 00 00 	mov    %rdx,0xf0(%rsp)
  30f469:	00 
  30f46a:	48 89 8c 24 b0 00 00 	mov    %rcx,0xb0(%rsp)
  30f471:	00 
  30f472:	77 1e                	ja     30f492 <<rexx_exec::Interp>::exec_parse+0x72>
  30f474:	48 8d 05 89 4f d4 ff 	lea    -0x2bb077(%rip),%rax        # 54404 <GCC_except_table5013+0x4a3c>
  30f47b:	48 63 3c b8          	movslq (%rax,%rdi,4),%rdi
  30f47f:	48 01 c7             	add    %rax,%rdi
  30f482:	ff e7                	jmp    *%rdi
  30f484:	49 c7 c7 fe ff ff ff 	mov    $0xfffffffffffffffe,%r15
  30f48b:	31 ed                	xor    %ebp,%ebp
  30f48d:	e9 ab 0d 00 00       	jmp    31023d <<rexx_exec::Interp>::exec_parse+0xe1d>
  30f492:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
  30f496:	0f 85 01 01 00 00    	jne    30f59d <<rexx_exec::Interp>::exec_parse+0x17d>
  30f49c:	bd 01 00 00 00       	mov    $0x1,%ebp
  30f4a1:	4c 8d 2d 61 77 d9 ff 	lea    -0x26889f(%rip),%r13        # a6c09 <panic_unwind::imp::CANARY+0x41ea4>
  30f4a8:	bb 05 00 00 00       	mov    $0x5,%ebx
  30f4ad:	45 31 ff             	xor    %r15d,%r15d
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30f4b0:	80 be 35 05 00 00 00 	cmpb   $0x0,0x535(%rsi)
  30f4b7:	75 23                	jne    30f4dc <<rexx_exec::Interp>::exec_parse+0xbc>
  30f4b9:	f6 86 2e 05 00 00 01 	testb  $0x1,0x52e(%rsi)
  30f4c0:	74 1a                	je     30f4dc <<rexx_exec::Interp>::exec_parse+0xbc>
<rexx_exec::Interp>::trace_literal:
  30f4c2:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:342
  30f4c5:	ba 01 00 00 00       	mov    $0x1,%edx
  30f4ca:	48 89 f7             	mov    %rsi,%rdi
  30f4cd:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  30f4d4:	00 
  30f4d5:	31 c9                	xor    %ecx,%ecx
  30f4d7:	e8 84 1a f7 ff       	call   280f60 <<rexx_exec::Interp>::trace_literal_line>
<rexx_exec::Interp>::parse_strings:
  30f4dc:	45 31 e4             	xor    %r12d,%r12d
  30f4df:	e9 da 03 00 00       	jmp    30f8be <<rexx_exec::Interp>::exec_parse+0x49e>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::pop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2873
  30f4e4:	48 8b 86 e0 0b 00 00 	mov    0xbe0(%rsi),%rax
  30f4eb:	48 85 c0             	test   %rax,%rax
  30f4ee:	0f 84 e8 01 00 00    	je     30f6dc <<rexx_exec::Interp>::exec_parse+0x2bc>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2877
  30f4f4:	48 ff c8             	dec    %rax
  30f4f7:	48 89 86 e0 0b 00 00 	mov    %rax,0xbe0(%rsi)
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f4fe:	48 8b 8e d8 0b 00 00 	mov    0xbd8(%rsi),%rcx
<*const alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs:871
  30f505:	48 8d 04 40          	lea    (%rax,%rax,2),%rax
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30f509:	4c 8b 3c c1          	mov    (%rcx,%rax,8),%r15
  30f50d:	f3 0f 6f 44 c1 08    	movdqu 0x8(%rcx,%rax,8),%xmm0
  30f513:	f3 0f 7f 44 24 48    	movdqu %xmm0,0x48(%rsp)
<alloc::vec::Vec<u8>>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1461
  30f519:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30f51e:	e9 d1 01 00 00       	jmp    30f6f4 <<rexx_exec::Interp>::exec_parse+0x2d4>
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:1784
  30f523:	48 8b 86 20 11 00 00 	mov    0x1120(%rsi),%rax
  30f52a:	48 85 c0             	test   %rax,%rax
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::pop_front:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2198
  30f52d:	0f 84 3f 01 00 00    	je     30f672 <<rexx_exec::Interp>::exec_parse+0x252>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2201
  30f533:	48 8b 96 18 11 00 00 	mov    0x1118(%rsi),%rdx
  30f53a:	49 89 f1             	mov    %rsi,%r9
<usize>::wrapping_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2652
  30f53d:	48 8d 72 01          	lea    0x1(%rdx),%rsi
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30f541:	49 8b b9 08 11 00 00 	mov    0x1108(%r9),%rdi
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f548:	49 8b 89 10 11 00 00 	mov    0x1110(%r9),%rcx
  30f54f:	45 31 c0             	xor    %r8d,%r8d
alloc::collections::vec_deque::index::wrap_index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:3700
  30f552:	48 39 fe             	cmp    %rdi,%rsi
  30f555:	4c 0f 43 c7          	cmovae %rdi,%r8
  30f559:	49 f7 d8             	neg    %r8
  30f55c:	4a 8d 34 02          	lea    (%rdx,%r8,1),%rsi
  30f560:	48 ff c6             	inc    %rsi
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::pop_front:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2202
  30f563:	49 89 b1 18 11 00 00 	mov    %rsi,0x1118(%r9)
  30f56a:	4c 89 ce             	mov    %r9,%rsi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2203
  30f56d:	48 ff c8             	dec    %rax
  30f570:	49 89 81 20 11 00 00 	mov    %rax,0x1120(%r9)
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30f577:	48 8d 04 52          	lea    (%rdx,%rdx,2),%rax
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30f57b:	4c 8b 3c c1          	mov    (%rcx,%rax,8),%r15
<rexx_exec::Interp>::pull_line:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:576
  30f57f:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30f583:	0f 84 e9 00 00 00    	je     30f672 <<rexx_exec::Interp>::exec_parse+0x252>
  30f589:	48 8d 04 c1          	lea    (%rcx,%rax,8),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:577
  30f58d:	f3 0f 6f 40 08       	movdqu 0x8(%rax),%xmm0
  30f592:	f3 0f 7f 44 24 48    	movdqu %xmm0,0x48(%rsp)
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30f598:	e9 ee 00 00 00       	jmp    30f68b <<rexx_exec::Interp>::exec_parse+0x26b>
<rexx_exec::Interp>::parse_strings:
  30f59d:	4c 8d 2d 65 76 d9 ff 	lea    -0x26899b(%rip),%r13        # a6c09 <panic_unwind::imp::CANARY+0x41ea4>
  30f5a4:	bb 05 00 00 00       	mov    $0x5,%ebx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:577
  30f5a9:	41 f6 c0 01          	test   $0x1,%r8b
  30f5ad:	0f 84 f3 00 00 00    	je     30f6a6 <<rexx_exec::Interp>::exec_parse+0x286>
  30f5b3:	4d 89 ce             	mov    %r9,%r14
  30f5b6:	e9 4d 09 00 00       	jmp    30ff08 <<rexx_exec::Interp>::exec_parse+0xae8>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::pop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2873
  30f5bb:	48 8b 86 e0 0b 00 00 	mov    0xbe0(%rsi),%rax
  30f5c2:	48 85 c0             	test   %rax,%rax
  30f5c5:	0f 84 8d 01 00 00    	je     30f758 <<rexx_exec::Interp>::exec_parse+0x338>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2877
  30f5cb:	48 ff c8             	dec    %rax
  30f5ce:	48 89 86 e0 0b 00 00 	mov    %rax,0xbe0(%rsi)
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f5d5:	48 8b 8e d8 0b 00 00 	mov    0xbd8(%rsi),%rcx
<*const alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs:871
  30f5dc:	48 8d 14 40          	lea    (%rax,%rax,2),%rdx
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30f5e0:	48 8b 04 d1          	mov    (%rcx,%rdx,8),%rax
  30f5e4:	f3 0f 6f 44 d1 08    	movdqu 0x8(%rcx,%rdx,8),%xmm0
  30f5ea:	f3 0f 7f 44 24 48    	movdqu %xmm0,0x48(%rsp)
<alloc::vec::Vec<u8>>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1461
  30f5f0:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30f5f5:	e9 75 01 00 00       	jmp    30f76f <<rexx_exec::Interp>::exec_parse+0x34f>
<rexx_exec::Interp>::parse_strings:
  30f5fa:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:630
  30f5ff:	e8 4c 68 04 00       	call   355e50 <<rexx_exec::Interp>::linein_line>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30f604:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
  30f609:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  30f60e:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30f613:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30f617:	0f 84 c5 0b 00 00    	je     3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
  30f61d:	4c 8d 2d 01 76 d9 ff 	lea    -0x2689ff(%rip),%r13        # a6c25 <panic_unwind::imp::CANARY+0x41ec0>
  30f624:	bb 06 00 00 00       	mov    $0x6,%ebx
  30f629:	e9 90 02 00 00       	jmp    30f8be <<rexx_exec::Interp>::exec_parse+0x49e>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:591
  30f62e:	8b 59 20             	mov    0x20(%rcx),%ebx
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:673
  30f631:	48 8b 42 08          	mov    0x8(%rdx),%rax
<alloc::vec::Vec<alloc::boxed::Box<str>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  30f635:	48 8b 70 10          	mov    0x10(%rax),%rsi
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  30f639:	48 39 de             	cmp    %rbx,%rsi
  30f63c:	0f 86 51 05 00 00    	jbe    30fb93 <<rexx_exec::Interp>::exec_parse+0x773>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::boxed::Box<str>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f642:	48 8b 40 08          	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  30f646:	48 89 d9             	mov    %rbx,%rcx
  30f649:	48 c1 e1 04          	shl    $0x4,%rcx
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  30f64d:	4c 8b 2c 08          	mov    (%rax,%rcx,1),%r13
  30f651:	4c 8b 64 08 08       	mov    0x8(%rax,%rcx,1),%r12
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  30f656:	4d 85 e4             	test   %r12,%r12
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:25
  30f659:	0f 84 11 06 00 00    	je     30fc70 <<rexx_exec::Interp>::exec_parse+0x850>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  30f65f:	49 83 fc 04          	cmp    $0x4,%r12
  30f663:	0f 83 d9 02 00 00    	jae    30f942 <<rexx_exec::Interp>::exec_parse+0x522>
<rexx_exec::Interp>::parse_strings:
  30f669:	31 c0                	xor    %eax,%eax
  30f66b:	31 c9                	xor    %ecx,%ecx
  30f66d:	e9 5f 05 00 00       	jmp    30fbd1 <<rexx_exec::Interp>::exec_parse+0x7b1>
  30f672:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<rexx_exec::Interp>::pull_line:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:578
  30f677:	e8 d4 67 04 00       	call   355e50 <<rexx_exec::Interp>::linein_line>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30f67c:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
  30f681:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30f685:	0f 84 4b 03 00 00    	je     30f9d6 <<rexx_exec::Interp>::exec_parse+0x5b6>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  30f68b:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  30f690:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30f695:	4c 8d 2d f4 33 d4 ff 	lea    -0x2bcc0c(%rip),%r13        # 52a90 <GCC_except_table5013+0x30c8>
  30f69c:	bb 04 00 00 00       	mov    $0x4,%ebx
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:629
  30f6a1:	e9 18 02 00 00       	jmp    30f8be <<rexx_exec::Interp>::exec_parse+0x49e>
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:127
  30f6a6:	c6 84 24 d8 00 00 00 	movb   $0x0,0xd8(%rsp)
  30f6ad:	00 
  30f6ae:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  30f6b3:	48 8d 94 24 d8 00 00 	lea    0xd8(%rsp),%rdx
  30f6ba:	00 
  30f6bb:	49 89 f4             	mov    %rsi,%r12
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:128
  30f6be:	e8 2d cd f6 ff       	call   27c3f0 <<rexx_exec::Interp>::enter_eval_node::<u8>>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30f6c3:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
  30f6c8:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30f6cc:	0f 84 13 03 00 00    	je     30f9e5 <<rexx_exec::Interp>::exec_parse+0x5c5>
<rexx_exec::Interp>::parse_strings:
  30f6d2:	4c 8b 64 24 48       	mov    0x48(%rsp),%r12
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30f6d7:	e9 06 0b 00 00       	jmp    3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
<alloc::vec::Vec<u8>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:462
  30f6dc:	48 c7 44 24 48 01 00 	movq   $0x1,0x48(%rsp)
  30f6e3:	00 00 
  30f6e5:	48 c7 44 24 50 00 00 	movq   $0x0,0x50(%rsp)
  30f6ec:	00 00 
  30f6ee:	45 31 e4             	xor    %r12d,%r12d
  30f6f1:	45 31 ff             	xor    %r15d,%r15d
<core::option::Option<alloc::vec::Vec<u8>>>::unwrap_or_default:
  30f6f4:	4c 89 7c 24 40       	mov    %r15,0x40(%rsp)
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30f6f9:	4c 89 f8             	mov    %r15,%rax
  30f6fc:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30f6ff:	48 83 f8 2c          	cmp    $0x2c,%rax
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30f703:	0f 86 c1 03 00 00    	jbe    30faca <<rexx_exec::Interp>::exec_parse+0x6aa>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f709:	4c 8b 74 24 48       	mov    0x48(%rsp),%r14
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30f70e:	0f 10 05 1a 99 d5 ff 	movups -0x2a66e6(%rip),%xmm0        # 6902f <panic_unwind::imp::CANARY+0x42ca>
  30f715:	43 0f 11 44 26 1d    	movups %xmm0,0x1d(%r14,%r12,1)
  30f71b:	0f 10 05 00 99 d5 ff 	movups -0x2a6700(%rip),%xmm0        # 69022 <panic_unwind::imp::CANARY+0x42bd>
  30f722:	43 0f 11 44 26 10    	movups %xmm0,0x10(%r14,%r12,1)
  30f728:	f3 0f 6f 05 e2 98 d5 	movdqu -0x2a671e(%rip),%xmm0        # 69012 <panic_unwind::imp::CANARY+0x42ad>
  30f72f:	ff 
  30f730:	f3 43 0f 7f 04 26    	movdqu %xmm0,(%r14,%r12,1)
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30f736:	49 83 c4 2d          	add    $0x2d,%r12
  30f73a:	bb 07 00 00 00       	mov    $0x7,%ebx
  30f73f:	4c 8d 2d f8 74 d9 ff 	lea    -0x268b08(%rip),%r13        # a6c3e <panic_unwind::imp::CANARY+0x41ed9>
  30f746:	4c 89 f5             	mov    %r14,%rbp
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:632
  30f749:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30f74d:	0f 85 6b 01 00 00    	jne    30f8be <<rexx_exec::Interp>::exec_parse+0x49e>
  30f753:	e9 b0 07 00 00       	jmp    30ff08 <<rexx_exec::Interp>::exec_parse+0xae8>
<alloc::vec::Vec<u8>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:462
  30f758:	48 c7 44 24 48 01 00 	movq   $0x1,0x48(%rsp)
  30f75f:	00 00 
  30f761:	48 c7 44 24 50 00 00 	movq   $0x0,0x50(%rsp)
  30f768:	00 00 
  30f76a:	45 31 e4             	xor    %r12d,%r12d
  30f76d:	31 c0                	xor    %eax,%eax
<core::option::Option<alloc::vec::Vec<u8>>>::unwrap_or_default:
  30f76f:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30f774:	48 89 c1             	mov    %rax,%rcx
  30f777:	4c 29 e1             	sub    %r12,%rcx
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30f77a:	48 83 f9 04          	cmp    $0x4,%rcx
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30f77e:	0f 86 72 03 00 00    	jbe    30faf6 <<rexx_exec::Interp>::exec_parse+0x6d6>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f784:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30f789:	43 c6 44 27 04 58    	movb   $0x58,0x4(%r15,%r12,1)
  30f78f:	43 c7 04 27 4c 49 4e 	movl   $0x554e494c,(%r15,%r12,1)
  30f796:	55 
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30f797:	49 8d 4c 24 05       	lea    0x5(%r12),%rcx
  30f79c:	48 89 4c 24 50       	mov    %rcx,0x50(%rsp)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30f7a1:	48 39 c1             	cmp    %rax,%rcx
  30f7a4:	75 0f                	jne    30f7b5 <<rexx_exec::Interp>::exec_parse+0x395>
  30f7a6:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30f7ab:	e8 70 26 e0 ff       	call   111e20 <<alloc::raw_vec::RawVec<u8>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f7b0:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
core::ptr::write::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30f7b5:	43 c6 44 27 05 20    	movb   $0x20,0x5(%r15,%r12,1)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30f7bb:	49 83 c4 06          	add    $0x6,%r12
  30f7bf:	4c 89 64 24 50       	mov    %r12,0x50(%rsp)
  30f7c4:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30f7c9:	48 8b 80 70 04 00 00 	mov    0x470(%rax),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  30f7d0:	48 85 c0             	test   %rax,%rax
  30f7d3:	0f 84 d4 02 00 00    	je     30faad <<rexx_exec::Interp>::exec_parse+0x68d>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30f7d9:	0f b6 80 d6 01 00 00 	movzbl 0x1d6(%rax),%eax
<rexx_exec::activation::CallType>::token:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:472
  30f7e0:	48 8d 0d 61 a2 d9 ff 	lea    -0x265d9f(%rip),%rcx        # a9a48 <core::unicode::unicode_data::white_space::WHITESPACE_MAP+0x16c2>
  30f7e7:	0f b6 1c 08          	movzbl (%rax,%rcx,1),%ebx
  30f7eb:	4c 8d 35 5e a2 d9 ff 	lea    -0x265da2(%rip),%r14        # a9a50 <core::unicode::unicode_data::white_space::WHITESPACE_MAP+0x16ca>
  30f7f2:	4d 63 2c 86          	movslq (%r14,%rax,4),%r13
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30f7f6:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30f7fb:	48 89 e8             	mov    %rbp,%rax
  30f7fe:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30f801:	48 39 d8             	cmp    %rbx,%rax
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30f804:	0f 82 18 03 00 00    	jb     30fb22 <<rexx_exec::Interp>::exec_parse+0x702>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30f80a:	4d 01 ee             	add    %r13,%r14
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30f80d:	4b 8d 3c 27          	lea    (%r15,%r12,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30f811:	4c 89 f6             	mov    %r14,%rsi
  30f814:	48 89 da             	mov    %rbx,%rdx
  30f817:	ff 15 1b 84 1c 00    	call   *0x1c841b(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30f81d:	49 01 dc             	add    %rbx,%r12
  30f820:	4c 89 64 24 50       	mov    %r12,0x50(%rsp)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30f825:	49 39 ec             	cmp    %rbp,%r12
  30f828:	75 0f                	jne    30f839 <<rexx_exec::Interp>::exec_parse+0x419>
  30f82a:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30f82f:	e8 ec 25 e0 ff       	call   111e20 <<alloc::raw_vec::RawVec<u8>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f834:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
core::ptr::write::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30f839:	43 c6 04 27 20       	movb   $0x20,(%r15,%r12,1)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30f83e:	49 ff c4             	inc    %r12
  30f841:	4c 89 64 24 50       	mov    %r12,0x50(%rsp)
  30f846:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:610
  30f84b:	48 8b 87 70 04 00 00 	mov    0x470(%rdi),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  30f852:	48 85 c0             	test   %rax,%rax
  30f855:	0f 84 52 02 00 00    	je     30faad <<rexx_exec::Interp>::exec_parse+0x68d>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:610
  30f85b:	48 8b b0 f0 00 00 00 	mov    0xf0(%rax),%rsi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:611
  30f862:	e8 39 c6 ea ff       	call   1bbea0 <<rexx_exec::Interp>::program_display_name>
  30f867:	49 89 c5             	mov    %rax,%r13
  30f86a:	48 89 d3             	mov    %rdx,%rbx
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30f86d:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30f872:	4c 89 f8             	mov    %r15,%rax
  30f875:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30f878:	48 39 c2             	cmp    %rax,%rdx
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30f87b:	0f 87 d0 02 00 00    	ja     30fb51 <<rexx_exec::Interp>::exec_parse+0x731>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2981
  30f881:	48 85 db             	test   %rbx,%rbx
  30f884:	74 17                	je     30f89d <<rexx_exec::Interp>::exec_parse+0x47d>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f886:	4c 8b 74 24 48       	mov    0x48(%rsp),%r14
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30f88b:	4b 8d 3c 26          	lea    (%r14,%r12,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30f88f:	4c 89 ee             	mov    %r13,%rsi
  30f892:	48 89 da             	mov    %rbx,%rdx
  30f895:	ff 15 9d 83 1c 00    	call   *0x1c839d(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
  30f89b:	eb 05                	jmp    30f8a2 <<rexx_exec::Interp>::exec_parse+0x482>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:612
  30f89d:	4c 8b 74 24 48       	mov    0x48(%rsp),%r14
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30f8a2:	49 01 dc             	add    %rbx,%r12
  30f8a5:	bb 06 00 00 00       	mov    $0x6,%ebx
  30f8aa:	4c 8d 2d 7f 73 d9 ff 	lea    -0x268c81(%rip),%r13        # a6c30 <panic_unwind::imp::CANARY+0x41ecb>
  30f8b1:	4c 89 f5             	mov    %r14,%rbp
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:632
  30f8b4:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30f8b8:	0f 84 4a 06 00 00    	je     30ff08 <<rexx_exec::Interp>::exec_parse+0xae8>
  30f8be:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30f8c3:	80 b8 35 05 00 00 00 	cmpb   $0x0,0x535(%rax)
  30f8ca:	75 67                	jne    30f933 <<rexx_exec::Interp>::exec_parse+0x513>
  30f8cc:	f6 80 2d 05 00 00 01 	testb  $0x1,0x52d(%rax)
  30f8d3:	74 5e                	je     30f933 <<rexx_exec::Interp>::exec_parse+0x513>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30f8d5:	48 8d b8 d8 10 00 00 	lea    0x10d8(%rax),%rdi
  30f8dc:	4c 8b b0 e8 10 00 00 	mov    0x10e8(%rax),%r14
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:298
  30f8e3:	4c 89 64 24 10       	mov    %r12,0x10(%rsp)
  30f8e8:	48 89 6c 24 08       	mov    %rbp,0x8(%rsp)
  30f8ed:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  30f8f4:	48 8d 35 8a b2 d5 ff 	lea    -0x2a4d76(%rip),%rsi        # 6ab85 <panic_unwind::imp::CANARY+0x5e20>
  30f8fb:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  30f902:	00 
  30f903:	b9 01 00 00 00       	mov    $0x1,%ecx
  30f908:	4d 89 e8             	mov    %r13,%r8
  30f90b:	49 89 d9             	mov    %rbx,%r9
  30f90e:	e8 3d 0a f7 ff       	call   280350 <rexx_exec::trace::format::push_tagged>
<[u8] as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:21
  30f913:	83 fb 04             	cmp    $0x4,%ebx
  30f916:	75 0e                	jne    30f926 <<rexx_exec::Interp>::exec_parse+0x506>
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  30f918:	41 81 7d 00 57 48 45 	cmpl   $0x4e454857,0x0(%r13)
  30f91f:	4e 
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:307
  30f920:	0f 84 55 02 00 00    	je     30fb7b <<rexx_exec::Interp>::exec_parse+0x75b>
  30f926:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:310
  30f92b:	4c 89 f6             	mov    %r14,%rsi
  30f92e:	e8 fd e6 e8 ff       	call   19e030 <<rexx_exec::Interp>::route_trace_line>
<core::result::Result<rexx_exec::parse_template::ParseStrings, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30f933:	49 83 ff fd          	cmp    $0xfffffffffffffffd,%r15
  30f937:	0f 84 a5 08 00 00    	je     3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
  30f93d:	e9 fb 08 00 00       	jmp    31023d <<rexx_exec::Interp>::exec_parse+0xe1d>
<rexx_exec::Interp>::exec_parse:
  30f942:	4c 89 e0             	mov    %r12,%rax
  30f945:	48 83 e0 fc          	and    $0xfffffffffffffffc,%rax
  30f949:	66 0f ef c0          	pxor   %xmm0,%xmm0
  30f94d:	31 c9                	xor    %ecx,%ecx
  30f94f:	66 0f 6f 0d f9 05 d4 	movdqa -0x2bfa07(%rip),%xmm1        # 4ff50 <GCC_except_table5013+0x588>
  30f956:	ff 
  30f957:	66 0f 6f 1d 71 0a d4 	movdqa -0x2bf58f(%rip),%xmm3        # 503d0 <GCC_except_table5013+0xa08>
  30f95e:	ff 
  30f95f:	66 0f ef d2          	pxor   %xmm2,%xmm2
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
  30f963:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  30f96a:	84 00 00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:274
  30f970:	41 0f b7 54 0d 00    	movzwl 0x0(%r13,%rcx,1),%edx
  30f976:	66 0f 6e e2          	movd   %edx,%xmm4
  30f97a:	41 0f b7 54 0d 02    	movzwl 0x2(%r13,%rcx,1),%edx
  30f980:	66 0f 6e ea          	movd   %edx,%xmm5
<core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize::<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/filter.rs:138
  30f984:	66 0f 74 e1          	pcmpeqb %xmm1,%xmm4
  30f988:	66 0f 60 e4          	punpcklbw %xmm4,%xmm4
  30f98c:	66 0f 61 e4          	punpcklwd %xmm4,%xmm4
  30f990:	66 0f 70 e4 d4       	pshufd $0xd4,%xmm4,%xmm4
  30f995:	66 0f db e3          	pand   %xmm3,%xmm4
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  30f999:	66 0f d4 d4          	paddq  %xmm4,%xmm2
<core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize::<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/filter.rs:138
  30f99d:	66 0f 74 e9          	pcmpeqb %xmm1,%xmm5
  30f9a1:	66 0f 60 ed          	punpcklbw %xmm5,%xmm5
  30f9a5:	66 0f 61 ed          	punpcklwd %xmm5,%xmm5
  30f9a9:	66 0f 70 e5 d4       	pshufd $0xd4,%xmm5,%xmm4
  30f9ae:	66 0f db e3          	pand   %xmm3,%xmm4
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  30f9b2:	66 0f d4 c4          	paddq  %xmm4,%xmm0
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
  30f9b6:	48 83 c1 04          	add    $0x4,%rcx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  30f9ba:	48 39 c8             	cmp    %rcx,%rax
  30f9bd:	75 b1                	jne    30f970 <<rexx_exec::Interp>::exec_parse+0x550>
  30f9bf:	66 0f d4 c2          	paddq  %xmm2,%xmm0
  30f9c3:	66 0f 70 c8 ee       	pshufd $0xee,%xmm0,%xmm1
  30f9c8:	66 0f d4 c8          	paddq  %xmm0,%xmm1
  30f9cc:	66 48 0f 7e c9       	movq   %xmm1,%rcx
  30f9d1:	e9 0c 02 00 00       	jmp    30fbe2 <<rexx_exec::Interp>::exec_parse+0x7c2>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  30f9d6:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  30f9db:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30f9e0:	e9 fd 07 00 00       	jmp    3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
<rexx_exec::Interp>::parse_strings:
  30f9e5:	48 8b 84 24 b0 00 00 	mov    0xb0(%rsp),%rax
  30f9ec:	00 
  30f9ed:	4c 8d 78 18          	lea    0x18(%rax),%r15
  30f9f1:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  30f9f8:	00 
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:129
  30f9f9:	4c 89 e6             	mov    %r12,%rsi
  30f9fc:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  30fa03:	00 
  30fa04:	4c 89 f9             	mov    %r15,%rcx
  30fa07:	e8 44 cc f6 ff       	call   27c650 <<rexx_exec::Interp>::eval_node>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:130
  30fa0c:	49 ff 8c 24 c8 04 00 	decq   0x4c8(%r12)
  30fa13:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:131
  30fa14:	48 8b ac 24 00 01 00 	mov    0x100(%rsp),%rbp
  30fa1b:	00 
  30fa1c:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30fa20:	74 0d                	je     30fa2f <<rexx_exec::Interp>::exec_parse+0x60f>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  30fa22:	4c 8b a4 24 08 01 00 	mov    0x108(%rsp),%r12
  30fa29:	00 
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fa2a:	e9 b3 07 00 00       	jmp    3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
<rexx_exec::Interp>::parse_strings:
  30fa2f:	4c 89 e1             	mov    %r12,%rcx
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  30fa32:	41 f6 84 24 36 05 00 	testb  $0x4,0x536(%r12)
  30fa39:	00 04 
<rexx_exec::Interp>::eval_in_parse:
  30fa3b:	4c 8b b4 24 08 01 00 	mov    0x108(%rsp),%r14
  30fa42:	00 
<rexx_exec::Interp>::trace_intermediate:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:161
  30fa43:	74 2d                	je     30fa72 <<rexx_exec::Interp>::exec_parse+0x652>
  30fa45:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:164
  30fa4a:	4c 89 e6             	mov    %r12,%rsi
  30fa4d:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  30fa54:	00 
  30fa55:	4c 89 f9             	mov    %r15,%rcx
  30fa58:	4d 89 f0             	mov    %r14,%r8
  30fa5b:	e8 10 f7 f6 ff       	call   27f170 <<rexx_exec::Interp>::trace_intermediate_event>
  30fa60:	4c 89 e1             	mov    %r12,%rcx
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fa63:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
  30fa68:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30fa6c:	0f 85 60 fc ff ff    	jne    30f6d2 <<rexx_exec::Interp>::exec_parse+0x2b2>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  30fa72:	4c 8b b9 f0 0a 00 00 	mov    0xaf0(%rcx),%r15
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30fa79:	4c 3b b9 e0 0a 00 00 	cmp    0xae0(%rcx),%r15
  30fa80:	74 1a                	je     30fa9c <<rexx_exec::Interp>::exec_parse+0x67c>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fa82:	48 8b 81 e8 0a 00 00 	mov    0xae8(%rcx),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30fa89:	4e 89 34 f8          	mov    %r14,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30fa8d:	49 ff c7             	inc    %r15
  30fa90:	4c 89 b9 f0 0a 00 00 	mov    %r15,0xaf0(%rcx)
  30fa97:	e9 6c 04 00 00       	jmp    30ff08 <<rexx_exec::Interp>::exec_parse+0xae8>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30fa9c:	48 8d b9 e0 0a 00 00 	lea    0xae0(%rcx),%rdi
  30faa3:	e8 08 c4 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  30faa8:	4c 89 e1             	mov    %r12,%rcx
  30faab:	eb d5                	jmp    30fa82 <<rexx_exec::Interp>::exec_parse+0x662>
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  30faad:	48 8d 3d ae a3 d5 ff 	lea    -0x2a5c52(%rip),%rdi        # 69e62 <panic_unwind::imp::CANARY+0x50fd>
  30fab4:	48 8d 15 45 03 1b 00 	lea    0x1b0345(%rip),%rdx        # 4bfe00 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  30fabb:	be 11 00 00 00       	mov    $0x11,%esi
  30fac0:	e8 eb f4 e0 ff       	call   11efb0 <core::option::expect_failed>
  30fac5:	e9 be 37 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::Interp>::parse_strings:
  30faca:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30facf:	ba 2d 00 00 00       	mov    $0x2d,%edx
  30fad4:	b9 01 00 00 00       	mov    $0x1,%ecx
  30fad9:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30fadf:	4c 89 e6             	mov    %r12,%rsi
  30fae2:	e8 19 0b e7 ff       	call   180600 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:617
  30fae7:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30faec:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30faf1:	e9 13 fc ff ff       	jmp    30f709 <<rexx_exec::Interp>::exec_parse+0x2e9>
<rexx_exec::Interp>::parse_strings:
  30faf6:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30fafb:	ba 05 00 00 00       	mov    $0x5,%edx
  30fb00:	b9 01 00 00 00       	mov    $0x1,%ecx
  30fb05:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30fb0b:	4c 89 e6             	mov    %r12,%rsi
  30fb0e:	e8 ed 0a e7 ff       	call   180600 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30fb13:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30fb18:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30fb1d:	e9 62 fc ff ff       	jmp    30f784 <<rexx_exec::Interp>::exec_parse+0x364>
<rexx_exec::Interp>::parse_strings:
  30fb22:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30fb27:	b9 01 00 00 00       	mov    $0x1,%ecx
  30fb2c:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30fb32:	4c 89 e6             	mov    %r12,%rsi
  30fb35:	48 89 da             	mov    %rbx,%rdx
  30fb38:	e8 c3 0a e7 ff       	call   180600 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30fb3d:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30fb42:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fb47:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  30fb4c:	e9 b9 fc ff ff       	jmp    30f80a <<rexx_exec::Interp>::exec_parse+0x3ea>
<rexx_exec::Interp>::parse_strings:
  30fb51:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30fb56:	b9 01 00 00 00       	mov    $0x1,%ecx
  30fb5b:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30fb61:	4c 89 e6             	mov    %r12,%rsi
  30fb64:	48 89 da             	mov    %rbx,%rdx
  30fb67:	e8 94 0a e7 ff       	call   180600 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:612
  30fb6c:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30fb71:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30fb76:	e9 0b fd ff ff       	jmp    30f886 <<rexx_exec::Interp>::exec_parse+0x466>
<rexx_exec::Interp>::parse_strings:
  30fb7b:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:308
  30fb80:	4c 89 f6             	mov    %r14,%rsi
  30fb83:	48 89 ea             	mov    %rbp,%rdx
  30fb86:	4c 89 e1             	mov    %r12,%rcx
  30fb89:	e8 c2 95 00 00       	call   319150 <<rexx_exec::Interp>::route_when_line>
  30fb8e:	e9 a0 fd ff ff       	jmp    30f933 <<rexx_exec::Interp>::exec_parse+0x513>
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  30fb93:	48 8d 15 e6 32 1c 00 	lea    0x1c32e6(%rip),%rdx        # 4d2e80 <rexx_api::ffi::CALL_CONTEXT+0x17e98>
  30fb9a:	48 89 df             	mov    %rbx,%rdi
  30fb9d:	e8 ff cb e0 ff       	call   11c7a1 <core::panicking::panic_bounds_check>
<rexx_exec::Interp>::parse_strings:
  30fba2:	eb 14                	jmp    30fbb8 <<rexx_exec::Interp>::exec_parse+0x798>
  30fba4:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  30fba7:	4d 85 ff             	test   %r15,%r15
  30fbaa:	0f 84 c7 38 00 00    	je     313477 <<rexx_exec::Interp>::exec_parse+0x4057>
<rexx_exec::Interp>::parse_strings:
  30fbb0:	48 89 ef             	mov    %rbp,%rdi
  30fbb3:	e9 b9 38 00 00       	jmp    313471 <<rexx_exec::Interp>::exec_parse+0x4051>
  30fbb8:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  30fbbb:	48 83 7c 24 40 00    	cmpq   $0x0,0x40(%rsp)
  30fbc1:	0f 84 b0 38 00 00    	je     313477 <<rexx_exec::Interp>::exec_parse+0x4057>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  30fbc7:	48 8b 7c 24 48       	mov    0x48(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  30fbcc:	e9 a0 38 00 00       	jmp    313471 <<rexx_exec::Interp>::exec_parse+0x4051>
rexx_exec::run::shape_of::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:4126
  30fbd1:	31 d2                	xor    %edx,%edx
  30fbd3:	41 80 7c 05 00 2e    	cmpb   $0x2e,0x0(%r13,%rax,1)
  30fbd9:	0f 94 c2             	sete   %dl
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  30fbdc:	48 01 d1             	add    %rdx,%rcx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  30fbdf:	48 ff c0             	inc    %rax
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  30fbe2:	49 39 c4             	cmp    %rax,%r12
  30fbe5:	75 ea                	jne    30fbd1 <<rexx_exec::Interp>::exec_parse+0x7b1>
rexx_exec::run::shape_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:4127
  30fbe7:	48 85 c9             	test   %rcx,%rcx
  30fbea:	0f 84 80 00 00 00    	je     30fc70 <<rexx_exec::Interp>::exec_parse+0x850>
  30fbf0:	48 83 f9 01          	cmp    $0x1,%rcx
  30fbf4:	0f 85 ed 00 00 00    	jne    30fce7 <<rexx_exec::Interp>::exec_parse+0x8c7>
<u8 as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2105
  30fbfa:	43 80 7c 25 ff 2e    	cmpb   $0x2e,-0x1(%r13,%r12,1)
rexx_exec::run::shape_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:4126
  30fc00:	0f 85 e1 00 00 00    	jne    30fce7 <<rexx_exec::Interp>::exec_parse+0x8c7>
<rexx_exec::Interp>::read_parse_var:
  30fc06:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::Interp>::read_stem:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:186
  30fc0b:	4c 89 ff             	mov    %r15,%rdi
  30fc0e:	4c 89 ee             	mov    %r13,%rsi
  30fc11:	4c 89 e2             	mov    %r12,%rdx
  30fc14:	31 c9                	xor    %ecx,%ecx
  30fc16:	e8 35 f4 f0 ff       	call   21f050 <<rexx_exec::Interp>::read_stem_at>
  30fc1b:	49 89 c6             	mov    %rax,%r14
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  30fc1e:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  30fc25:	04 
  30fc26:	0f 84 a7 02 00 00    	je     30fed3 <<rexx_exec::Interp>::exec_parse+0xab3>
<rexx_exec::Interp>::read_parse_var:
  30fc2c:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:689
  30fc31:	4c 89 fe             	mov    %r15,%rsi
  30fc34:	4c 89 f2             	mov    %r14,%rdx
  30fc37:	e8 f4 fb e9 ff       	call   1af830 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  30fc3c:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  30fc41:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  30fc46:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
  30fc4b:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  30fc50:	48 8b 6c 24 50       	mov    0x50(%rsp),%rbp
  30fc55:	48 85 ed             	test   %rbp,%rbp
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  30fc58:	0f 89 cc 2e 00 00    	jns    312b2a <<rexx_exec::Interp>::exec_parse+0x370a>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  30fc5e:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  30fc60:	48 89 df             	mov    %rbx,%rdi
  30fc63:	48 89 ee             	mov    %rbp,%rsi
  30fc66:	e8 39 1e e0 ff       	call   111aa4 <alloc::raw_vec::handle_error>
  30fc6b:	e9 18 36 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::Interp>::read_parse_var:
  30fc70:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::Interp>::read:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:355
  30fc75:	4c 89 ff             	mov    %r15,%rdi
  30fc78:	48 8b b4 24 f0 00 00 	mov    0xf0(%rsp),%rsi
  30fc7f:	00 
  30fc80:	89 da                	mov    %ebx,%edx
  30fc82:	31 c9                	xor    %ecx,%ecx
  30fc84:	e8 77 22 f7 ff       	call   281f00 <<rexx_exec::Interp>::read_at>
  30fc89:	49 89 c6             	mov    %rax,%r14
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:315
  30fc8c:	f6 c2 01             	test   $0x1,%dl
  30fc8f:	0f 85 c3 33 00 00    	jne    313058 <<rexx_exec::Interp>::exec_parse+0x3c38>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  30fc95:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  30fc9c:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:678
  30fc9d:	0f 84 30 02 00 00    	je     30fed3 <<rexx_exec::Interp>::exec_parse+0xab3>
  30fca3:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:679
  30fca8:	4c 89 fe             	mov    %r15,%rsi
  30fcab:	4c 89 f2             	mov    %r14,%rdx
  30fcae:	e8 7d fb e9 ff       	call   1af830 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  30fcb3:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  30fcb8:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  30fcbd:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
  30fcc2:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  30fcc7:	48 8b 6c 24 50       	mov    0x50(%rsp),%rbp
  30fccc:	48 85 ed             	test   %rbp,%rbp
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  30fccf:	0f 89 3d 01 00 00    	jns    30fe12 <<rexx_exec::Interp>::exec_parse+0x9f2>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  30fcd5:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  30fcd7:	48 89 df             	mov    %rbx,%rdi
  30fcda:	48 89 ee             	mov    %rbp,%rsi
  30fcdd:	e8 c2 1d e0 ff       	call   111aa4 <alloc::raw_vec::handle_error>
  30fce2:	e9 a1 35 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::Interp>::read_parse_var:
  30fce7:	48 8b 84 24 f0 00 00 	mov    0xf0(%rsp),%rax
  30fcee:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:695
  30fcef:	48 8b 40 20          	mov    0x20(%rax),%rax
<core::option::Option<&rexx_exec::plan::Plan> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  30fcf3:	48 85 c0             	test   %rax,%rax
  30fcf6:	74 26                	je     30fd1e <<rexx_exec::Interp>::exec_parse+0x8fe>
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::plan::CompoundName>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  30fcf8:	48 39 98 b0 00 00 00 	cmp    %rbx,0xb0(%rax)
  30fcff:	76 1d                	jbe    30fd1e <<rexx_exec::Interp>::exec_parse+0x8fe>
<rexx_exec::plan::Plan>::compound:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:424
  30fd01:	48 8b 80 a8 00 00 00 	mov    0xa8(%rax),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::plan::CompoundName>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  30fd08:	48 8d 0c 5b          	lea    (%rbx,%rbx,2),%rcx
  30fd0c:	48 c1 e1 04          	shl    $0x4,%rcx
<core::option::Option<rexx_exec::plan::CompoundName>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  30fd10:	48 8b 14 08          	mov    (%rax,%rcx,1),%rdx
  30fd14:	48 83 fa 02          	cmp    $0x2,%rdx
<rexx_exec::plan::Plan>::compound:
  30fd18:	0f 85 0a 2d 00 00    	jne    312a28 <<rexx_exec::Interp>::exec_parse+0x3608>
<rexx_exec::Code>::stem:
  30fd1e:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1288
  30fd23:	4c 89 ee             	mov    %r13,%rsi
  30fd26:	4c 89 e2             	mov    %r12,%rdx
  30fd29:	e8 c2 63 15 00       	call   4660f0 <rexx_parse::ast::compound_parts>
  30fd2e:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  30fd33:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  30fd38:	4c 8b 74 24 48       	mov    0x48(%rsp),%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  30fd3d:	48 83 7c 24 50 00    	cmpq   $0x0,0x50(%rsp)
  30fd43:	74 0b                	je     30fd50 <<rexx_exec::Interp>::exec_parse+0x930>
<rexx_exec::Code>::stem:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1288
  30fd45:	48 8b 7c 24 58       	mov    0x58(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  30fd4a:	ff 15 b8 7e 1c 00    	call   *0x1c7eb8(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  30fd50:	4c 89 74 24 20       	mov    %r14,0x20(%rsp)
  30fd55:	48 c7 84 24 18 01 00 	movq   $0x0,0x118(%rsp)
  30fd5c:	00 00 00 00 00 
  30fd61:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
core::mem::replace::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  30fd66:	f3 41 0f 6f 86 b8 0b 	movdqu 0xbb8(%r14),%xmm0
  30fd6d:	00 00 
  30fd6f:	66 0f 7f 84 24 70 01 	movdqa %xmm0,0x170(%rsp)
  30fd76:	00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  30fd78:	49 c7 86 b8 0b 00 00 	movq   $0x0,0xbb8(%r14)
  30fd7f:	00 00 00 00 
  30fd83:	49 c7 86 c0 0b 00 00 	movq   $0x1,0xbc0(%r14)
  30fd8a:	01 00 00 00 
  30fd8e:	49 c7 86 c8 0b 00 00 	movq   $0x0,0xbc8(%r14)
  30fd95:	00 00 00 00 
<alloc::vec::Vec<u8>>::clear:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3082
  30fd99:	48 c7 84 24 80 01 00 	movq   $0x0,0x180(%rsp)
  30fda0:	00 00 00 00 00 
  30fda5:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  30fdaa:	4c 8d 84 24 70 01 00 	lea    0x170(%rsp),%r8
  30fdb1:	00 
<rexx_exec::Interp>::tail_key_into:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:77
  30fdb2:	4c 89 f6             	mov    %r14,%rsi
  30fdb5:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  30fdbc:	00 
  30fdbd:	89 d9                	mov    %ebx,%ecx
  30fdbf:	e8 9c 10 fb ff       	call   2c0e60 <<rexx_exec::Interp>::append_tail_key>
<rexx_exec::Interp>::read_parse_var:
  30fdc4:	4d 8d be b8 0b 00 00 	lea    0xbb8(%r14),%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:697
  30fdcb:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
  30fdd0:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30fdd4:	74 6f                	je     30fe45 <<rexx_exec::Interp>::exec_parse+0xa25>
  30fdd6:	4c 8b 64 24 48       	mov    0x48(%rsp),%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  30fddb:	49 83 3f 00          	cmpq   $0x0,(%r15)
  30fddf:	74 12                	je     30fdf3 <<rexx_exec::Interp>::exec_parse+0x9d3>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
  30fde1:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  30fde6:	48 8b b8 c0 0b 00 00 	mov    0xbc0(%rax),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  30fded:	ff 15 15 7e 1c 00    	call   *0x1c7e15(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::give_key_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:64
  30fdf3:	48 8b 84 24 80 01 00 	mov    0x180(%rsp),%rax
  30fdfa:	00 
  30fdfb:	49 89 47 10          	mov    %rax,0x10(%r15)
  30fdff:	66 0f 6f 84 24 70 01 	movdqa 0x170(%rsp),%xmm0
  30fe06:	00 00 
  30fe08:	f3 41 0f 7f 07       	movdqu %xmm0,(%r15)
<rexx_exec::Interp>::read_parse_var:
  30fe0d:	e9 d0 03 00 00       	jmp    3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  30fe12:	74 5f                	je     30fe73 <<rexx_exec::Interp>::exec_parse+0xa53>
  30fe14:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  30fe19:	be 01 00 00 00       	mov    $0x1,%esi
  30fe1e:	48 89 ef             	mov    %rbp,%rdi
  30fe21:	e8 fa 78 e1 ff       	call   127720 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  30fe26:	48 85 c0             	test   %rax,%rax
  30fe29:	0f 84 a8 fe ff ff    	je     30fcd7 <<rexx_exec::Interp>::exec_parse+0x8b7>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  30fe2f:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fe32:	48 89 c7             	mov    %rax,%rdi
  30fe35:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  30fe3a:	48 89 ea             	mov    %rbp,%rdx
  30fe3d:	ff 15 f5 7d 1c 00    	call   *0x1c7df5(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  30fe43:	eb 34                	jmp    30fe79 <<rexx_exec::Interp>::exec_parse+0xa59>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  30fe45:	41 f6 86 36 05 00 00 	testb  $0x4,0x536(%r14)
  30fe4c:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:701
  30fe4d:	0f 84 73 2e 00 00    	je     312cc6 <<rexx_exec::Interp>::exec_parse+0x38a6>
  30fe53:	48 8b 5c 24 20       	mov    0x20(%rsp),%rbx
  30fe58:	48 85 db             	test   %rbx,%rbx
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  30fe5b:	0f 89 7e 2c 00 00    	jns    312adf <<rexx_exec::Interp>::exec_parse+0x36bf>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  30fe61:	31 ed                	xor    %ebp,%ebp
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  30fe63:	48 89 ef             	mov    %rbp,%rdi
  30fe66:	48 89 de             	mov    %rbx,%rsi
  30fe69:	e8 36 1c e0 ff       	call   111aa4 <alloc::raw_vec::handle_error>
  30fe6e:	e9 15 34 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::Interp>::read_parse_var:
  30fe73:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  30fe79:	48 83 7c 24 20 00    	cmpq   $0x0,0x20(%rsp)
  30fe7f:	7e 0b                	jle    30fe8c <<rexx_exec::Interp>::exec_parse+0xa6c>
  30fe81:	48 8b 7c 24 28       	mov    0x28(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  30fe86:	ff 15 7c 7d 1c 00    	call   *0x1c7d7c(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  30fe8c:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30fe91:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  30fe98:	75 26                	jne    30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
  30fe9a:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  30fea1:	74 1d                	je     30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  30fea3:	4c 89 34 24          	mov    %r14,(%rsp)
  30fea7:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  30feae:	00 
  30feaf:	4c 89 ea             	mov    %r13,%rdx
  30feb2:	4c 89 e1             	mov    %r12,%rcx
  30feb5:	4d 89 f8             	mov    %r15,%r8
  30feb8:	49 89 e9             	mov    %rbp,%r9
  30febb:	e8 90 0e f7 ff       	call   280d50 <<rexx_exec::Interp>::trace_variable_line>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  30fec0:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  30fec3:	74 09                	je     30fece <<rexx_exec::Interp>::exec_parse+0xaae>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  30fec5:	4c 89 ff             	mov    %r15,%rdi
  30fec8:	ff 15 3a 7d 1c 00    	call   *0x1c7d3a(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::parse_strings:
  30fece:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  30fed3:	49 8b 9f f0 0a 00 00 	mov    0xaf0(%r15),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30feda:	49 3b 9f e0 0a 00 00 	cmp    0xae0(%r15),%rbx
  30fee1:	0f 84 89 2b 00 00    	je     312a70 <<rexx_exec::Interp>::exec_parse+0x3650>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fee7:	49 8b 87 e8 0a 00 00 	mov    0xae8(%r15),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30feee:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30fef2:	48 ff c3             	inc    %rbx
  30fef5:	49 89 9f f0 0a 00 00 	mov    %rbx,0xaf0(%r15)
  30fefc:	4c 8d 2d 38 6d d9 ff 	lea    -0x2692c8(%rip),%r13        # a6c3b <panic_unwind::imp::CANARY+0x41ed6>
  30ff03:	bb 03 00 00 00       	mov    $0x3,%ebx
<rexx_exec::Interp>::parse_strings:
  30ff08:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30ff0d:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  30ff14:	00 00 
  30ff16:	0f 85 02 01 00 00    	jne    31001e <<rexx_exec::Interp>::exec_parse+0xbfe>
  30ff1c:	41 f6 84 24 2d 05 00 	testb  $0x1,0x52d(%r12)
  30ff23:	00 01 
  30ff25:	0f 84 f3 00 00 00    	je     31001e <<rexx_exec::Interp>::exec_parse+0xbfe>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  30ff2b:	4c 89 f0             	mov    %r14,%rax
  30ff2e:	48 83 e0 03          	and    $0x3,%rax
  30ff32:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  30ff37:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<rexx_exec::Interp>::traced_into_parse_buffer:
  30ff3c:	4c 89 e6             	mov    %r12,%rsi
  30ff3f:	4c 89 f2             	mov    %r14,%rdx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  30ff42:	0f 84 d6 2a 00 00    	je     312a1e <<rexx_exec::Interp>::exec_parse+0x35fe>
<rexx_exec::Interp>::traced_into_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:354
  30ff48:	e8 f3 62 04 00       	call   356240 <<rexx_exec::Interp>::rendered_into_parse_buffer>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30ff4d:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  30ff52:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  30ff57:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
  30ff5c:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  30ff61:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
  30ff65:	0f 84 77 02 00 00    	je     3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
<rexx_exec::Interp>::parse_strings:
  30ff6b:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30ff70:	41 80 bf 35 05 00 00 	cmpb   $0x0,0x535(%r15)
  30ff77:	00 
  30ff78:	75 60                	jne    30ffda <<rexx_exec::Interp>::exec_parse+0xbba>
  30ff7a:	41 f6 87 2d 05 00 00 	testb  $0x1,0x52d(%r15)
  30ff81:	01 
  30ff82:	74 56                	je     30ffda <<rexx_exec::Interp>::exec_parse+0xbba>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30ff84:	49 8d bf d8 10 00 00 	lea    0x10d8(%r15),%rdi
  30ff8b:	49 8b 87 e8 10 00 00 	mov    0x10e8(%r15),%rax
  30ff92:	48 89 84 24 18 01 00 	mov    %rax,0x118(%rsp)
  30ff99:	00 
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:298
  30ff9a:	4c 89 64 24 10       	mov    %r12,0x10(%rsp)
  30ff9f:	48 89 6c 24 08       	mov    %rbp,0x8(%rsp)
  30ffa4:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  30ffab:	48 8d 35 d3 ab d5 ff 	lea    -0x2a542d(%rip),%rsi        # 6ab85 <panic_unwind::imp::CANARY+0x5e20>
  30ffb2:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  30ffb9:	00 
  30ffba:	b9 01 00 00 00       	mov    $0x1,%ecx
  30ffbf:	4d 89 e8             	mov    %r13,%r8
  30ffc2:	49 89 d9             	mov    %rbx,%r9
  30ffc5:	e8 86 03 f7 ff       	call   280350 <rexx_exec::trace::format::push_tagged>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:310
  30ffca:	4c 89 ff             	mov    %r15,%rdi
  30ffcd:	48 8b b4 24 18 01 00 	mov    0x118(%rsp),%rsi
  30ffd4:	00 
  30ffd5:	e8 56 e0 e8 ff       	call   19e030 <<rexx_exec::Interp>::route_trace_line>
<rexx_exec::Interp>::parse_strings:
  30ffda:	4c 8b 7c 24 20       	mov    0x20(%rsp),%r15
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  30ffdf:	49 8d 87 ff ef ff ff 	lea    -0x1001(%r15),%rax
  30ffe6:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  30ffec:	0f 87 db 00 00 00    	ja     3100cd <<rexx_exec::Interp>::exec_parse+0xcad>
  30fff2:	4d 85 ff             	test   %r15,%r15
  30fff5:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  30fffa:	0f 84 53 01 00 00    	je     310153 <<rexx_exec::Interp>::exec_parse+0xd33>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  310000:	48 89 ef             	mov    %rbp,%rdi
  310003:	ff 15 ff 7b 1c 00    	call   *0x1c7bff(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  310009:	48 8d 05 0c 44 d4 ff 	lea    -0x2bbbf4(%rip),%rax        # 5441c <GCC_except_table5013+0x4a54>
  310010:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310015:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  310019:	48 01 c1             	add    %rax,%rcx
  31001c:	ff e1                	jmp    *%rcx
  31001e:	44 89 f0             	mov    %r14d,%eax
  310021:	83 e0 03             	and    $0x3,%eax
  310024:	48 8d 0d f1 43 d4 ff 	lea    -0x2bbc0f(%rip),%rcx        # 5441c <GCC_except_table5013+0x4a54>
  31002b:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  31002f:	48 01 c8             	add    %rcx,%rax
  310032:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:114
  310034:	41 80 bc 24 04 20 00 	cmpb   $0x1,0x2004(%r12)
  31003b:	00 01 
  31003d:	0f 85 4a 01 00 00    	jne    31018d <<rexx_exec::Interp>::exec_parse+0xd6d>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310043:	41 f6 c6 03          	test   $0x3,%r14b
  310047:	0f 85 1b 01 00 00    	jne    310168 <<rexx_exec::Interp>::exec_parse+0xd48>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  31004d:	4c 89 f0             	mov    %r14,%rax
  310050:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  310054:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310056:	49 39 8c 24 08 0a 00 	cmp    %rcx,0xa08(%r12)
  31005d:	00 
  31005e:	0f 86 04 01 00 00    	jbe    310168 <<rexx_exec::Interp>::exec_parse+0xd48>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  310064:	49 8b 84 24 00 0a 00 	mov    0xa00(%r12),%rax
  31006b:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  31006c:	4c 89 f2             	mov    %r14,%rdx
  31006f:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310073:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  310077:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  31007b:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  31007f:	0f 85 e3 00 00 00    	jne    310168 <<rexx_exec::Interp>::exec_parse+0xd48>
  310085:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  310088:	83 38 01             	cmpl   $0x1,(%rax)
  31008b:	0f 85 d7 00 00 00    	jne    310168 <<rexx_exec::Interp>::exec_parse+0xd48>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  310091:	80 78 08 01          	cmpb   $0x1,0x8(%rax)
  310095:	0f 87 cd 00 00 00    	ja     310168 <<rexx_exec::Interp>::exec_parse+0xd48>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  31009b:	49 8b 9c 24 f0 0a 00 	mov    0xaf0(%r12),%rbx
  3100a2:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3100a3:	49 3b 9c 24 e0 0a 00 	cmp    0xae0(%r12),%rbx
  3100aa:	00 
  3100ab:	0f 84 ce 2a 00 00    	je     312b7f <<rexx_exec::Interp>::exec_parse+0x375f>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3100b1:	49 8b 84 24 e8 0a 00 	mov    0xae8(%r12),%rax
  3100b8:	00 
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3100b9:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3100bd:	48 ff c3             	inc    %rbx
  3100c0:	49 89 9c 24 f0 0a 00 	mov    %rbx,0xaf0(%r12)
  3100c7:	00 
  3100c8:	e9 c0 00 00 00       	jmp    31018d <<rexx_exec::Interp>::exec_parse+0xd6d>
<rexx_exec::Interp>::parse_strings:
  3100cd:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3100d2:	49 8b 9c 24 e0 0b 00 	mov    0xbe0(%r12),%rbx
  3100d9:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3100da:	48 83 fb 08          	cmp    $0x8,%rbx
  3100de:	72 1e                	jb     3100fe <<rexx_exec::Interp>::exec_parse+0xcde>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3100e0:	48 89 ef             	mov    %rbp,%rdi
  3100e3:	ff 15 1f 7b 1c 00    	call   *0x1c7b1f(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  3100e9:	48 8d 05 2c 43 d4 ff 	lea    -0x2bbcd4(%rip),%rax        # 5441c <GCC_except_table5013+0x4a54>
  3100f0:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3100f5:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  3100f9:	48 01 c1             	add    %rax,%rcx
  3100fc:	ff e1                	jmp    *%rcx
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3100fe:	49 3b 9c 24 d0 0b 00 	cmp    0xbd0(%r12),%rbx
  310105:	00 
  310106:	75 0d                	jne    310115 <<rexx_exec::Interp>::exec_parse+0xcf5>
<rexx_exec::Interp>::give_parse_buffer:
  310108:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  31010f:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310110:	e8 8b 0a 18 00       	call   490ba0 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310115:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  31011c:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  31011d:	48 8d 0c 5b          	lea    (%rbx,%rbx,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310121:	4c 89 3c c8          	mov    %r15,(%rax,%rcx,8)
  310125:	48 89 6c c8 08       	mov    %rbp,0x8(%rax,%rcx,8)
  31012a:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  310131:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310133:	48 ff c3             	inc    %rbx
  310136:	49 89 9c 24 e0 0b 00 	mov    %rbx,0xbe0(%r12)
  31013d:	00 
  31013e:	48 8d 05 d7 42 d4 ff 	lea    -0x2bbd29(%rip),%rax        # 5441c <GCC_except_table5013+0x4a54>
  310145:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31014a:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  31014e:	48 01 c1             	add    %rax,%rcx
  310151:	ff e1                	jmp    *%rcx
  310153:	48 8d 05 c2 42 d4 ff 	lea    -0x2bbd3e(%rip),%rax        # 5441c <GCC_except_table5013+0x4a54>
  31015a:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
  31015f:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  310163:	48 01 c1             	add    %rax,%rcx
  310166:	ff e1                	jmp    *%rcx
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
  310168:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  31016d:	4c 89 e6             	mov    %r12,%rsi
  310170:	4c 89 f2             	mov    %r14,%rdx
  310173:	31 c9                	xor    %ecx,%ecx
  310175:	e8 76 6a ec ff       	call   1d6bf0 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31017a:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
  31017f:	4c 8b 74 24 48       	mov    0x48(%rsp),%r14
  310184:	4d 89 f4             	mov    %r14,%r12
  310187:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  31018b:	75 55                	jne    3101e2 <<rexx_exec::Interp>::exec_parse+0xdc2>
<rexx_exec::Interp>::parse_strings:
  31018d:	48 8b 84 24 b0 00 00 	mov    0xb0(%rsp),%rax
  310194:	00 
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:404
  310195:	80 78 68 00          	cmpb   $0x0,0x68(%rax)
  310199:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  31019e:	75 14                	jne    3101b4 <<rexx_exec::Interp>::exec_parse+0xd94>
  3101a0:	f6 40 69 01          	testb  $0x1,0x69(%rax)
  3101a4:	75 0e                	jne    3101b4 <<rexx_exec::Interp>::exec_parse+0xd94>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3101a6:	44 89 f0             	mov    %r14d,%eax
  3101a9:	83 e0 03             	and    $0x3,%eax
  3101ac:	8d 48 ff             	lea    -0x1(%rax),%ecx
  3101af:	83 f9 02             	cmp    $0x2,%ecx
  3101b2:	73 42                	jae    3101f6 <<rexx_exec::Interp>::exec_parse+0xdd6>
<rexx_exec::Interp>::source_text:
  3101b4:	48 8d bc 24 20 01 00 	lea    0x120(%rsp),%rdi
  3101bb:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:420
  3101bc:	4c 89 f2             	mov    %r14,%rdx
  3101bf:	e8 7c 60 04 00       	call   356240 <<rexx_exec::Interp>::rendered_into_parse_buffer>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:656
  3101c4:	4c 8b bc 24 20 01 00 	mov    0x120(%rsp),%r15
  3101cb:	00 
  3101cc:	48 8b ac 24 28 01 00 	mov    0x128(%rsp),%rbp
  3101d3:	00 
  3101d4:	4c 8b a4 24 30 01 00 	mov    0x130(%rsp),%r12
  3101db:	00 
<core::result::Result<rexx_exec::parse_template::ParseStrings, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3101dc:	49 83 ff fd          	cmp    $0xfffffffffffffffd,%r15
  3101e0:	75 5b                	jne    31023d <<rexx_exec::Interp>::exec_parse+0xe1d>
<rexx_exec::Interp>::exec_parse:
  3101e2:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  3101e9:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  3101ea:	48 89 28             	mov    %rbp,(%rax)
  3101ed:	4c 89 60 08          	mov    %r12,0x8(%rax)
<rexx_exec::Interp>::exec_parse:
  3101f1:	e9 79 27 00 00       	jmp    31296f <<rexx_exec::Interp>::exec_parse+0x354f>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3101f6:	83 f8 03             	cmp    $0x3,%eax
  3101f9:	0f 85 cd 25 00 00    	jne    3127cc <<rexx_exec::Interp>::exec_parse+0x33ac>
  3101ff:	48 8d 9c 24 28 01 00 	lea    0x128(%rsp),%rbx
  310206:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  310207:	44 89 f0             	mov    %r14d,%eax
  31020a:	c1 e8 02             	shr    $0x2,%eax
  31020d:	83 e0 07             	and    $0x7,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  310210:	49 81 e6 00 ff ff ff 	and    $0xffffffffffffff00,%r14
  310217:	49 09 c6             	or     %rax,%r14
<rexx_exec::Interp>::source_text:
  31021a:	49 bf 01 00 00 00 00 	movabs $0x8000000000000001,%r15
  310221:	00 00 80 
  310224:	49 ff cf             	dec    %r15
  310227:	4d 89 f4             	mov    %r14,%r12
  31022a:	4c 89 23             	mov    %r12,(%rbx)
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:656
  31022d:	48 8b ac 24 28 01 00 	mov    0x128(%rsp),%rbp
  310234:	00 
  310235:	4c 8b a4 24 30 01 00 	mov    0x130(%rsp),%r12
  31023c:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:481
  31023d:	4c 89 bc 24 c0 01 00 	mov    %r15,0x1c0(%rsp)
  310244:	00 
  310245:	48 89 ac 24 c8 01 00 	mov    %rbp,0x1c8(%rsp)
  31024c:	00 
  31024d:	4c 89 a4 24 d0 01 00 	mov    %r12,0x1d0(%rsp)
  310254:	00 
  310255:	48 8b 84 24 b0 00 00 	mov    0xb0(%rsp),%rax
  31025c:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  31025d:	0f b6 68 68          	movzbl 0x68(%rax),%ebp
  310261:	44 0f b6 68 69       	movzbl 0x69(%rax),%r13d
  310266:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  31026b:	48 8d 94 24 c0 01 00 	lea    0x1c0(%rsp),%rdx
  310272:	00 
  310273:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  310278:	4c 89 e6             	mov    %r12,%rsi
  31027b:	89 e9                	mov    %ebp,%ecx
  31027d:	45 89 e8             	mov    %r13d,%r8d
  310280:	4c 8b 8c 24 c0 00 00 	mov    0xc0(%rsp),%r9
  310287:	00 
  310288:	e8 e3 60 04 00       	call   356370 <<rexx_exec::Interp>::next_template>
<core::result::Result<(rexx_exec::parse_template::SourceText, rexx_exec::parse_template::Cursor), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31028d:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
  310292:	0f 10 44 24 48       	movups 0x48(%rsp),%xmm0
  310297:	0f 29 84 24 20 01 00 	movaps %xmm0,0x120(%rsp)
  31029e:	00 
  31029f:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  3102a3:	0f 84 a6 24 00 00    	je     31274f <<rexx_exec::Interp>::exec_parse+0x332f>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  3102a9:	48 8b 44 24 58       	mov    0x58(%rsp),%rax
  3102ae:	48 89 44 24 38       	mov    %rax,0x38(%rsp)
  3102b3:	48 8b 44 24 68       	mov    0x68(%rsp),%rax
  3102b8:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  3102bd:	48 8b 44 24 70       	mov    0x70(%rsp),%rax
  3102c2:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  3102c9:	00 
  3102ca:	48 8b 44 24 78       	mov    0x78(%rsp),%rax
  3102cf:	48 89 84 24 98 01 00 	mov    %rax,0x198(%rsp)
  3102d6:	00 
  3102d7:	48 8b 9c 24 80 00 00 	mov    0x80(%rsp),%rbx
  3102de:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  3102df:	66 0f 6f 84 24 20 01 	movdqa 0x120(%rsp),%xmm0
  3102e6:	00 00 
  3102e8:	f3 0f 7f 84 24 e0 00 	movdqu %xmm0,0xe0(%rsp)
  3102ef:	00 00 
  3102f1:	4c 89 bc 24 d8 00 00 	mov    %r15,0xd8(%rsp)
  3102f8:	00 
  3102f9:	48 8b 8c 24 b0 00 00 	mov    0xb0(%rsp),%rcx
  310300:	00 
<alloc::vec::Vec<core::option::Option<rexx_parse::ast::ParseTrigger>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310301:	48 8b 41 10          	mov    0x10(%rcx),%rax
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  310305:	48 85 c0             	test   %rax,%rax
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  310308:	0f 84 37 25 00 00    	je     312845 <<rexx_exec::Interp>::exec_parse+0x3425>
  31030e:	48 8b 49 08          	mov    0x8(%rcx),%rcx
  310312:	48 6b c0 70          	imul   $0x70,%rax,%rax
  310316:	48 89 8c 24 68 01 00 	mov    %rcx,0x168(%rsp)
  31031d:	00 
  31031e:	48 01 c8             	add    %rcx,%rax
  310321:	48 89 84 24 d8 01 00 	mov    %rax,0x1d8(%rsp)
  310328:	00 
  310329:	48 b9 01 00 00 00 00 	movabs $0x8000000000000001,%rcx
  310330:	00 00 80 
  310333:	49 8d 84 24 e0 0a 00 	lea    0xae0(%r12),%rax
  31033a:	00 
  31033b:	48 89 84 24 60 01 00 	mov    %rax,0x160(%rsp)
  310342:	00 
  310343:	48 8b 84 24 f0 00 00 	mov    0xf0(%rsp),%rax
  31034a:	00 
  31034b:	48 8b 50 18          	mov    0x18(%rax),%rdx
  31034f:	48 89 94 24 08 02 00 	mov    %rdx,0x208(%rsp)
  310356:	00 
  310357:	48 8b 50 08          	mov    0x8(%rax),%rdx
  31035b:	48 89 94 24 00 02 00 	mov    %rdx,0x200(%rsp)
  310362:	00 
  310363:	48 8b 40 10          	mov    0x10(%rax),%rax
  310367:	48 89 84 24 f8 01 00 	mov    %rax,0x1f8(%rsp)
  31036e:	00 
  31036f:	49 8d 84 24 b0 0a 00 	lea    0xab0(%r12),%rax
  310376:	00 
  310377:	48 89 84 24 f0 01 00 	mov    %rax,0x1f0(%rsp)
  31037e:	00 
  31037f:	49 8d 84 24 d0 0b 00 	lea    0xbd0(%r12),%rax
  310386:	00 
  310387:	48 89 84 24 58 01 00 	mov    %rax,0x158(%rsp)
  31038e:	00 
  31038f:	49 8d 84 24 d8 10 00 	lea    0x10d8(%r12),%rax
  310396:	00 
  310397:	48 89 84 24 e0 01 00 	mov    %rax,0x1e0(%rsp)
  31039e:	00 
  31039f:	48 8d 05 e2 ff 1a 00 	lea    0x1affe2(%rip),%rax        # 4c0388 <rexx_api::ffi::CALL_CONTEXT+0x53a0>
  3103a6:	48 89 84 24 90 01 00 	mov    %rax,0x190(%rsp)
  3103ad:	00 
  3103ae:	89 ac 24 b8 01 00 00 	mov    %ebp,0x1b8(%rsp)
  3103b5:	44 89 ac 24 b4 01 00 	mov    %r13d,0x1b4(%rsp)
  3103bc:	00 
  3103bd:	49 8d 04 0f          	lea    (%r15,%rcx,1),%rax
  3103c1:	4c 89 bc 24 48 01 00 	mov    %r15,0x148(%rsp)
  3103c8:	00 
  3103c9:	49 c1 ff 3f          	sar    $0x3f,%r15
  3103cd:	49 21 c7             	and    %rax,%r15
  3103d0:	4c 89 bc 24 d0 00 00 	mov    %r15,0xd0(%rsp)
  3103d7:	00 
  3103d8:	48 8b 84 24 68 01 00 	mov    0x168(%rsp),%rax
  3103df:	00 
  3103e0:	eb 2c                	jmp    31040e <<rexx_exec::Interp>::exec_parse+0xfee>
  3103e2:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  3103e9:	1f 84 00 00 00 00 00 
  3103f0:	4c 89 fb             	mov    %r15,%rbx
  3103f3:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  3103f8:	48 8b 84 24 68 01 00 	mov    0x168(%rsp),%rax
  3103ff:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  310400:	48 3b 84 24 d8 01 00 	cmp    0x1d8(%rsp),%rax
  310407:	00 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  310408:	0f 84 2f 24 00 00    	je     31283d <<rexx_exec::Interp>::exec_parse+0x341d>
  31040e:	4c 8b bc 24 98 01 00 	mov    0x198(%rsp),%r15
  310415:	00 
  310416:	48 8b 8c 24 c8 00 00 	mov    0xc8(%rsp),%rcx
  31041d:	00 
  31041e:	48 89 c6             	mov    %rax,%rsi
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:627
  310421:	48 83 c0 70          	add    $0x70,%rax
  310425:	48 89 84 24 68 01 00 	mov    %rax,0x168(%rsp)
  31042c:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:485
  31042d:	48 83 3e ff          	cmpq   $0xffffffffffffffff,(%rsi)
  310431:	0f 84 a4 21 00 00    	je     3125db <<rexx_exec::Interp>::exec_parse+0x31bb>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:31
  310437:	80 7e 68 00          	cmpb   $0x0,0x68(%rsi)
  31043b:	0f 84 df 01 00 00    	je     310620 <<rexx_exec::Interp>::exec_parse+0x1200>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:36
  310441:	48 83 7e 18 ff       	cmpq   $0xffffffffffffffff,0x18(%rsi)
  310446:	0f 84 35 25 00 00    	je     312981 <<rexx_exec::Interp>::exec_parse+0x3561>
  31044c:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
  310451:	48 89 b4 24 b0 00 00 	mov    %rsi,0xb0(%rsp)
  310458:	00 
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:127
  310459:	c6 84 24 00 01 00 00 	movb   $0x0,0x100(%rsp)
  310460:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:128
  310461:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  310466:	4c 89 e6             	mov    %r12,%rsi
  310469:	48 8d 94 24 00 01 00 	lea    0x100(%rsp),%rdx
  310470:	00 
  310471:	e8 7a bf f6 ff       	call   27c3f0 <<rexx_exec::Interp>::enter_eval_node::<u8>>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310476:	4c 8b 64 24 40       	mov    0x40(%rsp),%r12
  31047b:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  31047f:	0f 85 ae 23 00 00    	jne    312833 <<rexx_exec::Interp>::exec_parse+0x3413>
<rexx_exec::Interp>::apply_trigger:
  310485:	48 8b 84 24 b0 00 00 	mov    0xb0(%rsp),%rax
  31048c:	00 
  31048d:	4c 8d 70 18          	lea    0x18(%rax),%r14
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:129
  310491:	48 8d bc 24 20 01 00 	lea    0x120(%rsp),%rdi
  310498:	00 
  310499:	48 8b 6c 24 18       	mov    0x18(%rsp),%rbp
  31049e:	48 89 ee             	mov    %rbp,%rsi
  3104a1:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  3104a8:	00 
  3104a9:	4c 89 f1             	mov    %r14,%rcx
  3104ac:	e8 9f c1 f6 ff       	call   27c650 <<rexx_exec::Interp>::eval_node>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:130
  3104b1:	48 ff 8d c8 04 00 00 	decq   0x4c8(%rbp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:131
  3104b8:	4c 8b a4 24 20 01 00 	mov    0x120(%rsp),%r12
  3104bf:	00 
  3104c0:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  3104c4:	0f 85 29 25 00 00    	jne    3129f3 <<rexx_exec::Interp>::exec_parse+0x35d3>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  3104ca:	f6 85 36 05 00 00 04 	testb  $0x4,0x536(%rbp)
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  3104d1:	4c 8b ac 24 28 01 00 	mov    0x128(%rsp),%r13
  3104d8:	00 
<rexx_exec::Interp>::trace_intermediate:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:161
  3104d9:	74 2a                	je     310505 <<rexx_exec::Interp>::exec_parse+0x10e5>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:164
  3104db:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  3104e0:	48 89 ee             	mov    %rbp,%rsi
  3104e3:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  3104ea:	00 
  3104eb:	4c 89 f1             	mov    %r14,%rcx
  3104ee:	4d 89 e8             	mov    %r13,%r8
  3104f1:	e8 7a ec f6 ff       	call   27f170 <<rexx_exec::Interp>::trace_intermediate_event>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3104f6:	4c 8b 64 24 40       	mov    0x40(%rsp),%r12
  3104fb:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  3104ff:	0f 85 2e 23 00 00    	jne    312833 <<rexx_exec::Interp>::exec_parse+0x3413>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310505:	4c 8b b5 f0 0a 00 00 	mov    0xaf0(%rbp),%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  31050c:	4c 3b b5 e0 0a 00 00 	cmp    0xae0(%rbp),%r14
  310513:	4d 89 ec             	mov    %r13,%r12
  310516:	75 0d                	jne    310525 <<rexx_exec::Interp>::exec_parse+0x1105>
  310518:	48 8b bc 24 60 01 00 	mov    0x160(%rsp),%rdi
  31051f:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310520:	e8 8b b9 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310525:	48 8b 85 e8 0a 00 00 	mov    0xae8(%rbp),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  31052c:	4e 89 24 f0          	mov    %r12,(%rax,%r14,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310530:	49 ff c6             	inc    %r14
  310533:	4c 89 b5 f0 0a 00 00 	mov    %r14,0xaf0(%rbp)
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:55
  31053a:	80 bd 2d 05 00 00 01 	cmpb   $0x1,0x52d(%rbp)
  310541:	0f 84 05 20 00 00    	je     31254c <<rexx_exec::Interp>::exec_parse+0x312c>
  310547:	48 8b 84 24 b0 00 00 	mov    0xb0(%rsp),%rax
  31054e:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:59
  31054f:	0f b6 40 68          	movzbl 0x68(%rax),%eax
  310553:	83 f8 06             	cmp    $0x6,%eax
  310556:	0f 84 db 00 00 00    	je     310637 <<rexx_exec::Interp>::exec_parse+0x1217>
  31055c:	83 f8 07             	cmp    $0x7,%eax
  31055f:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310564:	0f 85 82 01 00 00    	jne    3106ec <<rexx_exec::Interp>::exec_parse+0x12cc>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31056a:	44 89 e8             	mov    %r13d,%eax
  31056d:	83 e0 03             	and    $0x3,%eax
  310570:	48 8d 15 b5 3e d4 ff 	lea    -0x2bc14b(%rip),%rdx        # 5442c <GCC_except_table5013+0x4a64>
  310577:	48 63 0c 82          	movslq (%rdx,%rax,4),%rcx
  31057b:	48 01 d1             	add    %rdx,%rcx
  31057e:	ff e1                	jmp    *%rcx
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:114
  310580:	41 80 be 04 20 00 00 	cmpb   $0x1,0x2004(%r14)
  310587:	01 
  310588:	0f 85 d0 01 00 00    	jne    31075e <<rexx_exec::Interp>::exec_parse+0x133e>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31058e:	48 85 c0             	test   %rax,%rax
  310591:	0f 85 9c 01 00 00    	jne    310733 <<rexx_exec::Interp>::exec_parse+0x1313>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310597:	4c 89 e8             	mov    %r13,%rax
  31059a:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  31059e:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  3105a0:	49 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%r14)
  3105a7:	0f 86 86 01 00 00    	jbe    310733 <<rexx_exec::Interp>::exec_parse+0x1313>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  3105ad:	49 8b 86 00 0a 00 00 	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  3105b4:	4c 89 ea             	mov    %r13,%rdx
  3105b7:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  3105bb:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  3105bf:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  3105c3:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  3105c7:	0f 85 66 01 00 00    	jne    310733 <<rexx_exec::Interp>::exec_parse+0x1313>
  3105cd:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  3105d0:	83 38 01             	cmpl   $0x1,(%rax)
  3105d3:	0f 85 5a 01 00 00    	jne    310733 <<rexx_exec::Interp>::exec_parse+0x1313>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  3105d9:	80 78 08 02          	cmpb   $0x2,0x8(%rax)
  3105dd:	0f 83 50 01 00 00    	jae    310733 <<rexx_exec::Interp>::exec_parse+0x1313>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  3105e3:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3105ea:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  3105f1:	75 0d                	jne    310600 <<rexx_exec::Interp>::exec_parse+0x11e0>
  3105f3:	48 8b bc 24 60 01 00 	mov    0x160(%rsp),%rdi
  3105fa:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3105fb:	e8 b0 b8 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310600:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310607:	4c 89 2c d8          	mov    %r13,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  31060b:	48 ff c3             	inc    %rbx
  31060e:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
  310615:	e9 44 01 00 00       	jmp    31075e <<rexx_exec::Interp>::exec_parse+0x133e>
<rexx_exec::Interp>::apply_trigger:
  31061a:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
  310620:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
  310625:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  31062a:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  310631:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:31
  310632:	e9 dd 0a 00 00       	jmp    311114 <<rexx_exec::Interp>::exec_parse+0x1cf4>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310637:	44 89 e8             	mov    %r13d,%eax
  31063a:	83 e0 03             	and    $0x3,%eax
  31063d:	48 8d 15 18 3e d4 ff 	lea    -0x2bc1e8(%rip),%rdx        # 5445c <GCC_except_table5013+0x4a94>
  310644:	48 63 0c 82          	movslq (%rdx,%rax,4),%rcx
  310648:	48 01 d1             	add    %rdx,%rcx
  31064b:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310650:	ff e1                	jmp    *%rcx
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:114
  310652:	41 80 be 04 20 00 00 	cmpb   $0x1,0x2004(%r14)
  310659:	01 
  31065a:	0f 85 1b 03 00 00    	jne    31097b <<rexx_exec::Interp>::exec_parse+0x155b>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310660:	48 85 c0             	test   %rax,%rax
  310663:	0f 85 e7 02 00 00    	jne    310950 <<rexx_exec::Interp>::exec_parse+0x1530>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310669:	4c 89 e8             	mov    %r13,%rax
  31066c:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  310670:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310672:	49 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%r14)
  310679:	0f 86 d1 02 00 00    	jbe    310950 <<rexx_exec::Interp>::exec_parse+0x1530>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  31067f:	49 8b 86 00 0a 00 00 	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  310686:	4c 89 ea             	mov    %r13,%rdx
  310689:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  31068d:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  310691:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  310695:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  310699:	0f 85 b1 02 00 00    	jne    310950 <<rexx_exec::Interp>::exec_parse+0x1530>
  31069f:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  3106a2:	83 38 01             	cmpl   $0x1,(%rax)
  3106a5:	0f 85 a5 02 00 00    	jne    310950 <<rexx_exec::Interp>::exec_parse+0x1530>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  3106ab:	80 78 08 02          	cmpb   $0x2,0x8(%rax)
  3106af:	0f 83 9b 02 00 00    	jae    310950 <<rexx_exec::Interp>::exec_parse+0x1530>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  3106b5:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3106bc:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  3106c3:	75 0d                	jne    3106d2 <<rexx_exec::Interp>::exec_parse+0x12b2>
  3106c5:	48 8b bc 24 60 01 00 	mov    0x160(%rsp),%rdi
  3106cc:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3106cd:	e8 de b7 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3106d2:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3106d9:	4c 89 2c d8          	mov    %r13,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3106dd:	48 ff c3             	inc    %rbx
  3106e0:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
  3106e7:	e9 8f 02 00 00       	jmp    31097b <<rexx_exec::Interp>::exec_parse+0x155b>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:86
  3106ec:	4c 89 f7             	mov    %r14,%rdi
  3106ef:	4c 89 ee             	mov    %r13,%rsi
  3106f2:	e8 19 64 04 00       	call   356b10 <<rexx_exec::Interp>::whole_nonneg>
  3106f7:	a8 01                	test   $0x1,%al
  3106f9:	0f 84 82 23 00 00    	je     312a81 <<rexx_exec::Interp>::exec_parse+0x3661>
  3106ff:	48 8b b4 24 b0 00 00 	mov    0xb0(%rsp),%rsi
  310706:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:95
  310707:	0f b6 46 68          	movzbl 0x68(%rsi),%eax
  31070b:	48 8d 0d 7a 3d d4 ff 	lea    -0x2bc286(%rip),%rcx        # 5448c <GCC_except_table5013+0x4ac4>
  310712:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310716:	48 01 c8             	add    %rcx,%rax
  310719:	ff e0                	jmp    *%rax
  31071b:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
  310720:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  310727:	00 
  310728:	4c 89 f8             	mov    %r15,%rax
  31072b:	49 89 df             	mov    %rbx,%r15
  31072e:	e9 e1 09 00 00       	jmp    311114 <<rexx_exec::Interp>::exec_parse+0x1cf4>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  310733:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  310738:	4c 89 f6             	mov    %r14,%rsi
  31073b:	4c 89 ea             	mov    %r13,%rdx
  31073e:	31 c9                	xor    %ecx,%ecx
  310740:	e8 ab 64 ec ff       	call   1d6bf0 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310745:	4c 8b 64 24 40       	mov    0x40(%rsp),%r12
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  31074a:	4c 8b 6c 24 48       	mov    0x48(%rsp),%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31074f:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  310753:	0f 85 71 24 00 00    	jne    312bca <<rexx_exec::Interp>::exec_parse+0x37aa>
<rexx_exec::Interp>::apply_trigger:
  310759:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  31075e:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310765:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  31076c:	75 0d                	jne    31077b <<rexx_exec::Interp>::exec_parse+0x135b>
  31076e:	48 8b bc 24 60 01 00 	mov    0x160(%rsp),%rdi
  310775:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310776:	e8 35 b7 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  31077b:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310782:	4c 89 2c d8          	mov    %r13,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310786:	48 ff c3             	inc    %rbx
  310789:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310790:	44 89 e8             	mov    %r13d,%eax
  310793:	83 e0 03             	and    $0x3,%eax
  310796:	48 8d 0d 9f 3c d4 ff 	lea    -0x2bc361(%rip),%rcx        # 5443c <GCC_except_table5013+0x4a74>
  31079d:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  3107a1:	48 01 c8             	add    %rcx,%rax
  3107a4:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  3107a6:	4c 89 f7             	mov    %r14,%rdi
  3107a9:	4c 89 ee             	mov    %r13,%rsi
  3107ac:	e8 ff e0 ec ff       	call   1de8b0 <<rexx_exec::Interp>::try_text>
<core::option::Option<&[u8]>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:634
  3107b1:	48 85 c0             	test   %rax,%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  3107b4:	0f 84 b3 03 00 00    	je     310b6d <<rexx_exec::Interp>::exec_parse+0x174d>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:955
  3107ba:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  3107bf:	c6 44 24 40 00       	movb   $0x0,0x40(%rsp)
  3107c4:	31 c0                	xor    %eax,%eax
  3107c6:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  3107cd:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  3107ce:	48 85 d2             	test   %rdx,%rdx
  3107d1:	0f 84 1c 01 00 00    	je     3108f3 <<rexx_exec::Interp>::exec_parse+0x14d3>
  3107d7:	48 83 fa 01          	cmp    $0x1,%rdx
  3107db:	75 2e                	jne    31080b <<rexx_exec::Interp>::exec_parse+0x13eb>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  3107dd:	44 0f b6 a4 24 e0 00 	movzbl 0xe0(%rsp),%r12d
  3107e4:	00 00 
  3107e6:	49 83 fc 08          	cmp    $0x8,%r12
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  3107ea:	0f 83 ad 28 00 00    	jae    31309d <<rexx_exec::Interp>::exec_parse+0x3c7d>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  3107f0:	0f b6 c0             	movzbl %al,%eax
  3107f3:	48 8d 0d 52 3c d4 ff 	lea    -0x2bc3ae(%rip),%rcx        # 5444c <GCC_except_table5013+0x4a84>
  3107fa:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  3107fe:	48 01 c8             	add    %rcx,%rax
  310801:	48 8d ac 24 e1 00 00 	lea    0xe1(%rsp),%rbp
  310808:	00 
  310809:	ff e0                	jmp    *%rax
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  31080b:	48 8b 94 24 e0 00 00 	mov    0xe0(%rsp),%rdx
  310812:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310813:	f6 c2 03             	test   $0x3,%dl
  310816:	0f 85 18 2a 00 00    	jne    313234 <<rexx_exec::Interp>::exec_parse+0x3e14>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  31081c:	48 89 d1             	mov    %rdx,%rcx
  31081f:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310823:	89 ce                	mov    %ecx,%esi
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310825:	49 39 b6 08 0a 00 00 	cmp    %rsi,0xa08(%r14)
  31082c:	0f 86 02 2a 00 00    	jbe    313234 <<rexx_exec::Interp>::exec_parse+0x3e14>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310832:	49 8b 8e 00 0a 00 00 	mov    0xa00(%r14),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310839:	48 8d 34 76          	lea    (%rsi,%rsi,2),%rsi
  31083d:	48 c1 e6 05          	shl    $0x5,%rsi
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310841:	83 3c 31 01          	cmpl   $0x1,(%rcx,%rsi,1)
  310845:	0f 85 e9 29 00 00    	jne    313234 <<rexx_exec::Interp>::exec_parse+0x3e14>
  31084b:	48 01 f1             	add    %rsi,%rcx
  31084e:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  310852:	0f 85 dc 29 00 00    	jne    313234 <<rexx_exec::Interp>::exec_parse+0x3e14>
  310858:	48 c1 ea 22          	shr    $0x22,%rdx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  31085c:	39 51 04             	cmp    %edx,0x4(%rcx)
  31085f:	0f 85 cf 29 00 00    	jne    313234 <<rexx_exec::Interp>::exec_parse+0x3e14>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  310865:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  310869:	0f b6 c0             	movzbl %al,%eax
  31086c:	0f 85 8f 03 00 00    	jne    310c01 <<rexx_exec::Interp>::exec_parse+0x17e1>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310872:	48 8b 69 20          	mov    0x20(%rcx),%rbp
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310876:	4c 8b 61 28          	mov    0x28(%rcx),%r12
  31087a:	48 8d 0d cb 3b d4 ff 	lea    -0x2bc435(%rip),%rcx        # 5444c <GCC_except_table5013+0x4a84>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310881:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310885:	48 01 c8             	add    %rcx,%rax
  310888:	ff e0                	jmp    *%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:167
  31088a:	4c 89 ee             	mov    %r13,%rsi
  31088d:	48 c1 fe 02          	sar    $0x2,%rsi
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:942
  310891:	66 0f ef c0          	pxor   %xmm0,%xmm0
  310895:	66 0f 7f 84 24 00 01 	movdqa %xmm0,0x100(%rsp)
  31089c:	00 00 
  31089e:	c7 84 24 10 01 00 00 	movl   $0x0,0x110(%rsp)
  3108a5:	00 00 00 00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:943
  3108a9:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  3108b0:	00 
  3108b1:	e8 9a f5 e9 ff       	call   1afe50 <rexx_exec::value::write_small_int>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:944
  3108b6:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  3108bb:	c6 44 24 40 03       	movb   $0x3,0x40(%rsp)
  3108c0:	88 44 24 41          	mov    %al,0x41(%rsp)
  3108c4:	8b 84 24 10 01 00 00 	mov    0x110(%rsp),%eax
  3108cb:	48 8d 4c 24 42       	lea    0x42(%rsp),%rcx
  3108d0:	89 41 10             	mov    %eax,0x10(%rcx)
  3108d3:	66 0f 6f 84 24 00 01 	movdqa 0x100(%rsp),%xmm0
  3108da:	00 00 
  3108dc:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  3108e0:	b0 03                	mov    $0x3,%al
  3108e2:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  3108e9:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  3108ea:	48 85 d2             	test   %rdx,%rdx
  3108ed:	0f 85 e4 fe ff ff    	jne    3107d7 <<rexx_exec::Interp>::exec_parse+0x13b7>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3108f3:	48 8b ac 24 e0 00 00 	mov    0xe0(%rsp),%rbp
  3108fa:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  3108fb:	4c 8b a4 24 e8 00 00 	mov    0xe8(%rsp),%r12
  310902:	00 
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310903:	0f b6 c0             	movzbl %al,%eax
  310906:	48 8d 0d 3f 3b d4 ff 	lea    -0x2bc4c1(%rip),%rcx        # 5444c <GCC_except_table5013+0x4a84>
  31090d:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310911:	48 01 c8             	add    %rcx,%rax
  310914:	ff e0                	jmp    *%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  310916:	44 89 e8             	mov    %r13d,%eax
  310919:	c1 e8 02             	shr    $0x2,%eax
  31091c:	83 e0 07             	and    $0x7,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  31091f:	4c 89 e9             	mov    %r13,%rcx
  310922:	48 81 e1 00 ff ff ff 	and    $0xffffffffffffff00,%rcx
  310929:	48 09 c1             	or     %rax,%rcx
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:931
  31092c:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  310931:	c6 44 24 40 02       	movb   $0x2,0x40(%rsp)
  310936:	48 89 4c 24 41       	mov    %rcx,0x41(%rsp)
  31093b:	b0 02                	mov    $0x2,%al
  31093d:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  310944:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310945:	48 85 d2             	test   %rdx,%rdx
  310948:	0f 85 89 fe ff ff    	jne    3107d7 <<rexx_exec::Interp>::exec_parse+0x13b7>
  31094e:	eb a3                	jmp    3108f3 <<rexx_exec::Interp>::exec_parse+0x14d3>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  310950:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  310955:	4c 89 f6             	mov    %r14,%rsi
  310958:	4c 89 ea             	mov    %r13,%rdx
  31095b:	31 c9                	xor    %ecx,%ecx
  31095d:	e8 8e 62 ec ff       	call   1d6bf0 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310962:	4c 8b 64 24 40       	mov    0x40(%rsp),%r12
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  310967:	4c 8b 6c 24 48       	mov    0x48(%rsp),%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31096c:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  310970:	0f 85 54 22 00 00    	jne    312bca <<rexx_exec::Interp>::exec_parse+0x37aa>
<rexx_exec::Interp>::apply_trigger:
  310976:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  31097b:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310982:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  310989:	75 0d                	jne    310998 <<rexx_exec::Interp>::exec_parse+0x1578>
  31098b:	48 8b bc 24 60 01 00 	mov    0x160(%rsp),%rdi
  310992:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310993:	e8 18 b5 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310998:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  31099f:	4c 89 2c d8          	mov    %r13,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3109a3:	48 ff c3             	inc    %rbx
  3109a6:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3109ad:	44 89 e8             	mov    %r13d,%eax
  3109b0:	83 e0 03             	and    $0x3,%eax
  3109b3:	48 8d 0d b2 3a d4 ff 	lea    -0x2bc54e(%rip),%rcx        # 5446c <GCC_except_table5013+0x4aa4>
  3109ba:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  3109be:	48 01 c8             	add    %rcx,%rax
  3109c1:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  3109c3:	4c 89 f7             	mov    %r14,%rdi
  3109c6:	4c 89 ee             	mov    %r13,%rsi
  3109c9:	e8 e2 de ec ff       	call   1de8b0 <<rexx_exec::Interp>::try_text>
<core::option::Option<&[u8]>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:634
  3109ce:	48 85 c0             	test   %rax,%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  3109d1:	0f 84 e0 01 00 00    	je     310bb7 <<rexx_exec::Interp>::exec_parse+0x1797>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:955
  3109d7:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  3109dc:	c6 44 24 40 00       	movb   $0x0,0x40(%rsp)
  3109e1:	31 c0                	xor    %eax,%eax
  3109e3:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  3109ea:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  3109eb:	48 85 d2             	test   %rdx,%rdx
  3109ee:	0f 84 1c 01 00 00    	je     310b10 <<rexx_exec::Interp>::exec_parse+0x16f0>
  3109f4:	48 83 fa 01          	cmp    $0x1,%rdx
  3109f8:	75 2e                	jne    310a28 <<rexx_exec::Interp>::exec_parse+0x1608>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  3109fa:	44 0f b6 a4 24 e0 00 	movzbl 0xe0(%rsp),%r12d
  310a01:	00 00 
  310a03:	49 83 fc 08          	cmp    $0x8,%r12
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310a07:	0f 83 72 26 00 00    	jae    31307f <<rexx_exec::Interp>::exec_parse+0x3c5f>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310a0d:	0f b6 c0             	movzbl %al,%eax
  310a10:	48 8d 0d 65 3a d4 ff 	lea    -0x2bc59b(%rip),%rcx        # 5447c <GCC_except_table5013+0x4ab4>
  310a17:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310a1b:	48 01 c8             	add    %rcx,%rax
  310a1e:	48 8d ac 24 e1 00 00 	lea    0xe1(%rsp),%rbp
  310a25:	00 
  310a26:	ff e0                	jmp    *%rax
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310a28:	48 8b 94 24 e0 00 00 	mov    0xe0(%rsp),%rdx
  310a2f:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310a30:	f6 c2 03             	test   $0x3,%dl
  310a33:	0f 85 15 28 00 00    	jne    31324e <<rexx_exec::Interp>::exec_parse+0x3e2e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310a39:	48 89 d1             	mov    %rdx,%rcx
  310a3c:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310a40:	89 ce                	mov    %ecx,%esi
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310a42:	49 39 b6 08 0a 00 00 	cmp    %rsi,0xa08(%r14)
  310a49:	0f 86 ff 27 00 00    	jbe    31324e <<rexx_exec::Interp>::exec_parse+0x3e2e>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310a4f:	49 8b 8e 00 0a 00 00 	mov    0xa00(%r14),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310a56:	48 8d 34 76          	lea    (%rsi,%rsi,2),%rsi
  310a5a:	48 c1 e6 05          	shl    $0x5,%rsi
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310a5e:	83 3c 31 01          	cmpl   $0x1,(%rcx,%rsi,1)
  310a62:	0f 85 e6 27 00 00    	jne    31324e <<rexx_exec::Interp>::exec_parse+0x3e2e>
  310a68:	48 01 f1             	add    %rsi,%rcx
  310a6b:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  310a6f:	0f 85 d9 27 00 00    	jne    31324e <<rexx_exec::Interp>::exec_parse+0x3e2e>
  310a75:	48 c1 ea 22          	shr    $0x22,%rdx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  310a79:	39 51 04             	cmp    %edx,0x4(%rcx)
  310a7c:	0f 85 cc 27 00 00    	jne    31324e <<rexx_exec::Interp>::exec_parse+0x3e2e>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  310a82:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  310a86:	0f b6 c0             	movzbl %al,%eax
  310a89:	0f 85 6a 02 00 00    	jne    310cf9 <<rexx_exec::Interp>::exec_parse+0x18d9>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310a8f:	48 8b 69 20          	mov    0x20(%rcx),%rbp
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310a93:	4c 8b 61 28          	mov    0x28(%rcx),%r12
  310a97:	48 8d 0d de 39 d4 ff 	lea    -0x2bc622(%rip),%rcx        # 5447c <GCC_except_table5013+0x4ab4>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310a9e:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310aa2:	48 01 c8             	add    %rcx,%rax
  310aa5:	ff e0                	jmp    *%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:167
  310aa7:	4c 89 ee             	mov    %r13,%rsi
  310aaa:	48 c1 fe 02          	sar    $0x2,%rsi
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:942
  310aae:	66 0f ef c0          	pxor   %xmm0,%xmm0
  310ab2:	66 0f 7f 84 24 00 01 	movdqa %xmm0,0x100(%rsp)
  310ab9:	00 00 
  310abb:	c7 84 24 10 01 00 00 	movl   $0x0,0x110(%rsp)
  310ac2:	00 00 00 00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:943
  310ac6:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  310acd:	00 
  310ace:	e8 7d f3 e9 ff       	call   1afe50 <rexx_exec::value::write_small_int>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:944
  310ad3:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  310ad8:	c6 44 24 40 03       	movb   $0x3,0x40(%rsp)
  310add:	88 44 24 41          	mov    %al,0x41(%rsp)
  310ae1:	8b 84 24 10 01 00 00 	mov    0x110(%rsp),%eax
  310ae8:	48 8d 4c 24 42       	lea    0x42(%rsp),%rcx
  310aed:	89 41 10             	mov    %eax,0x10(%rcx)
  310af0:	66 0f 6f 84 24 00 01 	movdqa 0x100(%rsp),%xmm0
  310af7:	00 00 
  310af9:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  310afd:	b0 03                	mov    $0x3,%al
  310aff:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  310b06:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310b07:	48 85 d2             	test   %rdx,%rdx
  310b0a:	0f 85 e4 fe ff ff    	jne    3109f4 <<rexx_exec::Interp>::exec_parse+0x15d4>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310b10:	48 8b ac 24 e0 00 00 	mov    0xe0(%rsp),%rbp
  310b17:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310b18:	4c 8b a4 24 e8 00 00 	mov    0xe8(%rsp),%r12
  310b1f:	00 
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310b20:	0f b6 c0             	movzbl %al,%eax
  310b23:	48 8d 0d 52 39 d4 ff 	lea    -0x2bc6ae(%rip),%rcx        # 5447c <GCC_except_table5013+0x4ab4>
  310b2a:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310b2e:	48 01 c8             	add    %rcx,%rax
  310b31:	ff e0                	jmp    *%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  310b33:	44 89 e8             	mov    %r13d,%eax
  310b36:	c1 e8 02             	shr    $0x2,%eax
  310b39:	83 e0 07             	and    $0x7,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  310b3c:	4c 89 e9             	mov    %r13,%rcx
  310b3f:	48 81 e1 00 ff ff ff 	and    $0xffffffffffffff00,%rcx
  310b46:	48 09 c1             	or     %rax,%rcx
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:931
  310b49:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  310b4e:	c6 44 24 40 02       	movb   $0x2,0x40(%rsp)
  310b53:	48 89 4c 24 41       	mov    %rcx,0x41(%rsp)
  310b58:	b0 02                	mov    $0x2,%al
  310b5a:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  310b61:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310b62:	48 85 d2             	test   %rdx,%rdx
  310b65:	0f 85 89 fe ff ff    	jne    3109f4 <<rexx_exec::Interp>::exec_parse+0x15d4>
  310b6b:	eb a3                	jmp    310b10 <<rexx_exec::Interp>::exec_parse+0x16f0>
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:960
  310b6d:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  310b74:	00 
  310b75:	4c 89 f6             	mov    %r14,%rsi
  310b78:	4c 89 ea             	mov    %r13,%rdx
  310b7b:	e8 b0 ec e9 ff       	call   1af830 <<rexx_exec::Interp>::to_text>
  310b80:	48 83 bc 24 00 01 00 	cmpq   $0xffffffffffffffff,0x100(%rsp)
  310b87:	00 ff 
  310b89:	0f 84 1f 04 00 00    	je     310fae <<rexx_exec::Interp>::exec_parse+0x1b8e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:962
  310b8f:	48 8b 84 24 10 01 00 	mov    0x110(%rsp),%rax
  310b96:	00 
  310b97:	48 8d 8c 24 27 01 00 	lea    0x127(%rsp),%rcx
  310b9e:	00 
  310b9f:	48 89 41 10          	mov    %rax,0x10(%rcx)
  310ba3:	f3 0f 6f 84 24 00 01 	movdqu 0x100(%rsp),%xmm0
  310baa:	00 00 
  310bac:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  310bb0:	b0 01                	mov    $0x1,%al
  310bb2:	e9 f9 03 00 00       	jmp    310fb0 <<rexx_exec::Interp>::exec_parse+0x1b90>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:960
  310bb7:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  310bbe:	00 
  310bbf:	4c 89 f6             	mov    %r14,%rsi
  310bc2:	4c 89 ea             	mov    %r13,%rdx
  310bc5:	e8 66 ec e9 ff       	call   1af830 <<rexx_exec::Interp>::to_text>
  310bca:	48 83 bc 24 00 01 00 	cmpq   $0xffffffffffffffff,0x100(%rsp)
  310bd1:	00 ff 
  310bd3:	0f 84 16 04 00 00    	je     310fef <<rexx_exec::Interp>::exec_parse+0x1bcf>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:962
  310bd9:	48 8b 84 24 10 01 00 	mov    0x110(%rsp),%rax
  310be0:	00 
  310be1:	48 8d 8c 24 27 01 00 	lea    0x127(%rsp),%rcx
  310be8:	00 
  310be9:	48 89 41 10          	mov    %rax,0x10(%rcx)
  310bed:	f3 0f 6f 84 24 00 01 	movdqu 0x100(%rsp),%xmm0
  310bf4:	00 00 
  310bf6:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  310bfa:	b0 01                	mov    $0x1,%al
  310bfc:	e9 f0 03 00 00       	jmp    310ff1 <<rexx_exec::Interp>::exec_parse+0x1bd1>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  310c01:	44 0f b6 61 11       	movzbl 0x11(%rcx),%r12d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  310c06:	48 83 c1 12          	add    $0x12,%rcx
  310c0a:	48 8d 15 3b 38 d4 ff 	lea    -0x2bc7c5(%rip),%rdx        # 5444c <GCC_except_table5013+0x4a84>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310c11:	48 63 04 82          	movslq (%rdx,%rax,4),%rax
  310c15:	48 01 d0             	add    %rdx,%rax
  310c18:	48 89 cd             	mov    %rcx,%rbp
  310c1b:	ff e0                	jmp    *%rax
  310c1d:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1695
  310c22:	4c 89 ee             	mov    %r13,%rsi
  310c25:	e8 86 dc ec ff       	call   1de8b0 <<rexx_exec::Interp>::try_text>
  310c2a:	49 89 c6             	mov    %rax,%r14
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  310c2d:	48 85 c0             	test   %rax,%rax
  310c30:	0f 84 a0 24 00 00    	je     3130d6 <<rexx_exec::Interp>::exec_parse+0x3cb6>
<rexx_exec::value::Rendered>::text:
  310c36:	48 89 d3             	mov    %rdx,%rbx
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  310c39:	eb 42                	jmp    310c7d <<rexx_exec::Interp>::exec_parse+0x185d>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  310c3b:	0f b6 5c 24 41       	movzbl 0x41(%rsp),%ebx
  310c40:	48 83 fb 07          	cmp    $0x7,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310c44:	0f 87 50 24 00 00    	ja     31309a <<rexx_exec::Interp>::exec_parse+0x3c7a>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
  310c4a:	4c 8d 74 24 42       	lea    0x42(%rsp),%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:86
  310c4f:	eb 2c                	jmp    310c7d <<rexx_exec::Interp>::exec_parse+0x185d>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  310c51:	44 0f b6 74 24 41    	movzbl 0x41(%rsp),%r14d
  310c57:	49 83 fe 14          	cmp    $0x14,%r14
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:544
  310c5b:	0f 87 57 24 00 00    	ja     3130b8 <<rexx_exec::Interp>::exec_parse+0x3c98>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  310c61:	bb 14 00 00 00       	mov    $0x14,%ebx
  310c66:	4c 29 f3             	sub    %r14,%rbx
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  310c69:	48 8d 44 24 42       	lea    0x42(%rsp),%rax
  310c6e:	49 01 c6             	add    %rax,%r14
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  310c71:	eb 0a                	jmp    310c7d <<rexx_exec::Interp>::exec_parse+0x185d>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310c73:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310c78:	48 8b 5c 24 58       	mov    0x58(%rsp),%rbx
<rexx_exec::Interp>::apply_trigger:
  310c7d:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
  310c82:	48 89 c1             	mov    %rax,%rcx
  310c85:	48 89 c2             	mov    %rax,%rdx
  310c88:	4d 29 fc             	sub    %r15,%r12
rexx_exec::parse_template::find:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:298
  310c8b:	0f 82 49 04 00 00    	jb     3110da <<rexx_exec::Interp>::exec_parse+0x1cba>
  310c91:	48 89 c1             	mov    %rax,%rcx
  310c94:	48 89 c2             	mov    %rax,%rdx
  310c97:	48 85 db             	test   %rbx,%rbx
  310c9a:	0f 84 3a 04 00 00    	je     3110da <<rexx_exec::Interp>::exec_parse+0x1cba>
  310ca0:	48 89 c1             	mov    %rax,%rcx
  310ca3:	48 89 c2             	mov    %rax,%rdx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:305
  310ca6:	4c 39 e3             	cmp    %r12,%rbx
  310ca9:	0f 87 2b 04 00 00    	ja     3110da <<rexx_exec::Interp>::exec_parse+0x1cba>
  310caf:	4c 01 fd             	add    %r15,%rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:316
  310cb2:	48 83 fb 01          	cmp    $0x1,%rbx
  310cb6:	0f 85 82 01 00 00    	jne    310e3e <<rexx_exec::Interp>::exec_parse+0x1a1e>
  310cbc:	41 0f b6 0e          	movzbl (%r14),%ecx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  310cc0:	8d 41 bf             	lea    -0x41(%rcx),%eax
  310cc3:	3c 1a                	cmp    $0x1a,%al
  310cc5:	0f 92 c0             	setb   %al
  310cc8:	c0 e0 05             	shl    $0x5,%al
  310ccb:	08 c8                	or     %cl,%al
  310ccd:	45 31 c0             	xor    %r8d,%r8d
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::parse_template::find::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  310cd0:	42 0f b6 4c 05 00    	movzbl 0x0(%rbp,%r8,1),%ecx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  310cd6:	8d 51 bf             	lea    -0x41(%rcx),%edx
  310cd9:	80 fa 1a             	cmp    $0x1a,%dl
  310cdc:	0f 92 c2             	setb   %dl
  310cdf:	c0 e2 05             	shl    $0x5,%dl
  310ce2:	08 ca                	or     %cl,%dl
rexx_exec::parse_template::find::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:320
  310ce4:	38 c2                	cmp    %al,%dl
  310ce6:	0f 84 e2 03 00 00    	je     3110ce <<rexx_exec::Interp>::exec_parse+0x1cae>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::parse_template::find::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:385
  310cec:	49 ff c0             	inc    %r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  310cef:	4d 39 c4             	cmp    %r8,%r12
  310cf2:	75 dc                	jne    310cd0 <<rexx_exec::Interp>::exec_parse+0x18b0>
  310cf4:	e9 79 01 00 00       	jmp    310e72 <<rexx_exec::Interp>::exec_parse+0x1a52>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  310cf9:	44 0f b6 61 11       	movzbl 0x11(%rcx),%r12d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  310cfe:	48 83 c1 12          	add    $0x12,%rcx
  310d02:	48 8d 15 73 37 d4 ff 	lea    -0x2bc88d(%rip),%rdx        # 5447c <GCC_except_table5013+0x4ab4>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310d09:	48 63 04 82          	movslq (%rdx,%rax,4),%rax
  310d0d:	48 01 d0             	add    %rdx,%rax
  310d10:	48 89 cd             	mov    %rcx,%rbp
  310d13:	ff e0                	jmp    *%rax
  310d15:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1695
  310d1a:	4c 89 ee             	mov    %r13,%rsi
  310d1d:	e8 8e db ec ff       	call   1de8b0 <<rexx_exec::Interp>::try_text>
  310d22:	49 89 c5             	mov    %rax,%r13
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  310d25:	48 85 c0             	test   %rax,%rax
  310d28:	0f 84 c5 23 00 00    	je     3130f3 <<rexx_exec::Interp>::exec_parse+0x3cd3>
<rexx_exec::value::Rendered>::text:
  310d2e:	48 89 d3             	mov    %rdx,%rbx
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  310d31:	eb 42                	jmp    310d75 <<rexx_exec::Interp>::exec_parse+0x1955>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  310d33:	0f b6 5c 24 41       	movzbl 0x41(%rsp),%ebx
  310d38:	48 83 fb 07          	cmp    $0x7,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310d3c:	0f 87 3a 23 00 00    	ja     31307c <<rexx_exec::Interp>::exec_parse+0x3c5c>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
  310d42:	4c 8d 6c 24 42       	lea    0x42(%rsp),%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:86
  310d47:	eb 2c                	jmp    310d75 <<rexx_exec::Interp>::exec_parse+0x1955>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  310d49:	44 0f b6 6c 24 41    	movzbl 0x41(%rsp),%r13d
  310d4f:	49 83 fd 14          	cmp    $0x14,%r13
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:544
  310d53:	0f 87 b7 23 00 00    	ja     313110 <<rexx_exec::Interp>::exec_parse+0x3cf0>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  310d59:	bb 14 00 00 00       	mov    $0x14,%ebx
  310d5e:	4c 29 eb             	sub    %r13,%rbx
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  310d61:	48 8d 44 24 42       	lea    0x42(%rsp),%rax
  310d66:	49 01 c5             	add    %rax,%r13
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  310d69:	eb 0a                	jmp    310d75 <<rexx_exec::Interp>::exec_parse+0x1955>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310d6b:	4c 8b 6c 24 50       	mov    0x50(%rsp),%r13
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310d70:	48 8b 5c 24 58       	mov    0x58(%rsp),%rbx
<rexx_exec::Interp>::apply_trigger:
  310d75:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
  310d7a:	48 89 c7             	mov    %rax,%rdi
  310d7d:	49 89 c0             	mov    %rax,%r8
  310d80:	4d 29 fc             	sub    %r15,%r12
rexx_exec::parse_template::find:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:298
  310d83:	0f 82 d3 02 00 00    	jb     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
  310d89:	48 89 c7             	mov    %rax,%rdi
  310d8c:	49 89 c0             	mov    %rax,%r8
  310d8f:	48 85 db             	test   %rbx,%rbx
  310d92:	0f 84 c4 02 00 00    	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
  310d98:	48 89 c7             	mov    %rax,%rdi
  310d9b:	49 89 c0             	mov    %rax,%r8
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:305
  310d9e:	4c 39 e3             	cmp    %r12,%rbx
  310da1:	0f 87 b5 02 00 00    	ja     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
  310da7:	4c 01 fd             	add    %r15,%rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:316
  310daa:	48 83 fb 01          	cmp    $0x1,%rbx
  310dae:	0f 85 ce 00 00 00    	jne    310e82 <<rexx_exec::Interp>::exec_parse+0x1a62>
  310db4:	41 0f b6 4d 00       	movzbl 0x0(%r13),%ecx
<[u8]>::as_chunks::<8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:1399
  310db9:	48 b8 01 00 00 00 00 	movabs $0x8000000000000001,%rax
  310dc0:	00 00 80 
  310dc3:	48 83 c0 f7          	add    $0xfffffffffffffff7,%rax
  310dc7:	4c 21 e0             	and    %r12,%rax
<[u8]>::split_at_unchecked:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:2062
  310dca:	44 89 e2             	mov    %r12d,%edx
  310dcd:	83 e2 07             	and    $0x7,%edx
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:160
  310dd0:	48 89 cf             	mov    %rcx,%rdi
  310dd3:	48 be 01 01 01 01 01 	movabs $0x101010101010101,%rsi
  310dda:	01 01 01 
  310ddd:	48 0f af fe          	imul   %rsi,%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:161
  310de1:	49 83 e4 f8          	and    $0xfffffffffffffff8,%r12
  310de5:	48 c7 c6 ff ff ff ff 	mov    $0xffffffffffffffff,%rsi
  310dec:	49 ba 80 80 80 80 80 	movabs $0x8080808080808080,%r10
  310df3:	80 80 80 
  310df6:	49 bb ff fe fe fe fe 	movabs $0xfefefefefefefeff,%r11
  310dfd:	fe fe fe 
<core::ptr::non_null::NonNull<[u8; 8]> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  310e00:	4d 85 e4             	test   %r12,%r12
<core::slice::iter::Iter<[u8; 8]> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  310e03:	0f 84 27 02 00 00    	je     311030 <<rexx_exec::Interp>::exec_parse+0x1c10>
rexx_exec::builtin::string::find_byte:
  310e09:	4c 8b 44 f5 08       	mov    0x8(%rbp,%rsi,8),%r8
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:162
  310e0e:	49 31 f8             	xor    %rdi,%r8
  310e11:	4d 89 c1             	mov    %r8,%r9
  310e14:	49 f7 d1             	not    %r9
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:163
  310e17:	4d 21 d1             	and    %r10,%r9
<u64>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  310e1a:	4d 01 d8             	add    %r11,%r8
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:164
  310e1d:	48 ff c6             	inc    %rsi
  310e20:	49 83 c4 f8          	add    $0xfffffffffffffff8,%r12
  310e24:	4d 21 c8             	and    %r9,%r8
  310e27:	74 d7                	je     310e00 <<rexx_exec::Interp>::exec_parse+0x19e0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:165
  310e29:	48 c1 e6 03          	shl    $0x3,%rsi
<u64>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:178
  310e2d:	f3 4d 0f bc f0       	tzcnt  %r8,%r14
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:165
  310e32:	41 c1 ee 03          	shr    $0x3,%r14d
  310e36:	49 09 f6             	or     %rsi,%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:171
  310e39:	e9 12 02 00 00       	jmp    311050 <<rexx_exec::Interp>::exec_parse+0x1c30>
<rexx_exec::Interp>::apply_trigger:
  310e3e:	48 83 fb 0f          	cmp    $0xf,%rbx
  310e42:	0f 86 28 02 00 00    	jbe    311070 <<rexx_exec::Interp>::exec_parse+0x1c50>
  310e48:	45 31 ed             	xor    %r13d,%r13d
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::position::check<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  310e4b:	48 89 ef             	mov    %rbp,%rdi
  310e4e:	4c 01 ef             	add    %r13,%rdi
<[u8]>::eq_ignore_ascii_case:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:76
  310e51:	48 89 de             	mov    %rbx,%rsi
  310e54:	4c 89 f2             	mov    %r14,%rdx
  310e57:	48 89 d9             	mov    %rbx,%rcx
  310e5a:	e8 91 65 e3 ff       	call   1473f0 <<[u8]>::eq_ignore_ascii_case_chunks::<16>>
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3142
  310e5f:	84 c0                	test   %al,%al
  310e61:	0f 85 64 02 00 00    	jne    3110cb <<rexx_exec::Interp>::exec_parse+0x1cab>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  310e67:	49 ff cc             	dec    %r12
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  310e6a:	49 ff c5             	inc    %r13
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  310e6d:	4c 39 e3             	cmp    %r12,%rbx
  310e70:	76 d9                	jbe    310e4b <<rexx_exec::Interp>::exec_parse+0x1a2b>
<rexx_exec::Interp>::apply_trigger:
  310e72:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
  310e77:	48 89 c1             	mov    %rax,%rcx
  310e7a:	48 89 c2             	mov    %rax,%rdx
  310e7d:	e9 58 02 00 00       	jmp    3110da <<rexx_exec::Interp>::exec_parse+0x1cba>
  310e82:	45 31 f6             	xor    %r14d,%r14d
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
  310e85:	66 66 2e 0f 1f 84 00 	data16 cs nopw 0x0(%rax,%rax,1)
  310e8c:	00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  310e90:	49 8d 3c 2e          	lea    (%r14,%rbp,1),%rdi
  310e94:	4c 89 ee             	mov    %r13,%rsi
  310e97:	48 89 da             	mov    %rbx,%rdx
  310e9a:	ff 15 a8 6d 1c 00    	call   *0x1c6da8(%rip)        # 4d7c48 <bcmp@GLIBC_2.2.5>
  310ea0:	85 c0                	test   %eax,%eax
  310ea2:	0f 84 a8 01 00 00    	je     311050 <<rexx_exec::Interp>::exec_parse+0x1c30>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  310ea8:	49 ff cc             	dec    %r12
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#1}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  310eab:	49 ff c6             	inc    %r14
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  310eae:	4c 39 e3             	cmp    %r12,%rbx
  310eb1:	76 dd                	jbe    310e90 <<rexx_exec::Interp>::exec_parse+0x1a70>
  310eb3:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
  310eb8:	48 89 c7             	mov    %rax,%rdi
  310ebb:	49 89 c0             	mov    %rax,%r8
  310ebe:	e9 99 01 00 00       	jmp    31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<rexx_exec::Interp>::apply_trigger:
  310ec3:	4c 8b 7c 24 28       	mov    0x28(%rsp),%r15
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  310ec8:	4c 89 f8             	mov    %r15,%rax
  310ecb:	48 29 d0             	sub    %rdx,%rax
  310ece:	b9 00 00 00 00       	mov    $0x0,%ecx
  310ed3:	48 0f 43 c8          	cmovae %rax,%rcx
  310ed7:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
  310edc:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  310ee1:	48 89 8c 24 c8 00 00 	mov    %rcx,0xc8(%rsp)
  310ee8:	00 
  310ee9:	48 89 c8             	mov    %rcx,%rax
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:97
  310eec:	e9 23 02 00 00       	jmp    311114 <<rexx_exec::Interp>::exec_parse+0x1cf4>
  310ef1:	4c 8b 7c 24 28       	mov    0x28(%rsp),%r15
<usize>::saturating_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2496
  310ef6:	4c 01 fa             	add    %r15,%rdx
  310ef9:	48 c7 c0 ff ff ff ff 	mov    $0xffffffffffffffff,%rax
  310f00:	48 0f 42 d0          	cmovb  %rax,%rdx
  310f04:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
<usize as core::cmp::Ord>::min:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:1090
  310f09:	48 39 d0             	cmp    %rdx,%rax
  310f0c:	48 0f 42 d0          	cmovb  %rax,%rdx
<rexx_exec::parse_template::Cursor>::forward:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:153
  310f10:	4c 39 fa             	cmp    %r15,%rdx
  310f13:	48 0f 47 c2          	cmova  %rdx,%rax
  310f17:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  310f1c:	49 0f 46 d7          	cmovbe %r15,%rdx
  310f20:	eb 52                	jmp    310f74 <<rexx_exec::Interp>::exec_parse+0x1b54>
<rexx_exec::Interp>::apply_trigger:
  310f22:	4c 8b 7c 24 28       	mov    0x28(%rsp),%r15
<usize>::saturating_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2496
  310f27:	4c 01 fa             	add    %r15,%rdx
  310f2a:	48 c7 c0 ff ff ff ff 	mov    $0xffffffffffffffff,%rax
  310f31:	48 0f 42 d0          	cmovb  %rax,%rdx
  310f35:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
<usize as core::cmp::Ord>::min:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:1090
  310f3a:	48 39 d0             	cmp    %rdx,%rax
  310f3d:	48 0f 42 d0          	cmovb  %rax,%rdx
  310f41:	48 89 54 24 20       	mov    %rdx,0x20(%rsp)
  310f46:	eb 2c                	jmp    310f74 <<rexx_exec::Interp>::exec_parse+0x1b54>
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  310f48:	48 83 ea 01          	sub    $0x1,%rdx
  310f4c:	b8 00 00 00 00       	mov    $0x0,%eax
  310f51:	48 0f 43 c2          	cmovae %rdx,%rax
  310f55:	48 8b 4c 24 38       	mov    0x38(%rsp),%rcx
<rexx_exec::parse_template::Cursor>::absolute:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:180
  310f5a:	48 39 c1             	cmp    %rax,%rcx
  310f5d:	48 89 c2             	mov    %rax,%rdx
  310f60:	48 0f 42 d1          	cmovb  %rcx,%rdx
  310f64:	4c 39 f8             	cmp    %r15,%rax
  310f67:	48 0f 47 ca          	cmova  %rdx,%rcx
  310f6b:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
  310f70:	48 0f 46 d0          	cmovbe %rax,%rdx
<rexx_exec::Interp>::apply_trigger:
  310f74:	48 89 94 24 c8 00 00 	mov    %rdx,0xc8(%rsp)
  310f7b:	00 
  310f7c:	48 89 d0             	mov    %rdx,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:99
  310f7f:	e9 90 01 00 00       	jmp    311114 <<rexx_exec::Interp>::exec_parse+0x1cf4>
  310f84:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  310f89:	48 89 c8             	mov    %rcx,%rax
  310f8c:	48 29 d0             	sub    %rdx,%rax
  310f8f:	41 bf 00 00 00 00    	mov    $0x0,%r15d
  310f95:	4c 0f 43 f8          	cmovae %rax,%r15
  310f99:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
  310f9e:	4c 89 f8             	mov    %r15,%rax
  310fa1:	4c 89 bc 24 c8 00 00 	mov    %r15,0xc8(%rsp)
  310fa8:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:100
  310fa9:	e9 66 01 00 00       	jmp    311114 <<rexx_exec::Interp>::exec_parse+0x1cf4>
  310fae:	31 c0                	xor    %eax,%eax
  310fb0:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  310fb7:	00 
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:964
  310fb8:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  310fbd:	88 44 24 40          	mov    %al,0x40(%rsp)
  310fc1:	f3 0f 6f 84 24 20 01 	movdqu 0x120(%rsp),%xmm0
  310fc8:	00 00 
  310fca:	f3 0f 6f 8c 24 2f 01 	movdqu 0x12f(%rsp),%xmm1
  310fd1:	00 00 
  310fd3:	48 8d 4c 24 41       	lea    0x41(%rsp),%rcx
  310fd8:	f3 0f 7f 49 0f       	movdqu %xmm1,0xf(%rcx)
  310fdd:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310fe1:	48 85 d2             	test   %rdx,%rdx
  310fe4:	0f 85 ed f7 ff ff    	jne    3107d7 <<rexx_exec::Interp>::exec_parse+0x13b7>
  310fea:	e9 04 f9 ff ff       	jmp    3108f3 <<rexx_exec::Interp>::exec_parse+0x14d3>
<rexx_exec::Interp>::apply_trigger:
  310fef:	31 c0                	xor    %eax,%eax
  310ff1:	48 8b 94 24 d0 00 00 	mov    0xd0(%rsp),%rdx
  310ff8:	00 
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:964
  310ff9:	4c 89 6c 24 60       	mov    %r13,0x60(%rsp)
  310ffe:	88 44 24 40          	mov    %al,0x40(%rsp)
  311002:	f3 0f 6f 84 24 20 01 	movdqu 0x120(%rsp),%xmm0
  311009:	00 00 
  31100b:	f3 0f 6f 8c 24 2f 01 	movdqu 0x12f(%rsp),%xmm1
  311012:	00 00 
  311014:	48 8d 4c 24 41       	lea    0x41(%rsp),%rcx
  311019:	f3 0f 7f 49 0f       	movdqu %xmm1,0xf(%rcx)
  31101e:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  311022:	48 85 d2             	test   %rdx,%rdx
  311025:	0f 85 c9 f9 ff ff    	jne    3109f4 <<rexx_exec::Interp>::exec_parse+0x15d4>
  31102b:	e9 e0 fa ff ff       	jmp    310b10 <<rexx_exec::Interp>::exec_parse+0x16f0>
<rexx_exec::Interp>::apply_trigger:
  311030:	48 8b 74 24 38       	mov    0x38(%rsp),%rsi
  311035:	48 89 f7             	mov    %rsi,%rdi
  311038:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31103b:	48 85 d2             	test   %rdx,%rdx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31103e:	74 1c                	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  311040:	38 4c 05 00          	cmp    %cl,0x0(%rbp,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  311044:	0f 85 38 14 00 00    	jne    312482 <<rexx_exec::Interp>::exec_parse+0x3062>
rexx_exec::builtin::string::find_byte:
  31104a:	45 31 f6             	xor    %r14d,%r14d
<core::option::Option<usize>>::map::<usize, rexx_exec::builtin::string::find_byte::{closure#1}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  31104d:	49 09 c6             	or     %rax,%r14
rexx_exec::parse_template::find::{closure#3}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:327
  311050:	4d 01 fe             	add    %r15,%r14
<rexx_exec::parse_template::Cursor>::match_at:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:233
  311053:	4c 01 f3             	add    %r14,%rbx
  311056:	4c 89 f7             	mov    %r14,%rdi
  311059:	49 89 d8             	mov    %rbx,%r8
<rexx_exec::Interp>::apply_trigger:
  31105c:	4c 89 c3             	mov    %r8,%rbx
  31105f:	48 89 7c 24 20       	mov    %rdi,0x20(%rsp)
core::ptr::drop_glue::<rexx_exec::value::Carried>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  311064:	80 7c 24 40 01       	cmpb   $0x1,0x40(%rsp)
  311069:	74 7e                	je     3110e9 <<rexx_exec::Interp>::exec_parse+0x1cc9>
  31106b:	e9 8c 00 00 00       	jmp    3110fc <<rexx_exec::Interp>::exec_parse+0x1cdc>
<rexx_exec::Interp>::apply_trigger:
  311070:	45 31 c0             	xor    %r8d,%r8d
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  311073:	49 ff cc             	dec    %r12
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  311076:	48 8d 45 01          	lea    0x1(%rbp),%rax
  31107a:	31 c9                	xor    %ecx,%ecx
<[u8]>::eq_ignore_ascii_case_simple:
  31107c:	0f 1f 40 00          	nopl   0x0(%rax)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:93
  311080:	48 39 cb             	cmp    %rcx,%rbx
  311083:	74 49                	je     3110ce <<rexx_exec::Interp>::exec_parse+0x1cae>
<u8>::to_ascii_lowercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:711
  311085:	0f b6 54 0d 00       	movzbl 0x0(%rbp,%rcx,1),%edx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  31108a:	8d 72 bf             	lea    -0x41(%rdx),%esi
  31108d:	40 80 fe 1a          	cmp    $0x1a,%sil
  311091:	40 0f 92 c6          	setb   %sil
  311095:	40 c0 e6 05          	shl    $0x5,%sil
  311099:	40 08 d6             	or     %dl,%sil
<u8>::to_ascii_lowercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:711
  31109c:	41 0f b6 14 0e       	movzbl (%r14,%rcx,1),%edx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  3110a1:	8d 7a bf             	lea    -0x41(%rdx),%edi
  3110a4:	40 80 ff 1a          	cmp    $0x1a,%dil
  3110a8:	40 0f 92 c7          	setb   %dil
  3110ac:	40 c0 e7 05          	shl    $0x5,%dil
  3110b0:	40 08 d7             	or     %dl,%dil
<[u8]>::eq_ignore_ascii_case_simple:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:94
  3110b3:	48 ff c1             	inc    %rcx
<u8>::eq_ignore_ascii_case:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:736
  3110b6:	40 38 fe             	cmp    %dil,%sil
<[u8]>::eq_ignore_ascii_case_simple:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:94
  3110b9:	74 c5                	je     311080 <<rexx_exec::Interp>::exec_parse+0x1c60>
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  3110bb:	49 ff c0             	inc    %r8
  3110be:	48 89 c5             	mov    %rax,%rbp
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  3110c1:	4c 39 e3             	cmp    %r12,%rbx
  3110c4:	76 ad                	jbe    311073 <<rexx_exec::Interp>::exec_parse+0x1c53>
  3110c6:	e9 a7 fd ff ff       	jmp    310e72 <<rexx_exec::Interp>::exec_parse+0x1a52>
rexx_exec::parse_template::find:
  3110cb:	4d 89 e8             	mov    %r13,%r8
  3110ce:	4c 89 c1             	mov    %r8,%rcx
rexx_exec::parse_template::find::{closure#3}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:327
  3110d1:	4c 01 f9             	add    %r15,%rcx
<rexx_exec::parse_template::Cursor>::match_at:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:233
  3110d4:	48 01 cb             	add    %rcx,%rbx
  3110d7:	48 89 da             	mov    %rbx,%rdx
<rexx_exec::Interp>::apply_trigger:
  3110da:	48 89 d3             	mov    %rdx,%rbx
  3110dd:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
core::ptr::drop_glue::<rexx_exec::value::Carried>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3110e2:	80 7c 24 40 01       	cmpb   $0x1,0x40(%rsp)
  3110e7:	75 13                	jne    3110fc <<rexx_exec::Interp>::exec_parse+0x1cdc>
  3110e9:	48 83 7c 24 48 00    	cmpq   $0x0,0x48(%rsp)
  3110ef:	74 0b                	je     3110fc <<rexx_exec::Interp>::exec_parse+0x1cdc>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3110f1:	48 8b 7c 24 50       	mov    0x50(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3110f6:	ff 15 0c 6b 1c 00    	call   *0x1c6b0c(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  3110fc:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
  311101:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  311108:	00 
  311109:	48 89 d8             	mov    %rbx,%rax
  31110c:	48 8b b4 24 b0 00 00 	mov    0xb0(%rsp),%rsi
  311113:	00 
  311114:	48 89 84 24 98 01 00 	mov    %rax,0x198(%rsp)
  31111b:	00 
<alloc::vec::Vec<core::option::Option<rexx_parse::ast::Expr>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  31111c:	48 8b 46 10          	mov    0x10(%rsi),%rax
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  311120:	48 89 c2             	mov    %rax,%rdx
  311123:	48 83 ea 01          	sub    $0x1,%rdx
  311127:	b9 00 00 00 00       	mov    $0x0,%ecx
  31112c:	48 0f 42 d1          	cmovb  %rcx,%rdx
  311130:	48 89 94 24 18 01 00 	mov    %rdx,0x118(%rsp)
  311137:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::Expr>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  311138:	48 85 c0             	test   %rax,%rax
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31113b:	0f 84 af f2 ff ff    	je     3103f0 <<rexx_exec::Interp>::exec_parse+0xfd0>
<rexx_exec::Interp>::assign_targets:
  311141:	4c 8b 6e 08          	mov    0x8(%rsi),%r13
  311145:	48 8d 04 80          	lea    (%rax,%rax,4),%rax
  311149:	48 c1 e0 04          	shl    $0x4,%rax
  31114d:	4c 01 e8             	add    %r13,%rax
  311150:	48 89 84 24 a8 01 00 	mov    %rax,0x1a8(%rsp)
  311157:	00 
  311158:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::traced_mode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:111
  31115d:	41 0f b6 8c 24 35 05 	movzbl 0x535(%r12),%ecx
  311164:	00 00 
  311166:	41 0f b6 84 24 2e 05 	movzbl 0x52e(%r12),%eax
  31116d:	00 00 
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31116f:	80 f1 01             	xor    $0x1,%cl
  311172:	89 ca                	mov    %ecx,%edx
  311174:	20 c2                	and    %al,%dl
  311176:	41 22 8c 24 2d 05 00 	and    0x52d(%r12),%cl
  31117d:	00 
  31117e:	88 54 24 37          	mov    %dl,0x37(%rsp)
  311182:	08 ca                	or     %cl,%dl
  311184:	88 94 24 bf 00 00 00 	mov    %dl,0xbf(%rsp)
  31118b:	80 f1 01             	xor    $0x1,%cl
  31118e:	08 c1                	or     %al,%cl
  311190:	88 8c 24 be 00 00 00 	mov    %cl,0xbe(%rsp)
  311197:	48 c7 84 24 b0 00 00 	movq   $0x0,0xb0(%rsp)
  31119e:	00 00 00 00 00 
  3111a3:	eb 58                	jmp    3111fd <<rexx_exec::Interp>::exec_parse+0x1ddd>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3111a5:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  3111ac:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  3111ad:	4b 8d 0c 7f          	lea    (%r15,%r15,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3111b1:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  3111b5:	48 8b 94 24 f8 00 00 	mov    0xf8(%rsp),%rdx
  3111bc:	00 
  3111bd:	48 89 54 c8 08       	mov    %rdx,0x8(%rax,%rcx,8)
  3111c2:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  3111c9:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3111cb:	49 ff c7             	inc    %r15
  3111ce:	4d 89 bc 24 e0 0b 00 	mov    %r15,0xbe0(%r12)
  3111d5:	00 
<core::iter::adapters::enumerate::Enumerate<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>>> as core::iter::traits::iterator::Iterator>::next:
  3111d6:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
  3111dd:	00 00 00 
  3111e0:	49 83 c5 50          	add    $0x50,%r13
  3111e4:	48 ff 84 24 b0 00 00 	incq   0xb0(%rsp)
  3111eb:	00 
  3111ec:	49 89 df             	mov    %rbx,%r15
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::Expr>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3111ef:	4c 3b ac 24 a8 01 00 	cmp    0x1a8(%rsp),%r13
  3111f6:	00 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3111f7:	0f 84 fb f1 ff ff    	je     3103f8 <<rexx_exec::Interp>::exec_parse+0xfd8>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:742
  3111fd:	49 83 7d 00 ff       	cmpq   $0xffffffffffffffff,0x0(%r13)
  311202:	0f 84 b8 00 00 00    	je     3112c0 <<rexx_exec::Interp>::exec_parse+0x1ea0>
  311208:	48 8b 84 24 d0 00 00 	mov    0xd0(%rsp),%rax
  31120f:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  311210:	48 85 c0             	test   %rax,%rax
  311213:	48 8b 74 24 38       	mov    0x38(%rsp),%rsi
  311218:	4c 89 6c 24 28       	mov    %r13,0x28(%rsp)
  31121d:	0f 84 ed 00 00 00    	je     311310 <<rexx_exec::Interp>::exec_parse+0x1ef0>
  311223:	48 83 f8 01          	cmp    $0x1,%rax
  311227:	0f 85 c3 01 00 00    	jne    3113f0 <<rexx_exec::Interp>::exec_parse+0x1fd0>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  31122d:	0f b6 84 24 e0 00 00 	movzbl 0xe0(%rsp),%eax
  311234:	00 
  311235:	4c 8d ac 24 e1 00 00 	lea    0xe1(%rsp),%r13
  31123c:	00 
  31123d:	48 83 f8 08          	cmp    $0x8,%rax
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311241:	0f 83 1d 19 00 00    	jae    312b64 <<rexx_exec::Interp>::exec_parse+0x3744>
<rexx_exec::Interp>::assign_targets:
  311247:	48 8b 8c 24 b0 00 00 	mov    0xb0(%rsp),%rcx
  31124e:	00 
  31124f:	48 3b 8c 24 18 01 00 	cmp    0x118(%rsp),%rcx
  311256:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  311257:	0f 85 d9 00 00 00    	jne    311336 <<rexx_exec::Interp>::exec_parse+0x1f16>
  31125d:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
  311262:	48 89 cb             	mov    %rcx,%rbx
  311265:	49 89 ce             	mov    %rcx,%r14
  311268:	49 39 cf             	cmp    %rcx,%r15
<rexx_exec::parse_template::Cursor>::remainder:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:286
  31126b:	0f 83 4f 04 00 00    	jae    3116c0 <<rexx_exec::Interp>::exec_parse+0x22a0>
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311271:	4d 89 f4             	mov    %r14,%r12
  311274:	4d 29 fc             	sub    %r15,%r12
  311277:	0f 82 d9 17 00 00    	jb     312a56 <<rexx_exec::Interp>::exec_parse+0x3636>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:408
  31127d:	49 39 c6             	cmp    %rax,%r14
  311280:	0f 87 d0 17 00 00    	ja     312a56 <<rexx_exec::Interp>::exec_parse+0x3636>
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  311286:	4d 01 fd             	add    %r15,%r13
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:131
  311289:	49 83 fc 07          	cmp    $0x7,%r12
  31128d:	0f 86 2b 02 00 00    	jbe    3114be <<rexx_exec::Interp>::exec_parse+0x209e>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:51
  311293:	49 83 fc 37          	cmp    $0x37,%r12
  311297:	0f 83 03 03 00 00    	jae    3115a0 <<rexx_exec::Interp>::exec_parse+0x2180>
<core::mem::maybe_uninit::MaybeUninit<u8>>::write:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/maybe_uninit.rs:575
  31129d:	48 8d 7c 24 4a       	lea    0x4a(%rsp),%rdi
  3112a2:	4c 89 ee             	mov    %r13,%rsi
  3112a5:	4c 89 e2             	mov    %r12,%rdx
  3112a8:	ff 15 8a 69 1c 00    	call   *0x1c698a(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:61
  3112ae:	c6 44 24 48 00       	movb   $0x0,0x48(%rsp)
  3112b3:	44 88 64 24 49       	mov    %r12b,0x49(%rsp)
  3112b8:	e9 1b 03 00 00       	jmp    3115d8 <<rexx_exec::Interp>::exec_parse+0x21b8>
<rexx_exec::Interp>::assign_targets:
  3112bd:	0f 1f 00             	nopl   (%rax)
  3112c0:	48 8b 84 24 b0 00 00 	mov    0xb0(%rsp),%rax
  3112c7:	00 
  3112c8:	48 3b 84 24 18 01 00 	cmp    0x118(%rsp),%rax
  3112cf:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:798
  3112d0:	0f 85 a2 01 00 00    	jne    311478 <<rexx_exec::Interp>::exec_parse+0x2058>
  3112d6:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
<rexx_exec::parse_template::Cursor>::remainder:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:286
  3112db:	49 39 c7             	cmp    %rax,%r15
  3112de:	48 89 c3             	mov    %rax,%rbx
  3112e1:	49 0f 47 df          	cmova  %r15,%rbx
  3112e5:	41 b8 00 00 00 00    	mov    $0x0,%r8d
  3112eb:	4c 0f 42 c0          	cmovb  %rax,%r8
  3112ef:	b8 00 00 00 00       	mov    $0x0,%eax
  3112f4:	4c 0f 43 f8          	cmovae %rax,%r15
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  3112f8:	f6 44 24 37 01       	testb  $0x1,0x37(%rsp)
  3112fd:	0f 84 dd fe ff ff    	je     3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
  311303:	e9 bc 07 00 00       	jmp    311ac4 <<rexx_exec::Interp>::exec_parse+0x26a4>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
  311308:	0f 1f 84 00 00 00 00 	nopl   0x0(%rax,%rax,1)
  31130f:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311310:	4c 8b ac 24 e0 00 00 	mov    0xe0(%rsp),%r13
  311317:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311318:	48 8b 84 24 e8 00 00 	mov    0xe8(%rsp),%rax
  31131f:	00 
  311320:	48 8b 8c 24 b0 00 00 	mov    0xb0(%rsp),%rcx
  311327:	00 
<rexx_exec::Interp>::assign_targets:
  311328:	48 3b 8c 24 18 01 00 	cmp    0x118(%rsp),%rcx
  31132f:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  311330:	0f 84 27 ff ff ff    	je     31125d <<rexx_exec::Interp>::exec_parse+0x1e3d>
  311336:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  31133b:	0f 83 7f 03 00 00    	jae    3116c0 <<rexx_exec::Interp>::exec_parse+0x22a0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  311341:	49 39 f7             	cmp    %rsi,%r15
  311344:	73 3f                	jae    311385 <<rexx_exec::Interp>::exec_parse+0x1f65>
  311346:	49 39 c7             	cmp    %rax,%r15
  311349:	48 89 c7             	mov    %rax,%rdi
  31134c:	49 0f 47 ff          	cmova  %r15,%rdi
  311350:	eb 1a                	jmp    31136c <<rexx_exec::Interp>::exec_parse+0x1f4c>
  311352:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  311359:	1f 84 00 00 00 00 00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:257
  311360:	49 ff c7             	inc    %r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  311363:	4c 39 fe             	cmp    %r15,%rsi
  311366:	0f 84 3c 03 00 00    	je     3116a8 <<rexx_exec::Interp>::exec_parse+0x2288>
  31136c:	4c 39 ff             	cmp    %r15,%rdi
  31136f:	0f 84 e5 1d 00 00    	je     31315a <<rexx_exec::Interp>::exec_parse+0x3d3a>
  311375:	43 0f b6 4c 3d 00    	movzbl 0x0(%r13,%r15,1),%ecx
  31137b:	83 f9 20             	cmp    $0x20,%ecx
rexx_exec::parse_template::is_blank:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:31
  31137e:	74 e0                	je     311360 <<rexx_exec::Interp>::exec_parse+0x1f40>
  311380:	83 f9 09             	cmp    $0x9,%ecx
  311383:	74 db                	je     311360 <<rexx_exec::Interp>::exec_parse+0x1f40>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311385:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  31138a:	0f 83 30 03 00 00    	jae    3116c0 <<rexx_exec::Interp>::exec_parse+0x22a0>
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2489
  311390:	49 39 c7             	cmp    %rax,%r15
  311393:	48 89 c7             	mov    %rax,%rdi
  311396:	49 0f 47 ff          	cmova  %r15,%rdi
  31139a:	4c 89 fb             	mov    %r15,%rbx
  31139d:	4c 8b 74 24 20       	mov    0x20(%rsp),%r14
<usize as core::cmp::PartialOrd>::lt:
  3113a2:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  3113a9:	1f 84 00 00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2144
  3113b0:	49 39 de             	cmp    %rbx,%r14
<core::ops::range::Range<usize> as core::iter::range::RangeIteratorImpl>::spec_next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1100
  3113b3:	0f 84 55 08 00 00    	je     311c0e <<rexx_exec::Interp>::exec_parse+0x27ee>
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  3113b9:	48 39 df             	cmp    %rbx,%rdi
  3113bc:	0f 84 89 1d 00 00    	je     31314b <<rexx_exec::Interp>::exec_parse+0x3d2b>
  3113c2:	41 0f b6 4c 1d 00    	movzbl 0x0(%r13,%rbx,1),%ecx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  3113c8:	48 ff c3             	inc    %rbx
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  3113cb:	83 f9 09             	cmp    $0x9,%ecx
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  3113ce:	74 05                	je     3113d5 <<rexx_exec::Interp>::exec_parse+0x1fb5>
  3113d0:	83 f9 20             	cmp    $0x20,%ecx
  3113d3:	75 db                	jne    3113b0 <<rexx_exec::Interp>::exec_parse+0x1f90>
  3113d5:	4c 8d 73 ff          	lea    -0x1(%rbx),%r14
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  3113d9:	4d 89 f4             	mov    %r14,%r12
  3113dc:	4d 29 fc             	sub    %r15,%r12
  3113df:	0f 83 98 fe ff ff    	jae    31127d <<rexx_exec::Interp>::exec_parse+0x1e5d>
  3113e5:	e9 6c 16 00 00       	jmp    312a56 <<rexx_exec::Interp>::exec_parse+0x3636>
<rexx_exec::parse_template::SourceText>::bytes:
  3113ea:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  3113f0:	48 8b 84 24 e0 00 00 	mov    0xe0(%rsp),%rax
  3113f7:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3113f8:	a8 03                	test   $0x3,%al
  3113fa:	0f 85 2e 1d 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  311400:	48 89 c1             	mov    %rax,%rcx
  311403:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311407:	89 ca                	mov    %ecx,%edx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  311409:	49 39 94 24 08 0a 00 	cmp    %rdx,0xa08(%r12)
  311410:	00 
  311411:	0f 86 17 1d 00 00    	jbe    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311417:	49 8b 8c 24 00 0a 00 	mov    0xa00(%r12),%rcx
  31141e:	00 
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  31141f:	48 8d 14 52          	lea    (%rdx,%rdx,2),%rdx
  311423:	48 c1 e2 05          	shl    $0x5,%rdx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311427:	83 3c 11 01          	cmpl   $0x1,(%rcx,%rdx,1)
  31142b:	0f 85 fd 1c 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
  311431:	48 01 d1             	add    %rdx,%rcx
  311434:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  311438:	0f 85 f0 1c 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
  31143e:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  311442:	39 41 04             	cmp    %eax,0x4(%rcx)
  311445:	0f 85 e3 1c 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  31144b:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  31144f:	0f 85 88 00 00 00    	jne    3114dd <<rexx_exec::Interp>::exec_parse+0x20bd>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311455:	4c 8b 69 20          	mov    0x20(%rcx),%r13
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311459:	48 8b 41 28          	mov    0x28(%rcx),%rax
  31145d:	48 8b 8c 24 b0 00 00 	mov    0xb0(%rsp),%rcx
  311464:	00 
<rexx_exec::Interp>::assign_targets:
  311465:	48 3b 8c 24 18 01 00 	cmp    0x118(%rsp),%rcx
  31146c:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  31146d:	0f 85 c3 fe ff ff    	jne    311336 <<rexx_exec::Interp>::exec_parse+0x1f16>
  311473:	e9 e5 fd ff ff       	jmp    31125d <<rexx_exec::Interp>::exec_parse+0x1e3d>
<rexx_exec::Interp>::assign_targets:
  311478:	48 8b 84 24 d0 00 00 	mov    0xd0(%rsp),%rax
  31147f:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  311480:	48 85 c0             	test   %rax,%rax
  311483:	48 8b 74 24 38       	mov    0x38(%rsp),%rsi
  311488:	74 79                	je     311503 <<rexx_exec::Interp>::exec_parse+0x20e3>
  31148a:	48 83 f8 01          	cmp    $0x1,%rax
  31148e:	0f 85 8f 00 00 00    	jne    311523 <<rexx_exec::Interp>::exec_parse+0x2103>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  311494:	0f b6 84 24 e0 00 00 	movzbl 0xe0(%rsp),%eax
  31149b:	00 
  31149c:	48 8d 8c 24 e1 00 00 	lea    0xe1(%rsp),%rcx
  3114a3:	00 
  3114a4:	48 83 f8 08          	cmp    $0x8,%rax
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  3114a8:	0f 83 b6 16 00 00    	jae    312b64 <<rexx_exec::Interp>::exec_parse+0x3744>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  3114ae:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  3114b3:	0f 83 f7 05 00 00    	jae    311ab0 <<rexx_exec::Interp>::exec_parse+0x2690>
  3114b9:	e9 5c 05 00 00       	jmp    311a1a <<rexx_exec::Interp>::exec_parse+0x25fa>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:134
  3114be:	48 c7 44 24 40 00 00 	movq   $0x0,0x40(%rsp)
  3114c5:	00 00 
  3114c7:	bd 03 00 00 00       	mov    $0x3,%ebp
<core::iter::adapters::zip::Zip<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>> as core::iter::adapters::zip::ZipImpl<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>>>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/zip.rs:300
  3114cc:	4d 39 fe             	cmp    %r15,%r14
  3114cf:	0f 85 8f 01 00 00    	jne    311664 <<rexx_exec::Interp>::exec_parse+0x2244>
<rexx_exec::Interp>::take_piece:
  3114d5:	4d 89 fe             	mov    %r15,%r14
  3114d8:	e9 a9 01 00 00       	jmp    311686 <<rexx_exec::Interp>::exec_parse+0x2266>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  3114dd:	0f b6 41 11          	movzbl 0x11(%rcx),%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  3114e1:	48 83 c1 12          	add    $0x12,%rcx
  3114e5:	49 89 cd             	mov    %rcx,%r13
  3114e8:	48 8b 8c 24 b0 00 00 	mov    0xb0(%rsp),%rcx
  3114ef:	00 
<rexx_exec::Interp>::assign_targets:
  3114f0:	48 3b 8c 24 18 01 00 	cmp    0x118(%rsp),%rcx
  3114f7:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  3114f8:	0f 84 5f fd ff ff    	je     31125d <<rexx_exec::Interp>::exec_parse+0x1e3d>
  3114fe:	e9 33 fe ff ff       	jmp    311336 <<rexx_exec::Interp>::exec_parse+0x1f16>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311503:	48 8b 8c 24 e0 00 00 	mov    0xe0(%rsp),%rcx
  31150a:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  31150b:	48 8b 84 24 e8 00 00 	mov    0xe8(%rsp),%rax
  311512:	00 
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311513:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311518:	0f 82 fc 04 00 00    	jb     311a1a <<rexx_exec::Interp>::exec_parse+0x25fa>
  31151e:	e9 8d 05 00 00       	jmp    311ab0 <<rexx_exec::Interp>::exec_parse+0x2690>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311523:	48 8b 84 24 e0 00 00 	mov    0xe0(%rsp),%rax
  31152a:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31152b:	a8 03                	test   $0x3,%al
  31152d:	0f 85 fb 1b 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  311533:	48 89 c1             	mov    %rax,%rcx
  311536:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  31153a:	89 c9                	mov    %ecx,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  31153c:	49 39 8c 24 08 0a 00 	cmp    %rcx,0xa08(%r12)
  311543:	00 
  311544:	0f 86 e4 1b 00 00    	jbe    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  31154a:	49 8b 94 24 00 0a 00 	mov    0xa00(%r12),%rdx
  311551:	00 
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  311552:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  311556:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  31155a:	83 3c 0a 01          	cmpl   $0x1,(%rdx,%rcx,1)
  31155e:	0f 85 ca 1b 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
  311564:	48 01 ca             	add    %rcx,%rdx
  311567:	80 7a 08 00          	cmpb   $0x0,0x8(%rdx)
  31156b:	0f 85 bd 1b 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
  311571:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  311575:	39 42 04             	cmp    %eax,0x4(%rdx)
  311578:	0f 85 b0 1b 00 00    	jne    31312e <<rexx_exec::Interp>::exec_parse+0x3d0e>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  31157e:	80 7a 10 01          	cmpb   $0x1,0x10(%rdx)
  311582:	0f 85 7c 04 00 00    	jne    311a04 <<rexx_exec::Interp>::exec_parse+0x25e4>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311588:	48 8b 4a 20          	mov    0x20(%rdx),%rcx
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  31158c:	48 8b 42 28          	mov    0x28(%rdx),%rax
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311590:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311595:	0f 83 15 05 00 00    	jae    311ab0 <<rexx_exec::Interp>::exec_parse+0x2690>
  31159b:	e9 7a 04 00 00       	jmp    311a1a <<rexx_exec::Interp>::exec_parse+0x25fa>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  3115a0:	4c 89 e7             	mov    %r12,%rdi
  3115a3:	ff 15 7f 66 1c 00    	call   *0x1c667f(%rip)        # 4d7c28 <malloc@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  3115a9:	48 85 c0             	test   %rax,%rax
  3115ac:	0f 84 73 1c 00 00    	je     313225 <<rexx_exec::Interp>::exec_parse+0x3e05>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  3115b2:	48 89 c5             	mov    %rax,%rbp
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  3115b5:	48 89 c7             	mov    %rax,%rdi
  3115b8:	4c 89 ee             	mov    %r13,%rsi
  3115bb:	4c 89 e2             	mov    %r12,%rdx
  3115be:	ff 15 74 66 1c 00    	call   *0x1c6674(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:66
  3115c4:	c6 44 24 48 01       	movb   $0x1,0x48(%rsp)
  3115c9:	4c 89 64 24 50       	mov    %r12,0x50(%rsp)
  3115ce:	48 89 6c 24 58       	mov    %rbp,0x58(%rsp)
  3115d3:	4c 89 64 24 60       	mov    %r12,0x60(%rsp)
<rexx_exec::Interp>::text_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:230
  3115d8:	48 c7 84 24 80 00 00 	movq   $0x0,0x80(%rsp)
  3115df:	00 00 00 00 00 
  3115e4:	c6 44 24 40 00       	movb   $0x0,0x40(%rsp)
  3115e9:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
  3115ee:	31 f6                	xor    %esi,%esi
  3115f0:	48 8d 54 24 40       	lea    0x40(%rsp),%rdx
  3115f5:	e8 06 ac e7 ff       	call   18c200 <<rexx_exec::Interp>::alloc_with>
  3115fa:	4c 8b 6c 24 28       	mov    0x28(%rsp),%r13
<rexx_exec::Interp>::take_piece:
  3115ff:	48 89 c5             	mov    %rax,%rbp
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::size_hint:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1190
  311602:	4c 89 f1             	mov    %r14,%rcx
  311605:	4c 29 f9             	sub    %r15,%rcx
  311608:	b8 00 00 00 00       	mov    $0x0,%eax
  31160d:	48 0f 43 c1          	cmovae %rcx,%rax
<rexx_exec::Interp>::charge_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:104
  311611:	48 83 f8 37          	cmp    $0x37,%rax
  311615:	72 76                	jb     31168d <<rexx_exec::Interp>::exec_parse+0x226d>
  311617:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_core::heap::Heap>::hold_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:183
  31161c:	49 01 84 24 a0 0a 00 	add    %rax,0xaa0(%r12)
  311623:	00 
<rexx_core::heap::Heap>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:176
  311624:	49 03 84 24 90 0a 00 	add    0xa90(%r12),%rax
  31162b:	00 
  31162c:	49 89 84 24 90 0a 00 	mov    %rax,0xa90(%r12)
  311633:	00 
<rexx_exec::Interp>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:3249
  311634:	49 3b 84 24 c8 1f 00 	cmp    0x1fc8(%r12),%rax
  31163b:	00 
  31163c:	0f 93 c0             	setae  %al
  31163f:	41 08 84 24 ff 1f 00 	or     %al,0x1fff(%r12)
  311646:	00 
  311647:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  31164a:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  311651:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311652:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  311659:	0f 84 88 00 00 00    	je     3116e7 <<rexx_exec::Interp>::exec_parse+0x22c7>
  31165f:	e9 90 00 00 00       	jmp    3116f4 <<rexx_exec::Interp>::exec_parse+0x22d4>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:136
  311664:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  311669:	4c 89 ee             	mov    %r13,%rsi
  31166c:	4c 89 e2             	mov    %r12,%rdx
  31166f:	ff 15 c3 65 1c 00    	call   *0x1c65c3(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:138
  311675:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:140
  31167a:	48 c1 e0 08          	shl    $0x8,%rax
  31167e:	4a 8d 2c a0          	lea    (%rax,%r12,4),%rbp
  311682:	48 83 c5 03          	add    $0x3,%rbp
<rexx_exec::Interp>::assign_targets:
  311686:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  31168b:	eb 41                	jmp    3116ce <<rexx_exec::Interp>::exec_parse+0x22ae>
  31168d:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311692:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311695:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  31169c:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  31169d:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  3116a4:	74 41                	je     3116e7 <<rexx_exec::Interp>::exec_parse+0x22c7>
  3116a6:	eb 4c                	jmp    3116f4 <<rexx_exec::Interp>::exec_parse+0x22d4>
<rexx_exec::Interp>::assign_targets:
  3116a8:	49 89 f7             	mov    %rsi,%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  3116ab:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  3116b0:	0f 82 da fc ff ff    	jb     311390 <<rexx_exec::Interp>::exec_parse+0x1f70>
<rexx_exec::Interp>::assign_targets:
  3116b6:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
  3116bd:	00 00 00 
  3116c0:	bd 03 00 00 00       	mov    $0x3,%ebp
  3116c5:	4c 89 fb             	mov    %r15,%rbx
  3116c8:	45 31 ff             	xor    %r15d,%r15d
  3116cb:	45 31 f6             	xor    %r14d,%r14d
  3116ce:	4c 8b 6c 24 28       	mov    0x28(%rsp),%r13
  3116d3:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  3116d6:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  3116dd:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3116de:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  3116e5:	75 0d                	jne    3116f4 <<rexx_exec::Interp>::exec_parse+0x22d4>
  3116e7:	48 8b bc 24 60 01 00 	mov    0x160(%rsp),%rdi
  3116ee:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3116ef:	e8 bc a7 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  3116f4:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3116f9:	48 8b 81 e8 0a 00 00 	mov    0xae8(%rcx),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  311700:	4a 89 2c e0          	mov    %rbp,(%rax,%r12,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  311704:	49 ff c4             	inc    %r12
  311707:	4c 89 a1 f0 0a 00 00 	mov    %r12,0xaf0(%rcx)
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:760
  31170e:	48 b8 01 00 00 00 00 	movabs $0x8000000000000001,%rax
  311715:	00 00 80 
  311718:	48 ff c0             	inc    %rax
  31171b:	49 39 45 00          	cmp    %rax,0x0(%r13)
  31171f:	0f 85 cb 00 00 00    	jne    3117f0 <<rexx_exec::Interp>::exec_parse+0x23d0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:761
  311725:	41 8b 45 08          	mov    0x8(%r13),%eax
<usize as core::slice::index::SliceIndex<[core::option::Option<usize>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  311729:	48 39 84 24 08 02 00 	cmp    %rax,0x208(%rsp)
  311730:	00 
  311731:	0f 86 b9 00 00 00    	jbe    3117f0 <<rexx_exec::Interp>::exec_parse+0x23d0>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  311737:	48 89 c1             	mov    %rax,%rcx
  31173a:	48 c1 e1 04          	shl    $0x4,%rcx
  31173e:	48 8b 94 24 f8 01 00 	mov    0x1f8(%rsp),%rdx
  311745:	00 
<core::option::Option<usize>>::map::<(rexx_parse::token::symbols::SymbolId, usize), <rexx_exec::Interp>::assign_targets::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  311746:	83 3c 0a 01          	cmpl   $0x1,(%rdx,%rcx,1)
  31174a:	0f 85 a0 00 00 00    	jne    3117f0 <<rexx_exec::Interp>::exec_parse+0x23d0>
<rexx_exec::Code>::slot_for:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1263
  311750:	48 01 d1             	add    %rdx,%rcx
<core::option::Option<&core::option::Option<usize>>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  311753:	48 8b 49 08          	mov    0x8(%rcx),%rcx
  311757:	48 89 8c 24 a0 01 00 	mov    %rcx,0x1a0(%rsp)
  31175e:	00 
  31175f:	41 b4 01             	mov    $0x1,%r12b
  311762:	89 84 24 bc 01 00 00 	mov    %eax,0x1bc(%rsp)
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:768
  311769:	f6 84 24 bf 00 00 00 	testb  $0x1,0xbf(%rsp)
  311770:	01 
  311771:	0f 85 86 00 00 00    	jne    3117fd <<rexx_exec::Interp>::exec_parse+0x23dd>
  311777:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  31177e:	00 00 
  311780:	49 c7 c6 ff ff ff ff 	mov    $0xffffffffffffffff,%r14
  311787:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  31178a:	45 84 e4             	test   %r12b,%r12b
  31178d:	0f 84 c3 00 00 00    	je     311856 <<rexx_exec::Interp>::exec_parse+0x2436>
  311793:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:73
  311798:	49 8b 84 24 70 04 00 	mov    0x470(%r12),%rax
  31179f:	00 
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  3117a0:	48 85 c0             	test   %rax,%rax
  3117a3:	0f 84 29 14 00 00    	je     312bd2 <<rexx_exec::Interp>::exec_parse+0x37b2>
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:73
  3117a9:	48 8b b0 f8 00 00 00 	mov    0xf8(%rax),%rsi
  3117b0:	48 8b 90 00 01 00 00 	mov    0x100(%rax),%rdx
<alloc::vec::Vec<(usize, rexx_exec::activation::InstanceVar)>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3126
  3117b7:	48 83 78 50 00       	cmpq   $0x0,0x50(%rax)
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:87
  3117bc:	0f 84 e6 00 00 00    	je     3118a8 <<rexx_exec::Interp>::exec_parse+0x2488>
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:40
  3117c2:	4c 89 e7             	mov    %r12,%rdi
  3117c5:	48 8b 8c 24 a0 01 00 	mov    0x1a0(%rsp),%rcx
  3117cc:	00 
  3117cd:	49 89 e8             	mov    %rbp,%r8
  3117d0:	e8 3b e7 f0 ff       	call   21ff10 <<rexx_exec::Interp>::set_exposed_variable>
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:77
  3117d5:	4d 85 ff             	test   %r15,%r15
  3117d8:	0f 85 14 01 00 00    	jne    3118f2 <<rexx_exec::Interp>::exec_parse+0x24d2>
  3117de:	e9 5b 01 00 00       	jmp    31193e <<rexx_exec::Interp>::exec_parse+0x251e>
<rexx_exec::Interp>::assign_targets:
  3117e3:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  3117ea:	84 00 00 00 00 00 
  3117f0:	45 31 e4             	xor    %r12d,%r12d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:768
  3117f3:	f6 84 24 bf 00 00 00 	testb  $0x1,0xbf(%rsp)
  3117fa:	01 
  3117fb:	74 83                	je     311780 <<rexx_exec::Interp>::exec_parse+0x2360>
<rexx_exec::Interp>::assign_targets::{closure#1}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:769
  3117fd:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  311802:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  311807:	48 8d 94 24 d8 00 00 	lea    0xd8(%rsp),%rdx
  31180e:	00 
  31180f:	4c 89 f9             	mov    %r15,%rcx
  311812:	4d 89 f0             	mov    %r14,%r8
  311815:	e8 66 56 04 00       	call   356e80 <<rexx_exec::Interp>::copied_piece>
<bool>::then::<alloc::vec::Vec<u8>, <rexx_exec::Interp>::assign_targets::{closure#1}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/bool.rs:66
  31181a:	4c 8b 74 24 40       	mov    0x40(%rsp),%r14
  31181f:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  311824:	48 8b 44 24 50       	mov    0x50(%rsp),%rax
  311829:	48 89 84 24 e8 01 00 	mov    %rax,0x1e8(%rsp)
  311830:	00 
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:770
  311831:	f6 44 24 37 01       	testb  $0x1,0x37(%rsp)
  311836:	4c 89 bc 24 f8 00 00 	mov    %r15,0xf8(%rsp)
  31183d:	00 
  31183e:	74 5a                	je     31189a <<rexx_exec::Interp>::exec_parse+0x247a>
<core::option::Option<alloc::vec::Vec<u8>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311840:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
  311844:	b8 00 00 00 00       	mov    $0x0,%eax
  311849:	4c 0f 44 f8          	cmove  %rax,%r15
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  31184d:	45 84 e4             	test   %r12b,%r12b
  311850:	0f 85 3d ff ff ff    	jne    311793 <<rexx_exec::Interp>::exec_parse+0x2373>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:778
  311856:	48 c7 04 24 00 00 00 	movq   $0x0,(%rsp)
  31185d:	00 
  31185e:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  311863:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311868:	4c 89 e6             	mov    %r12,%rsi
  31186b:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  311872:	00 
  311873:	4c 89 e9             	mov    %r13,%rcx
  311876:	49 89 e8             	mov    %rbp,%r8
  311879:	4c 8b 8c 24 c0 00 00 	mov    0xc0(%rsp),%r9
  311880:	00 
  311881:	e8 7a 84 03 00       	call   349d00 <<rexx_exec::Interp>::assign_expr_target>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  311886:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
  31188b:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  31188f:	0f 84 a9 00 00 00    	je     31193e <<rexx_exec::Interp>::exec_parse+0x251e>
  311895:	e9 cc 0e 00 00       	jmp    312766 <<rexx_exec::Interp>::exec_parse+0x3346>
<rexx_exec::Interp>::assign_targets:
  31189a:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  31189d:	45 84 e4             	test   %r12b,%r12b
  3118a0:	0f 85 ed fe ff ff    	jne    311793 <<rexx_exec::Interp>::exec_parse+0x2373>
  3118a6:	eb ae                	jmp    311856 <<rexx_exec::Interp>::exec_parse+0x2436>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:300
  3118a8:	49 3b 94 24 80 0b 00 	cmp    0xb80(%r12),%rdx
  3118af:	00 
  3118b0:	0f 85 af 0b 00 00    	jne    312465 <<rexx_exec::Interp>::exec_parse+0x3045>
  3118b6:	49 8b 84 24 78 0b 00 	mov    0xb78(%r12),%rax
  3118bd:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:302
  3118be:	48 03 84 24 a0 01 00 	add    0x1a0(%rsp),%rax
  3118c5:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  3118c6:	49 3b 84 24 08 0b 00 	cmp    0xb08(%r12),%rax
  3118cd:	00 
  3118ce:	0f 83 fc 16 00 00    	jae    312fd0 <<rexx_exec::Interp>::exec_parse+0x3bb0>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3118d4:	49 8b 8c 24 00 0b 00 	mov    0xb00(%r12),%rcx
  3118db:	00 
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:245
  3118dc:	48 c1 e0 04          	shl    $0x4,%rax
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:308
  3118e0:	48 c7 04 01 01 00 00 	movq   $0x1,(%rcx,%rax,1)
  3118e7:	00 
  3118e8:	48 89 6c 01 08       	mov    %rbp,0x8(%rcx,%rax,1)
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:77
  3118ed:	4d 85 ff             	test   %r15,%r15
  3118f0:	74 4c                	je     31193e <<rexx_exec::Interp>::exec_parse+0x251e>
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  3118f2:	8b bc 24 bc 01 00 00 	mov    0x1bc(%rsp),%edi
  3118f9:	48 8b 84 24 00 02 00 	mov    0x200(%rsp),%rax
  311900:	00 
<alloc::vec::Vec<alloc::boxed::Box<str>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311901:	48 8b 70 10          	mov    0x10(%rax),%rsi
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  311905:	48 39 fe             	cmp    %rdi,%rsi
  311908:	0f 86 f8 18 00 00    	jbe    313206 <<rexx_exec::Interp>::exec_parse+0x3de6>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::boxed::Box<str>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  31190e:	48 8b 40 08          	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  311912:	48 c1 e7 04          	shl    $0x4,%rdi
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  311916:	48 8b 14 38          	mov    (%rax,%rdi,1),%rdx
  31191a:	48 8b 4c 38 08       	mov    0x8(%rax,%rdi,1),%rcx
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:78
  31191f:	48 89 2c 24          	mov    %rbp,(%rsp)
  311923:	4c 89 e7             	mov    %r12,%rdi
  311926:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  31192d:	00 
  31192e:	4d 89 f8             	mov    %r15,%r8
  311931:	4c 8b 8c 24 e8 01 00 	mov    0x1e8(%rsp),%r9
  311938:	00 
  311939:	e8 82 81 03 00       	call   349ac0 <<rexx_exec::Interp>::trace_assignment>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:783
  31193e:	f6 84 24 be 00 00 00 	testb  $0x1,0xbe(%rsp)
  311945:	01 
  311946:	74 18                	je     311960 <<rexx_exec::Interp>::exec_parse+0x2540>
  311948:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:789
  31194c:	75 52                	jne    3119a0 <<rexx_exec::Interp>::exec_parse+0x2580>
  31194e:	e9 8d f8 ff ff       	jmp    3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
  311953:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  31195a:	84 00 00 00 00 00 
  311960:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
<core::option::Option<alloc::vec::Vec<u8>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311964:	0f 84 85 12 00 00    	je     312bef <<rexx_exec::Interp>::exec_parse+0x37cf>
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31196a:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  311971:	00 00 
  311973:	75 2b                	jne    3119a0 <<rexx_exec::Interp>::exec_parse+0x2580>
  311975:	41 f6 84 24 2d 05 00 	testb  $0x1,0x52d(%r12)
  31197c:	00 01 
  31197e:	74 20                	je     3119a0 <<rexx_exec::Interp>::exec_parse+0x2580>
<rexx_exec::Interp>::trace_result:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:225
  311980:	4c 89 e7             	mov    %r12,%rdi
  311983:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  31198a:	00 
  31198b:	48 8b 94 24 f8 00 00 	mov    0xf8(%rsp),%rdx
  311992:	00 
  311993:	48 8b 8c 24 e8 01 00 	mov    0x1e8(%rsp),%rcx
  31199a:	00 
  31199b:	e8 00 c2 f6 ff       	call   27dba0 <<rexx_exec::Interp>::trace_result_line>
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  3119a0:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  3119a7:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  3119ad:	77 11                	ja     3119c0 <<rexx_exec::Interp>::exec_parse+0x25a0>
  3119af:	4d 85 f6             	test   %r14,%r14
  3119b2:	75 1a                	jne    3119ce <<rexx_exec::Interp>::exec_parse+0x25ae>
  3119b4:	e9 27 f8 ff ff       	jmp    3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
  3119b9:	0f 1f 80 00 00 00 00 	nopl   0x0(%rax)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3119c0:	4d 8b bc 24 e0 0b 00 	mov    0xbe0(%r12),%r15
  3119c7:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3119c8:	49 83 ff 08          	cmp    $0x8,%r15
  3119cc:	72 13                	jb     3119e1 <<rexx_exec::Interp>::exec_parse+0x25c1>
<rexx_exec::Interp>::assign_targets:
  3119ce:	48 8b bc 24 f8 00 00 	mov    0xf8(%rsp),%rdi
  3119d5:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3119d6:	ff 15 2c 62 1c 00    	call   *0x1c622c(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  3119dc:	e9 ff f7 ff ff       	jmp    3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
<rexx_exec::Interp>::assign_targets:
  3119e1:	48 8b 84 24 58 01 00 	mov    0x158(%rsp),%rax
  3119e8:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3119e9:	4c 3b 38             	cmp    (%rax),%r15
  3119ec:	0f 85 b3 f7 ff ff    	jne    3111a5 <<rexx_exec::Interp>::exec_parse+0x1d85>
  3119f2:	48 8b bc 24 58 01 00 	mov    0x158(%rsp),%rdi
  3119f9:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3119fa:	e8 a1 f1 17 00       	call   490ba0 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  3119ff:	e9 a1 f7 ff ff       	jmp    3111a5 <<rexx_exec::Interp>::exec_parse+0x1d85>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  311a04:	0f b6 42 11          	movzbl 0x11(%rdx),%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  311a08:	48 83 c2 12          	add    $0x12,%rdx
  311a0c:	48 89 d1             	mov    %rdx,%rcx
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311a0f:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311a14:	0f 83 96 00 00 00    	jae    311ab0 <<rexx_exec::Interp>::exec_parse+0x2690>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  311a1a:	49 39 f7             	cmp    %rsi,%r15
  311a1d:	73 31                	jae    311a50 <<rexx_exec::Interp>::exec_parse+0x2630>
  311a1f:	49 39 c7             	cmp    %rax,%r15
  311a22:	48 89 c7             	mov    %rax,%rdi
  311a25:	49 0f 47 ff          	cmova  %r15,%rdi
  311a29:	eb 0d                	jmp    311a38 <<rexx_exec::Interp>::exec_parse+0x2618>
  311a2b:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:257
  311a30:	49 ff c7             	inc    %r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  311a33:	4c 39 fe             	cmp    %r15,%rsi
  311a36:	74 6e                	je     311aa6 <<rexx_exec::Interp>::exec_parse+0x2686>
  311a38:	4c 39 ff             	cmp    %r15,%rdi
  311a3b:	0f 84 19 17 00 00    	je     31315a <<rexx_exec::Interp>::exec_parse+0x3d3a>
  311a41:	42 0f b6 14 39       	movzbl (%rcx,%r15,1),%edx
  311a46:	83 fa 20             	cmp    $0x20,%edx
rexx_exec::parse_template::is_blank:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:31
  311a49:	74 e5                	je     311a30 <<rexx_exec::Interp>::exec_parse+0x2610>
  311a4b:	83 fa 09             	cmp    $0x9,%edx
  311a4e:	74 e0                	je     311a30 <<rexx_exec::Interp>::exec_parse+0x2610>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311a50:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311a55:	73 59                	jae    311ab0 <<rexx_exec::Interp>::exec_parse+0x2690>
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2489
  311a57:	49 39 c7             	cmp    %rax,%r15
  311a5a:	48 89 c7             	mov    %rax,%rdi
  311a5d:	49 0f 47 ff          	cmova  %r15,%rdi
  311a61:	4c 89 fb             	mov    %r15,%rbx
<usize as core::cmp::PartialOrd>::lt:
  311a64:	66 66 66 2e 0f 1f 84 	data16 data16 cs nopw 0x0(%rax,%rax,1)
  311a6b:	00 00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2144
  311a70:	48 39 5c 24 20       	cmp    %rbx,0x20(%rsp)
<core::ops::range::Range<usize> as core::iter::range::RangeIteratorImpl>::spec_next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1100
  311a75:	0f 84 4d 04 00 00    	je     311ec8 <<rexx_exec::Interp>::exec_parse+0x2aa8>
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  311a7b:	48 39 df             	cmp    %rbx,%rdi
  311a7e:	0f 84 c7 16 00 00    	je     31314b <<rexx_exec::Interp>::exec_parse+0x3d2b>
  311a84:	0f b6 14 19          	movzbl (%rcx,%rbx,1),%edx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  311a88:	48 ff c3             	inc    %rbx
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  311a8b:	83 fa 09             	cmp    $0x9,%edx
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  311a8e:	74 05                	je     311a95 <<rexx_exec::Interp>::exec_parse+0x2675>
  311a90:	83 fa 20             	cmp    $0x20,%edx
  311a93:	75 db                	jne    311a70 <<rexx_exec::Interp>::exec_parse+0x2650>
  311a95:	4c 8d 43 ff          	lea    -0x1(%rbx),%r8
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  311a99:	f6 44 24 37 01       	testb  $0x1,0x37(%rsp)
  311a9e:	0f 84 3c f7 ff ff    	je     3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
  311aa4:	eb 1e                	jmp    311ac4 <<rexx_exec::Interp>::exec_parse+0x26a4>
  311aa6:	49 89 f7             	mov    %rsi,%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311aa9:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311aae:	72 a7                	jb     311a57 <<rexx_exec::Interp>::exec_parse+0x2637>
<rexx_exec::Interp>::assign_targets:
  311ab0:	4c 89 fb             	mov    %r15,%rbx
  311ab3:	45 31 ff             	xor    %r15d,%r15d
  311ab6:	45 31 c0             	xor    %r8d,%r8d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  311ab9:	f6 44 24 37 01       	testb  $0x1,0x37(%rsp)
  311abe:	0f 84 1c f7 ff ff    	je     3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:804
  311ac4:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  311ac9:	4c 89 e6             	mov    %r12,%rsi
  311acc:	48 8d 94 24 d8 00 00 	lea    0xd8(%rsp),%rdx
  311ad3:	00 
  311ad4:	4c 89 f9             	mov    %r15,%rcx
  311ad7:	e8 a4 53 04 00       	call   356e80 <<rexx_exec::Interp>::copied_piece>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311adc:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  311ae1:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  311ae6:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  311aed:	00 00 
  311aef:	75 6f                	jne    311b60 <<rexx_exec::Interp>::exec_parse+0x2740>
  311af1:	41 f6 84 24 2e 05 00 	testb  $0x1,0x52e(%r12)
  311af8:	00 01 
  311afa:	74 64                	je     311b60 <<rexx_exec::Interp>::exec_parse+0x2740>
<rexx_exec::Interp>::trace_dummy:
  311afc:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  311b01:	49 8b ac 24 e8 10 00 	mov    0x10e8(%r12),%rbp
  311b08:	00 
  311b09:	48 8b bc 24 e0 01 00 	mov    0x1e0(%rsp),%rdi
  311b10:	00 
rexx_exec::trace::format::push_value:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:559
  311b11:	48 8d 35 67 90 d5 ff 	lea    -0x2a6f99(%rip),%rsi        # 6ab7f <panic_unwind::imp::CANARY+0x5e1a>
  311b18:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  311b1f:	00 
  311b20:	e8 3b d4 f6 ff       	call   27ef60 <rexx_exec::trace::format::push_prefixed_blanks>
  311b25:	48 8b bc 24 e0 01 00 	mov    0x1e0(%rsp),%rdi
  311b2c:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:560
  311b2d:	4c 89 fe             	mov    %r15,%rsi
  311b30:	4c 89 f2             	mov    %r14,%rdx
  311b33:	e8 58 d5 f6 ff       	call   27f090 <rexx_exec::trace::format::push_quoted>
<alloc::vec::Vec<u8>>::as_mut_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1904
  311b38:	49 8b 94 24 e8 10 00 	mov    0x10e8(%r12),%rdx
  311b3f:	00 
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:557
  311b40:	48 39 ea             	cmp    %rbp,%rdx
  311b43:	0f 82 ea 14 00 00    	jb     313033 <<rexx_exec::Interp>::exec_parse+0x3c13>
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  311b49:	48 89 d6             	mov    %rdx,%rsi
  311b4c:	48 29 ee             	sub    %rbp,%rsi
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  311b4f:	0f 85 9f 00 00 00    	jne    311bf4 <<rexx_exec::Interp>::exec_parse+0x27d4>
<rexx_exec::Interp>::trace_dummy:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:576
  311b55:	4c 89 e7             	mov    %r12,%rdi
  311b58:	48 89 ee             	mov    %rbp,%rsi
  311b5b:	e8 d0 c4 e8 ff       	call   19e030 <<rexx_exec::Interp>::route_trace_line>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:806
  311b60:	4c 8b 74 24 40       	mov    0x40(%rsp),%r14
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  311b65:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  311b6c:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  311b72:	77 11                	ja     311b85 <<rexx_exec::Interp>::exec_parse+0x2765>
  311b74:	4d 85 f6             	test   %r14,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  311b77:	0f 84 63 f6 ff ff    	je     3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  311b7d:	4c 89 ff             	mov    %r15,%rdi
  311b80:	e9 51 fe ff ff       	jmp    3119d6 <<rexx_exec::Interp>::exec_parse+0x25b6>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  311b85:	4d 8b a4 24 e0 0b 00 	mov    0xbe0(%r12),%r12
  311b8c:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  311b8d:	49 83 fc 08          	cmp    $0x8,%r12
  311b91:	72 13                	jb     311ba6 <<rexx_exec::Interp>::exec_parse+0x2786>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  311b93:	4c 89 ff             	mov    %r15,%rdi
  311b96:	ff 15 6c 60 1c 00    	call   *0x1c606c(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  311b9c:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  311ba1:	e9 3a f6 ff ff       	jmp    3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
  311ba6:	48 8b 84 24 58 01 00 	mov    0x158(%rsp),%rax
  311bad:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311bae:	4c 3b 20             	cmp    (%rax),%r12
  311bb1:	75 0d                	jne    311bc0 <<rexx_exec::Interp>::exec_parse+0x27a0>
  311bb3:	48 8b bc 24 58 01 00 	mov    0x158(%rsp),%rdi
  311bba:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  311bbb:	e8 e0 ef 17 00       	call   490ba0 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  311bc0:	48 8b 54 24 18       	mov    0x18(%rsp),%rdx
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311bc5:	48 8b 82 d8 0b 00 00 	mov    0xbd8(%rdx),%rax
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  311bcc:	4b 8d 0c 64          	lea    (%r12,%r12,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  311bd0:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  311bd4:	4c 89 7c c8 08       	mov    %r15,0x8(%rax,%rcx,8)
  311bd9:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  311be0:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  311be2:	49 ff c4             	inc    %r12
  311be5:	4c 89 a2 e0 0b 00 00 	mov    %r12,0xbe0(%rdx)
  311bec:	49 89 d4             	mov    %rdx,%r12
  311bef:	e9 ec f5 ff ff       	jmp    3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
rexx_exec::trace::format::push_value:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:561
  311bf4:	49 8b 84 24 e0 10 00 	mov    0x10e0(%r12),%rax
  311bfb:	00 
core::slice::index::get_offset_len_mut_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:102
  311bfc:	48 8d 3c 28          	lea    (%rax,%rbp,1),%rdi
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  311c00:	48 83 fe 10          	cmp    $0x10,%rsi
  311c04:	73 1c                	jae    311c22 <<rexx_exec::Interp>::exec_parse+0x2802>
  311c06:	48 89 f9             	mov    %rdi,%rcx
  311c09:	e9 1e 08 00 00       	jmp    31242c <<rexx_exec::Interp>::exec_parse+0x300c>
<rexx_exec::Interp>::assign_targets:
  311c0e:	4c 89 f3             	mov    %r14,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311c11:	4d 89 f4             	mov    %r14,%r12
  311c14:	4d 29 fc             	sub    %r15,%r12
  311c17:	0f 83 60 f6 ff ff    	jae    31127d <<rexx_exec::Interp>::exec_parse+0x1e5d>
  311c1d:	e9 34 0e 00 00       	jmp    312a56 <<rexx_exec::Interp>::exec_parse+0x3636>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  311c22:	48 83 fe 20          	cmp    $0x20,%rsi
  311c26:	0f 83 c2 02 00 00    	jae    311eee <<rexx_exec::Interp>::exec_parse+0x2ace>
  311c2c:	45 31 c0             	xor    %r8d,%r8d
  311c2f:	49 89 f1             	mov    %rsi,%r9
  311c32:	49 83 e1 f0          	and    $0xfffffffffffffff0,%r9
  311c36:	4a 8d 0c 0f          	lea    (%rdi,%r9,1),%rcx
  311c3a:	eb 11                	jmp    311c4d <<rexx_exec::Interp>::exec_parse+0x282d>
  311c3c:	0f 1f 40 00          	nopl   0x0(%rax)
  311c40:	49 83 c0 10          	add    $0x10,%r8
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  311c44:	4d 39 c1             	cmp    %r8,%r9
  311c47:	0f 84 93 02 00 00    	je     311ee0 <<rexx_exec::Interp>::exec_parse+0x2ac0>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  311c4d:	f3 42 0f 6f 0c 07    	movdqu (%rdi,%r8,1),%xmm1
  311c53:	66 0f 6f d1          	movdqa %xmm1,%xmm2
  311c57:	66 0f da 15 f1 e7 d3 	pminub -0x2c180f(%rip),%xmm2        # 50450 <GCC_except_table5013+0xa88>
  311c5e:	ff 
  311c5f:	66 0f 74 d1          	pcmpeqb %xmm1,%xmm2
  311c63:	66 0f 6f c1          	movdqa %xmm1,%xmm0
  311c67:	66 0f 74 05 41 e4 d3 	pcmpeqb -0x2c1bbf(%rip),%xmm0        # 500b0 <GCC_except_table5013+0x6e8>
  311c6e:	ff 
  311c6f:	66 0f fc 0d e9 e6 d3 	paddb  -0x2c1917(%rip),%xmm1        # 50360 <GCC_except_table5013+0x998>
  311c76:	ff 
  311c77:	66 0f 6f d9          	movdqa %xmm1,%xmm3
  311c7b:	66 0f da 1d 1d e3 d3 	pminub -0x2c1ce3(%rip),%xmm3        # 4ffa0 <GCC_except_table5013+0x5d8>
  311c82:	ff 
  311c83:	66 0f 74 d9          	pcmpeqb %xmm1,%xmm3
  311c87:	66 0f df c3          	pandn  %xmm3,%xmm0
  311c8b:	66 0f db c2          	pand   %xmm2,%xmm0
  311c8f:	66 0f 7f 84 24 10 02 	movdqa %xmm0,0x210(%rsp)
  311c96:	00 00 
  311c98:	f6 84 24 10 02 00 00 	testb  $0x1,0x210(%rsp)
  311c9f:	01 
  311ca0:	74 05                	je     311ca7 <<rexx_exec::Interp>::exec_parse+0x2887>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  311ca2:	42 c6 04 07 3f       	movb   $0x3f,(%rdi,%r8,1)
  311ca7:	66 41 0f 7e c2       	movd   %xmm0,%r10d
  311cac:	45 89 d3             	mov    %r10d,%r11d
  311caf:	41 c1 eb 08          	shr    $0x8,%r11d
  311cb3:	41 f6 c3 01          	test   $0x1,%r11b
  311cb7:	0f 84 03 01 00 00    	je     311dc0 <<rexx_exec::Interp>::exec_parse+0x29a0>
  311cbd:	42 c6 44 07 01 3f    	movb   $0x3f,0x1(%rdi,%r8,1)
  311cc3:	45 89 d3             	mov    %r10d,%r11d
  311cc6:	41 c1 eb 10          	shr    $0x10,%r11d
  311cca:	41 f6 c3 01          	test   $0x1,%r11b
  311cce:	0f 85 fd 00 00 00    	jne    311dd1 <<rexx_exec::Interp>::exec_parse+0x29b1>
  311cd4:	41 c1 ea 18          	shr    $0x18,%r10d
  311cd8:	41 f6 c2 01          	test   $0x1,%r10b
  311cdc:	0f 84 03 01 00 00    	je     311de5 <<rexx_exec::Interp>::exec_parse+0x29c5>
  311ce2:	42 c6 44 07 03 3f    	movb   $0x3f,0x3(%rdi,%r8,1)
  311ce8:	66 44 0f c5 d0 02    	pextrw $0x2,%xmm0,%r10d
  311cee:	41 f6 c2 01          	test   $0x1,%r10b
  311cf2:	0f 85 fd 00 00 00    	jne    311df5 <<rexx_exec::Interp>::exec_parse+0x29d5>
  311cf8:	41 c1 ea 08          	shr    $0x8,%r10d
  311cfc:	41 f6 c2 01          	test   $0x1,%r10b
  311d00:	0f 84 03 01 00 00    	je     311e09 <<rexx_exec::Interp>::exec_parse+0x29e9>
  311d06:	42 c6 44 07 05 3f    	movb   $0x3f,0x5(%rdi,%r8,1)
  311d0c:	66 44 0f c5 d0 03    	pextrw $0x3,%xmm0,%r10d
  311d12:	41 f6 c2 01          	test   $0x1,%r10b
  311d16:	0f 85 fd 00 00 00    	jne    311e19 <<rexx_exec::Interp>::exec_parse+0x29f9>
  311d1c:	41 c1 ea 08          	shr    $0x8,%r10d
  311d20:	41 f6 c2 01          	test   $0x1,%r10b
  311d24:	0f 84 03 01 00 00    	je     311e2d <<rexx_exec::Interp>::exec_parse+0x2a0d>
  311d2a:	42 c6 44 07 07 3f    	movb   $0x3f,0x7(%rdi,%r8,1)
  311d30:	66 44 0f c5 d0 04    	pextrw $0x4,%xmm0,%r10d
  311d36:	41 f6 c2 01          	test   $0x1,%r10b
  311d3a:	0f 85 fd 00 00 00    	jne    311e3d <<rexx_exec::Interp>::exec_parse+0x2a1d>
  311d40:	41 c1 ea 08          	shr    $0x8,%r10d
  311d44:	41 f6 c2 01          	test   $0x1,%r10b
  311d48:	0f 84 03 01 00 00    	je     311e51 <<rexx_exec::Interp>::exec_parse+0x2a31>
  311d4e:	42 c6 44 07 09 3f    	movb   $0x3f,0x9(%rdi,%r8,1)
  311d54:	66 44 0f c5 d0 05    	pextrw $0x5,%xmm0,%r10d
  311d5a:	41 f6 c2 01          	test   $0x1,%r10b
  311d5e:	0f 85 fd 00 00 00    	jne    311e61 <<rexx_exec::Interp>::exec_parse+0x2a41>
  311d64:	41 c1 ea 08          	shr    $0x8,%r10d
  311d68:	41 f6 c2 01          	test   $0x1,%r10b
  311d6c:	0f 84 03 01 00 00    	je     311e75 <<rexx_exec::Interp>::exec_parse+0x2a55>
  311d72:	42 c6 44 07 0b 3f    	movb   $0x3f,0xb(%rdi,%r8,1)
  311d78:	66 44 0f c5 d0 06    	pextrw $0x6,%xmm0,%r10d
  311d7e:	41 f6 c2 01          	test   $0x1,%r10b
  311d82:	0f 85 fd 00 00 00    	jne    311e85 <<rexx_exec::Interp>::exec_parse+0x2a65>
  311d88:	41 c1 ea 08          	shr    $0x8,%r10d
  311d8c:	41 f6 c2 01          	test   $0x1,%r10b
  311d90:	0f 84 03 01 00 00    	je     311e99 <<rexx_exec::Interp>::exec_parse+0x2a79>
  311d96:	42 c6 44 07 0d 3f    	movb   $0x3f,0xd(%rdi,%r8,1)
  311d9c:	66 44 0f c5 d0 07    	pextrw $0x7,%xmm0,%r10d
  311da2:	41 f6 c2 01          	test   $0x1,%r10b
  311da6:	0f 85 fd 00 00 00    	jne    311ea9 <<rexx_exec::Interp>::exec_parse+0x2a89>
  311dac:	41 c1 ea 08          	shr    $0x8,%r10d
  311db0:	41 f6 c2 01          	test   $0x1,%r10b
  311db4:	0f 84 86 fe ff ff    	je     311c40 <<rexx_exec::Interp>::exec_parse+0x2820>
  311dba:	e9 fe 00 00 00       	jmp    311ebd <<rexx_exec::Interp>::exec_parse+0x2a9d>
  311dbf:	90                   	nop
  311dc0:	45 89 d3             	mov    %r10d,%r11d
  311dc3:	41 c1 eb 10          	shr    $0x10,%r11d
  311dc7:	41 f6 c3 01          	test   $0x1,%r11b
  311dcb:	0f 84 03 ff ff ff    	je     311cd4 <<rexx_exec::Interp>::exec_parse+0x28b4>
  311dd1:	42 c6 44 07 02 3f    	movb   $0x3f,0x2(%rdi,%r8,1)
  311dd7:	41 c1 ea 18          	shr    $0x18,%r10d
  311ddb:	41 f6 c2 01          	test   $0x1,%r10b
  311ddf:	0f 85 fd fe ff ff    	jne    311ce2 <<rexx_exec::Interp>::exec_parse+0x28c2>
  311de5:	66 44 0f c5 d0 02    	pextrw $0x2,%xmm0,%r10d
  311deb:	41 f6 c2 01          	test   $0x1,%r10b
  311def:	0f 84 03 ff ff ff    	je     311cf8 <<rexx_exec::Interp>::exec_parse+0x28d8>
  311df5:	42 c6 44 07 04 3f    	movb   $0x3f,0x4(%rdi,%r8,1)
  311dfb:	41 c1 ea 08          	shr    $0x8,%r10d
  311dff:	41 f6 c2 01          	test   $0x1,%r10b
  311e03:	0f 85 fd fe ff ff    	jne    311d06 <<rexx_exec::Interp>::exec_parse+0x28e6>
  311e09:	66 44 0f c5 d0 03    	pextrw $0x3,%xmm0,%r10d
  311e0f:	41 f6 c2 01          	test   $0x1,%r10b
  311e13:	0f 84 03 ff ff ff    	je     311d1c <<rexx_exec::Interp>::exec_parse+0x28fc>
  311e19:	42 c6 44 07 06 3f    	movb   $0x3f,0x6(%rdi,%r8,1)
  311e1f:	41 c1 ea 08          	shr    $0x8,%r10d
  311e23:	41 f6 c2 01          	test   $0x1,%r10b
  311e27:	0f 85 fd fe ff ff    	jne    311d2a <<rexx_exec::Interp>::exec_parse+0x290a>
  311e2d:	66 44 0f c5 d0 04    	pextrw $0x4,%xmm0,%r10d
  311e33:	41 f6 c2 01          	test   $0x1,%r10b
  311e37:	0f 84 03 ff ff ff    	je     311d40 <<rexx_exec::Interp>::exec_parse+0x2920>
  311e3d:	42 c6 44 07 08 3f    	movb   $0x3f,0x8(%rdi,%r8,1)
  311e43:	41 c1 ea 08          	shr    $0x8,%r10d
  311e47:	41 f6 c2 01          	test   $0x1,%r10b
  311e4b:	0f 85 fd fe ff ff    	jne    311d4e <<rexx_exec::Interp>::exec_parse+0x292e>
  311e51:	66 44 0f c5 d0 05    	pextrw $0x5,%xmm0,%r10d
  311e57:	41 f6 c2 01          	test   $0x1,%r10b
  311e5b:	0f 84 03 ff ff ff    	je     311d64 <<rexx_exec::Interp>::exec_parse+0x2944>
  311e61:	42 c6 44 07 0a 3f    	movb   $0x3f,0xa(%rdi,%r8,1)
  311e67:	41 c1 ea 08          	shr    $0x8,%r10d
  311e6b:	41 f6 c2 01          	test   $0x1,%r10b
  311e6f:	0f 85 fd fe ff ff    	jne    311d72 <<rexx_exec::Interp>::exec_parse+0x2952>
  311e75:	66 44 0f c5 d0 06    	pextrw $0x6,%xmm0,%r10d
  311e7b:	41 f6 c2 01          	test   $0x1,%r10b
  311e7f:	0f 84 03 ff ff ff    	je     311d88 <<rexx_exec::Interp>::exec_parse+0x2968>
  311e85:	42 c6 44 07 0c 3f    	movb   $0x3f,0xc(%rdi,%r8,1)
  311e8b:	41 c1 ea 08          	shr    $0x8,%r10d
  311e8f:	41 f6 c2 01          	test   $0x1,%r10b
  311e93:	0f 85 fd fe ff ff    	jne    311d96 <<rexx_exec::Interp>::exec_parse+0x2976>
  311e99:	66 44 0f c5 d0 07    	pextrw $0x7,%xmm0,%r10d
  311e9f:	41 f6 c2 01          	test   $0x1,%r10b
  311ea3:	0f 84 03 ff ff ff    	je     311dac <<rexx_exec::Interp>::exec_parse+0x298c>
  311ea9:	42 c6 44 07 0e 3f    	movb   $0x3f,0xe(%rdi,%r8,1)
  311eaf:	41 c1 ea 08          	shr    $0x8,%r10d
  311eb3:	41 f6 c2 01          	test   $0x1,%r10b
  311eb7:	0f 84 83 fd ff ff    	je     311c40 <<rexx_exec::Interp>::exec_parse+0x2820>
  311ebd:	42 c6 44 07 0f 3f    	movb   $0x3f,0xf(%rdi,%r8,1)
  311ec3:	e9 78 fd ff ff       	jmp    311c40 <<rexx_exec::Interp>::exec_parse+0x2820>
<rexx_exec::Interp>::assign_targets:
  311ec8:	4c 8b 44 24 20       	mov    0x20(%rsp),%r8
  311ecd:	4c 89 c3             	mov    %r8,%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  311ed0:	f6 44 24 37 01       	testb  $0x1,0x37(%rsp)
  311ed5:	0f 84 05 f3 ff ff    	je     3111e0 <<rexx_exec::Interp>::exec_parse+0x1dc0>
  311edb:	e9 e4 fb ff ff       	jmp    311ac4 <<rexx_exec::Interp>::exec_parse+0x26a4>
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  311ee0:	4c 39 ce             	cmp    %r9,%rsi
  311ee3:	0f 84 6c fc ff ff    	je     311b55 <<rexx_exec::Interp>::exec_parse+0x2735>
  311ee9:	e9 3e 05 00 00       	jmp    31242c <<rexx_exec::Interp>::exec_parse+0x300c>
  311eee:	49 89 f0             	mov    %rsi,%r8
  311ef1:	49 83 e0 e0          	and    $0xffffffffffffffe0,%r8
  311ef5:	4a 8d 0c 07          	lea    (%rdi,%r8,1),%rcx
  311ef9:	45 31 c9             	xor    %r9d,%r9d
  311efc:	eb 0f                	jmp    311f0d <<rexx_exec::Interp>::exec_parse+0x2aed>
  311efe:	66 90                	xchg   %ax,%ax
  311f00:	49 83 c1 20          	add    $0x20,%r9
  311f04:	4d 39 c8             	cmp    %r9,%r8
  311f07:	0f 84 0c 05 00 00    	je     312419 <<rexx_exec::Interp>::exec_parse+0x2ff9>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  311f0d:	f3 42 0f 6f 14 0f    	movdqu (%rdi,%r9,1),%xmm2
  311f13:	f3 42 0f 6f 44 0f 10 	movdqu 0x10(%rdi,%r9,1),%xmm0
  311f1a:	66 0f 6f da          	movdqa %xmm2,%xmm3
  311f1e:	66 0f da 1d 2a e5 d3 	pminub -0x2c1ad6(%rip),%xmm3        # 50450 <GCC_except_table5013+0xa88>
  311f25:	ff 
  311f26:	66 0f 74 da          	pcmpeqb %xmm2,%xmm3
  311f2a:	66 0f 6f ca          	movdqa %xmm2,%xmm1
  311f2e:	66 0f 74 0d 7a e1 d3 	pcmpeqb -0x2c1e86(%rip),%xmm1        # 500b0 <GCC_except_table5013+0x6e8>
  311f35:	ff 
  311f36:	66 0f fc 15 22 e4 d3 	paddb  -0x2c1bde(%rip),%xmm2        # 50360 <GCC_except_table5013+0x998>
  311f3d:	ff 
  311f3e:	66 0f 6f e2          	movdqa %xmm2,%xmm4
  311f42:	66 0f da 25 56 e0 d3 	pminub -0x2c1faa(%rip),%xmm4        # 4ffa0 <GCC_except_table5013+0x5d8>
  311f49:	ff 
  311f4a:	66 0f 74 e2          	pcmpeqb %xmm2,%xmm4
  311f4e:	66 0f df cc          	pandn  %xmm4,%xmm1
  311f52:	66 0f db cb          	pand   %xmm3,%xmm1
  311f56:	66 0f 7f 8c 24 20 02 	movdqa %xmm1,0x220(%rsp)
  311f5d:	00 00 
  311f5f:	f6 84 24 20 02 00 00 	testb  $0x1,0x220(%rsp)
  311f66:	01 
  311f67:	74 05                	je     311f6e <<rexx_exec::Interp>::exec_parse+0x2b4e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  311f69:	42 c6 04 0f 3f       	movb   $0x3f,(%rdi,%r9,1)
  311f6e:	66 41 0f 7e ca       	movd   %xmm1,%r10d
  311f73:	45 89 d3             	mov    %r10d,%r11d
  311f76:	41 c1 eb 08          	shr    $0x8,%r11d
  311f7a:	41 f6 c3 01          	test   $0x1,%r11b
  311f7e:	0f 84 6c 02 00 00    	je     3121f0 <<rexx_exec::Interp>::exec_parse+0x2dd0>
  311f84:	42 c6 44 0f 01 3f    	movb   $0x3f,0x1(%rdi,%r9,1)
  311f8a:	45 89 d3             	mov    %r10d,%r11d
  311f8d:	41 c1 eb 10          	shr    $0x10,%r11d
  311f91:	41 f6 c3 01          	test   $0x1,%r11b
  311f95:	0f 85 66 02 00 00    	jne    312201 <<rexx_exec::Interp>::exec_parse+0x2de1>
  311f9b:	41 c1 ea 18          	shr    $0x18,%r10d
  311f9f:	41 f6 c2 01          	test   $0x1,%r10b
  311fa3:	0f 84 6c 02 00 00    	je     312215 <<rexx_exec::Interp>::exec_parse+0x2df5>
  311fa9:	42 c6 44 0f 03 3f    	movb   $0x3f,0x3(%rdi,%r9,1)
  311faf:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  311fb5:	41 f6 c2 01          	test   $0x1,%r10b
  311fb9:	0f 85 66 02 00 00    	jne    312225 <<rexx_exec::Interp>::exec_parse+0x2e05>
  311fbf:	41 c1 ea 08          	shr    $0x8,%r10d
  311fc3:	41 f6 c2 01          	test   $0x1,%r10b
  311fc7:	0f 84 6c 02 00 00    	je     312239 <<rexx_exec::Interp>::exec_parse+0x2e19>
  311fcd:	42 c6 44 0f 05 3f    	movb   $0x3f,0x5(%rdi,%r9,1)
  311fd3:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  311fd9:	41 f6 c2 01          	test   $0x1,%r10b
  311fdd:	0f 85 66 02 00 00    	jne    312249 <<rexx_exec::Interp>::exec_parse+0x2e29>
  311fe3:	41 c1 ea 08          	shr    $0x8,%r10d
  311fe7:	41 f6 c2 01          	test   $0x1,%r10b
  311feb:	0f 84 6c 02 00 00    	je     31225d <<rexx_exec::Interp>::exec_parse+0x2e3d>
  311ff1:	42 c6 44 0f 07 3f    	movb   $0x3f,0x7(%rdi,%r9,1)
  311ff7:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  311ffd:	41 f6 c2 01          	test   $0x1,%r10b
  312001:	0f 85 66 02 00 00    	jne    31226d <<rexx_exec::Interp>::exec_parse+0x2e4d>
  312007:	41 c1 ea 08          	shr    $0x8,%r10d
  31200b:	41 f6 c2 01          	test   $0x1,%r10b
  31200f:	0f 84 6c 02 00 00    	je     312281 <<rexx_exec::Interp>::exec_parse+0x2e61>
  312015:	42 c6 44 0f 09 3f    	movb   $0x3f,0x9(%rdi,%r9,1)
  31201b:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  312021:	41 f6 c2 01          	test   $0x1,%r10b
  312025:	0f 85 66 02 00 00    	jne    312291 <<rexx_exec::Interp>::exec_parse+0x2e71>
  31202b:	41 c1 ea 08          	shr    $0x8,%r10d
  31202f:	41 f6 c2 01          	test   $0x1,%r10b
  312033:	0f 84 6c 02 00 00    	je     3122a5 <<rexx_exec::Interp>::exec_parse+0x2e85>
  312039:	42 c6 44 0f 0b 3f    	movb   $0x3f,0xb(%rdi,%r9,1)
  31203f:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  312045:	41 f6 c2 01          	test   $0x1,%r10b
  312049:	0f 85 66 02 00 00    	jne    3122b5 <<rexx_exec::Interp>::exec_parse+0x2e95>
  31204f:	41 c1 ea 08          	shr    $0x8,%r10d
  312053:	41 f6 c2 01          	test   $0x1,%r10b
  312057:	0f 84 6c 02 00 00    	je     3122c9 <<rexx_exec::Interp>::exec_parse+0x2ea9>
  31205d:	42 c6 44 0f 0d 3f    	movb   $0x3f,0xd(%rdi,%r9,1)
  312063:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  312069:	41 f6 c2 01          	test   $0x1,%r10b
  31206d:	0f 85 66 02 00 00    	jne    3122d9 <<rexx_exec::Interp>::exec_parse+0x2eb9>
  312073:	41 c1 ea 08          	shr    $0x8,%r10d
  312077:	41 f6 c2 01          	test   $0x1,%r10b
  31207b:	74 06                	je     312083 <<rexx_exec::Interp>::exec_parse+0x2c63>
  31207d:	42 c6 44 0f 0f 3f    	movb   $0x3f,0xf(%rdi,%r9,1)
  312083:	66 0f 6f d0          	movdqa %xmm0,%xmm2
  312087:	66 0f da 15 c1 e3 d3 	pminub -0x2c1c3f(%rip),%xmm2        # 50450 <GCC_except_table5013+0xa88>
  31208e:	ff 
  31208f:	66 0f 74 d0          	pcmpeqb %xmm0,%xmm2
  312093:	66 0f 6f c8          	movdqa %xmm0,%xmm1
  312097:	66 0f 74 0d 11 e0 d3 	pcmpeqb -0x2c1fef(%rip),%xmm1        # 500b0 <GCC_except_table5013+0x6e8>
  31209e:	ff 
  31209f:	66 0f fc 05 b9 e2 d3 	paddb  -0x2c1d47(%rip),%xmm0        # 50360 <GCC_except_table5013+0x998>
  3120a6:	ff 
  3120a7:	66 0f 6f d8          	movdqa %xmm0,%xmm3
  3120ab:	66 0f da 1d ed de d3 	pminub -0x2c2113(%rip),%xmm3        # 4ffa0 <GCC_except_table5013+0x5d8>
  3120b2:	ff 
  3120b3:	66 0f 74 d8          	pcmpeqb %xmm0,%xmm3
  3120b7:	66 0f df cb          	pandn  %xmm3,%xmm1
  3120bb:	66 0f db ca          	pand   %xmm2,%xmm1
  3120bf:	66 41 0f 7e ca       	movd   %xmm1,%r10d
  3120c4:	41 f6 c2 01          	test   $0x1,%r10b
  3120c8:	0f 84 32 02 00 00    	je     312300 <<rexx_exec::Interp>::exec_parse+0x2ee0>
  3120ce:	42 c6 44 0f 10 3f    	movb   $0x3f,0x10(%rdi,%r9,1)
  3120d4:	45 89 d3             	mov    %r10d,%r11d
  3120d7:	41 c1 eb 08          	shr    $0x8,%r11d
  3120db:	41 f6 c3 01          	test   $0x1,%r11b
  3120df:	0f 85 2c 02 00 00    	jne    312311 <<rexx_exec::Interp>::exec_parse+0x2ef1>
  3120e5:	45 89 d3             	mov    %r10d,%r11d
  3120e8:	41 c1 eb 10          	shr    $0x10,%r11d
  3120ec:	41 f6 c3 01          	test   $0x1,%r11b
  3120f0:	0f 84 32 02 00 00    	je     312328 <<rexx_exec::Interp>::exec_parse+0x2f08>
  3120f6:	42 c6 44 0f 12 3f    	movb   $0x3f,0x12(%rdi,%r9,1)
  3120fc:	41 c1 ea 18          	shr    $0x18,%r10d
  312100:	41 f6 c2 01          	test   $0x1,%r10b
  312104:	0f 85 2c 02 00 00    	jne    312336 <<rexx_exec::Interp>::exec_parse+0x2f16>
  31210a:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  312110:	41 f6 c2 01          	test   $0x1,%r10b
  312114:	0f 84 32 02 00 00    	je     31234c <<rexx_exec::Interp>::exec_parse+0x2f2c>
  31211a:	42 c6 44 0f 14 3f    	movb   $0x3f,0x14(%rdi,%r9,1)
  312120:	41 c1 ea 08          	shr    $0x8,%r10d
  312124:	41 f6 c2 01          	test   $0x1,%r10b
  312128:	0f 85 2c 02 00 00    	jne    31235a <<rexx_exec::Interp>::exec_parse+0x2f3a>
  31212e:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  312134:	41 f6 c2 01          	test   $0x1,%r10b
  312138:	0f 84 32 02 00 00    	je     312370 <<rexx_exec::Interp>::exec_parse+0x2f50>
  31213e:	42 c6 44 0f 16 3f    	movb   $0x3f,0x16(%rdi,%r9,1)
  312144:	41 c1 ea 08          	shr    $0x8,%r10d
  312148:	41 f6 c2 01          	test   $0x1,%r10b
  31214c:	0f 85 2c 02 00 00    	jne    31237e <<rexx_exec::Interp>::exec_parse+0x2f5e>
  312152:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  312158:	41 f6 c2 01          	test   $0x1,%r10b
  31215c:	0f 84 32 02 00 00    	je     312394 <<rexx_exec::Interp>::exec_parse+0x2f74>
  312162:	42 c6 44 0f 18 3f    	movb   $0x3f,0x18(%rdi,%r9,1)
  312168:	41 c1 ea 08          	shr    $0x8,%r10d
  31216c:	41 f6 c2 01          	test   $0x1,%r10b
  312170:	0f 85 2c 02 00 00    	jne    3123a2 <<rexx_exec::Interp>::exec_parse+0x2f82>
  312176:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  31217c:	41 f6 c2 01          	test   $0x1,%r10b
  312180:	0f 84 32 02 00 00    	je     3123b8 <<rexx_exec::Interp>::exec_parse+0x2f98>
  312186:	42 c6 44 0f 1a 3f    	movb   $0x3f,0x1a(%rdi,%r9,1)
  31218c:	41 c1 ea 08          	shr    $0x8,%r10d
  312190:	41 f6 c2 01          	test   $0x1,%r10b
  312194:	0f 85 2c 02 00 00    	jne    3123c6 <<rexx_exec::Interp>::exec_parse+0x2fa6>
  31219a:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  3121a0:	41 f6 c2 01          	test   $0x1,%r10b
  3121a4:	0f 84 32 02 00 00    	je     3123dc <<rexx_exec::Interp>::exec_parse+0x2fbc>
  3121aa:	42 c6 44 0f 1c 3f    	movb   $0x3f,0x1c(%rdi,%r9,1)
  3121b0:	41 c1 ea 08          	shr    $0x8,%r10d
  3121b4:	41 f6 c2 01          	test   $0x1,%r10b
  3121b8:	0f 85 2c 02 00 00    	jne    3123ea <<rexx_exec::Interp>::exec_parse+0x2fca>
  3121be:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  3121c4:	41 f6 c2 01          	test   $0x1,%r10b
  3121c8:	0f 84 32 02 00 00    	je     312400 <<rexx_exec::Interp>::exec_parse+0x2fe0>
  3121ce:	42 c6 44 0f 1e 3f    	movb   $0x3f,0x1e(%rdi,%r9,1)
  3121d4:	41 c1 ea 08          	shr    $0x8,%r10d
  3121d8:	41 f6 c2 01          	test   $0x1,%r10b
  3121dc:	0f 84 1e fd ff ff    	je     311f00 <<rexx_exec::Interp>::exec_parse+0x2ae0>
  3121e2:	e9 27 02 00 00       	jmp    31240e <<rexx_exec::Interp>::exec_parse+0x2fee>
  3121e7:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  3121ee:	00 00 
  3121f0:	45 89 d3             	mov    %r10d,%r11d
  3121f3:	41 c1 eb 10          	shr    $0x10,%r11d
  3121f7:	41 f6 c3 01          	test   $0x1,%r11b
  3121fb:	0f 84 9a fd ff ff    	je     311f9b <<rexx_exec::Interp>::exec_parse+0x2b7b>
  312201:	42 c6 44 0f 02 3f    	movb   $0x3f,0x2(%rdi,%r9,1)
  312207:	41 c1 ea 18          	shr    $0x18,%r10d
  31220b:	41 f6 c2 01          	test   $0x1,%r10b
  31220f:	0f 85 94 fd ff ff    	jne    311fa9 <<rexx_exec::Interp>::exec_parse+0x2b89>
  312215:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  31221b:	41 f6 c2 01          	test   $0x1,%r10b
  31221f:	0f 84 9a fd ff ff    	je     311fbf <<rexx_exec::Interp>::exec_parse+0x2b9f>
  312225:	42 c6 44 0f 04 3f    	movb   $0x3f,0x4(%rdi,%r9,1)
  31222b:	41 c1 ea 08          	shr    $0x8,%r10d
  31222f:	41 f6 c2 01          	test   $0x1,%r10b
  312233:	0f 85 94 fd ff ff    	jne    311fcd <<rexx_exec::Interp>::exec_parse+0x2bad>
  312239:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  31223f:	41 f6 c2 01          	test   $0x1,%r10b
  312243:	0f 84 9a fd ff ff    	je     311fe3 <<rexx_exec::Interp>::exec_parse+0x2bc3>
  312249:	42 c6 44 0f 06 3f    	movb   $0x3f,0x6(%rdi,%r9,1)
  31224f:	41 c1 ea 08          	shr    $0x8,%r10d
  312253:	41 f6 c2 01          	test   $0x1,%r10b
  312257:	0f 85 94 fd ff ff    	jne    311ff1 <<rexx_exec::Interp>::exec_parse+0x2bd1>
  31225d:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  312263:	41 f6 c2 01          	test   $0x1,%r10b
  312267:	0f 84 9a fd ff ff    	je     312007 <<rexx_exec::Interp>::exec_parse+0x2be7>
  31226d:	42 c6 44 0f 08 3f    	movb   $0x3f,0x8(%rdi,%r9,1)
  312273:	41 c1 ea 08          	shr    $0x8,%r10d
  312277:	41 f6 c2 01          	test   $0x1,%r10b
  31227b:	0f 85 94 fd ff ff    	jne    312015 <<rexx_exec::Interp>::exec_parse+0x2bf5>
  312281:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  312287:	41 f6 c2 01          	test   $0x1,%r10b
  31228b:	0f 84 9a fd ff ff    	je     31202b <<rexx_exec::Interp>::exec_parse+0x2c0b>
  312291:	42 c6 44 0f 0a 3f    	movb   $0x3f,0xa(%rdi,%r9,1)
  312297:	41 c1 ea 08          	shr    $0x8,%r10d
  31229b:	41 f6 c2 01          	test   $0x1,%r10b
  31229f:	0f 85 94 fd ff ff    	jne    312039 <<rexx_exec::Interp>::exec_parse+0x2c19>
  3122a5:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  3122ab:	41 f6 c2 01          	test   $0x1,%r10b
  3122af:	0f 84 9a fd ff ff    	je     31204f <<rexx_exec::Interp>::exec_parse+0x2c2f>
  3122b5:	42 c6 44 0f 0c 3f    	movb   $0x3f,0xc(%rdi,%r9,1)
  3122bb:	41 c1 ea 08          	shr    $0x8,%r10d
  3122bf:	41 f6 c2 01          	test   $0x1,%r10b
  3122c3:	0f 85 94 fd ff ff    	jne    31205d <<rexx_exec::Interp>::exec_parse+0x2c3d>
  3122c9:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  3122cf:	41 f6 c2 01          	test   $0x1,%r10b
  3122d3:	0f 84 9a fd ff ff    	je     312073 <<rexx_exec::Interp>::exec_parse+0x2c53>
  3122d9:	42 c6 44 0f 0e 3f    	movb   $0x3f,0xe(%rdi,%r9,1)
  3122df:	41 c1 ea 08          	shr    $0x8,%r10d
  3122e3:	41 f6 c2 01          	test   $0x1,%r10b
  3122e7:	0f 85 90 fd ff ff    	jne    31207d <<rexx_exec::Interp>::exec_parse+0x2c5d>
  3122ed:	e9 91 fd ff ff       	jmp    312083 <<rexx_exec::Interp>::exec_parse+0x2c63>
  3122f2:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  3122f9:	1f 84 00 00 00 00 00 
  312300:	45 89 d3             	mov    %r10d,%r11d
  312303:	41 c1 eb 08          	shr    $0x8,%r11d
  312307:	41 f6 c3 01          	test   $0x1,%r11b
  31230b:	0f 84 d4 fd ff ff    	je     3120e5 <<rexx_exec::Interp>::exec_parse+0x2cc5>
  312311:	42 c6 44 0f 11 3f    	movb   $0x3f,0x11(%rdi,%r9,1)
  312317:	45 89 d3             	mov    %r10d,%r11d
  31231a:	41 c1 eb 10          	shr    $0x10,%r11d
  31231e:	41 f6 c3 01          	test   $0x1,%r11b
  312322:	0f 85 ce fd ff ff    	jne    3120f6 <<rexx_exec::Interp>::exec_parse+0x2cd6>
  312328:	41 c1 ea 18          	shr    $0x18,%r10d
  31232c:	41 f6 c2 01          	test   $0x1,%r10b
  312330:	0f 84 d4 fd ff ff    	je     31210a <<rexx_exec::Interp>::exec_parse+0x2cea>
  312336:	42 c6 44 0f 13 3f    	movb   $0x3f,0x13(%rdi,%r9,1)
  31233c:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  312342:	41 f6 c2 01          	test   $0x1,%r10b
  312346:	0f 85 ce fd ff ff    	jne    31211a <<rexx_exec::Interp>::exec_parse+0x2cfa>
  31234c:	41 c1 ea 08          	shr    $0x8,%r10d
  312350:	41 f6 c2 01          	test   $0x1,%r10b
  312354:	0f 84 d4 fd ff ff    	je     31212e <<rexx_exec::Interp>::exec_parse+0x2d0e>
  31235a:	42 c6 44 0f 15 3f    	movb   $0x3f,0x15(%rdi,%r9,1)
  312360:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  312366:	41 f6 c2 01          	test   $0x1,%r10b
  31236a:	0f 85 ce fd ff ff    	jne    31213e <<rexx_exec::Interp>::exec_parse+0x2d1e>
  312370:	41 c1 ea 08          	shr    $0x8,%r10d
  312374:	41 f6 c2 01          	test   $0x1,%r10b
  312378:	0f 84 d4 fd ff ff    	je     312152 <<rexx_exec::Interp>::exec_parse+0x2d32>
  31237e:	42 c6 44 0f 17 3f    	movb   $0x3f,0x17(%rdi,%r9,1)
  312384:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  31238a:	41 f6 c2 01          	test   $0x1,%r10b
  31238e:	0f 85 ce fd ff ff    	jne    312162 <<rexx_exec::Interp>::exec_parse+0x2d42>
  312394:	41 c1 ea 08          	shr    $0x8,%r10d
  312398:	41 f6 c2 01          	test   $0x1,%r10b
  31239c:	0f 84 d4 fd ff ff    	je     312176 <<rexx_exec::Interp>::exec_parse+0x2d56>
  3123a2:	42 c6 44 0f 19 3f    	movb   $0x3f,0x19(%rdi,%r9,1)
  3123a8:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  3123ae:	41 f6 c2 01          	test   $0x1,%r10b
  3123b2:	0f 85 ce fd ff ff    	jne    312186 <<rexx_exec::Interp>::exec_parse+0x2d66>
  3123b8:	41 c1 ea 08          	shr    $0x8,%r10d
  3123bc:	41 f6 c2 01          	test   $0x1,%r10b
  3123c0:	0f 84 d4 fd ff ff    	je     31219a <<rexx_exec::Interp>::exec_parse+0x2d7a>
  3123c6:	42 c6 44 0f 1b 3f    	movb   $0x3f,0x1b(%rdi,%r9,1)
  3123cc:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  3123d2:	41 f6 c2 01          	test   $0x1,%r10b
  3123d6:	0f 85 ce fd ff ff    	jne    3121aa <<rexx_exec::Interp>::exec_parse+0x2d8a>
  3123dc:	41 c1 ea 08          	shr    $0x8,%r10d
  3123e0:	41 f6 c2 01          	test   $0x1,%r10b
  3123e4:	0f 84 d4 fd ff ff    	je     3121be <<rexx_exec::Interp>::exec_parse+0x2d9e>
  3123ea:	42 c6 44 0f 1d 3f    	movb   $0x3f,0x1d(%rdi,%r9,1)
  3123f0:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  3123f6:	41 f6 c2 01          	test   $0x1,%r10b
  3123fa:	0f 85 ce fd ff ff    	jne    3121ce <<rexx_exec::Interp>::exec_parse+0x2dae>
  312400:	41 c1 ea 08          	shr    $0x8,%r10d
  312404:	41 f6 c2 01          	test   $0x1,%r10b
  312408:	0f 84 f2 fa ff ff    	je     311f00 <<rexx_exec::Interp>::exec_parse+0x2ae0>
  31240e:	42 c6 44 0f 1f 3f    	movb   $0x3f,0x1f(%rdi,%r9,1)
  312414:	e9 e7 fa ff ff       	jmp    311f00 <<rexx_exec::Interp>::exec_parse+0x2ae0>
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312419:	4c 39 c6             	cmp    %r8,%rsi
  31241c:	0f 84 33 f7 ff ff    	je     311b55 <<rexx_exec::Interp>::exec_parse+0x2735>
rexx_exec::error::displayable:
  312422:	40 f6 c6 10          	test   $0x10,%sil
  312426:	0f 85 03 f8 ff ff    	jne    311c2f <<rexx_exec::Interp>::exec_parse+0x280f>
  31242c:	48 01 d0             	add    %rdx,%rax
  31242f:	eb 1b                	jmp    31244c <<rexx_exec::Interp>::exec_parse+0x302c>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  312431:	c6 01 3f             	movb   $0x3f,(%rcx)
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
  312434:	66 66 66 2e 0f 1f 84 	data16 data16 cs nopw 0x0(%rax,%rax,1)
  31243b:	00 00 00 00 00 
  312440:	48 ff c1             	inc    %rcx
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312443:	48 39 c1             	cmp    %rax,%rcx
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312446:	0f 84 09 f7 ff ff    	je     311b55 <<rexx_exec::Interp>::exec_parse+0x2735>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  31244c:	0f b6 11             	movzbl (%rcx),%edx
  31244f:	83 fa 1f             	cmp    $0x1f,%edx
  312452:	77 ec                	ja     312440 <<rexx_exec::Interp>::exec_parse+0x3020>
  312454:	83 fa 0d             	cmp    $0xd,%edx
  312457:	77 d8                	ja     312431 <<rexx_exec::Interp>::exec_parse+0x3011>
<rexx_exec::Interp>::assign_targets:
  312459:	be 00 26 00 00       	mov    $0x2600,%esi
  31245e:	0f a3 d6             	bt     %edx,%esi
  312461:	72 dd                	jb     312440 <<rexx_exec::Interp>::exec_parse+0x3020>
  312463:	eb cc                	jmp    312431 <<rexx_exec::Interp>::exec_parse+0x3011>
  312465:	48 8b bc 24 f0 01 00 	mov    0x1f0(%rsp),%rdi
  31246c:	00 
  31246d:	48 8b 8c 24 a0 01 00 	mov    0x1a0(%rsp),%rcx
  312474:	00 
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:311
  312475:	49 89 e8             	mov    %rbp,%r8
  312478:	e8 e3 e2 e4 ff       	call   160760 <<rexx_core::roots::RootSet>::set_frame_slot_of>
  31247d:	e9 53 f3 ff ff       	jmp    3117d5 <<rexx_exec::Interp>::exec_parse+0x23b5>
<rexx_exec::Interp>::exec_parse:
  312482:	48 8b 74 24 38       	mov    0x38(%rsp),%rsi
  312487:	48 89 f7             	mov    %rsi,%rdi
  31248a:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31248d:	83 fa 01             	cmp    $0x1,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312490:	0f 84 c6 eb ff ff    	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312496:	41 be 01 00 00 00    	mov    $0x1,%r14d
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  31249c:	38 4c 05 01          	cmp    %cl,0x1(%rbp,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3124a0:	0f 84 a7 eb ff ff    	je     31104d <<rexx_exec::Interp>::exec_parse+0x1c2d>
  3124a6:	48 89 f7             	mov    %rsi,%rdi
  3124a9:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3124ac:	83 fa 02             	cmp    $0x2,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3124af:	0f 84 a7 eb ff ff    	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  3124b5:	41 be 02 00 00 00    	mov    $0x2,%r14d
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  3124bb:	38 4c 05 02          	cmp    %cl,0x2(%rbp,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3124bf:	0f 84 88 eb ff ff    	je     31104d <<rexx_exec::Interp>::exec_parse+0x1c2d>
  3124c5:	48 89 f7             	mov    %rsi,%rdi
  3124c8:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3124cb:	83 fa 03             	cmp    $0x3,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3124ce:	0f 84 88 eb ff ff    	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  3124d4:	41 be 03 00 00 00    	mov    $0x3,%r14d
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  3124da:	38 4c 05 03          	cmp    %cl,0x3(%rbp,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3124de:	0f 84 69 eb ff ff    	je     31104d <<rexx_exec::Interp>::exec_parse+0x1c2d>
  3124e4:	48 89 f7             	mov    %rsi,%rdi
  3124e7:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3124ea:	83 fa 04             	cmp    $0x4,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3124ed:	0f 84 69 eb ff ff    	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  3124f3:	41 be 04 00 00 00    	mov    $0x4,%r14d
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  3124f9:	38 4c 05 04          	cmp    %cl,0x4(%rbp,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3124fd:	0f 84 4a eb ff ff    	je     31104d <<rexx_exec::Interp>::exec_parse+0x1c2d>
  312503:	48 89 f7             	mov    %rsi,%rdi
  312506:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312509:	83 fa 05             	cmp    $0x5,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31250c:	0f 84 4a eb ff ff    	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312512:	41 be 05 00 00 00    	mov    $0x5,%r14d
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312518:	38 4c 05 05          	cmp    %cl,0x5(%rbp,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  31251c:	0f 84 2b eb ff ff    	je     31104d <<rexx_exec::Interp>::exec_parse+0x1c2d>
  312522:	48 89 f7             	mov    %rsi,%rdi
  312525:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312528:	83 fa 06             	cmp    $0x6,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31252b:	0f 84 2b eb ff ff    	je     31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312531:	41 be 06 00 00 00    	mov    $0x6,%r14d
  312537:	48 89 f7             	mov    %rsi,%rdi
  31253a:	49 89 f0             	mov    %rsi,%r8
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  31253d:	38 4c 05 06          	cmp    %cl,0x6(%rbp,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312541:	0f 84 06 eb ff ff    	je     31104d <<rexx_exec::Interp>::exec_parse+0x1c2d>
  312547:	e9 10 eb ff ff       	jmp    31105c <<rexx_exec::Interp>::exec_parse+0x1c3c>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:56
  31254c:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  312551:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  312556:	4c 89 ea             	mov    %r13,%rdx
  312559:	e8 3a b6 f6 ff       	call   27db98 <<rexx_exec::Interp>::traced_string_value_text>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31255e:	4c 8b 74 24 40       	mov    0x40(%rsp),%r14
  312563:	4c 8b 64 24 48       	mov    0x48(%rsp),%r12
  312568:	48 8b 4c 24 50       	mov    0x50(%rsp),%rcx
  31256d:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
  312571:	74 3e                	je     3125b1 <<rexx_exec::Interp>::exec_parse+0x3191>
<rexx_exec::Interp>::apply_trigger:
  312573:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  312578:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  31257f:	75 19                	jne    31259a <<rexx_exec::Interp>::exec_parse+0x317a>
  312581:	f6 87 2d 05 00 00 01 	testb  $0x1,0x52d(%rdi)
  312588:	74 10                	je     31259a <<rexx_exec::Interp>::exec_parse+0x317a>
<rexx_exec::Interp>::trace_result:
  31258a:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  312591:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:225
  312592:	4c 89 e2             	mov    %r12,%rdx
  312595:	e8 06 b6 f6 ff       	call   27dba0 <<rexx_exec::Interp>::trace_result_line>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31259a:	4d 85 f6             	test   %r14,%r14
  31259d:	0f 84 a4 df ff ff    	je     310547 <<rexx_exec::Interp>::exec_parse+0x1127>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3125a3:	4c 89 e7             	mov    %r12,%rdi
  3125a6:	ff 15 5c 56 1c 00    	call   *0x1c565c(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3125ac:	e9 96 df ff ff       	jmp    310547 <<rexx_exec::Interp>::exec_parse+0x1127>
<rexx_exec::Interp>::exec_parse:
  3125b1:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
  3125b6:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  3125bd:	00 
  3125be:	4c 89 f8             	mov    %r15,%rax
  3125c1:	49 89 df             	mov    %rbx,%r15
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3125c4:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  3125c8:	48 8b b4 24 b0 00 00 	mov    0xb0(%rsp),%rsi
  3125cf:	00 
  3125d0:	0f 84 3e eb ff ff    	je     311114 <<rexx_exec::Interp>::exec_parse+0x1cf4>
  3125d6:	e9 20 04 00 00       	jmp    3129fb <<rexx_exec::Interp>::exec_parse+0x35db>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  3125db:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  3125e0:	4c 89 e6             	mov    %r12,%rsi
  3125e3:	48 8d 94 24 c0 01 00 	lea    0x1c0(%rsp),%rdx
  3125ea:	00 
  3125eb:	8b 8c 24 b8 01 00 00 	mov    0x1b8(%rsp),%ecx
  3125f2:	44 8b 84 24 b4 01 00 	mov    0x1b4(%rsp),%r8d
  3125f9:	00 
  3125fa:	4c 8b 8c 24 c0 00 00 	mov    0xc0(%rsp),%r9
  312601:	00 
  312602:	e8 69 3d 04 00       	call   356370 <<rexx_exec::Interp>::next_template>
<core::result::Result<(rexx_exec::parse_template::SourceText, rexx_exec::parse_template::Cursor), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312607:	4c 8b 7c 24 40       	mov    0x40(%rsp),%r15
  31260c:	48 8d 44 24 41       	lea    0x41(%rsp),%rax
  312611:	0f 10 40 07          	movups 0x7(%rax),%xmm0
  312615:	0f 29 84 24 20 01 00 	movaps %xmm0,0x120(%rsp)
  31261c:	00 
  31261d:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  312621:	0f 84 80 09 00 00    	je     312fa7 <<rexx_exec::Interp>::exec_parse+0x3b87>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  312627:	48 8b 44 24 58       	mov    0x58(%rsp),%rax
  31262c:	48 89 44 24 38       	mov    %rax,0x38(%rsp)
  312631:	48 8b 44 24 68       	mov    0x68(%rsp),%rax
  312636:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  31263b:	48 8b 44 24 70       	mov    0x70(%rsp),%rax
  312640:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  312647:	00 
  312648:	48 8b 44 24 78       	mov    0x78(%rsp),%rax
  31264d:	48 89 84 24 98 01 00 	mov    %rax,0x198(%rsp)
  312654:	00 
  312655:	48 8b 9c 24 80 00 00 	mov    0x80(%rsp),%rbx
  31265c:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  31265d:	66 0f 6f 84 24 20 01 	movdqa 0x120(%rsp),%xmm0
  312664:	00 00 
  312666:	66 0f 7f 84 24 30 02 	movdqa %xmm0,0x230(%rsp)
  31266d:	00 00 
core::mem::replace::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  31266f:	4c 8b b4 24 e0 00 00 	mov    0xe0(%rsp),%r14
  312676:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  312677:	4c 89 bc 24 d8 00 00 	mov    %r15,0xd8(%rsp)
  31267e:	00 
  31267f:	48 8d 84 24 e0 00 00 	lea    0xe0(%rsp),%rax
  312686:	00 
  312687:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  31268b:	48 8b 8c 24 48 01 00 	mov    0x148(%rsp),%rcx
  312692:	00 
<rexx_exec::parse_template::SourceText>::into_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:97
  312693:	48 85 c9             	test   %rcx,%rcx
  312696:	0f 88 8e 00 00 00    	js     31272a <<rexx_exec::Interp>::exec_parse+0x330a>
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  31269c:	48 8d 81 ff ef ff ff 	lea    -0x1001(%rcx),%rax
  3126a3:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  3126a9:	77 10                	ja     3126bb <<rexx_exec::Interp>::exec_parse+0x329b>
  3126ab:	48 85 c9             	test   %rcx,%rcx
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3126ae:	74 7a                	je     31272a <<rexx_exec::Interp>::exec_parse+0x330a>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3126b0:	4c 89 f7             	mov    %r14,%rdi
  3126b3:	ff 15 4f 55 1c 00    	call   *0x1c554f(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3126b9:	eb 6f                	jmp    31272a <<rexx_exec::Interp>::exec_parse+0x330a>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3126bb:	4d 8b a4 24 e0 0b 00 	mov    0xbe0(%r12),%r12
  3126c2:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3126c3:	49 83 fc 08          	cmp    $0x8,%r12
  3126c7:	72 10                	jb     3126d9 <<rexx_exec::Interp>::exec_parse+0x32b9>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3126c9:	4c 89 f7             	mov    %r14,%rdi
  3126cc:	ff 15 36 55 1c 00    	call   *0x1c5536(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  3126d2:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3126d7:	eb 51                	jmp    31272a <<rexx_exec::Interp>::exec_parse+0x330a>
  3126d9:	48 8b 84 24 58 01 00 	mov    0x158(%rsp),%rax
  3126e0:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3126e1:	4c 3b 20             	cmp    (%rax),%r12
  3126e4:	75 0d                	jne    3126f3 <<rexx_exec::Interp>::exec_parse+0x32d3>
  3126e6:	48 8b bc 24 58 01 00 	mov    0x158(%rsp),%rdi
  3126ed:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3126ee:	e8 ad e4 17 00       	call   490ba0 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  3126f3:	48 8b 54 24 18       	mov    0x18(%rsp),%rdx
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3126f8:	48 8b 82 d8 0b 00 00 	mov    0xbd8(%rdx),%rax
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  3126ff:	4b 8d 0c 64          	lea    (%r12,%r12,2),%rcx
  312703:	48 8b b4 24 48 01 00 	mov    0x148(%rsp),%rsi
  31270a:	00 
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  31270b:	48 89 34 c8          	mov    %rsi,(%rax,%rcx,8)
  31270f:	4c 89 74 c8 08       	mov    %r14,0x8(%rax,%rcx,8)
  312714:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  31271b:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  31271d:	49 ff c4             	inc    %r12
  312720:	4c 89 a2 e0 0b 00 00 	mov    %r12,0xbe0(%rdx)
  312727:	49 89 d4             	mov    %rdx,%r12
<rexx_exec::Interp>::exec_parse:
  31272a:	48 8b 84 24 68 01 00 	mov    0x168(%rsp),%rax
  312731:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312732:	48 3b 84 24 d8 01 00 	cmp    0x1d8(%rsp),%rax
  312739:	00 
  31273a:	48 b9 01 00 00 00 00 	movabs $0x8000000000000001,%rcx
  312741:	00 00 80 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312744:	0f 85 73 dc ff ff    	jne    3103bd <<rexx_exec::Interp>::exec_parse+0xf9d>
  31274a:	e9 f6 00 00 00       	jmp    312845 <<rexx_exec::Interp>::exec_parse+0x3425>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  31274f:	66 0f 6f 84 24 20 01 	movdqa 0x120(%rsp),%xmm0
  312756:	00 00 
  312758:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  31275f:	00 
  312760:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  312764:	eb 44                	jmp    3127aa <<rexx_exec::Interp>::exec_parse+0x338a>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  312766:	48 8b 5c 24 48       	mov    0x48(%rsp),%rbx
core::ptr::drop_glue::<core::option::Option<alloc::vec::Vec<u8>>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  31276b:	49 ff ce             	dec    %r14
  31276e:	49 83 fe fd          	cmp    $0xfffffffffffffffd,%r14
  312772:	77 0e                	ja     312782 <<rexx_exec::Interp>::exec_parse+0x3362>
  312774:	48 8b bc 24 f8 00 00 	mov    0xf8(%rsp),%rdi
  31277b:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31277c:	ff 15 86 54 1c 00    	call   *0x1c5486(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  312782:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  312789:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  31278a:	4c 89 38             	mov    %r15,(%rax)
  31278d:	48 89 58 08          	mov    %rbx,0x8(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312791:	48 83 bc 24 48 01 00 	cmpq   $0x0,0x148(%rsp)
  312798:	00 00 
  31279a:	7e 0e                	jle    3127aa <<rexx_exec::Interp>::exec_parse+0x338a>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  31279c:	48 8b bc 24 e0 00 00 	mov    0xe0(%rsp),%rdi
  3127a3:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3127a4:	ff 15 5e 54 1c 00    	call   *0x1c545e(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  3127aa:	48 83 bc 24 c0 01 00 	cmpq   $0x0,0x1c0(%rsp)
  3127b1:	00 00 
core::ptr::drop_glue::<rexx_exec::parse_template::ParseStrings>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3127b3:	0f 8e b6 01 00 00    	jle    31296f <<rexx_exec::Interp>::exec_parse+0x354f>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  3127b9:	48 8b bc 24 c8 01 00 	mov    0x1c8(%rsp),%rdi
  3127c0:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3127c1:	ff 15 41 54 1c 00    	call   *0x1c5441(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  3127c7:	e9 a3 01 00 00       	jmp    31296f <<rexx_exec::Interp>::exec_parse+0x354f>
<rexx_exec::Interp>::exec_parse:
  3127cc:	48 8d 9c 24 30 01 00 	lea    0x130(%rsp),%rbx
  3127d3:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  3127d4:	4c 89 f0             	mov    %r14,%rax
  3127d7:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  3127db:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  3127dd:	48 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%rsi)
  3127e4:	0f 86 ca d9 ff ff    	jbe    3101b4 <<rexx_exec::Interp>::exec_parse+0xd94>
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:408
  3127ea:	48 8b 86 00 0a 00 00 	mov    0xa00(%rsi),%rax
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  3127f1:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  3127f5:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  3127f9:	83 3c 08 01          	cmpl   $0x1,(%rax,%rcx,1)
  3127fd:	0f 85 b1 d9 ff ff    	jne    3101b4 <<rexx_exec::Interp>::exec_parse+0xd94>
  312803:	48 01 c8             	add    %rcx,%rax
  312806:	80 78 08 00          	cmpb   $0x0,0x8(%rax)
  31280a:	0f 85 a4 d9 ff ff    	jne    3101b4 <<rexx_exec::Interp>::exec_parse+0xd94>
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:408
  312810:	4c 89 f1             	mov    %r14,%rcx
  312813:	48 c1 e9 22          	shr    $0x22,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  312817:	39 48 04             	cmp    %ecx,0x4(%rax)
  31281a:	0f 85 94 d9 ff ff    	jne    3101b4 <<rexx_exec::Interp>::exec_parse+0xd94>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  312820:	80 78 10 01          	cmpb   $0x1,0x10(%rax)
  312824:	0f 85 c3 07 00 00    	jne    312fed <<rexx_exec::Interp>::exec_parse+0x3bcd>
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  31282a:	4c 8b 60 28          	mov    0x28(%rax),%r12
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:100
  31282e:	e9 bf 07 00 00       	jmp    312ff2 <<rexx_exec::Interp>::exec_parse+0x3bd2>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  312833:	48 8b 4c 24 48       	mov    0x48(%rsp),%rcx
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312838:	e9 be 01 00 00       	jmp    3129fb <<rexx_exec::Interp>::exec_parse+0x35db>
<rexx_exec::Interp>::exec_parse:
  31283d:	4c 8b bc 24 48 01 00 	mov    0x148(%rsp),%r15
  312844:	00 
<rexx_exec::parse_template::SourceText>::into_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:97
  312845:	4d 85 ff             	test   %r15,%r15
  312848:	78 35                	js     31287f <<rexx_exec::Interp>::exec_parse+0x345f>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:502
  31284a:	48 8b 9c 24 e0 00 00 	mov    0xe0(%rsp),%rbx
  312851:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312852:	49 8d 87 ff ef ff ff 	lea    -0x1001(%r15),%rax
  312859:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  31285f:	77 07                	ja     312868 <<rexx_exec::Interp>::exec_parse+0x3448>
  312861:	4d 85 ff             	test   %r15,%r15
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312864:	75 10                	jne    312876 <<rexx_exec::Interp>::exec_parse+0x3456>
  312866:	eb 17                	jmp    31287f <<rexx_exec::Interp>::exec_parse+0x345f>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312868:	4d 8b b4 24 e0 0b 00 	mov    0xbe0(%r12),%r14
  31286f:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312870:	49 83 fe 08          	cmp    $0x8,%r14
  312874:	72 57                	jb     3128cd <<rexx_exec::Interp>::exec_parse+0x34ad>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312876:	48 89 df             	mov    %rbx,%rdi
  312879:	ff 15 89 53 1c 00    	call   *0x1c5389(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  31287f:	4c 8b b4 24 c0 01 00 	mov    0x1c0(%rsp),%r14
  312886:	00 
  312887:	4d 85 f6             	test   %r14,%r14
<rexx_exec::Interp>::give_parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:389
  31288a:	0f 88 d0 00 00 00    	js     312960 <<rexx_exec::Interp>::exec_parse+0x3540>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  312890:	48 8b 9c 24 c8 01 00 	mov    0x1c8(%rsp),%rbx
  312897:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312898:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  31289f:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  3128a5:	77 0a                	ja     3128b1 <<rexx_exec::Interp>::exec_parse+0x3491>
  3128a7:	4d 85 f6             	test   %r14,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3128aa:	75 13                	jne    3128bf <<rexx_exec::Interp>::exec_parse+0x349f>
  3128ac:	e9 af 00 00 00       	jmp    312960 <<rexx_exec::Interp>::exec_parse+0x3540>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3128b1:	4d 8b bc 24 e0 0b 00 	mov    0xbe0(%r12),%r15
  3128b8:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3128b9:	49 83 ff 08          	cmp    $0x8,%r15
  3128bd:	72 61                	jb     312920 <<rexx_exec::Interp>::exec_parse+0x3500>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3128bf:	48 89 df             	mov    %rbx,%rdi
  3128c2:	ff 15 40 53 1c 00    	call   *0x1c5340(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3128c8:	e9 93 00 00 00       	jmp    312960 <<rexx_exec::Interp>::exec_parse+0x3540>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3128cd:	4d 3b b4 24 d0 0b 00 	cmp    0xbd0(%r12),%r14
  3128d4:	00 
  3128d5:	75 0d                	jne    3128e4 <<rexx_exec::Interp>::exec_parse+0x34c4>
<rexx_exec::Interp>::give_parse_buffer:
  3128d7:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  3128de:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3128df:	e8 bc e2 17 00       	call   490ba0 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3128e4:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  3128eb:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  3128ec:	4b 8d 0c 76          	lea    (%r14,%r14,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3128f0:	4c 89 3c c8          	mov    %r15,(%rax,%rcx,8)
  3128f4:	48 89 5c c8 08       	mov    %rbx,0x8(%rax,%rcx,8)
  3128f9:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312900:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312902:	49 ff c6             	inc    %r14
  312905:	4d 89 b4 24 e0 0b 00 	mov    %r14,0xbe0(%r12)
  31290c:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  31290d:	4c 8b b4 24 c0 01 00 	mov    0x1c0(%rsp),%r14
  312914:	00 
  312915:	4d 85 f6             	test   %r14,%r14
<rexx_exec::Interp>::give_parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:389
  312918:	0f 89 72 ff ff ff    	jns    312890 <<rexx_exec::Interp>::exec_parse+0x3470>
  31291e:	eb 40                	jmp    312960 <<rexx_exec::Interp>::exec_parse+0x3540>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312920:	4d 3b bc 24 d0 0b 00 	cmp    0xbd0(%r12),%r15
  312927:	00 
  312928:	75 0d                	jne    312937 <<rexx_exec::Interp>::exec_parse+0x3517>
<rexx_exec::Interp>::give_parse_buffer:
  31292a:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  312931:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312932:	e8 69 e2 17 00       	call   490ba0 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312937:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  31293e:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  31293f:	4b 8d 0c 7f          	lea    (%r15,%r15,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312943:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  312947:	48 89 5c c8 08       	mov    %rbx,0x8(%rax,%rcx,8)
  31294c:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312953:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312955:	49 ff c7             	inc    %r15
  312958:	4d 89 bc 24 e0 0b 00 	mov    %r15,0xbe0(%r12)
  31295f:	00 
<rexx_exec::Interp>::exec_parse:
  312960:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  312967:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:504
  312968:	48 c7 00 ff ff ff ff 	movq   $0xffffffffffffffff,(%rax)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  31296f:	48 81 c4 48 02 00 00 	add    $0x248,%rsp
  312976:	5b                   	pop    %rbx
  312977:	41 5c                	pop    %r12
  312979:	41 5d                	pop    %r13
  31297b:	41 5e                	pop    %r14
  31297d:	41 5f                	pop    %r15
  31297f:	5d                   	pop    %rbp
  312980:	c3                   	ret
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  312981:	bf 34 00 00 00       	mov    $0x34,%edi
  312986:	ff 15 9c 52 1c 00    	call   *0x1c529c(%rip)        # 4d7c28 <malloc@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  31298c:	48 85 c0             	test   %rax,%rax
  31298f:	0f 84 d3 08 00 00    	je     313268 <<rexx_exec::Interp>::exec_parse+0x3e48>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  312995:	49 89 c6             	mov    %rax,%r14
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312998:	0f 10 05 c9 7c d5 ff 	movups -0x2a8337(%rip),%xmm0        # 6a668 <panic_unwind::imp::CANARY+0x5903>
  31299f:	0f 11 40 20          	movups %xmm0,0x20(%rax)
  3129a3:	0f 10 05 ae 7c d5 ff 	movups -0x2a8352(%rip),%xmm0        # 6a658 <panic_unwind::imp::CANARY+0x58f3>
  3129aa:	0f 11 40 10          	movups %xmm0,0x10(%rax)
  3129ae:	f3 0f 6f 05 92 7c d5 	movdqu -0x2a836e(%rip),%xmm0        # 6a648 <panic_unwind::imp::CANARY+0x58e3>
  3129b5:	ff 
  3129b6:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  3129ba:	c7 40 30 72 61 6e 64 	movl   $0x646e6172,0x30(%rax)
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  3129c1:	bf 18 00 00 00       	mov    $0x18,%edi
  3129c6:	ff 15 5c 52 1c 00    	call   *0x1c525c(%rip)        # 4d7c28 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  3129cc:	48 85 c0             	test   %rax,%rax
  3129cf:	0f 84 9a 07 00 00    	je     31316f <<rexx_exec::Interp>::exec_parse+0x3d4f>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  3129d5:	48 c7 00 34 00 00 00 	movq   $0x34,(%rax)
  3129dc:	4c 89 70 08          	mov    %r14,0x8(%rax)
  3129e0:	48 c7 40 10 34 00 00 	movq   $0x34,0x10(%rax)
  3129e7:	00 
  3129e8:	48 89 c1             	mov    %rax,%rcx
  3129eb:	41 bc 02 00 00 00    	mov    $0x2,%r12d
<rexx_exec::Interp>::apply_trigger:
  3129f1:	eb 08                	jmp    3129fb <<rexx_exec::Interp>::exec_parse+0x35db>
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  3129f3:	48 8b 8c 24 28 01 00 	mov    0x128(%rsp),%rcx
  3129fa:	00 
<rexx_exec::Interp>::exec_parse:
  3129fb:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  312a02:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  312a03:	4c 89 20             	mov    %r12,(%rax)
  312a06:	48 89 48 08          	mov    %rcx,0x8(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312a0a:	48 83 bc 24 48 01 00 	cmpq   $0x0,0x148(%rsp)
  312a11:	00 00 
  312a13:	0f 8f 83 fd ff ff    	jg     31279c <<rexx_exec::Interp>::exec_parse+0x337c>
  312a19:	e9 8c fd ff ff       	jmp    3127aa <<rexx_exec::Interp>::exec_parse+0x338a>
<rexx_exec::Interp>::traced_into_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:353
  312a1e:	e8 ad 36 04 00       	call   3560d0 <<rexx_exec::Interp>::object_into_parse_buffer>
  312a23:	e9 25 d5 ff ff       	jmp    30ff4d <<rexx_exec::Interp>::exec_parse+0xb2d>
<rexx_exec::Interp>::parse_strings:
  312a28:	48 89 94 24 18 01 00 	mov    %rdx,0x118(%rsp)
  312a2f:	00 
<rexx_exec::Code>::stem:
  312a30:	48 01 c8             	add    %rcx,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1287
  312a33:	48 8b 48 08          	mov    0x8(%rax),%rcx
  312a37:	48 89 8c 24 a8 01 00 	mov    %rcx,0x1a8(%rsp)
  312a3e:	00 
  312a3f:	48 8b 48 10          	mov    0x10(%rax),%rcx
  312a43:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
  312a48:	48 8b 40 18          	mov    0x18(%rax),%rax
  312a4c:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  312a51:	e9 0b d3 ff ff       	jmp    30fd61 <<rexx_exec::Interp>::exec_parse+0x941>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:413
  312a56:	48 8d 0d 43 a4 1a 00 	lea    0x1aa443(%rip),%rcx        # 4bcea0 <rexx_api::ffi::CALL_CONTEXT+0x1eb8>
  312a5d:	4c 89 ff             	mov    %r15,%rdi
  312a60:	4c 89 f6             	mov    %r14,%rsi
  312a63:	48 89 c2             	mov    %rax,%rdx
  312a66:	e8 55 9c e0 ff       	call   11c6c0 <core::slice::index::slice_index_fail>
  312a6b:	e9 18 08 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312a70:	49 8d bf e0 0a 00 00 	lea    0xae0(%r15),%rdi
  312a77:	e8 34 94 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  312a7c:	e9 66 d4 ff ff       	jmp    30fee7 <<rexx_exec::Interp>::exec_parse+0xac7>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  312a81:	bf 18 00 00 00       	mov    $0x18,%edi
  312a86:	ff 15 9c 51 1c 00    	call   *0x1c519c(%rip)        # 4d7c28 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  312a8c:	48 85 c0             	test   %rax,%rax
  312a8f:	0f 84 14 07 00 00    	je     3131a9 <<rexx_exec::Interp>::exec_parse+0x3d89>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:89
  312a95:	49 89 c7             	mov    %rax,%r15
  312a98:	48 8d bc 24 20 01 00 	lea    0x120(%rsp),%rdi
  312a9f:	00 
  312aa0:	4c 89 f6             	mov    %r14,%rsi
  312aa3:	4c 89 ea             	mov    %r13,%rdx
  312aa6:	e8 85 cd e9 ff       	call   1af830 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  312aab:	48 8b ac 24 20 01 00 	mov    0x120(%rsp),%rbp
  312ab2:	00 
  312ab3:	4c 8b b4 24 28 01 00 	mov    0x128(%rsp),%r14
  312aba:	00 
  312abb:	4c 8b a4 24 30 01 00 	mov    0x130(%rsp),%r12
  312ac2:	00 
  312ac3:	4d 85 e4             	test   %r12,%r12
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  312ac6:	0f 89 c5 00 00 00    	jns    312b91 <<rexx_exec::Interp>::exec_parse+0x3771>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  312acc:	45 31 ed             	xor    %r13d,%r13d
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  312acf:	4c 89 ef             	mov    %r13,%rdi
  312ad2:	4c 89 e6             	mov    %r12,%rsi
  312ad5:	e8 ca ef df ff       	call   111aa4 <alloc::raw_vec::handle_error>
  312ada:	e9 a9 07 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  312adf:	0f 84 27 01 00 00    	je     312c0c <<rexx_exec::Interp>::exec_parse+0x37ec>
  312ae5:	bd 01 00 00 00       	mov    $0x1,%ebp
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  312aea:	be 01 00 00 00       	mov    $0x1,%esi
  312aef:	48 89 df             	mov    %rbx,%rdi
  312af2:	e8 29 4c e1 ff       	call   127720 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312af7:	48 85 c0             	test   %rax,%rax
  312afa:	0f 84 63 d3 ff ff    	je     30fe63 <<rexx_exec::Interp>::exec_parse+0xa43>
<alloc::vec::Vec<u8>>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:969
  312b00:	48 89 5c 24 40       	mov    %rbx,0x40(%rsp)
  312b05:	48 89 44 24 48       	mov    %rax,0x48(%rsp)
  312b0a:	48 89 44 24 38       	mov    %rax,0x38(%rsp)
  312b0f:	48 89 c7             	mov    %rax,%rdi
  312b12:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312b17:	48 89 da             	mov    %rbx,%rdx
  312b1a:	ff 15 18 51 1c 00    	call   *0x1c5118(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<u8>>::set_len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2224
  312b20:	48 89 5c 24 50       	mov    %rbx,0x50(%rsp)
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  312b25:	e9 07 01 00 00       	jmp    312c31 <<rexx_exec::Interp>::exec_parse+0x3811>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  312b2a:	0f 84 92 02 00 00    	je     312dc2 <<rexx_exec::Interp>::exec_parse+0x39a2>
  312b30:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  312b35:	be 01 00 00 00       	mov    $0x1,%esi
  312b3a:	48 89 ef             	mov    %rbp,%rdi
  312b3d:	e8 de 4b e1 ff       	call   127720 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312b42:	48 85 c0             	test   %rax,%rax
  312b45:	0f 84 15 d1 ff ff    	je     30fc60 <<rexx_exec::Interp>::exec_parse+0x840>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  312b4b:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312b4e:	48 89 c7             	mov    %rax,%rdi
  312b51:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  312b56:	48 89 ea             	mov    %rbp,%rdx
  312b59:	ff 15 d9 50 1c 00    	call   *0x1c50d9(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  312b5f:	e9 64 02 00 00       	jmp    312dc8 <<rexx_exec::Interp>::exec_parse+0x39a8>
<rexx_exec::Interp>::assign_targets:
  312b64:	48 8d 0d bd 77 1a 00 	lea    0x1a77bd(%rip),%rcx        # 4ba328 <rexx_api::ffi::INSTANCE+0x5ef8>
  312b6b:	ba 07 00 00 00       	mov    $0x7,%edx
  312b70:	31 ff                	xor    %edi,%edi
  312b72:	48 89 c6             	mov    %rax,%rsi
  312b75:	e8 46 9b e0 ff       	call   11c6c0 <core::slice::index::slice_index_fail>
  312b7a:	e9 09 07 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312b7f:	49 8d bc 24 e0 0a 00 	lea    0xae0(%r12),%rdi
  312b86:	00 
  312b87:	e8 24 93 e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  312b8c:	e9 20 d5 ff ff       	jmp    3100b1 <<rexx_exec::Interp>::exec_parse+0xc91>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  312b91:	0f 84 df 02 00 00    	je     312e76 <<rexx_exec::Interp>::exec_parse+0x3a56>
  312b97:	41 bd 01 00 00 00    	mov    $0x1,%r13d
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  312b9d:	be 01 00 00 00       	mov    $0x1,%esi
  312ba2:	4c 89 e7             	mov    %r12,%rdi
  312ba5:	e8 76 4b e1 ff       	call   127720 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312baa:	48 85 c0             	test   %rax,%rax
  312bad:	0f 84 1c ff ff ff    	je     312acf <<rexx_exec::Interp>::exec_parse+0x36af>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  312bb3:	48 89 c3             	mov    %rax,%rbx
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312bb6:	48 89 c7             	mov    %rax,%rdi
  312bb9:	4c 89 f6             	mov    %r14,%rsi
  312bbc:	4c 89 e2             	mov    %r12,%rdx
  312bbf:	ff 15 73 50 1c 00    	call   *0x1c5073(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  312bc5:	e9 b1 02 00 00       	jmp    312e7b <<rexx_exec::Interp>::exec_parse+0x3a5b>
<rexx_exec::Interp>::apply_trigger:
  312bca:	4c 89 e9             	mov    %r13,%rcx
  312bcd:	e9 29 fe ff ff       	jmp    3129fb <<rexx_exec::Interp>::exec_parse+0x35db>
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  312bd2:	48 8d 3d 89 72 d5 ff 	lea    -0x2a8d77(%rip),%rdi        # 69e62 <panic_unwind::imp::CANARY+0x50fd>
  312bd9:	48 8d 15 20 d2 1a 00 	lea    0x1ad220(%rip),%rdx        # 4bfe00 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  312be0:	be 11 00 00 00       	mov    $0x11,%esi
  312be5:	e8 c6 c3 e0 ff       	call   11efb0 <core::option::expect_failed>
  312bea:	e9 99 06 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  312bef:	48 8d 3d 49 64 d5 ff 	lea    -0x2a9bb7(%rip),%rdi        # 6903f <panic_unwind::imp::CANARY+0x42da>
  312bf6:	48 8d 15 eb a2 1a 00 	lea    0x1aa2eb(%rip),%rdx        # 4bcee8 <rexx_api::ffi::CALL_CONTEXT+0x1f00>
  312bfd:	be 36 00 00 00       	mov    $0x36,%esi
  312c02:	e8 a9 c3 e0 ff       	call   11efb0 <core::option::expect_failed>
  312c07:	e9 7c 06 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::vec::Vec<u8>>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:969
  312c0c:	48 c7 44 24 40 00 00 	movq   $0x0,0x40(%rsp)
  312c13:	00 00 
  312c15:	48 c7 44 24 48 01 00 	movq   $0x1,0x48(%rsp)
  312c1c:	00 00 
  312c1e:	48 c7 44 24 50 00 00 	movq   $0x0,0x50(%rsp)
  312c25:	00 00 
  312c27:	b8 01 00 00 00       	mov    $0x1,%eax
  312c2c:	48 89 44 24 38       	mov    %rax,0x38(%rsp)
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  312c31:	48 8b ac 24 80 01 00 	mov    0x180(%rsp),%rbp
  312c38:	00 
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  312c39:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  312c3c:	0f 85 7b 05 00 00    	jne    3131bd <<rexx_exec::Interp>::exec_parse+0x3d9d>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  312c42:	48 01 eb             	add    %rbp,%rbx
  312c45:	48 89 5c 24 50       	mov    %rbx,0x50(%rsp)
  312c4a:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  312c4f:	41 80 be 35 05 00 00 	cmpb   $0x0,0x535(%r14)
  312c56:	00 
  312c57:	75 55                	jne    312cae <<rexx_exec::Interp>::exec_parse+0x388e>
  312c59:	41 f6 86 2e 05 00 00 	testb  $0x1,0x52e(%r14)
  312c60:	01 
  312c61:	74 4b                	je     312cae <<rexx_exec::Interp>::exec_parse+0x388e>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312c63:	49 8d be d8 10 00 00 	lea    0x10d8(%r14),%rdi
  312c6a:	49 8b ae e8 10 00 00 	mov    0x10e8(%r14),%rbp
<rexx_exec::Interp>::trace_compound_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:594
  312c71:	48 89 5c 24 10       	mov    %rbx,0x10(%rsp)
  312c76:	48 8b 44 24 38       	mov    0x38(%rsp),%rax
  312c7b:	48 89 44 24 08       	mov    %rax,0x8(%rsp)
  312c80:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  312c87:	48 8d 35 1f 7f d5 ff 	lea    -0x2a80e1(%rip),%rsi        # 6abad <panic_unwind::imp::CANARY+0x5e48>
  312c8e:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  312c95:	00 
  312c96:	31 c9                	xor    %ecx,%ecx
  312c98:	4d 89 e8             	mov    %r13,%r8
  312c9b:	4d 89 e1             	mov    %r12,%r9
  312c9e:	e8 ad d6 f6 ff       	call   280350 <rexx_exec::trace::format::push_tagged>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:595
  312ca3:	4c 89 f7             	mov    %r14,%rdi
  312ca6:	48 89 ee             	mov    %rbp,%rsi
  312ca9:	e8 82 b3 e8 ff       	call   19e030 <<rexx_exec::Interp>::route_trace_line>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312cae:	48 83 7c 24 40 00    	cmpq   $0x0,0x40(%rsp)
  312cb4:	74 0b                	je     312cc1 <<rexx_exec::Interp>::exec_parse+0x38a1>
<alloc::raw_vec::RawVecInner>::deallocate:
  312cb6:	48 8b 7c 24 38       	mov    0x38(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312cbb:	ff 15 47 4f 1c 00    	call   *0x1c4f47(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  312cc1:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312cc6:	4c 8b 8c 24 78 01 00 	mov    0x178(%rsp),%r9
  312ccd:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  312cce:	48 8b 84 24 80 01 00 	mov    0x180(%rsp),%rax
  312cd5:	00 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:706
  312cd6:	48 89 04 24          	mov    %rax,(%rsp)
  312cda:	4c 89 f7             	mov    %r14,%rdi
  312cdd:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  312ce2:	48 8b 54 24 20       	mov    0x20(%rsp),%rdx
  312ce7:	48 8b 8c 24 18 01 00 	mov    0x118(%rsp),%rcx
  312cee:	00 
  312cef:	4c 8b 84 24 a8 01 00 	mov    0x1a8(%rsp),%r8
  312cf6:	00 
  312cf7:	e8 b4 e2 fa ff       	call   2c0fb0 <<rexx_exec::Interp>::stem_get_at>
  312cfc:	49 89 c6             	mov    %rax,%r14
  312cff:	89 d3                	mov    %edx,%ebx
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312d01:	49 83 3f 00          	cmpq   $0x0,(%r15)
  312d05:	74 12                	je     312d19 <<rexx_exec::Interp>::exec_parse+0x38f9>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
  312d07:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312d0c:	48 8b b8 c0 0b 00 00 	mov    0xbc0(%rax),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312d13:	ff 15 ef 4e 1c 00    	call   *0x1c4eef(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::give_key_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:64
  312d19:	48 8b 84 24 80 01 00 	mov    0x180(%rsp),%rax
  312d20:	00 
  312d21:	49 89 47 10          	mov    %rax,0x10(%r15)
  312d25:	66 0f 6f 84 24 70 01 	movdqa 0x170(%rsp),%xmm0
  312d2c:	00 00 
  312d2e:	f3 41 0f 7f 07       	movdqu %xmm0,(%r15)
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:315
  312d33:	f6 c3 01             	test   $0x1,%bl
  312d36:	0f 85 47 04 00 00    	jne    313183 <<rexx_exec::Interp>::exec_parse+0x3d63>
<rexx_exec::Interp>::read_parse_var:
  312d3c:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  312d41:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  312d48:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:709
  312d49:	0f 84 84 d1 ff ff    	je     30fed3 <<rexx_exec::Interp>::exec_parse+0xab3>
  312d4f:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:710
  312d54:	4c 89 fe             	mov    %r15,%rsi
  312d57:	4c 89 f2             	mov    %r14,%rdx
  312d5a:	e8 d1 ca e9 ff       	call   1af830 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  312d5f:	48 8b 4c 24 40       	mov    0x40(%rsp),%rcx
  312d64:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
  312d69:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  312d6e:	48 8b 6c 24 50       	mov    0x50(%rsp),%rbp
  312d73:	48 85 ed             	test   %rbp,%rbp
  312d76:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  312d7b:	79 12                	jns    312d8f <<rexx_exec::Interp>::exec_parse+0x396f>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  312d7d:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  312d7f:	48 89 df             	mov    %rbx,%rdi
  312d82:	48 89 ee             	mov    %rbp,%rsi
  312d85:	e8 1a ed df ff       	call   111aa4 <alloc::raw_vec::handle_error>
  312d8a:	e9 f9 04 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  312d8f:	0f 84 87 00 00 00    	je     312e1c <<rexx_exec::Interp>::exec_parse+0x39fc>
  312d95:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  312d9a:	be 01 00 00 00       	mov    $0x1,%esi
  312d9f:	48 89 ef             	mov    %rbp,%rdi
  312da2:	e8 79 49 e1 ff       	call   127720 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312da7:	48 85 c0             	test   %rax,%rax
  312daa:	74 d3                	je     312d7f <<rexx_exec::Interp>::exec_parse+0x395f>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  312dac:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312daf:	48 89 c7             	mov    %rax,%rdi
  312db2:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  312db7:	48 89 ea             	mov    %rbp,%rdx
  312dba:	ff 15 78 4e 1c 00    	call   *0x1c4e78(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  312dc0:	eb 60                	jmp    312e22 <<rexx_exec::Interp>::exec_parse+0x3a02>
<rexx_exec::Interp>::read_parse_var:
  312dc2:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312dc8:	48 83 7c 24 20 00    	cmpq   $0x0,0x20(%rsp)
  312dce:	7e 0b                	jle    312ddb <<rexx_exec::Interp>::exec_parse+0x39bb>
  312dd0:	48 8b 7c 24 28       	mov    0x28(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312dd5:	ff 15 2d 4e 1c 00    	call   *0x1c4e2d(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  312ddb:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  312de0:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  312de7:	0f 85 d3 d0 ff ff    	jne    30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
  312ded:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  312df4:	0f 84 c6 d0 ff ff    	je     30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  312dfa:	4c 89 34 24          	mov    %r14,(%rsp)
  312dfe:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  312e05:	00 
  312e06:	4c 89 ea             	mov    %r13,%rdx
  312e09:	4c 89 e1             	mov    %r12,%rcx
  312e0c:	4d 89 f8             	mov    %r15,%r8
  312e0f:	49 89 e9             	mov    %rbp,%r9
  312e12:	e8 39 df f6 ff       	call   280d50 <<rexx_exec::Interp>::trace_variable_line>
  312e17:	e9 a4 d0 ff ff       	jmp    30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
<rexx_exec::Interp>::read_parse_var:
  312e1c:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312e22:	48 83 7c 24 20 00    	cmpq   $0x0,0x20(%rsp)
  312e28:	7e 0b                	jle    312e35 <<rexx_exec::Interp>::exec_parse+0x3a15>
  312e2a:	48 8b 7c 24 28       	mov    0x28(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312e2f:	ff 15 d3 4d 1c 00    	call   *0x1c4dd3(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  312e35:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  312e3a:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  312e41:	0f 85 79 d0 ff ff    	jne    30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
  312e47:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  312e4e:	0f 84 6c d0 ff ff    	je     30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  312e54:	4c 89 34 24          	mov    %r14,(%rsp)
  312e58:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  312e5f:	00 
  312e60:	4c 89 ea             	mov    %r13,%rdx
  312e63:	4c 89 e1             	mov    %r12,%rcx
  312e66:	4d 89 f8             	mov    %r15,%r8
  312e69:	49 89 e9             	mov    %rbp,%r9
  312e6c:	e8 df de f6 ff       	call   280d50 <<rexx_exec::Interp>::trace_variable_line>
  312e71:	e9 4a d0 ff ff       	jmp    30fec0 <<rexx_exec::Interp>::exec_parse+0xaa0>
<rexx_exec::Interp>::exec_parse:
  312e76:	bb 01 00 00 00       	mov    $0x1,%ebx
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:89
  312e7b:	4d 89 27             	mov    %r12,(%r15)
  312e7e:	49 89 5f 08          	mov    %rbx,0x8(%r15)
  312e82:	4d 89 67 10          	mov    %r12,0x10(%r15)
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  312e86:	bf 02 00 00 00       	mov    $0x2,%edi
  312e8b:	be 01 00 00 00       	mov    $0x1,%esi
  312e90:	e8 8b 48 e1 ff       	call   127720 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312e95:	48 85 c0             	test   %rax,%rax
  312e98:	0f 84 db 03 00 00    	je     313279 <<rexx_exec::Interp>::exec_parse+0x3e59>
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312e9e:	66 c7 00 32 36       	movw   $0x3632,(%rax)
<rexx_exec::error::Raised>::syntax:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:139
  312ea3:	48 c7 44 24 58 ff ff 	movq   $0xffffffffffffffff,0x58(%rsp)
  312eaa:	ff ff 
  312eac:	48 8d 0d 6c 3d d9 ff 	lea    -0x26c294(%rip),%rcx        # a6c1f <panic_unwind::imp::CANARY+0x41eba>
  312eb3:	48 89 4c 24 60       	mov    %rcx,0x60(%rsp)
  312eb8:	48 c7 44 24 68 06 00 	movq   $0x6,0x68(%rsp)
  312ebf:	00 00 
  312ec1:	48 c7 44 24 40 01 00 	movq   $0x1,0x40(%rsp)
  312ec8:	00 00 
  312eca:	4c 89 7c 24 48       	mov    %r15,0x48(%rsp)
  312ecf:	48 c7 44 24 50 01 00 	movq   $0x1,0x50(%rsp)
  312ed6:	00 00 
  312ed8:	48 c7 44 24 70 02 00 	movq   $0x2,0x70(%rsp)
  312edf:	00 00 
  312ee1:	48 89 44 24 78       	mov    %rax,0x78(%rsp)
  312ee6:	48 c7 84 24 80 00 00 	movq   $0x2,0x80(%rsp)
  312eed:	00 02 00 00 00 
  312ef2:	48 c7 84 24 88 00 00 	movq   $0xffffffffffffffff,0x88(%rsp)
  312ef9:	00 ff ff ff ff 
  312efe:	48 b8 00 00 00 00 1a 	movabs $0x4001a00000000,%rax
  312f05:	00 04 00 
  312f08:	48 89 84 24 a0 00 00 	mov    %rax,0xa0(%rsp)
  312f0f:	00 
  312f10:	c7 84 24 a8 00 00 00 	movl   $0x0,0xa8(%rsp)
  312f17:	00 00 00 00 
  312f1b:	66 c7 84 24 ac 00 00 	movw   $0x0,0xac(%rsp)
  312f22:	00 00 00 
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  312f25:	bf 70 00 00 00       	mov    $0x70,%edi
  312f2a:	ff 15 f8 4c 1c 00    	call   *0x1c4cf8(%rip)        # 4d7c28 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  312f30:	48 85 c0             	test   %rax,%rax
  312f33:	0f 84 db 02 00 00    	je     313214 <<rexx_exec::Interp>::exec_parse+0x3df4>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  312f39:	0f 10 84 24 a0 00 00 	movups 0xa0(%rsp),%xmm0
  312f40:	00 
  312f41:	0f 11 40 60          	movups %xmm0,0x60(%rax)
  312f45:	0f 10 84 24 90 00 00 	movups 0x90(%rsp),%xmm0
  312f4c:	00 
  312f4d:	0f 11 40 50          	movups %xmm0,0x50(%rax)
  312f51:	0f 10 84 24 80 00 00 	movups 0x80(%rsp),%xmm0
  312f58:	00 
  312f59:	0f 11 40 40          	movups %xmm0,0x40(%rax)
  312f5d:	f3 0f 6f 44 24 40    	movdqu 0x40(%rsp),%xmm0
  312f63:	f3 0f 6f 4c 24 50    	movdqu 0x50(%rsp),%xmm1
  312f69:	f3 0f 6f 54 24 60    	movdqu 0x60(%rsp),%xmm2
  312f6f:	f3 0f 6f 5c 24 70    	movdqu 0x70(%rsp),%xmm3
  312f75:	f3 0f 7f 58 30       	movdqu %xmm3,0x30(%rax)
  312f7a:	f3 0f 7f 50 20       	movdqu %xmm2,0x20(%rax)
  312f7f:	f3 0f 7f 48 10       	movdqu %xmm1,0x10(%rax)
  312f84:	48 89 c3             	mov    %rax,%rbx
  312f87:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312f8b:	48 85 ed             	test   %rbp,%rbp
  312f8e:	7e 09                	jle    312f99 <<rexx_exec::Interp>::exec_parse+0x3b79>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312f90:	4c 89 f7             	mov    %r14,%rdi
  312f93:	ff 15 6f 4c 1c 00    	call   *0x1c4c6f(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  312f99:	41 bc 03 00 00 00    	mov    $0x3,%r12d
  312f9f:	48 89 d9             	mov    %rbx,%rcx
  312fa2:	e9 54 fa ff ff       	jmp    3129fb <<rexx_exec::Interp>::exec_parse+0x35db>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  312fa7:	66 0f 6f 84 24 20 01 	movdqa 0x120(%rsp),%xmm0
  312fae:	00 00 
  312fb0:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  312fb7:	00 
  312fb8:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312fbc:	48 83 bc 24 48 01 00 	cmpq   $0x0,0x148(%rsp)
  312fc3:	00 00 
  312fc5:	0f 8f d1 f7 ff ff    	jg     31279c <<rexx_exec::Interp>::exec_parse+0x337c>
  312fcb:	e9 da f7 ff ff       	jmp    3127aa <<rexx_exec::Interp>::exec_parse+0x338a>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  312fd0:	48 8d 3d af 7c d5 ff 	lea    -0x2a8351(%rip),%rdi        # 6ac86 <panic_unwind::imp::CANARY+0x5f21>
  312fd7:	48 8d 15 42 d5 1a 00 	lea    0x1ad542(%rip),%rdx        # 4c0520 <rexx_api::ffi::CALL_CONTEXT+0x5538>
  312fde:	be 31 00 00 00       	mov    $0x31,%esi
  312fe3:	e8 a8 97 e0 ff       	call   11c790 <core::panicking::panic>
  312fe8:	e9 9b 02 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  312fed:	44 0f b6 60 11       	movzbl 0x11(%rax),%r12d
<rexx_exec::Interp>::source_text:
  312ff2:	48 8b 6c 24 18       	mov    0x18(%rsp),%rbp
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  312ff7:	4c 8b ad f0 0a 00 00 	mov    0xaf0(%rbp),%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312ffe:	4c 3b ad e0 0a 00 00 	cmp    0xae0(%rbp),%r13
  313005:	74 43                	je     31304a <<rexx_exec::Interp>::exec_parse+0x3c2a>
  313007:	49 bf 01 00 00 00 00 	movabs $0x8000000000000001,%r15
  31300e:	00 00 80 
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  313011:	48 8b 85 e8 0a 00 00 	mov    0xae8(%rbp),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  313018:	4e 89 34 e8          	mov    %r14,(%rax,%r13,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  31301c:	49 ff c5             	inc    %r13
  31301f:	4c 89 ad f0 0a 00 00 	mov    %r13,0xaf0(%rbp)
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:414
  313026:	4c 89 b4 24 28 01 00 	mov    %r14,0x128(%rsp)
  31302d:	00 
  31302e:	e9 f7 d1 ff ff       	jmp    31022a <<rexx_exec::Interp>::exec_parse+0xe0a>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:558
  313033:	48 8d 0d 56 0e 1b 00 	lea    0x1b0e56(%rip),%rcx        # 4c3e90 <rexx_api::ffi::CALL_CONTEXT+0x8ea8>
  31303a:	48 89 ef             	mov    %rbp,%rdi
  31303d:	48 89 d6             	mov    %rdx,%rsi
  313040:	e8 7b 96 e0 ff       	call   11c6c0 <core::slice::index::slice_index_fail>
  313045:	e9 3e 02 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31304a:	48 8d bd e0 0a 00 00 	lea    0xae0(%rbp),%rdi
  313051:	e8 5a 8e e4 ff       	call   15beb0 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  313056:	eb af                	jmp    313007 <<rexx_exec::Interp>::exec_parse+0x3be7>
<rexx_exec::Interp>::parse_strings:
  313058:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:318
  31305d:	4c 89 fe             	mov    %r15,%rsi
  313060:	4c 89 f2             	mov    %r14,%rdx
  313063:	e8 48 f5 f6 ff       	call   2825b0 <<rexx_exec::Interp>::novalue_raised>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  313068:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
  31306d:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  313071:	0f 85 5b c6 ff ff    	jne    30f6d2 <<rexx_exec::Interp>::exec_parse+0x2b2>
  313077:	e9 19 cc ff ff       	jmp    30fc95 <<rexx_exec::Interp>::exec_parse+0x875>
<rexx_exec::Interp>::exec_parse:
  31307c:	49 89 dc             	mov    %rbx,%r12
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:66
  31307f:	48 8d 0d a2 72 1a 00 	lea    0x1a72a2(%rip),%rcx        # 4ba328 <rexx_api::ffi::INSTANCE+0x5ef8>
  313086:	ba 07 00 00 00       	mov    $0x7,%edx
  31308b:	31 ff                	xor    %edi,%edi
  31308d:	4c 89 e6             	mov    %r12,%rsi
  313090:	e8 2b 96 e0 ff       	call   11c6c0 <core::slice::index::slice_index_fail>
  313095:	e9 ee 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
  31309a:	49 89 dc             	mov    %rbx,%r12
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:73
  31309d:	48 8d 0d 84 72 1a 00 	lea    0x1a7284(%rip),%rcx        # 4ba328 <rexx_api::ffi::INSTANCE+0x5ef8>
  3130a4:	ba 07 00 00 00       	mov    $0x7,%edx
  3130a9:	31 ff                	xor    %edi,%edi
  3130ab:	4c 89 e6             	mov    %r12,%rsi
  3130ae:	e8 0d 96 e0 ff       	call   11c6c0 <core::slice::index::slice_index_fail>
  3130b3:	e9 d0 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:545
  3130b8:	48 8d 0d a9 a4 1a 00 	lea    0x1aa4a9(%rip),%rcx        # 4bd568 <rexx_api::ffi::CALL_CONTEXT+0x2580>
  3130bf:	be 14 00 00 00       	mov    $0x14,%esi
  3130c4:	ba 14 00 00 00       	mov    $0x14,%edx
  3130c9:	4c 89 f7             	mov    %r14,%rdi
  3130cc:	e8 ef 95 e0 ff       	call   11c6c0 <core::slice::index::slice_index_fail>
  3130d1:	e9 b2 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  3130d6:	48 8d 3d 92 63 d5 ff 	lea    -0x2a9c6e(%rip),%rdi        # 6946f <panic_unwind::imp::CANARY+0x470a>
  3130dd:	48 8d 15 6c a4 1a 00 	lea    0x1aa46c(%rip),%rdx        # 4bd550 <rexx_api::ffi::CALL_CONTEXT+0x2568>
  3130e4:	be 35 00 00 00       	mov    $0x35,%esi
  3130e9:	e8 c2 be e0 ff       	call   11efb0 <core::option::expect_failed>
  3130ee:	e9 95 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
  3130f3:	48 8d 3d 75 63 d5 ff 	lea    -0x2a9c8b(%rip),%rdi        # 6946f <panic_unwind::imp::CANARY+0x470a>
  3130fa:	48 8d 15 4f a4 1a 00 	lea    0x1aa44f(%rip),%rdx        # 4bd550 <rexx_api::ffi::CALL_CONTEXT+0x2568>
  313101:	be 35 00 00 00       	mov    $0x35,%esi
  313106:	e8 a5 be e0 ff       	call   11efb0 <core::option::expect_failed>
  31310b:	e9 78 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:545
  313110:	48 8d 0d 51 a4 1a 00 	lea    0x1aa451(%rip),%rcx        # 4bd568 <rexx_api::ffi::CALL_CONTEXT+0x2580>
  313117:	be 14 00 00 00       	mov    $0x14,%esi
  31311c:	ba 14 00 00 00       	mov    $0x14,%edx
  313121:	4c 89 ef             	mov    %r13,%rdi
  313124:	e8 97 95 e0 ff       	call   11c6c0 <core::slice::index::slice_index_fail>
  313129:	e9 5a 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::Interp>::assign_targets:
  31312e:	48 8d 3d ff 52 d5 ff 	lea    -0x2aad01(%rip),%rdi        # 68434 <panic_unwind::imp::CANARY+0x36cf>
  313135:	48 8d 15 24 94 1a 00 	lea    0x1a9424(%rip),%rdx        # 4bc560 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  31313c:	be ab 00 00 00       	mov    $0xab,%esi
  313141:	e8 9a 77 e0 ff       	call   11a8e0 <core::panicking::panic_fmt>
  313146:	e9 3d 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
  31314b:	48 8d 0d 76 83 1a 00 	lea    0x1a8376(%rip),%rcx        # 4bb4c8 <rexx_api::ffi::CALL_CONTEXT+0x4e0>
  313152:	48 89 8c 24 90 01 00 	mov    %rcx,0x190(%rsp)
  313159:	00 
  31315a:	48 89 c6             	mov    %rax,%rsi
  31315d:	48 8b 94 24 90 01 00 	mov    0x190(%rsp),%rdx
  313164:	00 
  313165:	e8 37 96 e0 ff       	call   11c7a1 <core::panicking::panic_bounds_check>
  31316a:	e9 19 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  31316f:	bf 08 00 00 00       	mov    $0x8,%edi
  313174:	be 18 00 00 00       	mov    $0x18,%esi
  313179:	e8 39 e9 df ff       	call   111ab7 <alloc::alloc::handle_alloc_error>
  31317e:	e9 05 01 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::Interp>::exec_parse:
  313183:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
  313188:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:318
  31318d:	4c 89 f2             	mov    %r14,%rdx
  313190:	e8 1b f4 f6 ff       	call   2825b0 <<rexx_exec::Interp>::novalue_raised>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  313195:	48 8b 6c 24 40       	mov    0x40(%rsp),%rbp
  31319a:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  31319e:	0f 85 2e c5 ff ff    	jne    30f6d2 <<rexx_exec::Interp>::exec_parse+0x2b2>
  3131a4:	e9 93 fb ff ff       	jmp    312d3c <<rexx_exec::Interp>::exec_parse+0x391c>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  3131a9:	bf 08 00 00 00       	mov    $0x8,%edi
  3131ae:	be 18 00 00 00       	mov    $0x18,%esi
  3131b3:	e8 ff e8 df ff       	call   111ab7 <alloc::alloc::handle_alloc_error>
  3131b8:	e9 cb 00 00 00       	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:703
  3131bd:	4c 8b b4 24 78 01 00 	mov    0x178(%rsp),%r14
  3131c4:	00 
  3131c5:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  3131ca:	b9 01 00 00 00       	mov    $0x1,%ecx
  3131cf:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  3131d5:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
  3131da:	48 89 ea             	mov    %rbp,%rdx
  3131dd:	e8 1e d4 e6 ff       	call   180600 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3131e2:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3131e7:	48 8b 5c 24 50       	mov    0x50(%rsp),%rbx
  3131ec:	48 89 44 24 38       	mov    %rax,0x38(%rsp)
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  3131f1:	48 8d 3c 18          	lea    (%rax,%rbx,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  3131f5:	4c 89 f6             	mov    %r14,%rsi
  3131f8:	48 89 ea             	mov    %rbp,%rdx
  3131fb:	ff 15 37 4a 1c 00    	call   *0x1c4a37(%rip)        # 4d7c38 <memcpy@GLIBC_2.14>
  313201:	e9 3c fa ff ff       	jmp    312c42 <<rexx_exec::Interp>::exec_parse+0x3822>
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  313206:	48 8d 15 73 fc 1b 00 	lea    0x1bfc73(%rip),%rdx        # 4d2e80 <rexx_api::ffi::CALL_CONTEXT+0x17e98>
  31320d:	e8 8f 95 e0 ff       	call   11c7a1 <core::panicking::panic_bounds_check>
  313212:	eb 74                	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  313214:	bf 08 00 00 00       	mov    $0x8,%edi
  313219:	be 70 00 00 00       	mov    $0x70,%esi
  31321e:	e8 94 e8 df ff       	call   111ab7 <alloc::alloc::handle_alloc_error>
  313223:	eb 63                	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  313225:	bf 01 00 00 00       	mov    $0x1,%edi
  31322a:	4c 89 e6             	mov    %r12,%rsi
  31322d:	e8 72 e8 df ff       	call   111aa4 <alloc::raw_vec::handle_error>
  313232:	eb 54                	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:81
  313234:	48 8d 3d f9 51 d5 ff 	lea    -0x2aae07(%rip),%rdi        # 68434 <panic_unwind::imp::CANARY+0x36cf>
  31323b:	48 8d 15 1e 93 1a 00 	lea    0x1a931e(%rip),%rdx        # 4bc560 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  313242:	be ab 00 00 00       	mov    $0xab,%esi
  313247:	e8 94 76 e0 ff       	call   11a8e0 <core::panicking::panic_fmt>
  31324c:	eb 3a                	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
  31324e:	48 8d 3d df 51 d5 ff 	lea    -0x2aae21(%rip),%rdi        # 68434 <panic_unwind::imp::CANARY+0x36cf>
  313255:	48 8d 15 04 93 1a 00 	lea    0x1a9304(%rip),%rdx        # 4bc560 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  31325c:	be ab 00 00 00       	mov    $0xab,%esi
  313261:	e8 7a 76 e0 ff       	call   11a8e0 <core::panicking::panic_fmt>
  313266:	eb 20                	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  313268:	bf 01 00 00 00       	mov    $0x1,%edi
  31326d:	be 34 00 00 00       	mov    $0x34,%esi
  313272:	e8 2d e8 df ff       	call   111aa4 <alloc::raw_vec::handle_error>
  313277:	eb 0f                	jmp    313288 <<rexx_exec::Interp>::exec_parse+0x3e68>
  313279:	bf 01 00 00 00       	mov    $0x1,%edi
  31327e:	be 02 00 00 00       	mov    $0x2,%esi
  313283:	e8 1c e8 df ff       	call   111aa4 <alloc::raw_vec::handle_error>
<rexx_exec::Interp>::apply_trigger:
  313288:	0f 0b                	ud2
  31328a:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31328d:	4d 85 f6             	test   %r14,%r14
  313290:	0f 84 9d 01 00 00    	je     313433 <<rexx_exec::Interp>::exec_parse+0x4013>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313296:	4c 89 e7             	mov    %r12,%rdi
  313299:	ff 15 69 49 1c 00    	call   *0x1c4969(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  31329f:	e9 8f 01 00 00       	jmp    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::exec_parse:
  3132a4:	eb 5c                	jmp    313302 <<rexx_exec::Interp>::exec_parse+0x3ee2>
  3132a6:	eb 5a                	jmp    313302 <<rexx_exec::Interp>::exec_parse+0x3ee2>
  3132a8:	49 89 c5             	mov    %rax,%r13
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3132ab:	4c 89 f7             	mov    %r14,%rdi
  3132ae:	ff 15 54 49 1c 00    	call   *0x1c4954(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  3132b4:	4c 89 bc 24 48 01 00 	mov    %r15,0x148(%rsp)
  3132bb:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1027
  3132bc:	e9 72 01 00 00       	jmp    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::exec_parse:
  3132c1:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  3132c4:	4d 85 e4             	test   %r12,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3132c7:	0f 84 10 01 00 00    	je     3133dd <<rexx_exec::Interp>::exec_parse+0x3fbd>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3132cd:	48 89 df             	mov    %rbx,%rdi
  3132d0:	ff 15 32 49 1c 00    	call   *0x1c4932(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  3132d6:	e9 02 01 00 00       	jmp    3133dd <<rexx_exec::Interp>::exec_parse+0x3fbd>
<rexx_exec::Interp>::exec_parse:
  3132db:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3132de:	48 83 7c 24 40 00    	cmpq   $0x0,0x40(%rsp)
  3132e4:	0f 84 31 01 00 00    	je     31341b <<rexx_exec::Interp>::exec_parse+0x3ffb>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3132ea:	48 8b 7c 24 48       	mov    0x48(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3132ef:	ff 15 13 49 1c 00    	call   *0x1c4913(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3132f5:	e9 21 01 00 00       	jmp    31341b <<rexx_exec::Interp>::exec_parse+0x3ffb>
<rexx_exec::Interp>::exec_parse:
  3132fa:	49 89 c5             	mov    %rax,%r13
  3132fd:	e9 cb 00 00 00       	jmp    3133cd <<rexx_exec::Interp>::exec_parse+0x3fad>
  313302:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  313305:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313308:	0f 84 69 01 00 00    	je     313477 <<rexx_exec::Interp>::exec_parse+0x4057>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31330e:	4c 89 ff             	mov    %r15,%rdi
  313311:	e9 5b 01 00 00       	jmp    313471 <<rexx_exec::Interp>::exec_parse+0x4051>
<rexx_exec::Interp>::parse_strings:
  313316:	49 89 c5             	mov    %rax,%r13
  313319:	48 89 ef             	mov    %rbp,%rdi
  31331c:	e9 50 01 00 00       	jmp    313471 <<rexx_exec::Interp>::exec_parse+0x4051>
<rexx_exec::Interp>::exec_parse:
  313321:	49 89 c5             	mov    %rax,%r13
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313324:	48 89 df             	mov    %rbx,%rdi
  313327:	e9 45 01 00 00       	jmp    313471 <<rexx_exec::Interp>::exec_parse+0x4051>
<rexx_exec::Interp>::exec_parse:
  31332c:	49 89 c5             	mov    %rax,%r13
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31332f:	48 89 df             	mov    %rbx,%rdi
  313332:	e9 0f 01 00 00       	jmp    313446 <<rexx_exec::Interp>::exec_parse+0x4026>
<rexx_exec::Interp>::exec_parse:
  313337:	e9 f4 00 00 00       	jmp    313430 <<rexx_exec::Interp>::exec_parse+0x4010>
  31333c:	eb 61                	jmp    31339f <<rexx_exec::Interp>::exec_parse+0x3f7f>
  31333e:	eb 5f                	jmp    31339f <<rexx_exec::Interp>::exec_parse+0x3f7f>
  313340:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313343:	48 83 7c 24 20 00    	cmpq   $0x0,0x20(%rsp)
  313349:	0f 85 61 c8 ff ff    	jne    30fbb0 <<rexx_exec::Interp>::exec_parse+0x790>
  31334f:	e9 23 01 00 00       	jmp    313477 <<rexx_exec::Interp>::exec_parse+0x4057>
<rexx_exec::Interp>::exec_parse:
  313354:	49 89 c5             	mov    %rax,%r13
  313357:	eb 74                	jmp    3133cd <<rexx_exec::Interp>::exec_parse+0x3fad>
  313359:	49 89 c5             	mov    %rax,%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  31335c:	e9 eb 00 00 00       	jmp    31344c <<rexx_exec::Interp>::exec_parse+0x402c>
  313361:	49 89 c5             	mov    %rax,%r13
  313364:	48 8b bc 24 f8 00 00 	mov    0xf8(%rsp),%rdi
  31336b:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31336c:	ff 15 96 48 1c 00    	call   *0x1c4896(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313372:	e9 bc 00 00 00       	jmp    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::exec_parse:
  313377:	49 89 c5             	mov    %rax,%r13
  31337a:	48 8d 7c 24 40       	lea    0x40(%rsp),%rdi
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  31337f:	e8 ac 9a e5 ff       	call   16ce30 <core::ptr::drop_glue::<rexx_exec::error::Raised>>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  313384:	eb 60                	jmp    3133e6 <<rexx_exec::Interp>::exec_parse+0x3fc6>
<rexx_exec::Interp>::apply_trigger:
  313386:	e9 a5 00 00 00       	jmp    313430 <<rexx_exec::Interp>::exec_parse+0x4010>
  31338b:	eb 35                	jmp    3133c2 <<rexx_exec::Interp>::exec_parse+0x3fa2>
  31338d:	eb 67                	jmp    3133f6 <<rexx_exec::Interp>::exec_parse+0x3fd6>
  31338f:	e9 9c 00 00 00       	jmp    313430 <<rexx_exec::Interp>::exec_parse+0x4010>
  313394:	49 89 c5             	mov    %rax,%r13
  313397:	eb 52                	jmp    3133eb <<rexx_exec::Interp>::exec_parse+0x3fcb>
  313399:	eb 04                	jmp    31339f <<rexx_exec::Interp>::exec_parse+0x3f7f>
  31339b:	eb 02                	jmp    31339f <<rexx_exec::Interp>::exec_parse+0x3f7f>
  31339d:	eb 00                	jmp    31339f <<rexx_exec::Interp>::exec_parse+0x3f7f>
  31339f:	49 89 c5             	mov    %rax,%r13
core::ptr::drop_glue::<rexx_exec::value::Carried>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3133a2:	80 7c 24 40 01       	cmpb   $0x1,0x40(%rsp)
  3133a7:	0f 85 86 00 00 00    	jne    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
  3133ad:	48 83 7c 24 48 00    	cmpq   $0x0,0x48(%rsp)
  3133b3:	74 7e                	je     313433 <<rexx_exec::Interp>::exec_parse+0x4013>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3133b5:	48 8b 7c 24 50       	mov    0x50(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3133ba:	ff 15 48 48 1c 00    	call   *0x1c4848(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3133c0:	eb 71                	jmp    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::assign_targets:
  3133c2:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3133c5:	48 83 7c 24 40 00    	cmpq   $0x0,0x40(%rsp)
  3133cb:	74 66                	je     313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::exec_parse:
  3133cd:	4c 89 ff             	mov    %r15,%rdi
  3133d0:	ff 15 32 48 1c 00    	call   *0x1c4832(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3133d6:	eb 5b                	jmp    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::exec_parse:
  3133d8:	eb 56                	jmp    313430 <<rexx_exec::Interp>::exec_parse+0x4010>
  3133da:	49 89 c5             	mov    %rax,%r13
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:89
  3133dd:	4c 89 ff             	mov    %r15,%rdi
  3133e0:	ff 15 22 48 1c 00    	call   *0x1c4822(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3133e6:	48 85 ed             	test   %rbp,%rbp
  3133e9:	7e 48                	jle    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::apply_trigger:
  3133eb:	4c 89 f7             	mov    %r14,%rdi
  3133ee:	ff 15 14 48 1c 00    	call   *0x1c4814(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3133f4:	eb 3d                	jmp    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::assign_targets:
  3133f6:	49 89 c5             	mov    %rax,%r13
  3133f9:	48 8b bc 24 f8 00 00 	mov    0xf8(%rsp),%rdi
  313400:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:792
  313401:	49 ff ce             	dec    %r14
  313404:	49 83 fe fe          	cmp    $0xfffffffffffffffe,%r14
  313408:	73 29                	jae    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31340a:	ff 15 f8 47 1c 00    	call   *0x1c47f8(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313410:	eb 21                	jmp    313433 <<rexx_exec::Interp>::exec_parse+0x4013>
<rexx_exec::Interp>::exec_parse:
  313412:	eb 1c                	jmp    313430 <<rexx_exec::Interp>::exec_parse+0x4010>
  313414:	eb 4b                	jmp    313461 <<rexx_exec::Interp>::exec_parse+0x4041>
  313416:	eb 49                	jmp    313461 <<rexx_exec::Interp>::exec_parse+0x4041>
  313418:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31341b:	48 83 bc 24 70 01 00 	cmpq   $0x0,0x170(%rsp)
  313422:	00 00 
  313424:	74 51                	je     313477 <<rexx_exec::Interp>::exec_parse+0x4057>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313426:	48 8b bc 24 78 01 00 	mov    0x178(%rsp),%rdi
  31342d:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31342e:	eb 41                	jmp    313471 <<rexx_exec::Interp>::exec_parse+0x4051>
<rexx_exec::Interp>::exec_parse:
  313430:	49 89 c5             	mov    %rax,%r13
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313433:	48 83 bc 24 48 01 00 	cmpq   $0x0,0x148(%rsp)
  31343a:	00 00 
  31343c:	7e 0e                	jle    31344c <<rexx_exec::Interp>::exec_parse+0x402c>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  31343e:	48 8b bc 24 e0 00 00 	mov    0xe0(%rsp),%rdi
  313445:	00 
  313446:	ff 15 bc 47 1c 00    	call   *0x1c47bc(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  31344c:	48 83 bc 24 c0 01 00 	cmpq   $0x0,0x1c0(%rsp)
  313453:	00 00 
core::ptr::drop_glue::<rexx_exec::parse_template::ParseStrings>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313455:	7e 20                	jle    313477 <<rexx_exec::Interp>::exec_parse+0x4057>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  313457:	48 8b bc 24 c8 01 00 	mov    0x1c8(%rsp),%rdi
  31345e:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31345f:	eb 10                	jmp    313471 <<rexx_exec::Interp>::exec_parse+0x4051>
<rexx_exec::Interp>::exec_parse:
  313461:	49 89 c5             	mov    %rax,%r13
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313464:	48 83 7c 24 20 00    	cmpq   $0x0,0x20(%rsp)
  31346a:	7e 0b                	jle    313477 <<rexx_exec::Interp>::exec_parse+0x4057>
<rexx_exec::Interp>::exec_parse:
  31346c:	48 8b 7c 24 28       	mov    0x28(%rsp),%rdi
  313471:	ff 15 91 47 1c 00    	call   *0x1c4791(%rip)        # 4d7c08 <free@GLIBC_2.2.5>
  313477:	4c 89 ef             	mov    %r13,%rdi
  31347a:	e8 31 de 19 00       	call   4b12b0 <_Unwind_Resume@plt>

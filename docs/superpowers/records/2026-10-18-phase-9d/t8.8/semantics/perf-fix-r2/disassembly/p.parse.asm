
/tmp/codex-p9d-t88-pins/8.8f-perf-r1-rexx:     file format elf64-x86-64


Disassembly of section .text:

000000000030f8c0 <<rexx_exec::Interp>::exec_parse>:
<rexx_exec::Interp>::exec_parse:
  30f8c0:	55                   	push   %rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:474
  30f8c1:	41 57                	push   %r15
  30f8c3:	41 56                	push   %r14
  30f8c5:	41 55                	push   %r13
  30f8c7:	41 54                	push   %r12
  30f8c9:	53                   	push   %rbx
  30f8ca:	48 81 ec 98 02 00 00 	sub    $0x298,%rsp
  30f8d1:	48 89 bc 24 30 01 00 	mov    %rdi,0x130(%rsp)
  30f8d8:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:480
  30f8d9:	48 8b 86 50 04 00 00 	mov    0x450(%rsi),%rax
  30f8e0:	48 89 44 24 50       	mov    %rax,0x50(%rsp)
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:564
  30f8e5:	48 8b 41 18          	mov    0x18(%rcx),%rax
  30f8e9:	48 bf f1 ff ff ff ff 	movabs $0x7ffffffffffffff1,%rdi
  30f8f0:	ff ff 7f 
  30f8f3:	48 01 c7             	add    %rax,%rdi
  30f8f6:	48 83 ff 05          	cmp    $0x5,%rdi
  30f8fa:	48 89 74 24 18       	mov    %rsi,0x18(%rsp)
  30f8ff:	48 89 94 24 10 01 00 	mov    %rdx,0x110(%rsp)
  30f906:	00 
  30f907:	48 89 4c 24 30       	mov    %rcx,0x30(%rsp)
  30f90c:	77 1e                	ja     30f92c <<rexx_exec::Interp>::exec_parse+0x6c>
  30f90e:	48 8d 05 ff 4a d4 ff 	lea    -0x2bb501(%rip),%rax        # 54414 <GCC_except_table5014+0x4a30>
  30f915:	48 63 3c b8          	movslq (%rax,%rdi,4),%rdi
  30f919:	48 01 c7             	add    %rax,%rdi
  30f91c:	ff e7                	jmp    *%rdi
  30f91e:	49 c7 c7 fe ff ff ff 	mov    $0xfffffffffffffffe,%r15
  30f925:	31 ed                	xor    %ebp,%ebp
  30f927:	e9 95 0d 00 00       	jmp    3106c1 <<rexx_exec::Interp>::exec_parse+0xe01>
  30f92c:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
  30f930:	0f 85 fe 00 00 00    	jne    30fa34 <<rexx_exec::Interp>::exec_parse+0x174>
  30f936:	bd 01 00 00 00       	mov    $0x1,%ebp
  30f93b:	4c 8d 2d bf 72 d9 ff 	lea    -0x268d41(%rip),%r13        # a6c01 <panic_unwind::imp::CANARY+0x41ea4>
  30f942:	bb 05 00 00 00       	mov    $0x5,%ebx
  30f947:	45 31 ff             	xor    %r15d,%r15d
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30f94a:	80 be 35 05 00 00 00 	cmpb   $0x0,0x535(%rsi)
  30f951:	75 20                	jne    30f973 <<rexx_exec::Interp>::exec_parse+0xb3>
  30f953:	f6 86 2e 05 00 00 01 	testb  $0x1,0x52e(%rsi)
  30f95a:	74 17                	je     30f973 <<rexx_exec::Interp>::exec_parse+0xb3>
<rexx_exec::Interp>::trace_literal:
  30f95c:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:342
  30f95f:	ba 01 00 00 00       	mov    $0x1,%edx
  30f964:	48 89 f7             	mov    %rsi,%rdi
  30f967:	48 8b 74 24 50       	mov    0x50(%rsp),%rsi
  30f96c:	31 c9                	xor    %ecx,%ecx
  30f96e:	e8 8d 1a f7 ff       	call   281400 <<rexx_exec::Interp>::trace_literal_line>
<rexx_exec::Interp>::parse_strings:
  30f973:	45 31 e4             	xor    %r12d,%r12d
  30f976:	e9 da 03 00 00       	jmp    30fd55 <<rexx_exec::Interp>::exec_parse+0x495>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::pop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2873
  30f97b:	48 8b 86 e0 0b 00 00 	mov    0xbe0(%rsi),%rax
  30f982:	48 85 c0             	test   %rax,%rax
  30f985:	0f 84 e8 01 00 00    	je     30fb73 <<rexx_exec::Interp>::exec_parse+0x2b3>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2877
  30f98b:	48 ff c8             	dec    %rax
  30f98e:	48 89 86 e0 0b 00 00 	mov    %rax,0xbe0(%rsi)
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f995:	48 8b 8e d8 0b 00 00 	mov    0xbd8(%rsi),%rcx
<*const alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs:871
  30f99c:	48 8d 04 40          	lea    (%rax,%rax,2),%rax
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30f9a0:	4c 8b 3c c1          	mov    (%rcx,%rax,8),%r15
  30f9a4:	f3 0f 6f 44 c1 08    	movdqu 0x8(%rcx,%rax,8),%xmm0
  30f9aa:	f3 0f 7f 44 24 68    	movdqu %xmm0,0x68(%rsp)
<alloc::vec::Vec<u8>>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1461
  30f9b0:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  30f9b5:	e9 d1 01 00 00       	jmp    30fb8b <<rexx_exec::Interp>::exec_parse+0x2cb>
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:1784
  30f9ba:	48 8b 86 20 11 00 00 	mov    0x1120(%rsi),%rax
  30f9c1:	48 85 c0             	test   %rax,%rax
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::pop_front:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2198
  30f9c4:	0f 84 3f 01 00 00    	je     30fb09 <<rexx_exec::Interp>::exec_parse+0x249>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2201
  30f9ca:	48 8b 96 18 11 00 00 	mov    0x1118(%rsi),%rdx
  30f9d1:	49 89 f1             	mov    %rsi,%r9
<usize>::wrapping_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2652
  30f9d4:	48 8d 72 01          	lea    0x1(%rdx),%rsi
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30f9d8:	49 8b b9 08 11 00 00 	mov    0x1108(%r9),%rdi
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30f9df:	49 8b 89 10 11 00 00 	mov    0x1110(%r9),%rcx
  30f9e6:	45 31 c0             	xor    %r8d,%r8d
alloc::collections::vec_deque::index::wrap_index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:3700
  30f9e9:	48 39 fe             	cmp    %rdi,%rsi
  30f9ec:	4c 0f 43 c7          	cmovae %rdi,%r8
  30f9f0:	49 f7 d8             	neg    %r8
  30f9f3:	4a 8d 34 02          	lea    (%rdx,%r8,1),%rsi
  30f9f7:	48 ff c6             	inc    %rsi
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::pop_front:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2202
  30f9fa:	49 89 b1 18 11 00 00 	mov    %rsi,0x1118(%r9)
  30fa01:	4c 89 ce             	mov    %r9,%rsi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2203
  30fa04:	48 ff c8             	dec    %rax
  30fa07:	49 89 81 20 11 00 00 	mov    %rax,0x1120(%r9)
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30fa0e:	48 8d 04 52          	lea    (%rdx,%rdx,2),%rax
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30fa12:	4c 8b 3c c1          	mov    (%rcx,%rax,8),%r15
<rexx_exec::Interp>::pull_line:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:576
  30fa16:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fa1a:	0f 84 e9 00 00 00    	je     30fb09 <<rexx_exec::Interp>::exec_parse+0x249>
  30fa20:	48 8d 04 c1          	lea    (%rcx,%rax,8),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:577
  30fa24:	f3 0f 6f 40 08       	movdqu 0x8(%rax),%xmm0
  30fa29:	f3 0f 7f 44 24 68    	movdqu %xmm0,0x68(%rsp)
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fa2f:	e9 ee 00 00 00       	jmp    30fb22 <<rexx_exec::Interp>::exec_parse+0x262>
<rexx_exec::Interp>::parse_strings:
  30fa34:	4c 8d 2d c6 71 d9 ff 	lea    -0x268e3a(%rip),%r13        # a6c01 <panic_unwind::imp::CANARY+0x41ea4>
  30fa3b:	bb 05 00 00 00       	mov    $0x5,%ebx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:577
  30fa40:	41 f6 c0 01          	test   $0x1,%r8b
  30fa44:	0f 84 f3 00 00 00    	je     30fb3d <<rexx_exec::Interp>::exec_parse+0x27d>
  30fa4a:	4d 89 ce             	mov    %r9,%r14
  30fa4d:	e9 40 09 00 00       	jmp    310392 <<rexx_exec::Interp>::exec_parse+0xad2>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::pop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2873
  30fa52:	48 8b 86 e0 0b 00 00 	mov    0xbe0(%rsi),%rax
  30fa59:	48 85 c0             	test   %rax,%rax
  30fa5c:	0f 84 8d 01 00 00    	je     30fbef <<rexx_exec::Interp>::exec_parse+0x32f>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2877
  30fa62:	48 ff c8             	dec    %rax
  30fa65:	48 89 86 e0 0b 00 00 	mov    %rax,0xbe0(%rsi)
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fa6c:	48 8b 8e d8 0b 00 00 	mov    0xbd8(%rsi),%rcx
<*const alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs:871
  30fa73:	48 8d 14 40          	lea    (%rax,%rax,2),%rdx
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30fa77:	48 8b 04 d1          	mov    (%rcx,%rdx,8),%rax
  30fa7b:	f3 0f 6f 44 d1 08    	movdqu 0x8(%rcx,%rdx,8),%xmm0
  30fa81:	f3 0f 7f 44 24 68    	movdqu %xmm0,0x68(%rsp)
<alloc::vec::Vec<u8>>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1461
  30fa87:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  30fa8c:	e9 75 01 00 00       	jmp    30fc06 <<rexx_exec::Interp>::exec_parse+0x346>
<rexx_exec::Interp>::parse_strings:
  30fa91:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:630
  30fa96:	e8 45 68 04 00       	call   3562e0 <<rexx_exec::Interp>::linein_line>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fa9b:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
  30faa0:	48 8b 6c 24 68       	mov    0x68(%rsp),%rbp
  30faa5:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  30faaa:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30faae:	0f 84 b2 0b 00 00    	je     310666 <<rexx_exec::Interp>::exec_parse+0xda6>
  30fab4:	4c 8d 2d 62 71 d9 ff 	lea    -0x268e9e(%rip),%r13        # a6c1d <panic_unwind::imp::CANARY+0x41ec0>
  30fabb:	bb 06 00 00 00       	mov    $0x6,%ebx
  30fac0:	e9 90 02 00 00       	jmp    30fd55 <<rexx_exec::Interp>::exec_parse+0x495>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:591
  30fac5:	8b 59 20             	mov    0x20(%rcx),%ebx
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:673
  30fac8:	48 8b 42 08          	mov    0x8(%rdx),%rax
<alloc::vec::Vec<alloc::boxed::Box<str>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  30facc:	48 8b 70 10          	mov    0x10(%rax),%rsi
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  30fad0:	48 39 de             	cmp    %rbx,%rsi
  30fad3:	0f 86 47 05 00 00    	jbe    310020 <<rexx_exec::Interp>::exec_parse+0x760>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::boxed::Box<str>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fad9:	48 8b 40 08          	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  30fadd:	48 89 d9             	mov    %rbx,%rcx
  30fae0:	48 c1 e1 04          	shl    $0x4,%rcx
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  30fae4:	4c 8b 2c 08          	mov    (%rax,%rcx,1),%r13
  30fae8:	4c 8b 64 08 08       	mov    0x8(%rax,%rcx,1),%r12
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  30faed:	4d 85 e4             	test   %r12,%r12
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:25
  30faf0:	0f 84 07 06 00 00    	je     3100fd <<rexx_exec::Interp>::exec_parse+0x83d>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  30faf6:	49 83 fc 04          	cmp    $0x4,%r12
  30fafa:	0f 83 d6 02 00 00    	jae    30fdd6 <<rexx_exec::Interp>::exec_parse+0x516>
<rexx_exec::Interp>::parse_strings:
  30fb00:	31 c0                	xor    %eax,%eax
  30fb02:	31 c9                	xor    %ecx,%ecx
  30fb04:	e9 55 05 00 00       	jmp    31005e <<rexx_exec::Interp>::exec_parse+0x79e>
  30fb09:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<rexx_exec::Interp>::pull_line:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:578
  30fb0e:	e8 cd 67 04 00       	call   3562e0 <<rexx_exec::Interp>::linein_line>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fb13:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
  30fb18:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fb1c:	0f 84 44 03 00 00    	je     30fe66 <<rexx_exec::Interp>::exec_parse+0x5a6>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  30fb22:	48 8b 6c 24 68       	mov    0x68(%rsp),%rbp
  30fb27:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  30fb2c:	4c 8d 2d 6d 2f d4 ff 	lea    -0x2bd093(%rip),%r13        # 52aa0 <GCC_except_table5014+0x30bc>
  30fb33:	bb 04 00 00 00       	mov    $0x4,%ebx
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:629
  30fb38:	e9 18 02 00 00       	jmp    30fd55 <<rexx_exec::Interp>::exec_parse+0x495>
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:127
  30fb3d:	c6 84 24 b8 01 00 00 	movb   $0x0,0x1b8(%rsp)
  30fb44:	00 
  30fb45:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  30fb4a:	48 8d 94 24 b8 01 00 	lea    0x1b8(%rsp),%rdx
  30fb51:	00 
  30fb52:	49 89 f4             	mov    %rsi,%r12
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:128
  30fb55:	e8 36 cd f6 ff       	call   27c890 <<rexx_exec::Interp>::enter_eval_node::<u8>>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fb5a:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
  30fb5f:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30fb63:	0f 84 0c 03 00 00    	je     30fe75 <<rexx_exec::Interp>::exec_parse+0x5b5>
<rexx_exec::Interp>::parse_strings:
  30fb69:	4c 8b 64 24 68       	mov    0x68(%rsp),%r12
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fb6e:	e9 f3 0a 00 00       	jmp    310666 <<rexx_exec::Interp>::exec_parse+0xda6>
<alloc::vec::Vec<u8>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:462
  30fb73:	48 c7 44 24 68 01 00 	movq   $0x1,0x68(%rsp)
  30fb7a:	00 00 
  30fb7c:	48 c7 44 24 70 00 00 	movq   $0x0,0x70(%rsp)
  30fb83:	00 00 
  30fb85:	45 31 e4             	xor    %r12d,%r12d
  30fb88:	45 31 ff             	xor    %r15d,%r15d
<core::option::Option<alloc::vec::Vec<u8>>>::unwrap_or_default:
  30fb8b:	4c 89 7c 24 60       	mov    %r15,0x60(%rsp)
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fb90:	4c 89 f8             	mov    %r15,%rax
  30fb93:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fb96:	48 83 f8 2c          	cmp    $0x2c,%rax
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fb9a:	0f 86 b7 03 00 00    	jbe    30ff57 <<rexx_exec::Interp>::exec_parse+0x697>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fba0:	4c 8b 74 24 68       	mov    0x68(%rsp),%r14
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fba5:	0f 10 05 7b 94 d5 ff 	movups -0x2a6b85(%rip),%xmm0        # 69027 <panic_unwind::imp::CANARY+0x42ca>
  30fbac:	43 0f 11 44 26 1d    	movups %xmm0,0x1d(%r14,%r12,1)
  30fbb2:	0f 10 05 61 94 d5 ff 	movups -0x2a6b9f(%rip),%xmm0        # 6901a <panic_unwind::imp::CANARY+0x42bd>
  30fbb9:	43 0f 11 44 26 10    	movups %xmm0,0x10(%r14,%r12,1)
  30fbbf:	f3 0f 6f 05 43 94 d5 	movdqu -0x2a6bbd(%rip),%xmm0        # 6900a <panic_unwind::imp::CANARY+0x42ad>
  30fbc6:	ff 
  30fbc7:	f3 43 0f 7f 04 26    	movdqu %xmm0,(%r14,%r12,1)
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fbcd:	49 83 c4 2d          	add    $0x2d,%r12
  30fbd1:	bb 07 00 00 00       	mov    $0x7,%ebx
  30fbd6:	4c 8d 2d 59 70 d9 ff 	lea    -0x268fa7(%rip),%r13        # a6c36 <panic_unwind::imp::CANARY+0x41ed9>
  30fbdd:	4c 89 f5             	mov    %r14,%rbp
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:632
  30fbe0:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fbe4:	0f 85 6b 01 00 00    	jne    30fd55 <<rexx_exec::Interp>::exec_parse+0x495>
  30fbea:	e9 a3 07 00 00       	jmp    310392 <<rexx_exec::Interp>::exec_parse+0xad2>
<alloc::vec::Vec<u8>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:462
  30fbef:	48 c7 44 24 68 01 00 	movq   $0x1,0x68(%rsp)
  30fbf6:	00 00 
  30fbf8:	48 c7 44 24 70 00 00 	movq   $0x0,0x70(%rsp)
  30fbff:	00 00 
  30fc01:	45 31 e4             	xor    %r12d,%r12d
  30fc04:	31 c0                	xor    %eax,%eax
<core::option::Option<alloc::vec::Vec<u8>>>::unwrap_or_default:
  30fc06:	48 89 44 24 60       	mov    %rax,0x60(%rsp)
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fc0b:	48 89 c1             	mov    %rax,%rcx
  30fc0e:	4c 29 e1             	sub    %r12,%rcx
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fc11:	48 83 f9 04          	cmp    $0x4,%rcx
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fc15:	0f 86 68 03 00 00    	jbe    30ff83 <<rexx_exec::Interp>::exec_parse+0x6c3>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fc1b:	4c 8b 7c 24 68       	mov    0x68(%rsp),%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fc20:	43 c6 44 27 04 58    	movb   $0x58,0x4(%r15,%r12,1)
  30fc26:	43 c7 04 27 4c 49 4e 	movl   $0x554e494c,(%r15,%r12,1)
  30fc2d:	55 
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fc2e:	49 8d 4c 24 05       	lea    0x5(%r12),%rcx
  30fc33:	48 89 4c 24 70       	mov    %rcx,0x70(%rsp)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30fc38:	48 39 c1             	cmp    %rax,%rcx
  30fc3b:	75 0f                	jne    30fc4c <<rexx_exec::Interp>::exec_parse+0x38c>
  30fc3d:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30fc42:	e8 39 22 e0 ff       	call   111e80 <<alloc::raw_vec::RawVec<u8>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fc47:	4c 8b 7c 24 68       	mov    0x68(%rsp),%r15
core::ptr::write::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30fc4c:	43 c6 44 27 05 20    	movb   $0x20,0x5(%r15,%r12,1)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30fc52:	49 83 c4 06          	add    $0x6,%r12
  30fc56:	4c 89 64 24 70       	mov    %r12,0x70(%rsp)
  30fc5b:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30fc60:	48 8b 80 70 04 00 00 	mov    0x470(%rax),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  30fc67:	48 85 c0             	test   %rax,%rax
  30fc6a:	0f 84 ca 02 00 00    	je     30ff3a <<rexx_exec::Interp>::exec_parse+0x67a>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30fc70:	0f b6 80 d6 01 00 00 	movzbl 0x1d6(%rax),%eax
<rexx_exec::activation::CallType>::token:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:472
  30fc77:	48 8d 0d c2 9d d9 ff 	lea    -0x26623e(%rip),%rcx        # a9a40 <core::unicode::unicode_data::white_space::WHITESPACE_MAP+0x16c2>
  30fc7e:	0f b6 1c 08          	movzbl (%rax,%rcx,1),%ebx
  30fc82:	4c 8d 35 bf 9d d9 ff 	lea    -0x266241(%rip),%r14        # a9a48 <core::unicode::unicode_data::white_space::WHITESPACE_MAP+0x16ca>
  30fc89:	4d 63 2c 86          	movslq (%r14,%rax,4),%r13
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30fc8d:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fc92:	48 89 e8             	mov    %rbp,%rax
  30fc95:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fc98:	48 39 d8             	cmp    %rbx,%rax
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fc9b:	0f 82 0e 03 00 00    	jb     30ffaf <<rexx_exec::Interp>::exec_parse+0x6ef>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30fca1:	4d 01 ee             	add    %r13,%r14
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30fca4:	4b 8d 3c 27          	lea    (%r15,%r12,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fca8:	4c 89 f6             	mov    %r14,%rsi
  30fcab:	48 89 da             	mov    %rbx,%rdx
  30fcae:	ff 15 14 83 1c 00    	call   *0x1c8314(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fcb4:	49 01 dc             	add    %rbx,%r12
  30fcb7:	4c 89 64 24 70       	mov    %r12,0x70(%rsp)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30fcbc:	49 39 ec             	cmp    %rbp,%r12
  30fcbf:	75 0f                	jne    30fcd0 <<rexx_exec::Interp>::exec_parse+0x410>
  30fcc1:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30fcc6:	e8 b5 21 e0 ff       	call   111e80 <<alloc::raw_vec::RawVec<u8>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fccb:	4c 8b 7c 24 68       	mov    0x68(%rsp),%r15
core::ptr::write::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30fcd0:	43 c6 04 27 20       	movb   $0x20,(%r15,%r12,1)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30fcd5:	49 ff c4             	inc    %r12
  30fcd8:	4c 89 64 24 70       	mov    %r12,0x70(%rsp)
  30fcdd:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:610
  30fce2:	48 8b 87 70 04 00 00 	mov    0x470(%rdi),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  30fce9:	48 85 c0             	test   %rax,%rax
  30fcec:	0f 84 48 02 00 00    	je     30ff3a <<rexx_exec::Interp>::exec_parse+0x67a>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:610
  30fcf2:	48 8b b0 f0 00 00 00 	mov    0xf0(%rax),%rsi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:611
  30fcf9:	e8 c2 c2 ea ff       	call   1bbfc0 <<rexx_exec::Interp>::program_display_name>
  30fcfe:	49 89 c5             	mov    %rax,%r13
  30fd01:	48 89 d3             	mov    %rdx,%rbx
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30fd04:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fd09:	4c 89 f8             	mov    %r15,%rax
  30fd0c:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fd0f:	48 39 c2             	cmp    %rax,%rdx
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fd12:	0f 87 c6 02 00 00    	ja     30ffde <<rexx_exec::Interp>::exec_parse+0x71e>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2981
  30fd18:	48 85 db             	test   %rbx,%rbx
  30fd1b:	74 17                	je     30fd34 <<rexx_exec::Interp>::exec_parse+0x474>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fd1d:	4c 8b 74 24 68       	mov    0x68(%rsp),%r14
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30fd22:	4b 8d 3c 26          	lea    (%r14,%r12,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fd26:	4c 89 ee             	mov    %r13,%rsi
  30fd29:	48 89 da             	mov    %rbx,%rdx
  30fd2c:	ff 15 96 82 1c 00    	call   *0x1c8296(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
  30fd32:	eb 05                	jmp    30fd39 <<rexx_exec::Interp>::exec_parse+0x479>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:612
  30fd34:	4c 8b 74 24 68       	mov    0x68(%rsp),%r14
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fd39:	49 01 dc             	add    %rbx,%r12
  30fd3c:	bb 06 00 00 00       	mov    $0x6,%ebx
  30fd41:	4c 8d 2d e0 6e d9 ff 	lea    -0x269120(%rip),%r13        # a6c28 <panic_unwind::imp::CANARY+0x41ecb>
  30fd48:	4c 89 f5             	mov    %r14,%rbp
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:632
  30fd4b:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fd4f:	0f 84 3d 06 00 00    	je     310392 <<rexx_exec::Interp>::exec_parse+0xad2>
  30fd55:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30fd5a:	80 b8 35 05 00 00 00 	cmpb   $0x0,0x535(%rax)
  30fd61:	75 64                	jne    30fdc7 <<rexx_exec::Interp>::exec_parse+0x507>
  30fd63:	f6 80 2d 05 00 00 01 	testb  $0x1,0x52d(%rax)
  30fd6a:	74 5b                	je     30fdc7 <<rexx_exec::Interp>::exec_parse+0x507>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30fd6c:	48 8d b8 d8 10 00 00 	lea    0x10d8(%rax),%rdi
  30fd73:	4c 8b b0 e8 10 00 00 	mov    0x10e8(%rax),%r14
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:298
  30fd7a:	4c 89 64 24 10       	mov    %r12,0x10(%rsp)
  30fd7f:	48 89 6c 24 08       	mov    %rbp,0x8(%rsp)
  30fd84:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  30fd8b:	48 8d 35 eb ad d5 ff 	lea    -0x2a5215(%rip),%rsi        # 6ab7d <panic_unwind::imp::CANARY+0x5e20>
  30fd92:	48 8b 54 24 50       	mov    0x50(%rsp),%rdx
  30fd97:	b9 01 00 00 00       	mov    $0x1,%ecx
  30fd9c:	4d 89 e8             	mov    %r13,%r8
  30fd9f:	49 89 d9             	mov    %rbx,%r9
  30fda2:	e8 49 0a f7 ff       	call   2807f0 <rexx_exec::trace::format::push_tagged>
<[u8] as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:21
  30fda7:	83 fb 04             	cmp    $0x4,%ebx
  30fdaa:	75 0e                	jne    30fdba <<rexx_exec::Interp>::exec_parse+0x4fa>
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  30fdac:	41 81 7d 00 57 48 45 	cmpl   $0x4e454857,0x0(%r13)
  30fdb3:	4e 
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:307
  30fdb4:	0f 84 4e 02 00 00    	je     310008 <<rexx_exec::Interp>::exec_parse+0x748>
  30fdba:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:310
  30fdbf:	4c 89 f6             	mov    %r14,%rsi
  30fdc2:	e8 89 e3 e8 ff       	call   19e150 <<rexx_exec::Interp>::route_trace_line>
<core::result::Result<rexx_exec::parse_template::ParseStrings, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fdc7:	49 83 ff fd          	cmp    $0xfffffffffffffffd,%r15
  30fdcb:	0f 84 95 08 00 00    	je     310666 <<rexx_exec::Interp>::exec_parse+0xda6>
  30fdd1:	e9 eb 08 00 00       	jmp    3106c1 <<rexx_exec::Interp>::exec_parse+0xe01>
<rexx_exec::Interp>::exec_parse:
  30fdd6:	4c 89 e0             	mov    %r12,%rax
  30fdd9:	48 83 e0 fc          	and    $0xfffffffffffffffc,%rax
  30fddd:	66 0f ef c0          	pxor   %xmm0,%xmm0
  30fde1:	31 c9                	xor    %ecx,%ecx
  30fde3:	66 0f 6f 0d 75 01 d4 	movdqa -0x2bfe8b(%rip),%xmm1        # 4ff60 <GCC_except_table5014+0x57c>
  30fdea:	ff 
  30fdeb:	66 0f 6f 1d ed 05 d4 	movdqa -0x2bfa13(%rip),%xmm3        # 503e0 <GCC_except_table5014+0x9fc>
  30fdf2:	ff 
  30fdf3:	66 0f ef d2          	pxor   %xmm2,%xmm2
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
  30fdf7:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  30fdfe:	00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:274
  30fe00:	41 0f b7 54 0d 00    	movzwl 0x0(%r13,%rcx,1),%edx
  30fe06:	66 0f 6e e2          	movd   %edx,%xmm4
  30fe0a:	41 0f b7 54 0d 02    	movzwl 0x2(%r13,%rcx,1),%edx
  30fe10:	66 0f 6e ea          	movd   %edx,%xmm5
<core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize::<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/filter.rs:138
  30fe14:	66 0f 74 e1          	pcmpeqb %xmm1,%xmm4
  30fe18:	66 0f 60 e4          	punpcklbw %xmm4,%xmm4
  30fe1c:	66 0f 61 e4          	punpcklwd %xmm4,%xmm4
  30fe20:	66 0f 70 e4 d4       	pshufd $0xd4,%xmm4,%xmm4
  30fe25:	66 0f db e3          	pand   %xmm3,%xmm4
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  30fe29:	66 0f d4 d4          	paddq  %xmm4,%xmm2
<core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize::<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/filter.rs:138
  30fe2d:	66 0f 74 e9          	pcmpeqb %xmm1,%xmm5
  30fe31:	66 0f 60 ed          	punpcklbw %xmm5,%xmm5
  30fe35:	66 0f 61 ed          	punpcklwd %xmm5,%xmm5
  30fe39:	66 0f 70 e5 d4       	pshufd $0xd4,%xmm5,%xmm4
  30fe3e:	66 0f db e3          	pand   %xmm3,%xmm4
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  30fe42:	66 0f d4 c4          	paddq  %xmm4,%xmm0
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
  30fe46:	48 83 c1 04          	add    $0x4,%rcx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  30fe4a:	48 39 c8             	cmp    %rcx,%rax
  30fe4d:	75 b1                	jne    30fe00 <<rexx_exec::Interp>::exec_parse+0x540>
  30fe4f:	66 0f d4 c2          	paddq  %xmm2,%xmm0
  30fe53:	66 0f 70 c8 ee       	pshufd $0xee,%xmm0,%xmm1
  30fe58:	66 0f d4 c8          	paddq  %xmm0,%xmm1
  30fe5c:	66 48 0f 7e c9       	movq   %xmm1,%rcx
  30fe61:	e9 09 02 00 00       	jmp    31006f <<rexx_exec::Interp>::exec_parse+0x7af>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  30fe66:	48 8b 6c 24 68       	mov    0x68(%rsp),%rbp
  30fe6b:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  30fe70:	e9 f1 07 00 00       	jmp    310666 <<rexx_exec::Interp>::exec_parse+0xda6>
<rexx_exec::Interp>::parse_strings:
  30fe75:	48 8b 44 24 30       	mov    0x30(%rsp),%rax
  30fe7a:	4c 8d 78 18          	lea    0x18(%rax),%r15
  30fe7e:	48 8d bc 24 f8 00 00 	lea    0xf8(%rsp),%rdi
  30fe85:	00 
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:129
  30fe86:	4c 89 e6             	mov    %r12,%rsi
  30fe89:	48 8b 94 24 10 01 00 	mov    0x110(%rsp),%rdx
  30fe90:	00 
  30fe91:	4c 89 f9             	mov    %r15,%rcx
  30fe94:	e8 57 cc f6 ff       	call   27caf0 <<rexx_exec::Interp>::eval_node>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:130
  30fe99:	49 ff 8c 24 c8 04 00 	decq   0x4c8(%r12)
  30fea0:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:131
  30fea1:	48 8b ac 24 f8 00 00 	mov    0xf8(%rsp),%rbp
  30fea8:	00 
  30fea9:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30fead:	74 0d                	je     30febc <<rexx_exec::Interp>::exec_parse+0x5fc>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  30feaf:	4c 8b a4 24 00 01 00 	mov    0x100(%rsp),%r12
  30feb6:	00 
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30feb7:	e9 aa 07 00 00       	jmp    310666 <<rexx_exec::Interp>::exec_parse+0xda6>
<rexx_exec::Interp>::parse_strings:
  30febc:	4c 89 e1             	mov    %r12,%rcx
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  30febf:	41 f6 84 24 36 05 00 	testb  $0x4,0x536(%r12)
  30fec6:	00 04 
<rexx_exec::Interp>::eval_in_parse:
  30fec8:	4c 8b b4 24 00 01 00 	mov    0x100(%rsp),%r14
  30fecf:	00 
<rexx_exec::Interp>::trace_intermediate:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:161
  30fed0:	74 2d                	je     30feff <<rexx_exec::Interp>::exec_parse+0x63f>
  30fed2:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:164
  30fed7:	4c 89 e6             	mov    %r12,%rsi
  30feda:	48 8b 94 24 10 01 00 	mov    0x110(%rsp),%rdx
  30fee1:	00 
  30fee2:	4c 89 f9             	mov    %r15,%rcx
  30fee5:	4d 89 f0             	mov    %r14,%r8
  30fee8:	e8 23 f7 f6 ff       	call   27f610 <<rexx_exec::Interp>::trace_intermediate_event>
  30feed:	4c 89 e1             	mov    %r12,%rcx
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fef0:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
  30fef5:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30fef9:	0f 85 6a fc ff ff    	jne    30fb69 <<rexx_exec::Interp>::exec_parse+0x2a9>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  30feff:	4c 8b b9 f0 0a 00 00 	mov    0xaf0(%rcx),%r15
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30ff06:	4c 3b b9 e0 0a 00 00 	cmp    0xae0(%rcx),%r15
  30ff0d:	74 1a                	je     30ff29 <<rexx_exec::Interp>::exec_parse+0x669>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30ff0f:	48 8b 81 e8 0a 00 00 	mov    0xae8(%rcx),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30ff16:	4e 89 34 f8          	mov    %r14,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30ff1a:	49 ff c7             	inc    %r15
  30ff1d:	4c 89 b9 f0 0a 00 00 	mov    %r15,0xaf0(%rcx)
  30ff24:	e9 69 04 00 00       	jmp    310392 <<rexx_exec::Interp>::exec_parse+0xad2>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30ff29:	48 8d b9 e0 0a 00 00 	lea    0xae0(%rcx),%rdi
  30ff30:	e8 db bf e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  30ff35:	4c 89 e1             	mov    %r12,%rcx
  30ff38:	eb d5                	jmp    30ff0f <<rexx_exec::Interp>::exec_parse+0x64f>
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  30ff3a:	48 8d 3d 19 9f d5 ff 	lea    -0x2a60e7(%rip),%rdi        # 69e5a <panic_unwind::imp::CANARY+0x50fd>
  30ff41:	48 8d 15 60 02 1b 00 	lea    0x1b0260(%rip),%rdx        # 4c01a8 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  30ff48:	be 11 00 00 00       	mov    $0x11,%esi
  30ff4d:	e8 be f0 e0 ff       	call   11f010 <core::option::expect_failed>
  30ff52:	e9 aa 37 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::parse_strings:
  30ff57:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30ff5c:	ba 2d 00 00 00       	mov    $0x2d,%edx
  30ff61:	b9 01 00 00 00       	mov    $0x1,%ecx
  30ff66:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30ff6c:	4c 89 e6             	mov    %r12,%rsi
  30ff6f:	e8 ec 06 e7 ff       	call   180660 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:617
  30ff74:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30ff79:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  30ff7e:	e9 1d fc ff ff       	jmp    30fba0 <<rexx_exec::Interp>::exec_parse+0x2e0>
<rexx_exec::Interp>::parse_strings:
  30ff83:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30ff88:	ba 05 00 00 00       	mov    $0x5,%edx
  30ff8d:	b9 01 00 00 00       	mov    $0x1,%ecx
  30ff92:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30ff98:	4c 89 e6             	mov    %r12,%rsi
  30ff9b:	e8 c0 06 e7 ff       	call   180660 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30ffa0:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30ffa5:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  30ffaa:	e9 6c fc ff ff       	jmp    30fc1b <<rexx_exec::Interp>::exec_parse+0x35b>
<rexx_exec::Interp>::parse_strings:
  30ffaf:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30ffb4:	b9 01 00 00 00       	mov    $0x1,%ecx
  30ffb9:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30ffbf:	4c 89 e6             	mov    %r12,%rsi
  30ffc2:	48 89 da             	mov    %rbx,%rdx
  30ffc5:	e8 96 06 e7 ff       	call   180660 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30ffca:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30ffcf:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30ffd4:	4c 8b 7c 24 68       	mov    0x68(%rsp),%r15
  30ffd9:	e9 c3 fc ff ff       	jmp    30fca1 <<rexx_exec::Interp>::exec_parse+0x3e1>
<rexx_exec::Interp>::parse_strings:
  30ffde:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30ffe3:	b9 01 00 00 00       	mov    $0x1,%ecx
  30ffe8:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30ffee:	4c 89 e6             	mov    %r12,%rsi
  30fff1:	48 89 da             	mov    %rbx,%rdx
  30fff4:	e8 67 06 e7 ff       	call   180660 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:612
  30fff9:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30fffe:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  310003:	e9 15 fd ff ff       	jmp    30fd1d <<rexx_exec::Interp>::exec_parse+0x45d>
<rexx_exec::Interp>::parse_strings:
  310008:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:308
  31000d:	4c 89 f6             	mov    %r14,%rsi
  310010:	48 89 ea             	mov    %rbp,%rdx
  310013:	4c 89 e1             	mov    %r12,%rcx
  310016:	e8 e5 95 00 00       	call   319600 <<rexx_exec::Interp>::route_when_line>
  31001b:	e9 a7 fd ff ff       	jmp    30fdc7 <<rexx_exec::Interp>::exec_parse+0x507>
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  310020:	48 8d 15 e9 31 1c 00 	lea    0x1c31e9(%rip),%rdx        # 4d3210 <rexx_api::ffi::CALL_CONTEXT+0x17e80>
  310027:	48 89 df             	mov    %rbx,%rdi
  31002a:	e8 d2 c7 e0 ff       	call   11c801 <core::panicking::panic_bounds_check>
<rexx_exec::Interp>::parse_strings:
  31002f:	eb 14                	jmp    310045 <<rexx_exec::Interp>::exec_parse+0x785>
  310031:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  310034:	4d 85 ff             	test   %r15,%r15
  310037:	0f 84 e1 38 00 00    	je     31391e <<rexx_exec::Interp>::exec_parse+0x405e>
<rexx_exec::Interp>::parse_strings:
  31003d:	48 89 ef             	mov    %rbp,%rdi
  310040:	e9 d3 38 00 00       	jmp    313918 <<rexx_exec::Interp>::exec_parse+0x4058>
  310045:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  310048:	48 83 7c 24 60 00    	cmpq   $0x0,0x60(%rsp)
  31004e:	0f 84 ca 38 00 00    	je     31391e <<rexx_exec::Interp>::exec_parse+0x405e>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  310054:	48 8b 7c 24 68       	mov    0x68(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  310059:	e9 ba 38 00 00       	jmp    313918 <<rexx_exec::Interp>::exec_parse+0x4058>
rexx_exec::run::shape_of::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:3777
  31005e:	31 d2                	xor    %edx,%edx
  310060:	41 80 7c 05 00 2e    	cmpb   $0x2e,0x0(%r13,%rax,1)
  310066:	0f 94 c2             	sete   %dl
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  310069:	48 01 d1             	add    %rdx,%rcx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  31006c:	48 ff c0             	inc    %rax
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  31006f:	49 39 c4             	cmp    %rax,%r12
  310072:	75 ea                	jne    31005e <<rexx_exec::Interp>::exec_parse+0x79e>
rexx_exec::run::shape_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:3778
  310074:	48 85 c9             	test   %rcx,%rcx
  310077:	0f 84 80 00 00 00    	je     3100fd <<rexx_exec::Interp>::exec_parse+0x83d>
  31007d:	48 83 f9 01          	cmp    $0x1,%rcx
  310081:	0f 85 ed 00 00 00    	jne    310174 <<rexx_exec::Interp>::exec_parse+0x8b4>
<u8 as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2105
  310087:	43 80 7c 25 ff 2e    	cmpb   $0x2e,-0x1(%r13,%r12,1)
rexx_exec::run::shape_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:3777
  31008d:	0f 85 e1 00 00 00    	jne    310174 <<rexx_exec::Interp>::exec_parse+0x8b4>
<rexx_exec::Interp>::read_parse_var:
  310093:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::Interp>::read_stem:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:186
  310098:	4c 89 ff             	mov    %r15,%rdi
  31009b:	4c 89 ee             	mov    %r13,%rsi
  31009e:	4c 89 e2             	mov    %r12,%rdx
  3100a1:	31 c9                	xor    %ecx,%ecx
  3100a3:	e8 98 f3 f0 ff       	call   21f440 <<rexx_exec::Interp>::read_stem_at>
  3100a8:	49 89 c6             	mov    %rax,%r14
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  3100ab:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  3100b2:	04 
  3100b3:	0f 84 a4 02 00 00    	je     31035d <<rexx_exec::Interp>::exec_parse+0xa9d>
<rexx_exec::Interp>::read_parse_var:
  3100b9:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:689
  3100be:	4c 89 fe             	mov    %r15,%rsi
  3100c1:	4c 89 f2             	mov    %r14,%rdx
  3100c4:	e8 87 f8 e9 ff       	call   1af950 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  3100c9:	48 8b 4c 24 60       	mov    0x60(%rsp),%rcx
  3100ce:	48 8b 44 24 68       	mov    0x68(%rsp),%rax
  3100d3:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  3100d8:	48 8b 6c 24 70       	mov    0x70(%rsp),%rbp
  3100dd:	48 85 ed             	test   %rbp,%rbp
  3100e0:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  3100e5:	0f 89 e6 2e 00 00    	jns    312fd1 <<rexx_exec::Interp>::exec_parse+0x3711>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  3100eb:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  3100ed:	48 89 df             	mov    %rbx,%rdi
  3100f0:	48 89 ee             	mov    %rbp,%rsi
  3100f3:	e8 0c 1a e0 ff       	call   111b04 <alloc::raw_vec::handle_error>
  3100f8:	e9 04 36 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::read_parse_var:
  3100fd:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::Interp>::read:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:355
  310102:	4c 89 ff             	mov    %r15,%rdi
  310105:	48 8b b4 24 10 01 00 	mov    0x110(%rsp),%rsi
  31010c:	00 
  31010d:	89 da                	mov    %ebx,%edx
  31010f:	31 c9                	xor    %ecx,%ecx
  310111:	e8 8a 22 f7 ff       	call   2823a0 <<rexx_exec::Interp>::read_at>
  310116:	49 89 c6             	mov    %rax,%r14
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:315
  310119:	f6 c2 01             	test   $0x1,%dl
  31011c:	0f 85 16 34 00 00    	jne    313538 <<rexx_exec::Interp>::exec_parse+0x3c78>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  310122:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  310129:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:678
  31012a:	0f 84 2d 02 00 00    	je     31035d <<rexx_exec::Interp>::exec_parse+0xa9d>
  310130:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:679
  310135:	4c 89 fe             	mov    %r15,%rsi
  310138:	4c 89 f2             	mov    %r14,%rdx
  31013b:	e8 10 f8 e9 ff       	call   1af950 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  310140:	48 8b 4c 24 60       	mov    0x60(%rsp),%rcx
  310145:	48 8b 44 24 68       	mov    0x68(%rsp),%rax
  31014a:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  31014f:	48 8b 6c 24 70       	mov    0x70(%rsp),%rbp
  310154:	48 85 ed             	test   %rbp,%rbp
  310157:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  31015c:	0f 89 3d 01 00 00    	jns    31029f <<rexx_exec::Interp>::exec_parse+0x9df>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  310162:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  310164:	48 89 df             	mov    %rbx,%rdi
  310167:	48 89 ee             	mov    %rbp,%rsi
  31016a:	e8 95 19 e0 ff       	call   111b04 <alloc::raw_vec::handle_error>
  31016f:	e9 8d 35 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::read_parse_var:
  310174:	48 8b 84 24 10 01 00 	mov    0x110(%rsp),%rax
  31017b:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:695
  31017c:	48 8b 40 20          	mov    0x20(%rax),%rax
<core::option::Option<&rexx_exec::plan::Plan> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  310180:	48 85 c0             	test   %rax,%rax
  310183:	74 26                	je     3101ab <<rexx_exec::Interp>::exec_parse+0x8eb>
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::plan::CompoundName>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310185:	48 39 98 b0 00 00 00 	cmp    %rbx,0xb0(%rax)
  31018c:	76 1d                	jbe    3101ab <<rexx_exec::Interp>::exec_parse+0x8eb>
<rexx_exec::plan::Plan>::compound:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:424
  31018e:	48 8b 80 a8 00 00 00 	mov    0xa8(%rax),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::plan::CompoundName>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310195:	48 8d 0c 5b          	lea    (%rbx,%rbx,2),%rcx
  310199:	48 c1 e1 04          	shl    $0x4,%rcx
<core::option::Option<rexx_exec::plan::CompoundName>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  31019d:	48 8b 14 08          	mov    (%rax,%rcx,1),%rdx
  3101a1:	48 83 fa 02          	cmp    $0x2,%rdx
<rexx_exec::plan::Plan>::compound:
  3101a5:	0f 85 82 2d 00 00    	jne    312f2d <<rexx_exec::Interp>::exec_parse+0x366d>
<rexx_exec::Code>::stem:
  3101ab:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1288
  3101b0:	4c 89 ee             	mov    %r13,%rsi
  3101b3:	4c 89 e2             	mov    %r12,%rdx
  3101b6:	e8 c5 62 15 00       	call   466480 <rexx_parse::ast::compound_parts>
  3101bb:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
  3101c0:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  3101c5:	4c 8b 74 24 68       	mov    0x68(%rsp),%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3101ca:	48 83 7c 24 70 00    	cmpq   $0x0,0x70(%rsp)
  3101d0:	74 0b                	je     3101dd <<rexx_exec::Interp>::exec_parse+0x91d>
<rexx_exec::Code>::stem:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1288
  3101d2:	48 8b 7c 24 78       	mov    0x78(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3101d7:	ff 15 bb 7d 1c 00    	call   *0x1c7dbb(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  3101dd:	4c 89 74 24 28       	mov    %r14,0x28(%rsp)
  3101e2:	48 c7 84 24 20 01 00 	movq   $0x0,0x120(%rsp)
  3101e9:	00 00 00 00 00 
  3101ee:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
core::mem::replace::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  3101f3:	f3 41 0f 6f 86 b8 0b 	movdqu 0xbb8(%r14),%xmm0
  3101fa:	00 00 
  3101fc:	66 0f 7f 84 24 a0 01 	movdqa %xmm0,0x1a0(%rsp)
  310203:	00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  310205:	49 c7 86 b8 0b 00 00 	movq   $0x0,0xbb8(%r14)
  31020c:	00 00 00 00 
  310210:	49 c7 86 c0 0b 00 00 	movq   $0x1,0xbc0(%r14)
  310217:	01 00 00 00 
  31021b:	49 c7 86 c8 0b 00 00 	movq   $0x0,0xbc8(%r14)
  310222:	00 00 00 00 
<alloc::vec::Vec<u8>>::clear:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3082
  310226:	48 c7 84 24 b0 01 00 	movq   $0x0,0x1b0(%rsp)
  31022d:	00 00 00 00 00 
  310232:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  310237:	4c 8d 84 24 a0 01 00 	lea    0x1a0(%rsp),%r8
  31023e:	00 
<rexx_exec::Interp>::tail_key_into:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:77
  31023f:	4c 89 f6             	mov    %r14,%rsi
  310242:	48 8b 94 24 10 01 00 	mov    0x110(%rsp),%rdx
  310249:	00 
  31024a:	89 d9                	mov    %ebx,%ecx
  31024c:	e8 af 10 fb ff       	call   2c1300 <<rexx_exec::Interp>::append_tail_key>
<rexx_exec::Interp>::read_parse_var:
  310251:	4d 8d be b8 0b 00 00 	lea    0xbb8(%r14),%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:697
  310258:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
  31025d:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  310261:	74 6f                	je     3102d2 <<rexx_exec::Interp>::exec_parse+0xa12>
  310263:	4c 8b 64 24 68       	mov    0x68(%rsp),%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  310268:	49 83 3f 00          	cmpq   $0x0,(%r15)
  31026c:	74 12                	je     310280 <<rexx_exec::Interp>::exec_parse+0x9c0>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
  31026e:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  310273:	48 8b b8 c0 0b 00 00 	mov    0xbc0(%rax),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31027a:	ff 15 18 7d 1c 00    	call   *0x1c7d18(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::give_key_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:64
  310280:	48 8b 84 24 b0 01 00 	mov    0x1b0(%rsp),%rax
  310287:	00 
  310288:	49 89 47 10          	mov    %rax,0x10(%r15)
  31028c:	66 0f 6f 84 24 a0 01 	movdqa 0x1a0(%rsp),%xmm0
  310293:	00 00 
  310295:	f3 41 0f 7f 07       	movdqu %xmm0,(%r15)
<rexx_exec::Interp>::read_parse_var:
  31029a:	e9 c7 03 00 00       	jmp    310666 <<rexx_exec::Interp>::exec_parse+0xda6>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  31029f:	74 5f                	je     310300 <<rexx_exec::Interp>::exec_parse+0xa40>
  3102a1:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  3102a6:	be 01 00 00 00       	mov    $0x1,%esi
  3102ab:	48 89 ef             	mov    %rbp,%rdi
  3102ae:	e8 cd 74 e1 ff       	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  3102b3:	48 85 c0             	test   %rax,%rax
  3102b6:	0f 84 a8 fe ff ff    	je     310164 <<rexx_exec::Interp>::exec_parse+0x8a4>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  3102bc:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  3102bf:	48 89 c7             	mov    %rax,%rdi
  3102c2:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
  3102c7:	48 89 ea             	mov    %rbp,%rdx
  3102ca:	ff 15 f8 7c 1c 00    	call   *0x1c7cf8(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  3102d0:	eb 34                	jmp    310306 <<rexx_exec::Interp>::exec_parse+0xa46>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  3102d2:	41 f6 86 36 05 00 00 	testb  $0x4,0x536(%r14)
  3102d9:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:701
  3102da:	0f 84 82 2e 00 00    	je     313162 <<rexx_exec::Interp>::exec_parse+0x38a2>
  3102e0:	48 8b 5c 24 28       	mov    0x28(%rsp),%rbx
  3102e5:	48 85 db             	test   %rbx,%rbx
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  3102e8:	0f 89 98 2c 00 00    	jns    312f86 <<rexx_exec::Interp>::exec_parse+0x36c6>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  3102ee:	31 ed                	xor    %ebp,%ebp
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  3102f0:	48 89 ef             	mov    %rbp,%rdi
  3102f3:	48 89 de             	mov    %rbx,%rsi
  3102f6:	e8 09 18 e0 ff       	call   111b04 <alloc::raw_vec::handle_error>
  3102fb:	e9 01 34 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::read_parse_var:
  310300:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  310306:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  31030c:	7e 0b                	jle    310319 <<rexx_exec::Interp>::exec_parse+0xa59>
  31030e:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  310313:	ff 15 7f 7c 1c 00    	call   *0x1c7c7f(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  310319:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31031e:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  310325:	75 23                	jne    31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
  310327:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  31032e:	74 1a                	je     31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  310330:	4c 89 34 24          	mov    %r14,(%rsp)
  310334:	48 8b 74 24 50       	mov    0x50(%rsp),%rsi
  310339:	4c 89 ea             	mov    %r13,%rdx
  31033c:	4c 89 e1             	mov    %r12,%rcx
  31033f:	4d 89 f8             	mov    %r15,%r8
  310342:	49 89 e9             	mov    %rbp,%r9
  310345:	e8 a6 0e f7 ff       	call   2811f0 <<rexx_exec::Interp>::trace_variable_line>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  31034a:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31034d:	74 09                	je     310358 <<rexx_exec::Interp>::exec_parse+0xa98>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31034f:	4c 89 ff             	mov    %r15,%rdi
  310352:	ff 15 40 7c 1c 00    	call   *0x1c7c40(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::parse_strings:
  310358:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  31035d:	49 8b 9f f0 0a 00 00 	mov    0xaf0(%r15),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310364:	49 3b 9f e0 0a 00 00 	cmp    0xae0(%r15),%rbx
  31036b:	0f 84 04 2c 00 00    	je     312f75 <<rexx_exec::Interp>::exec_parse+0x36b5>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310371:	49 8b 87 e8 0a 00 00 	mov    0xae8(%r15),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310378:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  31037c:	48 ff c3             	inc    %rbx
  31037f:	49 89 9f f0 0a 00 00 	mov    %rbx,0xaf0(%r15)
  310386:	4c 8d 2d a6 68 d9 ff 	lea    -0x26975a(%rip),%r13        # a6c33 <panic_unwind::imp::CANARY+0x41ed6>
  31038d:	bb 03 00 00 00       	mov    $0x3,%ebx
<rexx_exec::Interp>::parse_strings:
  310392:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  310397:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  31039e:	00 00 
  3103a0:	0f 85 ff 00 00 00    	jne    3104a5 <<rexx_exec::Interp>::exec_parse+0xbe5>
  3103a6:	41 f6 84 24 2d 05 00 	testb  $0x1,0x52d(%r12)
  3103ad:	00 01 
  3103af:	0f 84 f0 00 00 00    	je     3104a5 <<rexx_exec::Interp>::exec_parse+0xbe5>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3103b5:	4c 89 f0             	mov    %r14,%rax
  3103b8:	48 83 e0 03          	and    $0x3,%rax
  3103bc:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  3103c1:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<rexx_exec::Interp>::traced_into_parse_buffer:
  3103c6:	4c 89 e6             	mov    %r12,%rsi
  3103c9:	4c 89 f2             	mov    %r14,%rdx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3103cc:	0f 84 f1 2a 00 00    	je     312ec3 <<rexx_exec::Interp>::exec_parse+0x3603>
<rexx_exec::Interp>::traced_into_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:354
  3103d2:	e8 f9 62 04 00       	call   3566d0 <<rexx_exec::Interp>::rendered_into_parse_buffer>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3103d7:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
  3103dc:	48 8b 6c 24 68       	mov    0x68(%rsp),%rbp
  3103e1:	4c 8b 64 24 70       	mov    0x70(%rsp),%r12
  3103e6:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  3103eb:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
  3103ef:	0f 84 71 02 00 00    	je     310666 <<rexx_exec::Interp>::exec_parse+0xda6>
<rexx_exec::Interp>::parse_strings:
  3103f5:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  3103fa:	41 80 bf 35 05 00 00 	cmpb   $0x0,0x535(%r15)
  310401:	00 
  310402:	75 5d                	jne    310461 <<rexx_exec::Interp>::exec_parse+0xba1>
  310404:	41 f6 87 2d 05 00 00 	testb  $0x1,0x52d(%r15)
  31040b:	01 
  31040c:	74 53                	je     310461 <<rexx_exec::Interp>::exec_parse+0xba1>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  31040e:	49 8d bf d8 10 00 00 	lea    0x10d8(%r15),%rdi
  310415:	49 8b 87 e8 10 00 00 	mov    0x10e8(%r15),%rax
  31041c:	48 89 84 24 20 01 00 	mov    %rax,0x120(%rsp)
  310423:	00 
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:298
  310424:	4c 89 64 24 10       	mov    %r12,0x10(%rsp)
  310429:	48 89 6c 24 08       	mov    %rbp,0x8(%rsp)
  31042e:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  310435:	48 8d 35 41 a7 d5 ff 	lea    -0x2a58bf(%rip),%rsi        # 6ab7d <panic_unwind::imp::CANARY+0x5e20>
  31043c:	48 8b 54 24 50       	mov    0x50(%rsp),%rdx
  310441:	b9 01 00 00 00       	mov    $0x1,%ecx
  310446:	4d 89 e8             	mov    %r13,%r8
  310449:	49 89 d9             	mov    %rbx,%r9
  31044c:	e8 9f 03 f7 ff       	call   2807f0 <rexx_exec::trace::format::push_tagged>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:310
  310451:	4c 89 ff             	mov    %r15,%rdi
  310454:	48 8b b4 24 20 01 00 	mov    0x120(%rsp),%rsi
  31045b:	00 
  31045c:	e8 ef dc e8 ff       	call   19e150 <<rexx_exec::Interp>::route_trace_line>
<rexx_exec::Interp>::parse_strings:
  310461:	4c 8b 7c 24 28       	mov    0x28(%rsp),%r15
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  310466:	49 8d 87 ff ef ff ff 	lea    -0x1001(%r15),%rax
  31046d:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  310473:	0f 87 db 00 00 00    	ja     310554 <<rexx_exec::Interp>::exec_parse+0xc94>
  310479:	4d 85 ff             	test   %r15,%r15
  31047c:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  310481:	0f 84 53 01 00 00    	je     3105da <<rexx_exec::Interp>::exec_parse+0xd1a>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  310487:	48 89 ef             	mov    %rbp,%rdi
  31048a:	ff 15 08 7b 1c 00    	call   *0x1c7b08(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  310490:	48 8d 05 95 3f d4 ff 	lea    -0x2bc06b(%rip),%rax        # 5442c <GCC_except_table5014+0x4a48>
  310497:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31049c:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  3104a0:	48 01 c1             	add    %rax,%rcx
  3104a3:	ff e1                	jmp    *%rcx
  3104a5:	44 89 f0             	mov    %r14d,%eax
  3104a8:	83 e0 03             	and    $0x3,%eax
  3104ab:	48 8d 0d 7a 3f d4 ff 	lea    -0x2bc086(%rip),%rcx        # 5442c <GCC_except_table5014+0x4a48>
  3104b2:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  3104b6:	48 01 c8             	add    %rcx,%rax
  3104b9:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:114
  3104bb:	41 80 bc 24 04 20 00 	cmpb   $0x1,0x2004(%r12)
  3104c2:	00 01 
  3104c4:	0f 85 4a 01 00 00    	jne    310614 <<rexx_exec::Interp>::exec_parse+0xd54>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3104ca:	41 f6 c6 03          	test   $0x3,%r14b
  3104ce:	0f 85 1b 01 00 00    	jne    3105ef <<rexx_exec::Interp>::exec_parse+0xd2f>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  3104d4:	4c 89 f0             	mov    %r14,%rax
  3104d7:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  3104db:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  3104dd:	49 39 8c 24 08 0a 00 	cmp    %rcx,0xa08(%r12)
  3104e4:	00 
  3104e5:	0f 86 04 01 00 00    	jbe    3105ef <<rexx_exec::Interp>::exec_parse+0xd2f>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  3104eb:	49 8b 84 24 00 0a 00 	mov    0xa00(%r12),%rax
  3104f2:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  3104f3:	4c 89 f2             	mov    %r14,%rdx
  3104f6:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  3104fa:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  3104fe:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  310502:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  310506:	0f 85 e3 00 00 00    	jne    3105ef <<rexx_exec::Interp>::exec_parse+0xd2f>
  31050c:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  31050f:	83 38 01             	cmpl   $0x1,(%rax)
  310512:	0f 85 d7 00 00 00    	jne    3105ef <<rexx_exec::Interp>::exec_parse+0xd2f>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  310518:	80 78 08 01          	cmpb   $0x1,0x8(%rax)
  31051c:	0f 87 cd 00 00 00    	ja     3105ef <<rexx_exec::Interp>::exec_parse+0xd2f>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310522:	49 8b 9c 24 f0 0a 00 	mov    0xaf0(%r12),%rbx
  310529:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  31052a:	49 3b 9c 24 e0 0a 00 	cmp    0xae0(%r12),%rbx
  310531:	00 
  310532:	0f 84 27 2b 00 00    	je     31305f <<rexx_exec::Interp>::exec_parse+0x379f>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310538:	49 8b 84 24 e8 0a 00 	mov    0xae8(%r12),%rax
  31053f:	00 
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310540:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310544:	48 ff c3             	inc    %rbx
  310547:	49 89 9c 24 f0 0a 00 	mov    %rbx,0xaf0(%r12)
  31054e:	00 
  31054f:	e9 c0 00 00 00       	jmp    310614 <<rexx_exec::Interp>::exec_parse+0xd54>
<rexx_exec::Interp>::parse_strings:
  310554:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  310559:	49 8b 9c 24 e0 0b 00 	mov    0xbe0(%r12),%rbx
  310560:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  310561:	48 83 fb 08          	cmp    $0x8,%rbx
  310565:	72 1e                	jb     310585 <<rexx_exec::Interp>::exec_parse+0xcc5>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  310567:	48 89 ef             	mov    %rbp,%rdi
  31056a:	ff 15 28 7a 1c 00    	call   *0x1c7a28(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  310570:	48 8d 05 b5 3e d4 ff 	lea    -0x2bc14b(%rip),%rax        # 5442c <GCC_except_table5014+0x4a48>
  310577:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31057c:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  310580:	48 01 c1             	add    %rax,%rcx
  310583:	ff e1                	jmp    *%rcx
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310585:	49 3b 9c 24 d0 0b 00 	cmp    0xbd0(%r12),%rbx
  31058c:	00 
  31058d:	75 0d                	jne    31059c <<rexx_exec::Interp>::exec_parse+0xcdc>
<rexx_exec::Interp>::give_parse_buffer:
  31058f:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  310596:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310597:	e8 94 09 18 00       	call   490f30 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  31059c:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  3105a3:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  3105a4:	48 8d 0c 5b          	lea    (%rbx,%rbx,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3105a8:	4c 89 3c c8          	mov    %r15,(%rax,%rcx,8)
  3105ac:	48 89 6c c8 08       	mov    %rbp,0x8(%rax,%rcx,8)
  3105b1:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  3105b8:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3105ba:	48 ff c3             	inc    %rbx
  3105bd:	49 89 9c 24 e0 0b 00 	mov    %rbx,0xbe0(%r12)
  3105c4:	00 
  3105c5:	48 8d 05 60 3e d4 ff 	lea    -0x2bc1a0(%rip),%rax        # 5442c <GCC_except_table5014+0x4a48>
  3105cc:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3105d1:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  3105d5:	48 01 c1             	add    %rax,%rcx
  3105d8:	ff e1                	jmp    *%rcx
  3105da:	48 8d 05 4b 3e d4 ff 	lea    -0x2bc1b5(%rip),%rax        # 5442c <GCC_except_table5014+0x4a48>
  3105e1:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
  3105e6:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  3105ea:	48 01 c1             	add    %rax,%rcx
  3105ed:	ff e1                	jmp    *%rcx
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
  3105ef:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  3105f4:	4c 89 e6             	mov    %r12,%rsi
  3105f7:	4c 89 f2             	mov    %r14,%rdx
  3105fa:	31 c9                	xor    %ecx,%ecx
  3105fc:	e8 df 69 ec ff       	call   1d6fe0 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310601:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
  310606:	4c 8b 74 24 68       	mov    0x68(%rsp),%r14
  31060b:	4d 89 f4             	mov    %r14,%r12
  31060e:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  310612:	75 52                	jne    310666 <<rexx_exec::Interp>::exec_parse+0xda6>
<rexx_exec::Interp>::parse_strings:
  310614:	48 8b 44 24 30       	mov    0x30(%rsp),%rax
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:404
  310619:	80 78 68 00          	cmpb   $0x0,0x68(%rax)
  31061d:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  310622:	75 14                	jne    310638 <<rexx_exec::Interp>::exec_parse+0xd78>
  310624:	f6 40 69 01          	testb  $0x1,0x69(%rax)
  310628:	75 0e                	jne    310638 <<rexx_exec::Interp>::exec_parse+0xd78>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31062a:	44 89 f0             	mov    %r14d,%eax
  31062d:	83 e0 03             	and    $0x3,%eax
  310630:	8d 48 ff             	lea    -0x1(%rax),%ecx
  310633:	83 f9 02             	cmp    $0x2,%ecx
  310636:	73 42                	jae    31067a <<rexx_exec::Interp>::exec_parse+0xdba>
<rexx_exec::Interp>::source_text:
  310638:	48 8d bc 24 d0 00 00 	lea    0xd0(%rsp),%rdi
  31063f:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:420
  310640:	4c 89 f2             	mov    %r14,%rdx
  310643:	e8 88 60 04 00       	call   3566d0 <<rexx_exec::Interp>::rendered_into_parse_buffer>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:656
  310648:	4c 8b bc 24 d0 00 00 	mov    0xd0(%rsp),%r15
  31064f:	00 
  310650:	48 8b ac 24 d8 00 00 	mov    0xd8(%rsp),%rbp
  310657:	00 
  310658:	4c 8b a4 24 e0 00 00 	mov    0xe0(%rsp),%r12
  31065f:	00 
<core::result::Result<rexx_exec::parse_template::ParseStrings, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310660:	49 83 ff fd          	cmp    $0xfffffffffffffffd,%r15
  310664:	75 5b                	jne    3106c1 <<rexx_exec::Interp>::exec_parse+0xe01>
<rexx_exec::Interp>::exec_parse:
  310666:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  31066d:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  31066e:	48 89 28             	mov    %rbp,(%rax)
  310671:	4c 89 60 08          	mov    %r12,0x8(%rax)
<rexx_exec::Interp>::exec_parse:
  310675:	e9 9a 27 00 00       	jmp    312e14 <<rexx_exec::Interp>::exec_parse+0x3554>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31067a:	83 f8 03             	cmp    $0x3,%eax
  31067d:	0f 85 ee 25 00 00    	jne    312c71 <<rexx_exec::Interp>::exec_parse+0x33b1>
  310683:	48 8d 9c 24 d8 00 00 	lea    0xd8(%rsp),%rbx
  31068a:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  31068b:	44 89 f0             	mov    %r14d,%eax
  31068e:	c1 e8 02             	shr    $0x2,%eax
  310691:	83 e0 07             	and    $0x7,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  310694:	49 81 e6 00 ff ff ff 	and    $0xffffffffffffff00,%r14
  31069b:	49 09 c6             	or     %rax,%r14
<rexx_exec::Interp>::source_text:
  31069e:	49 bf 01 00 00 00 00 	movabs $0x8000000000000001,%r15
  3106a5:	00 00 80 
  3106a8:	49 ff cf             	dec    %r15
  3106ab:	4d 89 f4             	mov    %r14,%r12
  3106ae:	4c 89 23             	mov    %r12,(%rbx)
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:656
  3106b1:	48 8b ac 24 d8 00 00 	mov    0xd8(%rsp),%rbp
  3106b8:	00 
  3106b9:	4c 8b a4 24 e0 00 00 	mov    0xe0(%rsp),%r12
  3106c0:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:481
  3106c1:	4c 89 bc 24 b8 01 00 	mov    %r15,0x1b8(%rsp)
  3106c8:	00 
  3106c9:	48 89 ac 24 c0 01 00 	mov    %rbp,0x1c0(%rsp)
  3106d0:	00 
  3106d1:	4c 89 a4 24 c8 01 00 	mov    %r12,0x1c8(%rsp)
  3106d8:	00 
  3106d9:	48 8b 44 24 30       	mov    0x30(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  3106de:	0f b6 68 68          	movzbl 0x68(%rax),%ebp
  3106e2:	44 0f b6 68 69       	movzbl 0x69(%rax),%r13d
  3106e7:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  3106ec:	48 8d 94 24 b8 01 00 	lea    0x1b8(%rsp),%rdx
  3106f3:	00 
  3106f4:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  3106f9:	4c 89 e6             	mov    %r12,%rsi
  3106fc:	89 e9                	mov    %ebp,%ecx
  3106fe:	45 89 e8             	mov    %r13d,%r8d
  310701:	4c 8b 4c 24 50       	mov    0x50(%rsp),%r9
  310706:	e8 f5 60 04 00       	call   356800 <<rexx_exec::Interp>::next_template>
<core::result::Result<(rexx_exec::parse_template::SourceText, rexx_exec::parse_template::Cursor), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31070b:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
  310710:	0f 10 44 24 68       	movups 0x68(%rsp),%xmm0
  310715:	0f 29 84 24 d0 00 00 	movaps %xmm0,0xd0(%rsp)
  31071c:	00 
  31071d:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  310721:	0f 84 cd 24 00 00    	je     312bf4 <<rexx_exec::Interp>::exec_parse+0x3334>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  310727:	48 8b 44 24 78       	mov    0x78(%rsp),%rax
  31072c:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
  310731:	48 8b 84 24 88 00 00 	mov    0x88(%rsp),%rax
  310738:	00 
  310739:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  31073e:	48 8b 84 24 90 00 00 	mov    0x90(%rsp),%rax
  310745:	00 
  310746:	48 89 84 24 f0 00 00 	mov    %rax,0xf0(%rsp)
  31074d:	00 
  31074e:	48 8b 84 24 98 00 00 	mov    0x98(%rsp),%rax
  310755:	00 
  310756:	48 89 84 24 d8 01 00 	mov    %rax,0x1d8(%rsp)
  31075d:	00 
  31075e:	48 8b 9c 24 a0 00 00 	mov    0xa0(%rsp),%rbx
  310765:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  310766:	66 0f 6f 84 24 d0 00 	movdqa 0xd0(%rsp),%xmm0
  31076d:	00 00 
  31076f:	f3 0f 7f 84 24 00 01 	movdqu %xmm0,0x100(%rsp)
  310776:	00 00 
  310778:	4c 89 bc 24 f8 00 00 	mov    %r15,0xf8(%rsp)
  31077f:	00 
  310780:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
<alloc::vec::Vec<core::option::Option<rexx_parse::ast::ParseTrigger>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310785:	48 8b 41 10          	mov    0x10(%rcx),%rax
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  310789:	48 85 c0             	test   %rax,%rax
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31078c:	0f 84 58 25 00 00    	je     312cea <<rexx_exec::Interp>::exec_parse+0x342a>
  310792:	48 8b 49 08          	mov    0x8(%rcx),%rcx
  310796:	48 6b c0 70          	imul   $0x70,%rax,%rax
  31079a:	48 89 4c 24 58       	mov    %rcx,0x58(%rsp)
  31079f:	48 01 c8             	add    %rcx,%rax
  3107a2:	48 89 84 24 08 02 00 	mov    %rax,0x208(%rsp)
  3107a9:	00 
  3107aa:	48 b9 01 00 00 00 00 	movabs $0x8000000000000001,%rcx
  3107b1:	00 00 80 
  3107b4:	49 8d 84 24 e0 0a 00 	lea    0xae0(%r12),%rax
  3107bb:	00 
  3107bc:	48 89 84 24 90 01 00 	mov    %rax,0x190(%rsp)
  3107c3:	00 
  3107c4:	48 8b 84 24 10 01 00 	mov    0x110(%rsp),%rax
  3107cb:	00 
  3107cc:	48 8b 50 18          	mov    0x18(%rax),%rdx
  3107d0:	48 89 94 24 50 02 00 	mov    %rdx,0x250(%rsp)
  3107d7:	00 
  3107d8:	48 8b 50 08          	mov    0x8(%rax),%rdx
  3107dc:	48 89 94 24 48 02 00 	mov    %rdx,0x248(%rsp)
  3107e3:	00 
  3107e4:	48 8b 40 10          	mov    0x10(%rax),%rax
  3107e8:	48 89 84 24 40 02 00 	mov    %rax,0x240(%rsp)
  3107ef:	00 
  3107f0:	49 8d 84 24 b0 0a 00 	lea    0xab0(%r12),%rax
  3107f7:	00 
  3107f8:	48 89 84 24 38 02 00 	mov    %rax,0x238(%rsp)
  3107ff:	00 
  310800:	49 8d 84 24 d0 0b 00 	lea    0xbd0(%r12),%rax
  310807:	00 
  310808:	48 89 84 24 38 01 00 	mov    %rax,0x138(%rsp)
  31080f:	00 
  310810:	49 8d 84 24 d8 10 00 	lea    0x10d8(%r12),%rax
  310817:	00 
  310818:	48 89 84 24 10 02 00 	mov    %rax,0x210(%rsp)
  31081f:	00 
  310820:	48 8d 05 09 ff 1a 00 	lea    0x1aff09(%rip),%rax        # 4c0730 <rexx_api::ffi::CALL_CONTEXT+0x53a0>
  310827:	48 89 84 24 d0 01 00 	mov    %rax,0x1d0(%rsp)
  31082e:	00 
  31082f:	48 89 84 24 58 01 00 	mov    %rax,0x158(%rsp)
  310836:	00 
  310837:	89 ac 24 f8 01 00 00 	mov    %ebp,0x1f8(%rsp)
  31083e:	44 89 ac 24 f4 01 00 	mov    %r13d,0x1f4(%rsp)
  310845:	00 
  310846:	49 8d 04 0f          	lea    (%r15,%rcx,1),%rax
  31084a:	4c 89 bc 24 28 01 00 	mov    %r15,0x128(%rsp)
  310851:	00 
  310852:	49 c1 ff 3f          	sar    $0x3f,%r15
  310856:	49 21 c7             	and    %rax,%r15
  310859:	4c 89 bc 24 98 01 00 	mov    %r15,0x198(%rsp)
  310860:	00 
  310861:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
  310866:	eb 23                	jmp    31088b <<rexx_exec::Interp>::exec_parse+0xfcb>
  310868:	0f 1f 84 00 00 00 00 	nopl   0x0(%rax,%rax,1)
  31086f:	00 
  310870:	4c 89 fb             	mov    %r15,%rbx
  310873:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  310878:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31087d:	4c 3b ac 24 08 02 00 	cmp    0x208(%rsp),%r13
  310884:	00 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  310885:	0f 84 57 24 00 00    	je     312ce2 <<rexx_exec::Interp>::exec_parse+0x3422>
  31088b:	4c 8b bc 24 d8 01 00 	mov    0x1d8(%rsp),%r15
  310892:	00 
  310893:	48 8b ac 24 f0 00 00 	mov    0xf0(%rsp),%rbp
  31089a:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:627
  31089b:	49 8d 45 70          	lea    0x70(%r13),%rax
  31089f:	48 89 44 24 58       	mov    %rax,0x58(%rsp)
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:485
  3108a4:	49 83 7d 00 ff       	cmpq   $0xffffffffffffffff,0x0(%r13)
  3108a9:	0f 84 ce 21 00 00    	je     312a7d <<rexx_exec::Interp>::exec_parse+0x31bd>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:31
  3108af:	41 80 7d 68 00       	cmpb   $0x0,0x68(%r13)
  3108b4:	0f 84 e6 01 00 00    	je     310aa0 <<rexx_exec::Interp>::exec_parse+0x11e0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:36
  3108ba:	49 83 7d 18 ff       	cmpq   $0xffffffffffffffff,0x18(%r13)
  3108bf:	0f 84 61 25 00 00    	je     312e26 <<rexx_exec::Interp>::exec_parse+0x3566>
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:127
  3108c5:	c6 84 24 70 01 00 00 	movb   $0x0,0x170(%rsp)
  3108cc:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:128
  3108cd:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  3108d2:	4c 89 e6             	mov    %r12,%rsi
  3108d5:	48 8d 94 24 70 01 00 	lea    0x170(%rsp),%rdx
  3108dc:	00 
  3108dd:	e8 ae bf f6 ff       	call   27c890 <<rexx_exec::Interp>::enter_eval_node::<u8>>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3108e2:	4c 8b 64 24 60       	mov    0x60(%rsp),%r12
  3108e7:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  3108eb:	0f 85 e7 23 00 00    	jne    312cd8 <<rexx_exec::Interp>::exec_parse+0x3418>
<rexx_exec::Interp>::apply_trigger:
  3108f1:	4d 8d 75 18          	lea    0x18(%r13),%r14
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:129
  3108f5:	48 8d bc 24 d0 00 00 	lea    0xd0(%rsp),%rdi
  3108fc:	00 
  3108fd:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  310902:	4c 89 e6             	mov    %r12,%rsi
  310905:	48 8b 94 24 10 01 00 	mov    0x110(%rsp),%rdx
  31090c:	00 
  31090d:	4c 89 f1             	mov    %r14,%rcx
  310910:	e8 db c1 f6 ff       	call   27caf0 <<rexx_exec::Interp>::eval_node>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:130
  310915:	49 ff 8c 24 c8 04 00 	decq   0x4c8(%r12)
  31091c:	00 
  31091d:	4c 89 e6             	mov    %r12,%rsi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:131
  310920:	4c 8b a4 24 d0 00 00 	mov    0xd0(%rsp),%r12
  310927:	00 
  310928:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  31092c:	0f 85 66 25 00 00    	jne    312e98 <<rexx_exec::Interp>::exec_parse+0x35d8>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  310932:	f6 86 36 05 00 00 04 	testb  $0x4,0x536(%rsi)
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  310939:	48 8b 84 24 d8 00 00 	mov    0xd8(%rsp),%rax
  310940:	00 
  310941:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
<rexx_exec::Interp>::trace_intermediate:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:161
  310946:	74 2e                	je     310976 <<rexx_exec::Interp>::exec_parse+0x10b6>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:164
  310948:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  31094d:	48 8b 94 24 10 01 00 	mov    0x110(%rsp),%rdx
  310954:	00 
  310955:	4c 89 f1             	mov    %r14,%rcx
  310958:	4c 8b 44 24 30       	mov    0x30(%rsp),%r8
  31095d:	e8 ae ec f6 ff       	call   27f610 <<rexx_exec::Interp>::trace_intermediate_event>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310962:	4c 8b 64 24 60       	mov    0x60(%rsp),%r12
  310967:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  31096b:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  310970:	0f 85 62 23 00 00    	jne    312cd8 <<rexx_exec::Interp>::exec_parse+0x3418>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310976:	4c 8b b6 f0 0a 00 00 	mov    0xaf0(%rsi),%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  31097d:	4c 3b b6 e0 0a 00 00 	cmp    0xae0(%rsi),%r14
  310984:	49 89 f4             	mov    %rsi,%r12
  310987:	75 0d                	jne    310996 <<rexx_exec::Interp>::exec_parse+0x10d6>
  310989:	48 8b bc 24 90 01 00 	mov    0x190(%rsp),%rdi
  310990:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310991:	e8 7a b5 e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310996:	49 8b 84 24 e8 0a 00 	mov    0xae8(%r12),%rax
  31099d:	00 
  31099e:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3109a3:	4a 89 0c f0          	mov    %rcx,(%rax,%r14,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3109a7:	49 ff c6             	inc    %r14
  3109aa:	4d 89 b4 24 f0 0a 00 	mov    %r14,0xaf0(%r12)
  3109b1:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:55
  3109b2:	41 80 bc 24 2d 05 00 	cmpb   $0x1,0x52d(%r12)
  3109b9:	00 01 
  3109bb:	0f 84 3b 20 00 00    	je     3129fc <<rexx_exec::Interp>::exec_parse+0x313c>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:59
  3109c1:	41 0f b6 45 68       	movzbl 0x68(%r13),%eax
  3109c6:	f6 d0                	not    %al
  3109c8:	a8 06                	test   $0x6,%al
  3109ca:	0f 85 e7 00 00 00    	jne    310ab7 <<rexx_exec::Interp>::exec_parse+0x11f7>
  3109d0:	48 8b 5c 24 30       	mov    0x30(%rsp),%rbx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3109d5:	89 d8                	mov    %ebx,%eax
  3109d7:	83 e0 03             	and    $0x3,%eax
  3109da:	48 8d 15 7b 3a d4 ff 	lea    -0x2bc585(%rip),%rdx        # 5445c <GCC_except_table5014+0x4a78>
  3109e1:	48 63 0c 82          	movslq (%rdx,%rax,4),%rcx
  3109e5:	48 01 d1             	add    %rdx,%rcx
  3109e8:	ff e1                	jmp    *%rcx
  3109ea:	83 f8 01             	cmp    $0x1,%eax
  3109ed:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  3109f2:	0f 84 c0 02 00 00    	je     310cb8 <<rexx_exec::Interp>::exec_parse+0x13f8>
  3109f8:	41 80 be 04 20 00 00 	cmpb   $0x0,0x2004(%r14)
  3109ff:	00 
  310a00:	0f 84 b2 02 00 00    	je     310cb8 <<rexx_exec::Interp>::exec_parse+0x13f8>
  310a06:	48 85 c0             	test   %rax,%rax
  310a09:	0f 85 70 02 00 00    	jne    310c7f <<rexx_exec::Interp>::exec_parse+0x13bf>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310a0f:	48 89 d8             	mov    %rbx,%rax
  310a12:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  310a16:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310a18:	49 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%r14)
  310a1f:	0f 86 5a 02 00 00    	jbe    310c7f <<rexx_exec::Interp>::exec_parse+0x13bf>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  310a25:	49 8b 86 00 0a 00 00 	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  310a2c:	48 89 da             	mov    %rbx,%rdx
  310a2f:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310a33:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  310a37:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  310a3b:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  310a3f:	0f 85 3a 02 00 00    	jne    310c7f <<rexx_exec::Interp>::exec_parse+0x13bf>
  310a45:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  310a48:	83 38 01             	cmpl   $0x1,(%rax)
  310a4b:	0f 85 2e 02 00 00    	jne    310c7f <<rexx_exec::Interp>::exec_parse+0x13bf>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  310a51:	80 78 08 02          	cmpb   $0x2,0x8(%rax)
  310a55:	0f 83 24 02 00 00    	jae    310c7f <<rexx_exec::Interp>::exec_parse+0x13bf>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310a5b:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310a62:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  310a69:	75 0d                	jne    310a78 <<rexx_exec::Interp>::exec_parse+0x11b8>
  310a6b:	48 8b bc 24 90 01 00 	mov    0x190(%rsp),%rdi
  310a72:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310a73:	e8 98 b4 e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310a78:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
  310a7f:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310a84:	48 89 0c d8          	mov    %rcx,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310a88:	48 ff c3             	inc    %rbx
  310a8b:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
  310a92:	e9 21 02 00 00       	jmp    310cb8 <<rexx_exec::Interp>::exec_parse+0x13f8>
<rexx_exec::Interp>::apply_trigger:
  310a97:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  310a9e:	00 00 
  310aa0:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  310aa5:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  310aaa:	48 89 84 24 f0 00 00 	mov    %rax,0xf0(%rsp)
  310ab1:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:31
  310ab2:	e9 1a 0b 00 00       	jmp    3115d1 <<rexx_exec::Interp>::exec_parse+0x1d11>
  310ab7:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:97
  310abc:	4c 89 f7             	mov    %r14,%rdi
  310abf:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
  310ac4:	e8 d7 64 04 00       	call   356fa0 <<rexx_exec::Interp>::whole_nonneg>
  310ac9:	a8 01                	test   $0x1,%al
  310acb:	0f 84 fc 23 00 00    	je     312ecd <<rexx_exec::Interp>::exec_parse+0x360d>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:106
  310ad1:	41 0f b6 45 68       	movzbl 0x68(%r13),%eax
  310ad6:	48 8d 0d 5f 39 d4 ff 	lea    -0x2bc6a1(%rip),%rcx        # 5443c <GCC_except_table5014+0x4a58>
  310add:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310ae1:	48 01 c8             	add    %rcx,%rax
  310ae4:	ff e0                	jmp    *%rax
  310ae6:	48 89 ac 24 f0 00 00 	mov    %rbp,0xf0(%rsp)
  310aed:	00 
  310aee:	4c 89 f8             	mov    %r15,%rax
  310af1:	49 89 df             	mov    %rbx,%r15
  310af4:	e9 d8 0a 00 00       	jmp    3115d1 <<rexx_exec::Interp>::exec_parse+0x1d11>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:167
  310af9:	48 89 de             	mov    %rbx,%rsi
  310afc:	48 c1 fe 02          	sar    $0x2,%rsi
  310b00:	48 89 b4 24 67 01 00 	mov    %rsi,0x167(%rsp)
  310b07:	00 
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:942
  310b08:	66 0f ef c0          	pxor   %xmm0,%xmm0
  310b0c:	66 0f 7f 84 24 70 01 	movdqa %xmm0,0x170(%rsp)
  310b13:	00 00 
  310b15:	c7 84 24 80 01 00 00 	movl   $0x0,0x180(%rsp)
  310b1c:	00 00 00 00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:943
  310b20:	48 8d bc 24 70 01 00 	lea    0x170(%rsp),%rdi
  310b27:	00 
  310b28:	e8 43 f4 e9 ff       	call   1aff70 <rexx_exec::value::write_small_int>
  310b2d:	49 89 c1             	mov    %rax,%r9
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:944
  310b30:	0f b6 84 24 76 01 00 	movzbl 0x176(%rsp),%eax
  310b37:	00 
  310b38:	c1 e0 10             	shl    $0x10,%eax
  310b3b:	0f b7 8c 24 74 01 00 	movzwl 0x174(%rsp),%ecx
  310b42:	00 
  310b43:	09 c1                	or     %eax,%ecx
  310b45:	8b 84 24 70 01 00 00 	mov    0x170(%rsp),%eax
  310b4c:	48 8d b4 24 77 01 00 	lea    0x177(%rsp),%rsi
  310b53:	00 
  310b54:	48 8b 16             	mov    (%rsi),%rdx
  310b57:	48 8b 76 05          	mov    0x5(%rsi),%rsi
  310b5b:	48 89 b4 24 d5 00 00 	mov    %rsi,0xd5(%rsp)
  310b62:	00 
  310b63:	48 89 94 24 d0 00 00 	mov    %rdx,0xd0(%rsp)
  310b6a:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:71
  310b6b:	48 c1 e1 28          	shl    $0x28,%rcx
  310b6f:	48 c1 e0 08          	shl    $0x8,%rax
  310b73:	48 09 c8             	or     %rcx,%rax
  310b76:	4c 09 c8             	or     %r9,%rax
  310b79:	c6 44 24 60 03       	movb   $0x3,0x60(%rsp)
  310b7e:	48 89 44 24 61       	mov    %rax,0x61(%rsp)
  310b83:	48 8b 84 24 df 00 00 	mov    0xdf(%rsp),%rax
  310b8a:	00 
  310b8b:	48 8d 4c 24 62       	lea    0x62(%rsp),%rcx
  310b90:	48 89 41 16          	mov    %rax,0x16(%rcx)
  310b94:	48 8b 84 24 d0 00 00 	mov    0xd0(%rsp),%rax
  310b9b:	00 
  310b9c:	48 89 41 07          	mov    %rax,0x7(%rcx)
  310ba0:	8b 84 24 d8 00 00 00 	mov    0xd8(%rsp),%eax
  310ba7:	89 41 0f             	mov    %eax,0xf(%rcx)
  310baa:	0f b6 84 24 dc 00 00 	movzbl 0xdc(%rsp),%eax
  310bb1:	00 
  310bb2:	88 41 13             	mov    %al,0x13(%rcx)
  310bb5:	0f b7 84 24 dd 00 00 	movzwl 0xdd(%rsp),%eax
  310bbc:	00 
  310bbd:	66 89 41 14          	mov    %ax,0x14(%rcx)
  310bc1:	0f b6 84 24 df 00 00 	movzbl 0xdf(%rsp),%eax
  310bc8:	00 
  310bc9:	88 41 16             	mov    %al,0x16(%rcx)
  310bcc:	48 89 9c 24 80 00 00 	mov    %rbx,0x80(%rsp)
  310bd3:	00 
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  310bd4:	b8 14 00 00 00       	mov    $0x14,%eax
  310bd9:	4c 29 c8             	sub    %r9,%rax
  310bdc:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  310be1:	e9 9d 02 00 00       	jmp    310e83 <<rexx_exec::Interp>::exec_parse+0x15c3>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  310be6:	89 d8                	mov    %ebx,%eax
  310be8:	c0 e8 02             	shr    $0x2,%al
  310beb:	24 07                	and    $0x7,%al
<u64>::to_ne_bytes:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:4058
  310bed:	89 d9                	mov    %ebx,%ecx
  310bef:	c1 e9 10             	shr    $0x10,%ecx
  310bf2:	89 da                	mov    %ebx,%edx
  310bf4:	c1 ea 18             	shr    $0x18,%edx
  310bf7:	48 89 de             	mov    %rbx,%rsi
  310bfa:	48 c1 ee 20          	shr    $0x20,%rsi
  310bfe:	48 89 df             	mov    %rbx,%rdi
  310c01:	48 c1 ef 28          	shr    $0x28,%rdi
  310c05:	49 89 d8             	mov    %rbx,%r8
  310c08:	49 c1 e8 30          	shr    $0x30,%r8
  310c0c:	49 89 d9             	mov    %rbx,%r9
  310c0f:	49 c1 e9 38          	shr    $0x38,%r9
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  310c13:	88 84 24 60 01 00 00 	mov    %al,0x160(%rsp)
  310c1a:	88 bc 24 61 01 00 00 	mov    %bh,0x161(%rsp)
  310c21:	88 8c 24 62 01 00 00 	mov    %cl,0x162(%rsp)
  310c28:	88 94 24 63 01 00 00 	mov    %dl,0x163(%rsp)
  310c2f:	40 88 b4 24 64 01 00 	mov    %sil,0x164(%rsp)
  310c36:	00 
  310c37:	40 88 bc 24 65 01 00 	mov    %dil,0x165(%rsp)
  310c3e:	00 
  310c3f:	44 88 84 24 66 01 00 	mov    %r8b,0x166(%rsp)
  310c46:	00 
  310c47:	44 88 8c 24 67 01 00 	mov    %r9b,0x167(%rsp)
  310c4e:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:66
  310c4f:	48 8b 84 24 60 01 00 	mov    0x160(%rsp),%rax
  310c56:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:67
  310c57:	48 89 84 24 58 02 00 	mov    %rax,0x258(%rsp)
  310c5e:	00 
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  310c5f:	0f b6 c0             	movzbl %al,%eax
  310c62:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  310c67:	a8 f8                	test   $0xf8,%al
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310c69:	0f 85 ac 28 00 00    	jne    31351b <<rexx_exec::Interp>::exec_parse+0x3c5b>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
  310c6f:	40 b6 01             	mov    $0x1,%sil
  310c72:	4c 8d 8c 24 59 02 00 	lea    0x259(%rsp),%r9
  310c79:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:86
  310c7a:	e9 09 02 00 00       	jmp    310e88 <<rexx_exec::Interp>::exec_parse+0x15c8>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  310c7f:	48 8d bc 24 d0 00 00 	lea    0xd0(%rsp),%rdi
  310c86:	00 
  310c87:	4c 89 f6             	mov    %r14,%rsi
  310c8a:	48 89 da             	mov    %rbx,%rdx
  310c8d:	31 c9                	xor    %ecx,%ecx
  310c8f:	e8 4c 63 ec ff       	call   1d6fe0 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310c94:	4c 8b a4 24 d0 00 00 	mov    0xd0(%rsp),%r12
  310c9b:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  310c9c:	48 8b 9c 24 d8 00 00 	mov    0xd8(%rsp),%rbx
  310ca3:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310ca4:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  310ca8:	0f 85 f2 21 00 00    	jne    312ea0 <<rexx_exec::Interp>::exec_parse+0x35e0>
<rexx_exec::Interp>::apply_trigger:
  310cae:	48 89 5c 24 30       	mov    %rbx,0x30(%rsp)
  310cb3:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310cb8:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310cbf:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  310cc6:	75 0d                	jne    310cd5 <<rexx_exec::Interp>::exec_parse+0x1415>
  310cc8:	48 8b bc 24 90 01 00 	mov    0x190(%rsp),%rdi
  310ccf:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310cd0:	e8 3b b2 e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310cd5:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
  310cdc:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310ce1:	48 89 0c d8          	mov    %rcx,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310ce5:	48 ff c3             	inc    %rbx
  310ce8:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
  310cef:	48 89 cb             	mov    %rcx,%rbx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310cf2:	89 d8                	mov    %ebx,%eax
  310cf4:	83 e0 03             	and    $0x3,%eax
  310cf7:	48 8d 0d 6e 37 d4 ff 	lea    -0x2bc892(%rip),%rcx        # 5446c <GCC_except_table5014+0x4a88>
  310cfe:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310d02:	48 01 c8             	add    %rcx,%rax
  310d05:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  310d07:	4c 89 f7             	mov    %r14,%rdi
  310d0a:	48 89 de             	mov    %rbx,%rsi
  310d0d:	e8 8e df ec ff       	call   1deca0 <<rexx_exec::Interp>::try_text>
<core::option::Option<&[u8]>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:634
  310d12:	48 85 c0             	test   %rax,%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  310d15:	0f 84 5d 04 00 00    	je     311178 <<rexx_exec::Interp>::exec_parse+0x18b8>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:77
  310d1b:	c6 44 24 60 00       	movb   $0x0,0x60(%rsp)
  310d20:	48 8b 84 24 88 01 00 	mov    0x188(%rsp),%rax
  310d27:	00 
  310d28:	48 c1 e0 08          	shl    $0x8,%rax
  310d2c:	0f b6 8c 24 58 01 00 	movzbl 0x158(%rsp),%ecx
  310d33:	00 
  310d34:	48 09 c1             	or     %rax,%rcx
  310d37:	48 89 4c 24 61       	mov    %rcx,0x61(%rsp)
  310d3c:	48 8b 84 24 4f 01 00 	mov    0x14f(%rsp),%rax
  310d43:	00 
  310d44:	48 8d 4c 24 62       	lea    0x62(%rsp),%rcx
  310d49:	48 89 41 16          	mov    %rax,0x16(%rcx)
  310d4d:	f3 0f 6f 84 24 40 01 	movdqu 0x140(%rsp),%xmm0
  310d54:	00 00 
  310d56:	f3 0f 7f 41 07       	movdqu %xmm0,0x7(%rcx)
  310d5b:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
  310d60:	48 89 b4 24 80 00 00 	mov    %rsi,0x80(%rsp)
  310d67:	00 
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1695
  310d68:	4c 89 f7             	mov    %r14,%rdi
  310d6b:	e8 30 df ec ff       	call   1deca0 <<rexx_exec::Interp>::try_text>
  310d70:	48 89 54 24 28       	mov    %rdx,0x28(%rsp)
  310d75:	49 89 c1             	mov    %rax,%r9
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  310d78:	48 85 c0             	test   %rax,%rax
  310d7b:	0f 85 05 01 00 00    	jne    310e86 <<rexx_exec::Interp>::exec_parse+0x15c6>
  310d81:	e9 f3 27 00 00       	jmp    313579 <<rexx_exec::Interp>::exec_parse+0x3cb9>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:167
  310d86:	48 89 de             	mov    %rbx,%rsi
  310d89:	48 c1 fe 02          	sar    $0x2,%rsi
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:942
  310d8d:	66 0f ef c0          	pxor   %xmm0,%xmm0
  310d91:	66 0f 7f 84 24 20 02 	movdqa %xmm0,0x220(%rsp)
  310d98:	00 00 
  310d9a:	c7 84 24 30 02 00 00 	movl   $0x0,0x230(%rsp)
  310da1:	00 00 00 00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:943
  310da5:	48 8d bc 24 20 02 00 	lea    0x220(%rsp),%rdi
  310dac:	00 
  310dad:	e8 be f1 e9 ff       	call   1aff70 <rexx_exec::value::write_small_int>
  310db2:	49 89 c1             	mov    %rax,%r9
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:944
  310db5:	0f b6 84 24 26 02 00 	movzbl 0x226(%rsp),%eax
  310dbc:	00 
  310dbd:	c1 e0 10             	shl    $0x10,%eax
  310dc0:	0f b7 8c 24 24 02 00 	movzwl 0x224(%rsp),%ecx
  310dc7:	00 
  310dc8:	09 c1                	or     %eax,%ecx
  310dca:	48 c1 e1 20          	shl    $0x20,%rcx
  310dce:	8b 94 24 20 02 00 00 	mov    0x220(%rsp),%edx
  310dd5:	48 09 ca             	or     %rcx,%rdx
  310dd8:	48 8d 8c 24 27 02 00 	lea    0x227(%rsp),%rcx
  310ddf:	00 
  310de0:	48 8b 01             	mov    (%rcx),%rax
  310de3:	48 8b 49 05          	mov    0x5(%rcx),%rcx
  310de7:	48 89 8c 24 45 01 00 	mov    %rcx,0x145(%rsp)
  310dee:	00 
  310def:	48 89 84 24 40 01 00 	mov    %rax,0x140(%rsp)
  310df6:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:77
  310df7:	c6 44 24 60 03       	movb   $0x3,0x60(%rsp)
  310dfc:	48 89 94 24 88 01 00 	mov    %rdx,0x188(%rsp)
  310e03:	00 
  310e04:	48 89 d0             	mov    %rdx,%rax
  310e07:	48 c1 e0 08          	shl    $0x8,%rax
  310e0b:	4c 09 c8             	or     %r9,%rax
  310e0e:	48 89 44 24 61       	mov    %rax,0x61(%rsp)
  310e13:	48 8b 84 24 40 01 00 	mov    0x140(%rsp),%rax
  310e1a:	00 
  310e1b:	48 8d 4c 24 62       	lea    0x62(%rsp),%rcx
  310e20:	48 89 41 07          	mov    %rax,0x7(%rcx)
  310e24:	8b 84 24 48 01 00 00 	mov    0x148(%rsp),%eax
  310e2b:	89 41 0f             	mov    %eax,0xf(%rcx)
  310e2e:	0f b6 84 24 4c 01 00 	movzbl 0x14c(%rsp),%eax
  310e35:	00 
  310e36:	88 41 13             	mov    %al,0x13(%rcx)
  310e39:	0f b7 84 24 4d 01 00 	movzwl 0x14d(%rsp),%eax
  310e40:	00 
  310e41:	66 89 41 14          	mov    %ax,0x14(%rcx)
  310e45:	0f b6 84 24 4f 01 00 	movzbl 0x14f(%rsp),%eax
  310e4c:	00 
  310e4d:	88 41 16             	mov    %al,0x16(%rcx)
  310e50:	48 8b 84 24 4f 01 00 	mov    0x14f(%rsp),%rax
  310e57:	00 
  310e58:	48 89 41 16          	mov    %rax,0x16(%rcx)
  310e5c:	48 89 9c 24 80 00 00 	mov    %rbx,0x80(%rsp)
  310e63:	00 
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:544
  310e64:	49 83 f9 14          	cmp    $0x14,%r9
  310e68:	0f 87 69 27 00 00    	ja     3135d7 <<rexx_exec::Interp>::exec_parse+0x3d17>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  310e6e:	b8 14 00 00 00       	mov    $0x14,%eax
  310e73:	4c 29 c8             	sub    %r9,%rax
  310e76:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  310e7b:	4c 89 8c 24 58 01 00 	mov    %r9,0x158(%rsp)
  310e82:	00 
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  310e83:	49 01 c9             	add    %rcx,%r9
<rexx_exec::Interp>::apply_trigger:
  310e86:	31 f6                	xor    %esi,%esi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:81
  310e88:	41 80 7d 68 06       	cmpb   $0x6,0x68(%r13)
  310e8d:	89 74 24 30          	mov    %esi,0x30(%rsp)
  310e91:	75 32                	jne    310ec5 <<rexx_exec::Interp>::exec_parse+0x1605>
  310e93:	48 8b 84 24 98 01 00 	mov    0x198(%rsp),%rax
  310e9a:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310e9b:	48 85 c0             	test   %rax,%rax
  310e9e:	74 5b                	je     310efb <<rexx_exec::Interp>::exec_parse+0x163b>
  310ea0:	48 83 f8 01          	cmp    $0x1,%rax
  310ea4:	75 7f                	jne    310f25 <<rexx_exec::Interp>::exec_parse+0x1665>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  310ea6:	0f b6 9c 24 00 01 00 	movzbl 0x100(%rsp),%ebx
  310ead:	00 
  310eae:	4c 8d b4 24 01 01 00 	lea    0x101(%rsp),%r14
  310eb5:	00 
  310eb6:	48 83 fb 08          	cmp    $0x8,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310eba:	0f 82 c2 03 00 00    	jb     311282 <<rexx_exec::Interp>::exec_parse+0x19c2>
  310ec0:	e9 2d 26 00 00       	jmp    3134f2 <<rexx_exec::Interp>::exec_parse+0x3c32>
<rexx_exec::Interp>::apply_trigger:
  310ec5:	48 8b 84 24 98 01 00 	mov    0x198(%rsp),%rax
  310ecc:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310ecd:	48 85 c0             	test   %rax,%rax
  310ed0:	74 3e                	je     310f10 <<rexx_exec::Interp>::exec_parse+0x1650>
  310ed2:	48 83 f8 01          	cmp    $0x1,%rax
  310ed6:	0f 85 c3 00 00 00    	jne    310f9f <<rexx_exec::Interp>::exec_parse+0x16df>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  310edc:	0f b6 9c 24 00 01 00 	movzbl 0x100(%rsp),%ebx
  310ee3:	00 
  310ee4:	4c 8d b4 24 01 01 00 	lea    0x101(%rsp),%r14
  310eeb:	00 
  310eec:	48 83 fb 08          	cmp    $0x8,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310ef0:	0f 82 64 04 00 00    	jb     31135a <<rexx_exec::Interp>::exec_parse+0x1a9a>
  310ef6:	e9 f7 25 00 00       	jmp    3134f2 <<rexx_exec::Interp>::exec_parse+0x3c32>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310efb:	4c 8b b4 24 00 01 00 	mov    0x100(%rsp),%r14
  310f02:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310f03:	48 8b 9c 24 08 01 00 	mov    0x108(%rsp),%rbx
  310f0a:	00 
  310f0b:	e9 72 03 00 00       	jmp    311282 <<rexx_exec::Interp>::exec_parse+0x19c2>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310f10:	4c 8b b4 24 00 01 00 	mov    0x100(%rsp),%r14
  310f17:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310f18:	48 8b 9c 24 08 01 00 	mov    0x108(%rsp),%rbx
  310f1f:	00 
  310f20:	e9 35 04 00 00       	jmp    31135a <<rexx_exec::Interp>::exec_parse+0x1a9a>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310f25:	48 8b 84 24 00 01 00 	mov    0x100(%rsp),%rax
  310f2c:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310f2d:	a8 03                	test   $0x3,%al
  310f2f:	0f 85 90 27 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310f35:	48 89 c1             	mov    %rax,%rcx
  310f38:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310f3c:	89 ca                	mov    %ecx,%edx
  310f3e:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310f43:	48 39 91 08 0a 00 00 	cmp    %rdx,0xa08(%rcx)
  310f4a:	0f 86 75 27 00 00    	jbe    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
<rexx_exec::parse_template::SourceText>::bytes:
  310f50:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310f55:	48 8b 89 00 0a 00 00 	mov    0xa00(%rcx),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310f5c:	48 8d 14 52          	lea    (%rdx,%rdx,2),%rdx
  310f60:	48 c1 e2 05          	shl    $0x5,%rdx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310f64:	83 3c 11 01          	cmpl   $0x1,(%rcx,%rdx,1)
  310f68:	0f 85 57 27 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
  310f6e:	48 01 d1             	add    %rdx,%rcx
  310f71:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  310f75:	0f 85 4a 27 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
  310f7b:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  310f7f:	39 41 04             	cmp    %eax,0x4(%rcx)
  310f82:	0f 85 3d 27 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  310f88:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  310f8c:	0f 85 e5 02 00 00    	jne    311277 <<rexx_exec::Interp>::exec_parse+0x19b7>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310f92:	4c 8b 71 20          	mov    0x20(%rcx),%r14
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310f96:	48 8b 59 28          	mov    0x28(%rcx),%rbx
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:100
  310f9a:	e9 e3 02 00 00       	jmp    311282 <<rexx_exec::Interp>::exec_parse+0x19c2>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310f9f:	48 8b 84 24 00 01 00 	mov    0x100(%rsp),%rax
  310fa6:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310fa7:	a8 03                	test   $0x3,%al
  310fa9:	0f 85 16 27 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310faf:	48 89 c1             	mov    %rax,%rcx
  310fb2:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310fb6:	89 ca                	mov    %ecx,%edx
  310fb8:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310fbd:	48 39 91 08 0a 00 00 	cmp    %rdx,0xa08(%rcx)
  310fc4:	0f 86 fb 26 00 00    	jbe    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
<rexx_exec::parse_template::SourceText>::bytes:
  310fca:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310fcf:	48 8b 89 00 0a 00 00 	mov    0xa00(%rcx),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310fd6:	48 8d 14 52          	lea    (%rdx,%rdx,2),%rdx
  310fda:	48 c1 e2 05          	shl    $0x5,%rdx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310fde:	83 3c 11 01          	cmpl   $0x1,(%rcx,%rdx,1)
  310fe2:	0f 85 dd 26 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
  310fe8:	48 01 d1             	add    %rdx,%rcx
  310feb:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  310fef:	0f 85 d0 26 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
  310ff5:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  310ff9:	39 41 04             	cmp    %eax,0x4(%rcx)
  310ffc:	0f 85 c3 26 00 00    	jne    3136c5 <<rexx_exec::Interp>::exec_parse+0x3e05>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  311002:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  311006:	0f 85 43 03 00 00    	jne    31134f <<rexx_exec::Interp>::exec_parse+0x1a8f>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  31100c:	4c 8b 71 20          	mov    0x20(%rcx),%r14
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311010:	48 8b 59 28          	mov    0x28(%rcx),%rbx
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:100
  311014:	e9 41 03 00 00       	jmp    31135a <<rexx_exec::Interp>::exec_parse+0x1a9a>
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:931
  311019:	89 d9                	mov    %ebx,%ecx
  31101b:	c0 e9 02             	shr    $0x2,%cl
  31101e:	80 e1 07             	and    $0x7,%cl
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:77
  311021:	c6 44 24 60 02       	movb   $0x2,0x60(%rsp)
  311026:	48 89 d8             	mov    %rbx,%rax
  311029:	48 25 00 ff ff ff    	and    $0xffffffffffffff00,%rax
  31102f:	48 89 8c 24 58 01 00 	mov    %rcx,0x158(%rsp)
  311036:	00 
  311037:	0f b6 d1             	movzbl %cl,%edx
  31103a:	48 09 d0             	or     %rdx,%rax
  31103d:	48 89 44 24 61       	mov    %rax,0x61(%rsp)
  311042:	48 8b 84 24 4f 01 00 	mov    0x14f(%rsp),%rax
  311049:	00 
  31104a:	48 8d 4c 24 62       	lea    0x62(%rsp),%rcx
  31104f:	48 89 41 16          	mov    %rax,0x16(%rcx)
  311053:	f3 0f 6f 84 24 40 01 	movdqu 0x140(%rsp),%xmm0
  31105a:	00 00 
  31105c:	f3 0f 7f 41 07       	movdqu %xmm0,0x7(%rcx)
  311061:	48 89 9c 24 80 00 00 	mov    %rbx,0x80(%rsp)
  311068:	00 
  311069:	48 89 54 24 28       	mov    %rdx,0x28(%rsp)
  31106e:	80 fa 08             	cmp    $0x8,%dl
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311071:	0f 83 e5 24 00 00    	jae    31355c <<rexx_exec::Interp>::exec_parse+0x3c9c>
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:931
  311077:	48 c1 eb 08          	shr    $0x8,%rbx
  31107b:	31 f6                	xor    %esi,%esi
  31107d:	48 89 9c 24 88 01 00 	mov    %rbx,0x188(%rsp)
  311084:	00 
  311085:	49 89 c9             	mov    %rcx,%r9
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:86
  311088:	e9 fb fd ff ff       	jmp    310e88 <<rexx_exec::Interp>::exec_parse+0x15c8>
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  31108d:	48 89 e8             	mov    %rbp,%rax
  311090:	48 29 d0             	sub    %rdx,%rax
  311093:	b9 00 00 00 00       	mov    $0x0,%ecx
  311098:	48 0f 43 c8          	cmovae %rax,%rcx
  31109c:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  3110a1:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  3110a6:	48 89 8c 24 f0 00 00 	mov    %rcx,0xf0(%rsp)
  3110ad:	00 
  3110ae:	48 89 c8             	mov    %rcx,%rax
  3110b1:	49 89 ef             	mov    %rbp,%r15
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:110
  3110b4:	e9 18 05 00 00       	jmp    3115d1 <<rexx_exec::Interp>::exec_parse+0x1d11>
<usize>::saturating_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2496
  3110b9:	48 01 ea             	add    %rbp,%rdx
  3110bc:	48 c7 c0 ff ff ff ff 	mov    $0xffffffffffffffff,%rax
  3110c3:	48 0f 42 d0          	cmovb  %rax,%rdx
  3110c7:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
<usize as core::cmp::Ord>::min:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:1090
  3110cc:	48 39 d0             	cmp    %rdx,%rax
  3110cf:	48 0f 42 d0          	cmovb  %rax,%rdx
<rexx_exec::parse_template::Cursor>::forward:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:153
  3110d3:	48 39 ea             	cmp    %rbp,%rdx
  3110d6:	48 0f 47 c2          	cmova  %rdx,%rax
  3110da:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  3110df:	48 0f 46 d5          	cmovbe %rbp,%rdx
  3110e3:	eb 1f                	jmp    311104 <<rexx_exec::Interp>::exec_parse+0x1844>
<usize>::saturating_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2496
  3110e5:	48 01 ea             	add    %rbp,%rdx
  3110e8:	48 c7 c0 ff ff ff ff 	mov    $0xffffffffffffffff,%rax
  3110ef:	48 0f 42 d0          	cmovb  %rax,%rdx
  3110f3:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
<usize as core::cmp::Ord>::min:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:1090
  3110f8:	48 39 d0             	cmp    %rdx,%rax
  3110fb:	48 0f 42 d0          	cmovb  %rax,%rdx
  3110ff:	48 89 54 24 20       	mov    %rdx,0x20(%rsp)
<rexx_exec::Interp>::apply_trigger:
  311104:	48 89 94 24 f0 00 00 	mov    %rdx,0xf0(%rsp)
  31110b:	00 
  31110c:	48 89 d0             	mov    %rdx,%rax
  31110f:	49 89 ef             	mov    %rbp,%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:110
  311112:	e9 ba 04 00 00       	jmp    3115d1 <<rexx_exec::Interp>::exec_parse+0x1d11>
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  311117:	48 83 ea 01          	sub    $0x1,%rdx
  31111b:	b8 00 00 00 00       	mov    $0x0,%eax
  311120:	48 0f 43 c2          	cmovae %rdx,%rax
  311124:	48 8b 4c 24 40       	mov    0x40(%rsp),%rcx
<rexx_exec::parse_template::Cursor>::absolute:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:180
  311129:	48 39 c1             	cmp    %rax,%rcx
  31112c:	48 89 c2             	mov    %rax,%rdx
  31112f:	48 0f 42 d1          	cmovb  %rcx,%rdx
  311133:	4c 39 f8             	cmp    %r15,%rax
  311136:	48 0f 47 ca          	cmova  %rdx,%rcx
  31113a:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
  31113f:	48 0f 46 d0          	cmovbe %rax,%rdx
  311143:	48 89 94 24 f0 00 00 	mov    %rdx,0xf0(%rsp)
  31114a:	00 
  31114b:	48 89 d0             	mov    %rdx,%rax
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:109
  31114e:	e9 7e 04 00 00       	jmp    3115d1 <<rexx_exec::Interp>::exec_parse+0x1d11>
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  311153:	48 89 e8             	mov    %rbp,%rax
  311156:	48 29 d0             	sub    %rdx,%rax
  311159:	41 bf 00 00 00 00    	mov    $0x0,%r15d
  31115f:	4c 0f 43 f8          	cmovae %rax,%r15
  311163:	48 89 6c 24 20       	mov    %rbp,0x20(%rsp)
  311168:	4c 89 f8             	mov    %r15,%rax
  31116b:	4c 89 bc 24 f0 00 00 	mov    %r15,0xf0(%rsp)
  311172:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:111
  311173:	e9 59 04 00 00       	jmp    3115d1 <<rexx_exec::Interp>::exec_parse+0x1d11>
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:960
  311178:	48 8d bc 24 d0 00 00 	lea    0xd0(%rsp),%rdi
  31117f:	00 
  311180:	4c 89 f6             	mov    %r14,%rsi
  311183:	48 8b 54 24 30       	mov    0x30(%rsp),%rdx
  311188:	e8 c3 e7 e9 ff       	call   1af950 <<rexx_exec::Interp>::to_text>
  31118d:	48 8b 84 24 d0 00 00 	mov    0xd0(%rsp),%rax
  311194:	00 
  311195:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
  311199:	0f 84 d9 02 00 00    	je     311478 <<rexx_exec::Interp>::exec_parse+0x1bb8>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:962
  31119f:	48 c1 e0 38          	shl    $0x38,%rax
  3111a3:	48 8b 8c 24 00 02 00 	mov    0x200(%rsp),%rcx
  3111aa:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:964
  3111ab:	89 ce                	mov    %ecx,%esi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:962
  3111ad:	48 ba ff ff ff ff ff 	movabs $0xffffffffffffff,%rdx
  3111b4:	ff ff 00 
  3111b7:	48 21 d1             	and    %rdx,%rcx
  3111ba:	48 09 c1             	or     %rax,%rcx
  3111bd:	48 8d 94 24 d1 00 00 	lea    0xd1(%rsp),%rdx
  3111c4:	00 
  3111c5:	48 8b 42 0f          	mov    0xf(%rdx),%rax
  3111c9:	48 89 84 24 7f 01 00 	mov    %rax,0x17f(%rsp)
  3111d0:	00 
  3111d1:	0f 10 02             	movups (%rdx),%xmm0
  3111d4:	0f 29 84 24 70 01 00 	movaps %xmm0,0x170(%rsp)
  3111db:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:964
  3111dc:	48 89 c8             	mov    %rcx,%rax
  3111df:	48 c1 e8 08          	shr    $0x8,%rax
  3111e3:	48 89 84 24 88 01 00 	mov    %rax,0x188(%rsp)
  3111ea:	00 
  3111eb:	0f 28 84 24 70 01 00 	movaps 0x170(%rsp),%xmm0
  3111f2:	00 
  3111f3:	0f 29 84 24 40 01 00 	movaps %xmm0,0x140(%rsp)
  3111fa:	00 
  3111fb:	48 8b 84 24 7f 01 00 	mov    0x17f(%rsp),%rax
  311202:	00 
  311203:	48 89 84 24 4f 01 00 	mov    %rax,0x14f(%rsp)
  31120a:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:77
  31120b:	c6 44 24 60 01       	movb   $0x1,0x60(%rsp)
  311210:	48 89 c8             	mov    %rcx,%rax
  311213:	48 25 00 ff ff ff    	and    $0xffffffffffffff00,%rax
  311219:	48 89 b4 24 58 01 00 	mov    %rsi,0x158(%rsp)
  311220:	00 
  311221:	40 0f b6 d6          	movzbl %sil,%edx
  311225:	48 09 c2             	or     %rax,%rdx
  311228:	48 89 54 24 61       	mov    %rdx,0x61(%rsp)
  31122d:	48 8b 84 24 4f 01 00 	mov    0x14f(%rsp),%rax
  311234:	00 
  311235:	48 8d 54 24 62       	lea    0x62(%rsp),%rdx
  31123a:	48 89 42 16          	mov    %rax,0x16(%rdx)
  31123e:	66 0f 6f 84 24 40 01 	movdqa 0x140(%rsp),%xmm0
  311245:	00 00 
  311247:	f3 0f 7f 42 07       	movdqu %xmm0,0x7(%rdx)
  31124c:	48 8b 44 24 30       	mov    0x30(%rsp),%rax
  311251:	48 89 84 24 80 00 00 	mov    %rax,0x80(%rsp)
  311258:	00 
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311259:	4c 8b 4c 24 70       	mov    0x70(%rsp),%r9
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  31125e:	48 8b 44 24 78       	mov    0x78(%rsp),%rax
  311263:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  311268:	31 f6                	xor    %esi,%esi
  31126a:	48 89 8c 24 00 02 00 	mov    %rcx,0x200(%rsp)
  311271:	00 
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1691
  311272:	e9 11 fc ff ff       	jmp    310e88 <<rexx_exec::Interp>::exec_parse+0x15c8>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  311277:	0f b6 59 11          	movzbl 0x11(%rcx),%ebx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  31127b:	48 83 c1 12          	add    $0x12,%rcx
  31127f:	49 89 ce             	mov    %rcx,%r14
<rexx_exec::Interp>::apply_trigger:
  311282:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  311287:	49 89 c0             	mov    %rax,%r8
  31128a:	48 89 c7             	mov    %rax,%rdi
rexx_exec::parse_template::find:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:298
  31128d:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  311293:	0f 84 fd 02 00 00    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
  311299:	49 89 c0             	mov    %rax,%r8
  31129c:	48 89 c7             	mov    %rax,%rdi
  31129f:	4c 29 fb             	sub    %r15,%rbx
  3112a2:	0f 82 ee 02 00 00    	jb     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
  3112a8:	49 89 c0             	mov    %rax,%r8
  3112ab:	48 89 c7             	mov    %rax,%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:305
  3112ae:	48 39 5c 24 28       	cmp    %rbx,0x28(%rsp)
  3112b3:	0f 87 dd 02 00 00    	ja     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
  3112b9:	4d 01 fe             	add    %r15,%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:316
  3112bc:	48 83 7c 24 28 01    	cmpq   $0x1,0x28(%rsp)
  3112c2:	0f 85 0d 01 00 00    	jne    3113d5 <<rexx_exec::Interp>::exec_parse+0x1b15>
  3112c8:	41 0f b6 09          	movzbl (%r9),%ecx
<[u8]>::as_chunks::<8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:1399
  3112cc:	48 b8 01 00 00 00 00 	movabs $0x8000000000000001,%rax
  3112d3:	00 00 80 
  3112d6:	48 83 c0 f7          	add    $0xfffffffffffffff7,%rax
  3112da:	48 21 d8             	and    %rbx,%rax
<[u8]>::split_at_unchecked:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:2062
  3112dd:	89 da                	mov    %ebx,%edx
  3112df:	83 e2 07             	and    $0x7,%edx
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:160
  3112e2:	48 89 cf             	mov    %rcx,%rdi
  3112e5:	48 be 01 01 01 01 01 	movabs $0x101010101010101,%rsi
  3112ec:	01 01 01 
  3112ef:	48 0f af fe          	imul   %rsi,%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:161
  3112f3:	48 83 e3 f8          	and    $0xfffffffffffffff8,%rbx
  3112f7:	48 c7 c6 ff ff ff ff 	mov    $0xffffffffffffffff,%rsi
  3112fe:	49 ba 80 80 80 80 80 	movabs $0x8080808080808080,%r10
  311305:	80 80 80 
  311308:	49 bb ff fe fe fe fe 	movabs $0xfefefefefefefeff,%r11
  31130f:	fe fe fe 
<core::ptr::non_null::NonNull<[u8; 8]> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  311312:	48 85 db             	test   %rbx,%rbx
<core::slice::iter::Iter<[u8; 8]> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  311315:	0f 84 d7 01 00 00    	je     3114f2 <<rexx_exec::Interp>::exec_parse+0x1c32>
rexx_exec::builtin::string::find_byte:
  31131b:	4d 8b 44 f6 08       	mov    0x8(%r14,%rsi,8),%r8
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:162
  311320:	49 31 f8             	xor    %rdi,%r8
  311323:	4d 89 c1             	mov    %r8,%r9
  311326:	49 f7 d1             	not    %r9
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:163
  311329:	4d 21 d1             	and    %r10,%r9
<u64>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  31132c:	4d 01 d8             	add    %r11,%r8
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:164
  31132f:	48 ff c6             	inc    %rsi
  311332:	48 83 c3 f8          	add    $0xfffffffffffffff8,%rbx
  311336:	4d 21 c8             	and    %r9,%r8
  311339:	74 d7                	je     311312 <<rexx_exec::Interp>::exec_parse+0x1a52>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:165
  31133b:	48 c1 e6 03          	shl    $0x3,%rsi
<u64>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:178
  31133f:	f3 49 0f bc e8       	tzcnt  %r8,%rbp
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:165
  311344:	c1 ed 03             	shr    $0x3,%ebp
  311347:	48 09 f5             	or     %rsi,%rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:171
  31134a:	e9 c6 01 00 00       	jmp    311515 <<rexx_exec::Interp>::exec_parse+0x1c55>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  31134f:	0f b6 59 11          	movzbl 0x11(%rcx),%ebx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  311353:	48 83 c1 12          	add    $0x12,%rcx
  311357:	49 89 ce             	mov    %rcx,%r14
<rexx_exec::Interp>::apply_trigger:
  31135a:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  31135f:	49 89 c0             	mov    %rax,%r8
  311362:	48 89 c7             	mov    %rax,%rdi
rexx_exec::parse_template::find:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:298
  311365:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  31136b:	0f 84 25 02 00 00    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
  311371:	49 89 c0             	mov    %rax,%r8
  311374:	48 89 c7             	mov    %rax,%rdi
  311377:	4c 29 fb             	sub    %r15,%rbx
  31137a:	0f 82 16 02 00 00    	jb     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
  311380:	49 89 c0             	mov    %rax,%r8
  311383:	48 89 c7             	mov    %rax,%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:305
  311386:	48 39 5c 24 28       	cmp    %rbx,0x28(%rsp)
  31138b:	0f 87 05 02 00 00    	ja     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
  311391:	4d 01 fe             	add    %r15,%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:316
  311394:	48 83 7c 24 28 01    	cmpq   $0x1,0x28(%rsp)
  31139a:	75 7e                	jne    31141a <<rexx_exec::Interp>::exec_parse+0x1b5a>
  31139c:	41 0f b6 09          	movzbl (%r9),%ecx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  3113a0:	8d 41 bf             	lea    -0x41(%rcx),%eax
  3113a3:	3c 1a                	cmp    $0x1a,%al
  3113a5:	0f 92 c0             	setb   %al
  3113a8:	c0 e0 05             	shl    $0x5,%al
  3113ab:	08 c8                	or     %cl,%al
  3113ad:	45 31 c0             	xor    %r8d,%r8d
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::parse_template::find::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3113b0:	43 0f b6 0c 06       	movzbl (%r14,%r8,1),%ecx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  3113b5:	8d 51 bf             	lea    -0x41(%rcx),%edx
  3113b8:	80 fa 1a             	cmp    $0x1a,%dl
  3113bb:	0f 92 c2             	setb   %dl
  3113be:	c0 e2 05             	shl    $0x5,%dl
  3113c1:	08 ca                	or     %cl,%dl
rexx_exec::parse_template::find::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:320
  3113c3:	38 c2                	cmp    %al,%dl
  3113c5:	0f 84 bd 01 00 00    	je     311588 <<rexx_exec::Interp>::exec_parse+0x1cc8>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::parse_template::find::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:385
  3113cb:	49 ff c0             	inc    %r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3113ce:	4c 39 c3             	cmp    %r8,%rbx
  3113d1:	75 dd                	jne    3113b0 <<rexx_exec::Interp>::exec_parse+0x1af0>
  3113d3:	eb 35                	jmp    31140a <<rexx_exec::Interp>::exec_parse+0x1b4a>
<rexx_exec::Interp>::apply_trigger:
  3113d5:	31 ed                	xor    %ebp,%ebp
  3113d7:	4d 89 f4             	mov    %r14,%r12
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  3113da:	49 8d 3c 2e          	lea    (%r14,%rbp,1),%rdi
  3113de:	4d 89 ce             	mov    %r9,%r14
  3113e1:	4c 89 ce             	mov    %r9,%rsi
  3113e4:	48 8b 54 24 28       	mov    0x28(%rsp),%rdx
  3113e9:	ff 15 e9 6b 1c 00    	call   *0x1c6be9(%rip)        # 4d7fd8 <bcmp@GLIBC_2.2.5>
  3113ef:	85 c0                	test   %eax,%eax
  3113f1:	0f 84 1e 01 00 00    	je     311515 <<rexx_exec::Interp>::exec_parse+0x1c55>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  3113f7:	48 ff cb             	dec    %rbx
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#1}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  3113fa:	48 ff c5             	inc    %rbp
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  3113fd:	48 39 5c 24 28       	cmp    %rbx,0x28(%rsp)
  311402:	4d 89 f1             	mov    %r14,%r9
  311405:	4d 89 e6             	mov    %r12,%r14
  311408:	76 cd                	jbe    3113d7 <<rexx_exec::Interp>::exec_parse+0x1b17>
<rexx_exec::Interp>::apply_trigger:
  31140a:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  31140f:	49 89 c0             	mov    %rax,%r8
  311412:	48 89 c7             	mov    %rax,%rdi
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  311415:	e9 7c 01 00 00       	jmp    311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::position::check<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
  31141a:	48 83 7c 24 28 0f    	cmpq   $0xf,0x28(%rsp)
  311420:	0f 86 ff 00 00 00    	jbe    311525 <<rexx_exec::Interp>::exec_parse+0x1c65>
  311426:	4c 89 6c 24 20       	mov    %r13,0x20(%rsp)
  31142b:	31 ed                	xor    %ebp,%ebp
  31142d:	4d 89 f5             	mov    %r14,%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  311430:	49 8d 3c 2e          	lea    (%r14,%rbp,1),%rdi
  311434:	4c 8b 64 24 28       	mov    0x28(%rsp),%r12
<[u8]>::eq_ignore_ascii_case:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:76
  311439:	4c 89 e6             	mov    %r12,%rsi
  31143c:	4d 89 ce             	mov    %r9,%r14
  31143f:	4c 89 ca             	mov    %r9,%rdx
  311442:	4c 89 e1             	mov    %r12,%rcx
  311445:	e8 06 60 e3 ff       	call   147450 <<[u8]>::eq_ignore_ascii_case_chunks::<16>>
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3142
  31144a:	84 c0                	test   %al,%al
  31144c:	0f 85 2e 01 00 00    	jne    311580 <<rexx_exec::Interp>::exec_parse+0x1cc0>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  311452:	48 ff cb             	dec    %rbx
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  311455:	48 ff c5             	inc    %rbp
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  311458:	49 39 dc             	cmp    %rbx,%r12
  31145b:	4d 89 f1             	mov    %r14,%r9
  31145e:	4d 89 ee             	mov    %r13,%r14
  311461:	76 ca                	jbe    31142d <<rexx_exec::Interp>::exec_parse+0x1b6d>
  311463:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  311468:	49 89 c0             	mov    %rax,%r8
  31146b:	48 89 c7             	mov    %rax,%rdi
  31146e:	4c 8b 6c 24 20       	mov    0x20(%rsp),%r13
  311473:	e9 1e 01 00 00       	jmp    311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<rexx_exec::Interp>::apply_trigger:
  311478:	48 8b 94 24 00 02 00 	mov    0x200(%rsp),%rdx
  31147f:	00 
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:964
  311480:	48 89 d0             	mov    %rdx,%rax
  311483:	48 c1 e8 08          	shr    $0x8,%rax
  311487:	48 89 84 24 88 01 00 	mov    %rax,0x188(%rsp)
  31148e:	00 
  31148f:	48 8b 84 24 7f 01 00 	mov    0x17f(%rsp),%rax
  311496:	00 
  311497:	48 89 84 24 4f 01 00 	mov    %rax,0x14f(%rsp)
  31149e:	00 
  31149f:	0f 28 84 24 70 01 00 	movaps 0x170(%rsp),%xmm0
  3114a6:	00 
  3114a7:	0f 29 84 24 40 01 00 	movaps %xmm0,0x140(%rsp)
  3114ae:	00 
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:77
  3114af:	c6 44 24 60 00       	movb   $0x0,0x60(%rsp)
  3114b4:	48 89 54 24 61       	mov    %rdx,0x61(%rsp)
  3114b9:	48 8b 84 24 4f 01 00 	mov    0x14f(%rsp),%rax
  3114c0:	00 
  3114c1:	48 8d 4c 24 62       	lea    0x62(%rsp),%rcx
  3114c6:	48 89 41 16          	mov    %rax,0x16(%rcx)
  3114ca:	66 0f 6f 84 24 40 01 	movdqa 0x140(%rsp),%xmm0
  3114d1:	00 00 
  3114d3:	f3 0f 7f 41 07       	movdqu %xmm0,0x7(%rcx)
  3114d8:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
  3114dd:	48 89 b4 24 80 00 00 	mov    %rsi,0x80(%rsp)
  3114e4:	00 
  3114e5:	48 89 94 24 58 01 00 	mov    %rdx,0x158(%rsp)
  3114ec:	00 
  3114ed:	e9 76 f8 ff ff       	jmp    310d68 <<rexx_exec::Interp>::exec_parse+0x14a8>
  3114f2:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  3114f7:	49 89 f0             	mov    %rsi,%r8
  3114fa:	48 89 f7             	mov    %rsi,%rdi
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3114fd:	48 85 d2             	test   %rdx,%rdx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  311500:	0f 84 90 00 00 00    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  311506:	41 38 0c 06          	cmp    %cl,(%r14,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  31150a:	0f 85 22 14 00 00    	jne    312932 <<rexx_exec::Interp>::exec_parse+0x3072>
rexx_exec::builtin::string::find_byte:
  311510:	31 ed                	xor    %ebp,%ebp
<core::option::Option<usize>>::map::<usize, rexx_exec::builtin::string::find_byte::{closure#1}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  311512:	48 09 c5             	or     %rax,%rbp
rexx_exec::parse_template::find::{closure#3}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:327
  311515:	4c 01 fd             	add    %r15,%rbp
  311518:	4c 8b 44 24 28       	mov    0x28(%rsp),%r8
<rexx_exec::parse_template::Cursor>::match_at:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:233
  31151d:	49 01 e8             	add    %rbp,%r8
  311520:	48 89 ef             	mov    %rbp,%rdi
  311523:	eb 71                	jmp    311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<rexx_exec::Interp>::apply_trigger:
  311525:	45 31 c0             	xor    %r8d,%r8d
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  311528:	48 ff cb             	dec    %rbx
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  31152b:	49 8d 46 01          	lea    0x1(%r14),%rax
  31152f:	31 c9                	xor    %ecx,%ecx
<[u8]>::eq_ignore_ascii_case_simple:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:93
  311531:	48 39 4c 24 28       	cmp    %rcx,0x28(%rsp)
  311536:	74 50                	je     311588 <<rexx_exec::Interp>::exec_parse+0x1cc8>
<u8>::to_ascii_lowercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:711
  311538:	41 0f b6 14 0e       	movzbl (%r14,%rcx,1),%edx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  31153d:	8d 72 bf             	lea    -0x41(%rdx),%esi
  311540:	40 80 fe 1a          	cmp    $0x1a,%sil
  311544:	40 0f 92 c6          	setb   %sil
  311548:	40 c0 e6 05          	shl    $0x5,%sil
  31154c:	40 08 d6             	or     %dl,%sil
<u8>::to_ascii_lowercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:711
  31154f:	41 0f b6 14 09       	movzbl (%r9,%rcx,1),%edx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  311554:	8d 7a bf             	lea    -0x41(%rdx),%edi
  311557:	40 80 ff 1a          	cmp    $0x1a,%dil
  31155b:	40 0f 92 c7          	setb   %dil
  31155f:	40 c0 e7 05          	shl    $0x5,%dil
  311563:	40 08 d7             	or     %dl,%dil
<[u8]>::eq_ignore_ascii_case_simple:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:94
  311566:	48 ff c1             	inc    %rcx
<u8>::eq_ignore_ascii_case:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:736
  311569:	40 38 fe             	cmp    %dil,%sil
<[u8]>::eq_ignore_ascii_case_simple:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:94
  31156c:	74 c3                	je     311531 <<rexx_exec::Interp>::exec_parse+0x1c71>
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  31156e:	49 ff c0             	inc    %r8
  311571:	49 89 c6             	mov    %rax,%r14
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  311574:	48 39 5c 24 28       	cmp    %rbx,0x28(%rsp)
  311579:	76 ad                	jbe    311528 <<rexx_exec::Interp>::exec_parse+0x1c68>
  31157b:	e9 8a fe ff ff       	jmp    31140a <<rexx_exec::Interp>::exec_parse+0x1b4a>
rexx_exec::parse_template::find:
  311580:	49 89 e8             	mov    %rbp,%r8
  311583:	4c 8b 6c 24 20       	mov    0x20(%rsp),%r13
  311588:	4c 89 c7             	mov    %r8,%rdi
rexx_exec::parse_template::find::{closure#3}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:327
  31158b:	4c 01 ff             	add    %r15,%rdi
  31158e:	4c 8b 44 24 28       	mov    0x28(%rsp),%r8
<rexx_exec::parse_template::Cursor>::match_at:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:233
  311593:	49 01 f8             	add    %rdi,%r8
<rexx_exec::Interp>::apply_trigger:
  311596:	4c 89 c3             	mov    %r8,%rbx
  311599:	48 89 7c 24 20       	mov    %rdi,0x20(%rsp)
  31159e:	80 7c 24 60 01       	cmpb   $0x1,0x60(%rsp)
  3115a3:	0f 95 c0             	setne  %al
  3115a6:	8b 4c 24 30          	mov    0x30(%rsp),%ecx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:86
  3115aa:	08 c1                	or     %al,%cl
  3115ac:	75 13                	jne    3115c1 <<rexx_exec::Interp>::exec_parse+0x1d01>
  3115ae:	48 83 7c 24 68 00    	cmpq   $0x0,0x68(%rsp)
  3115b4:	74 0b                	je     3115c1 <<rexx_exec::Interp>::exec_parse+0x1d01>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3115b6:	48 8b 7c 24 70       	mov    0x70(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3115bb:	ff 15 d7 69 1c 00    	call   *0x1c69d7(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  3115c1:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
  3115c6:	48 89 84 24 f0 00 00 	mov    %rax,0xf0(%rsp)
  3115cd:	00 
  3115ce:	48 89 d8             	mov    %rbx,%rax
  3115d1:	48 89 84 24 d8 01 00 	mov    %rax,0x1d8(%rsp)
  3115d8:	00 
<alloc::vec::Vec<core::option::Option<rexx_parse::ast::Expr>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3115d9:	49 8b 45 10          	mov    0x10(%r13),%rax
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  3115dd:	48 89 c2             	mov    %rax,%rdx
  3115e0:	48 83 ea 01          	sub    $0x1,%rdx
  3115e4:	b9 00 00 00 00       	mov    $0x0,%ecx
  3115e9:	48 0f 42 d1          	cmovb  %rcx,%rdx
  3115ed:	48 89 94 24 20 01 00 	mov    %rdx,0x120(%rsp)
  3115f4:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::Expr>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3115f5:	48 85 c0             	test   %rax,%rax
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3115f8:	0f 84 72 f2 ff ff    	je     310870 <<rexx_exec::Interp>::exec_parse+0xfb0>
<rexx_exec::Interp>::assign_targets:
  3115fe:	4d 8b 4d 08          	mov    0x8(%r13),%r9
  311602:	48 8d 04 80          	lea    (%rax,%rax,4),%rax
  311606:	48 c1 e0 04          	shl    $0x4,%rax
  31160a:	4c 01 c8             	add    %r9,%rax
  31160d:	48 89 84 24 e8 01 00 	mov    %rax,0x1e8(%rsp)
  311614:	00 
  311615:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::traced_mode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:111
  31161a:	41 0f b6 8c 24 35 05 	movzbl 0x535(%r12),%ecx
  311621:	00 00 
  311623:	41 0f b6 84 24 2e 05 	movzbl 0x52e(%r12),%eax
  31162a:	00 00 
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31162c:	80 f1 01             	xor    $0x1,%cl
  31162f:	89 ca                	mov    %ecx,%edx
  311631:	20 c2                	and    %al,%dl
  311633:	41 22 8c 24 2d 05 00 	and    0x52d(%r12),%cl
  31163a:	00 
  31163b:	88 54 24 3f          	mov    %dl,0x3f(%rsp)
  31163f:	08 ca                	or     %cl,%dl
  311641:	88 54 24 4f          	mov    %dl,0x4f(%rsp)
  311645:	80 f1 01             	xor    $0x1,%cl
  311648:	08 c1                	or     %al,%cl
  31164a:	88 4c 24 4e          	mov    %cl,0x4e(%rsp)
  31164e:	48 c7 44 24 28 00 00 	movq   $0x0,0x28(%rsp)
  311655:	00 00 
  311657:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
  31165c:	eb 51                	jmp    3116af <<rexx_exec::Interp>::exec_parse+0x1def>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  31165e:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  311665:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  311666:	4b 8d 0c 7f          	lea    (%r15,%r15,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  31166a:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  31166e:	48 8b 94 24 18 01 00 	mov    0x118(%rsp),%rdx
  311675:	00 
  311676:	48 89 54 c8 08       	mov    %rdx,0x8(%rax,%rcx,8)
  31167b:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  311682:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  311684:	49 ff c7             	inc    %r15
  311687:	4d 89 bc 24 e0 0b 00 	mov    %r15,0xbe0(%r12)
  31168e:	00 
<rexx_exec::Interp>::assign_targets:
  31168f:	90                   	nop
  311690:	4c 8b 4c 24 30       	mov    0x30(%rsp),%r9
<core::iter::adapters::enumerate::Enumerate<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>>> as core::iter::traits::iterator::Iterator>::next:
  311695:	49 83 c1 50          	add    $0x50,%r9
  311699:	48 ff 44 24 28       	incq   0x28(%rsp)
  31169e:	49 89 df             	mov    %rbx,%r15
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::Expr>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3116a1:	4c 3b 8c 24 e8 01 00 	cmp    0x1e8(%rsp),%r9
  3116a8:	00 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3116a9:	0f 84 ce f1 ff ff    	je     31087d <<rexx_exec::Interp>::exec_parse+0xfbd>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:742
  3116af:	49 83 39 ff          	cmpq   $0xffffffffffffffff,(%r9)
  3116b3:	4c 89 4c 24 30       	mov    %r9,0x30(%rsp)
  3116b8:	0f 84 b2 00 00 00    	je     311770 <<rexx_exec::Interp>::exec_parse+0x1eb0>
  3116be:	48 8b 84 24 98 01 00 	mov    0x198(%rsp),%rax
  3116c5:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  3116c6:	48 85 c0             	test   %rax,%rax
  3116c9:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  3116ce:	0f 84 ec 00 00 00    	je     3117c0 <<rexx_exec::Interp>::exec_parse+0x1f00>
  3116d4:	48 83 f8 01          	cmp    $0x1,%rax
  3116d8:	0f 85 b2 01 00 00    	jne    311890 <<rexx_exec::Interp>::exec_parse+0x1fd0>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  3116de:	0f b6 84 24 00 01 00 	movzbl 0x100(%rsp),%eax
  3116e5:	00 
  3116e6:	4c 8d ac 24 01 01 00 	lea    0x101(%rsp),%r13
  3116ed:	00 
  3116ee:	48 83 f8 08          	cmp    $0x8,%rax
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  3116f2:	0f 83 4c 19 00 00    	jae    313044 <<rexx_exec::Interp>::exec_parse+0x3784>
<rexx_exec::Interp>::assign_targets:
  3116f8:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
  3116fd:	48 3b 8c 24 20 01 00 	cmp    0x120(%rsp),%rcx
  311704:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  311705:	0f 85 d8 00 00 00    	jne    3117e3 <<rexx_exec::Interp>::exec_parse+0x1f23>
  31170b:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
  311710:	48 89 cb             	mov    %rcx,%rbx
  311713:	49 89 ce             	mov    %rcx,%r14
  311716:	49 39 cf             	cmp    %rcx,%r15
<rexx_exec::parse_template::Cursor>::remainder:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:286
  311719:	0f 83 e1 04 00 00    	jae    311c00 <<rexx_exec::Interp>::exec_parse+0x2340>
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  31171f:	4d 89 f4             	mov    %r14,%r12
  311722:	4d 29 fc             	sub    %r15,%r12
  311725:	0f 82 30 18 00 00    	jb     312f5b <<rexx_exec::Interp>::exec_parse+0x369b>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:408
  31172b:	49 39 c6             	cmp    %rax,%r14
  31172e:	0f 87 27 18 00 00    	ja     312f5b <<rexx_exec::Interp>::exec_parse+0x369b>
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  311734:	4d 01 fd             	add    %r15,%r13
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:131
  311737:	49 83 fc 07          	cmp    $0x7,%r12
  31173b:	0f 86 a8 02 00 00    	jbe    3119e9 <<rexx_exec::Interp>::exec_parse+0x2129>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:51
  311741:	49 83 fc 37          	cmp    $0x37,%r12
  311745:	0f 83 96 03 00 00    	jae    311ae1 <<rexx_exec::Interp>::exec_parse+0x2221>
<core::mem::maybe_uninit::MaybeUninit<u8>>::write:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/maybe_uninit.rs:575
  31174b:	48 8d 7c 24 6a       	lea    0x6a(%rsp),%rdi
  311750:	4c 89 ee             	mov    %r13,%rsi
  311753:	4c 89 e2             	mov    %r12,%rdx
  311756:	ff 15 6c 68 1c 00    	call   *0x1c686c(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:61
  31175c:	c6 44 24 68 00       	movb   $0x0,0x68(%rsp)
  311761:	44 88 64 24 69       	mov    %r12b,0x69(%rsp)
  311766:	e9 b1 03 00 00       	jmp    311b1c <<rexx_exec::Interp>::exec_parse+0x225c>
<rexx_exec::Interp>::assign_targets:
  31176b:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
  311770:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
  311775:	48 3b 84 24 20 01 00 	cmp    0x120(%rsp),%rax
  31177c:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:798
  31177d:	0f 85 92 01 00 00    	jne    311915 <<rexx_exec::Interp>::exec_parse+0x2055>
  311783:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
<rexx_exec::parse_template::Cursor>::remainder:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:286
  311788:	49 39 c7             	cmp    %rax,%r15
  31178b:	48 89 c3             	mov    %rax,%rbx
  31178e:	49 0f 47 df          	cmova  %r15,%rbx
  311792:	41 b8 00 00 00 00    	mov    $0x0,%r8d
  311798:	4c 0f 42 c0          	cmovb  %rax,%r8
  31179c:	b8 00 00 00 00       	mov    $0x0,%eax
  3117a1:	4c 0f 43 f8          	cmovae %rax,%r15
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  3117a5:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  3117aa:	0f 84 e5 fe ff ff    	je     311695 <<rexx_exec::Interp>::exec_parse+0x1dd5>
  3117b0:	e9 e5 07 00 00       	jmp    311f9a <<rexx_exec::Interp>::exec_parse+0x26da>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
  3117b5:	66 66 2e 0f 1f 84 00 	data16 cs nopw 0x0(%rax,%rax,1)
  3117bc:	00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3117c0:	4c 8b ac 24 00 01 00 	mov    0x100(%rsp),%r13
  3117c7:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  3117c8:	48 8b 84 24 08 01 00 	mov    0x108(%rsp),%rax
  3117cf:	00 
  3117d0:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
<rexx_exec::Interp>::assign_targets:
  3117d5:	48 3b 8c 24 20 01 00 	cmp    0x120(%rsp),%rcx
  3117dc:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  3117dd:	0f 84 28 ff ff ff    	je     31170b <<rexx_exec::Interp>::exec_parse+0x1e4b>
  3117e3:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  3117e8:	0f 83 12 04 00 00    	jae    311c00 <<rexx_exec::Interp>::exec_parse+0x2340>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  3117ee:	49 39 f7             	cmp    %rsi,%r15
  3117f1:	73 32                	jae    311825 <<rexx_exec::Interp>::exec_parse+0x1f65>
  3117f3:	49 39 c7             	cmp    %rax,%r15
  3117f6:	48 89 c7             	mov    %rax,%rdi
  3117f9:	49 0f 47 ff          	cmova  %r15,%rdi
  3117fd:	eb 0d                	jmp    31180c <<rexx_exec::Interp>::exec_parse+0x1f4c>
  3117ff:	90                   	nop
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:257
  311800:	49 ff c7             	inc    %r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  311803:	4c 39 fe             	cmp    %r15,%rsi
  311806:	0f 84 e0 03 00 00    	je     311bec <<rexx_exec::Interp>::exec_parse+0x232c>
  31180c:	4c 39 ff             	cmp    %r15,%rdi
  31180f:	0f 84 ad 1d 00 00    	je     3135c2 <<rexx_exec::Interp>::exec_parse+0x3d02>
  311815:	43 0f b6 4c 3d 00    	movzbl 0x0(%r13,%r15,1),%ecx
  31181b:	83 f9 20             	cmp    $0x20,%ecx
rexx_exec::parse_template::is_blank:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:31
  31181e:	74 e0                	je     311800 <<rexx_exec::Interp>::exec_parse+0x1f40>
  311820:	83 f9 09             	cmp    $0x9,%ecx
  311823:	74 db                	je     311800 <<rexx_exec::Interp>::exec_parse+0x1f40>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311825:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  31182a:	0f 83 d0 03 00 00    	jae    311c00 <<rexx_exec::Interp>::exec_parse+0x2340>
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2489
  311830:	49 39 c7             	cmp    %rax,%r15
  311833:	48 89 c7             	mov    %rax,%rdi
  311836:	49 0f 47 ff          	cmova  %r15,%rdi
  31183a:	4c 89 fb             	mov    %r15,%rbx
  31183d:	4c 8b 74 24 20       	mov    0x20(%rsp),%r14
<usize as core::cmp::PartialOrd>::lt:
  311842:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  311849:	1f 84 00 00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2144
  311850:	49 39 de             	cmp    %rbx,%r14
<core::ops::range::Range<usize> as core::iter::range::RangeIteratorImpl>::spec_next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1100
  311853:	0f 84 1a 07 00 00    	je     311f73 <<rexx_exec::Interp>::exec_parse+0x26b3>
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  311859:	48 39 df             	cmp    %rbx,%rdi
  31185c:	0f 84 51 1d 00 00    	je     3135b3 <<rexx_exec::Interp>::exec_parse+0x3cf3>
  311862:	41 0f b6 4c 1d 00    	movzbl 0x0(%r13,%rbx,1),%ecx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  311868:	48 ff c3             	inc    %rbx
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  31186b:	83 f9 09             	cmp    $0x9,%ecx
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  31186e:	74 05                	je     311875 <<rexx_exec::Interp>::exec_parse+0x1fb5>
  311870:	83 f9 20             	cmp    $0x20,%ecx
  311873:	75 db                	jne    311850 <<rexx_exec::Interp>::exec_parse+0x1f90>
  311875:	4c 8d 73 ff          	lea    -0x1(%rbx),%r14
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311879:	4d 89 f4             	mov    %r14,%r12
  31187c:	4d 29 fc             	sub    %r15,%r12
  31187f:	0f 83 a6 fe ff ff    	jae    31172b <<rexx_exec::Interp>::exec_parse+0x1e6b>
  311885:	e9 d1 16 00 00       	jmp    312f5b <<rexx_exec::Interp>::exec_parse+0x369b>
<rexx_exec::parse_template::SourceText>::bytes:
  31188a:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311890:	48 8b 84 24 00 01 00 	mov    0x100(%rsp),%rax
  311897:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  311898:	a8 03                	test   $0x3,%al
  31189a:	0f 85 f6 1c 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  3118a0:	48 89 c1             	mov    %rax,%rcx
  3118a3:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  3118a7:	89 ca                	mov    %ecx,%edx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  3118a9:	49 39 94 24 08 0a 00 	cmp    %rdx,0xa08(%r12)
  3118b0:	00 
  3118b1:	0f 86 df 1c 00 00    	jbe    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  3118b7:	49 8b 8c 24 00 0a 00 	mov    0xa00(%r12),%rcx
  3118be:	00 
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  3118bf:	48 8d 14 52          	lea    (%rdx,%rdx,2),%rdx
  3118c3:	48 c1 e2 05          	shl    $0x5,%rdx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  3118c7:	83 3c 11 01          	cmpl   $0x1,(%rcx,%rdx,1)
  3118cb:	0f 85 c5 1c 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
  3118d1:	48 01 d1             	add    %rdx,%rcx
  3118d4:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  3118d8:	0f 85 b8 1c 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
  3118de:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  3118e2:	39 41 04             	cmp    %eax,0x4(%rcx)
  3118e5:	0f 85 ab 1c 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  3118eb:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  3118ef:	0f 85 13 01 00 00    	jne    311a08 <<rexx_exec::Interp>::exec_parse+0x2148>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3118f5:	4c 8b 69 20          	mov    0x20(%rcx),%r13
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  3118f9:	48 8b 41 28          	mov    0x28(%rcx),%rax
  3118fd:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
<rexx_exec::Interp>::assign_targets:
  311902:	48 3b 8c 24 20 01 00 	cmp    0x120(%rsp),%rcx
  311909:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  31190a:	0f 85 d3 fe ff ff    	jne    3117e3 <<rexx_exec::Interp>::exec_parse+0x1f23>
  311910:	e9 f6 fd ff ff       	jmp    31170b <<rexx_exec::Interp>::exec_parse+0x1e4b>
<rexx_exec::Interp>::assign_targets:
  311915:	48 8b 84 24 98 01 00 	mov    0x198(%rsp),%rax
  31191c:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  31191d:	48 85 c0             	test   %rax,%rax
  311920:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  311925:	0f 84 00 01 00 00    	je     311a2b <<rexx_exec::Interp>::exec_parse+0x216b>
  31192b:	48 83 f8 01          	cmp    $0x1,%rax
  31192f:	0f 85 2f 01 00 00    	jne    311a64 <<rexx_exec::Interp>::exec_parse+0x21a4>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  311935:	0f b6 84 24 00 01 00 	movzbl 0x100(%rsp),%eax
  31193c:	00 
  31193d:	48 8d 8c 24 01 01 00 	lea    0x101(%rsp),%rcx
  311944:	00 
  311945:	48 83 f8 08          	cmp    $0x8,%rax
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311949:	0f 83 f5 16 00 00    	jae    313044 <<rexx_exec::Interp>::exec_parse+0x3784>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  31194f:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311954:	0f 83 ec 00 00 00    	jae    311a46 <<rexx_exec::Interp>::exec_parse+0x2186>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  31195a:	49 39 f7             	cmp    %rsi,%r15
  31195d:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
  311962:	73 30                	jae    311994 <<rexx_exec::Interp>::exec_parse+0x20d4>
  311964:	49 39 c7             	cmp    %rax,%r15
  311967:	48 89 c7             	mov    %rax,%rdi
  31196a:	49 0f 47 ff          	cmova  %r15,%rdi
  31196e:	eb 0c                	jmp    31197c <<rexx_exec::Interp>::exec_parse+0x20bc>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:257
  311970:	49 ff c7             	inc    %r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  311973:	4c 39 fe             	cmp    %r15,%rsi
  311976:	0f 84 d3 05 00 00    	je     311f4f <<rexx_exec::Interp>::exec_parse+0x268f>
  31197c:	4c 39 ff             	cmp    %r15,%rdi
  31197f:	0f 84 3d 1c 00 00    	je     3135c2 <<rexx_exec::Interp>::exec_parse+0x3d02>
  311985:	42 0f b6 14 39       	movzbl (%rcx,%r15,1),%edx
  31198a:	83 fa 20             	cmp    $0x20,%edx
rexx_exec::parse_template::is_blank:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:31
  31198d:	74 e1                	je     311970 <<rexx_exec::Interp>::exec_parse+0x20b0>
  31198f:	83 fa 09             	cmp    $0x9,%edx
  311992:	74 dc                	je     311970 <<rexx_exec::Interp>::exec_parse+0x20b0>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311994:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311999:	0f 83 be 05 00 00    	jae    311f5d <<rexx_exec::Interp>::exec_parse+0x269d>
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2489
  31199f:	49 39 c7             	cmp    %rax,%r15
  3119a2:	48 89 c7             	mov    %rax,%rdi
  3119a5:	49 0f 47 ff          	cmova  %r15,%rdi
  3119a9:	4c 89 fb             	mov    %r15,%rbx
<usize as core::cmp::PartialOrd>::lt:
  3119ac:	0f 1f 40 00          	nopl   0x0(%rax)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2144
  3119b0:	48 39 5c 24 20       	cmp    %rbx,0x20(%rsp)
<core::ops::range::Range<usize> as core::iter::range::RangeIteratorImpl>::spec_next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1100
  3119b5:	0f 84 cc 05 00 00    	je     311f87 <<rexx_exec::Interp>::exec_parse+0x26c7>
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  3119bb:	48 39 df             	cmp    %rbx,%rdi
  3119be:	0f 84 ef 1b 00 00    	je     3135b3 <<rexx_exec::Interp>::exec_parse+0x3cf3>
  3119c4:	0f b6 14 19          	movzbl (%rcx,%rbx,1),%edx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  3119c8:	48 ff c3             	inc    %rbx
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  3119cb:	83 fa 09             	cmp    $0x9,%edx
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  3119ce:	74 05                	je     3119d5 <<rexx_exec::Interp>::exec_parse+0x2115>
  3119d0:	83 fa 20             	cmp    $0x20,%edx
  3119d3:	75 db                	jne    3119b0 <<rexx_exec::Interp>::exec_parse+0x20f0>
  3119d5:	4c 8d 43 ff          	lea    -0x1(%rbx),%r8
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  3119d9:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  3119de:	0f 84 b1 fc ff ff    	je     311695 <<rexx_exec::Interp>::exec_parse+0x1dd5>
  3119e4:	e9 b1 05 00 00       	jmp    311f9a <<rexx_exec::Interp>::exec_parse+0x26da>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:134
  3119e9:	48 c7 44 24 60 00 00 	movq   $0x0,0x60(%rsp)
  3119f0:	00 00 
  3119f2:	bd 03 00 00 00       	mov    $0x3,%ebp
<core::iter::adapters::zip::Zip<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>> as core::iter::adapters::zip::ZipImpl<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>>>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/zip.rs:300
  3119f7:	4d 39 fe             	cmp    %r15,%r14
  3119fa:	0f 85 a8 01 00 00    	jne    311ba8 <<rexx_exec::Interp>::exec_parse+0x22e8>
<rexx_exec::Interp>::take_piece:
  311a00:	4d 89 fe             	mov    %r15,%r14
  311a03:	e9 c2 01 00 00       	jmp    311bca <<rexx_exec::Interp>::exec_parse+0x230a>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  311a08:	0f b6 41 11          	movzbl 0x11(%rcx),%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  311a0c:	48 83 c1 12          	add    $0x12,%rcx
  311a10:	49 89 cd             	mov    %rcx,%r13
  311a13:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
<rexx_exec::Interp>::assign_targets:
  311a18:	48 3b 8c 24 20 01 00 	cmp    0x120(%rsp),%rcx
  311a1f:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  311a20:	0f 84 e5 fc ff ff    	je     31170b <<rexx_exec::Interp>::exec_parse+0x1e4b>
  311a26:	e9 b8 fd ff ff       	jmp    3117e3 <<rexx_exec::Interp>::exec_parse+0x1f23>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311a2b:	48 8b 8c 24 00 01 00 	mov    0x100(%rsp),%rcx
  311a32:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311a33:	48 8b 84 24 08 01 00 	mov    0x108(%rsp),%rax
  311a3a:	00 
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311a3b:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311a40:	0f 82 14 ff ff ff    	jb     31195a <<rexx_exec::Interp>::exec_parse+0x209a>
<rexx_exec::Interp>::assign_targets:
  311a46:	4c 89 fb             	mov    %r15,%rbx
  311a49:	45 31 ff             	xor    %r15d,%r15d
  311a4c:	45 31 c0             	xor    %r8d,%r8d
  311a4f:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  311a54:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  311a59:	0f 84 36 fc ff ff    	je     311695 <<rexx_exec::Interp>::exec_parse+0x1dd5>
  311a5f:	e9 36 05 00 00       	jmp    311f9a <<rexx_exec::Interp>::exec_parse+0x26da>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311a64:	48 8b 84 24 00 01 00 	mov    0x100(%rsp),%rax
  311a6b:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  311a6c:	a8 03                	test   $0x3,%al
  311a6e:	0f 85 22 1b 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  311a74:	48 89 c1             	mov    %rax,%rcx
  311a77:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311a7b:	89 c9                	mov    %ecx,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  311a7d:	49 39 8c 24 08 0a 00 	cmp    %rcx,0xa08(%r12)
  311a84:	00 
  311a85:	0f 86 0b 1b 00 00    	jbe    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311a8b:	49 8b 94 24 00 0a 00 	mov    0xa00(%r12),%rdx
  311a92:	00 
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  311a93:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  311a97:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311a9b:	83 3c 0a 01          	cmpl   $0x1,(%rdx,%rcx,1)
  311a9f:	0f 85 f1 1a 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
  311aa5:	48 01 ca             	add    %rcx,%rdx
  311aa8:	80 7a 08 00          	cmpb   $0x0,0x8(%rdx)
  311aac:	0f 85 e4 1a 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
  311ab2:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  311ab6:	39 42 04             	cmp    %eax,0x4(%rdx)
  311ab9:	0f 85 d7 1a 00 00    	jne    313596 <<rexx_exec::Interp>::exec_parse+0x3cd6>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  311abf:	80 7a 10 01          	cmpb   $0x1,0x10(%rdx)
  311ac3:	0f 85 6b 04 00 00    	jne    311f34 <<rexx_exec::Interp>::exec_parse+0x2674>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311ac9:	48 8b 4a 20          	mov    0x20(%rdx),%rcx
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311acd:	48 8b 42 28          	mov    0x28(%rdx),%rax
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311ad1:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311ad6:	0f 83 6a ff ff ff    	jae    311a46 <<rexx_exec::Interp>::exec_parse+0x2186>
  311adc:	e9 79 fe ff ff       	jmp    31195a <<rexx_exec::Interp>::exec_parse+0x209a>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  311ae1:	4c 89 e7             	mov    %r12,%rdi
  311ae4:	ff 15 ce 64 1c 00    	call   *0x1c64ce(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  311aea:	48 85 c0             	test   %rax,%rax
  311aed:	0f 84 c3 1b 00 00    	je     3136b6 <<rexx_exec::Interp>::exec_parse+0x3df6>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  311af3:	48 89 c5             	mov    %rax,%rbp
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  311af6:	48 89 c7             	mov    %rax,%rdi
  311af9:	4c 89 ee             	mov    %r13,%rsi
  311afc:	4c 89 e2             	mov    %r12,%rdx
  311aff:	ff 15 c3 64 1c 00    	call   *0x1c64c3(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:66
  311b05:	c6 44 24 68 01       	movb   $0x1,0x68(%rsp)
  311b0a:	4c 89 64 24 70       	mov    %r12,0x70(%rsp)
  311b0f:	48 89 6c 24 78       	mov    %rbp,0x78(%rsp)
  311b14:	4c 89 a4 24 80 00 00 	mov    %r12,0x80(%rsp)
  311b1b:	00 
<rexx_exec::Interp>::text_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:230
  311b1c:	48 c7 84 24 a0 00 00 	movq   $0x0,0xa0(%rsp)
  311b23:	00 00 00 00 00 
  311b28:	c6 44 24 60 00       	movb   $0x0,0x60(%rsp)
  311b2d:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
  311b32:	31 f6                	xor    %esi,%esi
  311b34:	48 8d 54 24 60       	lea    0x60(%rsp),%rdx
  311b39:	e8 22 a7 e7 ff       	call   18c260 <<rexx_exec::Interp>::alloc_with>
  311b3e:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
<rexx_exec::Interp>::take_piece:
  311b43:	48 89 c5             	mov    %rax,%rbp
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::size_hint:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1190
  311b46:	4c 89 f1             	mov    %r14,%rcx
  311b49:	4c 29 f9             	sub    %r15,%rcx
  311b4c:	b8 00 00 00 00       	mov    $0x0,%eax
  311b51:	48 0f 43 c1          	cmovae %rcx,%rax
<rexx_exec::Interp>::charge_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:104
  311b55:	48 83 f8 37          	cmp    $0x37,%rax
  311b59:	72 76                	jb     311bd1 <<rexx_exec::Interp>::exec_parse+0x2311>
  311b5b:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_core::heap::Heap>::hold_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:183
  311b60:	49 01 84 24 a0 0a 00 	add    %rax,0xaa0(%r12)
  311b67:	00 
<rexx_core::heap::Heap>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:176
  311b68:	49 03 84 24 90 0a 00 	add    0xa90(%r12),%rax
  311b6f:	00 
  311b70:	49 89 84 24 90 0a 00 	mov    %rax,0xa90(%r12)
  311b77:	00 
<rexx_exec::Interp>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:3249
  311b78:	49 3b 84 24 c8 1f 00 	cmp    0x1fc8(%r12),%rax
  311b7f:	00 
  311b80:	0f 93 c0             	setae  %al
  311b83:	41 08 84 24 ff 1f 00 	or     %al,0x1fff(%r12)
  311b8a:	00 
  311b8b:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311b8e:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  311b95:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311b96:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  311b9d:	0f 84 84 00 00 00    	je     311c27 <<rexx_exec::Interp>::exec_parse+0x2367>
  311ba3:	e9 8c 00 00 00       	jmp    311c34 <<rexx_exec::Interp>::exec_parse+0x2374>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:136
  311ba8:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  311bad:	4c 89 ee             	mov    %r13,%rsi
  311bb0:	4c 89 e2             	mov    %r12,%rdx
  311bb3:	ff 15 0f 64 1c 00    	call   *0x1c640f(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:138
  311bb9:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:140
  311bbe:	48 c1 e0 08          	shl    $0x8,%rax
  311bc2:	4a 8d 2c a0          	lea    (%rax,%r12,4),%rbp
  311bc6:	48 83 c5 03          	add    $0x3,%rbp
<rexx_exec::Interp>::assign_targets:
  311bca:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311bcf:	eb 3d                	jmp    311c0e <<rexx_exec::Interp>::exec_parse+0x234e>
  311bd1:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311bd6:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311bd9:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  311be0:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311be1:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  311be8:	74 3d                	je     311c27 <<rexx_exec::Interp>::exec_parse+0x2367>
  311bea:	eb 48                	jmp    311c34 <<rexx_exec::Interp>::exec_parse+0x2374>
<rexx_exec::Interp>::assign_targets:
  311bec:	49 89 f7             	mov    %rsi,%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311bef:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311bf4:	0f 82 36 fc ff ff    	jb     311830 <<rexx_exec::Interp>::exec_parse+0x1f70>
<rexx_exec::Interp>::assign_targets:
  311bfa:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
  311c00:	bd 03 00 00 00       	mov    $0x3,%ebp
  311c05:	4c 89 fb             	mov    %r15,%rbx
  311c08:	45 31 ff             	xor    %r15d,%r15d
  311c0b:	45 31 f6             	xor    %r14d,%r14d
  311c0e:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
  311c13:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311c16:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  311c1d:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311c1e:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  311c25:	75 0d                	jne    311c34 <<rexx_exec::Interp>::exec_parse+0x2374>
  311c27:	48 8b bc 24 90 01 00 	mov    0x190(%rsp),%rdi
  311c2e:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  311c2f:	e8 dc a2 e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  311c34:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311c39:	48 8b 81 e8 0a 00 00 	mov    0xae8(%rcx),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  311c40:	4a 89 2c e0          	mov    %rbp,(%rax,%r12,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  311c44:	49 ff c4             	inc    %r12
  311c47:	4c 89 a1 f0 0a 00 00 	mov    %r12,0xaf0(%rcx)
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:760
  311c4e:	48 b8 01 00 00 00 00 	movabs $0x8000000000000001,%rax
  311c55:	00 00 80 
  311c58:	48 ff c0             	inc    %rax
  311c5b:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
  311c60:	48 39 01             	cmp    %rax,(%rcx)
  311c63:	0f 85 c7 00 00 00    	jne    311d30 <<rexx_exec::Interp>::exec_parse+0x2470>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:761
  311c69:	8b 41 08             	mov    0x8(%rcx),%eax
<usize as core::slice::index::SliceIndex<[core::option::Option<usize>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  311c6c:	48 39 84 24 50 02 00 	cmp    %rax,0x250(%rsp)
  311c73:	00 
  311c74:	0f 86 b6 00 00 00    	jbe    311d30 <<rexx_exec::Interp>::exec_parse+0x2470>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  311c7a:	48 89 c1             	mov    %rax,%rcx
  311c7d:	48 c1 e1 04          	shl    $0x4,%rcx
  311c81:	48 8b 94 24 40 02 00 	mov    0x240(%rsp),%rdx
  311c88:	00 
<core::option::Option<usize>>::map::<(rexx_parse::token::symbols::SymbolId, usize), <rexx_exec::Interp>::assign_targets::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  311c89:	83 3c 0a 01          	cmpl   $0x1,(%rdx,%rcx,1)
  311c8d:	0f 85 9d 00 00 00    	jne    311d30 <<rexx_exec::Interp>::exec_parse+0x2470>
<rexx_exec::Code>::slot_for:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1263
  311c93:	48 01 d1             	add    %rdx,%rcx
<core::option::Option<&core::option::Option<usize>>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  311c96:	48 8b 49 08          	mov    0x8(%rcx),%rcx
  311c9a:	48 89 8c 24 e0 01 00 	mov    %rcx,0x1e0(%rsp)
  311ca1:	00 
  311ca2:	41 b4 01             	mov    $0x1,%r12b
  311ca5:	89 84 24 fc 01 00 00 	mov    %eax,0x1fc(%rsp)
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:768
  311cac:	f6 44 24 4f 01       	testb  $0x1,0x4f(%rsp)
  311cb1:	0f 85 83 00 00 00    	jne    311d3a <<rexx_exec::Interp>::exec_parse+0x247a>
  311cb7:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  311cbe:	00 00 
  311cc0:	49 c7 c6 ff ff ff ff 	mov    $0xffffffffffffffff,%r14
  311cc7:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  311cca:	45 84 e4             	test   %r12b,%r12b
  311ccd:	0f 84 c0 00 00 00    	je     311d93 <<rexx_exec::Interp>::exec_parse+0x24d3>
  311cd3:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:73
  311cd8:	49 8b 84 24 70 04 00 	mov    0x470(%r12),%rax
  311cdf:	00 
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311ce0:	48 85 c0             	test   %rax,%rax
  311ce3:	0f 84 88 13 00 00    	je     313071 <<rexx_exec::Interp>::exec_parse+0x37b1>
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:73
  311ce9:	48 8b b0 f8 00 00 00 	mov    0xf8(%rax),%rsi
  311cf0:	48 8b 90 00 01 00 00 	mov    0x100(%rax),%rdx
<alloc::vec::Vec<(usize, rexx_exec::activation::InstanceVar)>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3126
  311cf7:	48 83 78 50 00       	cmpq   $0x0,0x50(%rax)
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:87
  311cfc:	0f 84 e2 00 00 00    	je     311de4 <<rexx_exec::Interp>::exec_parse+0x2524>
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:40
  311d02:	4c 89 e7             	mov    %r12,%rdi
  311d05:	48 8b 8c 24 e0 01 00 	mov    0x1e0(%rsp),%rcx
  311d0c:	00 
  311d0d:	49 89 e8             	mov    %rbp,%r8
  311d10:	e8 eb e5 f0 ff       	call   220300 <<rexx_exec::Interp>::set_exposed_variable>
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:77
  311d15:	4d 85 ff             	test   %r15,%r15
  311d18:	0f 85 10 01 00 00    	jne    311e2e <<rexx_exec::Interp>::exec_parse+0x256e>
  311d1e:	e9 54 01 00 00       	jmp    311e77 <<rexx_exec::Interp>::exec_parse+0x25b7>
<rexx_exec::Interp>::assign_targets:
  311d23:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  311d2a:	84 00 00 00 00 00 
  311d30:	45 31 e4             	xor    %r12d,%r12d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:768
  311d33:	f6 44 24 4f 01       	testb  $0x1,0x4f(%rsp)
  311d38:	74 86                	je     311cc0 <<rexx_exec::Interp>::exec_parse+0x2400>
<rexx_exec::Interp>::assign_targets::{closure#1}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:769
  311d3a:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  311d3f:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  311d44:	48 8d 94 24 f8 00 00 	lea    0xf8(%rsp),%rdx
  311d4b:	00 
  311d4c:	4c 89 f9             	mov    %r15,%rcx
  311d4f:	4d 89 f0             	mov    %r14,%r8
  311d52:	e8 b9 55 04 00       	call   357310 <<rexx_exec::Interp>::copied_piece>
<bool>::then::<alloc::vec::Vec<u8>, <rexx_exec::Interp>::assign_targets::{closure#1}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/bool.rs:66
  311d57:	4c 8b 74 24 60       	mov    0x60(%rsp),%r14
  311d5c:	4c 8b 7c 24 68       	mov    0x68(%rsp),%r15
  311d61:	48 8b 44 24 70       	mov    0x70(%rsp),%rax
  311d66:	48 89 84 24 18 02 00 	mov    %rax,0x218(%rsp)
  311d6d:	00 
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:770
  311d6e:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  311d73:	4c 89 bc 24 18 01 00 	mov    %r15,0x118(%rsp)
  311d7a:	00 
  311d7b:	74 59                	je     311dd6 <<rexx_exec::Interp>::exec_parse+0x2516>
<core::option::Option<alloc::vec::Vec<u8>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311d7d:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
  311d81:	b8 00 00 00 00       	mov    $0x0,%eax
  311d86:	4c 0f 44 f8          	cmove  %rax,%r15
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  311d8a:	45 84 e4             	test   %r12b,%r12b
  311d8d:	0f 85 40 ff ff ff    	jne    311cd3 <<rexx_exec::Interp>::exec_parse+0x2413>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:778
  311d93:	48 c7 04 24 00 00 00 	movq   $0x0,(%rsp)
  311d9a:	00 
  311d9b:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  311da0:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311da5:	4c 89 e6             	mov    %r12,%rsi
  311da8:	48 8b 94 24 10 01 00 	mov    0x110(%rsp),%rdx
  311daf:	00 
  311db0:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
  311db5:	49 89 e8             	mov    %rbp,%r8
  311db8:	4c 8b 4c 24 50       	mov    0x50(%rsp),%r9
  311dbd:	e8 ce 83 03 00       	call   34a190 <<rexx_exec::Interp>::assign_expr_target>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  311dc2:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
  311dc7:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  311dcb:	0f 84 a6 00 00 00    	je     311e77 <<rexx_exec::Interp>::exec_parse+0x25b7>
  311dd1:	e9 35 0e 00 00       	jmp    312c0b <<rexx_exec::Interp>::exec_parse+0x334b>
<rexx_exec::Interp>::assign_targets:
  311dd6:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  311dd9:	45 84 e4             	test   %r12b,%r12b
  311ddc:	0f 85 f1 fe ff ff    	jne    311cd3 <<rexx_exec::Interp>::exec_parse+0x2413>
  311de2:	eb af                	jmp    311d93 <<rexx_exec::Interp>::exec_parse+0x24d3>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:300
  311de4:	49 3b 94 24 80 0b 00 	cmp    0xb80(%r12),%rdx
  311deb:	00 
  311dec:	0f 85 23 0b 00 00    	jne    312915 <<rexx_exec::Interp>::exec_parse+0x3055>
  311df2:	49 8b 84 24 78 0b 00 	mov    0xb78(%r12),%rax
  311df9:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:302
  311dfa:	48 03 84 24 e0 01 00 	add    0x1e0(%rsp),%rax
  311e01:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  311e02:	49 3b 84 24 08 0b 00 	cmp    0xb08(%r12),%rax
  311e09:	00 
  311e0a:	0f 83 68 16 00 00    	jae    313478 <<rexx_exec::Interp>::exec_parse+0x3bb8>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311e10:	49 8b 8c 24 00 0b 00 	mov    0xb00(%r12),%rcx
  311e17:	00 
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:245
  311e18:	48 c1 e0 04          	shl    $0x4,%rax
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:308
  311e1c:	48 c7 04 01 01 00 00 	movq   $0x1,(%rcx,%rax,1)
  311e23:	00 
  311e24:	48 89 6c 01 08       	mov    %rbp,0x8(%rcx,%rax,1)
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:77
  311e29:	4d 85 ff             	test   %r15,%r15
  311e2c:	74 49                	je     311e77 <<rexx_exec::Interp>::exec_parse+0x25b7>
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  311e2e:	8b bc 24 fc 01 00 00 	mov    0x1fc(%rsp),%edi
  311e35:	48 8b 84 24 48 02 00 	mov    0x248(%rsp),%rax
  311e3c:	00 
<alloc::vec::Vec<alloc::boxed::Box<str>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311e3d:	48 8b 70 10          	mov    0x10(%rax),%rsi
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  311e41:	48 39 fe             	cmp    %rdi,%rsi
  311e44:	0f 86 5e 18 00 00    	jbe    3136a8 <<rexx_exec::Interp>::exec_parse+0x3de8>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::boxed::Box<str>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311e4a:	48 8b 40 08          	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  311e4e:	48 c1 e7 04          	shl    $0x4,%rdi
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  311e52:	48 8b 14 38          	mov    (%rax,%rdi,1),%rdx
  311e56:	48 8b 4c 38 08       	mov    0x8(%rax,%rdi,1),%rcx
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:78
  311e5b:	48 89 2c 24          	mov    %rbp,(%rsp)
  311e5f:	4c 89 e7             	mov    %r12,%rdi
  311e62:	48 8b 74 24 50       	mov    0x50(%rsp),%rsi
  311e67:	4d 89 f8             	mov    %r15,%r8
  311e6a:	4c 8b 8c 24 18 02 00 	mov    0x218(%rsp),%r9
  311e71:	00 
  311e72:	e8 d9 80 03 00       	call   349f50 <<rexx_exec::Interp>::trace_assignment>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:783
  311e77:	f6 44 24 4e 01       	testb  $0x1,0x4e(%rsp)
  311e7c:	74 12                	je     311e90 <<rexx_exec::Interp>::exec_parse+0x25d0>
  311e7e:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:789
  311e82:	75 49                	jne    311ecd <<rexx_exec::Interp>::exec_parse+0x260d>
  311e84:	e9 07 f8 ff ff       	jmp    311690 <<rexx_exec::Interp>::exec_parse+0x1dd0>
  311e89:	0f 1f 80 00 00 00 00 	nopl   0x0(%rax)
  311e90:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
<core::option::Option<alloc::vec::Vec<u8>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311e94:	0f 84 f4 11 00 00    	je     31308e <<rexx_exec::Interp>::exec_parse+0x37ce>
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  311e9a:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  311ea1:	00 00 
  311ea3:	75 28                	jne    311ecd <<rexx_exec::Interp>::exec_parse+0x260d>
  311ea5:	41 f6 84 24 2d 05 00 	testb  $0x1,0x52d(%r12)
  311eac:	00 01 
  311eae:	74 1d                	je     311ecd <<rexx_exec::Interp>::exec_parse+0x260d>
<rexx_exec::Interp>::trace_result:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:225
  311eb0:	4c 89 e7             	mov    %r12,%rdi
  311eb3:	48 8b 74 24 50       	mov    0x50(%rsp),%rsi
  311eb8:	48 8b 94 24 18 01 00 	mov    0x118(%rsp),%rdx
  311ebf:	00 
  311ec0:	48 8b 8c 24 18 02 00 	mov    0x218(%rsp),%rcx
  311ec7:	00 
  311ec8:	e8 73 c1 f6 ff       	call   27e040 <<rexx_exec::Interp>::trace_result_line>
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  311ecd:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  311ed4:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  311eda:	77 14                	ja     311ef0 <<rexx_exec::Interp>::exec_parse+0x2630>
  311edc:	4d 85 f6             	test   %r14,%r14
  311edf:	75 1d                	jne    311efe <<rexx_exec::Interp>::exec_parse+0x263e>
  311ee1:	e9 aa f7 ff ff       	jmp    311690 <<rexx_exec::Interp>::exec_parse+0x1dd0>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
  311ee6:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
  311eed:	00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  311ef0:	4d 8b bc 24 e0 0b 00 	mov    0xbe0(%r12),%r15
  311ef7:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  311ef8:	49 83 ff 08          	cmp    $0x8,%r15
  311efc:	72 13                	jb     311f11 <<rexx_exec::Interp>::exec_parse+0x2651>
<rexx_exec::Interp>::assign_targets:
  311efe:	48 8b bc 24 18 01 00 	mov    0x118(%rsp),%rdi
  311f05:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  311f06:	ff 15 8c 60 1c 00    	call   *0x1c608c(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  311f0c:	e9 7f f7 ff ff       	jmp    311690 <<rexx_exec::Interp>::exec_parse+0x1dd0>
<rexx_exec::Interp>::assign_targets:
  311f11:	48 8b 84 24 38 01 00 	mov    0x138(%rsp),%rax
  311f18:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311f19:	4c 3b 38             	cmp    (%rax),%r15
  311f1c:	0f 85 3c f7 ff ff    	jne    31165e <<rexx_exec::Interp>::exec_parse+0x1d9e>
  311f22:	48 8b bc 24 38 01 00 	mov    0x138(%rsp),%rdi
  311f29:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  311f2a:	e8 01 f0 17 00       	call   490f30 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  311f2f:	e9 2a f7 ff ff       	jmp    31165e <<rexx_exec::Interp>::exec_parse+0x1d9e>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  311f34:	0f b6 42 11          	movzbl 0x11(%rdx),%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  311f38:	48 83 c2 12          	add    $0x12,%rdx
  311f3c:	48 89 d1             	mov    %rdx,%rcx
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311f3f:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311f44:	0f 82 10 fa ff ff    	jb     31195a <<rexx_exec::Interp>::exec_parse+0x209a>
  311f4a:	e9 f7 fa ff ff       	jmp    311a46 <<rexx_exec::Interp>::exec_parse+0x2186>
  311f4f:	49 89 f7             	mov    %rsi,%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311f52:	4c 3b 7c 24 20       	cmp    0x20(%rsp),%r15
  311f57:	0f 82 42 fa ff ff    	jb     31199f <<rexx_exec::Interp>::exec_parse+0x20df>
<rexx_exec::Interp>::assign_targets:
  311f5d:	4c 89 fb             	mov    %r15,%rbx
  311f60:	45 31 ff             	xor    %r15d,%r15d
  311f63:	45 31 c0             	xor    %r8d,%r8d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  311f66:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  311f6b:	0f 84 24 f7 ff ff    	je     311695 <<rexx_exec::Interp>::exec_parse+0x1dd5>
  311f71:	eb 27                	jmp    311f9a <<rexx_exec::Interp>::exec_parse+0x26da>
  311f73:	4c 89 f3             	mov    %r14,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311f76:	4d 89 f4             	mov    %r14,%r12
  311f79:	4d 29 fc             	sub    %r15,%r12
  311f7c:	0f 83 a9 f7 ff ff    	jae    31172b <<rexx_exec::Interp>::exec_parse+0x1e6b>
  311f82:	e9 d4 0f 00 00       	jmp    312f5b <<rexx_exec::Interp>::exec_parse+0x369b>
<rexx_exec::Interp>::assign_targets:
  311f87:	4c 8b 44 24 20       	mov    0x20(%rsp),%r8
  311f8c:	4c 89 c3             	mov    %r8,%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  311f8f:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  311f94:	0f 84 fb f6 ff ff    	je     311695 <<rexx_exec::Interp>::exec_parse+0x1dd5>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:804
  311f9a:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  311f9f:	4c 89 e6             	mov    %r12,%rsi
  311fa2:	48 8d 94 24 f8 00 00 	lea    0xf8(%rsp),%rdx
  311fa9:	00 
  311faa:	4c 89 f9             	mov    %r15,%rcx
  311fad:	e8 5e 53 04 00       	call   357310 <<rexx_exec::Interp>::copied_piece>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311fb2:	4c 8b 7c 24 68       	mov    0x68(%rsp),%r15
  311fb7:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  311fbc:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  311fc3:	00 00 
  311fc5:	75 6c                	jne    312033 <<rexx_exec::Interp>::exec_parse+0x2773>
  311fc7:	41 f6 84 24 2e 05 00 	testb  $0x1,0x52e(%r12)
  311fce:	00 01 
  311fd0:	74 61                	je     312033 <<rexx_exec::Interp>::exec_parse+0x2773>
<rexx_exec::Interp>::trace_dummy:
  311fd2:	4c 8b 74 24 70       	mov    0x70(%rsp),%r14
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  311fd7:	49 8b ac 24 e8 10 00 	mov    0x10e8(%r12),%rbp
  311fde:	00 
  311fdf:	48 8b bc 24 10 02 00 	mov    0x210(%rsp),%rdi
  311fe6:	00 
rexx_exec::trace::format::push_value:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:559
  311fe7:	48 8d 35 89 8b d5 ff 	lea    -0x2a7477(%rip),%rsi        # 6ab77 <panic_unwind::imp::CANARY+0x5e1a>
  311fee:	48 8b 54 24 50       	mov    0x50(%rsp),%rdx
  311ff3:	e8 08 d4 f6 ff       	call   27f400 <rexx_exec::trace::format::push_prefixed_blanks>
  311ff8:	48 8b bc 24 10 02 00 	mov    0x210(%rsp),%rdi
  311fff:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:560
  312000:	4c 89 fe             	mov    %r15,%rsi
  312003:	4c 89 f2             	mov    %r14,%rdx
  312006:	e8 25 d5 f6 ff       	call   27f530 <rexx_exec::trace::format::push_quoted>
<alloc::vec::Vec<u8>>::as_mut_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1904
  31200b:	49 8b 94 24 e8 10 00 	mov    0x10e8(%r12),%rdx
  312012:	00 
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:557
  312013:	48 39 ea             	cmp    %rbp,%rdx
  312016:	0f 82 bf 14 00 00    	jb     3134db <<rexx_exec::Interp>::exec_parse+0x3c1b>
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31201c:	48 89 d6             	mov    %rdx,%rsi
  31201f:	48 29 ee             	sub    %rbp,%rsi
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312022:	0f 85 9f 00 00 00    	jne    3120c7 <<rexx_exec::Interp>::exec_parse+0x2807>
<rexx_exec::Interp>::trace_dummy:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:576
  312028:	4c 89 e7             	mov    %r12,%rdi
  31202b:	48 89 ee             	mov    %rbp,%rsi
  31202e:	e8 1d c1 e8 ff       	call   19e150 <<rexx_exec::Interp>::route_trace_line>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:806
  312033:	4c 8b 74 24 60       	mov    0x60(%rsp),%r14
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312038:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  31203f:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  312045:	77 11                	ja     312058 <<rexx_exec::Interp>::exec_parse+0x2798>
  312047:	4d 85 f6             	test   %r14,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31204a:	0f 84 40 f6 ff ff    	je     311690 <<rexx_exec::Interp>::exec_parse+0x1dd0>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312050:	4c 89 ff             	mov    %r15,%rdi
  312053:	e9 ae fe ff ff       	jmp    311f06 <<rexx_exec::Interp>::exec_parse+0x2646>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312058:	4d 8b a4 24 e0 0b 00 	mov    0xbe0(%r12),%r12
  31205f:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312060:	49 83 fc 08          	cmp    $0x8,%r12
  312064:	72 13                	jb     312079 <<rexx_exec::Interp>::exec_parse+0x27b9>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312066:	4c 89 ff             	mov    %r15,%rdi
  312069:	ff 15 29 5f 1c 00    	call   *0x1c5f29(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  31206f:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312074:	e9 17 f6 ff ff       	jmp    311690 <<rexx_exec::Interp>::exec_parse+0x1dd0>
  312079:	48 8b 84 24 38 01 00 	mov    0x138(%rsp),%rax
  312080:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312081:	4c 3b 20             	cmp    (%rax),%r12
  312084:	75 0d                	jne    312093 <<rexx_exec::Interp>::exec_parse+0x27d3>
  312086:	48 8b bc 24 38 01 00 	mov    0x138(%rsp),%rdi
  31208d:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31208e:	e8 9d ee 17 00       	call   490f30 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  312093:	48 8b 54 24 18       	mov    0x18(%rsp),%rdx
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312098:	48 8b 82 d8 0b 00 00 	mov    0xbd8(%rdx),%rax
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  31209f:	4b 8d 0c 64          	lea    (%r12,%r12,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3120a3:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  3120a7:	4c 89 7c c8 08       	mov    %r15,0x8(%rax,%rcx,8)
  3120ac:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  3120b3:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3120b5:	49 ff c4             	inc    %r12
  3120b8:	4c 89 a2 e0 0b 00 00 	mov    %r12,0xbe0(%rdx)
  3120bf:	49 89 d4             	mov    %rdx,%r12
  3120c2:	e9 c9 f5 ff ff       	jmp    311690 <<rexx_exec::Interp>::exec_parse+0x1dd0>
rexx_exec::trace::format::push_value:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:561
  3120c7:	49 8b 84 24 e0 10 00 	mov    0x10e0(%r12),%rax
  3120ce:	00 
core::slice::index::get_offset_len_mut_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:102
  3120cf:	48 8d 3c 28          	lea    (%rax,%rbp,1),%rdi
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  3120d3:	48 83 fe 10          	cmp    $0x10,%rsi
  3120d7:	73 08                	jae    3120e1 <<rexx_exec::Interp>::exec_parse+0x2821>
  3120d9:	48 89 f9             	mov    %rdi,%rcx
  3120dc:	e9 fb 07 00 00       	jmp    3128dc <<rexx_exec::Interp>::exec_parse+0x301c>
  3120e1:	48 83 fe 20          	cmp    $0x20,%rsi
  3120e5:	0f 83 ab 02 00 00    	jae    312396 <<rexx_exec::Interp>::exec_parse+0x2ad6>
  3120eb:	45 31 c0             	xor    %r8d,%r8d
  3120ee:	49 89 f1             	mov    %rsi,%r9
  3120f1:	49 83 e1 f0          	and    $0xfffffffffffffff0,%r9
  3120f5:	4a 8d 0c 0f          	lea    (%rdi,%r9,1),%rcx
  3120f9:	eb 12                	jmp    31210d <<rexx_exec::Interp>::exec_parse+0x284d>
  3120fb:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
  312100:	49 83 c0 10          	add    $0x10,%r8
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312104:	4d 39 c1             	cmp    %r8,%r9
  312107:	0f 84 7b 02 00 00    	je     312388 <<rexx_exec::Interp>::exec_parse+0x2ac8>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  31210d:	f3 42 0f 6f 0c 07    	movdqu (%rdi,%r8,1),%xmm1
  312113:	66 0f 6f d1          	movdqa %xmm1,%xmm2
  312117:	66 0f da 15 41 e3 d3 	pminub -0x2c1cbf(%rip),%xmm2        # 50460 <GCC_except_table5014+0xa7c>
  31211e:	ff 
  31211f:	66 0f 74 d1          	pcmpeqb %xmm1,%xmm2
  312123:	66 0f 6f c1          	movdqa %xmm1,%xmm0
  312127:	66 0f 74 05 91 df d3 	pcmpeqb -0x2c206f(%rip),%xmm0        # 500c0 <GCC_except_table5014+0x6dc>
  31212e:	ff 
  31212f:	66 0f fc 0d 39 e2 d3 	paddb  -0x2c1dc7(%rip),%xmm1        # 50370 <GCC_except_table5014+0x98c>
  312136:	ff 
  312137:	66 0f 6f d9          	movdqa %xmm1,%xmm3
  31213b:	66 0f da 1d 6d de d3 	pminub -0x2c2193(%rip),%xmm3        # 4ffb0 <GCC_except_table5014+0x5cc>
  312142:	ff 
  312143:	66 0f 74 d9          	pcmpeqb %xmm1,%xmm3
  312147:	66 0f df c3          	pandn  %xmm3,%xmm0
  31214b:	66 0f db c2          	pand   %xmm2,%xmm0
  31214f:	66 0f 7f 84 24 60 02 	movdqa %xmm0,0x260(%rsp)
  312156:	00 00 
  312158:	f6 84 24 60 02 00 00 	testb  $0x1,0x260(%rsp)
  31215f:	01 
  312160:	74 05                	je     312167 <<rexx_exec::Interp>::exec_parse+0x28a7>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  312162:	42 c6 04 07 3f       	movb   $0x3f,(%rdi,%r8,1)
  312167:	66 41 0f 7e c2       	movd   %xmm0,%r10d
  31216c:	45 89 d3             	mov    %r10d,%r11d
  31216f:	41 c1 eb 08          	shr    $0x8,%r11d
  312173:	41 f6 c3 01          	test   $0x1,%r11b
  312177:	0f 84 03 01 00 00    	je     312280 <<rexx_exec::Interp>::exec_parse+0x29c0>
  31217d:	42 c6 44 07 01 3f    	movb   $0x3f,0x1(%rdi,%r8,1)
  312183:	45 89 d3             	mov    %r10d,%r11d
  312186:	41 c1 eb 10          	shr    $0x10,%r11d
  31218a:	41 f6 c3 01          	test   $0x1,%r11b
  31218e:	0f 85 fd 00 00 00    	jne    312291 <<rexx_exec::Interp>::exec_parse+0x29d1>
  312194:	41 c1 ea 18          	shr    $0x18,%r10d
  312198:	41 f6 c2 01          	test   $0x1,%r10b
  31219c:	0f 84 03 01 00 00    	je     3122a5 <<rexx_exec::Interp>::exec_parse+0x29e5>
  3121a2:	42 c6 44 07 03 3f    	movb   $0x3f,0x3(%rdi,%r8,1)
  3121a8:	66 44 0f c5 d0 02    	pextrw $0x2,%xmm0,%r10d
  3121ae:	41 f6 c2 01          	test   $0x1,%r10b
  3121b2:	0f 85 fd 00 00 00    	jne    3122b5 <<rexx_exec::Interp>::exec_parse+0x29f5>
  3121b8:	41 c1 ea 08          	shr    $0x8,%r10d
  3121bc:	41 f6 c2 01          	test   $0x1,%r10b
  3121c0:	0f 84 03 01 00 00    	je     3122c9 <<rexx_exec::Interp>::exec_parse+0x2a09>
  3121c6:	42 c6 44 07 05 3f    	movb   $0x3f,0x5(%rdi,%r8,1)
  3121cc:	66 44 0f c5 d0 03    	pextrw $0x3,%xmm0,%r10d
  3121d2:	41 f6 c2 01          	test   $0x1,%r10b
  3121d6:	0f 85 fd 00 00 00    	jne    3122d9 <<rexx_exec::Interp>::exec_parse+0x2a19>
  3121dc:	41 c1 ea 08          	shr    $0x8,%r10d
  3121e0:	41 f6 c2 01          	test   $0x1,%r10b
  3121e4:	0f 84 03 01 00 00    	je     3122ed <<rexx_exec::Interp>::exec_parse+0x2a2d>
  3121ea:	42 c6 44 07 07 3f    	movb   $0x3f,0x7(%rdi,%r8,1)
  3121f0:	66 44 0f c5 d0 04    	pextrw $0x4,%xmm0,%r10d
  3121f6:	41 f6 c2 01          	test   $0x1,%r10b
  3121fa:	0f 85 fd 00 00 00    	jne    3122fd <<rexx_exec::Interp>::exec_parse+0x2a3d>
  312200:	41 c1 ea 08          	shr    $0x8,%r10d
  312204:	41 f6 c2 01          	test   $0x1,%r10b
  312208:	0f 84 03 01 00 00    	je     312311 <<rexx_exec::Interp>::exec_parse+0x2a51>
  31220e:	42 c6 44 07 09 3f    	movb   $0x3f,0x9(%rdi,%r8,1)
  312214:	66 44 0f c5 d0 05    	pextrw $0x5,%xmm0,%r10d
  31221a:	41 f6 c2 01          	test   $0x1,%r10b
  31221e:	0f 85 fd 00 00 00    	jne    312321 <<rexx_exec::Interp>::exec_parse+0x2a61>
  312224:	41 c1 ea 08          	shr    $0x8,%r10d
  312228:	41 f6 c2 01          	test   $0x1,%r10b
  31222c:	0f 84 03 01 00 00    	je     312335 <<rexx_exec::Interp>::exec_parse+0x2a75>
  312232:	42 c6 44 07 0b 3f    	movb   $0x3f,0xb(%rdi,%r8,1)
  312238:	66 44 0f c5 d0 06    	pextrw $0x6,%xmm0,%r10d
  31223e:	41 f6 c2 01          	test   $0x1,%r10b
  312242:	0f 85 fd 00 00 00    	jne    312345 <<rexx_exec::Interp>::exec_parse+0x2a85>
  312248:	41 c1 ea 08          	shr    $0x8,%r10d
  31224c:	41 f6 c2 01          	test   $0x1,%r10b
  312250:	0f 84 03 01 00 00    	je     312359 <<rexx_exec::Interp>::exec_parse+0x2a99>
  312256:	42 c6 44 07 0d 3f    	movb   $0x3f,0xd(%rdi,%r8,1)
  31225c:	66 44 0f c5 d0 07    	pextrw $0x7,%xmm0,%r10d
  312262:	41 f6 c2 01          	test   $0x1,%r10b
  312266:	0f 85 fd 00 00 00    	jne    312369 <<rexx_exec::Interp>::exec_parse+0x2aa9>
  31226c:	41 c1 ea 08          	shr    $0x8,%r10d
  312270:	41 f6 c2 01          	test   $0x1,%r10b
  312274:	0f 84 86 fe ff ff    	je     312100 <<rexx_exec::Interp>::exec_parse+0x2840>
  31227a:	e9 fe 00 00 00       	jmp    31237d <<rexx_exec::Interp>::exec_parse+0x2abd>
  31227f:	90                   	nop
  312280:	45 89 d3             	mov    %r10d,%r11d
  312283:	41 c1 eb 10          	shr    $0x10,%r11d
  312287:	41 f6 c3 01          	test   $0x1,%r11b
  31228b:	0f 84 03 ff ff ff    	je     312194 <<rexx_exec::Interp>::exec_parse+0x28d4>
  312291:	42 c6 44 07 02 3f    	movb   $0x3f,0x2(%rdi,%r8,1)
  312297:	41 c1 ea 18          	shr    $0x18,%r10d
  31229b:	41 f6 c2 01          	test   $0x1,%r10b
  31229f:	0f 85 fd fe ff ff    	jne    3121a2 <<rexx_exec::Interp>::exec_parse+0x28e2>
  3122a5:	66 44 0f c5 d0 02    	pextrw $0x2,%xmm0,%r10d
  3122ab:	41 f6 c2 01          	test   $0x1,%r10b
  3122af:	0f 84 03 ff ff ff    	je     3121b8 <<rexx_exec::Interp>::exec_parse+0x28f8>
  3122b5:	42 c6 44 07 04 3f    	movb   $0x3f,0x4(%rdi,%r8,1)
  3122bb:	41 c1 ea 08          	shr    $0x8,%r10d
  3122bf:	41 f6 c2 01          	test   $0x1,%r10b
  3122c3:	0f 85 fd fe ff ff    	jne    3121c6 <<rexx_exec::Interp>::exec_parse+0x2906>
  3122c9:	66 44 0f c5 d0 03    	pextrw $0x3,%xmm0,%r10d
  3122cf:	41 f6 c2 01          	test   $0x1,%r10b
  3122d3:	0f 84 03 ff ff ff    	je     3121dc <<rexx_exec::Interp>::exec_parse+0x291c>
  3122d9:	42 c6 44 07 06 3f    	movb   $0x3f,0x6(%rdi,%r8,1)
  3122df:	41 c1 ea 08          	shr    $0x8,%r10d
  3122e3:	41 f6 c2 01          	test   $0x1,%r10b
  3122e7:	0f 85 fd fe ff ff    	jne    3121ea <<rexx_exec::Interp>::exec_parse+0x292a>
  3122ed:	66 44 0f c5 d0 04    	pextrw $0x4,%xmm0,%r10d
  3122f3:	41 f6 c2 01          	test   $0x1,%r10b
  3122f7:	0f 84 03 ff ff ff    	je     312200 <<rexx_exec::Interp>::exec_parse+0x2940>
  3122fd:	42 c6 44 07 08 3f    	movb   $0x3f,0x8(%rdi,%r8,1)
  312303:	41 c1 ea 08          	shr    $0x8,%r10d
  312307:	41 f6 c2 01          	test   $0x1,%r10b
  31230b:	0f 85 fd fe ff ff    	jne    31220e <<rexx_exec::Interp>::exec_parse+0x294e>
  312311:	66 44 0f c5 d0 05    	pextrw $0x5,%xmm0,%r10d
  312317:	41 f6 c2 01          	test   $0x1,%r10b
  31231b:	0f 84 03 ff ff ff    	je     312224 <<rexx_exec::Interp>::exec_parse+0x2964>
  312321:	42 c6 44 07 0a 3f    	movb   $0x3f,0xa(%rdi,%r8,1)
  312327:	41 c1 ea 08          	shr    $0x8,%r10d
  31232b:	41 f6 c2 01          	test   $0x1,%r10b
  31232f:	0f 85 fd fe ff ff    	jne    312232 <<rexx_exec::Interp>::exec_parse+0x2972>
  312335:	66 44 0f c5 d0 06    	pextrw $0x6,%xmm0,%r10d
  31233b:	41 f6 c2 01          	test   $0x1,%r10b
  31233f:	0f 84 03 ff ff ff    	je     312248 <<rexx_exec::Interp>::exec_parse+0x2988>
  312345:	42 c6 44 07 0c 3f    	movb   $0x3f,0xc(%rdi,%r8,1)
  31234b:	41 c1 ea 08          	shr    $0x8,%r10d
  31234f:	41 f6 c2 01          	test   $0x1,%r10b
  312353:	0f 85 fd fe ff ff    	jne    312256 <<rexx_exec::Interp>::exec_parse+0x2996>
  312359:	66 44 0f c5 d0 07    	pextrw $0x7,%xmm0,%r10d
  31235f:	41 f6 c2 01          	test   $0x1,%r10b
  312363:	0f 84 03 ff ff ff    	je     31226c <<rexx_exec::Interp>::exec_parse+0x29ac>
  312369:	42 c6 44 07 0e 3f    	movb   $0x3f,0xe(%rdi,%r8,1)
  31236f:	41 c1 ea 08          	shr    $0x8,%r10d
  312373:	41 f6 c2 01          	test   $0x1,%r10b
  312377:	0f 84 83 fd ff ff    	je     312100 <<rexx_exec::Interp>::exec_parse+0x2840>
  31237d:	42 c6 44 07 0f 3f    	movb   $0x3f,0xf(%rdi,%r8,1)
  312383:	e9 78 fd ff ff       	jmp    312100 <<rexx_exec::Interp>::exec_parse+0x2840>
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312388:	4c 39 ce             	cmp    %r9,%rsi
  31238b:	0f 84 97 fc ff ff    	je     312028 <<rexx_exec::Interp>::exec_parse+0x2768>
  312391:	e9 46 05 00 00       	jmp    3128dc <<rexx_exec::Interp>::exec_parse+0x301c>
  312396:	49 89 f0             	mov    %rsi,%r8
  312399:	49 83 e0 e0          	and    $0xffffffffffffffe0,%r8
  31239d:	4a 8d 0c 07          	lea    (%rdi,%r8,1),%rcx
  3123a1:	45 31 c9             	xor    %r9d,%r9d
  3123a4:	eb 17                	jmp    3123bd <<rexx_exec::Interp>::exec_parse+0x2afd>
  3123a6:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
  3123ad:	00 00 00 
  3123b0:	49 83 c1 20          	add    $0x20,%r9
  3123b4:	4d 39 c8             	cmp    %r9,%r8
  3123b7:	0f 84 0c 05 00 00    	je     3128c9 <<rexx_exec::Interp>::exec_parse+0x3009>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  3123bd:	f3 42 0f 6f 14 0f    	movdqu (%rdi,%r9,1),%xmm2
  3123c3:	f3 42 0f 6f 44 0f 10 	movdqu 0x10(%rdi,%r9,1),%xmm0
  3123ca:	66 0f 6f da          	movdqa %xmm2,%xmm3
  3123ce:	66 0f da 1d 8a e0 d3 	pminub -0x2c1f76(%rip),%xmm3        # 50460 <GCC_except_table5014+0xa7c>
  3123d5:	ff 
  3123d6:	66 0f 74 da          	pcmpeqb %xmm2,%xmm3
  3123da:	66 0f 6f ca          	movdqa %xmm2,%xmm1
  3123de:	66 0f 74 0d da dc d3 	pcmpeqb -0x2c2326(%rip),%xmm1        # 500c0 <GCC_except_table5014+0x6dc>
  3123e5:	ff 
  3123e6:	66 0f fc 15 82 df d3 	paddb  -0x2c207e(%rip),%xmm2        # 50370 <GCC_except_table5014+0x98c>
  3123ed:	ff 
  3123ee:	66 0f 6f e2          	movdqa %xmm2,%xmm4
  3123f2:	66 0f da 25 b6 db d3 	pminub -0x2c244a(%rip),%xmm4        # 4ffb0 <GCC_except_table5014+0x5cc>
  3123f9:	ff 
  3123fa:	66 0f 74 e2          	pcmpeqb %xmm2,%xmm4
  3123fe:	66 0f df cc          	pandn  %xmm4,%xmm1
  312402:	66 0f db cb          	pand   %xmm3,%xmm1
  312406:	66 0f 7f 8c 24 70 02 	movdqa %xmm1,0x270(%rsp)
  31240d:	00 00 
  31240f:	f6 84 24 70 02 00 00 	testb  $0x1,0x270(%rsp)
  312416:	01 
  312417:	74 05                	je     31241e <<rexx_exec::Interp>::exec_parse+0x2b5e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  312419:	42 c6 04 0f 3f       	movb   $0x3f,(%rdi,%r9,1)
  31241e:	66 41 0f 7e ca       	movd   %xmm1,%r10d
  312423:	45 89 d3             	mov    %r10d,%r11d
  312426:	41 c1 eb 08          	shr    $0x8,%r11d
  31242a:	41 f6 c3 01          	test   $0x1,%r11b
  31242e:	0f 84 6c 02 00 00    	je     3126a0 <<rexx_exec::Interp>::exec_parse+0x2de0>
  312434:	42 c6 44 0f 01 3f    	movb   $0x3f,0x1(%rdi,%r9,1)
  31243a:	45 89 d3             	mov    %r10d,%r11d
  31243d:	41 c1 eb 10          	shr    $0x10,%r11d
  312441:	41 f6 c3 01          	test   $0x1,%r11b
  312445:	0f 85 66 02 00 00    	jne    3126b1 <<rexx_exec::Interp>::exec_parse+0x2df1>
  31244b:	41 c1 ea 18          	shr    $0x18,%r10d
  31244f:	41 f6 c2 01          	test   $0x1,%r10b
  312453:	0f 84 6c 02 00 00    	je     3126c5 <<rexx_exec::Interp>::exec_parse+0x2e05>
  312459:	42 c6 44 0f 03 3f    	movb   $0x3f,0x3(%rdi,%r9,1)
  31245f:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  312465:	41 f6 c2 01          	test   $0x1,%r10b
  312469:	0f 85 66 02 00 00    	jne    3126d5 <<rexx_exec::Interp>::exec_parse+0x2e15>
  31246f:	41 c1 ea 08          	shr    $0x8,%r10d
  312473:	41 f6 c2 01          	test   $0x1,%r10b
  312477:	0f 84 6c 02 00 00    	je     3126e9 <<rexx_exec::Interp>::exec_parse+0x2e29>
  31247d:	42 c6 44 0f 05 3f    	movb   $0x3f,0x5(%rdi,%r9,1)
  312483:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  312489:	41 f6 c2 01          	test   $0x1,%r10b
  31248d:	0f 85 66 02 00 00    	jne    3126f9 <<rexx_exec::Interp>::exec_parse+0x2e39>
  312493:	41 c1 ea 08          	shr    $0x8,%r10d
  312497:	41 f6 c2 01          	test   $0x1,%r10b
  31249b:	0f 84 6c 02 00 00    	je     31270d <<rexx_exec::Interp>::exec_parse+0x2e4d>
  3124a1:	42 c6 44 0f 07 3f    	movb   $0x3f,0x7(%rdi,%r9,1)
  3124a7:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  3124ad:	41 f6 c2 01          	test   $0x1,%r10b
  3124b1:	0f 85 66 02 00 00    	jne    31271d <<rexx_exec::Interp>::exec_parse+0x2e5d>
  3124b7:	41 c1 ea 08          	shr    $0x8,%r10d
  3124bb:	41 f6 c2 01          	test   $0x1,%r10b
  3124bf:	0f 84 6c 02 00 00    	je     312731 <<rexx_exec::Interp>::exec_parse+0x2e71>
  3124c5:	42 c6 44 0f 09 3f    	movb   $0x3f,0x9(%rdi,%r9,1)
  3124cb:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  3124d1:	41 f6 c2 01          	test   $0x1,%r10b
  3124d5:	0f 85 66 02 00 00    	jne    312741 <<rexx_exec::Interp>::exec_parse+0x2e81>
  3124db:	41 c1 ea 08          	shr    $0x8,%r10d
  3124df:	41 f6 c2 01          	test   $0x1,%r10b
  3124e3:	0f 84 6c 02 00 00    	je     312755 <<rexx_exec::Interp>::exec_parse+0x2e95>
  3124e9:	42 c6 44 0f 0b 3f    	movb   $0x3f,0xb(%rdi,%r9,1)
  3124ef:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  3124f5:	41 f6 c2 01          	test   $0x1,%r10b
  3124f9:	0f 85 66 02 00 00    	jne    312765 <<rexx_exec::Interp>::exec_parse+0x2ea5>
  3124ff:	41 c1 ea 08          	shr    $0x8,%r10d
  312503:	41 f6 c2 01          	test   $0x1,%r10b
  312507:	0f 84 6c 02 00 00    	je     312779 <<rexx_exec::Interp>::exec_parse+0x2eb9>
  31250d:	42 c6 44 0f 0d 3f    	movb   $0x3f,0xd(%rdi,%r9,1)
  312513:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  312519:	41 f6 c2 01          	test   $0x1,%r10b
  31251d:	0f 85 66 02 00 00    	jne    312789 <<rexx_exec::Interp>::exec_parse+0x2ec9>
  312523:	41 c1 ea 08          	shr    $0x8,%r10d
  312527:	41 f6 c2 01          	test   $0x1,%r10b
  31252b:	74 06                	je     312533 <<rexx_exec::Interp>::exec_parse+0x2c73>
  31252d:	42 c6 44 0f 0f 3f    	movb   $0x3f,0xf(%rdi,%r9,1)
  312533:	66 0f 6f d0          	movdqa %xmm0,%xmm2
  312537:	66 0f da 15 21 df d3 	pminub -0x2c20df(%rip),%xmm2        # 50460 <GCC_except_table5014+0xa7c>
  31253e:	ff 
  31253f:	66 0f 74 d0          	pcmpeqb %xmm0,%xmm2
  312543:	66 0f 6f c8          	movdqa %xmm0,%xmm1
  312547:	66 0f 74 0d 71 db d3 	pcmpeqb -0x2c248f(%rip),%xmm1        # 500c0 <GCC_except_table5014+0x6dc>
  31254e:	ff 
  31254f:	66 0f fc 05 19 de d3 	paddb  -0x2c21e7(%rip),%xmm0        # 50370 <GCC_except_table5014+0x98c>
  312556:	ff 
  312557:	66 0f 6f d8          	movdqa %xmm0,%xmm3
  31255b:	66 0f da 1d 4d da d3 	pminub -0x2c25b3(%rip),%xmm3        # 4ffb0 <GCC_except_table5014+0x5cc>
  312562:	ff 
  312563:	66 0f 74 d8          	pcmpeqb %xmm0,%xmm3
  312567:	66 0f df cb          	pandn  %xmm3,%xmm1
  31256b:	66 0f db ca          	pand   %xmm2,%xmm1
  31256f:	66 41 0f 7e ca       	movd   %xmm1,%r10d
  312574:	41 f6 c2 01          	test   $0x1,%r10b
  312578:	0f 84 32 02 00 00    	je     3127b0 <<rexx_exec::Interp>::exec_parse+0x2ef0>
  31257e:	42 c6 44 0f 10 3f    	movb   $0x3f,0x10(%rdi,%r9,1)
  312584:	45 89 d3             	mov    %r10d,%r11d
  312587:	41 c1 eb 08          	shr    $0x8,%r11d
  31258b:	41 f6 c3 01          	test   $0x1,%r11b
  31258f:	0f 85 2c 02 00 00    	jne    3127c1 <<rexx_exec::Interp>::exec_parse+0x2f01>
  312595:	45 89 d3             	mov    %r10d,%r11d
  312598:	41 c1 eb 10          	shr    $0x10,%r11d
  31259c:	41 f6 c3 01          	test   $0x1,%r11b
  3125a0:	0f 84 32 02 00 00    	je     3127d8 <<rexx_exec::Interp>::exec_parse+0x2f18>
  3125a6:	42 c6 44 0f 12 3f    	movb   $0x3f,0x12(%rdi,%r9,1)
  3125ac:	41 c1 ea 18          	shr    $0x18,%r10d
  3125b0:	41 f6 c2 01          	test   $0x1,%r10b
  3125b4:	0f 85 2c 02 00 00    	jne    3127e6 <<rexx_exec::Interp>::exec_parse+0x2f26>
  3125ba:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  3125c0:	41 f6 c2 01          	test   $0x1,%r10b
  3125c4:	0f 84 32 02 00 00    	je     3127fc <<rexx_exec::Interp>::exec_parse+0x2f3c>
  3125ca:	42 c6 44 0f 14 3f    	movb   $0x3f,0x14(%rdi,%r9,1)
  3125d0:	41 c1 ea 08          	shr    $0x8,%r10d
  3125d4:	41 f6 c2 01          	test   $0x1,%r10b
  3125d8:	0f 85 2c 02 00 00    	jne    31280a <<rexx_exec::Interp>::exec_parse+0x2f4a>
  3125de:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  3125e4:	41 f6 c2 01          	test   $0x1,%r10b
  3125e8:	0f 84 32 02 00 00    	je     312820 <<rexx_exec::Interp>::exec_parse+0x2f60>
  3125ee:	42 c6 44 0f 16 3f    	movb   $0x3f,0x16(%rdi,%r9,1)
  3125f4:	41 c1 ea 08          	shr    $0x8,%r10d
  3125f8:	41 f6 c2 01          	test   $0x1,%r10b
  3125fc:	0f 85 2c 02 00 00    	jne    31282e <<rexx_exec::Interp>::exec_parse+0x2f6e>
  312602:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  312608:	41 f6 c2 01          	test   $0x1,%r10b
  31260c:	0f 84 32 02 00 00    	je     312844 <<rexx_exec::Interp>::exec_parse+0x2f84>
  312612:	42 c6 44 0f 18 3f    	movb   $0x3f,0x18(%rdi,%r9,1)
  312618:	41 c1 ea 08          	shr    $0x8,%r10d
  31261c:	41 f6 c2 01          	test   $0x1,%r10b
  312620:	0f 85 2c 02 00 00    	jne    312852 <<rexx_exec::Interp>::exec_parse+0x2f92>
  312626:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  31262c:	41 f6 c2 01          	test   $0x1,%r10b
  312630:	0f 84 32 02 00 00    	je     312868 <<rexx_exec::Interp>::exec_parse+0x2fa8>
  312636:	42 c6 44 0f 1a 3f    	movb   $0x3f,0x1a(%rdi,%r9,1)
  31263c:	41 c1 ea 08          	shr    $0x8,%r10d
  312640:	41 f6 c2 01          	test   $0x1,%r10b
  312644:	0f 85 2c 02 00 00    	jne    312876 <<rexx_exec::Interp>::exec_parse+0x2fb6>
  31264a:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  312650:	41 f6 c2 01          	test   $0x1,%r10b
  312654:	0f 84 32 02 00 00    	je     31288c <<rexx_exec::Interp>::exec_parse+0x2fcc>
  31265a:	42 c6 44 0f 1c 3f    	movb   $0x3f,0x1c(%rdi,%r9,1)
  312660:	41 c1 ea 08          	shr    $0x8,%r10d
  312664:	41 f6 c2 01          	test   $0x1,%r10b
  312668:	0f 85 2c 02 00 00    	jne    31289a <<rexx_exec::Interp>::exec_parse+0x2fda>
  31266e:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  312674:	41 f6 c2 01          	test   $0x1,%r10b
  312678:	0f 84 32 02 00 00    	je     3128b0 <<rexx_exec::Interp>::exec_parse+0x2ff0>
  31267e:	42 c6 44 0f 1e 3f    	movb   $0x3f,0x1e(%rdi,%r9,1)
  312684:	41 c1 ea 08          	shr    $0x8,%r10d
  312688:	41 f6 c2 01          	test   $0x1,%r10b
  31268c:	0f 84 1e fd ff ff    	je     3123b0 <<rexx_exec::Interp>::exec_parse+0x2af0>
  312692:	e9 27 02 00 00       	jmp    3128be <<rexx_exec::Interp>::exec_parse+0x2ffe>
  312697:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  31269e:	00 00 
  3126a0:	45 89 d3             	mov    %r10d,%r11d
  3126a3:	41 c1 eb 10          	shr    $0x10,%r11d
  3126a7:	41 f6 c3 01          	test   $0x1,%r11b
  3126ab:	0f 84 9a fd ff ff    	je     31244b <<rexx_exec::Interp>::exec_parse+0x2b8b>
  3126b1:	42 c6 44 0f 02 3f    	movb   $0x3f,0x2(%rdi,%r9,1)
  3126b7:	41 c1 ea 18          	shr    $0x18,%r10d
  3126bb:	41 f6 c2 01          	test   $0x1,%r10b
  3126bf:	0f 85 94 fd ff ff    	jne    312459 <<rexx_exec::Interp>::exec_parse+0x2b99>
  3126c5:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  3126cb:	41 f6 c2 01          	test   $0x1,%r10b
  3126cf:	0f 84 9a fd ff ff    	je     31246f <<rexx_exec::Interp>::exec_parse+0x2baf>
  3126d5:	42 c6 44 0f 04 3f    	movb   $0x3f,0x4(%rdi,%r9,1)
  3126db:	41 c1 ea 08          	shr    $0x8,%r10d
  3126df:	41 f6 c2 01          	test   $0x1,%r10b
  3126e3:	0f 85 94 fd ff ff    	jne    31247d <<rexx_exec::Interp>::exec_parse+0x2bbd>
  3126e9:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  3126ef:	41 f6 c2 01          	test   $0x1,%r10b
  3126f3:	0f 84 9a fd ff ff    	je     312493 <<rexx_exec::Interp>::exec_parse+0x2bd3>
  3126f9:	42 c6 44 0f 06 3f    	movb   $0x3f,0x6(%rdi,%r9,1)
  3126ff:	41 c1 ea 08          	shr    $0x8,%r10d
  312703:	41 f6 c2 01          	test   $0x1,%r10b
  312707:	0f 85 94 fd ff ff    	jne    3124a1 <<rexx_exec::Interp>::exec_parse+0x2be1>
  31270d:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  312713:	41 f6 c2 01          	test   $0x1,%r10b
  312717:	0f 84 9a fd ff ff    	je     3124b7 <<rexx_exec::Interp>::exec_parse+0x2bf7>
  31271d:	42 c6 44 0f 08 3f    	movb   $0x3f,0x8(%rdi,%r9,1)
  312723:	41 c1 ea 08          	shr    $0x8,%r10d
  312727:	41 f6 c2 01          	test   $0x1,%r10b
  31272b:	0f 85 94 fd ff ff    	jne    3124c5 <<rexx_exec::Interp>::exec_parse+0x2c05>
  312731:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  312737:	41 f6 c2 01          	test   $0x1,%r10b
  31273b:	0f 84 9a fd ff ff    	je     3124db <<rexx_exec::Interp>::exec_parse+0x2c1b>
  312741:	42 c6 44 0f 0a 3f    	movb   $0x3f,0xa(%rdi,%r9,1)
  312747:	41 c1 ea 08          	shr    $0x8,%r10d
  31274b:	41 f6 c2 01          	test   $0x1,%r10b
  31274f:	0f 85 94 fd ff ff    	jne    3124e9 <<rexx_exec::Interp>::exec_parse+0x2c29>
  312755:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  31275b:	41 f6 c2 01          	test   $0x1,%r10b
  31275f:	0f 84 9a fd ff ff    	je     3124ff <<rexx_exec::Interp>::exec_parse+0x2c3f>
  312765:	42 c6 44 0f 0c 3f    	movb   $0x3f,0xc(%rdi,%r9,1)
  31276b:	41 c1 ea 08          	shr    $0x8,%r10d
  31276f:	41 f6 c2 01          	test   $0x1,%r10b
  312773:	0f 85 94 fd ff ff    	jne    31250d <<rexx_exec::Interp>::exec_parse+0x2c4d>
  312779:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  31277f:	41 f6 c2 01          	test   $0x1,%r10b
  312783:	0f 84 9a fd ff ff    	je     312523 <<rexx_exec::Interp>::exec_parse+0x2c63>
  312789:	42 c6 44 0f 0e 3f    	movb   $0x3f,0xe(%rdi,%r9,1)
  31278f:	41 c1 ea 08          	shr    $0x8,%r10d
  312793:	41 f6 c2 01          	test   $0x1,%r10b
  312797:	0f 85 90 fd ff ff    	jne    31252d <<rexx_exec::Interp>::exec_parse+0x2c6d>
  31279d:	e9 91 fd ff ff       	jmp    312533 <<rexx_exec::Interp>::exec_parse+0x2c73>
  3127a2:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  3127a9:	1f 84 00 00 00 00 00 
  3127b0:	45 89 d3             	mov    %r10d,%r11d
  3127b3:	41 c1 eb 08          	shr    $0x8,%r11d
  3127b7:	41 f6 c3 01          	test   $0x1,%r11b
  3127bb:	0f 84 d4 fd ff ff    	je     312595 <<rexx_exec::Interp>::exec_parse+0x2cd5>
  3127c1:	42 c6 44 0f 11 3f    	movb   $0x3f,0x11(%rdi,%r9,1)
  3127c7:	45 89 d3             	mov    %r10d,%r11d
  3127ca:	41 c1 eb 10          	shr    $0x10,%r11d
  3127ce:	41 f6 c3 01          	test   $0x1,%r11b
  3127d2:	0f 85 ce fd ff ff    	jne    3125a6 <<rexx_exec::Interp>::exec_parse+0x2ce6>
  3127d8:	41 c1 ea 18          	shr    $0x18,%r10d
  3127dc:	41 f6 c2 01          	test   $0x1,%r10b
  3127e0:	0f 84 d4 fd ff ff    	je     3125ba <<rexx_exec::Interp>::exec_parse+0x2cfa>
  3127e6:	42 c6 44 0f 13 3f    	movb   $0x3f,0x13(%rdi,%r9,1)
  3127ec:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  3127f2:	41 f6 c2 01          	test   $0x1,%r10b
  3127f6:	0f 85 ce fd ff ff    	jne    3125ca <<rexx_exec::Interp>::exec_parse+0x2d0a>
  3127fc:	41 c1 ea 08          	shr    $0x8,%r10d
  312800:	41 f6 c2 01          	test   $0x1,%r10b
  312804:	0f 84 d4 fd ff ff    	je     3125de <<rexx_exec::Interp>::exec_parse+0x2d1e>
  31280a:	42 c6 44 0f 15 3f    	movb   $0x3f,0x15(%rdi,%r9,1)
  312810:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  312816:	41 f6 c2 01          	test   $0x1,%r10b
  31281a:	0f 85 ce fd ff ff    	jne    3125ee <<rexx_exec::Interp>::exec_parse+0x2d2e>
  312820:	41 c1 ea 08          	shr    $0x8,%r10d
  312824:	41 f6 c2 01          	test   $0x1,%r10b
  312828:	0f 84 d4 fd ff ff    	je     312602 <<rexx_exec::Interp>::exec_parse+0x2d42>
  31282e:	42 c6 44 0f 17 3f    	movb   $0x3f,0x17(%rdi,%r9,1)
  312834:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  31283a:	41 f6 c2 01          	test   $0x1,%r10b
  31283e:	0f 85 ce fd ff ff    	jne    312612 <<rexx_exec::Interp>::exec_parse+0x2d52>
  312844:	41 c1 ea 08          	shr    $0x8,%r10d
  312848:	41 f6 c2 01          	test   $0x1,%r10b
  31284c:	0f 84 d4 fd ff ff    	je     312626 <<rexx_exec::Interp>::exec_parse+0x2d66>
  312852:	42 c6 44 0f 19 3f    	movb   $0x3f,0x19(%rdi,%r9,1)
  312858:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  31285e:	41 f6 c2 01          	test   $0x1,%r10b
  312862:	0f 85 ce fd ff ff    	jne    312636 <<rexx_exec::Interp>::exec_parse+0x2d76>
  312868:	41 c1 ea 08          	shr    $0x8,%r10d
  31286c:	41 f6 c2 01          	test   $0x1,%r10b
  312870:	0f 84 d4 fd ff ff    	je     31264a <<rexx_exec::Interp>::exec_parse+0x2d8a>
  312876:	42 c6 44 0f 1b 3f    	movb   $0x3f,0x1b(%rdi,%r9,1)
  31287c:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  312882:	41 f6 c2 01          	test   $0x1,%r10b
  312886:	0f 85 ce fd ff ff    	jne    31265a <<rexx_exec::Interp>::exec_parse+0x2d9a>
  31288c:	41 c1 ea 08          	shr    $0x8,%r10d
  312890:	41 f6 c2 01          	test   $0x1,%r10b
  312894:	0f 84 d4 fd ff ff    	je     31266e <<rexx_exec::Interp>::exec_parse+0x2dae>
  31289a:	42 c6 44 0f 1d 3f    	movb   $0x3f,0x1d(%rdi,%r9,1)
  3128a0:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  3128a6:	41 f6 c2 01          	test   $0x1,%r10b
  3128aa:	0f 85 ce fd ff ff    	jne    31267e <<rexx_exec::Interp>::exec_parse+0x2dbe>
  3128b0:	41 c1 ea 08          	shr    $0x8,%r10d
  3128b4:	41 f6 c2 01          	test   $0x1,%r10b
  3128b8:	0f 84 f2 fa ff ff    	je     3123b0 <<rexx_exec::Interp>::exec_parse+0x2af0>
  3128be:	42 c6 44 0f 1f 3f    	movb   $0x3f,0x1f(%rdi,%r9,1)
  3128c4:	e9 e7 fa ff ff       	jmp    3123b0 <<rexx_exec::Interp>::exec_parse+0x2af0>
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3128c9:	4c 39 c6             	cmp    %r8,%rsi
  3128cc:	0f 84 56 f7 ff ff    	je     312028 <<rexx_exec::Interp>::exec_parse+0x2768>
rexx_exec::error::displayable:
  3128d2:	40 f6 c6 10          	test   $0x10,%sil
  3128d6:	0f 85 12 f8 ff ff    	jne    3120ee <<rexx_exec::Interp>::exec_parse+0x282e>
  3128dc:	48 01 d0             	add    %rdx,%rax
  3128df:	eb 1b                	jmp    3128fc <<rexx_exec::Interp>::exec_parse+0x303c>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  3128e1:	c6 01 3f             	movb   $0x3f,(%rcx)
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
  3128e4:	66 66 66 2e 0f 1f 84 	data16 data16 cs nopw 0x0(%rax,%rax,1)
  3128eb:	00 00 00 00 00 
  3128f0:	48 ff c1             	inc    %rcx
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3128f3:	48 39 c1             	cmp    %rax,%rcx
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3128f6:	0f 84 2c f7 ff ff    	je     312028 <<rexx_exec::Interp>::exec_parse+0x2768>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  3128fc:	0f b6 11             	movzbl (%rcx),%edx
  3128ff:	83 fa 1f             	cmp    $0x1f,%edx
  312902:	77 ec                	ja     3128f0 <<rexx_exec::Interp>::exec_parse+0x3030>
  312904:	83 fa 0d             	cmp    $0xd,%edx
  312907:	77 d8                	ja     3128e1 <<rexx_exec::Interp>::exec_parse+0x3021>
<rexx_exec::Interp>::assign_targets:
  312909:	be 00 26 00 00       	mov    $0x2600,%esi
  31290e:	0f a3 d6             	bt     %edx,%esi
  312911:	72 dd                	jb     3128f0 <<rexx_exec::Interp>::exec_parse+0x3030>
  312913:	eb cc                	jmp    3128e1 <<rexx_exec::Interp>::exec_parse+0x3021>
  312915:	48 8b bc 24 38 02 00 	mov    0x238(%rsp),%rdi
  31291c:	00 
  31291d:	48 8b 8c 24 e0 01 00 	mov    0x1e0(%rsp),%rcx
  312924:	00 
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:311
  312925:	49 89 e8             	mov    %rbp,%r8
  312928:	e8 93 de e4 ff       	call   1607c0 <<rexx_core::roots::RootSet>::set_frame_slot_of>
  31292d:	e9 e3 f3 ff ff       	jmp    311d15 <<rexx_exec::Interp>::exec_parse+0x2455>
<rexx_exec::Interp>::exec_parse:
  312932:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  312937:	49 89 f0             	mov    %rsi,%r8
  31293a:	48 89 f7             	mov    %rsi,%rdi
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31293d:	83 fa 01             	cmp    $0x1,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312940:	0f 84 50 ec ff ff    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312946:	bd 01 00 00 00       	mov    $0x1,%ebp
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  31294b:	41 38 4c 06 01       	cmp    %cl,0x1(%r14,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312950:	0f 84 bc eb ff ff    	je     311512 <<rexx_exec::Interp>::exec_parse+0x1c52>
  312956:	49 89 f0             	mov    %rsi,%r8
  312959:	48 89 f7             	mov    %rsi,%rdi
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31295c:	83 fa 02             	cmp    $0x2,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31295f:	0f 84 31 ec ff ff    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312965:	bd 02 00 00 00       	mov    $0x2,%ebp
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  31296a:	41 38 4c 06 02       	cmp    %cl,0x2(%r14,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  31296f:	0f 84 9d eb ff ff    	je     311512 <<rexx_exec::Interp>::exec_parse+0x1c52>
  312975:	49 89 f0             	mov    %rsi,%r8
  312978:	48 89 f7             	mov    %rsi,%rdi
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31297b:	83 fa 03             	cmp    $0x3,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31297e:	0f 84 12 ec ff ff    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312984:	bd 03 00 00 00       	mov    $0x3,%ebp
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312989:	41 38 4c 06 03       	cmp    %cl,0x3(%r14,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  31298e:	0f 84 7e eb ff ff    	je     311512 <<rexx_exec::Interp>::exec_parse+0x1c52>
  312994:	49 89 f0             	mov    %rsi,%r8
  312997:	48 89 f7             	mov    %rsi,%rdi
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31299a:	83 fa 04             	cmp    $0x4,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31299d:	0f 84 f3 eb ff ff    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  3129a3:	bd 04 00 00 00       	mov    $0x4,%ebp
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  3129a8:	41 38 4c 06 04       	cmp    %cl,0x4(%r14,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3129ad:	0f 84 5f eb ff ff    	je     311512 <<rexx_exec::Interp>::exec_parse+0x1c52>
  3129b3:	49 89 f0             	mov    %rsi,%r8
  3129b6:	48 89 f7             	mov    %rsi,%rdi
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3129b9:	83 fa 05             	cmp    $0x5,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3129bc:	0f 84 d4 eb ff ff    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  3129c2:	bd 05 00 00 00       	mov    $0x5,%ebp
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  3129c7:	41 38 4c 06 05       	cmp    %cl,0x5(%r14,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3129cc:	0f 84 40 eb ff ff    	je     311512 <<rexx_exec::Interp>::exec_parse+0x1c52>
  3129d2:	49 89 f0             	mov    %rsi,%r8
  3129d5:	48 89 f7             	mov    %rsi,%rdi
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3129d8:	83 fa 06             	cmp    $0x6,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3129db:	0f 84 b5 eb ff ff    	je     311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  3129e1:	bd 06 00 00 00       	mov    $0x6,%ebp
  3129e6:	49 89 f0             	mov    %rsi,%r8
  3129e9:	48 89 f7             	mov    %rsi,%rdi
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  3129ec:	41 38 4c 06 06       	cmp    %cl,0x6(%r14,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3129f1:	0f 84 1b eb ff ff    	je     311512 <<rexx_exec::Interp>::exec_parse+0x1c52>
  3129f7:	e9 9a eb ff ff       	jmp    311596 <<rexx_exec::Interp>::exec_parse+0x1cd6>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:56
  3129fc:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  312a01:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  312a06:	48 8b 54 24 30       	mov    0x30(%rsp),%rdx
  312a0b:	e8 28 b6 f6 ff       	call   27e038 <<rexx_exec::Interp>::traced_string_value_text>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312a10:	4c 8b 74 24 60       	mov    0x60(%rsp),%r14
  312a15:	4c 8b 64 24 68       	mov    0x68(%rsp),%r12
  312a1a:	48 8b 4c 24 70       	mov    0x70(%rsp),%rcx
  312a1f:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
  312a23:	74 3b                	je     312a60 <<rexx_exec::Interp>::exec_parse+0x31a0>
<rexx_exec::Interp>::apply_trigger:
  312a25:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  312a2a:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  312a31:	75 16                	jne    312a49 <<rexx_exec::Interp>::exec_parse+0x3189>
  312a33:	f6 87 2d 05 00 00 01 	testb  $0x1,0x52d(%rdi)
  312a3a:	74 0d                	je     312a49 <<rexx_exec::Interp>::exec_parse+0x3189>
<rexx_exec::Interp>::trace_result:
  312a3c:	48 8b 74 24 50       	mov    0x50(%rsp),%rsi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:225
  312a41:	4c 89 e2             	mov    %r12,%rdx
  312a44:	e8 f7 b5 f6 ff       	call   27e040 <<rexx_exec::Interp>::trace_result_line>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312a49:	4d 85 f6             	test   %r14,%r14
  312a4c:	0f 84 6f df ff ff    	je     3109c1 <<rexx_exec::Interp>::exec_parse+0x1101>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312a52:	4c 89 e7             	mov    %r12,%rdi
  312a55:	ff 15 3d 55 1c 00    	call   *0x1c553d(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  312a5b:	e9 61 df ff ff       	jmp    3109c1 <<rexx_exec::Interp>::exec_parse+0x1101>
<rexx_exec::Interp>::exec_parse:
  312a60:	48 89 ac 24 f0 00 00 	mov    %rbp,0xf0(%rsp)
  312a67:	00 
  312a68:	4c 89 f8             	mov    %r15,%rax
  312a6b:	49 89 df             	mov    %rbx,%r15
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312a6e:	49 83 fc ff          	cmp    $0xffffffffffffffff,%r12
  312a72:	0f 84 59 eb ff ff    	je     3115d1 <<rexx_exec::Interp>::exec_parse+0x1d11>
  312a78:	e9 da 0b 00 00       	jmp    313657 <<rexx_exec::Interp>::exec_parse+0x3d97>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  312a7d:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  312a82:	4c 89 e6             	mov    %r12,%rsi
  312a85:	48 8d 94 24 b8 01 00 	lea    0x1b8(%rsp),%rdx
  312a8c:	00 
  312a8d:	8b 8c 24 f8 01 00 00 	mov    0x1f8(%rsp),%ecx
  312a94:	44 8b 84 24 f4 01 00 	mov    0x1f4(%rsp),%r8d
  312a9b:	00 
  312a9c:	4c 8b 4c 24 50       	mov    0x50(%rsp),%r9
  312aa1:	e8 5a 3d 04 00       	call   356800 <<rexx_exec::Interp>::next_template>
<core::result::Result<(rexx_exec::parse_template::SourceText, rexx_exec::parse_template::Cursor), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312aa6:	4c 8b 7c 24 60       	mov    0x60(%rsp),%r15
  312aab:	48 8d 44 24 62       	lea    0x62(%rsp),%rax
  312ab0:	0f 10 40 06          	movups 0x6(%rax),%xmm0
  312ab4:	0f 29 84 24 d0 00 00 	movaps %xmm0,0xd0(%rsp)
  312abb:	00 
  312abc:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  312ac0:	0f 84 89 09 00 00    	je     31344f <<rexx_exec::Interp>::exec_parse+0x3b8f>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  312ac6:	48 8b 44 24 78       	mov    0x78(%rsp),%rax
  312acb:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
  312ad0:	48 8b 84 24 88 00 00 	mov    0x88(%rsp),%rax
  312ad7:	00 
  312ad8:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  312add:	48 8b 84 24 90 00 00 	mov    0x90(%rsp),%rax
  312ae4:	00 
  312ae5:	48 89 84 24 f0 00 00 	mov    %rax,0xf0(%rsp)
  312aec:	00 
  312aed:	48 8b 84 24 98 00 00 	mov    0x98(%rsp),%rax
  312af4:	00 
  312af5:	48 89 84 24 d8 01 00 	mov    %rax,0x1d8(%rsp)
  312afc:	00 
  312afd:	48 8b 9c 24 a0 00 00 	mov    0xa0(%rsp),%rbx
  312b04:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  312b05:	66 0f 6f 84 24 d0 00 	movdqa 0xd0(%rsp),%xmm0
  312b0c:	00 00 
  312b0e:	66 0f 7f 84 24 80 02 	movdqa %xmm0,0x280(%rsp)
  312b15:	00 00 
core::mem::replace::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  312b17:	4c 8b b4 24 00 01 00 	mov    0x100(%rsp),%r14
  312b1e:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  312b1f:	4c 89 bc 24 f8 00 00 	mov    %r15,0xf8(%rsp)
  312b26:	00 
  312b27:	48 8d 84 24 00 01 00 	lea    0x100(%rsp),%rax
  312b2e:	00 
  312b2f:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  312b33:	48 8b 8c 24 28 01 00 	mov    0x128(%rsp),%rcx
  312b3a:	00 
<rexx_exec::parse_template::SourceText>::into_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:97
  312b3b:	48 85 c9             	test   %rcx,%rcx
  312b3e:	0f 88 8e 00 00 00    	js     312bd2 <<rexx_exec::Interp>::exec_parse+0x3312>
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312b44:	48 8d 81 ff ef ff ff 	lea    -0x1001(%rcx),%rax
  312b4b:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  312b51:	77 10                	ja     312b63 <<rexx_exec::Interp>::exec_parse+0x32a3>
  312b53:	48 85 c9             	test   %rcx,%rcx
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312b56:	74 7a                	je     312bd2 <<rexx_exec::Interp>::exec_parse+0x3312>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312b58:	4c 89 f7             	mov    %r14,%rdi
  312b5b:	ff 15 37 54 1c 00    	call   *0x1c5437(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312b61:	eb 6f                	jmp    312bd2 <<rexx_exec::Interp>::exec_parse+0x3312>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312b63:	4d 8b a4 24 e0 0b 00 	mov    0xbe0(%r12),%r12
  312b6a:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312b6b:	49 83 fc 08          	cmp    $0x8,%r12
  312b6f:	72 10                	jb     312b81 <<rexx_exec::Interp>::exec_parse+0x32c1>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312b71:	4c 89 f7             	mov    %r14,%rdi
  312b74:	ff 15 1e 54 1c 00    	call   *0x1c541e(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  312b7a:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312b7f:	eb 51                	jmp    312bd2 <<rexx_exec::Interp>::exec_parse+0x3312>
  312b81:	48 8b 84 24 38 01 00 	mov    0x138(%rsp),%rax
  312b88:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312b89:	4c 3b 20             	cmp    (%rax),%r12
  312b8c:	75 0d                	jne    312b9b <<rexx_exec::Interp>::exec_parse+0x32db>
  312b8e:	48 8b bc 24 38 01 00 	mov    0x138(%rsp),%rdi
  312b95:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312b96:	e8 95 e3 17 00       	call   490f30 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  312b9b:	48 8b 54 24 18       	mov    0x18(%rsp),%rdx
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312ba0:	48 8b 82 d8 0b 00 00 	mov    0xbd8(%rdx),%rax
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  312ba7:	4b 8d 0c 64          	lea    (%r12,%r12,2),%rcx
  312bab:	48 8b b4 24 28 01 00 	mov    0x128(%rsp),%rsi
  312bb2:	00 
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312bb3:	48 89 34 c8          	mov    %rsi,(%rax,%rcx,8)
  312bb7:	4c 89 74 c8 08       	mov    %r14,0x8(%rax,%rcx,8)
  312bbc:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312bc3:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312bc5:	49 ff c4             	inc    %r12
  312bc8:	4c 89 a2 e0 0b 00 00 	mov    %r12,0xbe0(%rdx)
  312bcf:	49 89 d4             	mov    %rdx,%r12
<rexx_exec::Interp>::exec_parse:
  312bd2:	48 8b 44 24 58       	mov    0x58(%rsp),%rax
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312bd7:	48 3b 84 24 08 02 00 	cmp    0x208(%rsp),%rax
  312bde:	00 
  312bdf:	48 b9 01 00 00 00 00 	movabs $0x8000000000000001,%rcx
  312be6:	00 00 80 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312be9:	0f 85 57 dc ff ff    	jne    310846 <<rexx_exec::Interp>::exec_parse+0xf86>
  312bef:	e9 f6 00 00 00       	jmp    312cea <<rexx_exec::Interp>::exec_parse+0x342a>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  312bf4:	66 0f 6f 84 24 d0 00 	movdqa 0xd0(%rsp),%xmm0
  312bfb:	00 00 
  312bfd:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  312c04:	00 
  312c05:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  312c09:	eb 44                	jmp    312c4f <<rexx_exec::Interp>::exec_parse+0x338f>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  312c0b:	48 8b 5c 24 68       	mov    0x68(%rsp),%rbx
core::ptr::drop_glue::<core::option::Option<alloc::vec::Vec<u8>>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312c10:	49 ff ce             	dec    %r14
  312c13:	49 83 fe fd          	cmp    $0xfffffffffffffffd,%r14
  312c17:	77 0e                	ja     312c27 <<rexx_exec::Interp>::exec_parse+0x3367>
  312c19:	48 8b bc 24 18 01 00 	mov    0x118(%rsp),%rdi
  312c20:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312c21:	ff 15 71 53 1c 00    	call   *0x1c5371(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  312c27:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  312c2e:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  312c2f:	4c 89 38             	mov    %r15,(%rax)
  312c32:	48 89 58 08          	mov    %rbx,0x8(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312c36:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  312c3d:	00 00 
  312c3f:	7e 0e                	jle    312c4f <<rexx_exec::Interp>::exec_parse+0x338f>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  312c41:	48 8b bc 24 00 01 00 	mov    0x100(%rsp),%rdi
  312c48:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312c49:	ff 15 49 53 1c 00    	call   *0x1c5349(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  312c4f:	48 83 bc 24 b8 01 00 	cmpq   $0x0,0x1b8(%rsp)
  312c56:	00 00 
core::ptr::drop_glue::<rexx_exec::parse_template::ParseStrings>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312c58:	0f 8e b6 01 00 00    	jle    312e14 <<rexx_exec::Interp>::exec_parse+0x3554>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  312c5e:	48 8b bc 24 c0 01 00 	mov    0x1c0(%rsp),%rdi
  312c65:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312c66:	ff 15 2c 53 1c 00    	call   *0x1c532c(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  312c6c:	e9 a3 01 00 00       	jmp    312e14 <<rexx_exec::Interp>::exec_parse+0x3554>
<rexx_exec::Interp>::exec_parse:
  312c71:	48 8d 9c 24 e0 00 00 	lea    0xe0(%rsp),%rbx
  312c78:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  312c79:	4c 89 f0             	mov    %r14,%rax
  312c7c:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  312c80:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  312c82:	48 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%rsi)
  312c89:	0f 86 a9 d9 ff ff    	jbe    310638 <<rexx_exec::Interp>::exec_parse+0xd78>
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:408
  312c8f:	48 8b 86 00 0a 00 00 	mov    0xa00(%rsi),%rax
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  312c96:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  312c9a:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  312c9e:	83 3c 08 01          	cmpl   $0x1,(%rax,%rcx,1)
  312ca2:	0f 85 90 d9 ff ff    	jne    310638 <<rexx_exec::Interp>::exec_parse+0xd78>
  312ca8:	48 01 c8             	add    %rcx,%rax
  312cab:	80 78 08 00          	cmpb   $0x0,0x8(%rax)
  312caf:	0f 85 83 d9 ff ff    	jne    310638 <<rexx_exec::Interp>::exec_parse+0xd78>
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:408
  312cb5:	4c 89 f1             	mov    %r14,%rcx
  312cb8:	48 c1 e9 22          	shr    $0x22,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  312cbc:	39 48 04             	cmp    %ecx,0x4(%rax)
  312cbf:	0f 85 73 d9 ff ff    	jne    310638 <<rexx_exec::Interp>::exec_parse+0xd78>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  312cc5:	80 78 10 01          	cmpb   $0x1,0x10(%rax)
  312cc9:	0f 85 c6 07 00 00    	jne    313495 <<rexx_exec::Interp>::exec_parse+0x3bd5>
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  312ccf:	4c 8b 60 28          	mov    0x28(%rax),%r12
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:100
  312cd3:	e9 c2 07 00 00       	jmp    31349a <<rexx_exec::Interp>::exec_parse+0x3bda>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  312cd8:	48 8b 5c 24 68       	mov    0x68(%rsp),%rbx
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312cdd:	e9 be 01 00 00       	jmp    312ea0 <<rexx_exec::Interp>::exec_parse+0x35e0>
<rexx_exec::Interp>::exec_parse:
  312ce2:	4c 8b bc 24 28 01 00 	mov    0x128(%rsp),%r15
  312ce9:	00 
<rexx_exec::parse_template::SourceText>::into_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:97
  312cea:	4d 85 ff             	test   %r15,%r15
  312ced:	78 35                	js     312d24 <<rexx_exec::Interp>::exec_parse+0x3464>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:502
  312cef:	48 8b 9c 24 00 01 00 	mov    0x100(%rsp),%rbx
  312cf6:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312cf7:	49 8d 87 ff ef ff ff 	lea    -0x1001(%r15),%rax
  312cfe:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  312d04:	77 07                	ja     312d0d <<rexx_exec::Interp>::exec_parse+0x344d>
  312d06:	4d 85 ff             	test   %r15,%r15
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312d09:	75 10                	jne    312d1b <<rexx_exec::Interp>::exec_parse+0x345b>
  312d0b:	eb 17                	jmp    312d24 <<rexx_exec::Interp>::exec_parse+0x3464>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312d0d:	4d 8b b4 24 e0 0b 00 	mov    0xbe0(%r12),%r14
  312d14:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312d15:	49 83 fe 08          	cmp    $0x8,%r14
  312d19:	72 57                	jb     312d72 <<rexx_exec::Interp>::exec_parse+0x34b2>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312d1b:	48 89 df             	mov    %rbx,%rdi
  312d1e:	ff 15 74 52 1c 00    	call   *0x1c5274(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  312d24:	4c 8b b4 24 b8 01 00 	mov    0x1b8(%rsp),%r14
  312d2b:	00 
  312d2c:	4d 85 f6             	test   %r14,%r14
<rexx_exec::Interp>::give_parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:389
  312d2f:	0f 88 d0 00 00 00    	js     312e05 <<rexx_exec::Interp>::exec_parse+0x3545>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  312d35:	48 8b 9c 24 c0 01 00 	mov    0x1c0(%rsp),%rbx
  312d3c:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312d3d:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  312d44:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  312d4a:	77 0a                	ja     312d56 <<rexx_exec::Interp>::exec_parse+0x3496>
  312d4c:	4d 85 f6             	test   %r14,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312d4f:	75 13                	jne    312d64 <<rexx_exec::Interp>::exec_parse+0x34a4>
  312d51:	e9 af 00 00 00       	jmp    312e05 <<rexx_exec::Interp>::exec_parse+0x3545>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312d56:	4d 8b bc 24 e0 0b 00 	mov    0xbe0(%r12),%r15
  312d5d:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312d5e:	49 83 ff 08          	cmp    $0x8,%r15
  312d62:	72 61                	jb     312dc5 <<rexx_exec::Interp>::exec_parse+0x3505>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312d64:	48 89 df             	mov    %rbx,%rdi
  312d67:	ff 15 2b 52 1c 00    	call   *0x1c522b(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312d6d:	e9 93 00 00 00       	jmp    312e05 <<rexx_exec::Interp>::exec_parse+0x3545>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312d72:	4d 3b b4 24 d0 0b 00 	cmp    0xbd0(%r12),%r14
  312d79:	00 
  312d7a:	75 0d                	jne    312d89 <<rexx_exec::Interp>::exec_parse+0x34c9>
<rexx_exec::Interp>::give_parse_buffer:
  312d7c:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  312d83:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312d84:	e8 a7 e1 17 00       	call   490f30 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312d89:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  312d90:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  312d91:	4b 8d 0c 76          	lea    (%r14,%r14,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312d95:	4c 89 3c c8          	mov    %r15,(%rax,%rcx,8)
  312d99:	48 89 5c c8 08       	mov    %rbx,0x8(%rax,%rcx,8)
  312d9e:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312da5:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312da7:	49 ff c6             	inc    %r14
  312daa:	4d 89 b4 24 e0 0b 00 	mov    %r14,0xbe0(%r12)
  312db1:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  312db2:	4c 8b b4 24 b8 01 00 	mov    0x1b8(%rsp),%r14
  312db9:	00 
  312dba:	4d 85 f6             	test   %r14,%r14
<rexx_exec::Interp>::give_parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:389
  312dbd:	0f 89 72 ff ff ff    	jns    312d35 <<rexx_exec::Interp>::exec_parse+0x3475>
  312dc3:	eb 40                	jmp    312e05 <<rexx_exec::Interp>::exec_parse+0x3545>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312dc5:	4d 3b bc 24 d0 0b 00 	cmp    0xbd0(%r12),%r15
  312dcc:	00 
  312dcd:	75 0d                	jne    312ddc <<rexx_exec::Interp>::exec_parse+0x351c>
<rexx_exec::Interp>::give_parse_buffer:
  312dcf:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  312dd6:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312dd7:	e8 54 e1 17 00       	call   490f30 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312ddc:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  312de3:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  312de4:	4b 8d 0c 7f          	lea    (%r15,%r15,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312de8:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  312dec:	48 89 5c c8 08       	mov    %rbx,0x8(%rax,%rcx,8)
  312df1:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312df8:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312dfa:	49 ff c7             	inc    %r15
  312dfd:	4d 89 bc 24 e0 0b 00 	mov    %r15,0xbe0(%r12)
  312e04:	00 
<rexx_exec::Interp>::exec_parse:
  312e05:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  312e0c:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:504
  312e0d:	48 c7 00 ff ff ff ff 	movq   $0xffffffffffffffff,(%rax)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  312e14:	48 81 c4 98 02 00 00 	add    $0x298,%rsp
  312e1b:	5b                   	pop    %rbx
  312e1c:	41 5c                	pop    %r12
  312e1e:	41 5d                	pop    %r13
  312e20:	41 5e                	pop    %r14
  312e22:	41 5f                	pop    %r15
  312e24:	5d                   	pop    %rbp
  312e25:	c3                   	ret
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  312e26:	bf 34 00 00 00       	mov    $0x34,%edi
  312e2b:	ff 15 87 51 1c 00    	call   *0x1c5187(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312e31:	48 85 c0             	test   %rax,%rax
  312e34:	0f 84 a7 08 00 00    	je     3136e1 <<rexx_exec::Interp>::exec_parse+0x3e21>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  312e3a:	49 89 c6             	mov    %rax,%r14
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312e3d:	0f 10 05 1c 78 d5 ff 	movups -0x2a87e4(%rip),%xmm0        # 6a660 <panic_unwind::imp::CANARY+0x5903>
  312e44:	0f 11 40 20          	movups %xmm0,0x20(%rax)
  312e48:	0f 10 05 01 78 d5 ff 	movups -0x2a87ff(%rip),%xmm0        # 6a650 <panic_unwind::imp::CANARY+0x58f3>
  312e4f:	0f 11 40 10          	movups %xmm0,0x10(%rax)
  312e53:	f3 0f 6f 05 e5 77 d5 	movdqu -0x2a881b(%rip),%xmm0        # 6a640 <panic_unwind::imp::CANARY+0x58e3>
  312e5a:	ff 
  312e5b:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  312e5f:	c7 40 30 72 61 6e 64 	movl   $0x646e6172,0x30(%rax)
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  312e66:	bf 18 00 00 00       	mov    $0x18,%edi
  312e6b:	ff 15 47 51 1c 00    	call   *0x1c5147(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  312e71:	48 85 c0             	test   %rax,%rax
  312e74:	0f 84 7b 07 00 00    	je     3135f5 <<rexx_exec::Interp>::exec_parse+0x3d35>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  312e7a:	48 89 c3             	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  312e7d:	48 c7 00 34 00 00 00 	movq   $0x34,(%rax)
  312e84:	4c 89 70 08          	mov    %r14,0x8(%rax)
  312e88:	48 c7 40 10 34 00 00 	movq   $0x34,0x10(%rax)
  312e8f:	00 
  312e90:	41 bc 02 00 00 00    	mov    $0x2,%r12d
<rexx_exec::Interp>::apply_trigger:
  312e96:	eb 08                	jmp    312ea0 <<rexx_exec::Interp>::exec_parse+0x35e0>
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  312e98:	48 8b 9c 24 d8 00 00 	mov    0xd8(%rsp),%rbx
  312e9f:	00 
<rexx_exec::Interp>::exec_parse:
  312ea0:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  312ea7:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  312ea8:	4c 89 20             	mov    %r12,(%rax)
  312eab:	48 89 58 08          	mov    %rbx,0x8(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312eaf:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  312eb6:	00 00 
  312eb8:	0f 8f 83 fd ff ff    	jg     312c41 <<rexx_exec::Interp>::exec_parse+0x3381>
  312ebe:	e9 8c fd ff ff       	jmp    312c4f <<rexx_exec::Interp>::exec_parse+0x338f>
<rexx_exec::Interp>::traced_into_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:353
  312ec3:	e8 98 36 04 00       	call   356560 <<rexx_exec::Interp>::object_into_parse_buffer>
  312ec8:	e9 0a d5 ff ff       	jmp    3103d7 <<rexx_exec::Interp>::exec_parse+0xb17>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  312ecd:	bf 18 00 00 00       	mov    $0x18,%edi
  312ed2:	ff 15 e0 50 1c 00    	call   *0x1c50e0(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  312ed8:	48 85 c0             	test   %rax,%rax
  312edb:	0f 84 4e 07 00 00    	je     31362f <<rexx_exec::Interp>::exec_parse+0x3d6f>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:100
  312ee1:	49 89 c7             	mov    %rax,%r15
  312ee4:	48 8d bc 24 d0 00 00 	lea    0xd0(%rsp),%rdi
  312eeb:	00 
  312eec:	4c 89 f6             	mov    %r14,%rsi
  312eef:	48 8b 54 24 30       	mov    0x30(%rsp),%rdx
  312ef4:	e8 57 ca e9 ff       	call   1af950 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  312ef9:	48 8b ac 24 d0 00 00 	mov    0xd0(%rsp),%rbp
  312f00:	00 
  312f01:	4c 8b b4 24 d8 00 00 	mov    0xd8(%rsp),%r14
  312f08:	00 
  312f09:	4c 8b a4 24 e0 00 00 	mov    0xe0(%rsp),%r12
  312f10:	00 
  312f11:	4d 85 e4             	test   %r12,%r12
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  312f14:	0f 89 f1 00 00 00    	jns    31300b <<rexx_exec::Interp>::exec_parse+0x374b>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  312f1a:	45 31 ed             	xor    %r13d,%r13d
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  312f1d:	4c 89 ef             	mov    %r13,%rdi
  312f20:	4c 89 e6             	mov    %r12,%rsi
  312f23:	e8 dc eb df ff       	call   111b04 <alloc::raw_vec::handle_error>
  312f28:	e9 d4 07 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::exec_parse:
  312f2d:	48 89 94 24 20 01 00 	mov    %rdx,0x120(%rsp)
  312f34:	00 
<rexx_exec::Code>::stem:
  312f35:	48 01 c8             	add    %rcx,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1287
  312f38:	48 8b 48 08          	mov    0x8(%rax),%rcx
  312f3c:	48 89 8c 24 e8 01 00 	mov    %rcx,0x1e8(%rsp)
  312f43:	00 
  312f44:	48 8b 48 10          	mov    0x10(%rax),%rcx
  312f48:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
  312f4d:	48 8b 40 18          	mov    0x18(%rax),%rax
  312f51:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  312f56:	e9 93 d2 ff ff       	jmp    3101ee <<rexx_exec::Interp>::exec_parse+0x92e>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:413
  312f5b:	48 8d 0d e6 a2 1a 00 	lea    0x1aa2e6(%rip),%rcx        # 4bd248 <rexx_api::ffi::CALL_CONTEXT+0x1eb8>
  312f62:	4c 89 ff             	mov    %r15,%rdi
  312f65:	4c 89 f6             	mov    %r14,%rsi
  312f68:	48 89 c2             	mov    %rax,%rdx
  312f6b:	e8 b0 97 e0 ff       	call   11c720 <core::slice::index::slice_index_fail>
  312f70:	e9 8c 07 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312f75:	49 8d bf e0 0a 00 00 	lea    0xae0(%r15),%rdi
  312f7c:	e8 8f 8f e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  312f81:	e9 eb d3 ff ff       	jmp    310371 <<rexx_exec::Interp>::exec_parse+0xab1>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  312f86:	0f 84 1f 01 00 00    	je     3130ab <<rexx_exec::Interp>::exec_parse+0x37eb>
  312f8c:	bd 01 00 00 00       	mov    $0x1,%ebp
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  312f91:	be 01 00 00 00       	mov    $0x1,%esi
  312f96:	48 89 df             	mov    %rbx,%rdi
  312f99:	e8 e2 47 e1 ff       	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312f9e:	48 85 c0             	test   %rax,%rax
  312fa1:	0f 84 49 d3 ff ff    	je     3102f0 <<rexx_exec::Interp>::exec_parse+0xa30>
<alloc::vec::Vec<u8>>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:969
  312fa7:	48 89 5c 24 60       	mov    %rbx,0x60(%rsp)
  312fac:	48 89 44 24 68       	mov    %rax,0x68(%rsp)
  312fb1:	48 89 44 24 58       	mov    %rax,0x58(%rsp)
  312fb6:	48 89 c7             	mov    %rax,%rdi
  312fb9:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312fbe:	48 89 da             	mov    %rbx,%rdx
  312fc1:	ff 15 01 50 1c 00    	call   *0x1c5001(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<u8>>::set_len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2224
  312fc7:	48 89 5c 24 70       	mov    %rbx,0x70(%rsp)
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  312fcc:	e9 ff 00 00 00       	jmp    3130d0 <<rexx_exec::Interp>::exec_parse+0x3810>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  312fd1:	0f 84 8a 02 00 00    	je     313261 <<rexx_exec::Interp>::exec_parse+0x39a1>
  312fd7:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  312fdc:	be 01 00 00 00       	mov    $0x1,%esi
  312fe1:	48 89 ef             	mov    %rbp,%rdi
  312fe4:	e8 97 47 e1 ff       	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312fe9:	48 85 c0             	test   %rax,%rax
  312fec:	0f 84 fb d0 ff ff    	je     3100ed <<rexx_exec::Interp>::exec_parse+0x82d>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  312ff2:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312ff5:	48 89 c7             	mov    %rax,%rdi
  312ff8:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
  312ffd:	48 89 ea             	mov    %rbp,%rdx
  313000:	ff 15 c2 4f 1c 00    	call   *0x1c4fc2(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  313006:	e9 5c 02 00 00       	jmp    313267 <<rexx_exec::Interp>::exec_parse+0x39a7>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  31300b:	0f 84 a7 02 00 00    	je     3132b8 <<rexx_exec::Interp>::exec_parse+0x39f8>
  313011:	41 bd 01 00 00 00    	mov    $0x1,%r13d
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  313017:	be 01 00 00 00       	mov    $0x1,%esi
  31301c:	4c 89 e7             	mov    %r12,%rdi
  31301f:	e8 5c 47 e1 ff       	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  313024:	48 85 c0             	test   %rax,%rax
  313027:	0f 84 f0 fe ff ff    	je     312f1d <<rexx_exec::Interp>::exec_parse+0x365d>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  31302d:	48 89 c3             	mov    %rax,%rbx
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  313030:	48 89 c7             	mov    %rax,%rdi
  313033:	4c 89 f6             	mov    %r14,%rsi
  313036:	4c 89 e2             	mov    %r12,%rdx
  313039:	ff 15 89 4f 1c 00    	call   *0x1c4f89(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  31303f:	e9 79 02 00 00       	jmp    3132bd <<rexx_exec::Interp>::exec_parse+0x39fd>
<rexx_exec::Interp>::assign_targets:
  313044:	48 8d 0d 6d 76 1a 00 	lea    0x1a766d(%rip),%rcx        # 4ba6b8 <rexx_api::ffi::INSTANCE+0x5ef8>
  31304b:	ba 07 00 00 00       	mov    $0x7,%edx
  313050:	31 ff                	xor    %edi,%edi
  313052:	48 89 c6             	mov    %rax,%rsi
  313055:	e8 c6 96 e0 ff       	call   11c720 <core::slice::index::slice_index_fail>
  31305a:	e9 a2 06 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31305f:	49 8d bc 24 e0 0a 00 	lea    0xae0(%r12),%rdi
  313066:	00 
  313067:	e8 a4 8e e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  31306c:	e9 c7 d4 ff ff       	jmp    310538 <<rexx_exec::Interp>::exec_parse+0xc78>
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  313071:	48 8d 3d e2 6d d5 ff 	lea    -0x2a921e(%rip),%rdi        # 69e5a <panic_unwind::imp::CANARY+0x50fd>
  313078:	48 8d 15 29 d1 1a 00 	lea    0x1ad129(%rip),%rdx        # 4c01a8 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  31307f:	be 11 00 00 00       	mov    $0x11,%esi
  313084:	e8 87 bf e0 ff       	call   11f010 <core::option::expect_failed>
  313089:	e9 73 06 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  31308e:	48 8d 3d a2 5f d5 ff 	lea    -0x2aa05e(%rip),%rdi        # 69037 <panic_unwind::imp::CANARY+0x42da>
  313095:	48 8d 15 f4 a1 1a 00 	lea    0x1aa1f4(%rip),%rdx        # 4bd290 <rexx_api::ffi::CALL_CONTEXT+0x1f00>
  31309c:	be 36 00 00 00       	mov    $0x36,%esi
  3130a1:	e8 6a bf e0 ff       	call   11f010 <core::option::expect_failed>
  3130a6:	e9 56 06 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<alloc::vec::Vec<u8>>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:969
  3130ab:	48 c7 44 24 60 00 00 	movq   $0x0,0x60(%rsp)
  3130b2:	00 00 
  3130b4:	48 c7 44 24 68 01 00 	movq   $0x1,0x68(%rsp)
  3130bb:	00 00 
  3130bd:	48 c7 44 24 70 00 00 	movq   $0x0,0x70(%rsp)
  3130c4:	00 00 
  3130c6:	b8 01 00 00 00       	mov    $0x1,%eax
  3130cb:	48 89 44 24 58       	mov    %rax,0x58(%rsp)
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  3130d0:	48 8b ac 24 b0 01 00 	mov    0x1b0(%rsp),%rbp
  3130d7:	00 
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  3130d8:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  3130db:	0f 85 7e 05 00 00    	jne    31365f <<rexx_exec::Interp>::exec_parse+0x3d9f>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  3130e1:	48 01 eb             	add    %rbp,%rbx
  3130e4:	48 89 5c 24 70       	mov    %rbx,0x70(%rsp)
  3130e9:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  3130ee:	41 80 be 35 05 00 00 	cmpb   $0x0,0x535(%r14)
  3130f5:	00 
  3130f6:	75 52                	jne    31314a <<rexx_exec::Interp>::exec_parse+0x388a>
  3130f8:	41 f6 86 2e 05 00 00 	testb  $0x1,0x52e(%r14)
  3130ff:	01 
  313100:	74 48                	je     31314a <<rexx_exec::Interp>::exec_parse+0x388a>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  313102:	49 8d be d8 10 00 00 	lea    0x10d8(%r14),%rdi
  313109:	49 8b ae e8 10 00 00 	mov    0x10e8(%r14),%rbp
<rexx_exec::Interp>::trace_compound_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:594
  313110:	48 89 5c 24 10       	mov    %rbx,0x10(%rsp)
  313115:	48 8b 44 24 58       	mov    0x58(%rsp),%rax
  31311a:	48 89 44 24 08       	mov    %rax,0x8(%rsp)
  31311f:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  313126:	48 8d 35 78 7a d5 ff 	lea    -0x2a8588(%rip),%rsi        # 6aba5 <panic_unwind::imp::CANARY+0x5e48>
  31312d:	48 8b 54 24 50       	mov    0x50(%rsp),%rdx
  313132:	31 c9                	xor    %ecx,%ecx
  313134:	4d 89 e8             	mov    %r13,%r8
  313137:	4d 89 e1             	mov    %r12,%r9
  31313a:	e8 b1 d6 f6 ff       	call   2807f0 <rexx_exec::trace::format::push_tagged>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:595
  31313f:	4c 89 f7             	mov    %r14,%rdi
  313142:	48 89 ee             	mov    %rbp,%rsi
  313145:	e8 06 b0 e8 ff       	call   19e150 <<rexx_exec::Interp>::route_trace_line>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31314a:	48 83 7c 24 60 00    	cmpq   $0x0,0x60(%rsp)
  313150:	74 0b                	je     31315d <<rexx_exec::Interp>::exec_parse+0x389d>
<alloc::raw_vec::RawVecInner>::deallocate:
  313152:	48 8b 7c 24 58       	mov    0x58(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313157:	ff 15 3b 4e 1c 00    	call   *0x1c4e3b(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  31315d:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  313162:	4c 8b 8c 24 a8 01 00 	mov    0x1a8(%rsp),%r9
  313169:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  31316a:	48 8b 84 24 b0 01 00 	mov    0x1b0(%rsp),%rax
  313171:	00 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:706
  313172:	48 89 04 24          	mov    %rax,(%rsp)
  313176:	4c 89 f7             	mov    %r14,%rdi
  313179:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
  31317e:	48 8b 54 24 28       	mov    0x28(%rsp),%rdx
  313183:	48 8b 8c 24 20 01 00 	mov    0x120(%rsp),%rcx
  31318a:	00 
  31318b:	4c 8b 84 24 e8 01 00 	mov    0x1e8(%rsp),%r8
  313192:	00 
  313193:	e8 b8 e2 fa ff       	call   2c1450 <<rexx_exec::Interp>::stem_get_at>
  313198:	49 89 c6             	mov    %rax,%r14
  31319b:	89 d3                	mov    %edx,%ebx
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31319d:	49 83 3f 00          	cmpq   $0x0,(%r15)
  3131a1:	74 12                	je     3131b5 <<rexx_exec::Interp>::exec_parse+0x38f5>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
  3131a3:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3131a8:	48 8b b8 c0 0b 00 00 	mov    0xbc0(%rax),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3131af:	ff 15 e3 4d 1c 00    	call   *0x1c4de3(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::give_key_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:64
  3131b5:	48 8b 84 24 b0 01 00 	mov    0x1b0(%rsp),%rax
  3131bc:	00 
  3131bd:	49 89 47 10          	mov    %rax,0x10(%r15)
  3131c1:	66 0f 6f 84 24 a0 01 	movdqa 0x1a0(%rsp),%xmm0
  3131c8:	00 00 
  3131ca:	f3 41 0f 7f 07       	movdqu %xmm0,(%r15)
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:315
  3131cf:	f6 c3 01             	test   $0x1,%bl
  3131d2:	0f 85 31 04 00 00    	jne    313609 <<rexx_exec::Interp>::exec_parse+0x3d49>
<rexx_exec::Interp>::read_parse_var:
  3131d8:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  3131dd:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  3131e4:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:709
  3131e5:	0f 84 72 d1 ff ff    	je     31035d <<rexx_exec::Interp>::exec_parse+0xa9d>
  3131eb:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:710
  3131f0:	4c 89 fe             	mov    %r15,%rsi
  3131f3:	4c 89 f2             	mov    %r14,%rdx
  3131f6:	e8 55 c7 e9 ff       	call   1af950 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  3131fb:	48 8b 4c 24 60       	mov    0x60(%rsp),%rcx
  313200:	48 8b 44 24 68       	mov    0x68(%rsp),%rax
  313205:	48 89 44 24 20       	mov    %rax,0x20(%rsp)
  31320a:	48 8b 6c 24 70       	mov    0x70(%rsp),%rbp
  31320f:	48 85 ed             	test   %rbp,%rbp
  313212:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  313217:	79 12                	jns    31322b <<rexx_exec::Interp>::exec_parse+0x396b>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  313219:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  31321b:	48 89 df             	mov    %rbx,%rdi
  31321e:	48 89 ee             	mov    %rbp,%rsi
  313221:	e8 de e8 df ff       	call   111b04 <alloc::raw_vec::handle_error>
  313226:	e9 d6 04 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  31322b:	0f 84 c7 01 00 00    	je     3133f8 <<rexx_exec::Interp>::exec_parse+0x3b38>
  313231:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  313236:	be 01 00 00 00       	mov    $0x1,%esi
  31323b:	48 89 ef             	mov    %rbp,%rdi
  31323e:	e8 3d 45 e1 ff       	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  313243:	48 85 c0             	test   %rax,%rax
  313246:	74 d3                	je     31321b <<rexx_exec::Interp>::exec_parse+0x395b>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  313248:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  31324b:	48 89 c7             	mov    %rax,%rdi
  31324e:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
  313253:	48 89 ea             	mov    %rbp,%rdx
  313256:	ff 15 6c 4d 1c 00    	call   *0x1c4d6c(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  31325c:	e9 9d 01 00 00       	jmp    3133fe <<rexx_exec::Interp>::exec_parse+0x3b3e>
<rexx_exec::Interp>::read_parse_var:
  313261:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313267:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  31326d:	7e 0b                	jle    31327a <<rexx_exec::Interp>::exec_parse+0x39ba>
  31326f:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313274:	ff 15 1e 4d 1c 00    	call   *0x1c4d1e(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  31327a:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31327f:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  313286:	0f 85 be d0 ff ff    	jne    31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
  31328c:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  313293:	0f 84 b1 d0 ff ff    	je     31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  313299:	4c 89 34 24          	mov    %r14,(%rsp)
  31329d:	48 8b 74 24 50       	mov    0x50(%rsp),%rsi
  3132a2:	4c 89 ea             	mov    %r13,%rdx
  3132a5:	4c 89 e1             	mov    %r12,%rcx
  3132a8:	4d 89 f8             	mov    %r15,%r8
  3132ab:	49 89 e9             	mov    %rbp,%r9
  3132ae:	e8 3d df f6 ff       	call   2811f0 <<rexx_exec::Interp>::trace_variable_line>
  3132b3:	e9 92 d0 ff ff       	jmp    31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
<rexx_exec::Interp>::exec_parse:
  3132b8:	bb 01 00 00 00       	mov    $0x1,%ebx
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:100
  3132bd:	4d 89 27             	mov    %r12,(%r15)
  3132c0:	49 89 5f 08          	mov    %rbx,0x8(%r15)
  3132c4:	4d 89 67 10          	mov    %r12,0x10(%r15)
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  3132c8:	bf 02 00 00 00       	mov    $0x2,%edi
  3132cd:	be 01 00 00 00       	mov    $0x1,%esi
  3132d2:	e8 a9 44 e1 ff       	call   127780 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  3132d7:	48 85 c0             	test   %rax,%rax
  3132da:	0f 84 12 04 00 00    	je     3136f2 <<rexx_exec::Interp>::exec_parse+0x3e32>
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  3132e0:	66 c7 00 32 36       	movw   $0x3632,(%rax)
<rexx_exec::error::Raised>::syntax:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:139
  3132e5:	48 c7 44 24 78 ff ff 	movq   $0xffffffffffffffff,0x78(%rsp)
  3132ec:	ff ff 
  3132ee:	48 8d 0d 22 39 d9 ff 	lea    -0x26c6de(%rip),%rcx        # a6c17 <panic_unwind::imp::CANARY+0x41eba>
  3132f5:	48 89 8c 24 80 00 00 	mov    %rcx,0x80(%rsp)
  3132fc:	00 
  3132fd:	48 c7 84 24 88 00 00 	movq   $0x6,0x88(%rsp)
  313304:	00 06 00 00 00 
  313309:	48 c7 44 24 60 01 00 	movq   $0x1,0x60(%rsp)
  313310:	00 00 
  313312:	4c 89 7c 24 68       	mov    %r15,0x68(%rsp)
  313317:	48 c7 44 24 70 01 00 	movq   $0x1,0x70(%rsp)
  31331e:	00 00 
  313320:	48 c7 84 24 90 00 00 	movq   $0x2,0x90(%rsp)
  313327:	00 02 00 00 00 
  31332c:	48 89 84 24 98 00 00 	mov    %rax,0x98(%rsp)
  313333:	00 
  313334:	48 c7 84 24 a0 00 00 	movq   $0x2,0xa0(%rsp)
  31333b:	00 02 00 00 00 
  313340:	48 c7 84 24 a8 00 00 	movq   $0xffffffffffffffff,0xa8(%rsp)
  313347:	00 ff ff ff ff 
  31334c:	48 b8 00 00 00 00 1a 	movabs $0x4001a00000000,%rax
  313353:	00 04 00 
  313356:	48 89 84 24 c0 00 00 	mov    %rax,0xc0(%rsp)
  31335d:	00 
  31335e:	c7 84 24 c8 00 00 00 	movl   $0x0,0xc8(%rsp)
  313365:	00 00 00 00 
  313369:	66 c7 84 24 cc 00 00 	movw   $0x0,0xcc(%rsp)
  313370:	00 00 00 
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  313373:	bf 70 00 00 00       	mov    $0x70,%edi
  313378:	ff 15 3a 4c 1c 00    	call   *0x1c4c3a(%rip)        # 4d7fb8 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  31337e:	48 85 c0             	test   %rax,%rax
  313381:	0f 84 bc 02 00 00    	je     313643 <<rexx_exec::Interp>::exec_parse+0x3d83>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
  313387:	48 89 c3             	mov    %rax,%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  31338a:	0f 10 84 24 c0 00 00 	movups 0xc0(%rsp),%xmm0
  313391:	00 
  313392:	0f 11 40 60          	movups %xmm0,0x60(%rax)
  313396:	0f 10 84 24 b0 00 00 	movups 0xb0(%rsp),%xmm0
  31339d:	00 
  31339e:	0f 11 40 50          	movups %xmm0,0x50(%rax)
  3133a2:	0f 10 84 24 a0 00 00 	movups 0xa0(%rsp),%xmm0
  3133a9:	00 
  3133aa:	0f 11 40 40          	movups %xmm0,0x40(%rax)
  3133ae:	f3 0f 6f 44 24 60    	movdqu 0x60(%rsp),%xmm0
  3133b4:	f3 0f 6f 4c 24 70    	movdqu 0x70(%rsp),%xmm1
  3133ba:	f3 0f 6f 94 24 80 00 	movdqu 0x80(%rsp),%xmm2
  3133c1:	00 00 
  3133c3:	f3 0f 6f 9c 24 90 00 	movdqu 0x90(%rsp),%xmm3
  3133ca:	00 00 
  3133cc:	f3 0f 7f 58 30       	movdqu %xmm3,0x30(%rax)
  3133d1:	f3 0f 7f 50 20       	movdqu %xmm2,0x20(%rax)
  3133d6:	f3 0f 7f 48 10       	movdqu %xmm1,0x10(%rax)
  3133db:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3133df:	48 85 ed             	test   %rbp,%rbp
  3133e2:	7e 09                	jle    3133ed <<rexx_exec::Interp>::exec_parse+0x3b2d>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3133e4:	4c 89 f7             	mov    %r14,%rdi
  3133e7:	ff 15 ab 4b 1c 00    	call   *0x1c4bab(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  3133ed:	41 bc 03 00 00 00    	mov    $0x3,%r12d
  3133f3:	e9 a8 fa ff ff       	jmp    312ea0 <<rexx_exec::Interp>::exec_parse+0x35e0>
  3133f8:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3133fe:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  313404:	7e 0b                	jle    313411 <<rexx_exec::Interp>::exec_parse+0x3b51>
  313406:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31340b:	ff 15 87 4b 1c 00    	call   *0x1c4b87(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  313411:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  313416:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  31341d:	0f 85 27 cf ff ff    	jne    31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
  313423:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  31342a:	0f 84 1a cf ff ff    	je     31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  313430:	4c 89 34 24          	mov    %r14,(%rsp)
  313434:	48 8b 74 24 50       	mov    0x50(%rsp),%rsi
  313439:	4c 89 ea             	mov    %r13,%rdx
  31343c:	4c 89 e1             	mov    %r12,%rcx
  31343f:	4d 89 f8             	mov    %r15,%r8
  313442:	49 89 e9             	mov    %rbp,%r9
  313445:	e8 a6 dd f6 ff       	call   2811f0 <<rexx_exec::Interp>::trace_variable_line>
  31344a:	e9 fb ce ff ff       	jmp    31034a <<rexx_exec::Interp>::exec_parse+0xa8a>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  31344f:	66 0f 6f 84 24 d0 00 	movdqa 0xd0(%rsp),%xmm0
  313456:	00 00 
  313458:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  31345f:	00 
  313460:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313464:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  31346b:	00 00 
  31346d:	0f 8f ce f7 ff ff    	jg     312c41 <<rexx_exec::Interp>::exec_parse+0x3381>
  313473:	e9 d7 f7 ff ff       	jmp    312c4f <<rexx_exec::Interp>::exec_parse+0x338f>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  313478:	48 8d 3d ff 77 d5 ff 	lea    -0x2a8801(%rip),%rdi        # 6ac7e <panic_unwind::imp::CANARY+0x5f21>
  31347f:	48 8d 15 42 d4 1a 00 	lea    0x1ad442(%rip),%rdx        # 4c08c8 <rexx_api::ffi::CALL_CONTEXT+0x5538>
  313486:	be 31 00 00 00       	mov    $0x31,%esi
  31348b:	e8 60 93 e0 ff       	call   11c7f0 <core::panicking::panic>
  313490:	e9 6c 02 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  313495:	44 0f b6 60 11       	movzbl 0x11(%rax),%r12d
<rexx_exec::Interp>::source_text:
  31349a:	48 8b 6c 24 18       	mov    0x18(%rsp),%rbp
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  31349f:	4c 8b ad f0 0a 00 00 	mov    0xaf0(%rbp),%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3134a6:	4c 3b ad e0 0a 00 00 	cmp    0xae0(%rbp),%r13
  3134ad:	74 5e                	je     31350d <<rexx_exec::Interp>::exec_parse+0x3c4d>
  3134af:	49 bf 01 00 00 00 00 	movabs $0x8000000000000001,%r15
  3134b6:	00 00 80 
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3134b9:	48 8b 85 e8 0a 00 00 	mov    0xae8(%rbp),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3134c0:	4e 89 34 e8          	mov    %r14,(%rax,%r13,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3134c4:	49 ff c5             	inc    %r13
  3134c7:	4c 89 ad f0 0a 00 00 	mov    %r13,0xaf0(%rbp)
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:414
  3134ce:	4c 89 b4 24 d8 00 00 	mov    %r14,0xd8(%rsp)
  3134d5:	00 
  3134d6:	e9 d3 d1 ff ff       	jmp    3106ae <<rexx_exec::Interp>::exec_parse+0xdee>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:558
  3134db:	48 8d 0d 56 0d 1b 00 	lea    0x1b0d56(%rip),%rcx        # 4c4238 <rexx_api::ffi::CALL_CONTEXT+0x8ea8>
  3134e2:	48 89 ef             	mov    %rbp,%rdi
  3134e5:	48 89 d6             	mov    %rdx,%rsi
  3134e8:	e8 33 92 e0 ff       	call   11c720 <core::slice::index::slice_index_fail>
  3134ed:	e9 0f 02 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:413
  3134f2:	48 8d 0d bf 71 1a 00 	lea    0x1a71bf(%rip),%rcx        # 4ba6b8 <rexx_api::ffi::INSTANCE+0x5ef8>
  3134f9:	ba 07 00 00 00       	mov    $0x7,%edx
  3134fe:	31 ff                	xor    %edi,%edi
  313500:	48 89 de             	mov    %rbx,%rsi
  313503:	e8 18 92 e0 ff       	call   11c720 <core::slice::index::slice_index_fail>
  313508:	e9 f4 01 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31350d:	48 8d bd e0 0a 00 00 	lea    0xae0(%rbp),%rdi
  313514:	e8 f7 89 e4 ff       	call   15bf10 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  313519:	eb 94                	jmp    3134af <<rexx_exec::Interp>::exec_parse+0x3bef>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:413
  31351b:	48 8d 0d 96 71 1a 00 	lea    0x1a7196(%rip),%rcx        # 4ba6b8 <rexx_api::ffi::INSTANCE+0x5ef8>
  313522:	ba 07 00 00 00       	mov    $0x7,%edx
  313527:	31 ff                	xor    %edi,%edi
  313529:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  31352e:	e8 ed 91 e0 ff       	call   11c720 <core::slice::index::slice_index_fail>
  313533:	e9 c9 01 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::exec_parse:
  313538:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:318
  31353d:	4c 89 fe             	mov    %r15,%rsi
  313540:	4c 89 f2             	mov    %r14,%rdx
  313543:	e8 08 f5 f6 ff       	call   282a50 <<rexx_exec::Interp>::novalue_raised>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  313548:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
  31354d:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  313551:	0f 85 12 c6 ff ff    	jne    30fb69 <<rexx_exec::Interp>::exec_parse+0x2a9>
  313557:	e9 c6 cb ff ff       	jmp    310122 <<rexx_exec::Interp>::exec_parse+0x862>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:413
  31355c:	48 8d 0d 55 71 1a 00 	lea    0x1a7155(%rip),%rcx        # 4ba6b8 <rexx_api::ffi::INSTANCE+0x5ef8>
  313563:	ba 07 00 00 00       	mov    $0x7,%edx
  313568:	31 ff                	xor    %edi,%edi
  31356a:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  31356f:	e8 ac 91 e0 ff       	call   11c720 <core::slice::index::slice_index_fail>
  313574:	e9 88 01 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  313579:	48 8d 3d e7 5e d5 ff 	lea    -0x2aa119(%rip),%rdi        # 69467 <panic_unwind::imp::CANARY+0x470a>
  313580:	48 8d 15 71 a3 1a 00 	lea    0x1aa371(%rip),%rdx        # 4bd8f8 <rexx_api::ffi::CALL_CONTEXT+0x2568>
  313587:	be 35 00 00 00       	mov    $0x35,%esi
  31358c:	e8 7f ba e0 ff       	call   11f010 <core::option::expect_failed>
  313591:	e9 6b 01 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::assign_targets:
  313596:	48 8d 3d 8f 4e d5 ff 	lea    -0x2ab171(%rip),%rdi        # 6842c <panic_unwind::imp::CANARY+0x36cf>
  31359d:	48 8d 15 64 93 1a 00 	lea    0x1a9364(%rip),%rdx        # 4bc908 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  3135a4:	be ab 00 00 00       	mov    $0xab,%esi
  3135a9:	e8 92 73 e0 ff       	call   11a940 <core::panicking::panic_fmt>
  3135ae:	e9 4e 01 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
  3135b3:	48 8d 0d b6 82 1a 00 	lea    0x1a82b6(%rip),%rcx        # 4bb870 <rexx_api::ffi::CALL_CONTEXT+0x4e0>
  3135ba:	48 89 8c 24 d0 01 00 	mov    %rcx,0x1d0(%rsp)
  3135c1:	00 
  3135c2:	48 89 c6             	mov    %rax,%rsi
  3135c5:	48 8b 94 24 d0 01 00 	mov    0x1d0(%rsp),%rdx
  3135cc:	00 
  3135cd:	e8 2f 92 e0 ff       	call   11c801 <core::panicking::panic_bounds_check>
  3135d2:	e9 2a 01 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:545
  3135d7:	48 8d 0d 32 a3 1a 00 	lea    0x1aa332(%rip),%rcx        # 4bd910 <rexx_api::ffi::CALL_CONTEXT+0x2580>
  3135de:	be 14 00 00 00       	mov    $0x14,%esi
  3135e3:	ba 14 00 00 00       	mov    $0x14,%edx
  3135e8:	4c 89 cf             	mov    %r9,%rdi
  3135eb:	e8 30 91 e0 ff       	call   11c720 <core::slice::index::slice_index_fail>
  3135f0:	e9 0c 01 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  3135f5:	bf 08 00 00 00       	mov    $0x8,%edi
  3135fa:	be 18 00 00 00       	mov    $0x18,%esi
  3135ff:	e8 13 e5 df ff       	call   111b17 <alloc::alloc::handle_alloc_error>
  313604:	e9 f8 00 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::exec_parse:
  313609:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
  31360e:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:318
  313613:	4c 89 f2             	mov    %r14,%rdx
  313616:	e8 35 f4 f6 ff       	call   282a50 <<rexx_exec::Interp>::novalue_raised>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31361b:	48 8b 6c 24 60       	mov    0x60(%rsp),%rbp
  313620:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  313624:	0f 85 3f c5 ff ff    	jne    30fb69 <<rexx_exec::Interp>::exec_parse+0x2a9>
  31362a:	e9 a9 fb ff ff       	jmp    3131d8 <<rexx_exec::Interp>::exec_parse+0x3918>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  31362f:	bf 08 00 00 00       	mov    $0x8,%edi
  313634:	be 18 00 00 00       	mov    $0x18,%esi
  313639:	e8 d9 e4 df ff       	call   111b17 <alloc::alloc::handle_alloc_error>
  31363e:	e9 be 00 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
  313643:	bf 08 00 00 00       	mov    $0x8,%edi
  313648:	be 70 00 00 00       	mov    $0x70,%esi
  31364d:	e8 c5 e4 df ff       	call   111b17 <alloc::alloc::handle_alloc_error>
  313652:	e9 aa 00 00 00       	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::exec_parse:
  313657:	48 89 cb             	mov    %rcx,%rbx
  31365a:	e9 41 f8 ff ff       	jmp    312ea0 <<rexx_exec::Interp>::exec_parse+0x35e0>
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:703
  31365f:	4c 8b b4 24 a8 01 00 	mov    0x1a8(%rsp),%r14
  313666:	00 
  313667:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  31366c:	b9 01 00 00 00       	mov    $0x1,%ecx
  313671:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  313677:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  31367c:	48 89 ea             	mov    %rbp,%rdx
  31367f:	e8 dc cf e6 ff       	call   180660 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  313684:	48 8b 44 24 68       	mov    0x68(%rsp),%rax
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  313689:	48 8b 5c 24 70       	mov    0x70(%rsp),%rbx
  31368e:	48 89 44 24 58       	mov    %rax,0x58(%rsp)
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  313693:	48 8d 3c 18          	lea    (%rax,%rbx,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  313697:	4c 89 f6             	mov    %r14,%rsi
  31369a:	48 89 ea             	mov    %rbp,%rdx
  31369d:	ff 15 25 49 1c 00    	call   *0x1c4925(%rip)        # 4d7fc8 <memcpy@GLIBC_2.14>
  3136a3:	e9 39 fa ff ff       	jmp    3130e1 <<rexx_exec::Interp>::exec_parse+0x3821>
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  3136a8:	48 8d 15 61 fb 1b 00 	lea    0x1bfb61(%rip),%rdx        # 4d3210 <rexx_api::ffi::CALL_CONTEXT+0x17e80>
  3136af:	e8 4d 91 e0 ff       	call   11c801 <core::panicking::panic_bounds_check>
  3136b4:	eb 4b                	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  3136b6:	bf 01 00 00 00       	mov    $0x1,%edi
  3136bb:	4c 89 e6             	mov    %r12,%rsi
  3136be:	e8 41 e4 df ff       	call   111b04 <alloc::raw_vec::handle_error>
  3136c3:	eb 3c                	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<rexx_exec::Interp>::exec_parse:
  3136c5:	89 f3                	mov    %esi,%ebx
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:81
  3136c7:	48 8d 3d 5e 4d d5 ff 	lea    -0x2ab2a2(%rip),%rdi        # 6842c <panic_unwind::imp::CANARY+0x36cf>
  3136ce:	48 8d 15 33 92 1a 00 	lea    0x1a9233(%rip),%rdx        # 4bc908 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  3136d5:	be ab 00 00 00       	mov    $0xab,%esi
  3136da:	e8 61 72 e0 ff       	call   11a940 <core::panicking::panic_fmt>
  3136df:	eb 20                	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  3136e1:	bf 01 00 00 00       	mov    $0x1,%edi
  3136e6:	be 34 00 00 00       	mov    $0x34,%esi
  3136eb:	e8 14 e4 df ff       	call   111b04 <alloc::raw_vec::handle_error>
  3136f0:	eb 0f                	jmp    313701 <<rexx_exec::Interp>::exec_parse+0x3e41>
  3136f2:	bf 01 00 00 00       	mov    $0x1,%edi
  3136f7:	be 02 00 00 00       	mov    $0x2,%esi
  3136fc:	e8 03 e4 df ff       	call   111b04 <alloc::raw_vec::handle_error>
<rexx_exec::Interp>::apply_trigger:
  313701:	0f 0b                	ud2
  313703:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313706:	4d 85 f6             	test   %r14,%r14
  313709:	0f 84 cb 01 00 00    	je     3138da <<rexx_exec::Interp>::exec_parse+0x401a>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31370f:	4c 89 e7             	mov    %r12,%rdi
  313712:	ff 15 80 48 1c 00    	call   *0x1c4880(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313718:	e9 bd 01 00 00       	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::exec_parse:
  31371d:	eb 5c                	jmp    31377b <<rexx_exec::Interp>::exec_parse+0x3ebb>
  31371f:	eb 5a                	jmp    31377b <<rexx_exec::Interp>::exec_parse+0x3ebb>
  313721:	49 89 c5             	mov    %rax,%r13
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313724:	4c 89 f7             	mov    %r14,%rdi
  313727:	ff 15 6b 48 1c 00    	call   *0x1c486b(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  31372d:	4c 89 bc 24 28 01 00 	mov    %r15,0x128(%rsp)
  313734:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1027
  313735:	e9 a0 01 00 00       	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::exec_parse:
  31373a:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  31373d:	4d 85 e4             	test   %r12,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313740:	0f 84 5e 01 00 00    	je     3138a4 <<rexx_exec::Interp>::exec_parse+0x3fe4>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313746:	48 89 df             	mov    %rbx,%rdi
  313749:	ff 15 49 48 1c 00    	call   *0x1c4849(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  31374f:	e9 50 01 00 00       	jmp    3138a4 <<rexx_exec::Interp>::exec_parse+0x3fe4>
<rexx_exec::Interp>::exec_parse:
  313754:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313757:	48 83 7c 24 60 00    	cmpq   $0x0,0x60(%rsp)
  31375d:	0f 84 5f 01 00 00    	je     3138c2 <<rexx_exec::Interp>::exec_parse+0x4002>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313763:	48 8b 7c 24 68       	mov    0x68(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313768:	ff 15 2a 48 1c 00    	call   *0x1c482a(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  31376e:	e9 4f 01 00 00       	jmp    3138c2 <<rexx_exec::Interp>::exec_parse+0x4002>
<rexx_exec::Interp>::exec_parse:
  313773:	49 89 c5             	mov    %rax,%r13
  313776:	e9 f9 00 00 00       	jmp    313874 <<rexx_exec::Interp>::exec_parse+0x3fb4>
  31377b:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  31377e:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313781:	0f 84 97 01 00 00    	je     31391e <<rexx_exec::Interp>::exec_parse+0x405e>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313787:	4c 89 ff             	mov    %r15,%rdi
  31378a:	e9 89 01 00 00       	jmp    313918 <<rexx_exec::Interp>::exec_parse+0x4058>
<rexx_exec::Interp>::parse_strings:
  31378f:	49 89 c5             	mov    %rax,%r13
  313792:	48 89 ef             	mov    %rbp,%rdi
  313795:	e9 7e 01 00 00       	jmp    313918 <<rexx_exec::Interp>::exec_parse+0x4058>
<rexx_exec::Interp>::exec_parse:
  31379a:	49 89 c5             	mov    %rax,%r13
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31379d:	48 89 df             	mov    %rbx,%rdi
  3137a0:	e9 73 01 00 00       	jmp    313918 <<rexx_exec::Interp>::exec_parse+0x4058>
<rexx_exec::Interp>::exec_parse:
  3137a5:	49 89 c5             	mov    %rax,%r13
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3137a8:	48 89 df             	mov    %rbx,%rdi
  3137ab:	e9 3d 01 00 00       	jmp    3138ed <<rexx_exec::Interp>::exec_parse+0x402d>
<rexx_exec::Interp>::exec_parse:
  3137b0:	e9 22 01 00 00       	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  3137b5:	eb 7a                	jmp    313831 <<rexx_exec::Interp>::exec_parse+0x3f71>
  3137b7:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3137ba:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  3137c0:	0f 85 77 c8 ff ff    	jne    31003d <<rexx_exec::Interp>::exec_parse+0x77d>
  3137c6:	e9 53 01 00 00       	jmp    31391e <<rexx_exec::Interp>::exec_parse+0x405e>
<rexx_exec::Interp>::exec_parse:
  3137cb:	e9 07 01 00 00       	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  3137d0:	e9 02 01 00 00       	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  3137d5:	49 89 c5             	mov    %rax,%r13
  3137d8:	e9 97 00 00 00       	jmp    313874 <<rexx_exec::Interp>::exec_parse+0x3fb4>
  3137dd:	49 89 c5             	mov    %rax,%r13
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:86
  3137e0:	84 db                	test   %bl,%bl
  3137e2:	74 65                	je     313849 <<rexx_exec::Interp>::exec_parse+0x3f89>
  3137e4:	e9 f1 00 00 00       	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::exec_parse:
  3137e9:	49 89 c5             	mov    %rax,%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  3137ec:	e9 02 01 00 00       	jmp    3138f3 <<rexx_exec::Interp>::exec_parse+0x4033>
  3137f1:	49 89 c5             	mov    %rax,%r13
  3137f4:	48 8b bc 24 18 01 00 	mov    0x118(%rsp),%rdi
  3137fb:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3137fc:	ff 15 96 47 1c 00    	call   *0x1c4796(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313802:	e9 d3 00 00 00       	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::exec_parse:
  313807:	e9 cb 00 00 00       	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  31380c:	49 89 c5             	mov    %rax,%r13
  31380f:	48 8d 7c 24 60       	lea    0x60(%rsp),%rdi
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  313814:	e8 77 96 e5 ff       	call   16ce90 <core::ptr::drop_glue::<rexx_exec::error::Raised>>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  313819:	e9 8f 00 00 00       	jmp    3138ad <<rexx_exec::Interp>::exec_parse+0x3fed>
<rexx_exec::Interp>::apply_trigger:
  31381e:	eb 49                	jmp    313869 <<rexx_exec::Interp>::exec_parse+0x3fa9>
  313820:	eb 5f                	jmp    313881 <<rexx_exec::Interp>::exec_parse+0x3fc1>
  313822:	e9 b0 00 00 00       	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  313827:	49 89 c5             	mov    %rax,%r13
  31382a:	e9 83 00 00 00       	jmp    3138b2 <<rexx_exec::Interp>::exec_parse+0x3ff2>
  31382f:	eb 00                	jmp    313831 <<rexx_exec::Interp>::exec_parse+0x3f71>
  313831:	49 89 c5             	mov    %rax,%r13
  313834:	eb 13                	jmp    313849 <<rexx_exec::Interp>::exec_parse+0x3f89>
  313836:	e9 9c 00 00 00       	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  31383b:	49 89 c5             	mov    %rax,%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:86
  31383e:	80 7c 24 30 00       	cmpb   $0x0,0x30(%rsp)
  313843:	0f 85 91 00 00 00    	jne    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
core::ptr::drop_glue::<rexx_exec::value::Carried>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313849:	80 7c 24 60 01       	cmpb   $0x1,0x60(%rsp)
  31384e:	0f 85 86 00 00 00    	jne    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
  313854:	48 83 7c 24 68 00    	cmpq   $0x0,0x68(%rsp)
  31385a:	74 7e                	je     3138da <<rexx_exec::Interp>::exec_parse+0x401a>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  31385c:	48 8b 7c 24 70       	mov    0x70(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313861:	ff 15 31 47 1c 00    	call   *0x1c4731(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313867:	eb 71                	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::assign_targets:
  313869:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31386c:	48 83 7c 24 60 00    	cmpq   $0x0,0x60(%rsp)
  313872:	74 66                	je     3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::exec_parse:
  313874:	4c 89 ff             	mov    %r15,%rdi
  313877:	ff 15 1b 47 1c 00    	call   *0x1c471b(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  31387d:	eb 5b                	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::assign_targets:
  31387f:	eb 56                	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  313881:	49 89 c5             	mov    %rax,%r13
  313884:	48 8b bc 24 18 01 00 	mov    0x118(%rsp),%rdi
  31388b:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:792
  31388c:	49 ff ce             	dec    %r14
  31388f:	49 83 fe fe          	cmp    $0xfffffffffffffffe,%r14
  313893:	73 45                	jae    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313895:	ff 15 fd 46 1c 00    	call   *0x1c46fd(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  31389b:	eb 3d                	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::exec_parse:
  31389d:	eb 38                	jmp    3138d7 <<rexx_exec::Interp>::exec_parse+0x4017>
  31389f:	eb 67                	jmp    313908 <<rexx_exec::Interp>::exec_parse+0x4048>
  3138a1:	49 89 c5             	mov    %rax,%r13
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:100
  3138a4:	4c 89 ff             	mov    %r15,%rdi
  3138a7:	ff 15 eb 46 1c 00    	call   *0x1c46eb(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3138ad:	48 85 ed             	test   %rbp,%rbp
  3138b0:	7e 28                	jle    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::apply_trigger:
  3138b2:	4c 89 f7             	mov    %r14,%rdi
  3138b5:	ff 15 dd 46 1c 00    	call   *0x1c46dd(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3138bb:	eb 1d                	jmp    3138da <<rexx_exec::Interp>::exec_parse+0x401a>
<rexx_exec::Interp>::exec_parse:
  3138bd:	eb 49                	jmp    313908 <<rexx_exec::Interp>::exec_parse+0x4048>
  3138bf:	49 89 c5             	mov    %rax,%r13
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3138c2:	48 83 bc 24 a0 01 00 	cmpq   $0x0,0x1a0(%rsp)
  3138c9:	00 00 
  3138cb:	74 51                	je     31391e <<rexx_exec::Interp>::exec_parse+0x405e>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3138cd:	48 8b bc 24 a8 01 00 	mov    0x1a8(%rsp),%rdi
  3138d4:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3138d5:	eb 41                	jmp    313918 <<rexx_exec::Interp>::exec_parse+0x4058>
<rexx_exec::Interp>::exec_parse:
  3138d7:	49 89 c5             	mov    %rax,%r13
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3138da:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  3138e1:	00 00 
  3138e3:	7e 0e                	jle    3138f3 <<rexx_exec::Interp>::exec_parse+0x4033>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  3138e5:	48 8b bc 24 00 01 00 	mov    0x100(%rsp),%rdi
  3138ec:	00 
  3138ed:	ff 15 a5 46 1c 00    	call   *0x1c46a5(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  3138f3:	48 83 bc 24 b8 01 00 	cmpq   $0x0,0x1b8(%rsp)
  3138fa:	00 00 
core::ptr::drop_glue::<rexx_exec::parse_template::ParseStrings>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3138fc:	7e 20                	jle    31391e <<rexx_exec::Interp>::exec_parse+0x405e>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  3138fe:	48 8b bc 24 c0 01 00 	mov    0x1c0(%rsp),%rdi
  313905:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313906:	eb 10                	jmp    313918 <<rexx_exec::Interp>::exec_parse+0x4058>
<rexx_exec::Interp>::exec_parse:
  313908:	49 89 c5             	mov    %rax,%r13
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  31390b:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  313911:	7e 0b                	jle    31391e <<rexx_exec::Interp>::exec_parse+0x405e>
<rexx_exec::Interp>::exec_parse:
  313913:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
  313918:	ff 15 7a 46 1c 00    	call   *0x1c467a(%rip)        # 4d7f98 <free@GLIBC_2.2.5>
  31391e:	4c 89 ef             	mov    %r13,%rdi
  313921:	e8 1a dd 19 00       	call   4b1640 <_Unwind_Resume@plt>

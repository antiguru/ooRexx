
/tmp/codex-p9d-t88-pins/8.8f-perf-r2-rexx:     file format elf64-x86-64


Disassembly of section .text:

000000000030f930 <<rexx_exec::Interp>::exec_parse>:
<rexx_exec::Interp>::exec_parse:
  30f930:	55                   	push   %rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:474
  30f931:	41 57                	push   %r15
  30f933:	41 56                	push   %r14
  30f935:	41 55                	push   %r13
  30f937:	41 54                	push   %r12
  30f939:	53                   	push   %rbx
  30f93a:	48 81 ec 68 02 00 00 	sub    $0x268,%rsp
  30f941:	48 89 bc 24 30 01 00 	mov    %rdi,0x130(%rsp)
  30f948:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:480
  30f949:	48 8b 86 50 04 00 00 	mov    0x450(%rsi),%rax
  30f950:	48 89 84 24 c0 00 00 	mov    %rax,0xc0(%rsp)
  30f957:	00 
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:564
  30f958:	48 8b 41 18          	mov    0x18(%rcx),%rax
  30f95c:	48 bf f1 ff ff ff ff 	movabs $0x7ffffffffffffff1,%rdi
  30f963:	ff ff 7f 
  30f966:	48 01 c7             	add    %rax,%rdi
  30f969:	48 83 ff 05          	cmp    $0x5,%rdi
  30f96d:	48 89 74 24 18       	mov    %rsi,0x18(%rsp)
  30f972:	48 89 94 24 e8 00 00 	mov    %rdx,0xe8(%rsp)
  30f979:	00 
  30f97a:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
  30f97f:	77 1e                	ja     30f99f <<rexx_exec::Interp>::exec_parse+0x6f>
  30f981:	48 8d 05 bc 4a d4 ff 	lea    -0x2bb544(%rip),%rax        # 54444 <GCC_except_table5014+0x4a30>
  30f988:	48 63 3c b8          	movslq (%rax,%rdi,4),%rdi
  30f98c:	48 01 c7             	add    %rax,%rdi
  30f98f:	ff e7                	jmp    *%rdi
  30f991:	49 c7 c7 fe ff ff ff 	mov    $0xfffffffffffffffe,%r15
  30f998:	31 ed                	xor    %ebp,%ebp
  30f99a:	e9 98 0d 00 00       	jmp    310737 <<rexx_exec::Interp>::exec_parse+0xe07>
  30f99f:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
  30f9a3:	0f 85 01 01 00 00    	jne    30faaa <<rexx_exec::Interp>::exec_parse+0x17a>
  30f9a9:	bd 01 00 00 00       	mov    $0x1,%ebp
  30f9ae:	4c 8d 2d bc 72 d9 ff 	lea    -0x268d44(%rip),%r13        # a6c71 <panic_unwind::imp::CANARY+0x41ea4>
  30f9b5:	bb 05 00 00 00       	mov    $0x5,%ebx
  30f9ba:	45 31 ff             	xor    %r15d,%r15d
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30f9bd:	80 be 35 05 00 00 00 	cmpb   $0x0,0x535(%rsi)
  30f9c4:	75 23                	jne    30f9e9 <<rexx_exec::Interp>::exec_parse+0xb9>
  30f9c6:	f6 86 2e 05 00 00 01 	testb  $0x1,0x52e(%rsi)
  30f9cd:	74 1a                	je     30f9e9 <<rexx_exec::Interp>::exec_parse+0xb9>
<rexx_exec::Interp>::trace_literal:
  30f9cf:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:342
  30f9d2:	ba 01 00 00 00       	mov    $0x1,%edx
  30f9d7:	48 89 f7             	mov    %rsi,%rdi
  30f9da:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  30f9e1:	00 
  30f9e2:	31 c9                	xor    %ecx,%ecx
  30f9e4:	e8 87 1a f7 ff       	call   281470 <<rexx_exec::Interp>::trace_literal_line>
<rexx_exec::Interp>::parse_strings:
  30f9e9:	45 31 e4             	xor    %r12d,%r12d
  30f9ec:	e9 da 03 00 00       	jmp    30fdcb <<rexx_exec::Interp>::exec_parse+0x49b>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::pop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2873
  30f9f1:	48 8b 86 e0 0b 00 00 	mov    0xbe0(%rsi),%rax
  30f9f8:	48 85 c0             	test   %rax,%rax
  30f9fb:	0f 84 e8 01 00 00    	je     30fbe9 <<rexx_exec::Interp>::exec_parse+0x2b9>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2877
  30fa01:	48 ff c8             	dec    %rax
  30fa04:	48 89 86 e0 0b 00 00 	mov    %rax,0xbe0(%rsi)
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fa0b:	48 8b 8e d8 0b 00 00 	mov    0xbd8(%rsi),%rcx
<*const alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs:871
  30fa12:	48 8d 04 40          	lea    (%rax,%rax,2),%rax
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30fa16:	4c 8b 3c c1          	mov    (%rcx,%rax,8),%r15
  30fa1a:	f3 0f 6f 44 c1 08    	movdqu 0x8(%rcx,%rax,8),%xmm0
  30fa20:	f3 0f 7f 44 24 50    	movdqu %xmm0,0x50(%rsp)
<alloc::vec::Vec<u8>>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1461
  30fa26:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  30fa2b:	e9 d1 01 00 00       	jmp    30fc01 <<rexx_exec::Interp>::exec_parse+0x2d1>
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:1784
  30fa30:	48 8b 86 20 11 00 00 	mov    0x1120(%rsi),%rax
  30fa37:	48 85 c0             	test   %rax,%rax
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::pop_front:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2198
  30fa3a:	0f 84 3f 01 00 00    	je     30fb7f <<rexx_exec::Interp>::exec_parse+0x24f>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2201
  30fa40:	48 8b 96 18 11 00 00 	mov    0x1118(%rsi),%rdx
  30fa47:	49 89 f1             	mov    %rsi,%r9
<usize>::wrapping_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2652
  30fa4a:	48 8d 72 01          	lea    0x1(%rdx),%rsi
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30fa4e:	49 8b b9 08 11 00 00 	mov    0x1108(%r9),%rdi
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fa55:	49 8b 89 10 11 00 00 	mov    0x1110(%r9),%rcx
  30fa5c:	45 31 c0             	xor    %r8d,%r8d
alloc::collections::vec_deque::index::wrap_index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:3700
  30fa5f:	48 39 fe             	cmp    %rdi,%rsi
  30fa62:	4c 0f 43 c7          	cmovae %rdi,%r8
  30fa66:	49 f7 d8             	neg    %r8
  30fa69:	4a 8d 34 02          	lea    (%rdx,%r8,1),%rsi
  30fa6d:	48 ff c6             	inc    %rsi
<alloc::collections::vec_deque::VecDeque<alloc::vec::Vec<u8>>>::pop_front:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2202
  30fa70:	49 89 b1 18 11 00 00 	mov    %rsi,0x1118(%r9)
  30fa77:	4c 89 ce             	mov    %r9,%rsi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/collections/vec_deque/mod.rs:2203
  30fa7a:	48 ff c8             	dec    %rax
  30fa7d:	49 89 81 20 11 00 00 	mov    %rax,0x1120(%r9)
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30fa84:	48 8d 04 52          	lea    (%rdx,%rdx,2),%rax
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30fa88:	4c 8b 3c c1          	mov    (%rcx,%rax,8),%r15
<rexx_exec::Interp>::pull_line:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:576
  30fa8c:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fa90:	0f 84 e9 00 00 00    	je     30fb7f <<rexx_exec::Interp>::exec_parse+0x24f>
  30fa96:	48 8d 04 c1          	lea    (%rcx,%rax,8),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:577
  30fa9a:	f3 0f 6f 40 08       	movdqu 0x8(%rax),%xmm0
  30fa9f:	f3 0f 7f 44 24 50    	movdqu %xmm0,0x50(%rsp)
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30faa5:	e9 ee 00 00 00       	jmp    30fb98 <<rexx_exec::Interp>::exec_parse+0x268>
<rexx_exec::Interp>::parse_strings:
  30faaa:	4c 8d 2d c0 71 d9 ff 	lea    -0x268e40(%rip),%r13        # a6c71 <panic_unwind::imp::CANARY+0x41ea4>
  30fab1:	bb 05 00 00 00       	mov    $0x5,%ebx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:577
  30fab6:	41 f6 c0 01          	test   $0x1,%r8b
  30faba:	0f 84 f3 00 00 00    	je     30fbb3 <<rexx_exec::Interp>::exec_parse+0x283>
  30fac0:	4d 89 ce             	mov    %r9,%r14
  30fac3:	e9 3d 09 00 00       	jmp    310405 <<rexx_exec::Interp>::exec_parse+0xad5>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::pop:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2873
  30fac8:	48 8b 86 e0 0b 00 00 	mov    0xbe0(%rsi),%rax
  30facf:	48 85 c0             	test   %rax,%rax
  30fad2:	0f 84 8d 01 00 00    	je     30fc65 <<rexx_exec::Interp>::exec_parse+0x335>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2877
  30fad8:	48 ff c8             	dec    %rax
  30fadb:	48 89 86 e0 0b 00 00 	mov    %rax,0xbe0(%rsi)
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fae2:	48 8b 8e d8 0b 00 00 	mov    0xbd8(%rsi),%rcx
<*const alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/const_ptr.rs:871
  30fae9:	48 8d 14 40          	lea    (%rax,%rax,2),%rdx
core::ptr::read::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1755
  30faed:	48 8b 04 d1          	mov    (%rcx,%rdx,8),%rax
  30faf1:	f3 0f 6f 44 d1 08    	movdqu 0x8(%rcx,%rdx,8),%xmm0
  30faf7:	f3 0f 7f 44 24 50    	movdqu %xmm0,0x50(%rsp)
<alloc::vec::Vec<u8>>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1461
  30fafd:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  30fb02:	e9 75 01 00 00       	jmp    30fc7c <<rexx_exec::Interp>::exec_parse+0x34c>
<rexx_exec::Interp>::parse_strings:
  30fb07:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:630
  30fb0c:	e8 9f 69 04 00       	call   3564b0 <<rexx_exec::Interp>::linein_line>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fb11:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  30fb16:	48 8b 6c 24 50       	mov    0x50(%rsp),%rbp
  30fb1b:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  30fb20:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fb24:	0f 84 b2 0b 00 00    	je     3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
  30fb2a:	4c 8d 2d 5c 71 d9 ff 	lea    -0x268ea4(%rip),%r13        # a6c8d <panic_unwind::imp::CANARY+0x41ec0>
  30fb31:	bb 06 00 00 00       	mov    $0x6,%ebx
  30fb36:	e9 90 02 00 00       	jmp    30fdcb <<rexx_exec::Interp>::exec_parse+0x49b>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:591
  30fb3b:	8b 59 20             	mov    0x20(%rcx),%ebx
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:673
  30fb3e:	48 8b 42 08          	mov    0x8(%rdx),%rax
<alloc::vec::Vec<alloc::boxed::Box<str>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  30fb42:	48 8b 70 10          	mov    0x10(%rax),%rsi
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  30fb46:	48 39 de             	cmp    %rbx,%rsi
  30fb49:	0f 86 41 05 00 00    	jbe    310090 <<rexx_exec::Interp>::exec_parse+0x760>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::boxed::Box<str>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fb4f:	48 8b 40 08          	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  30fb53:	48 89 d9             	mov    %rbx,%rcx
  30fb56:	48 c1 e1 04          	shl    $0x4,%rcx
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  30fb5a:	4c 8b 2c 08          	mov    (%rax,%rcx,1),%r13
  30fb5e:	4c 8b 64 08 08       	mov    0x8(%rax,%rcx,1),%r12
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  30fb63:	4d 85 e4             	test   %r12,%r12
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:25
  30fb66:	0f 84 01 06 00 00    	je     31016d <<rexx_exec::Interp>::exec_parse+0x83d>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  30fb6c:	49 83 fc 04          	cmp    $0x4,%r12
  30fb70:	0f 83 d9 02 00 00    	jae    30fe4f <<rexx_exec::Interp>::exec_parse+0x51f>
<rexx_exec::Interp>::parse_strings:
  30fb76:	31 c0                	xor    %eax,%eax
  30fb78:	31 c9                	xor    %ecx,%ecx
  30fb7a:	e9 4f 05 00 00       	jmp    3100ce <<rexx_exec::Interp>::exec_parse+0x79e>
  30fb7f:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<rexx_exec::Interp>::pull_line:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/input.rs:578
  30fb84:	e8 27 69 04 00       	call   3564b0 <<rexx_exec::Interp>::linein_line>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fb89:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  30fb8e:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fb92:	0f 84 3e 03 00 00    	je     30fed6 <<rexx_exec::Interp>::exec_parse+0x5a6>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  30fb98:	48 8b 6c 24 50       	mov    0x50(%rsp),%rbp
  30fb9d:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  30fba2:	4c 8d 2d 27 2f d4 ff 	lea    -0x2bd0d9(%rip),%r13        # 52ad0 <GCC_except_table5014+0x30bc>
  30fba9:	bb 04 00 00 00       	mov    $0x4,%ebx
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:629
  30fbae:	e9 18 02 00 00       	jmp    30fdcb <<rexx_exec::Interp>::exec_parse+0x49b>
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:127
  30fbb3:	c6 84 24 d0 00 00 00 	movb   $0x0,0xd0(%rsp)
  30fbba:	00 
  30fbbb:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  30fbc0:	48 8d 94 24 d0 00 00 	lea    0xd0(%rsp),%rdx
  30fbc7:	00 
  30fbc8:	49 89 f4             	mov    %rsi,%r12
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:128
  30fbcb:	e8 30 cd f6 ff       	call   27c900 <<rexx_exec::Interp>::enter_eval_node::<u8>>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fbd0:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  30fbd5:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30fbd9:	0f 84 06 03 00 00    	je     30fee5 <<rexx_exec::Interp>::exec_parse+0x5b5>
<rexx_exec::Interp>::parse_strings:
  30fbdf:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fbe4:	e9 f3 0a 00 00       	jmp    3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
<alloc::vec::Vec<u8>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:462
  30fbe9:	48 c7 44 24 50 01 00 	movq   $0x1,0x50(%rsp)
  30fbf0:	00 00 
  30fbf2:	48 c7 44 24 58 00 00 	movq   $0x0,0x58(%rsp)
  30fbf9:	00 00 
  30fbfb:	45 31 e4             	xor    %r12d,%r12d
  30fbfe:	45 31 ff             	xor    %r15d,%r15d
<core::option::Option<alloc::vec::Vec<u8>>>::unwrap_or_default:
  30fc01:	4c 89 7c 24 48       	mov    %r15,0x48(%rsp)
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fc06:	4c 89 f8             	mov    %r15,%rax
  30fc09:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fc0c:	48 83 f8 2c          	cmp    $0x2c,%rax
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fc10:	0f 86 b1 03 00 00    	jbe    30ffc7 <<rexx_exec::Interp>::exec_parse+0x697>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fc16:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fc1b:	0f 10 05 75 94 d5 ff 	movups -0x2a6b8b(%rip),%xmm0        # 69097 <panic_unwind::imp::CANARY+0x42ca>
  30fc22:	43 0f 11 44 26 1d    	movups %xmm0,0x1d(%r14,%r12,1)
  30fc28:	0f 10 05 5b 94 d5 ff 	movups -0x2a6ba5(%rip),%xmm0        # 6908a <panic_unwind::imp::CANARY+0x42bd>
  30fc2f:	43 0f 11 44 26 10    	movups %xmm0,0x10(%r14,%r12,1)
  30fc35:	f3 0f 6f 05 3d 94 d5 	movdqu -0x2a6bc3(%rip),%xmm0        # 6907a <panic_unwind::imp::CANARY+0x42ad>
  30fc3c:	ff 
  30fc3d:	f3 43 0f 7f 04 26    	movdqu %xmm0,(%r14,%r12,1)
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fc43:	49 83 c4 2d          	add    $0x2d,%r12
  30fc47:	bb 07 00 00 00       	mov    $0x7,%ebx
  30fc4c:	4c 8d 2d 53 70 d9 ff 	lea    -0x268fad(%rip),%r13        # a6ca6 <panic_unwind::imp::CANARY+0x41ed9>
  30fc53:	4c 89 f5             	mov    %r14,%rbp
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:632
  30fc56:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fc5a:	0f 85 6b 01 00 00    	jne    30fdcb <<rexx_exec::Interp>::exec_parse+0x49b>
  30fc60:	e9 a0 07 00 00       	jmp    310405 <<rexx_exec::Interp>::exec_parse+0xad5>
<alloc::vec::Vec<u8>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:462
  30fc65:	48 c7 44 24 50 01 00 	movq   $0x1,0x50(%rsp)
  30fc6c:	00 00 
  30fc6e:	48 c7 44 24 58 00 00 	movq   $0x0,0x58(%rsp)
  30fc75:	00 00 
  30fc77:	45 31 e4             	xor    %r12d,%r12d
  30fc7a:	31 c0                	xor    %eax,%eax
<core::option::Option<alloc::vec::Vec<u8>>>::unwrap_or_default:
  30fc7c:	48 89 44 24 48       	mov    %rax,0x48(%rsp)
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fc81:	48 89 c1             	mov    %rax,%rcx
  30fc84:	4c 29 e1             	sub    %r12,%rcx
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fc87:	48 83 f9 04          	cmp    $0x4,%rcx
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fc8b:	0f 86 62 03 00 00    	jbe    30fff3 <<rexx_exec::Interp>::exec_parse+0x6c3>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fc91:	4c 8b 7c 24 50       	mov    0x50(%rsp),%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fc96:	43 c6 44 27 04 58    	movb   $0x58,0x4(%r15,%r12,1)
  30fc9c:	43 c7 04 27 4c 49 4e 	movl   $0x554e494c,(%r15,%r12,1)
  30fca3:	55 
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fca4:	49 8d 4c 24 05       	lea    0x5(%r12),%rcx
  30fca9:	48 89 4c 24 58       	mov    %rcx,0x58(%rsp)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30fcae:	48 39 c1             	cmp    %rax,%rcx
  30fcb1:	75 0f                	jne    30fcc2 <<rexx_exec::Interp>::exec_parse+0x392>
  30fcb3:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30fcb8:	e8 33 22 e0 ff       	call   111ef0 <<alloc::raw_vec::RawVec<u8>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fcbd:	4c 8b 7c 24 50       	mov    0x50(%rsp),%r15
core::ptr::write::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30fcc2:	43 c6 44 27 05 20    	movb   $0x20,0x5(%r15,%r12,1)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30fcc8:	49 83 c4 06          	add    $0x6,%r12
  30fccc:	4c 89 64 24 58       	mov    %r12,0x58(%rsp)
  30fcd1:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30fcd6:	48 8b 80 70 04 00 00 	mov    0x470(%rax),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  30fcdd:	48 85 c0             	test   %rax,%rax
  30fce0:	0f 84 c4 02 00 00    	je     30ffaa <<rexx_exec::Interp>::exec_parse+0x67a>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30fce6:	0f b6 80 d6 01 00 00 	movzbl 0x1d6(%rax),%eax
<rexx_exec::activation::CallType>::token:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/activation.rs:472
  30fced:	48 8d 0d bc 9d d9 ff 	lea    -0x266244(%rip),%rcx        # a9ab0 <core::unicode::unicode_data::white_space::WHITESPACE_MAP+0x16c2>
  30fcf4:	0f b6 1c 08          	movzbl (%rax,%rcx,1),%ebx
  30fcf8:	4c 8d 35 b9 9d d9 ff 	lea    -0x266247(%rip),%r14        # a9ab8 <core::unicode::unicode_data::white_space::WHITESPACE_MAP+0x16ca>
  30fcff:	4d 63 2c 86          	movslq (%r14,%rax,4),%r13
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30fd03:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fd08:	48 89 e8             	mov    %rbp,%rax
  30fd0b:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fd0e:	48 39 d8             	cmp    %rbx,%rax
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fd11:	0f 82 08 03 00 00    	jb     31001f <<rexx_exec::Interp>::exec_parse+0x6ef>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:603
  30fd17:	4d 01 ee             	add    %r13,%r14
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30fd1a:	4b 8d 3c 27          	lea    (%r15,%r12,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fd1e:	4c 89 f6             	mov    %r14,%rsi
  30fd21:	48 89 da             	mov    %rbx,%rdx
  30fd24:	ff 15 6e 84 1c 00    	call   *0x1c846e(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fd2a:	49 01 dc             	add    %rbx,%r12
  30fd2d:	4c 89 64 24 58       	mov    %r12,0x58(%rsp)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30fd32:	49 39 ec             	cmp    %rbp,%r12
  30fd35:	75 0f                	jne    30fd46 <<rexx_exec::Interp>::exec_parse+0x416>
  30fd37:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30fd3c:	e8 af 21 e0 ff       	call   111ef0 <<alloc::raw_vec::RawVec<u8>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fd41:	4c 8b 7c 24 50       	mov    0x50(%rsp),%r15
core::ptr::write::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30fd46:	43 c6 04 27 20       	movb   $0x20,(%r15,%r12,1)
<alloc::vec::Vec<u8>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30fd4b:	49 ff c4             	inc    %r12
  30fd4e:	4c 89 64 24 58       	mov    %r12,0x58(%rsp)
  30fd53:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:610
  30fd58:	48 8b 87 70 04 00 00 	mov    0x470(%rdi),%rax
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  30fd5f:	48 85 c0             	test   %rax,%rax
  30fd62:	0f 84 42 02 00 00    	je     30ffaa <<rexx_exec::Interp>::exec_parse+0x67a>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:610
  30fd68:	48 8b b0 f0 00 00 00 	mov    0xf0(%rax),%rsi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:611
  30fd6f:	e8 bc c2 ea ff       	call   1bc030 <<rexx_exec::Interp>::program_display_name>
  30fd74:	49 89 c5             	mov    %rax,%r13
  30fd77:	48 89 d3             	mov    %rdx,%rbx
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  30fd7a:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
<usize>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  30fd7f:	4c 89 f8             	mov    %r15,%rax
  30fd82:	4c 29 e0             	sub    %r12,%rax
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  30fd85:	48 39 c2             	cmp    %rax,%rdx
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  30fd88:	0f 87 c0 02 00 00    	ja     31004e <<rexx_exec::Interp>::exec_parse+0x71e>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2981
  30fd8e:	48 85 db             	test   %rbx,%rbx
  30fd91:	74 17                	je     30fdaa <<rexx_exec::Interp>::exec_parse+0x47a>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30fd93:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  30fd98:	4b 8d 3c 26          	lea    (%r14,%r12,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  30fd9c:	4c 89 ee             	mov    %r13,%rsi
  30fd9f:	48 89 da             	mov    %rbx,%rdx
  30fda2:	ff 15 f0 83 1c 00    	call   *0x1c83f0(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
  30fda8:	eb 05                	jmp    30fdaf <<rexx_exec::Interp>::exec_parse+0x47f>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:612
  30fdaa:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  30fdaf:	49 01 dc             	add    %rbx,%r12
  30fdb2:	bb 06 00 00 00       	mov    $0x6,%ebx
  30fdb7:	4c 8d 2d da 6e d9 ff 	lea    -0x269126(%rip),%r13        # a6c98 <panic_unwind::imp::CANARY+0x41ecb>
  30fdbe:	4c 89 f5             	mov    %r14,%rbp
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:632
  30fdc1:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  30fdc5:	0f 84 3a 06 00 00    	je     310405 <<rexx_exec::Interp>::exec_parse+0xad5>
  30fdcb:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  30fdd0:	80 b8 35 05 00 00 00 	cmpb   $0x0,0x535(%rax)
  30fdd7:	75 67                	jne    30fe40 <<rexx_exec::Interp>::exec_parse+0x510>
  30fdd9:	f6 80 2d 05 00 00 01 	testb  $0x1,0x52d(%rax)
  30fde0:	74 5e                	je     30fe40 <<rexx_exec::Interp>::exec_parse+0x510>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30fde2:	48 8d b8 d8 10 00 00 	lea    0x10d8(%rax),%rdi
  30fde9:	4c 8b b0 e8 10 00 00 	mov    0x10e8(%rax),%r14
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:298
  30fdf0:	4c 89 64 24 10       	mov    %r12,0x10(%rsp)
  30fdf5:	48 89 6c 24 08       	mov    %rbp,0x8(%rsp)
  30fdfa:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  30fe01:	48 8d 35 e5 ad d5 ff 	lea    -0x2a521b(%rip),%rsi        # 6abed <panic_unwind::imp::CANARY+0x5e20>
  30fe08:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  30fe0f:	00 
  30fe10:	b9 01 00 00 00       	mov    $0x1,%ecx
  30fe15:	4d 89 e8             	mov    %r13,%r8
  30fe18:	49 89 d9             	mov    %rbx,%r9
  30fe1b:	e8 40 0a f7 ff       	call   280860 <rexx_exec::trace::format::push_tagged>
<[u8] as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:21
  30fe20:	83 fb 04             	cmp    $0x4,%ebx
  30fe23:	75 0e                	jne    30fe33 <<rexx_exec::Interp>::exec_parse+0x503>
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  30fe25:	41 81 7d 00 57 48 45 	cmpl   $0x4e454857,0x0(%r13)
  30fe2c:	4e 
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:307
  30fe2d:	0f 84 45 02 00 00    	je     310078 <<rexx_exec::Interp>::exec_parse+0x748>
  30fe33:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:310
  30fe38:	4c 89 f6             	mov    %r14,%rsi
  30fe3b:	e8 80 e3 e8 ff       	call   19e1c0 <<rexx_exec::Interp>::route_trace_line>
<core::result::Result<rexx_exec::parse_template::ParseStrings, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30fe40:	49 83 ff fd          	cmp    $0xfffffffffffffffd,%r15
  30fe44:	0f 84 92 08 00 00    	je     3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
  30fe4a:	e9 e8 08 00 00       	jmp    310737 <<rexx_exec::Interp>::exec_parse+0xe07>
<rexx_exec::Interp>::exec_parse:
  30fe4f:	4c 89 e0             	mov    %r12,%rax
  30fe52:	48 83 e0 fc          	and    $0xfffffffffffffffc,%rax
  30fe56:	66 0f ef c0          	pxor   %xmm0,%xmm0
  30fe5a:	31 c9                	xor    %ecx,%ecx
  30fe5c:	66 0f 6f 0d 2c 01 d4 	movdqa -0x2bfed4(%rip),%xmm1        # 4ff90 <GCC_except_table5014+0x57c>
  30fe63:	ff 
  30fe64:	66 0f 6f 1d a4 05 d4 	movdqa -0x2bfa5c(%rip),%xmm3        # 50410 <GCC_except_table5014+0x9fc>
  30fe6b:	ff 
  30fe6c:	66 0f ef d2          	pxor   %xmm2,%xmm2
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:274
  30fe70:	41 0f b7 54 0d 00    	movzwl 0x0(%r13,%rcx,1),%edx
  30fe76:	66 0f 6e e2          	movd   %edx,%xmm4
  30fe7a:	41 0f b7 54 0d 02    	movzwl 0x2(%r13,%rcx,1),%edx
  30fe80:	66 0f 6e ea          	movd   %edx,%xmm5
<core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize::<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/filter.rs:138
  30fe84:	66 0f 74 e1          	pcmpeqb %xmm1,%xmm4
  30fe88:	66 0f 60 e4          	punpcklbw %xmm4,%xmm4
  30fe8c:	66 0f 61 e4          	punpcklwd %xmm4,%xmm4
  30fe90:	66 0f 70 e4 d4       	pshufd $0xd4,%xmm4,%xmm4
  30fe95:	66 0f db e3          	pand   %xmm3,%xmm4
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  30fe99:	66 0f d4 d4          	paddq  %xmm4,%xmm2
<core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize::<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/filter.rs:138
  30fe9d:	66 0f 74 e9          	pcmpeqb %xmm1,%xmm5
  30fea1:	66 0f 60 ed          	punpcklbw %xmm5,%xmm5
  30fea5:	66 0f 61 ed          	punpcklwd %xmm5,%xmm5
  30fea9:	66 0f 70 e5 d4       	pshufd $0xd4,%xmm5,%xmm4
  30feae:	66 0f db e3          	pand   %xmm3,%xmm4
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  30feb2:	66 0f d4 c4          	paddq  %xmm4,%xmm0
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
  30feb6:	48 83 c1 04          	add    $0x4,%rcx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  30feba:	48 39 c8             	cmp    %rcx,%rax
  30febd:	75 b1                	jne    30fe70 <<rexx_exec::Interp>::exec_parse+0x540>
  30febf:	66 0f d4 c2          	paddq  %xmm2,%xmm0
  30fec3:	66 0f 70 c8 ee       	pshufd $0xee,%xmm0,%xmm1
  30fec8:	66 0f d4 c8          	paddq  %xmm0,%xmm1
  30fecc:	66 48 0f 7e c9       	movq   %xmm1,%rcx
  30fed1:	e9 09 02 00 00       	jmp    3100df <<rexx_exec::Interp>::exec_parse+0x7af>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  30fed6:	48 8b 6c 24 50       	mov    0x50(%rsp),%rbp
  30fedb:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  30fee0:	e9 f7 07 00 00       	jmp    3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
<rexx_exec::Interp>::parse_strings:
  30fee5:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
  30feea:	4c 8d 78 18          	lea    0x18(%rax),%r15
  30feee:	48 8d bc 24 58 01 00 	lea    0x158(%rsp),%rdi
  30fef5:	00 
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:129
  30fef6:	4c 89 e6             	mov    %r12,%rsi
  30fef9:	48 8b 94 24 e8 00 00 	mov    0xe8(%rsp),%rdx
  30ff00:	00 
  30ff01:	4c 89 f9             	mov    %r15,%rcx
  30ff04:	e8 57 cc f6 ff       	call   27cb60 <<rexx_exec::Interp>::eval_node>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:130
  30ff09:	49 ff 8c 24 c8 04 00 	decq   0x4c8(%r12)
  30ff10:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:131
  30ff11:	48 8b ac 24 58 01 00 	mov    0x158(%rsp),%rbp
  30ff18:	00 
  30ff19:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30ff1d:	74 0d                	je     30ff2c <<rexx_exec::Interp>::exec_parse+0x5fc>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  30ff1f:	4c 8b a4 24 60 01 00 	mov    0x160(%rsp),%r12
  30ff26:	00 
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30ff27:	e9 b0 07 00 00       	jmp    3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
<rexx_exec::Interp>::parse_strings:
  30ff2c:	4c 89 e1             	mov    %r12,%rcx
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  30ff2f:	41 f6 84 24 36 05 00 	testb  $0x4,0x536(%r12)
  30ff36:	00 04 
<rexx_exec::Interp>::eval_in_parse:
  30ff38:	4c 8b b4 24 60 01 00 	mov    0x160(%rsp),%r14
  30ff3f:	00 
<rexx_exec::Interp>::trace_intermediate:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:161
  30ff40:	74 2d                	je     30ff6f <<rexx_exec::Interp>::exec_parse+0x63f>
  30ff42:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:164
  30ff47:	4c 89 e6             	mov    %r12,%rsi
  30ff4a:	48 8b 94 24 e8 00 00 	mov    0xe8(%rsp),%rdx
  30ff51:	00 
  30ff52:	4c 89 f9             	mov    %r15,%rcx
  30ff55:	4d 89 f0             	mov    %r14,%r8
  30ff58:	e8 23 f7 f6 ff       	call   27f680 <<rexx_exec::Interp>::trace_intermediate_event>
  30ff5d:	4c 89 e1             	mov    %r12,%rcx
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  30ff60:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  30ff65:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  30ff69:	0f 85 70 fc ff ff    	jne    30fbdf <<rexx_exec::Interp>::exec_parse+0x2af>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  30ff6f:	4c 8b b9 f0 0a 00 00 	mov    0xaf0(%rcx),%r15
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  30ff76:	4c 3b b9 e0 0a 00 00 	cmp    0xae0(%rcx),%r15
  30ff7d:	74 1a                	je     30ff99 <<rexx_exec::Interp>::exec_parse+0x669>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  30ff7f:	48 8b 81 e8 0a 00 00 	mov    0xae8(%rcx),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  30ff86:	4e 89 34 f8          	mov    %r14,(%rax,%r15,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  30ff8a:	49 ff c7             	inc    %r15
  30ff8d:	4c 89 b9 f0 0a 00 00 	mov    %r15,0xaf0(%rcx)
  30ff94:	e9 6c 04 00 00       	jmp    310405 <<rexx_exec::Interp>::exec_parse+0xad5>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  30ff99:	48 8d b9 e0 0a 00 00 	lea    0xae0(%rcx),%rdi
  30ffa0:	e8 db bf e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  30ffa5:	4c 89 e1             	mov    %r12,%rcx
  30ffa8:	eb d5                	jmp    30ff7f <<rexx_exec::Interp>::exec_parse+0x64f>
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  30ffaa:	48 8d 3d 19 9f d5 ff 	lea    -0x2a60e7(%rip),%rdi        # 69eca <panic_unwind::imp::CANARY+0x50fd>
  30ffb1:	48 8d 15 c0 03 1b 00 	lea    0x1b03c0(%rip),%rdx        # 4c0378 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  30ffb8:	be 11 00 00 00       	mov    $0x11,%esi
  30ffbd:	e8 be f0 e0 ff       	call   11f080 <core::option::expect_failed>
  30ffc2:	e9 16 39 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::Interp>::parse_strings:
  30ffc7:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30ffcc:	ba 2d 00 00 00       	mov    $0x2d,%edx
  30ffd1:	b9 01 00 00 00       	mov    $0x1,%ecx
  30ffd6:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  30ffdc:	4c 89 e6             	mov    %r12,%rsi
  30ffdf:	e8 ec 06 e7 ff       	call   1806d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:617
  30ffe4:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  30ffe9:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  30ffee:	e9 23 fc ff ff       	jmp    30fc16 <<rexx_exec::Interp>::exec_parse+0x2e6>
<rexx_exec::Interp>::parse_strings:
  30fff3:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  30fff8:	ba 05 00 00 00       	mov    $0x5,%edx
  30fffd:	b9 01 00 00 00       	mov    $0x1,%ecx
  310002:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  310008:	4c 89 e6             	mov    %r12,%rsi
  31000b:	e8 c0 06 e7 ff       	call   1806d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  310010:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  310015:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  31001a:	e9 72 fc ff ff       	jmp    30fc91 <<rexx_exec::Interp>::exec_parse+0x361>
<rexx_exec::Interp>::parse_strings:
  31001f:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  310024:	b9 01 00 00 00       	mov    $0x1,%ecx
  310029:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  31002f:	4c 89 e6             	mov    %r12,%rsi
  310032:	48 89 da             	mov    %rbx,%rdx
  310035:	e8 96 06 e7 ff       	call   1806d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  31003a:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
<alloc::raw_vec::RawVecInner>::capacity:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:616
  31003f:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310044:	4c 8b 7c 24 50       	mov    0x50(%rsp),%r15
  310049:	e9 c9 fc ff ff       	jmp    30fd17 <<rexx_exec::Interp>::exec_parse+0x3e7>
<rexx_exec::Interp>::parse_strings:
  31004e:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  310053:	b9 01 00 00 00       	mov    $0x1,%ecx
  310058:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  31005e:	4c 89 e6             	mov    %r12,%rsi
  310061:	48 89 da             	mov    %rbx,%rdx
  310064:	e8 67 06 e7 ff       	call   1806d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:612
  310069:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  31006e:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  310073:	e9 1b fd ff ff       	jmp    30fd93 <<rexx_exec::Interp>::exec_parse+0x463>
<rexx_exec::Interp>::parse_strings:
  310078:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:308
  31007d:	4c 89 f6             	mov    %r14,%rsi
  310080:	48 89 ea             	mov    %rbp,%rdx
  310083:	4c 89 e1             	mov    %r12,%rcx
  310086:	e8 45 97 00 00       	call   3197d0 <<rexx_exec::Interp>::route_when_line>
  31008b:	e9 b0 fd ff ff       	jmp    30fe40 <<rexx_exec::Interp>::exec_parse+0x510>
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  310090:	48 8d 15 49 33 1c 00 	lea    0x1c3349(%rip),%rdx        # 4d33e0 <rexx_api::ffi::CALL_CONTEXT+0x17e80>
  310097:	48 89 df             	mov    %rbx,%rdi
  31009a:	e8 d2 c7 e0 ff       	call   11c871 <core::panicking::panic_bounds_check>
<rexx_exec::Interp>::parse_strings:
  31009f:	eb 14                	jmp    3100b5 <<rexx_exec::Interp>::exec_parse+0x785>
  3100a1:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3100a4:	4d 85 ff             	test   %r15,%r15
  3100a7:	0f 84 41 3a 00 00    	je     313aee <<rexx_exec::Interp>::exec_parse+0x41be>
<rexx_exec::Interp>::parse_strings:
  3100ad:	48 89 ef             	mov    %rbp,%rdi
  3100b0:	e9 33 3a 00 00       	jmp    313ae8 <<rexx_exec::Interp>::exec_parse+0x41b8>
  3100b5:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3100b8:	48 83 7c 24 48 00    	cmpq   $0x0,0x48(%rsp)
  3100be:	0f 84 2a 3a 00 00    	je     313aee <<rexx_exec::Interp>::exec_parse+0x41be>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3100c4:	48 8b 7c 24 50       	mov    0x50(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3100c9:	e9 1a 3a 00 00       	jmp    313ae8 <<rexx_exec::Interp>::exec_parse+0x41b8>
rexx_exec::run::shape_of::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:3777
  3100ce:	31 d2                	xor    %edx,%edx
  3100d0:	41 80 7c 05 00 2e    	cmpb   $0x2e,0x0(%r13,%rax,1)
  3100d6:	0f 94 c2             	sete   %dl
<usize as core::iter::traits::accum::Sum>::sum::<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/accum.rs:55
  3100d9:	48 01 d1             	add    %rdx,%rcx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  3100dc:	48 ff c0             	inc    %rax
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::fold::<usize, core::iter::adapters::map::map_fold<&u8, usize, usize, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}, <usize as core::iter::traits::accum::Sum>::sum<core::iter::adapters::map::Map<core::slice::iter::Iter<u8>, <core::iter::adapters::filter::Filter<_, _> as core::iter::traits::iterator::Iterator>::count::to_usize<&u8, rexx_exec::run::shape_of::{closure#0}>::{closure#0}>>::{closure#0}>::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:279
  3100df:	49 39 c4             	cmp    %rax,%r12
  3100e2:	75 ea                	jne    3100ce <<rexx_exec::Interp>::exec_parse+0x79e>
rexx_exec::run::shape_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:3778
  3100e4:	48 85 c9             	test   %rcx,%rcx
  3100e7:	0f 84 80 00 00 00    	je     31016d <<rexx_exec::Interp>::exec_parse+0x83d>
  3100ed:	48 83 f9 01          	cmp    $0x1,%rcx
  3100f1:	0f 85 ed 00 00 00    	jne    3101e4 <<rexx_exec::Interp>::exec_parse+0x8b4>
<u8 as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2105
  3100f7:	43 80 7c 25 ff 2e    	cmpb   $0x2e,-0x1(%r13,%r12,1)
rexx_exec::run::shape_of:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run.rs:3777
  3100fd:	0f 85 e1 00 00 00    	jne    3101e4 <<rexx_exec::Interp>::exec_parse+0x8b4>
<rexx_exec::Interp>::read_parse_var:
  310103:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::Interp>::read_stem:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:186
  310108:	4c 89 ff             	mov    %r15,%rdi
  31010b:	4c 89 ee             	mov    %r13,%rsi
  31010e:	4c 89 e2             	mov    %r12,%rdx
  310111:	31 c9                	xor    %ecx,%ecx
  310113:	e8 98 f3 f0 ff       	call   21f4b0 <<rexx_exec::Interp>::read_stem_at>
  310118:	49 89 c6             	mov    %rax,%r14
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  31011b:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  310122:	04 
  310123:	0f 84 a7 02 00 00    	je     3103d0 <<rexx_exec::Interp>::exec_parse+0xaa0>
<rexx_exec::Interp>::read_parse_var:
  310129:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:689
  31012e:	4c 89 fe             	mov    %r15,%rsi
  310131:	4c 89 f2             	mov    %r14,%rdx
  310134:	e8 87 f8 e9 ff       	call   1af9c0 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  310139:	48 8b 4c 24 48       	mov    0x48(%rsp),%rcx
  31013e:	48 8b 44 24 50       	mov    0x50(%rsp),%rax
  310143:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
  310148:	48 8b 6c 24 58       	mov    0x58(%rsp),%rbp
  31014d:	48 85 ed             	test   %rbp,%rbp
  310150:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  310155:	0f 89 29 30 00 00    	jns    313184 <<rexx_exec::Interp>::exec_parse+0x3854>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  31015b:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  31015d:	48 89 df             	mov    %rbx,%rdi
  310160:	48 89 ee             	mov    %rbp,%rsi
  310163:	e8 0c 1a e0 ff       	call   111b74 <alloc::raw_vec::handle_error>
  310168:	e9 70 37 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::Interp>::read_parse_var:
  31016d:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::Interp>::read:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:355
  310172:	4c 89 ff             	mov    %r15,%rdi
  310175:	48 8b b4 24 e8 00 00 	mov    0xe8(%rsp),%rsi
  31017c:	00 
  31017d:	89 da                	mov    %ebx,%edx
  31017f:	31 c9                	xor    %ecx,%ecx
  310181:	e8 8a 22 f7 ff       	call   282410 <<rexx_exec::Interp>::read_at>
  310186:	49 89 c6             	mov    %rax,%r14
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:315
  310189:	f6 c2 01             	test   $0x1,%dl
  31018c:	0f 85 1b 35 00 00    	jne    3136ad <<rexx_exec::Interp>::exec_parse+0x3d7d>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  310192:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  310199:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:678
  31019a:	0f 84 30 02 00 00    	je     3103d0 <<rexx_exec::Interp>::exec_parse+0xaa0>
  3101a0:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:679
  3101a5:	4c 89 fe             	mov    %r15,%rsi
  3101a8:	4c 89 f2             	mov    %r14,%rdx
  3101ab:	e8 10 f8 e9 ff       	call   1af9c0 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  3101b0:	48 8b 4c 24 48       	mov    0x48(%rsp),%rcx
  3101b5:	48 8b 44 24 50       	mov    0x50(%rsp),%rax
  3101ba:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
  3101bf:	48 8b 6c 24 58       	mov    0x58(%rsp),%rbp
  3101c4:	48 85 ed             	test   %rbp,%rbp
  3101c7:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  3101cc:	0f 89 3d 01 00 00    	jns    31030f <<rexx_exec::Interp>::exec_parse+0x9df>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  3101d2:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  3101d4:	48 89 df             	mov    %rbx,%rdi
  3101d7:	48 89 ee             	mov    %rbp,%rsi
  3101da:	e8 95 19 e0 ff       	call   111b74 <alloc::raw_vec::handle_error>
  3101df:	e9 f9 36 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::Interp>::read_parse_var:
  3101e4:	48 8b 84 24 e8 00 00 	mov    0xe8(%rsp),%rax
  3101eb:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:695
  3101ec:	48 8b 40 20          	mov    0x20(%rax),%rax
<core::option::Option<&rexx_exec::plan::Plan> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  3101f0:	48 85 c0             	test   %rax,%rax
  3101f3:	74 26                	je     31021b <<rexx_exec::Interp>::exec_parse+0x8eb>
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::plan::CompoundName>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  3101f5:	48 39 98 b0 00 00 00 	cmp    %rbx,0xb0(%rax)
  3101fc:	76 1d                	jbe    31021b <<rexx_exec::Interp>::exec_parse+0x8eb>
<rexx_exec::plan::Plan>::compound:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/plan.rs:424
  3101fe:	48 8b 80 a8 00 00 00 	mov    0xa8(%rax),%rax
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_exec::plan::CompoundName>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310205:	48 8d 0c 5b          	lea    (%rbx,%rbx,2),%rcx
  310209:	48 c1 e1 04          	shl    $0x4,%rcx
<core::option::Option<rexx_exec::plan::CompoundName>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  31020d:	48 8b 14 08          	mov    (%rax,%rcx,1),%rdx
  310211:	48 83 fa 02          	cmp    $0x2,%rdx
<rexx_exec::plan::Plan>::compound:
  310215:	0f 85 65 2e 00 00    	jne    313080 <<rexx_exec::Interp>::exec_parse+0x3750>
<rexx_exec::Code>::stem:
  31021b:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1288
  310220:	4c 89 ee             	mov    %r13,%rsi
  310223:	4c 89 e2             	mov    %r12,%rdx
  310226:	e8 25 64 15 00       	call   466650 <rexx_parse::ast::compound_parts>
  31022b:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
  310230:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
  310235:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31023a:	48 83 7c 24 58 00    	cmpq   $0x0,0x58(%rsp)
  310240:	74 0b                	je     31024d <<rexx_exec::Interp>::exec_parse+0x91d>
<rexx_exec::Code>::stem:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1288
  310242:	48 8b 7c 24 60       	mov    0x60(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  310247:	ff 15 1b 7f 1c 00    	call   *0x1c7f1b(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  31024d:	4c 89 74 24 28       	mov    %r14,0x28(%rsp)
  310252:	48 c7 84 24 f8 00 00 	movq   $0x0,0xf8(%rsp)
  310259:	00 00 00 00 00 
  31025e:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
core::mem::replace::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  310263:	f3 41 0f 6f 86 b8 0b 	movdqu 0xbb8(%r14),%xmm0
  31026a:	00 00 
  31026c:	66 0f 7f 84 24 70 01 	movdqa %xmm0,0x170(%rsp)
  310273:	00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  310275:	49 c7 86 b8 0b 00 00 	movq   $0x0,0xbb8(%r14)
  31027c:	00 00 00 00 
  310280:	49 c7 86 c0 0b 00 00 	movq   $0x1,0xbc0(%r14)
  310287:	01 00 00 00 
  31028b:	49 c7 86 c8 0b 00 00 	movq   $0x0,0xbc8(%r14)
  310292:	00 00 00 00 
<alloc::vec::Vec<u8>>::clear:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3082
  310296:	48 c7 84 24 80 01 00 	movq   $0x0,0x180(%rsp)
  31029d:	00 00 00 00 00 
  3102a2:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  3102a7:	4c 8d 84 24 70 01 00 	lea    0x170(%rsp),%r8
  3102ae:	00 
<rexx_exec::Interp>::tail_key_into:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:77
  3102af:	4c 89 f6             	mov    %r14,%rsi
  3102b2:	48 8b 94 24 e8 00 00 	mov    0xe8(%rsp),%rdx
  3102b9:	00 
  3102ba:	89 d9                	mov    %ebx,%ecx
  3102bc:	e8 af 10 fb ff       	call   2c1370 <<rexx_exec::Interp>::append_tail_key>
<rexx_exec::Interp>::read_parse_var:
  3102c1:	4d 8d be b8 0b 00 00 	lea    0xbb8(%r14),%r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:697
  3102c8:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  3102cd:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  3102d1:	74 6f                	je     310342 <<rexx_exec::Interp>::exec_parse+0xa12>
  3102d3:	4c 8b 64 24 50       	mov    0x50(%rsp),%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3102d8:	49 83 3f 00          	cmpq   $0x0,(%r15)
  3102dc:	74 12                	je     3102f0 <<rexx_exec::Interp>::exec_parse+0x9c0>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
  3102de:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3102e3:	48 8b b8 c0 0b 00 00 	mov    0xbc0(%rax),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3102ea:	ff 15 78 7e 1c 00    	call   *0x1c7e78(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::give_key_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:64
  3102f0:	48 8b 84 24 80 01 00 	mov    0x180(%rsp),%rax
  3102f7:	00 
  3102f8:	49 89 47 10          	mov    %rax,0x10(%r15)
  3102fc:	66 0f 6f 84 24 70 01 	movdqa 0x170(%rsp),%xmm0
  310303:	00 00 
  310305:	f3 41 0f 7f 07       	movdqu %xmm0,(%r15)
<rexx_exec::Interp>::read_parse_var:
  31030a:	e9 cd 03 00 00       	jmp    3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  31030f:	74 5f                	je     310370 <<rexx_exec::Interp>::exec_parse+0xa40>
  310311:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  310316:	be 01 00 00 00       	mov    $0x1,%esi
  31031b:	48 89 ef             	mov    %rbp,%rdi
  31031e:	e8 cd 74 e1 ff       	call   1277f0 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  310323:	48 85 c0             	test   %rax,%rax
  310326:	0f 84 a8 fe ff ff    	je     3101d4 <<rexx_exec::Interp>::exec_parse+0x8a4>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  31032c:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  31032f:	48 89 c7             	mov    %rax,%rdi
  310332:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
  310337:	48 89 ea             	mov    %rbp,%rdx
  31033a:	ff 15 58 7e 1c 00    	call   *0x1c7e58(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  310340:	eb 34                	jmp    310376 <<rexx_exec::Interp>::exec_parse+0xa46>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  310342:	41 f6 86 36 05 00 00 	testb  $0x4,0x536(%r14)
  310349:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:701
  31034a:	0f 84 c8 2f 00 00    	je     313318 <<rexx_exec::Interp>::exec_parse+0x39e8>
  310350:	48 8b 5c 24 28       	mov    0x28(%rsp),%rbx
  310355:	48 85 db             	test   %rbx,%rbx
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  310358:	0f 89 db 2d 00 00    	jns    313139 <<rexx_exec::Interp>::exec_parse+0x3809>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  31035e:	31 ed                	xor    %ebp,%ebp
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  310360:	48 89 ef             	mov    %rbp,%rdi
  310363:	48 89 de             	mov    %rbx,%rsi
  310366:	e8 09 18 e0 ff       	call   111b74 <alloc::raw_vec::handle_error>
  31036b:	e9 6d 35 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::Interp>::read_parse_var:
  310370:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  310376:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  31037c:	7e 0b                	jle    310389 <<rexx_exec::Interp>::exec_parse+0xa59>
  31037e:	48 8b 7c 24 30       	mov    0x30(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  310383:	ff 15 df 7d 1c 00    	call   *0x1c7ddf(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  310389:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31038e:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  310395:	75 26                	jne    3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
  310397:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  31039e:	74 1d                	je     3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  3103a0:	4c 89 34 24          	mov    %r14,(%rsp)
  3103a4:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  3103ab:	00 
  3103ac:	4c 89 ea             	mov    %r13,%rdx
  3103af:	4c 89 e1             	mov    %r12,%rcx
  3103b2:	4d 89 f8             	mov    %r15,%r8
  3103b5:	49 89 e9             	mov    %rbp,%r9
  3103b8:	e8 a3 0e f7 ff       	call   281260 <<rexx_exec::Interp>::trace_variable_line>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  3103bd:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3103c0:	74 09                	je     3103cb <<rexx_exec::Interp>::exec_parse+0xa9b>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3103c2:	4c 89 ff             	mov    %r15,%rdi
  3103c5:	ff 15 9d 7d 1c 00    	call   *0x1c7d9d(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::parse_strings:
  3103cb:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  3103d0:	49 8b 9f f0 0a 00 00 	mov    0xaf0(%r15),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3103d7:	49 3b 9f e0 0a 00 00 	cmp    0xae0(%r15),%rbx
  3103de:	0f 84 e4 2c 00 00    	je     3130c8 <<rexx_exec::Interp>::exec_parse+0x3798>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3103e4:	49 8b 87 e8 0a 00 00 	mov    0xae8(%r15),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3103eb:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3103ef:	48 ff c3             	inc    %rbx
  3103f2:	49 89 9f f0 0a 00 00 	mov    %rbx,0xaf0(%r15)
  3103f9:	4c 8d 2d a3 68 d9 ff 	lea    -0x26975d(%rip),%r13        # a6ca3 <panic_unwind::imp::CANARY+0x41ed6>
  310400:	bb 03 00 00 00       	mov    $0x3,%ebx
<rexx_exec::Interp>::parse_strings:
  310405:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31040a:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  310411:	00 00 
  310413:	0f 85 02 01 00 00    	jne    31051b <<rexx_exec::Interp>::exec_parse+0xbeb>
  310419:	41 f6 84 24 2d 05 00 	testb  $0x1,0x52d(%r12)
  310420:	00 01 
  310422:	0f 84 f3 00 00 00    	je     31051b <<rexx_exec::Interp>::exec_parse+0xbeb>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310428:	4c 89 f0             	mov    %r14,%rax
  31042b:	48 83 e0 03          	and    $0x3,%rax
  31042f:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
  310434:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<rexx_exec::Interp>::traced_into_parse_buffer:
  310439:	4c 89 e6             	mov    %r12,%rsi
  31043c:	4c 89 f2             	mov    %r14,%rdx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  31043f:	0f 84 31 2c 00 00    	je     313076 <<rexx_exec::Interp>::exec_parse+0x3746>
<rexx_exec::Interp>::traced_into_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:354
  310445:	e8 56 64 04 00       	call   3568a0 <<rexx_exec::Interp>::rendered_into_parse_buffer>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31044a:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
  31044f:	48 8b 6c 24 50       	mov    0x50(%rsp),%rbp
  310454:	4c 8b 64 24 58       	mov    0x58(%rsp),%r12
  310459:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  31045e:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
  310462:	0f 84 74 02 00 00    	je     3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
<rexx_exec::Interp>::parse_strings:
  310468:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31046d:	41 80 bf 35 05 00 00 	cmpb   $0x0,0x535(%r15)
  310474:	00 
  310475:	75 60                	jne    3104d7 <<rexx_exec::Interp>::exec_parse+0xba7>
  310477:	41 f6 87 2d 05 00 00 	testb  $0x1,0x52d(%r15)
  31047e:	01 
  31047f:	74 56                	je     3104d7 <<rexx_exec::Interp>::exec_parse+0xba7>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  310481:	49 8d bf d8 10 00 00 	lea    0x10d8(%r15),%rdi
  310488:	49 8b 87 e8 10 00 00 	mov    0x10e8(%r15),%rax
  31048f:	48 89 84 24 f8 00 00 	mov    %rax,0xf8(%rsp)
  310496:	00 
<rexx_exec::Interp>::trace_keyword:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:298
  310497:	4c 89 64 24 10       	mov    %r12,0x10(%rsp)
  31049c:	48 89 6c 24 08       	mov    %rbp,0x8(%rsp)
  3104a1:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  3104a8:	48 8d 35 3e a7 d5 ff 	lea    -0x2a58c2(%rip),%rsi        # 6abed <panic_unwind::imp::CANARY+0x5e20>
  3104af:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  3104b6:	00 
  3104b7:	b9 01 00 00 00       	mov    $0x1,%ecx
  3104bc:	4d 89 e8             	mov    %r13,%r8
  3104bf:	49 89 d9             	mov    %rbx,%r9
  3104c2:	e8 99 03 f7 ff       	call   280860 <rexx_exec::trace::format::push_tagged>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:310
  3104c7:	4c 89 ff             	mov    %r15,%rdi
  3104ca:	48 8b b4 24 f8 00 00 	mov    0xf8(%rsp),%rsi
  3104d1:	00 
  3104d2:	e8 e9 dc e8 ff       	call   19e1c0 <<rexx_exec::Interp>::route_trace_line>
<rexx_exec::Interp>::parse_strings:
  3104d7:	4c 8b 7c 24 28       	mov    0x28(%rsp),%r15
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  3104dc:	49 8d 87 ff ef ff ff 	lea    -0x1001(%r15),%rax
  3104e3:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  3104e9:	0f 87 db 00 00 00    	ja     3105ca <<rexx_exec::Interp>::exec_parse+0xc9a>
  3104ef:	4d 85 ff             	test   %r15,%r15
  3104f2:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3104f7:	0f 84 53 01 00 00    	je     310650 <<rexx_exec::Interp>::exec_parse+0xd20>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3104fd:	48 89 ef             	mov    %rbp,%rdi
  310500:	ff 15 62 7c 1c 00    	call   *0x1c7c62(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  310506:	48 8d 05 4f 3f d4 ff 	lea    -0x2bc0b1(%rip),%rax        # 5445c <GCC_except_table5014+0x4a48>
  31050d:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310512:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  310516:	48 01 c1             	add    %rax,%rcx
  310519:	ff e1                	jmp    *%rcx
  31051b:	44 89 f0             	mov    %r14d,%eax
  31051e:	83 e0 03             	and    $0x3,%eax
  310521:	48 8d 0d 34 3f d4 ff 	lea    -0x2bc0cc(%rip),%rcx        # 5445c <GCC_except_table5014+0x4a48>
  310528:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  31052c:	48 01 c8             	add    %rcx,%rax
  31052f:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:114
  310531:	41 80 bc 24 04 20 00 	cmpb   $0x1,0x2004(%r12)
  310538:	00 01 
  31053a:	0f 85 4a 01 00 00    	jne    31068a <<rexx_exec::Interp>::exec_parse+0xd5a>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310540:	41 f6 c6 03          	test   $0x3,%r14b
  310544:	0f 85 1b 01 00 00    	jne    310665 <<rexx_exec::Interp>::exec_parse+0xd35>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  31054a:	4c 89 f0             	mov    %r14,%rax
  31054d:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  310551:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310553:	49 39 8c 24 08 0a 00 	cmp    %rcx,0xa08(%r12)
  31055a:	00 
  31055b:	0f 86 04 01 00 00    	jbe    310665 <<rexx_exec::Interp>::exec_parse+0xd35>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  310561:	49 8b 84 24 00 0a 00 	mov    0xa00(%r12),%rax
  310568:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  310569:	4c 89 f2             	mov    %r14,%rdx
  31056c:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310570:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  310574:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  310578:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  31057c:	0f 85 e3 00 00 00    	jne    310665 <<rexx_exec::Interp>::exec_parse+0xd35>
  310582:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  310585:	83 38 01             	cmpl   $0x1,(%rax)
  310588:	0f 85 d7 00 00 00    	jne    310665 <<rexx_exec::Interp>::exec_parse+0xd35>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  31058e:	80 78 08 01          	cmpb   $0x1,0x8(%rax)
  310592:	0f 87 cd 00 00 00    	ja     310665 <<rexx_exec::Interp>::exec_parse+0xd35>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310598:	49 8b 9c 24 f0 0a 00 	mov    0xaf0(%r12),%rbx
  31059f:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3105a0:	49 3b 9c 24 e0 0a 00 	cmp    0xae0(%r12),%rbx
  3105a7:	00 
  3105a8:	0f 84 2b 2c 00 00    	je     3131d9 <<rexx_exec::Interp>::exec_parse+0x38a9>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3105ae:	49 8b 84 24 e8 0a 00 	mov    0xae8(%r12),%rax
  3105b5:	00 
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3105b6:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  3105ba:	48 ff c3             	inc    %rbx
  3105bd:	49 89 9c 24 f0 0a 00 	mov    %rbx,0xaf0(%r12)
  3105c4:	00 
  3105c5:	e9 c0 00 00 00       	jmp    31068a <<rexx_exec::Interp>::exec_parse+0xd5a>
<rexx_exec::Interp>::parse_strings:
  3105ca:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3105cf:	49 8b 9c 24 e0 0b 00 	mov    0xbe0(%r12),%rbx
  3105d6:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3105d7:	48 83 fb 08          	cmp    $0x8,%rbx
  3105db:	72 1e                	jb     3105fb <<rexx_exec::Interp>::exec_parse+0xccb>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3105dd:	48 89 ef             	mov    %rbp,%rdi
  3105e0:	ff 15 82 7b 1c 00    	call   *0x1c7b82(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  3105e6:	48 8d 05 6f 3e d4 ff 	lea    -0x2bc191(%rip),%rax        # 5445c <GCC_except_table5014+0x4a48>
  3105ed:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3105f2:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  3105f6:	48 01 c1             	add    %rax,%rcx
  3105f9:	ff e1                	jmp    *%rcx
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3105fb:	49 3b 9c 24 d0 0b 00 	cmp    0xbd0(%r12),%rbx
  310602:	00 
  310603:	75 0d                	jne    310612 <<rexx_exec::Interp>::exec_parse+0xce2>
<rexx_exec::Interp>::give_parse_buffer:
  310605:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  31060c:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31060d:	e8 ee 0a 18 00       	call   491100 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310612:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  310619:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  31061a:	48 8d 0c 5b          	lea    (%rbx,%rbx,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  31061e:	4c 89 3c c8          	mov    %r15,(%rax,%rcx,8)
  310622:	48 89 6c c8 08       	mov    %rbp,0x8(%rax,%rcx,8)
  310627:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  31062e:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310630:	48 ff c3             	inc    %rbx
  310633:	49 89 9c 24 e0 0b 00 	mov    %rbx,0xbe0(%r12)
  31063a:	00 
  31063b:	48 8d 05 1a 3e d4 ff 	lea    -0x2bc1e6(%rip),%rax        # 5445c <GCC_except_table5014+0x4a48>
  310642:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310647:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  31064b:	48 01 c1             	add    %rax,%rcx
  31064e:	ff e1                	jmp    *%rcx
  310650:	48 8d 05 05 3e d4 ff 	lea    -0x2bc1fb(%rip),%rax        # 5445c <GCC_except_table5014+0x4a48>
  310657:	48 8b 4c 24 30       	mov    0x30(%rsp),%rcx
  31065c:	48 63 0c 88          	movslq (%rax,%rcx,4),%rcx
  310660:	48 01 c1             	add    %rax,%rcx
  310663:	ff e1                	jmp    *%rcx
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
  310665:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  31066a:	4c 89 e6             	mov    %r12,%rsi
  31066d:	4c 89 f2             	mov    %r14,%rdx
  310670:	31 c9                	xor    %ecx,%ecx
  310672:	e8 d9 69 ec ff       	call   1d7050 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310677:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  31067c:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
  310681:	4d 89 f4             	mov    %r14,%r12
  310684:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  310688:	75 52                	jne    3106dc <<rexx_exec::Interp>::exec_parse+0xdac>
<rexx_exec::Interp>::parse_strings:
  31068a:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:404
  31068f:	80 78 68 00          	cmpb   $0x0,0x68(%rax)
  310693:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  310698:	75 14                	jne    3106ae <<rexx_exec::Interp>::exec_parse+0xd7e>
  31069a:	f6 40 69 01          	testb  $0x1,0x69(%rax)
  31069e:	75 0e                	jne    3106ae <<rexx_exec::Interp>::exec_parse+0xd7e>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3106a0:	44 89 f0             	mov    %r14d,%eax
  3106a3:	83 e0 03             	and    $0x3,%eax
  3106a6:	8d 48 ff             	lea    -0x1(%rax),%ecx
  3106a9:	83 f9 02             	cmp    $0x2,%ecx
  3106ac:	73 42                	jae    3106f0 <<rexx_exec::Interp>::exec_parse+0xdc0>
<rexx_exec::Interp>::source_text:
  3106ae:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  3106b5:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:420
  3106b6:	4c 89 f2             	mov    %r14,%rdx
  3106b9:	e8 e2 61 04 00       	call   3568a0 <<rexx_exec::Interp>::rendered_into_parse_buffer>
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:656
  3106be:	4c 8b bc 24 00 01 00 	mov    0x100(%rsp),%r15
  3106c5:	00 
  3106c6:	48 8b ac 24 08 01 00 	mov    0x108(%rsp),%rbp
  3106cd:	00 
  3106ce:	4c 8b a4 24 10 01 00 	mov    0x110(%rsp),%r12
  3106d5:	00 
<core::result::Result<rexx_exec::parse_template::ParseStrings, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3106d6:	49 83 ff fd          	cmp    $0xfffffffffffffffd,%r15
  3106da:	75 5b                	jne    310737 <<rexx_exec::Interp>::exec_parse+0xe07>
<rexx_exec::Interp>::exec_parse:
  3106dc:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  3106e3:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  3106e4:	48 89 28             	mov    %rbp,(%rax)
  3106e7:	4c 89 60 08          	mov    %r12,0x8(%rax)
<rexx_exec::Interp>::exec_parse:
  3106eb:	e9 d7 28 00 00       	jmp    312fc7 <<rexx_exec::Interp>::exec_parse+0x3697>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  3106f0:	83 f8 03             	cmp    $0x3,%eax
  3106f3:	0f 85 2b 27 00 00    	jne    312e24 <<rexx_exec::Interp>::exec_parse+0x34f4>
  3106f9:	48 8d 9c 24 08 01 00 	lea    0x108(%rsp),%rbx
  310700:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  310701:	44 89 f0             	mov    %r14d,%eax
  310704:	c1 e8 02             	shr    $0x2,%eax
  310707:	83 e0 07             	and    $0x7,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  31070a:	49 81 e6 00 ff ff ff 	and    $0xffffffffffffff00,%r14
  310711:	49 09 c6             	or     %rax,%r14
<rexx_exec::Interp>::source_text:
  310714:	49 bf 01 00 00 00 00 	movabs $0x8000000000000001,%r15
  31071b:	00 00 80 
  31071e:	49 ff cf             	dec    %r15
  310721:	4d 89 f4             	mov    %r14,%r12
  310724:	4c 89 23             	mov    %r12,(%rbx)
<rexx_exec::Interp>::parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:656
  310727:	48 8b ac 24 08 01 00 	mov    0x108(%rsp),%rbp
  31072e:	00 
  31072f:	4c 8b a4 24 10 01 00 	mov    0x110(%rsp),%r12
  310736:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:481
  310737:	4c 89 bc 24 e0 01 00 	mov    %r15,0x1e0(%rsp)
  31073e:	00 
  31073f:	48 89 ac 24 e8 01 00 	mov    %rbp,0x1e8(%rsp)
  310746:	00 
  310747:	4c 89 a4 24 f0 01 00 	mov    %r12,0x1f0(%rsp)
  31074e:	00 
  31074f:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  310754:	0f b6 58 68          	movzbl 0x68(%rax),%ebx
  310758:	44 0f b6 40 69       	movzbl 0x69(%rax),%r8d
  31075d:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  310762:	48 8d 94 24 e0 01 00 	lea    0x1e0(%rsp),%rdx
  310769:	00 
  31076a:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  31076f:	4c 89 e6             	mov    %r12,%rsi
  310772:	89 d9                	mov    %ebx,%ecx
  310774:	44 89 84 24 d8 01 00 	mov    %r8d,0x1d8(%rsp)
  31077b:	00 
  31077c:	4c 8b 8c 24 c0 00 00 	mov    0xc0(%rsp),%r9
  310783:	00 
  310784:	e8 47 62 04 00       	call   3569d0 <<rexx_exec::Interp>::next_template>
<core::result::Result<(rexx_exec::parse_template::SourceText, rexx_exec::parse_template::Cursor), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310789:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  31078e:	0f 10 44 24 50       	movups 0x50(%rsp),%xmm0
  310793:	0f 29 84 24 00 01 00 	movaps %xmm0,0x100(%rsp)
  31079a:	00 
  31079b:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  31079f:	0f 84 02 26 00 00    	je     312da7 <<rexx_exec::Interp>::exec_parse+0x3477>
  3107a5:	89 9c 24 d4 01 00 00 	mov    %ebx,0x1d4(%rsp)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  3107ac:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
  3107b1:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
  3107b6:	48 8b 44 24 70       	mov    0x70(%rsp),%rax
  3107bb:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  3107c0:	48 8b 44 24 78       	mov    0x78(%rsp),%rax
  3107c5:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  3107cc:	00 
  3107cd:	48 8b 84 24 80 00 00 	mov    0x80(%rsp),%rax
  3107d4:	00 
  3107d5:	48 89 84 24 b8 01 00 	mov    %rax,0x1b8(%rsp)
  3107dc:	00 
  3107dd:	48 8b 9c 24 88 00 00 	mov    0x88(%rsp),%rbx
  3107e4:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  3107e5:	66 0f 6f 84 24 00 01 	movdqa 0x100(%rsp),%xmm0
  3107ec:	00 00 
  3107ee:	f3 0f 7f 84 24 d8 00 	movdqu %xmm0,0xd8(%rsp)
  3107f5:	00 00 
  3107f7:	4c 89 bc 24 d0 00 00 	mov    %r15,0xd0(%rsp)
  3107fe:	00 
  3107ff:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
<alloc::vec::Vec<core::option::Option<rexx_parse::ast::ParseTrigger>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310804:	48 8b 41 10          	mov    0x10(%rcx),%rax
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  310808:	48 85 c0             	test   %rax,%rax
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31080b:	0f 84 8c 26 00 00    	je     312e9d <<rexx_exec::Interp>::exec_parse+0x356d>
  310811:	48 8b 49 08          	mov    0x8(%rcx),%rcx
  310815:	48 6b c0 70          	imul   $0x70,%rax,%rax
  310819:	48 89 8c 24 48 01 00 	mov    %rcx,0x148(%rsp)
  310820:	00 
  310821:	48 01 c8             	add    %rcx,%rax
  310824:	48 89 84 24 f8 01 00 	mov    %rax,0x1f8(%rsp)
  31082b:	00 
  31082c:	48 b9 01 00 00 00 00 	movabs $0x8000000000000001,%rcx
  310833:	00 00 80 
  310836:	49 8d 84 24 e0 0a 00 	lea    0xae0(%r12),%rax
  31083d:	00 
  31083e:	48 89 84 24 40 01 00 	mov    %rax,0x140(%rsp)
  310845:	00 
  310846:	48 8b 84 24 e8 00 00 	mov    0xe8(%rsp),%rax
  31084d:	00 
  31084e:	48 8b 50 18          	mov    0x18(%rax),%rdx
  310852:	48 89 94 24 28 02 00 	mov    %rdx,0x228(%rsp)
  310859:	00 
  31085a:	48 8b 50 08          	mov    0x8(%rax),%rdx
  31085e:	48 89 94 24 20 02 00 	mov    %rdx,0x220(%rsp)
  310865:	00 
  310866:	48 8b 40 10          	mov    0x10(%rax),%rax
  31086a:	48 89 84 24 18 02 00 	mov    %rax,0x218(%rsp)
  310871:	00 
  310872:	49 8d 84 24 b0 0a 00 	lea    0xab0(%r12),%rax
  310879:	00 
  31087a:	48 89 84 24 10 02 00 	mov    %rax,0x210(%rsp)
  310881:	00 
  310882:	49 8d 84 24 d0 0b 00 	lea    0xbd0(%r12),%rax
  310889:	00 
  31088a:	48 89 84 24 38 01 00 	mov    %rax,0x138(%rsp)
  310891:	00 
  310892:	49 8d 84 24 d8 10 00 	lea    0x10d8(%r12),%rax
  310899:	00 
  31089a:	48 89 84 24 00 02 00 	mov    %rax,0x200(%rsp)
  3108a1:	00 
  3108a2:	48 8d 05 57 00 1b 00 	lea    0x1b0057(%rip),%rax        # 4c0900 <rexx_api::ffi::CALL_CONTEXT+0x53a0>
  3108a9:	48 89 84 24 b0 01 00 	mov    %rax,0x1b0(%rsp)
  3108b0:	00 
  3108b1:	49 8d 04 0f          	lea    (%r15,%rcx,1),%rax
  3108b5:	4c 89 bc 24 28 01 00 	mov    %r15,0x128(%rsp)
  3108bc:	00 
  3108bd:	49 c1 ff 3f          	sar    $0x3f,%r15
  3108c1:	49 21 c7             	and    %rax,%r15
  3108c4:	4c 89 bc 24 50 01 00 	mov    %r15,0x150(%rsp)
  3108cb:	00 
  3108cc:	48 8b 84 24 48 01 00 	mov    0x148(%rsp),%rax
  3108d3:	00 
  3108d4:	eb 23                	jmp    3108f9 <<rexx_exec::Interp>::exec_parse+0xfc9>
  3108d6:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
  3108dd:	00 00 00 
  3108e0:	4c 89 fb             	mov    %r15,%rbx
  3108e3:	48 8b 84 24 48 01 00 	mov    0x148(%rsp),%rax
  3108ea:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3108eb:	48 3b 84 24 f8 01 00 	cmp    0x1f8(%rsp),%rax
  3108f2:	00 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  3108f3:	0f 84 9c 25 00 00    	je     312e95 <<rexx_exec::Interp>::exec_parse+0x3565>
  3108f9:	4c 8b bc 24 b8 01 00 	mov    0x1b8(%rsp),%r15
  310900:	00 
  310901:	4c 8b a4 24 c8 00 00 	mov    0xc8(%rsp),%r12
  310908:	00 
  310909:	48 89 c5             	mov    %rax,%rbp
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:627
  31090c:	48 83 c0 70          	add    $0x70,%rax
  310910:	48 89 84 24 48 01 00 	mov    %rax,0x148(%rsp)
  310917:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:485
  310918:	48 83 7d 00 ff       	cmpq   $0xffffffffffffffff,0x0(%rbp)
  31091d:	0f 84 08 23 00 00    	je     312c2b <<rexx_exec::Interp>::exec_parse+0x32fb>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:31
  310923:	80 7d 68 00          	cmpb   $0x0,0x68(%rbp)
  310927:	0f 84 03 02 00 00    	je     310b30 <<rexx_exec::Interp>::exec_parse+0x1200>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:36
  31092d:	48 83 7d 18 ff       	cmpq   $0xffffffffffffffff,0x18(%rbp)
  310932:	0f 84 a1 26 00 00    	je     312fd9 <<rexx_exec::Interp>::exec_parse+0x36a9>
  310938:	48 89 6c 24 20       	mov    %rbp,0x20(%rsp)
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:127
  31093d:	c6 84 24 58 01 00 00 	movb   $0x0,0x158(%rsp)
  310944:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:128
  310945:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  31094a:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  31094f:	48 8d 94 24 58 01 00 	lea    0x158(%rsp),%rdx
  310956:	00 
  310957:	e8 a4 bf f6 ff       	call   27c900 <<rexx_exec::Interp>::enter_eval_node::<u8>>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  31095c:	4c 8b 6c 24 48       	mov    0x48(%rsp),%r13
  310961:	49 83 fd ff          	cmp    $0xffffffffffffffff,%r13
  310965:	0f 85 20 25 00 00    	jne    312e8b <<rexx_exec::Interp>::exec_parse+0x355b>
<rexx_exec::Interp>::apply_trigger:
  31096b:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
  310970:	4c 8d 70 18          	lea    0x18(%rax),%r14
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:129
  310974:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  31097b:	00 
  31097c:	48 8b 6c 24 18       	mov    0x18(%rsp),%rbp
  310981:	48 89 ee             	mov    %rbp,%rsi
  310984:	48 8b 94 24 e8 00 00 	mov    0xe8(%rsp),%rdx
  31098b:	00 
  31098c:	4c 89 f1             	mov    %r14,%rcx
  31098f:	e8 cc c1 f6 ff       	call   27cb60 <<rexx_exec::Interp>::eval_node>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:130
  310994:	48 ff 8d c8 04 00 00 	decq   0x4c8(%rbp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:131
  31099b:	4c 8b ac 24 00 01 00 	mov    0x100(%rsp),%r13
  3109a2:	00 
  3109a3:	49 83 fd ff          	cmp    $0xffffffffffffffff,%r13
  3109a7:	0f 85 9e 26 00 00    	jne    31304b <<rexx_exec::Interp>::exec_parse+0x371b>
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  3109ad:	f6 85 36 05 00 00 04 	testb  $0x4,0x536(%rbp)
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  3109b4:	48 8b 84 24 08 01 00 	mov    0x108(%rsp),%rax
  3109bb:	00 
  3109bc:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
<rexx_exec::Interp>::trace_intermediate:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:161
  3109c1:	74 31                	je     3109f4 <<rexx_exec::Interp>::exec_parse+0x10c4>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:164
  3109c3:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  3109c8:	48 89 ee             	mov    %rbp,%rsi
  3109cb:	48 8b 94 24 e8 00 00 	mov    0xe8(%rsp),%rdx
  3109d2:	00 
  3109d3:	4c 89 f1             	mov    %r14,%rcx
  3109d6:	4c 8b 44 24 30       	mov    0x30(%rsp),%r8
  3109db:	e8 a0 ec f6 ff       	call   27f680 <<rexx_exec::Interp>::trace_intermediate_event>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3109e0:	4c 8b 6c 24 48       	mov    0x48(%rsp),%r13
  3109e5:	49 83 fd ff          	cmp    $0xffffffffffffffff,%r13
  3109e9:	48 8b 6c 24 18       	mov    0x18(%rsp),%rbp
  3109ee:	0f 85 97 24 00 00    	jne    312e8b <<rexx_exec::Interp>::exec_parse+0x355b>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  3109f4:	4c 8b b5 f0 0a 00 00 	mov    0xaf0(%rbp),%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3109fb:	4c 3b b5 e0 0a 00 00 	cmp    0xae0(%rbp),%r14
  310a02:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
  310a07:	75 0d                	jne    310a16 <<rexx_exec::Interp>::exec_parse+0x10e6>
  310a09:	48 8b bc 24 40 01 00 	mov    0x140(%rsp),%rdi
  310a10:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310a11:	e8 6a b5 e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310a16:	48 8b 85 e8 0a 00 00 	mov    0xae8(%rbp),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310a1d:	4e 89 2c f0          	mov    %r13,(%rax,%r14,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310a21:	49 ff c6             	inc    %r14
  310a24:	4c 89 b5 f0 0a 00 00 	mov    %r14,0xaf0(%rbp)
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:55
  310a2b:	80 bd 2d 05 00 00 01 	cmpb   $0x1,0x52d(%rbp)
  310a32:	0f 84 62 21 00 00    	je     312b9a <<rexx_exec::Interp>::exec_parse+0x326a>
  310a38:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:59
  310a3d:	0f b6 45 68          	movzbl 0x68(%rbp),%eax
  310a41:	83 f8 06             	cmp    $0x6,%eax
  310a44:	0f 84 02 01 00 00    	je     310b4c <<rexx_exec::Interp>::exec_parse+0x121c>
  310a4a:	83 f8 07             	cmp    $0x7,%eax
  310a4d:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
  310a52:	0f 85 c4 01 00 00    	jne    310c1c <<rexx_exec::Interp>::exec_parse+0x12ec>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310a58:	44 89 e8             	mov    %r13d,%eax
  310a5b:	83 e0 03             	and    $0x3,%eax
  310a5e:	48 8d 15 07 3a d4 ff 	lea    -0x2bc5f9(%rip),%rdx        # 5446c <GCC_except_table5014+0x4a58>
  310a65:	48 63 0c 82          	movslq (%rdx,%rax,4),%rcx
  310a69:	48 01 d1             	add    %rdx,%rcx
  310a6c:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310a71:	ff e1                	jmp    *%rcx
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:114
  310a73:	41 80 be 04 20 00 00 	cmpb   $0x1,0x2004(%r14)
  310a7a:	01 
  310a7b:	0f 85 dc 01 00 00    	jne    310c5d <<rexx_exec::Interp>::exec_parse+0x132d>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310a81:	48 85 c0             	test   %rax,%rax
  310a84:	0f 85 f3 03 00 00    	jne    310e7d <<rexx_exec::Interp>::exec_parse+0x154d>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310a8a:	4c 89 e8             	mov    %r13,%rax
  310a8d:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  310a91:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310a93:	49 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%r14)
  310a9a:	0f 86 dd 03 00 00    	jbe    310e7d <<rexx_exec::Interp>::exec_parse+0x154d>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  310aa0:	49 8b 86 00 0a 00 00 	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  310aa7:	4c 89 ea             	mov    %r13,%rdx
  310aaa:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310aae:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  310ab2:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  310ab6:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  310aba:	0f 85 bd 03 00 00    	jne    310e7d <<rexx_exec::Interp>::exec_parse+0x154d>
  310ac0:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  310ac3:	83 38 01             	cmpl   $0x1,(%rax)
  310ac6:	0f 85 b1 03 00 00    	jne    310e7d <<rexx_exec::Interp>::exec_parse+0x154d>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  310acc:	80 78 08 02          	cmpb   $0x2,0x8(%rax)
  310ad0:	0f 83 a7 03 00 00    	jae    310e7d <<rexx_exec::Interp>::exec_parse+0x154d>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310ad6:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310add:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  310ae4:	75 0d                	jne    310af3 <<rexx_exec::Interp>::exec_parse+0x11c3>
  310ae6:	48 8b bc 24 40 01 00 	mov    0x140(%rsp),%rdi
  310aed:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310aee:	e8 8d b4 e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310af3:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310afa:	4c 89 2c d8          	mov    %r13,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310afe:	48 ff c3             	inc    %rbx
  310b01:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310b08:	44 89 e8             	mov    %r13d,%eax
  310b0b:	83 e0 03             	and    $0x3,%eax
  310b0e:	48 8d 0d 67 39 d4 ff 	lea    -0x2bc699(%rip),%rcx        # 5447c <GCC_except_table5014+0x4a68>
  310b15:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310b19:	48 01 c8             	add    %rcx,%rax
  310b1c:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  310b21:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::apply_trigger:
  310b23:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  310b2a:	84 00 00 00 00 00 
  310b30:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  310b35:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  310b3a:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  310b41:	00 
  310b42:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  310b47:	e9 1b 0c 00 00       	jmp    311767 <<rexx_exec::Interp>::exec_parse+0x1e37>
  310b4c:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310b51:	44 89 e8             	mov    %r13d,%eax
  310b54:	83 e0 03             	and    $0x3,%eax
  310b57:	48 8d 15 3e 39 d4 ff 	lea    -0x2bc6c2(%rip),%rdx        # 5449c <GCC_except_table5014+0x4a88>
  310b5e:	48 63 0c 82          	movslq (%rdx,%rax,4),%rcx
  310b62:	48 01 d1             	add    %rdx,%rcx
  310b65:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310b6a:	ff e1                	jmp    *%rcx
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:114
  310b6c:	41 80 be 04 20 00 00 	cmpb   $0x1,0x2004(%r14)
  310b73:	01 
  310b74:	0f 85 6f 01 00 00    	jne    310ce9 <<rexx_exec::Interp>::exec_parse+0x13b9>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310b7a:	48 85 c0             	test   %rax,%rax
  310b7d:	0f 85 4e 03 00 00    	jne    310ed1 <<rexx_exec::Interp>::exec_parse+0x15a1>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310b83:	4c 89 e8             	mov    %r13,%rax
  310b86:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:628
  310b8a:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310b8c:	49 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%r14)
  310b93:	0f 86 38 03 00 00    	jbe    310ed1 <<rexx_exec::Interp>::exec_parse+0x15a1>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:123
  310b99:	49 8b 86 00 0a 00 00 	mov    0xa00(%r14),%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:165
  310ba0:	4c 89 ea             	mov    %r13,%rdx
  310ba3:	48 c1 ea 22          	shr    $0x22,%rdx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310ba7:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  310bab:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::resolve:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:629
  310baf:	39 54 08 04          	cmp    %edx,0x4(%rax,%rcx,1)
  310bb3:	0f 85 18 03 00 00    	jne    310ed1 <<rexx_exec::Interp>::exec_parse+0x15a1>
  310bb9:	48 01 c8             	add    %rcx,%rax
<core::option::Option<usize> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2869
  310bbc:	83 38 01             	cmpl   $0x1,(%rax)
  310bbf:	0f 85 0c 03 00 00    	jne    310ed1 <<rexx_exec::Interp>::exec_parse+0x15a1>
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:122
  310bc5:	80 78 08 02          	cmpb   $0x2,0x8(%rax)
  310bc9:	0f 83 02 03 00 00    	jae    310ed1 <<rexx_exec::Interp>::exec_parse+0x15a1>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  310bcf:	49 8b 9e f0 0a 00 00 	mov    0xaf0(%r14),%rbx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  310bd6:	49 3b 9e e0 0a 00 00 	cmp    0xae0(%r14),%rbx
  310bdd:	75 0d                	jne    310bec <<rexx_exec::Interp>::exec_parse+0x12bc>
  310bdf:	48 8b bc 24 40 01 00 	mov    0x140(%rsp),%rdi
  310be6:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  310be7:	e8 94 b3 e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310bec:	49 8b 86 e8 0a 00 00 	mov    0xae8(%r14),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  310bf3:	4c 89 2c d8          	mov    %r13,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  310bf7:	48 ff c3             	inc    %rbx
  310bfa:	49 89 9e f0 0a 00 00 	mov    %rbx,0xaf0(%r14)
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310c01:	44 89 e8             	mov    %r13d,%eax
  310c04:	83 e0 03             	and    $0x3,%eax
  310c07:	48 8d 0d 9e 38 d4 ff 	lea    -0x2bc762(%rip),%rcx        # 544ac <GCC_except_table5014+0x4a98>
  310c0e:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310c12:	48 01 c8             	add    %rcx,%rax
  310c15:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  310c1a:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::apply_trigger:
  310c1c:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:92
  310c21:	4c 89 ee             	mov    %r13,%rsi
  310c24:	e8 47 65 04 00       	call   357170 <<rexx_exec::Interp>::whole_nonneg>
  310c29:	a8 01                	test   $0x1,%al
  310c2b:	0f 84 a8 24 00 00    	je     3130d9 <<rexx_exec::Interp>::exec_parse+0x37a9>
  310c31:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:101
  310c36:	0f b6 45 68          	movzbl 0x68(%rbp),%eax
  310c3a:	48 8d 0d 8b 38 d4 ff 	lea    -0x2bc775(%rip),%rcx        # 544cc <GCC_except_table5014+0x4ab8>
  310c41:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310c45:	48 01 c8             	add    %rcx,%rax
  310c48:	ff e0                	jmp    *%rax
  310c4a:	4c 89 a4 24 c8 00 00 	mov    %r12,0xc8(%rsp)
  310c51:	00 
  310c52:	4c 89 f8             	mov    %r15,%rax
  310c55:	49 89 df             	mov    %rbx,%r15
  310c58:	e9 e5 fe ff ff       	jmp    310b42 <<rexx_exec::Interp>::exec_parse+0x1212>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310c5d:	44 89 e8             	mov    %r13d,%eax
  310c60:	83 e0 03             	and    $0x3,%eax
  310c63:	48 8d 0d 12 38 d4 ff 	lea    -0x2bc7ee(%rip),%rcx        # 5447c <GCC_except_table5014+0x4a68>
  310c6a:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310c6e:	48 01 c8             	add    %rcx,%rax
  310c71:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  310c73:	4c 89 f7             	mov    %r14,%rdi
  310c76:	4c 89 ee             	mov    %r13,%rsi
  310c79:	e8 92 e0 ec ff       	call   1ded10 <<rexx_exec::Interp>::try_text>
<core::option::Option<&[u8]>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:634
  310c7e:	48 85 c0             	test   %rax,%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  310c81:	0f 84 9e 02 00 00    	je     310f25 <<rexx_exec::Interp>::exec_parse+0x15f5>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:955
  310c87:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  310c8c:	c6 44 24 48 00       	movb   $0x0,0x48(%rsp)
  310c91:	31 c0                	xor    %eax,%eax
  310c93:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310c98:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  310c9d:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  310ca4:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310ca5:	48 85 c9             	test   %rcx,%rcx
  310ca8:	0f 84 fc 04 00 00    	je     3111aa <<rexx_exec::Interp>::exec_parse+0x187a>
  310cae:	48 83 f9 01          	cmp    $0x1,%rcx
  310cb2:	0f 85 bf 00 00 00    	jne    310d77 <<rexx_exec::Interp>::exec_parse+0x1447>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  310cb8:	44 0f b6 a4 24 d8 00 	movzbl 0xd8(%rsp),%r12d
  310cbf:	00 00 
  310cc1:	49 83 fc 08          	cmp    $0x8,%r12
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310cc5:	0f 83 27 2a 00 00    	jae    3136f2 <<rexx_exec::Interp>::exec_parse+0x3dc2>
<rexx_exec::Interp>::apply_trigger:
  310ccb:	4c 89 ee             	mov    %r13,%rsi
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310cce:	0f b6 c0             	movzbl %al,%eax
  310cd1:	48 8d 0d b4 37 d4 ff 	lea    -0x2bc84c(%rip),%rcx        # 5448c <GCC_except_table5014+0x4a78>
  310cd8:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310cdc:	48 01 c8             	add    %rcx,%rax
  310cdf:	4c 8d ac 24 d9 00 00 	lea    0xd9(%rsp),%r13
  310ce6:	00 
  310ce7:	ff e0                	jmp    *%rax
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310ce9:	44 89 e8             	mov    %r13d,%eax
  310cec:	83 e0 03             	and    $0x3,%eax
  310cef:	48 8d 0d b6 37 d4 ff 	lea    -0x2bc84a(%rip),%rcx        # 544ac <GCC_except_table5014+0x4a98>
  310cf6:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310cfa:	48 01 c8             	add    %rcx,%rax
  310cfd:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  310cff:	4c 89 f7             	mov    %r14,%rdi
  310d02:	4c 89 ee             	mov    %r13,%rsi
  310d05:	e8 06 e0 ec ff       	call   1ded10 <<rexx_exec::Interp>::try_text>
<core::option::Option<&[u8]>>::is_some:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:634
  310d0a:	48 85 c0             	test   %rax,%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:954
  310d0d:	0f 84 5e 02 00 00    	je     310f71 <<rexx_exec::Interp>::exec_parse+0x1641>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:955
  310d13:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  310d18:	c6 44 24 48 00       	movb   $0x0,0x48(%rsp)
  310d1d:	31 c0                	xor    %eax,%eax
  310d1f:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310d24:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  310d29:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  310d30:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  310d31:	48 85 c9             	test   %rcx,%rcx
  310d34:	0f 84 c7 06 00 00    	je     311401 <<rexx_exec::Interp>::exec_parse+0x1ad1>
  310d3a:	48 83 f9 01          	cmp    $0x1,%rcx
  310d3e:	0f 85 b5 00 00 00    	jne    310df9 <<rexx_exec::Interp>::exec_parse+0x14c9>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  310d44:	44 0f b6 a4 24 d8 00 	movzbl 0xd8(%rsp),%r12d
  310d4b:	00 00 
  310d4d:	49 83 fc 08          	cmp    $0x8,%r12
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  310d51:	0f 83 7d 29 00 00    	jae    3136d4 <<rexx_exec::Interp>::exec_parse+0x3da4>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310d57:	0f b6 c0             	movzbl %al,%eax
  310d5a:	48 8d 0d 5b 37 d4 ff 	lea    -0x2bc8a5(%rip),%rcx        # 544bc <GCC_except_table5014+0x4aa8>
  310d61:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310d65:	48 01 c8             	add    %rcx,%rax
  310d68:	48 8d 8c 24 d9 00 00 	lea    0xd9(%rsp),%rcx
  310d6f:	00 
  310d70:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
  310d75:	ff e0                	jmp    *%rax
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310d77:	48 8b 94 24 d8 00 00 	mov    0xd8(%rsp),%rdx
  310d7e:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310d7f:	f6 c2 03             	test   $0x3,%dl
  310d82:	0f 85 01 2b 00 00    	jne    313889 <<rexx_exec::Interp>::exec_parse+0x3f59>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310d88:	48 89 d1             	mov    %rdx,%rcx
  310d8b:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310d8f:	89 ce                	mov    %ecx,%esi
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310d91:	49 39 b6 08 0a 00 00 	cmp    %rsi,0xa08(%r14)
  310d98:	0f 86 eb 2a 00 00    	jbe    313889 <<rexx_exec::Interp>::exec_parse+0x3f59>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310d9e:	49 8b 8e 00 0a 00 00 	mov    0xa00(%r14),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310da5:	48 8d 34 76          	lea    (%rsi,%rsi,2),%rsi
  310da9:	48 c1 e6 05          	shl    $0x5,%rsi
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310dad:	83 3c 31 01          	cmpl   $0x1,(%rcx,%rsi,1)
  310db1:	0f 85 d2 2a 00 00    	jne    313889 <<rexx_exec::Interp>::exec_parse+0x3f59>
  310db7:	48 01 f1             	add    %rsi,%rcx
  310dba:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  310dbe:	0f 85 c5 2a 00 00    	jne    313889 <<rexx_exec::Interp>::exec_parse+0x3f59>
  310dc4:	48 c1 ea 22          	shr    $0x22,%rdx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  310dc8:	39 51 04             	cmp    %edx,0x4(%rcx)
  310dcb:	0f 85 b8 2a 00 00    	jne    313889 <<rexx_exec::Interp>::exec_parse+0x3f59>
  310dd1:	4c 89 ee             	mov    %r13,%rsi
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  310dd4:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  310dd8:	0f b6 c0             	movzbl %al,%eax
  310ddb:	0f 85 dc 01 00 00    	jne    310fbd <<rexx_exec::Interp>::exec_parse+0x168d>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310de1:	4c 8b 69 20          	mov    0x20(%rcx),%r13
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310de5:	4c 8b 61 28          	mov    0x28(%rcx),%r12
  310de9:	48 8d 0d 9c 36 d4 ff 	lea    -0x2bc964(%rip),%rcx        # 5448c <GCC_except_table5014+0x4a78>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310df0:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310df4:	48 01 c8             	add    %rcx,%rax
  310df7:	ff e0                	jmp    *%rax
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310df9:	48 8b 94 24 d8 00 00 	mov    0xd8(%rsp),%rdx
  310e00:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310e01:	f6 c2 03             	test   $0x3,%dl
  310e04:	0f 85 99 2a 00 00    	jne    3138a3 <<rexx_exec::Interp>::exec_parse+0x3f73>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  310e0a:	48 89 d1             	mov    %rdx,%rcx
  310e0d:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310e11:	89 ce                	mov    %ecx,%esi
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  310e13:	49 39 b6 08 0a 00 00 	cmp    %rsi,0xa08(%r14)
  310e1a:	0f 86 83 2a 00 00    	jbe    3138a3 <<rexx_exec::Interp>::exec_parse+0x3f73>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  310e20:	49 8b 8e 00 0a 00 00 	mov    0xa00(%r14),%rcx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  310e27:	48 8d 34 76          	lea    (%rsi,%rsi,2),%rsi
  310e2b:	48 c1 e6 05          	shl    $0x5,%rsi
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  310e2f:	83 3c 31 01          	cmpl   $0x1,(%rcx,%rsi,1)
  310e33:	0f 85 6a 2a 00 00    	jne    3138a3 <<rexx_exec::Interp>::exec_parse+0x3f73>
  310e39:	48 01 f1             	add    %rsi,%rcx
  310e3c:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  310e40:	0f 85 5d 2a 00 00    	jne    3138a3 <<rexx_exec::Interp>::exec_parse+0x3f73>
  310e46:	48 c1 ea 22          	shr    $0x22,%rdx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  310e4a:	39 51 04             	cmp    %edx,0x4(%rcx)
  310e4d:	0f 85 50 2a 00 00    	jne    3138a3 <<rexx_exec::Interp>::exec_parse+0x3f73>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  310e53:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  310e57:	0f 85 7c 01 00 00    	jne    310fd9 <<rexx_exec::Interp>::exec_parse+0x16a9>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  310e5d:	48 8b 51 20          	mov    0x20(%rcx),%rdx
  310e61:	48 89 54 24 20       	mov    %rdx,0x20(%rsp)
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  310e66:	4c 8b 61 28          	mov    0x28(%rcx),%r12
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310e6a:	0f b6 c0             	movzbl %al,%eax
  310e6d:	48 8d 0d 48 36 d4 ff 	lea    -0x2bc9b8(%rip),%rcx        # 544bc <GCC_except_table5014+0x4aa8>
  310e74:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310e78:	48 01 c8             	add    %rcx,%rax
  310e7b:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  310e7d:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  310e82:	4c 89 f6             	mov    %r14,%rsi
  310e85:	4c 89 ea             	mov    %r13,%rdx
  310e88:	31 c9                	xor    %ecx,%ecx
  310e8a:	e8 c1 61 ec ff       	call   1d7050 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310e8f:	4c 8b 6c 24 48       	mov    0x48(%rsp),%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  310e94:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310e99:	49 83 fd ff          	cmp    $0xffffffffffffffff,%r13
  310e9d:	0f 85 b0 21 00 00    	jne    313053 <<rexx_exec::Interp>::exec_parse+0x3723>
<rexx_exec::Interp>::apply_trigger:
  310ea3:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
<rexx_core::handle::ObjRef as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:71
  310ea8:	4d 39 ee             	cmp    %r13,%r14
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:75
  310eab:	0f 85 34 02 00 00    	jne    3110e5 <<rexx_exec::Interp>::exec_parse+0x17b5>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310eb1:	44 89 e8             	mov    %r13d,%eax
  310eb4:	83 e0 03             	and    $0x3,%eax
  310eb7:	48 8d 0d be 35 d4 ff 	lea    -0x2bca42(%rip),%rcx        # 5447c <GCC_except_table5014+0x4a68>
  310ebe:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310ec2:	48 01 c8             	add    %rcx,%rax
  310ec5:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310eca:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  310ecf:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::required_string_or::<<rexx_exec::Interp>::required_string_value::{closure#0}>:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/dispatch/reqstr.rs:129
  310ed1:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  310ed6:	4c 89 f6             	mov    %r14,%rsi
  310ed9:	4c 89 ea             	mov    %r13,%rdx
  310edc:	31 c9                	xor    %ecx,%ecx
  310ede:	e8 6d 61 ec ff       	call   1d7050 <<rexx_exec::Interp>::required_string_dispatch>
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310ee3:	4c 8b 6c 24 48       	mov    0x48(%rsp),%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  310ee8:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  310eed:	49 83 fd ff          	cmp    $0xffffffffffffffff,%r13
  310ef1:	0f 85 5c 21 00 00    	jne    313053 <<rexx_exec::Interp>::exec_parse+0x3723>
<rexx_exec::Interp>::apply_trigger:
  310ef7:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
<rexx_core::handle::ObjRef as core::cmp::PartialEq>::eq:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:71
  310efc:	4d 39 ee             	cmp    %r13,%r14
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:65
  310eff:	0f 85 37 04 00 00    	jne    31133c <<rexx_exec::Interp>::exec_parse+0x1a0c>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  310f05:	44 89 e8             	mov    %r13d,%eax
  310f08:	83 e0 03             	and    $0x3,%eax
  310f0b:	48 8d 0d 9a 35 d4 ff 	lea    -0x2bca66(%rip),%rcx        # 544ac <GCC_except_table5014+0x4a98>
  310f12:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  310f16:	48 01 c8             	add    %rcx,%rax
  310f19:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  310f1e:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  310f23:	ff e0                	jmp    *%rax
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:960
  310f25:	48 8d bc 24 58 01 00 	lea    0x158(%rsp),%rdi
  310f2c:	00 
  310f2d:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  310f32:	4c 89 ea             	mov    %r13,%rdx
  310f35:	e8 86 ea e9 ff       	call   1af9c0 <<rexx_exec::Interp>::to_text>
  310f3a:	48 83 bc 24 58 01 00 	cmpq   $0xffffffffffffffff,0x158(%rsp)
  310f41:	00 ff 
  310f43:	0f 84 8f 06 00 00    	je     3115d8 <<rexx_exec::Interp>::exec_parse+0x1ca8>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:962
  310f49:	48 8b 84 24 68 01 00 	mov    0x168(%rsp),%rax
  310f50:	00 
  310f51:	48 8d 8c 24 07 01 00 	lea    0x107(%rsp),%rcx
  310f58:	00 
  310f59:	48 89 41 10          	mov    %rax,0x10(%rcx)
  310f5d:	f3 0f 6f 84 24 58 01 	movdqu 0x158(%rsp),%xmm0
  310f64:	00 00 
  310f66:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  310f6a:	b0 01                	mov    $0x1,%al
  310f6c:	e9 69 06 00 00       	jmp    3115da <<rexx_exec::Interp>::exec_parse+0x1caa>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:960
  310f71:	48 8d bc 24 58 01 00 	lea    0x158(%rsp),%rdi
  310f78:	00 
  310f79:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  310f7e:	4c 89 ea             	mov    %r13,%rdx
  310f81:	e8 3a ea e9 ff       	call   1af9c0 <<rexx_exec::Interp>::to_text>
  310f86:	48 83 bc 24 58 01 00 	cmpq   $0xffffffffffffffff,0x158(%rsp)
  310f8d:	00 ff 
  310f8f:	0f 84 8e 06 00 00    	je     311623 <<rexx_exec::Interp>::exec_parse+0x1cf3>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:962
  310f95:	48 8b 84 24 68 01 00 	mov    0x168(%rsp),%rax
  310f9c:	00 
  310f9d:	48 8d 8c 24 07 01 00 	lea    0x107(%rsp),%rcx
  310fa4:	00 
  310fa5:	48 89 41 10          	mov    %rax,0x10(%rcx)
  310fa9:	f3 0f 6f 84 24 58 01 	movdqu 0x158(%rsp),%xmm0
  310fb0:	00 00 
  310fb2:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  310fb6:	b0 01                	mov    $0x1,%al
  310fb8:	e9 68 06 00 00       	jmp    311625 <<rexx_exec::Interp>::exec_parse+0x1cf5>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  310fbd:	44 0f b6 61 11       	movzbl 0x11(%rcx),%r12d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  310fc2:	48 83 c1 12          	add    $0x12,%rcx
  310fc6:	48 8d 15 bf 34 d4 ff 	lea    -0x2bcb41(%rip),%rdx        # 5448c <GCC_except_table5014+0x4a78>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310fcd:	48 63 04 82          	movslq (%rdx,%rax,4),%rax
  310fd1:	48 01 d0             	add    %rdx,%rax
  310fd4:	49 89 cd             	mov    %rcx,%r13
  310fd7:	ff e0                	jmp    *%rax
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  310fd9:	44 0f b6 61 11       	movzbl 0x11(%rcx),%r12d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  310fde:	48 83 c1 12          	add    $0x12,%rcx
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  310fe2:	0f b6 c0             	movzbl %al,%eax
  310fe5:	48 8d 15 d0 34 d4 ff 	lea    -0x2bcb30(%rip),%rdx        # 544bc <GCC_except_table5014+0x4aa8>
  310fec:	48 63 04 82          	movslq (%rdx,%rax,4),%rax
  310ff0:	48 01 d0             	add    %rdx,%rax
  310ff3:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
  310ff8:	ff e0                	jmp    *%rax
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  310ffa:	4c 89 e0             	mov    %r12,%rax
  310ffd:	48 29 d0             	sub    %rdx,%rax
  311000:	b9 00 00 00 00       	mov    $0x0,%ecx
  311005:	48 0f 43 c8          	cmovae %rax,%rcx
  311009:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  31100e:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  311013:	48 89 8c 24 c8 00 00 	mov    %rcx,0xc8(%rsp)
  31101a:	00 
  31101b:	48 89 c8             	mov    %rcx,%rax
  31101e:	4d 89 e7             	mov    %r12,%r15
  311021:	e9 1c fb ff ff       	jmp    310b42 <<rexx_exec::Interp>::exec_parse+0x1212>
<usize>::saturating_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2496
  311026:	4c 01 e2             	add    %r12,%rdx
  311029:	48 c7 c0 ff ff ff ff 	mov    $0xffffffffffffffff,%rax
  311030:	48 0f 42 d0          	cmovb  %rax,%rdx
  311034:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
<usize as core::cmp::Ord>::min:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:1090
  311039:	48 39 d0             	cmp    %rdx,%rax
  31103c:	48 0f 42 d0          	cmovb  %rax,%rdx
<rexx_exec::parse_template::Cursor>::forward:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:153
  311040:	4c 39 e2             	cmp    %r12,%rdx
  311043:	48 0f 47 c2          	cmova  %rdx,%rax
  311047:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  31104c:	49 0f 46 d4          	cmovbe %r12,%rdx
  311050:	eb 1f                	jmp    311071 <<rexx_exec::Interp>::exec_parse+0x1741>
<usize>::saturating_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2496
  311052:	4c 01 e2             	add    %r12,%rdx
  311055:	48 c7 c0 ff ff ff ff 	mov    $0xffffffffffffffff,%rax
  31105c:	48 0f 42 d0          	cmovb  %rax,%rdx
  311060:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
<usize as core::cmp::Ord>::min:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:1090
  311065:	48 39 d0             	cmp    %rdx,%rax
  311068:	48 0f 42 d0          	cmovb  %rax,%rdx
  31106c:	48 89 54 24 28       	mov    %rdx,0x28(%rsp)
<rexx_exec::Interp>::apply_trigger:
  311071:	48 89 94 24 c8 00 00 	mov    %rdx,0xc8(%rsp)
  311078:	00 
  311079:	48 89 d0             	mov    %rdx,%rax
  31107c:	4d 89 e7             	mov    %r12,%r15
  31107f:	e9 be fa ff ff       	jmp    310b42 <<rexx_exec::Interp>::exec_parse+0x1212>
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  311084:	48 83 ea 01          	sub    $0x1,%rdx
  311088:	b8 00 00 00 00       	mov    $0x0,%eax
  31108d:	48 0f 43 c2          	cmovae %rdx,%rax
  311091:	48 8b 4c 24 40       	mov    0x40(%rsp),%rcx
<rexx_exec::parse_template::Cursor>::absolute:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:180
  311096:	48 39 c1             	cmp    %rax,%rcx
  311099:	48 89 c2             	mov    %rax,%rdx
  31109c:	48 0f 42 d1          	cmovb  %rcx,%rdx
  3110a0:	4c 39 f8             	cmp    %r15,%rax
  3110a3:	48 0f 47 ca          	cmova  %rdx,%rcx
  3110a7:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
  3110ac:	48 0f 46 d0          	cmovbe %rax,%rdx
  3110b0:	48 89 94 24 c8 00 00 	mov    %rdx,0xc8(%rsp)
  3110b7:	00 
  3110b8:	48 89 d0             	mov    %rdx,%rax
  3110bb:	e9 82 fa ff ff       	jmp    310b42 <<rexx_exec::Interp>::exec_parse+0x1212>
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  3110c0:	4c 89 e0             	mov    %r12,%rax
  3110c3:	48 29 d0             	sub    %rdx,%rax
  3110c6:	41 bf 00 00 00 00    	mov    $0x0,%r15d
  3110cc:	4c 0f 43 f8          	cmovae %rax,%r15
  3110d0:	4c 89 64 24 28       	mov    %r12,0x28(%rsp)
  3110d5:	4c 89 f8             	mov    %r15,%rax
  3110d8:	4c 89 bc 24 c8 00 00 	mov    %r15,0xc8(%rsp)
  3110df:	00 
  3110e0:	e9 5d fa ff ff       	jmp    310b42 <<rexx_exec::Interp>::exec_parse+0x1212>
<rexx_exec::Interp>::apply_trigger:
  3110e5:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  3110ea:	49 8b 9c 24 f0 0a 00 	mov    0xaf0(%r12),%rbx
  3110f1:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3110f2:	49 3b 9c 24 e0 0a 00 	cmp    0xae0(%r12),%rbx
  3110f9:	00 
  3110fa:	75 0d                	jne    311109 <<rexx_exec::Interp>::exec_parse+0x17d9>
  3110fc:	48 8b bc 24 40 01 00 	mov    0x140(%rsp),%rdi
  311103:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  311104:	e8 77 ae e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311109:	49 8b 84 24 e8 0a 00 	mov    0xae8(%r12),%rax
  311110:	00 
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  311111:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  311115:	48 ff c3             	inc    %rbx
  311118:	49 89 9c 24 f0 0a 00 	mov    %rbx,0xaf0(%r12)
  31111f:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  311120:	44 89 f0             	mov    %r14d,%eax
  311123:	83 e0 03             	and    $0x3,%eax
  311126:	48 8d 0d 4f 33 d4 ff 	lea    -0x2bccb1(%rip),%rcx        # 5447c <GCC_except_table5014+0x4a68>
  31112d:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  311131:	48 01 c8             	add    %rcx,%rax
  311134:	4d 89 f5             	mov    %r14,%r13
  311137:	4d 89 e6             	mov    %r12,%r14
  31113a:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  31113f:	ff e0                	jmp    *%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:167
  311141:	4c 89 ee             	mov    %r13,%rsi
  311144:	48 c1 fe 02          	sar    $0x2,%rsi
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:942
  311148:	66 0f ef c0          	pxor   %xmm0,%xmm0
  31114c:	66 0f 7f 84 24 90 01 	movdqa %xmm0,0x190(%rsp)
  311153:	00 00 
  311155:	c7 84 24 a0 01 00 00 	movl   $0x0,0x1a0(%rsp)
  31115c:	00 00 00 00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:943
  311160:	48 8d bc 24 90 01 00 	lea    0x190(%rsp),%rdi
  311167:	00 
  311168:	e8 73 ee e9 ff       	call   1affe0 <rexx_exec::value::write_small_int>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:944
  31116d:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  311172:	c6 44 24 48 03       	movb   $0x3,0x48(%rsp)
  311177:	88 44 24 49          	mov    %al,0x49(%rsp)
  31117b:	8b 84 24 a0 01 00 00 	mov    0x1a0(%rsp),%eax
  311182:	48 8d 4c 24 4a       	lea    0x4a(%rsp),%rcx
  311187:	89 41 10             	mov    %eax,0x10(%rcx)
  31118a:	66 0f 6f 84 24 90 01 	movdqa 0x190(%rsp),%xmm0
  311191:	00 00 
  311193:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  311197:	b0 03                	mov    $0x3,%al
  311199:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  3111a0:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  3111a1:	48 85 c9             	test   %rcx,%rcx
  3111a4:	0f 85 04 fb ff ff    	jne    310cae <<rexx_exec::Interp>::exec_parse+0x137e>
  3111aa:	4c 89 ee             	mov    %r13,%rsi
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3111ad:	4c 8b ac 24 d8 00 00 	mov    0xd8(%rsp),%r13
  3111b4:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  3111b5:	4c 8b a4 24 e0 00 00 	mov    0xe0(%rsp),%r12
  3111bc:	00 
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  3111bd:	0f b6 c0             	movzbl %al,%eax
  3111c0:	48 8d 0d c5 32 d4 ff 	lea    -0x2bcd3b(%rip),%rcx        # 5448c <GCC_except_table5014+0x4a78>
  3111c7:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  3111cb:	48 01 c8             	add    %rcx,%rax
  3111ce:	ff e0                	jmp    *%rax
  3111d0:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1695
  3111d5:	e8 36 db ec ff       	call   1ded10 <<rexx_exec::Interp>::try_text>
  3111da:	49 89 c6             	mov    %rax,%r14
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  3111dd:	48 85 c0             	test   %rax,%rax
  3111e0:	0f 84 62 25 00 00    	je     313748 <<rexx_exec::Interp>::exec_parse+0x3e18>
<rexx_exec::value::Rendered>::text:
  3111e6:	48 89 d3             	mov    %rdx,%rbx
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  3111e9:	eb 7c                	jmp    311267 <<rexx_exec::Interp>::exec_parse+0x1937>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  3111eb:	44 89 e8             	mov    %r13d,%eax
  3111ee:	c1 e8 02             	shr    $0x2,%eax
  3111f1:	83 e0 07             	and    $0x7,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  3111f4:	4c 89 e9             	mov    %r13,%rcx
  3111f7:	48 81 e1 00 ff ff ff 	and    $0xffffffffffffff00,%rcx
  3111fe:	48 09 c1             	or     %rax,%rcx
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:931
  311201:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  311206:	c6 44 24 48 02       	movb   $0x2,0x48(%rsp)
  31120b:	48 89 4c 24 49       	mov    %rcx,0x49(%rsp)
  311210:	b0 02                	mov    $0x2,%al
  311212:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  311219:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  31121a:	48 85 c9             	test   %rcx,%rcx
  31121d:	0f 85 8b fa ff ff    	jne    310cae <<rexx_exec::Interp>::exec_parse+0x137e>
  311223:	eb 85                	jmp    3111aa <<rexx_exec::Interp>::exec_parse+0x187a>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  311225:	0f b6 5c 24 49       	movzbl 0x49(%rsp),%ebx
  31122a:	48 83 fb 07          	cmp    $0x7,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  31122e:	0f 87 bb 24 00 00    	ja     3136ef <<rexx_exec::Interp>::exec_parse+0x3dbf>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
  311234:	4c 8d 74 24 4a       	lea    0x4a(%rsp),%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:86
  311239:	eb 2c                	jmp    311267 <<rexx_exec::Interp>::exec_parse+0x1937>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  31123b:	44 0f b6 74 24 49    	movzbl 0x49(%rsp),%r14d
  311241:	49 83 fe 14          	cmp    $0x14,%r14
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:544
  311245:	0f 87 c2 24 00 00    	ja     31370d <<rexx_exec::Interp>::exec_parse+0x3ddd>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  31124b:	bb 14 00 00 00       	mov    $0x14,%ebx
  311250:	4c 29 f3             	sub    %r14,%rbx
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  311253:	48 8d 44 24 4a       	lea    0x4a(%rsp),%rax
  311258:	49 01 c6             	add    %rax,%r14
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  31125b:	eb 0a                	jmp    311267 <<rexx_exec::Interp>::exec_parse+0x1937>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  31125d:	4c 8b 74 24 58       	mov    0x58(%rsp),%r14
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311262:	48 8b 5c 24 60       	mov    0x60(%rsp),%rbx
<rexx_exec::Interp>::apply_trigger:
  311267:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  31126c:	48 89 c1             	mov    %rax,%rcx
  31126f:	48 89 c2             	mov    %rax,%rdx
  311272:	4d 29 fc             	sub    %r15,%r12
rexx_exec::parse_template::find:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:298
  311275:	0f 82 b5 04 00 00    	jb     311730 <<rexx_exec::Interp>::exec_parse+0x1e00>
  31127b:	48 89 c1             	mov    %rax,%rcx
  31127e:	48 89 c2             	mov    %rax,%rdx
  311281:	48 85 db             	test   %rbx,%rbx
  311284:	0f 84 a6 04 00 00    	je     311730 <<rexx_exec::Interp>::exec_parse+0x1e00>
  31128a:	48 89 c1             	mov    %rax,%rcx
  31128d:	48 89 c2             	mov    %rax,%rdx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:305
  311290:	4c 39 e3             	cmp    %r12,%rbx
  311293:	0f 87 97 04 00 00    	ja     311730 <<rexx_exec::Interp>::exec_parse+0x1e00>
  311299:	4d 01 fd             	add    %r15,%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:316
  31129c:	48 83 fb 01          	cmp    $0x1,%rbx
  3112a0:	75 52                	jne    3112f4 <<rexx_exec::Interp>::exec_parse+0x19c4>
  3112a2:	41 0f b6 0e          	movzbl (%r14),%ecx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  3112a6:	8d 41 bf             	lea    -0x41(%rcx),%eax
  3112a9:	3c 1a                	cmp    $0x1a,%al
  3112ab:	0f 92 c0             	setb   %al
  3112ae:	c0 e0 05             	shl    $0x5,%al
  3112b1:	08 c8                	or     %cl,%al
  3112b3:	45 31 c0             	xor    %r8d,%r8d
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::parse_template::find::{closure#0}>:
  3112b6:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
  3112bd:	00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  3112c0:	43 0f b6 4c 05 00    	movzbl 0x0(%r13,%r8,1),%ecx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  3112c6:	8d 51 bf             	lea    -0x41(%rcx),%edx
  3112c9:	80 fa 1a             	cmp    $0x1a,%dl
  3112cc:	0f 92 c2             	setb   %dl
  3112cf:	c0 e2 05             	shl    $0x5,%dl
  3112d2:	08 ca                	or     %cl,%dl
rexx_exec::parse_template::find::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:320
  3112d4:	38 c2                	cmp    %al,%dl
  3112d6:	0f 84 48 04 00 00    	je     311724 <<rexx_exec::Interp>::exec_parse+0x1df4>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::parse_template::find::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:385
  3112dc:	49 ff c0             	inc    %r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  3112df:	4d 39 c4             	cmp    %r8,%r12
  3112e2:	75 dc                	jne    3112c0 <<rexx_exec::Interp>::exec_parse+0x1990>
rexx_exec::parse_template::find:
  3112e4:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  3112e9:	48 89 c1             	mov    %rax,%rcx
  3112ec:	48 89 c2             	mov    %rax,%rdx
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  3112ef:	e9 3c 04 00 00       	jmp    311730 <<rexx_exec::Interp>::exec_parse+0x1e00>
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::position::check<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
  3112f4:	48 83 fb 0f          	cmp    $0xf,%rbx
  3112f8:	0f 86 b8 03 00 00    	jbe    3116b6 <<rexx_exec::Interp>::exec_parse+0x1d86>
  3112fe:	31 ed                	xor    %ebp,%ebp
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  311300:	4c 89 ef             	mov    %r13,%rdi
  311303:	48 01 ef             	add    %rbp,%rdi
<[u8]>::eq_ignore_ascii_case:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:76
  311306:	48 89 de             	mov    %rbx,%rsi
  311309:	4c 89 f2             	mov    %r14,%rdx
  31130c:	48 89 d9             	mov    %rbx,%rcx
  31130f:	e8 ac 61 e3 ff       	call   1474c0 <<[u8]>::eq_ignore_ascii_case_chunks::<16>>
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3142
  311314:	84 c0                	test   %al,%al
  311316:	0f 85 00 04 00 00    	jne    31171c <<rexx_exec::Interp>::exec_parse+0x1dec>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  31131c:	49 ff cc             	dec    %r12
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  31131f:	48 ff c5             	inc    %rbp
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  311322:	4c 39 e3             	cmp    %r12,%rbx
  311325:	76 d9                	jbe    311300 <<rexx_exec::Interp>::exec_parse+0x19d0>
<rexx_exec::Interp>::apply_trigger:
  311327:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  31132c:	48 89 c1             	mov    %rax,%rcx
  31132f:	48 89 c2             	mov    %rax,%rdx
  311332:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  311337:	e9 f4 03 00 00       	jmp    311730 <<rexx_exec::Interp>::exec_parse+0x1e00>
  31133c:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311341:	49 8b 9c 24 f0 0a 00 	mov    0xaf0(%r12),%rbx
  311348:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311349:	49 3b 9c 24 e0 0a 00 	cmp    0xae0(%r12),%rbx
  311350:	00 
  311351:	75 0d                	jne    311360 <<rexx_exec::Interp>::exec_parse+0x1a30>
  311353:	48 8b bc 24 40 01 00 	mov    0x140(%rsp),%rdi
  31135a:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31135b:	e8 20 ac e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311360:	49 8b 84 24 e8 0a 00 	mov    0xae8(%r12),%rax
  311367:	00 
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  311368:	4c 89 34 d8          	mov    %r14,(%rax,%rbx,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  31136c:	48 ff c3             	inc    %rbx
  31136f:	49 89 9c 24 f0 0a 00 	mov    %rbx,0xaf0(%r12)
  311376:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  311377:	44 89 f0             	mov    %r14d,%eax
  31137a:	83 e0 03             	and    $0x3,%eax
  31137d:	48 8d 0d 28 31 d4 ff 	lea    -0x2bced8(%rip),%rcx        # 544ac <GCC_except_table5014+0x4a98>
  311384:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  311388:	48 01 c8             	add    %rcx,%rax
  31138b:	4d 89 f5             	mov    %r14,%r13
  31138e:	4d 89 e6             	mov    %r12,%r14
  311391:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  311396:	ff e0                	jmp    *%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:167
  311398:	4c 89 ee             	mov    %r13,%rsi
  31139b:	48 c1 fe 02          	sar    $0x2,%rsi
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:942
  31139f:	66 0f ef c0          	pxor   %xmm0,%xmm0
  3113a3:	66 0f 7f 84 24 90 01 	movdqa %xmm0,0x190(%rsp)
  3113aa:	00 00 
  3113ac:	c7 84 24 a0 01 00 00 	movl   $0x0,0x1a0(%rsp)
  3113b3:	00 00 00 00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:943
  3113b7:	48 8d bc 24 90 01 00 	lea    0x190(%rsp),%rdi
  3113be:	00 
  3113bf:	e8 1c ec e9 ff       	call   1affe0 <rexx_exec::value::write_small_int>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:944
  3113c4:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  3113c9:	c6 44 24 48 03       	movb   $0x3,0x48(%rsp)
  3113ce:	88 44 24 49          	mov    %al,0x49(%rsp)
  3113d2:	8b 84 24 a0 01 00 00 	mov    0x1a0(%rsp),%eax
  3113d9:	48 8d 4c 24 4a       	lea    0x4a(%rsp),%rcx
  3113de:	89 41 10             	mov    %eax,0x10(%rcx)
  3113e1:	66 0f 6f 84 24 90 01 	movdqa 0x190(%rsp),%xmm0
  3113e8:	00 00 
  3113ea:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  3113ee:	b0 03                	mov    $0x3,%al
  3113f0:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  3113f7:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  3113f8:	48 85 c9             	test   %rcx,%rcx
  3113fb:	0f 85 39 f9 ff ff    	jne    310d3a <<rexx_exec::Interp>::exec_parse+0x140a>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311401:	48 8b 8c 24 d8 00 00 	mov    0xd8(%rsp),%rcx
  311408:	00 
  311409:	48 89 4c 24 20       	mov    %rcx,0x20(%rsp)
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  31140e:	4c 8b a4 24 e0 00 00 	mov    0xe0(%rsp),%r12
  311415:	00 
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1690
  311416:	0f b6 c0             	movzbl %al,%eax
  311419:	48 8d 0d 9c 30 d4 ff 	lea    -0x2bcf64(%rip),%rcx        # 544bc <GCC_except_table5014+0x4aa8>
  311420:	48 63 04 81          	movslq (%rcx,%rax,4),%rax
  311424:	48 01 c8             	add    %rcx,%rax
  311427:	ff e0                	jmp    *%rax
  311429:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1695
  31142e:	4c 89 ee             	mov    %r13,%rsi
  311431:	e8 da d8 ec ff       	call   1ded10 <<rexx_exec::Interp>::try_text>
  311436:	49 89 c5             	mov    %rax,%r13
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  311439:	48 85 c0             	test   %rax,%rax
  31143c:	0f 84 e9 22 00 00    	je     31372b <<rexx_exec::Interp>::exec_parse+0x3dfb>
<rexx_exec::value::Rendered>::text:
  311442:	48 89 d3             	mov    %rdx,%rbx
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:967
  311445:	eb 7c                	jmp    3114c3 <<rexx_exec::Interp>::exec_parse+0x1b93>
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:169
  311447:	44 89 e8             	mov    %r13d,%eax
  31144a:	c1 e8 02             	shr    $0x2,%eax
  31144d:	83 e0 07             	and    $0x7,%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:171
  311450:	4c 89 e9             	mov    %r13,%rcx
  311453:	48 81 e1 00 ff ff ff 	and    $0xffffffffffffff00,%rcx
  31145a:	48 09 c1             	or     %rax,%rcx
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:931
  31145d:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  311462:	c6 44 24 48 02       	movb   $0x2,0x48(%rsp)
  311467:	48 89 4c 24 49       	mov    %rcx,0x49(%rsp)
  31146c:	b0 02                	mov    $0x2,%al
  31146e:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  311475:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  311476:	48 85 c9             	test   %rcx,%rcx
  311479:	0f 85 bb f8 ff ff    	jne    310d3a <<rexx_exec::Interp>::exec_parse+0x140a>
  31147f:	eb 80                	jmp    311401 <<rexx_exec::Interp>::exec_parse+0x1ad1>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  311481:	0f b6 5c 24 49       	movzbl 0x49(%rsp),%ebx
  311486:	48 83 fb 07          	cmp    $0x7,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  31148a:	0f 87 41 22 00 00    	ja     3136d1 <<rexx_exec::Interp>::exec_parse+0x3da1>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
  311490:	4c 8d 6c 24 4a       	lea    0x4a(%rsp),%r13
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:86
  311495:	eb 2c                	jmp    3114c3 <<rexx_exec::Interp>::exec_parse+0x1b93>
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  311497:	44 0f b6 6c 24 49    	movzbl 0x49(%rsp),%r13d
  31149d:	49 83 fd 14          	cmp    $0x14,%r13
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:544
  3114a1:	0f 87 be 22 00 00    	ja     313765 <<rexx_exec::Interp>::exec_parse+0x3e35>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  3114a7:	bb 14 00 00 00       	mov    $0x14,%ebx
  3114ac:	4c 29 eb             	sub    %r13,%rbx
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  3114af:	48 8d 44 24 4a       	lea    0x4a(%rsp),%rax
  3114b4:	49 01 c5             	add    %rax,%r13
<rexx_exec::value::Rendered>::text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:1693
  3114b7:	eb 0a                	jmp    3114c3 <<rexx_exec::Interp>::exec_parse+0x1b93>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3114b9:	4c 8b 6c 24 58       	mov    0x58(%rsp),%r13
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  3114be:	48 8b 5c 24 60       	mov    0x60(%rsp),%rbx
<rexx_exec::Interp>::apply_trigger:
  3114c3:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  3114c8:	48 89 c7             	mov    %rax,%rdi
  3114cb:	49 89 c0             	mov    %rax,%r8
  3114ce:	4d 29 fc             	sub    %r15,%r12
rexx_exec::parse_template::find:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:298
  3114d1:	0f 82 c7 01 00 00    	jb     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
  3114d7:	48 89 c7             	mov    %rax,%rdi
  3114da:	49 89 c0             	mov    %rax,%r8
  3114dd:	48 85 db             	test   %rbx,%rbx
  3114e0:	0f 84 b8 01 00 00    	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
  3114e6:	48 89 c7             	mov    %rax,%rdi
  3114e9:	49 89 c0             	mov    %rax,%r8
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:305
  3114ec:	4c 39 e3             	cmp    %r12,%rbx
  3114ef:	0f 87 a9 01 00 00    	ja     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
  3114f5:	4c 01 7c 24 20       	add    %r15,0x20(%rsp)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:316
  3114fa:	48 83 fb 01          	cmp    $0x1,%rbx
  3114fe:	0f 85 93 00 00 00    	jne    311597 <<rexx_exec::Interp>::exec_parse+0x1c67>
  311504:	41 0f b6 4d 00       	movzbl 0x0(%r13),%ecx
<[u8]>::as_chunks::<8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:1399
  311509:	48 b8 01 00 00 00 00 	movabs $0x8000000000000001,%rax
  311510:	00 00 80 
  311513:	48 83 c0 f7          	add    $0xfffffffffffffff7,%rax
  311517:	4c 21 e0             	and    %r12,%rax
<[u8]>::split_at_unchecked:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/mod.rs:2062
  31151a:	44 89 e2             	mov    %r12d,%edx
  31151d:	83 e2 07             	and    $0x7,%edx
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:160
  311520:	48 89 cf             	mov    %rcx,%rdi
  311523:	48 be 01 01 01 01 01 	movabs $0x101010101010101,%rsi
  31152a:	01 01 01 
  31152d:	48 0f af fe          	imul   %rsi,%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:161
  311531:	49 83 e4 f8          	and    $0xfffffffffffffff8,%r12
  311535:	48 c7 c6 ff ff ff ff 	mov    $0xffffffffffffffff,%rsi
<core::ptr::non_null::NonNull<[u8; 8]> as core::cmp::PartialEq>::eq:
  31153c:	0f 1f 40 00          	nopl   0x0(%rax)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  311540:	4d 85 e4             	test   %r12,%r12
<core::slice::iter::Iter<[u8; 8]> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  311543:	0f 84 25 01 00 00    	je     31166e <<rexx_exec::Interp>::exec_parse+0x1d3e>
rexx_exec::builtin::string::find_byte:
  311549:	4c 8b 44 24 20       	mov    0x20(%rsp),%r8
  31154e:	4d 8b 44 f0 08       	mov    0x8(%r8,%rsi,8),%r8
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:162
  311553:	49 31 f8             	xor    %rdi,%r8
  311556:	4d 89 c1             	mov    %r8,%r9
  311559:	49 f7 d1             	not    %r9
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:163
  31155c:	49 ba 80 80 80 80 80 	movabs $0x8080808080808080,%r10
  311563:	80 80 80 
  311566:	4d 21 d1             	and    %r10,%r9
<u64>::wrapping_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2689
  311569:	49 ba ff fe fe fe fe 	movabs $0xfefefefefefefeff,%r10
  311570:	fe fe fe 
  311573:	4d 01 d0             	add    %r10,%r8
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:164
  311576:	48 ff c6             	inc    %rsi
  311579:	49 83 c4 f8          	add    $0xfffffffffffffff8,%r12
  31157d:	4d 21 c8             	and    %r9,%r8
  311580:	74 be                	je     311540 <<rexx_exec::Interp>::exec_parse+0x1c10>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:165
  311582:	48 c1 e6 03          	shl    $0x3,%rsi
<u64>::trailing_zeros:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:178
  311586:	f3 4d 0f bc f0       	tzcnt  %r8,%r14
rexx_exec::builtin::string::find_byte:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:165
  31158b:	41 c1 ee 03          	shr    $0x3,%r14d
  31158f:	49 09 f6             	or     %rsi,%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:171
  311592:	e9 fb 00 00 00       	jmp    311692 <<rexx_exec::Interp>::exec_parse+0x1d62>
rexx_exec::parse_template::find:
  311597:	45 31 f6             	xor    %r14d,%r14d
  31159a:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
  3115a0:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
<u8 as core::slice::cmp::SlicePartialEq<u8>>::equal_same_length:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/cmp.rs:157
  3115a5:	4a 8d 3c 30          	lea    (%rax,%r14,1),%rdi
  3115a9:	4c 89 ee             	mov    %r13,%rsi
  3115ac:	48 89 da             	mov    %rbx,%rdx
  3115af:	ff 15 f3 6b 1c 00    	call   *0x1c6bf3(%rip)        # 4d81a8 <bcmp@GLIBC_2.2.5>
  3115b5:	85 c0                	test   %eax,%eax
  3115b7:	0f 84 d5 00 00 00    	je     311692 <<rexx_exec::Interp>::exec_parse+0x1d62>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  3115bd:	49 ff cc             	dec    %r12
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#1}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  3115c0:	49 ff c6             	inc    %r14
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  3115c3:	4c 39 e3             	cmp    %r12,%rbx
  3115c6:	76 d8                	jbe    3115a0 <<rexx_exec::Interp>::exec_parse+0x1c70>
  3115c8:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  3115cd:	48 89 c7             	mov    %rax,%rdi
  3115d0:	49 89 c0             	mov    %rax,%r8
  3115d3:	e9 c6 00 00 00       	jmp    31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<rexx_exec::Interp>::apply_trigger:
  3115d8:	31 c0                	xor    %eax,%eax
  3115da:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  3115df:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:964
  3115e4:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  3115e9:	88 44 24 48          	mov    %al,0x48(%rsp)
  3115ed:	f3 0f 6f 84 24 00 01 	movdqu 0x100(%rsp),%xmm0
  3115f4:	00 00 
  3115f6:	f3 0f 6f 8c 24 0f 01 	movdqu 0x10f(%rsp),%xmm1
  3115fd:	00 00 
  3115ff:	48 8d 4c 24 49       	lea    0x49(%rsp),%rcx
  311604:	f3 0f 7f 49 0f       	movdqu %xmm1,0xf(%rcx)
  311609:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  31160d:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  311614:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  311615:	48 85 c9             	test   %rcx,%rcx
  311618:	0f 85 90 f6 ff ff    	jne    310cae <<rexx_exec::Interp>::exec_parse+0x137e>
  31161e:	e9 87 fb ff ff       	jmp    3111aa <<rexx_exec::Interp>::exec_parse+0x187a>
<rexx_exec::Interp>::apply_trigger:
  311623:	31 c0                	xor    %eax,%eax
  311625:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
  31162a:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
<rexx_exec::Interp>::render:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:964
  31162f:	4c 89 6c 24 68       	mov    %r13,0x68(%rsp)
  311634:	88 44 24 48          	mov    %al,0x48(%rsp)
  311638:	f3 0f 6f 84 24 00 01 	movdqu 0x100(%rsp),%xmm0
  31163f:	00 00 
  311641:	f3 0f 6f 8c 24 0f 01 	movdqu 0x10f(%rsp),%xmm1
  311648:	00 00 
  31164a:	48 8d 4c 24 49       	lea    0x49(%rsp),%rcx
  31164f:	f3 0f 7f 49 0f       	movdqu %xmm1,0xf(%rcx)
  311654:	f3 0f 7f 01          	movdqu %xmm0,(%rcx)
  311658:	48 8b 8c 24 50 01 00 	mov    0x150(%rsp),%rcx
  31165f:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  311660:	48 85 c9             	test   %rcx,%rcx
  311663:	0f 85 d1 f6 ff ff    	jne    310d3a <<rexx_exec::Interp>::exec_parse+0x140a>
  311669:	e9 93 fd ff ff       	jmp    311401 <<rexx_exec::Interp>::exec_parse+0x1ad1>
<rexx_exec::Interp>::apply_trigger:
  31166e:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  311673:	48 89 f7             	mov    %rsi,%rdi
  311676:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  311679:	48 85 d2             	test   %rdx,%rdx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31167c:	74 20                	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  31167e:	48 8b 74 24 20       	mov    0x20(%rsp),%rsi
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  311683:	38 0c 06             	cmp    %cl,(%rsi,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  311686:	0f 85 26 14 00 00    	jne    312ab2 <<rexx_exec::Interp>::exec_parse+0x3182>
rexx_exec::builtin::string::find_byte:
  31168c:	45 31 f6             	xor    %r14d,%r14d
<core::option::Option<usize>>::map::<usize, rexx_exec::builtin::string::find_byte::{closure#1}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  31168f:	49 09 c6             	or     %rax,%r14
rexx_exec::parse_template::find::{closure#3}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:327
  311692:	4d 01 fe             	add    %r15,%r14
<rexx_exec::parse_template::Cursor>::match_at:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:233
  311695:	4c 01 f3             	add    %r14,%rbx
  311698:	4c 89 f7             	mov    %r14,%rdi
  31169b:	49 89 d8             	mov    %rbx,%r8
<rexx_exec::Interp>::apply_trigger:
  31169e:	4c 89 c3             	mov    %r8,%rbx
  3116a1:	48 89 7c 24 28       	mov    %rdi,0x28(%rsp)
core::ptr::drop_glue::<rexx_exec::value::Carried>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3116a6:	80 7c 24 48 01       	cmpb   $0x1,0x48(%rsp)
  3116ab:	0f 84 8e 00 00 00    	je     31173f <<rexx_exec::Interp>::exec_parse+0x1e0f>
  3116b1:	e9 9c 00 00 00       	jmp    311752 <<rexx_exec::Interp>::exec_parse+0x1e22>
<rexx_exec::Interp>::apply_trigger:
  3116b6:	45 31 c0             	xor    %r8d,%r8d
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:549
  3116b9:	49 ff cc             	dec    %r12
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  3116bc:	49 8d 45 01          	lea    0x1(%r13),%rax
  3116c0:	31 c9                	xor    %ecx,%ecx
<[u8]>::eq_ignore_ascii_case_simple:
  3116c2:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  3116c9:	1f 84 00 00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:93
  3116d0:	48 39 cb             	cmp    %rcx,%rbx
  3116d3:	74 4f                	je     311724 <<rexx_exec::Interp>::exec_parse+0x1df4>
<u8>::to_ascii_lowercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:711
  3116d5:	41 0f b6 54 0d 00    	movzbl 0x0(%r13,%rcx,1),%edx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  3116db:	8d 72 bf             	lea    -0x41(%rdx),%esi
  3116de:	40 80 fe 1a          	cmp    $0x1a,%sil
  3116e2:	40 0f 92 c6          	setb   %sil
  3116e6:	40 c0 e6 05          	shl    $0x5,%sil
  3116ea:	40 08 d6             	or     %dl,%sil
<u8>::to_ascii_lowercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:711
  3116ed:	41 0f b6 14 0e       	movzbl (%r14,%rcx,1),%edx
<u8>::is_ascii_uppercase:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:858
  3116f2:	8d 7a bf             	lea    -0x41(%rdx),%edi
  3116f5:	40 80 ff 1a          	cmp    $0x1a,%dil
  3116f9:	40 0f 92 c7          	setb   %dil
  3116fd:	40 c0 e7 05          	shl    $0x5,%dil
  311701:	40 08 d7             	or     %dl,%dil
<[u8]>::eq_ignore_ascii_case_simple:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:94
  311704:	48 ff c1             	inc    %rcx
<u8>::eq_ignore_ascii_case:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/mod.rs:736
  311707:	40 38 fe             	cmp    %dil,%sil
<[u8]>::eq_ignore_ascii_case_simple:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/ascii.rs:94
  31170a:	74 c4                	je     3116d0 <<rexx_exec::Interp>::exec_parse+0x1da0>
core::iter::traits::iterator::Iterator::position::check::<&[u8], rexx_exec::parse_template::find::{closure#2}>::{closure#0}:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:3145
  31170c:	49 ff c0             	inc    %r8
  31170f:	49 89 c5             	mov    %rax,%r13
<core::slice::iter::Windows<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter.rs:1355
  311712:	4c 39 e3             	cmp    %r12,%rbx
  311715:	76 a2                	jbe    3116b9 <<rexx_exec::Interp>::exec_parse+0x1d89>
  311717:	e9 c8 fb ff ff       	jmp    3112e4 <<rexx_exec::Interp>::exec_parse+0x19b4>
rexx_exec::parse_template::find:
  31171c:	49 89 e8             	mov    %rbp,%r8
  31171f:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  311724:	4c 89 c1             	mov    %r8,%rcx
rexx_exec::parse_template::find::{closure#3}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:327
  311727:	4c 01 f9             	add    %r15,%rcx
<rexx_exec::parse_template::Cursor>::match_at:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:233
  31172a:	48 01 cb             	add    %rcx,%rbx
  31172d:	48 89 da             	mov    %rbx,%rdx
<rexx_exec::Interp>::apply_trigger:
  311730:	48 89 d3             	mov    %rdx,%rbx
  311733:	48 89 4c 24 28       	mov    %rcx,0x28(%rsp)
core::ptr::drop_glue::<rexx_exec::value::Carried>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  311738:	80 7c 24 48 01       	cmpb   $0x1,0x48(%rsp)
  31173d:	75 13                	jne    311752 <<rexx_exec::Interp>::exec_parse+0x1e22>
  31173f:	48 83 7c 24 50 00    	cmpq   $0x0,0x50(%rsp)
  311745:	74 0b                	je     311752 <<rexx_exec::Interp>::exec_parse+0x1e22>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  311747:	48 8b 7c 24 58       	mov    0x58(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31174c:	ff 15 16 6a 1c 00    	call   *0x1c6a16(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  311752:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
  311757:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  31175e:	00 
  31175f:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311764:	48 89 d8             	mov    %rbx,%rax
  311767:	48 89 84 24 b8 01 00 	mov    %rax,0x1b8(%rsp)
  31176e:	00 
<alloc::vec::Vec<core::option::Option<rexx_parse::ast::Expr>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  31176f:	48 8b 45 10          	mov    0x10(%rbp),%rax
<usize>::saturating_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:2540
  311773:	48 89 c2             	mov    %rax,%rdx
  311776:	48 83 ea 01          	sub    $0x1,%rdx
  31177a:	b9 00 00 00 00       	mov    $0x0,%ecx
  31177f:	48 0f 42 d1          	cmovb  %rcx,%rdx
  311783:	48 89 94 24 f8 00 00 	mov    %rdx,0xf8(%rsp)
  31178a:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::Expr>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31178b:	48 85 c0             	test   %rax,%rax
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31178e:	0f 84 4c f1 ff ff    	je     3108e0 <<rexx_exec::Interp>::exec_parse+0xfb0>
<rexx_exec::Interp>::assign_targets:
  311794:	4c 8b 6d 08          	mov    0x8(%rbp),%r13
  311798:	48 8d 04 80          	lea    (%rax,%rax,4),%rax
  31179c:	48 c1 e0 04          	shl    $0x4,%rax
  3117a0:	4c 01 e8             	add    %r13,%rax
  3117a3:	48 89 84 24 c8 01 00 	mov    %rax,0x1c8(%rsp)
  3117aa:	00 
<rexx_exec::Interp>::traced_mode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:111
  3117ab:	41 0f b6 8c 24 35 05 	movzbl 0x535(%r12),%ecx
  3117b2:	00 00 
  3117b4:	41 0f b6 84 24 2e 05 	movzbl 0x52e(%r12),%eax
  3117bb:	00 00 
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  3117bd:	80 f1 01             	xor    $0x1,%cl
  3117c0:	89 ca                	mov    %ecx,%edx
  3117c2:	20 c2                	and    %al,%dl
  3117c4:	41 22 8c 24 2d 05 00 	and    0x52d(%r12),%cl
  3117cb:	00 
  3117cc:	88 54 24 3f          	mov    %dl,0x3f(%rsp)
  3117d0:	08 ca                	or     %cl,%dl
  3117d2:	88 94 24 bf 00 00 00 	mov    %dl,0xbf(%rsp)
  3117d9:	80 f1 01             	xor    $0x1,%cl
  3117dc:	08 c1                	or     %al,%cl
  3117de:	88 8c 24 be 00 00 00 	mov    %cl,0xbe(%rsp)
  3117e5:	48 c7 44 24 20 00 00 	movq   $0x0,0x20(%rsp)
  3117ec:	00 00 
  3117ee:	eb 5a                	jmp    31184a <<rexx_exec::Interp>::exec_parse+0x1f1a>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3117f0:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  3117f7:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  3117f8:	4b 8d 0c 7f          	lea    (%r15,%r15,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  3117fc:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  311800:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  311807:	00 
  311808:	48 89 54 c8 08       	mov    %rdx,0x8(%rax,%rcx,8)
  31180d:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  311814:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  311816:	49 ff c7             	inc    %r15
  311819:	4d 89 bc 24 e0 0b 00 	mov    %r15,0xbe0(%r12)
  311820:	00 
<core::iter::adapters::enumerate::Enumerate<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>>> as core::iter::traits::iterator::Iterator>::next:
  311821:	66 66 66 66 66 66 2e 	data16 data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  311828:	0f 1f 84 00 00 00 00 
  31182f:	00 
  311830:	49 83 c5 50          	add    $0x50,%r13
  311834:	48 ff 44 24 20       	incq   0x20(%rsp)
  311839:	49 89 df             	mov    %rbx,%r15
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::Expr>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  31183c:	4c 3b ac 24 c8 01 00 	cmp    0x1c8(%rsp),%r13
  311843:	00 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::Expr>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  311844:	0f 84 99 f0 ff ff    	je     3108e3 <<rexx_exec::Interp>::exec_parse+0xfb3>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:742
  31184a:	49 83 7d 00 ff       	cmpq   $0xffffffffffffffff,0x0(%r13)
  31184f:	0f 84 bb 00 00 00    	je     311910 <<rexx_exec::Interp>::exec_parse+0x1fe0>
  311855:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  31185c:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  31185d:	48 85 c0             	test   %rax,%rax
  311860:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  311865:	4c 89 6c 24 30       	mov    %r13,0x30(%rsp)
  31186a:	0f 84 f0 00 00 00    	je     311960 <<rexx_exec::Interp>::exec_parse+0x2030>
  311870:	48 83 f8 01          	cmp    $0x1,%rax
  311874:	0f 85 b6 01 00 00    	jne    311a30 <<rexx_exec::Interp>::exec_parse+0x2100>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  31187a:	0f b6 84 24 d8 00 00 	movzbl 0xd8(%rsp),%eax
  311881:	00 
  311882:	4c 8d ac 24 d9 00 00 	lea    0xd9(%rsp),%r13
  311889:	00 
  31188a:	48 83 f8 08          	cmp    $0x8,%rax
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  31188e:	0f 83 2a 19 00 00    	jae    3131be <<rexx_exec::Interp>::exec_parse+0x388e>
<rexx_exec::Interp>::assign_targets:
  311894:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
  311899:	48 3b 8c 24 f8 00 00 	cmp    0xf8(%rsp),%rcx
  3118a0:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  3118a1:	0f 85 dc 00 00 00    	jne    311983 <<rexx_exec::Interp>::exec_parse+0x2053>
  3118a7:	48 8b 4c 24 28       	mov    0x28(%rsp),%rcx
  3118ac:	48 89 cb             	mov    %rcx,%rbx
  3118af:	49 89 ce             	mov    %rcx,%r14
  3118b2:	49 39 cf             	cmp    %rcx,%r15
<rexx_exec::parse_template::Cursor>::remainder:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:286
  3118b5:	0f 83 35 04 00 00    	jae    311cf0 <<rexx_exec::Interp>::exec_parse+0x23c0>
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  3118bb:	4d 89 f4             	mov    %r14,%r12
  3118be:	4d 29 fc             	sub    %r15,%r12
  3118c1:	0f 82 e7 17 00 00    	jb     3130ae <<rexx_exec::Interp>::exec_parse+0x377e>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:408
  3118c7:	49 39 c6             	cmp    %rax,%r14
  3118ca:	0f 87 de 17 00 00    	ja     3130ae <<rexx_exec::Interp>::exec_parse+0x377e>
core::slice::index::get_offset_len_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:90
  3118d0:	4d 01 fd             	add    %r15,%r13
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:131
  3118d3:	49 83 fc 07          	cmp    $0x7,%r12
  3118d7:	0f 86 1e 02 00 00    	jbe    311afb <<rexx_exec::Interp>::exec_parse+0x21cb>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:51
  3118dd:	49 83 fc 37          	cmp    $0x37,%r12
  3118e1:	0f 83 f3 02 00 00    	jae    311bda <<rexx_exec::Interp>::exec_parse+0x22aa>
<core::mem::maybe_uninit::MaybeUninit<u8>>::write:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/maybe_uninit.rs:575
  3118e7:	48 8d 7c 24 52       	lea    0x52(%rsp),%rdi
  3118ec:	4c 89 ee             	mov    %r13,%rsi
  3118ef:	4c 89 e2             	mov    %r12,%rdx
  3118f2:	ff 15 a0 68 1c 00    	call   *0x1c68a0(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:61
  3118f8:	c6 44 24 50 00       	movb   $0x0,0x50(%rsp)
  3118fd:	44 88 64 24 51       	mov    %r12b,0x51(%rsp)
  311902:	e9 0b 03 00 00       	jmp    311c12 <<rexx_exec::Interp>::exec_parse+0x22e2>
<rexx_exec::Interp>::assign_targets:
  311907:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  31190e:	00 00 
  311910:	48 8b 44 24 20       	mov    0x20(%rsp),%rax
  311915:	48 3b 84 24 f8 00 00 	cmp    0xf8(%rsp),%rax
  31191c:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:798
  31191d:	0f 85 92 01 00 00    	jne    311ab5 <<rexx_exec::Interp>::exec_parse+0x2185>
  311923:	48 8b 44 24 28       	mov    0x28(%rsp),%rax
<rexx_exec::parse_template::Cursor>::remainder:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:286
  311928:	49 39 c7             	cmp    %rax,%r15
  31192b:	48 89 c3             	mov    %rax,%rbx
  31192e:	49 0f 47 df          	cmova  %r15,%rbx
  311932:	41 b8 00 00 00 00    	mov    $0x0,%r8d
  311938:	4c 0f 42 c0          	cmovb  %rax,%r8
  31193c:	b8 00 00 00 00       	mov    $0x0,%eax
  311941:	4c 0f 43 f8          	cmovae %rax,%r15
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  311945:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  31194a:	0f 84 e0 fe ff ff    	je     311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
  311950:	e9 9f 07 00 00       	jmp    3120f4 <<rexx_exec::Interp>::exec_parse+0x27c4>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
  311955:	66 66 2e 0f 1f 84 00 	data16 cs nopw 0x0(%rax,%rax,1)
  31195c:	00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311960:	4c 8b ac 24 d8 00 00 	mov    0xd8(%rsp),%r13
  311967:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311968:	48 8b 84 24 e0 00 00 	mov    0xe0(%rsp),%rax
  31196f:	00 
  311970:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
<rexx_exec::Interp>::assign_targets:
  311975:	48 3b 8c 24 f8 00 00 	cmp    0xf8(%rsp),%rcx
  31197c:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  31197d:	0f 84 24 ff ff ff    	je     3118a7 <<rexx_exec::Interp>::exec_parse+0x1f77>
  311983:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311988:	0f 83 62 03 00 00    	jae    311cf0 <<rexx_exec::Interp>::exec_parse+0x23c0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  31198e:	49 39 f7             	cmp    %rsi,%r15
  311991:	73 32                	jae    3119c5 <<rexx_exec::Interp>::exec_parse+0x2095>
  311993:	49 39 c7             	cmp    %rax,%r15
  311996:	48 89 c7             	mov    %rax,%rdi
  311999:	49 0f 47 ff          	cmova  %r15,%rdi
  31199d:	eb 0d                	jmp    3119ac <<rexx_exec::Interp>::exec_parse+0x207c>
  31199f:	90                   	nop
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:257
  3119a0:	49 ff c7             	inc    %r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  3119a3:	4c 39 fe             	cmp    %r15,%rsi
  3119a6:	0f 84 36 03 00 00    	je     311ce2 <<rexx_exec::Interp>::exec_parse+0x23b2>
  3119ac:	4c 39 ff             	cmp    %r15,%rdi
  3119af:	0f 84 fa 1d 00 00    	je     3137af <<rexx_exec::Interp>::exec_parse+0x3e7f>
  3119b5:	43 0f b6 4c 3d 00    	movzbl 0x0(%r13,%r15,1),%ecx
  3119bb:	83 f9 20             	cmp    $0x20,%ecx
rexx_exec::parse_template::is_blank:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:31
  3119be:	74 e0                	je     3119a0 <<rexx_exec::Interp>::exec_parse+0x2070>
  3119c0:	83 f9 09             	cmp    $0x9,%ecx
  3119c3:	74 db                	je     3119a0 <<rexx_exec::Interp>::exec_parse+0x2070>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  3119c5:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  3119ca:	0f 83 20 03 00 00    	jae    311cf0 <<rexx_exec::Interp>::exec_parse+0x23c0>
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2489
  3119d0:	49 39 c7             	cmp    %rax,%r15
  3119d3:	48 89 c7             	mov    %rax,%rdi
  3119d6:	49 0f 47 ff          	cmova  %r15,%rdi
  3119da:	4c 89 fb             	mov    %r15,%rbx
  3119dd:	4c 8b 74 24 28       	mov    0x28(%rsp),%r14
<usize as core::cmp::PartialOrd>::lt:
  3119e2:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  3119e9:	1f 84 00 00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2144
  3119f0:	49 39 de             	cmp    %rbx,%r14
<core::ops::range::Range<usize> as core::iter::range::RangeIteratorImpl>::spec_next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1100
  3119f3:	0f 84 45 08 00 00    	je     31223e <<rexx_exec::Interp>::exec_parse+0x290e>
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  3119f9:	48 39 df             	cmp    %rbx,%rdi
  3119fc:	0f 84 9e 1d 00 00    	je     3137a0 <<rexx_exec::Interp>::exec_parse+0x3e70>
  311a02:	41 0f b6 4c 1d 00    	movzbl 0x0(%r13,%rbx,1),%ecx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  311a08:	48 ff c3             	inc    %rbx
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  311a0b:	83 f9 09             	cmp    $0x9,%ecx
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  311a0e:	74 05                	je     311a15 <<rexx_exec::Interp>::exec_parse+0x20e5>
  311a10:	83 f9 20             	cmp    $0x20,%ecx
  311a13:	75 db                	jne    3119f0 <<rexx_exec::Interp>::exec_parse+0x20c0>
  311a15:	4c 8d 73 ff          	lea    -0x1(%rbx),%r14
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311a19:	4d 89 f4             	mov    %r14,%r12
  311a1c:	4d 29 fc             	sub    %r15,%r12
  311a1f:	0f 83 a2 fe ff ff    	jae    3118c7 <<rexx_exec::Interp>::exec_parse+0x1f97>
  311a25:	e9 84 16 00 00       	jmp    3130ae <<rexx_exec::Interp>::exec_parse+0x377e>
<rexx_exec::parse_template::SourceText>::bytes:
  311a2a:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311a30:	48 8b 84 24 d8 00 00 	mov    0xd8(%rsp),%rax
  311a37:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  311a38:	a8 03                	test   $0x3,%al
  311a3a:	0f 85 43 1d 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  311a40:	48 89 c1             	mov    %rax,%rcx
  311a43:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311a47:	89 ca                	mov    %ecx,%edx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  311a49:	49 39 94 24 08 0a 00 	cmp    %rdx,0xa08(%r12)
  311a50:	00 
  311a51:	0f 86 2c 1d 00 00    	jbe    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311a57:	49 8b 8c 24 00 0a 00 	mov    0xa00(%r12),%rcx
  311a5e:	00 
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  311a5f:	48 8d 14 52          	lea    (%rdx,%rdx,2),%rdx
  311a63:	48 c1 e2 05          	shl    $0x5,%rdx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311a67:	83 3c 11 01          	cmpl   $0x1,(%rcx,%rdx,1)
  311a6b:	0f 85 12 1d 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
  311a71:	48 01 d1             	add    %rdx,%rcx
  311a74:	80 79 08 00          	cmpb   $0x0,0x8(%rcx)
  311a78:	0f 85 05 1d 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
  311a7e:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  311a82:	39 41 04             	cmp    %eax,0x4(%rcx)
  311a85:	0f 85 f8 1c 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  311a8b:	80 79 10 01          	cmpb   $0x1,0x10(%rcx)
  311a8f:	0f 85 85 00 00 00    	jne    311b1a <<rexx_exec::Interp>::exec_parse+0x21ea>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311a95:	4c 8b 69 20          	mov    0x20(%rcx),%r13
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311a99:	48 8b 41 28          	mov    0x28(%rcx),%rax
  311a9d:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
<rexx_exec::Interp>::assign_targets:
  311aa2:	48 3b 8c 24 f8 00 00 	cmp    0xf8(%rsp),%rcx
  311aa9:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  311aaa:	0f 85 d3 fe ff ff    	jne    311983 <<rexx_exec::Interp>::exec_parse+0x2053>
  311ab0:	e9 f2 fd ff ff       	jmp    3118a7 <<rexx_exec::Interp>::exec_parse+0x1f77>
<rexx_exec::Interp>::assign_targets:
  311ab5:	48 8b 84 24 50 01 00 	mov    0x150(%rsp),%rax
  311abc:	00 
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:73
  311abd:	48 85 c0             	test   %rax,%rax
  311ac0:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  311ac5:	74 76                	je     311b3d <<rexx_exec::Interp>::exec_parse+0x220d>
  311ac7:	48 83 f8 01          	cmp    $0x1,%rax
  311acb:	0f 85 8c 00 00 00    	jne    311b5d <<rexx_exec::Interp>::exec_parse+0x222d>
<rexx_core::handle::InlineText as core::ops::deref::Deref>::deref:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:85
  311ad1:	0f b6 84 24 d8 00 00 	movzbl 0xd8(%rsp),%eax
  311ad8:	00 
  311ad9:	48 8d 8c 24 d9 00 00 	lea    0xd9(%rsp),%rcx
  311ae0:	00 
  311ae1:	48 83 f8 08          	cmp    $0x8,%rax
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  311ae5:	0f 83 d3 16 00 00    	jae    3131be <<rexx_exec::Interp>::exec_parse+0x388e>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311aeb:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  311af0:	0f 83 ea 05 00 00    	jae    3120e0 <<rexx_exec::Interp>::exec_parse+0x27b0>
  311af6:	e9 4f 05 00 00       	jmp    31204a <<rexx_exec::Interp>::exec_parse+0x271a>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:134
  311afb:	48 c7 44 24 48 00 00 	movq   $0x0,0x48(%rsp)
  311b02:	00 00 
  311b04:	bd 03 00 00 00       	mov    $0x3,%ebp
<core::iter::adapters::zip::Zip<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>> as core::iter::adapters::zip::ZipImpl<core::slice::iter::IterMut<u8>, core::slice::iter::Iter<u8>>>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/adapters/zip.rs:300
  311b09:	4d 39 fe             	cmp    %r15,%r14
  311b0c:	0f 85 8c 01 00 00    	jne    311c9e <<rexx_exec::Interp>::exec_parse+0x236e>
<rexx_exec::Interp>::take_piece:
  311b12:	4d 89 fe             	mov    %r15,%r14
  311b15:	e9 a6 01 00 00       	jmp    311cc0 <<rexx_exec::Interp>::exec_parse+0x2390>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  311b1a:	0f b6 41 11          	movzbl 0x11(%rcx),%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  311b1e:	48 83 c1 12          	add    $0x12,%rcx
  311b22:	49 89 cd             	mov    %rcx,%r13
  311b25:	48 8b 4c 24 20       	mov    0x20(%rsp),%rcx
<rexx_exec::Interp>::assign_targets:
  311b2a:	48 3b 8c 24 f8 00 00 	cmp    0xf8(%rsp),%rcx
  311b31:	00 
<rexx_exec::Interp>::take_piece:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:441
  311b32:	0f 84 6f fd ff ff    	je     3118a7 <<rexx_exec::Interp>::exec_parse+0x1f77>
  311b38:	e9 46 fe ff ff       	jmp    311983 <<rexx_exec::Interp>::exec_parse+0x2053>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311b3d:	48 8b 8c 24 d8 00 00 	mov    0xd8(%rsp),%rcx
  311b44:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311b45:	48 8b 84 24 e0 00 00 	mov    0xe0(%rsp),%rax
  311b4c:	00 
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311b4d:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  311b52:	0f 82 f2 04 00 00    	jb     31204a <<rexx_exec::Interp>::exec_parse+0x271a>
  311b58:	e9 83 05 00 00       	jmp    3120e0 <<rexx_exec::Interp>::exec_parse+0x27b0>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311b5d:	48 8b 84 24 d8 00 00 	mov    0xd8(%rsp),%rax
  311b64:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:162
  311b65:	a8 03                	test   $0x3,%al
  311b67:	0f 85 16 1c 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  311b6d:	48 89 c1             	mov    %rax,%rcx
  311b70:	48 c1 e9 02          	shr    $0x2,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311b74:	89 c9                	mov    %ecx,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  311b76:	49 39 8c 24 08 0a 00 	cmp    %rcx,0xa08(%r12)
  311b7d:	00 
  311b7e:	0f 86 ff 1b 00 00    	jbe    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:76
  311b84:	49 8b 94 24 00 0a 00 	mov    0xa00(%r12),%rdx
  311b8b:	00 
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  311b8c:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  311b90:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  311b94:	83 3c 0a 01          	cmpl   $0x1,(%rdx,%rcx,1)
  311b98:	0f 85 e5 1b 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
  311b9e:	48 01 ca             	add    %rcx,%rdx
  311ba1:	80 7a 08 00          	cmpb   $0x0,0x8(%rdx)
  311ba5:	0f 85 d8 1b 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
  311bab:	48 c1 e8 22          	shr    $0x22,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  311baf:	39 42 04             	cmp    %eax,0x4(%rdx)
  311bb2:	0f 85 cb 1b 00 00    	jne    313783 <<rexx_exec::Interp>::exec_parse+0x3e53>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  311bb8:	80 7a 10 01          	cmpb   $0x1,0x10(%rdx)
  311bbc:	0f 85 72 04 00 00    	jne    312034 <<rexx_exec::Interp>::exec_parse+0x2704>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311bc2:	48 8b 4a 20          	mov    0x20(%rdx),%rcx
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311bc6:	48 8b 42 28          	mov    0x28(%rdx),%rax
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  311bca:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  311bcf:	0f 83 0b 05 00 00    	jae    3120e0 <<rexx_exec::Interp>::exec_parse+0x27b0>
  311bd5:	e9 70 04 00 00       	jmp    31204a <<rexx_exec::Interp>::exec_parse+0x271a>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  311bda:	4c 89 e7             	mov    %r12,%rdi
  311bdd:	ff 15 a5 65 1c 00    	call   *0x1c65a5(%rip)        # 4d8188 <malloc@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  311be3:	48 85 c0             	test   %rax,%rax
  311be6:	0f 84 8e 1c 00 00    	je     31387a <<rexx_exec::Interp>::exec_parse+0x3f4a>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  311bec:	48 89 c5             	mov    %rax,%rbp
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  311bef:	48 89 c7             	mov    %rax,%rdi
  311bf2:	4c 89 ee             	mov    %r13,%rsi
  311bf5:	4c 89 e2             	mov    %r12,%rdx
  311bf8:	ff 15 9a 65 1c 00    	call   *0x1c659a(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<rexx_core::bytes::Bytes>::from_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:66
  311bfe:	c6 44 24 50 01       	movb   $0x1,0x50(%rsp)
  311c03:	4c 89 64 24 58       	mov    %r12,0x58(%rsp)
  311c08:	48 89 6c 24 60       	mov    %rbp,0x60(%rsp)
  311c0d:	4c 89 64 24 68       	mov    %r12,0x68(%rsp)
<rexx_exec::Interp>::text_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:230
  311c12:	48 c7 84 24 88 00 00 	movq   $0x0,0x88(%rsp)
  311c19:	00 00 00 00 00 
  311c1e:	c6 44 24 48 00       	movb   $0x0,0x48(%rsp)
  311c23:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
  311c28:	31 f6                	xor    %esi,%esi
  311c2a:	48 8d 54 24 48       	lea    0x48(%rsp),%rdx
  311c2f:	e8 9c a6 e7 ff       	call   18c2d0 <<rexx_exec::Interp>::alloc_with>
  311c34:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
<rexx_exec::Interp>::take_piece:
  311c39:	48 89 c5             	mov    %rax,%rbp
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::size_hint:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1190
  311c3c:	4c 89 f1             	mov    %r14,%rcx
  311c3f:	4c 29 f9             	sub    %r15,%rcx
  311c42:	b8 00 00 00 00       	mov    $0x0,%eax
  311c47:	48 0f 43 c1          	cmovae %rcx,%rax
<rexx_exec::Interp>::charge_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/value.rs:104
  311c4b:	48 83 f8 37          	cmp    $0x37,%rax
  311c4f:	72 76                	jb     311cc7 <<rexx_exec::Interp>::exec_parse+0x2397>
  311c51:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_core::heap::Heap>::hold_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:183
  311c56:	49 01 84 24 a0 0a 00 	add    %rax,0xaa0(%r12)
  311c5d:	00 
<rexx_core::heap::Heap>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:176
  311c5e:	49 03 84 24 90 0a 00 	add    0xa90(%r12),%rax
  311c65:	00 
  311c66:	49 89 84 24 90 0a 00 	mov    %rax,0xa90(%r12)
  311c6d:	00 
<rexx_exec::Interp>::charge_body_bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:3249
  311c6e:	49 3b 84 24 c8 1f 00 	cmp    0x1fc8(%r12),%rax
  311c75:	00 
  311c76:	0f 93 c0             	setae  %al
  311c79:	41 08 84 24 ff 1f 00 	or     %al,0x1fff(%r12)
  311c80:	00 
  311c81:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311c84:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  311c8b:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311c8c:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  311c93:	0f 84 7e 00 00 00    	je     311d17 <<rexx_exec::Interp>::exec_parse+0x23e7>
  311c99:	e9 86 00 00 00       	jmp    311d24 <<rexx_exec::Interp>::exec_parse+0x23f4>
<rexx_core::handle::ObjRef>::inline_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:136
  311c9e:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  311ca3:	4c 89 ee             	mov    %r13,%rsi
  311ca6:	4c 89 e2             	mov    %r12,%rdx
  311ca9:	ff 15 e9 64 1c 00    	call   *0x1c64e9(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:138
  311caf:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:140
  311cb4:	48 c1 e0 08          	shl    $0x8,%rax
  311cb8:	4a 8d 2c a0          	lea    (%rax,%r12,4),%rbp
  311cbc:	48 83 c5 03          	add    $0x3,%rbp
<rexx_exec::Interp>::assign_targets:
  311cc0:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311cc5:	eb 37                	jmp    311cfe <<rexx_exec::Interp>::exec_parse+0x23ce>
  311cc7:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311ccc:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311ccf:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  311cd6:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311cd7:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  311cde:	74 37                	je     311d17 <<rexx_exec::Interp>::exec_parse+0x23e7>
  311ce0:	eb 42                	jmp    311d24 <<rexx_exec::Interp>::exec_parse+0x23f4>
<rexx_exec::Interp>::assign_targets:
  311ce2:	49 89 f7             	mov    %rsi,%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  311ce5:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  311cea:	0f 82 e0 fc ff ff    	jb     3119d0 <<rexx_exec::Interp>::exec_parse+0x20a0>
<rexx_exec::Interp>::assign_targets:
  311cf0:	bd 03 00 00 00       	mov    $0x3,%ebp
  311cf5:	4c 89 fb             	mov    %r15,%rbx
  311cf8:	45 31 ff             	xor    %r15d,%r15d
  311cfb:	45 31 f6             	xor    %r14d,%r14d
  311cfe:	4c 8b 6c 24 30       	mov    0x30(%rsp),%r13
  311d03:	4c 89 e0             	mov    %r12,%rax
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  311d06:	4d 8b a4 24 f0 0a 00 	mov    0xaf0(%r12),%r12
  311d0d:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  311d0e:	4c 3b a0 e0 0a 00 00 	cmp    0xae0(%rax),%r12
  311d15:	75 0d                	jne    311d24 <<rexx_exec::Interp>::exec_parse+0x23f4>
  311d17:	48 8b bc 24 40 01 00 	mov    0x140(%rsp),%rdi
  311d1e:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  311d1f:	e8 5c a2 e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  311d24:	48 8b 4c 24 18       	mov    0x18(%rsp),%rcx
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311d29:	48 8b 81 e8 0a 00 00 	mov    0xae8(%rcx),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  311d30:	4a 89 2c e0          	mov    %rbp,(%rax,%r12,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  311d34:	49 ff c4             	inc    %r12
  311d37:	4c 89 a1 f0 0a 00 00 	mov    %r12,0xaf0(%rcx)
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:760
  311d3e:	48 b8 01 00 00 00 00 	movabs $0x8000000000000001,%rax
  311d45:	00 00 80 
  311d48:	48 ff c0             	inc    %rax
  311d4b:	49 39 45 00          	cmp    %rax,0x0(%r13)
  311d4f:	0f 85 cb 00 00 00    	jne    311e20 <<rexx_exec::Interp>::exec_parse+0x24f0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:761
  311d55:	41 8b 45 08          	mov    0x8(%r13),%eax
<usize as core::slice::index::SliceIndex<[core::option::Option<usize>]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  311d59:	48 39 84 24 28 02 00 	cmp    %rax,0x228(%rsp)
  311d60:	00 
  311d61:	0f 86 b9 00 00 00    	jbe    311e20 <<rexx_exec::Interp>::exec_parse+0x24f0>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  311d67:	48 89 c1             	mov    %rax,%rcx
  311d6a:	48 c1 e1 04          	shl    $0x4,%rcx
  311d6e:	48 8b 94 24 18 02 00 	mov    0x218(%rsp),%rdx
  311d75:	00 
<core::option::Option<usize>>::map::<(rexx_parse::token::symbols::SymbolId, usize), <rexx_exec::Interp>::assign_targets::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:1162
  311d76:	83 3c 0a 01          	cmpl   $0x1,(%rdx,%rcx,1)
  311d7a:	0f 85 a0 00 00 00    	jne    311e20 <<rexx_exec::Interp>::exec_parse+0x24f0>
<rexx_exec::Code>::slot_for:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1263
  311d80:	48 01 d1             	add    %rdx,%rcx
<core::option::Option<&core::option::Option<usize>>>::copied:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:2138
  311d83:	48 8b 49 08          	mov    0x8(%rcx),%rcx
  311d87:	48 89 8c 24 c0 01 00 	mov    %rcx,0x1c0(%rsp)
  311d8e:	00 
  311d8f:	41 b4 01             	mov    $0x1,%r12b
  311d92:	89 84 24 dc 01 00 00 	mov    %eax,0x1dc(%rsp)
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:768
  311d99:	f6 84 24 bf 00 00 00 	testb  $0x1,0xbf(%rsp)
  311da0:	01 
  311da1:	0f 85 86 00 00 00    	jne    311e2d <<rexx_exec::Interp>::exec_parse+0x24fd>
  311da7:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  311dae:	00 00 
  311db0:	49 c7 c6 ff ff ff ff 	mov    $0xffffffffffffffff,%r14
  311db7:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  311dba:	45 84 e4             	test   %r12b,%r12b
  311dbd:	0f 84 c3 00 00 00    	je     311e86 <<rexx_exec::Interp>::exec_parse+0x2556>
  311dc3:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:73
  311dc8:	49 8b 84 24 70 04 00 	mov    0x470(%r12),%rax
  311dcf:	00 
<core::option::Option<alloc::boxed::Box<rexx_exec::activation::Activation>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311dd0:	48 85 c0             	test   %rax,%rax
  311dd3:	0f 84 4b 14 00 00    	je     313224 <<rexx_exec::Interp>::exec_parse+0x38f4>
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:73
  311dd9:	48 8b b0 f8 00 00 00 	mov    0xf8(%rax),%rsi
  311de0:	48 8b 90 00 01 00 00 	mov    0x100(%rax),%rdx
<alloc::vec::Vec<(usize, rexx_exec::activation::InstanceVar)>>::is_empty:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3126
  311de7:	48 83 78 50 00       	cmpq   $0x0,0x50(%rax)
<rexx_exec::Interp>::activation_exposes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:87
  311dec:	0f 84 e6 00 00 00    	je     311ed8 <<rexx_exec::Interp>::exec_parse+0x25a8>
<rexx_exec::Interp>::set_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/variables.rs:40
  311df2:	4c 89 e7             	mov    %r12,%rdi
  311df5:	48 8b 8c 24 c0 01 00 	mov    0x1c0(%rsp),%rcx
  311dfc:	00 
  311dfd:	49 89 e8             	mov    %rbp,%r8
  311e00:	e8 6b e5 f0 ff       	call   220370 <<rexx_exec::Interp>::set_exposed_variable>
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:77
  311e05:	4d 85 ff             	test   %r15,%r15
  311e08:	0f 85 14 01 00 00    	jne    311f22 <<rexx_exec::Interp>::exec_parse+0x25f2>
  311e0e:	e9 5b 01 00 00       	jmp    311f6e <<rexx_exec::Interp>::exec_parse+0x263e>
<rexx_exec::Interp>::assign_targets:
  311e13:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  311e1a:	84 00 00 00 00 00 
  311e20:	45 31 e4             	xor    %r12d,%r12d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:768
  311e23:	f6 84 24 bf 00 00 00 	testb  $0x1,0xbf(%rsp)
  311e2a:	01 
  311e2b:	74 83                	je     311db0 <<rexx_exec::Interp>::exec_parse+0x2480>
<rexx_exec::Interp>::assign_targets::{closure#1}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:769
  311e2d:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  311e32:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  311e37:	48 8d 94 24 d0 00 00 	lea    0xd0(%rsp),%rdx
  311e3e:	00 
  311e3f:	4c 89 f9             	mov    %r15,%rcx
  311e42:	4d 89 f0             	mov    %r14,%r8
  311e45:	e8 96 56 04 00       	call   3574e0 <<rexx_exec::Interp>::copied_piece>
<bool>::then::<alloc::vec::Vec<u8>, <rexx_exec::Interp>::assign_targets::{closure#1}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/bool.rs:66
  311e4a:	4c 8b 74 24 48       	mov    0x48(%rsp),%r14
  311e4f:	4c 8b 7c 24 50       	mov    0x50(%rsp),%r15
  311e54:	48 8b 44 24 58       	mov    0x58(%rsp),%rax
  311e59:	48 89 84 24 08 02 00 	mov    %rax,0x208(%rsp)
  311e60:	00 
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:770
  311e61:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  311e66:	4c 89 bc 24 f0 00 00 	mov    %r15,0xf0(%rsp)
  311e6d:	00 
  311e6e:	74 5a                	je     311eca <<rexx_exec::Interp>::exec_parse+0x259a>
<core::option::Option<alloc::vec::Vec<u8>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311e70:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
  311e74:	b8 00 00 00 00       	mov    $0x0,%eax
  311e79:	4c 0f 44 f8          	cmove  %rax,%r15
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  311e7d:	45 84 e4             	test   %r12b,%r12b
  311e80:	0f 85 3d ff ff ff    	jne    311dc3 <<rexx_exec::Interp>::exec_parse+0x2493>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:778
  311e86:	48 c7 04 24 00 00 00 	movq   $0x0,(%rsp)
  311e8d:	00 
  311e8e:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  311e93:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  311e98:	4c 89 e6             	mov    %r12,%rsi
  311e9b:	48 8b 94 24 e8 00 00 	mov    0xe8(%rsp),%rdx
  311ea2:	00 
  311ea3:	4c 89 e9             	mov    %r13,%rcx
  311ea6:	49 89 e8             	mov    %rbp,%r8
  311ea9:	4c 8b 8c 24 c0 00 00 	mov    0xc0(%rsp),%r9
  311eb0:	00 
  311eb1:	e8 aa 84 03 00       	call   34a360 <<rexx_exec::Interp>::assign_expr_target>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  311eb6:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  311ebb:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  311ebf:	0f 84 a9 00 00 00    	je     311f6e <<rexx_exec::Interp>::exec_parse+0x263e>
  311ec5:	e9 f4 0e 00 00       	jmp    312dbe <<rexx_exec::Interp>::exec_parse+0x348e>
<rexx_exec::Interp>::assign_targets:
  311eca:	45 31 ff             	xor    %r15d,%r15d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:775
  311ecd:	45 84 e4             	test   %r12b,%r12b
  311ed0:	0f 85 ed fe ff ff    	jne    311dc3 <<rexx_exec::Interp>::exec_parse+0x2493>
  311ed6:	eb ae                	jmp    311e86 <<rexx_exec::Interp>::exec_parse+0x2556>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:300
  311ed8:	49 3b 94 24 80 0b 00 	cmp    0xb80(%r12),%rdx
  311edf:	00 
  311ee0:	0f 85 af 0b 00 00    	jne    312a95 <<rexx_exec::Interp>::exec_parse+0x3165>
  311ee6:	49 8b 84 24 78 0b 00 	mov    0xb78(%r12),%rax
  311eed:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:302
  311eee:	48 03 84 24 c0 01 00 	add    0x1c0(%rsp),%rax
  311ef5:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  311ef6:	49 3b 84 24 08 0b 00 	cmp    0xb08(%r12),%rax
  311efd:	00 
  311efe:	0f 83 21 17 00 00    	jae    313625 <<rexx_exec::Interp>::exec_parse+0x3cf5>
<alloc::raw_vec::RawVecInner>::non_null::<core::option::Option<rexx_core::handle::ObjRef>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311f04:	49 8b 8c 24 00 0b 00 	mov    0xb00(%r12),%rcx
  311f0b:	00 
<usize as core::slice::index::SliceIndex<[core::option::Option<rexx_core::handle::ObjRef>]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:245
  311f0c:	48 c1 e0 04          	shl    $0x4,%rax
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:308
  311f10:	48 c7 04 01 01 00 00 	movq   $0x1,(%rcx,%rax,1)
  311f17:	00 
  311f18:	48 89 6c 01 08       	mov    %rbp,0x8(%rcx,%rax,1)
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:77
  311f1d:	4d 85 ff             	test   %r15,%r15
  311f20:	74 4c                	je     311f6e <<rexx_exec::Interp>::exec_parse+0x263e>
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  311f22:	8b bc 24 dc 01 00 00 	mov    0x1dc(%rsp),%edi
  311f29:	48 8b 84 24 20 02 00 	mov    0x220(%rsp),%rax
  311f30:	00 
<alloc::vec::Vec<alloc::boxed::Box<str>>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  311f31:	48 8b 70 10          	mov    0x10(%rax),%rsi
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  311f35:	48 39 fe             	cmp    %rdi,%rsi
  311f38:	0f 86 1d 19 00 00    	jbe    31385b <<rexx_exec::Interp>::exec_parse+0x3f2b>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::boxed::Box<str>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  311f3e:	48 8b 40 08          	mov    0x8(%rax),%rax
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  311f42:	48 c1 e7 04          	shl    $0x4,%rdi
<rexx_parse::token::symbols::SymbolTable>::name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-parse/src/token/symbols.rs:68
  311f46:	48 8b 14 38          	mov    (%rax,%rdi,1),%rdx
  311f4a:	48 8b 4c 38 08       	mov    0x8(%rax,%rdi,1),%rcx
<rexx_exec::Interp>::assign_bound_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/assignment.rs:78
  311f4f:	48 89 2c 24          	mov    %rbp,(%rsp)
  311f53:	4c 89 e7             	mov    %r12,%rdi
  311f56:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  311f5d:	00 
  311f5e:	4d 89 f8             	mov    %r15,%r8
  311f61:	4c 8b 8c 24 08 02 00 	mov    0x208(%rsp),%r9
  311f68:	00 
  311f69:	e8 b2 81 03 00       	call   34a120 <<rexx_exec::Interp>::trace_assignment>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:783
  311f6e:	f6 84 24 be 00 00 00 	testb  $0x1,0xbe(%rsp)
  311f75:	01 
  311f76:	74 18                	je     311f90 <<rexx_exec::Interp>::exec_parse+0x2660>
  311f78:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:789
  311f7c:	75 52                	jne    311fd0 <<rexx_exec::Interp>::exec_parse+0x26a0>
  311f7e:	e9 ad f8 ff ff       	jmp    311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
  311f83:	66 66 66 66 2e 0f 1f 	data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  311f8a:	84 00 00 00 00 00 
  311f90:	49 83 fe ff          	cmp    $0xffffffffffffffff,%r14
<core::option::Option<alloc::vec::Vec<u8>>>::as_ref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:743
  311f94:	0f 84 a7 12 00 00    	je     313241 <<rexx_exec::Interp>::exec_parse+0x3911>
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  311f9a:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  311fa1:	00 00 
  311fa3:	75 2b                	jne    311fd0 <<rexx_exec::Interp>::exec_parse+0x26a0>
  311fa5:	41 f6 84 24 2d 05 00 	testb  $0x1,0x52d(%r12)
  311fac:	00 01 
  311fae:	74 20                	je     311fd0 <<rexx_exec::Interp>::exec_parse+0x26a0>
<rexx_exec::Interp>::trace_result:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:225
  311fb0:	4c 89 e7             	mov    %r12,%rdi
  311fb3:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  311fba:	00 
  311fbb:	48 8b 94 24 f0 00 00 	mov    0xf0(%rsp),%rdx
  311fc2:	00 
  311fc3:	48 8b 8c 24 08 02 00 	mov    0x208(%rsp),%rcx
  311fca:	00 
  311fcb:	e8 e0 c0 f6 ff       	call   27e0b0 <<rexx_exec::Interp>::trace_result_line>
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  311fd0:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  311fd7:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  311fdd:	77 11                	ja     311ff0 <<rexx_exec::Interp>::exec_parse+0x26c0>
  311fdf:	4d 85 f6             	test   %r14,%r14
  311fe2:	75 1a                	jne    311ffe <<rexx_exec::Interp>::exec_parse+0x26ce>
  311fe4:	e9 47 f8 ff ff       	jmp    311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
  311fe9:	0f 1f 80 00 00 00 00 	nopl   0x0(%rax)
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  311ff0:	4d 8b bc 24 e0 0b 00 	mov    0xbe0(%r12),%r15
  311ff7:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  311ff8:	49 83 ff 08          	cmp    $0x8,%r15
  311ffc:	72 13                	jb     312011 <<rexx_exec::Interp>::exec_parse+0x26e1>
<rexx_exec::Interp>::assign_targets:
  311ffe:	48 8b bc 24 f0 00 00 	mov    0xf0(%rsp),%rdi
  312005:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312006:	ff 15 5c 61 1c 00    	call   *0x1c615c(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  31200c:	e9 1f f8 ff ff       	jmp    311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
<rexx_exec::Interp>::assign_targets:
  312011:	48 8b 84 24 38 01 00 	mov    0x138(%rsp),%rax
  312018:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312019:	4c 3b 38             	cmp    (%rax),%r15
  31201c:	0f 85 ce f7 ff ff    	jne    3117f0 <<rexx_exec::Interp>::exec_parse+0x1ec0>
  312022:	48 8b bc 24 38 01 00 	mov    0x138(%rsp),%rdi
  312029:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31202a:	e8 d1 f0 17 00       	call   491100 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  31202f:	e9 bc f7 ff ff       	jmp    3117f0 <<rexx_exec::Interp>::exec_parse+0x1ec0>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  312034:	0f b6 42 11          	movzbl 0x11(%rdx),%eax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:84
  312038:	48 83 c2 12          	add    $0x12,%rdx
  31203c:	48 89 d1             	mov    %rdx,%rcx
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:252
  31203f:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  312044:	0f 83 96 00 00 00    	jae    3120e0 <<rexx_exec::Interp>::exec_parse+0x27b0>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  31204a:	49 39 f7             	cmp    %rsi,%r15
  31204d:	73 31                	jae    312080 <<rexx_exec::Interp>::exec_parse+0x2750>
  31204f:	49 39 c7             	cmp    %rax,%r15
  312052:	48 89 c7             	mov    %rax,%rdi
  312055:	49 0f 47 ff          	cmova  %r15,%rdi
  312059:	eb 0d                	jmp    312068 <<rexx_exec::Interp>::exec_parse+0x2738>
  31205b:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:257
  312060:	49 ff c7             	inc    %r15
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:256
  312063:	4c 39 fe             	cmp    %r15,%rsi
  312066:	74 6e                	je     3120d6 <<rexx_exec::Interp>::exec_parse+0x27a6>
  312068:	4c 39 ff             	cmp    %r15,%rdi
  31206b:	0f 84 3e 17 00 00    	je     3137af <<rexx_exec::Interp>::exec_parse+0x3e7f>
  312071:	42 0f b6 14 39       	movzbl (%rcx,%r15,1),%edx
  312076:	83 fa 20             	cmp    $0x20,%edx
rexx_exec::parse_template::is_blank:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:31
  312079:	74 e5                	je     312060 <<rexx_exec::Interp>::exec_parse+0x2730>
  31207b:	83 fa 09             	cmp    $0x9,%edx
  31207e:	74 e0                	je     312060 <<rexx_exec::Interp>::exec_parse+0x2730>
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  312080:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  312085:	73 59                	jae    3120e0 <<rexx_exec::Interp>::exec_parse+0x27b0>
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2489
  312087:	49 39 c7             	cmp    %rax,%r15
  31208a:	48 89 c7             	mov    %rax,%rdi
  31208d:	49 0f 47 ff          	cmova  %r15,%rdi
  312091:	4c 89 fb             	mov    %r15,%rbx
<usize as core::cmp::PartialOrd>::lt:
  312094:	66 66 66 2e 0f 1f 84 	data16 data16 cs nopw 0x0(%rax,%rax,1)
  31209b:	00 00 00 00 00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/cmp.rs:2144
  3120a0:	48 39 5c 24 28       	cmp    %rbx,0x28(%rsp)
<core::ops::range::Range<usize> as core::iter::range::RangeIteratorImpl>::spec_next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/range.rs:1100
  3120a5:	0f 84 4d 04 00 00    	je     3124f8 <<rexx_exec::Interp>::exec_parse+0x2bc8>
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  3120ab:	48 39 df             	cmp    %rbx,%rdi
  3120ae:	0f 84 ec 16 00 00    	je     3137a0 <<rexx_exec::Interp>::exec_parse+0x3e70>
  3120b4:	0f b6 14 19          	movzbl (%rcx,%rbx,1),%edx
<usize>::unchecked_add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1013
  3120b8:	48 ff c3             	inc    %rbx
<rexx_exec::parse_template::Cursor>::next_word::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:263
  3120bb:	83 fa 09             	cmp    $0x9,%edx
<core::ops::range::Range<usize> as core::iter::traits::iterator::Iterator>::try_fold::<(), core::iter::traits::iterator::Iterator::find::check<usize, <rexx_exec::parse_template::Cursor>::next_word::{closure#0}>::{closure#0}, core::ops::control_flow::ControlFlow<usize>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/iterator.rs:2490
  3120be:	74 05                	je     3120c5 <<rexx_exec::Interp>::exec_parse+0x2795>
  3120c0:	83 fa 20             	cmp    $0x20,%edx
  3120c3:	75 db                	jne    3120a0 <<rexx_exec::Interp>::exec_parse+0x2770>
  3120c5:	4c 8d 43 ff          	lea    -0x1(%rbx),%r8
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  3120c9:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  3120ce:	0f 84 5c f7 ff ff    	je     311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
  3120d4:	eb 1e                	jmp    3120f4 <<rexx_exec::Interp>::exec_parse+0x27c4>
  3120d6:	49 89 f7             	mov    %rsi,%r15
<rexx_exec::parse_template::Cursor>::next_word:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:260
  3120d9:	4c 3b 7c 24 28       	cmp    0x28(%rsp),%r15
  3120de:	72 a7                	jb     312087 <<rexx_exec::Interp>::exec_parse+0x2757>
<rexx_exec::Interp>::assign_targets:
  3120e0:	4c 89 fb             	mov    %r15,%rbx
  3120e3:	45 31 ff             	xor    %r15d,%r15d
  3120e6:	45 31 c0             	xor    %r8d,%r8d
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  3120e9:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  3120ee:	0f 84 3c f7 ff ff    	je     311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:804
  3120f4:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  3120f9:	4c 89 e6             	mov    %r12,%rsi
  3120fc:	48 8d 94 24 d0 00 00 	lea    0xd0(%rsp),%rdx
  312103:	00 
  312104:	4c 89 f9             	mov    %r15,%rcx
  312107:	e8 d4 53 04 00       	call   3574e0 <<rexx_exec::Interp>::copied_piece>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  31210c:	4c 8b 7c 24 50       	mov    0x50(%rsp),%r15
  312111:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  312116:	41 80 bc 24 35 05 00 	cmpb   $0x0,0x535(%r12)
  31211d:	00 00 
  31211f:	75 6f                	jne    312190 <<rexx_exec::Interp>::exec_parse+0x2860>
  312121:	41 f6 84 24 2e 05 00 	testb  $0x1,0x52e(%r12)
  312128:	00 01 
  31212a:	74 64                	je     312190 <<rexx_exec::Interp>::exec_parse+0x2860>
<rexx_exec::Interp>::trace_dummy:
  31212c:	4c 8b 74 24 58       	mov    0x58(%rsp),%r14
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312131:	49 8b ac 24 e8 10 00 	mov    0x10e8(%r12),%rbp
  312138:	00 
  312139:	48 8b bc 24 00 02 00 	mov    0x200(%rsp),%rdi
  312140:	00 
rexx_exec::trace::format::push_value:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:559
  312141:	48 8d 35 9f 8a d5 ff 	lea    -0x2a7561(%rip),%rsi        # 6abe7 <panic_unwind::imp::CANARY+0x5e1a>
  312148:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  31214f:	00 
  312150:	e8 1b d3 f6 ff       	call   27f470 <rexx_exec::trace::format::push_prefixed_blanks>
  312155:	48 8b bc 24 00 02 00 	mov    0x200(%rsp),%rdi
  31215c:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:560
  31215d:	4c 89 fe             	mov    %r15,%rsi
  312160:	4c 89 f2             	mov    %r14,%rdx
  312163:	e8 38 d4 f6 ff       	call   27f5a0 <rexx_exec::trace::format::push_quoted>
<alloc::vec::Vec<u8>>::as_mut_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1904
  312168:	49 8b 94 24 e8 10 00 	mov    0x10e8(%r12),%rdx
  31216f:	00 
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:557
  312170:	48 39 ea             	cmp    %rbp,%rdx
  312173:	0f 82 0f 15 00 00    	jb     313688 <<rexx_exec::Interp>::exec_parse+0x3d58>
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312179:	48 89 d6             	mov    %rdx,%rsi
  31217c:	48 29 ee             	sub    %rbp,%rsi
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  31217f:	0f 85 9f 00 00 00    	jne    312224 <<rexx_exec::Interp>::exec_parse+0x28f4>
<rexx_exec::Interp>::trace_dummy:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:576
  312185:	4c 89 e7             	mov    %r12,%rdi
  312188:	48 89 ee             	mov    %rbp,%rsi
  31218b:	e8 30 c0 e8 ff       	call   19e1c0 <<rexx_exec::Interp>::route_trace_line>
<rexx_exec::Interp>::assign_targets:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:806
  312190:	4c 8b 74 24 48       	mov    0x48(%rsp),%r14
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312195:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  31219c:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  3121a2:	77 11                	ja     3121b5 <<rexx_exec::Interp>::exec_parse+0x2885>
  3121a4:	4d 85 f6             	test   %r14,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3121a7:	0f 84 83 f6 ff ff    	je     311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3121ad:	4c 89 ff             	mov    %r15,%rdi
  3121b0:	e9 51 fe ff ff       	jmp    312006 <<rexx_exec::Interp>::exec_parse+0x26d6>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3121b5:	4d 8b a4 24 e0 0b 00 	mov    0xbe0(%r12),%r12
  3121bc:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3121bd:	49 83 fc 08          	cmp    $0x8,%r12
  3121c1:	72 13                	jb     3121d6 <<rexx_exec::Interp>::exec_parse+0x28a6>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3121c3:	4c 89 ff             	mov    %r15,%rdi
  3121c6:	ff 15 9c 5f 1c 00    	call   *0x1c5f9c(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  3121cc:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  3121d1:	e9 5a f6 ff ff       	jmp    311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
  3121d6:	48 8b 84 24 38 01 00 	mov    0x138(%rsp),%rax
  3121dd:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  3121de:	4c 3b 20             	cmp    (%rax),%r12
  3121e1:	75 0d                	jne    3121f0 <<rexx_exec::Interp>::exec_parse+0x28c0>
  3121e3:	48 8b bc 24 38 01 00 	mov    0x138(%rsp),%rdi
  3121ea:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3121eb:	e8 10 ef 17 00       	call   491100 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  3121f0:	48 8b 54 24 18       	mov    0x18(%rsp),%rdx
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  3121f5:	48 8b 82 d8 0b 00 00 	mov    0xbd8(%rdx),%rax
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  3121fc:	4b 8d 0c 64          	lea    (%r12,%r12,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312200:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  312204:	4c 89 7c c8 08       	mov    %r15,0x8(%rax,%rcx,8)
  312209:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312210:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312212:	49 ff c4             	inc    %r12
  312215:	4c 89 a2 e0 0b 00 00 	mov    %r12,0xbe0(%rdx)
  31221c:	49 89 d4             	mov    %rdx,%r12
  31221f:	e9 0c f6 ff ff       	jmp    311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
rexx_exec::trace::format::push_value:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:561
  312224:	49 8b 84 24 e0 10 00 	mov    0x10e0(%r12),%rax
  31222b:	00 
core::slice::index::get_offset_len_mut_noubcheck::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:102
  31222c:	48 8d 3c 28          	lea    (%rax,%rbp,1),%rdi
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  312230:	48 83 fe 10          	cmp    $0x10,%rsi
  312234:	73 1c                	jae    312252 <<rexx_exec::Interp>::exec_parse+0x2922>
  312236:	48 89 f9             	mov    %rdi,%rcx
  312239:	e9 1e 08 00 00       	jmp    312a5c <<rexx_exec::Interp>::exec_parse+0x312c>
<rexx_exec::Interp>::assign_targets:
  31223e:	4c 89 f3             	mov    %r14,%rbx
<usize>::checked_sub:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/num/uint_macros.rs:1092
  312241:	4d 89 f4             	mov    %r14,%r12
  312244:	4d 29 fc             	sub    %r15,%r12
  312247:	0f 83 7a f6 ff ff    	jae    3118c7 <<rexx_exec::Interp>::exec_parse+0x1f97>
  31224d:	e9 5c 0e 00 00       	jmp    3130ae <<rexx_exec::Interp>::exec_parse+0x377e>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  312252:	48 83 fe 20          	cmp    $0x20,%rsi
  312256:	0f 83 c2 02 00 00    	jae    31251e <<rexx_exec::Interp>::exec_parse+0x2bee>
  31225c:	45 31 c0             	xor    %r8d,%r8d
  31225f:	49 89 f1             	mov    %rsi,%r9
  312262:	49 83 e1 f0          	and    $0xfffffffffffffff0,%r9
  312266:	4a 8d 0c 0f          	lea    (%rdi,%r9,1),%rcx
  31226a:	eb 11                	jmp    31227d <<rexx_exec::Interp>::exec_parse+0x294d>
  31226c:	0f 1f 40 00          	nopl   0x0(%rax)
  312270:	49 83 c0 10          	add    $0x10,%r8
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312274:	4d 39 c1             	cmp    %r8,%r9
  312277:	0f 84 93 02 00 00    	je     312510 <<rexx_exec::Interp>::exec_parse+0x2be0>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  31227d:	f3 42 0f 6f 0c 07    	movdqu (%rdi,%r8,1),%xmm1
  312283:	66 0f 6f d1          	movdqa %xmm1,%xmm2
  312287:	66 0f da 15 01 e2 d3 	pminub -0x2c1dff(%rip),%xmm2        # 50490 <GCC_except_table5014+0xa7c>
  31228e:	ff 
  31228f:	66 0f 74 d1          	pcmpeqb %xmm1,%xmm2
  312293:	66 0f 6f c1          	movdqa %xmm1,%xmm0
  312297:	66 0f 74 05 51 de d3 	pcmpeqb -0x2c21af(%rip),%xmm0        # 500f0 <GCC_except_table5014+0x6dc>
  31229e:	ff 
  31229f:	66 0f fc 0d f9 e0 d3 	paddb  -0x2c1f07(%rip),%xmm1        # 503a0 <GCC_except_table5014+0x98c>
  3122a6:	ff 
  3122a7:	66 0f 6f d9          	movdqa %xmm1,%xmm3
  3122ab:	66 0f da 1d 2d dd d3 	pminub -0x2c22d3(%rip),%xmm3        # 4ffe0 <GCC_except_table5014+0x5cc>
  3122b2:	ff 
  3122b3:	66 0f 74 d9          	pcmpeqb %xmm1,%xmm3
  3122b7:	66 0f df c3          	pandn  %xmm3,%xmm0
  3122bb:	66 0f db c2          	pand   %xmm2,%xmm0
  3122bf:	66 0f 7f 84 24 30 02 	movdqa %xmm0,0x230(%rsp)
  3122c6:	00 00 
  3122c8:	f6 84 24 30 02 00 00 	testb  $0x1,0x230(%rsp)
  3122cf:	01 
  3122d0:	74 05                	je     3122d7 <<rexx_exec::Interp>::exec_parse+0x29a7>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  3122d2:	42 c6 04 07 3f       	movb   $0x3f,(%rdi,%r8,1)
  3122d7:	66 41 0f 7e c2       	movd   %xmm0,%r10d
  3122dc:	45 89 d3             	mov    %r10d,%r11d
  3122df:	41 c1 eb 08          	shr    $0x8,%r11d
  3122e3:	41 f6 c3 01          	test   $0x1,%r11b
  3122e7:	0f 84 03 01 00 00    	je     3123f0 <<rexx_exec::Interp>::exec_parse+0x2ac0>
  3122ed:	42 c6 44 07 01 3f    	movb   $0x3f,0x1(%rdi,%r8,1)
  3122f3:	45 89 d3             	mov    %r10d,%r11d
  3122f6:	41 c1 eb 10          	shr    $0x10,%r11d
  3122fa:	41 f6 c3 01          	test   $0x1,%r11b
  3122fe:	0f 85 fd 00 00 00    	jne    312401 <<rexx_exec::Interp>::exec_parse+0x2ad1>
  312304:	41 c1 ea 18          	shr    $0x18,%r10d
  312308:	41 f6 c2 01          	test   $0x1,%r10b
  31230c:	0f 84 03 01 00 00    	je     312415 <<rexx_exec::Interp>::exec_parse+0x2ae5>
  312312:	42 c6 44 07 03 3f    	movb   $0x3f,0x3(%rdi,%r8,1)
  312318:	66 44 0f c5 d0 02    	pextrw $0x2,%xmm0,%r10d
  31231e:	41 f6 c2 01          	test   $0x1,%r10b
  312322:	0f 85 fd 00 00 00    	jne    312425 <<rexx_exec::Interp>::exec_parse+0x2af5>
  312328:	41 c1 ea 08          	shr    $0x8,%r10d
  31232c:	41 f6 c2 01          	test   $0x1,%r10b
  312330:	0f 84 03 01 00 00    	je     312439 <<rexx_exec::Interp>::exec_parse+0x2b09>
  312336:	42 c6 44 07 05 3f    	movb   $0x3f,0x5(%rdi,%r8,1)
  31233c:	66 44 0f c5 d0 03    	pextrw $0x3,%xmm0,%r10d
  312342:	41 f6 c2 01          	test   $0x1,%r10b
  312346:	0f 85 fd 00 00 00    	jne    312449 <<rexx_exec::Interp>::exec_parse+0x2b19>
  31234c:	41 c1 ea 08          	shr    $0x8,%r10d
  312350:	41 f6 c2 01          	test   $0x1,%r10b
  312354:	0f 84 03 01 00 00    	je     31245d <<rexx_exec::Interp>::exec_parse+0x2b2d>
  31235a:	42 c6 44 07 07 3f    	movb   $0x3f,0x7(%rdi,%r8,1)
  312360:	66 44 0f c5 d0 04    	pextrw $0x4,%xmm0,%r10d
  312366:	41 f6 c2 01          	test   $0x1,%r10b
  31236a:	0f 85 fd 00 00 00    	jne    31246d <<rexx_exec::Interp>::exec_parse+0x2b3d>
  312370:	41 c1 ea 08          	shr    $0x8,%r10d
  312374:	41 f6 c2 01          	test   $0x1,%r10b
  312378:	0f 84 03 01 00 00    	je     312481 <<rexx_exec::Interp>::exec_parse+0x2b51>
  31237e:	42 c6 44 07 09 3f    	movb   $0x3f,0x9(%rdi,%r8,1)
  312384:	66 44 0f c5 d0 05    	pextrw $0x5,%xmm0,%r10d
  31238a:	41 f6 c2 01          	test   $0x1,%r10b
  31238e:	0f 85 fd 00 00 00    	jne    312491 <<rexx_exec::Interp>::exec_parse+0x2b61>
  312394:	41 c1 ea 08          	shr    $0x8,%r10d
  312398:	41 f6 c2 01          	test   $0x1,%r10b
  31239c:	0f 84 03 01 00 00    	je     3124a5 <<rexx_exec::Interp>::exec_parse+0x2b75>
  3123a2:	42 c6 44 07 0b 3f    	movb   $0x3f,0xb(%rdi,%r8,1)
  3123a8:	66 44 0f c5 d0 06    	pextrw $0x6,%xmm0,%r10d
  3123ae:	41 f6 c2 01          	test   $0x1,%r10b
  3123b2:	0f 85 fd 00 00 00    	jne    3124b5 <<rexx_exec::Interp>::exec_parse+0x2b85>
  3123b8:	41 c1 ea 08          	shr    $0x8,%r10d
  3123bc:	41 f6 c2 01          	test   $0x1,%r10b
  3123c0:	0f 84 03 01 00 00    	je     3124c9 <<rexx_exec::Interp>::exec_parse+0x2b99>
  3123c6:	42 c6 44 07 0d 3f    	movb   $0x3f,0xd(%rdi,%r8,1)
  3123cc:	66 44 0f c5 d0 07    	pextrw $0x7,%xmm0,%r10d
  3123d2:	41 f6 c2 01          	test   $0x1,%r10b
  3123d6:	0f 85 fd 00 00 00    	jne    3124d9 <<rexx_exec::Interp>::exec_parse+0x2ba9>
  3123dc:	41 c1 ea 08          	shr    $0x8,%r10d
  3123e0:	41 f6 c2 01          	test   $0x1,%r10b
  3123e4:	0f 84 86 fe ff ff    	je     312270 <<rexx_exec::Interp>::exec_parse+0x2940>
  3123ea:	e9 fe 00 00 00       	jmp    3124ed <<rexx_exec::Interp>::exec_parse+0x2bbd>
  3123ef:	90                   	nop
  3123f0:	45 89 d3             	mov    %r10d,%r11d
  3123f3:	41 c1 eb 10          	shr    $0x10,%r11d
  3123f7:	41 f6 c3 01          	test   $0x1,%r11b
  3123fb:	0f 84 03 ff ff ff    	je     312304 <<rexx_exec::Interp>::exec_parse+0x29d4>
  312401:	42 c6 44 07 02 3f    	movb   $0x3f,0x2(%rdi,%r8,1)
  312407:	41 c1 ea 18          	shr    $0x18,%r10d
  31240b:	41 f6 c2 01          	test   $0x1,%r10b
  31240f:	0f 85 fd fe ff ff    	jne    312312 <<rexx_exec::Interp>::exec_parse+0x29e2>
  312415:	66 44 0f c5 d0 02    	pextrw $0x2,%xmm0,%r10d
  31241b:	41 f6 c2 01          	test   $0x1,%r10b
  31241f:	0f 84 03 ff ff ff    	je     312328 <<rexx_exec::Interp>::exec_parse+0x29f8>
  312425:	42 c6 44 07 04 3f    	movb   $0x3f,0x4(%rdi,%r8,1)
  31242b:	41 c1 ea 08          	shr    $0x8,%r10d
  31242f:	41 f6 c2 01          	test   $0x1,%r10b
  312433:	0f 85 fd fe ff ff    	jne    312336 <<rexx_exec::Interp>::exec_parse+0x2a06>
  312439:	66 44 0f c5 d0 03    	pextrw $0x3,%xmm0,%r10d
  31243f:	41 f6 c2 01          	test   $0x1,%r10b
  312443:	0f 84 03 ff ff ff    	je     31234c <<rexx_exec::Interp>::exec_parse+0x2a1c>
  312449:	42 c6 44 07 06 3f    	movb   $0x3f,0x6(%rdi,%r8,1)
  31244f:	41 c1 ea 08          	shr    $0x8,%r10d
  312453:	41 f6 c2 01          	test   $0x1,%r10b
  312457:	0f 85 fd fe ff ff    	jne    31235a <<rexx_exec::Interp>::exec_parse+0x2a2a>
  31245d:	66 44 0f c5 d0 04    	pextrw $0x4,%xmm0,%r10d
  312463:	41 f6 c2 01          	test   $0x1,%r10b
  312467:	0f 84 03 ff ff ff    	je     312370 <<rexx_exec::Interp>::exec_parse+0x2a40>
  31246d:	42 c6 44 07 08 3f    	movb   $0x3f,0x8(%rdi,%r8,1)
  312473:	41 c1 ea 08          	shr    $0x8,%r10d
  312477:	41 f6 c2 01          	test   $0x1,%r10b
  31247b:	0f 85 fd fe ff ff    	jne    31237e <<rexx_exec::Interp>::exec_parse+0x2a4e>
  312481:	66 44 0f c5 d0 05    	pextrw $0x5,%xmm0,%r10d
  312487:	41 f6 c2 01          	test   $0x1,%r10b
  31248b:	0f 84 03 ff ff ff    	je     312394 <<rexx_exec::Interp>::exec_parse+0x2a64>
  312491:	42 c6 44 07 0a 3f    	movb   $0x3f,0xa(%rdi,%r8,1)
  312497:	41 c1 ea 08          	shr    $0x8,%r10d
  31249b:	41 f6 c2 01          	test   $0x1,%r10b
  31249f:	0f 85 fd fe ff ff    	jne    3123a2 <<rexx_exec::Interp>::exec_parse+0x2a72>
  3124a5:	66 44 0f c5 d0 06    	pextrw $0x6,%xmm0,%r10d
  3124ab:	41 f6 c2 01          	test   $0x1,%r10b
  3124af:	0f 84 03 ff ff ff    	je     3123b8 <<rexx_exec::Interp>::exec_parse+0x2a88>
  3124b5:	42 c6 44 07 0c 3f    	movb   $0x3f,0xc(%rdi,%r8,1)
  3124bb:	41 c1 ea 08          	shr    $0x8,%r10d
  3124bf:	41 f6 c2 01          	test   $0x1,%r10b
  3124c3:	0f 85 fd fe ff ff    	jne    3123c6 <<rexx_exec::Interp>::exec_parse+0x2a96>
  3124c9:	66 44 0f c5 d0 07    	pextrw $0x7,%xmm0,%r10d
  3124cf:	41 f6 c2 01          	test   $0x1,%r10b
  3124d3:	0f 84 03 ff ff ff    	je     3123dc <<rexx_exec::Interp>::exec_parse+0x2aac>
  3124d9:	42 c6 44 07 0e 3f    	movb   $0x3f,0xe(%rdi,%r8,1)
  3124df:	41 c1 ea 08          	shr    $0x8,%r10d
  3124e3:	41 f6 c2 01          	test   $0x1,%r10b
  3124e7:	0f 84 83 fd ff ff    	je     312270 <<rexx_exec::Interp>::exec_parse+0x2940>
  3124ed:	42 c6 44 07 0f 3f    	movb   $0x3f,0xf(%rdi,%r8,1)
  3124f3:	e9 78 fd ff ff       	jmp    312270 <<rexx_exec::Interp>::exec_parse+0x2940>
<rexx_exec::Interp>::assign_targets:
  3124f8:	4c 8b 44 24 28       	mov    0x28(%rsp),%r8
  3124fd:	4c 89 c3             	mov    %r8,%rbx
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:803
  312500:	f6 44 24 3f 01       	testb  $0x1,0x3f(%rsp)
  312505:	0f 84 25 f3 ff ff    	je     311830 <<rexx_exec::Interp>::exec_parse+0x1f00>
  31250b:	e9 e4 fb ff ff       	jmp    3120f4 <<rexx_exec::Interp>::exec_parse+0x27c4>
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312510:	4c 39 ce             	cmp    %r9,%rsi
  312513:	0f 84 6c fc ff ff    	je     312185 <<rexx_exec::Interp>::exec_parse+0x2855>
  312519:	e9 3e 05 00 00       	jmp    312a5c <<rexx_exec::Interp>::exec_parse+0x312c>
  31251e:	49 89 f0             	mov    %rsi,%r8
  312521:	49 83 e0 e0          	and    $0xffffffffffffffe0,%r8
  312525:	4a 8d 0c 07          	lea    (%rdi,%r8,1),%rcx
  312529:	45 31 c9             	xor    %r9d,%r9d
  31252c:	eb 0f                	jmp    31253d <<rexx_exec::Interp>::exec_parse+0x2c0d>
  31252e:	66 90                	xchg   %ax,%ax
  312530:	49 83 c1 20          	add    $0x20,%r9
  312534:	4d 39 c8             	cmp    %r9,%r8
  312537:	0f 84 0c 05 00 00    	je     312a49 <<rexx_exec::Interp>::exec_parse+0x3119>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  31253d:	f3 42 0f 6f 14 0f    	movdqu (%rdi,%r9,1),%xmm2
  312543:	f3 42 0f 6f 44 0f 10 	movdqu 0x10(%rdi,%r9,1),%xmm0
  31254a:	66 0f 6f da          	movdqa %xmm2,%xmm3
  31254e:	66 0f da 1d 3a df d3 	pminub -0x2c20c6(%rip),%xmm3        # 50490 <GCC_except_table5014+0xa7c>
  312555:	ff 
  312556:	66 0f 74 da          	pcmpeqb %xmm2,%xmm3
  31255a:	66 0f 6f ca          	movdqa %xmm2,%xmm1
  31255e:	66 0f 74 0d 8a db d3 	pcmpeqb -0x2c2476(%rip),%xmm1        # 500f0 <GCC_except_table5014+0x6dc>
  312565:	ff 
  312566:	66 0f fc 15 32 de d3 	paddb  -0x2c21ce(%rip),%xmm2        # 503a0 <GCC_except_table5014+0x98c>
  31256d:	ff 
  31256e:	66 0f 6f e2          	movdqa %xmm2,%xmm4
  312572:	66 0f da 25 66 da d3 	pminub -0x2c259a(%rip),%xmm4        # 4ffe0 <GCC_except_table5014+0x5cc>
  312579:	ff 
  31257a:	66 0f 74 e2          	pcmpeqb %xmm2,%xmm4
  31257e:	66 0f df cc          	pandn  %xmm4,%xmm1
  312582:	66 0f db cb          	pand   %xmm3,%xmm1
  312586:	66 0f 7f 8c 24 40 02 	movdqa %xmm1,0x240(%rsp)
  31258d:	00 00 
  31258f:	f6 84 24 40 02 00 00 	testb  $0x1,0x240(%rsp)
  312596:	01 
  312597:	74 05                	je     31259e <<rexx_exec::Interp>::exec_parse+0x2c6e>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  312599:	42 c6 04 0f 3f       	movb   $0x3f,(%rdi,%r9,1)
  31259e:	66 41 0f 7e ca       	movd   %xmm1,%r10d
  3125a3:	45 89 d3             	mov    %r10d,%r11d
  3125a6:	41 c1 eb 08          	shr    $0x8,%r11d
  3125aa:	41 f6 c3 01          	test   $0x1,%r11b
  3125ae:	0f 84 6c 02 00 00    	je     312820 <<rexx_exec::Interp>::exec_parse+0x2ef0>
  3125b4:	42 c6 44 0f 01 3f    	movb   $0x3f,0x1(%rdi,%r9,1)
  3125ba:	45 89 d3             	mov    %r10d,%r11d
  3125bd:	41 c1 eb 10          	shr    $0x10,%r11d
  3125c1:	41 f6 c3 01          	test   $0x1,%r11b
  3125c5:	0f 85 66 02 00 00    	jne    312831 <<rexx_exec::Interp>::exec_parse+0x2f01>
  3125cb:	41 c1 ea 18          	shr    $0x18,%r10d
  3125cf:	41 f6 c2 01          	test   $0x1,%r10b
  3125d3:	0f 84 6c 02 00 00    	je     312845 <<rexx_exec::Interp>::exec_parse+0x2f15>
  3125d9:	42 c6 44 0f 03 3f    	movb   $0x3f,0x3(%rdi,%r9,1)
  3125df:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  3125e5:	41 f6 c2 01          	test   $0x1,%r10b
  3125e9:	0f 85 66 02 00 00    	jne    312855 <<rexx_exec::Interp>::exec_parse+0x2f25>
  3125ef:	41 c1 ea 08          	shr    $0x8,%r10d
  3125f3:	41 f6 c2 01          	test   $0x1,%r10b
  3125f7:	0f 84 6c 02 00 00    	je     312869 <<rexx_exec::Interp>::exec_parse+0x2f39>
  3125fd:	42 c6 44 0f 05 3f    	movb   $0x3f,0x5(%rdi,%r9,1)
  312603:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  312609:	41 f6 c2 01          	test   $0x1,%r10b
  31260d:	0f 85 66 02 00 00    	jne    312879 <<rexx_exec::Interp>::exec_parse+0x2f49>
  312613:	41 c1 ea 08          	shr    $0x8,%r10d
  312617:	41 f6 c2 01          	test   $0x1,%r10b
  31261b:	0f 84 6c 02 00 00    	je     31288d <<rexx_exec::Interp>::exec_parse+0x2f5d>
  312621:	42 c6 44 0f 07 3f    	movb   $0x3f,0x7(%rdi,%r9,1)
  312627:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  31262d:	41 f6 c2 01          	test   $0x1,%r10b
  312631:	0f 85 66 02 00 00    	jne    31289d <<rexx_exec::Interp>::exec_parse+0x2f6d>
  312637:	41 c1 ea 08          	shr    $0x8,%r10d
  31263b:	41 f6 c2 01          	test   $0x1,%r10b
  31263f:	0f 84 6c 02 00 00    	je     3128b1 <<rexx_exec::Interp>::exec_parse+0x2f81>
  312645:	42 c6 44 0f 09 3f    	movb   $0x3f,0x9(%rdi,%r9,1)
  31264b:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  312651:	41 f6 c2 01          	test   $0x1,%r10b
  312655:	0f 85 66 02 00 00    	jne    3128c1 <<rexx_exec::Interp>::exec_parse+0x2f91>
  31265b:	41 c1 ea 08          	shr    $0x8,%r10d
  31265f:	41 f6 c2 01          	test   $0x1,%r10b
  312663:	0f 84 6c 02 00 00    	je     3128d5 <<rexx_exec::Interp>::exec_parse+0x2fa5>
  312669:	42 c6 44 0f 0b 3f    	movb   $0x3f,0xb(%rdi,%r9,1)
  31266f:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  312675:	41 f6 c2 01          	test   $0x1,%r10b
  312679:	0f 85 66 02 00 00    	jne    3128e5 <<rexx_exec::Interp>::exec_parse+0x2fb5>
  31267f:	41 c1 ea 08          	shr    $0x8,%r10d
  312683:	41 f6 c2 01          	test   $0x1,%r10b
  312687:	0f 84 6c 02 00 00    	je     3128f9 <<rexx_exec::Interp>::exec_parse+0x2fc9>
  31268d:	42 c6 44 0f 0d 3f    	movb   $0x3f,0xd(%rdi,%r9,1)
  312693:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  312699:	41 f6 c2 01          	test   $0x1,%r10b
  31269d:	0f 85 66 02 00 00    	jne    312909 <<rexx_exec::Interp>::exec_parse+0x2fd9>
  3126a3:	41 c1 ea 08          	shr    $0x8,%r10d
  3126a7:	41 f6 c2 01          	test   $0x1,%r10b
  3126ab:	74 06                	je     3126b3 <<rexx_exec::Interp>::exec_parse+0x2d83>
  3126ad:	42 c6 44 0f 0f 3f    	movb   $0x3f,0xf(%rdi,%r9,1)
  3126b3:	66 0f 6f d0          	movdqa %xmm0,%xmm2
  3126b7:	66 0f da 15 d1 dd d3 	pminub -0x2c222f(%rip),%xmm2        # 50490 <GCC_except_table5014+0xa7c>
  3126be:	ff 
  3126bf:	66 0f 74 d0          	pcmpeqb %xmm0,%xmm2
  3126c3:	66 0f 6f c8          	movdqa %xmm0,%xmm1
  3126c7:	66 0f 74 0d 21 da d3 	pcmpeqb -0x2c25df(%rip),%xmm1        # 500f0 <GCC_except_table5014+0x6dc>
  3126ce:	ff 
  3126cf:	66 0f fc 05 c9 dc d3 	paddb  -0x2c2337(%rip),%xmm0        # 503a0 <GCC_except_table5014+0x98c>
  3126d6:	ff 
  3126d7:	66 0f 6f d8          	movdqa %xmm0,%xmm3
  3126db:	66 0f da 1d fd d8 d3 	pminub -0x2c2703(%rip),%xmm3        # 4ffe0 <GCC_except_table5014+0x5cc>
  3126e2:	ff 
  3126e3:	66 0f 74 d8          	pcmpeqb %xmm0,%xmm3
  3126e7:	66 0f df cb          	pandn  %xmm3,%xmm1
  3126eb:	66 0f db ca          	pand   %xmm2,%xmm1
  3126ef:	66 41 0f 7e ca       	movd   %xmm1,%r10d
  3126f4:	41 f6 c2 01          	test   $0x1,%r10b
  3126f8:	0f 84 32 02 00 00    	je     312930 <<rexx_exec::Interp>::exec_parse+0x3000>
  3126fe:	42 c6 44 0f 10 3f    	movb   $0x3f,0x10(%rdi,%r9,1)
  312704:	45 89 d3             	mov    %r10d,%r11d
  312707:	41 c1 eb 08          	shr    $0x8,%r11d
  31270b:	41 f6 c3 01          	test   $0x1,%r11b
  31270f:	0f 85 2c 02 00 00    	jne    312941 <<rexx_exec::Interp>::exec_parse+0x3011>
  312715:	45 89 d3             	mov    %r10d,%r11d
  312718:	41 c1 eb 10          	shr    $0x10,%r11d
  31271c:	41 f6 c3 01          	test   $0x1,%r11b
  312720:	0f 84 32 02 00 00    	je     312958 <<rexx_exec::Interp>::exec_parse+0x3028>
  312726:	42 c6 44 0f 12 3f    	movb   $0x3f,0x12(%rdi,%r9,1)
  31272c:	41 c1 ea 18          	shr    $0x18,%r10d
  312730:	41 f6 c2 01          	test   $0x1,%r10b
  312734:	0f 85 2c 02 00 00    	jne    312966 <<rexx_exec::Interp>::exec_parse+0x3036>
  31273a:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  312740:	41 f6 c2 01          	test   $0x1,%r10b
  312744:	0f 84 32 02 00 00    	je     31297c <<rexx_exec::Interp>::exec_parse+0x304c>
  31274a:	42 c6 44 0f 14 3f    	movb   $0x3f,0x14(%rdi,%r9,1)
  312750:	41 c1 ea 08          	shr    $0x8,%r10d
  312754:	41 f6 c2 01          	test   $0x1,%r10b
  312758:	0f 85 2c 02 00 00    	jne    31298a <<rexx_exec::Interp>::exec_parse+0x305a>
  31275e:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  312764:	41 f6 c2 01          	test   $0x1,%r10b
  312768:	0f 84 32 02 00 00    	je     3129a0 <<rexx_exec::Interp>::exec_parse+0x3070>
  31276e:	42 c6 44 0f 16 3f    	movb   $0x3f,0x16(%rdi,%r9,1)
  312774:	41 c1 ea 08          	shr    $0x8,%r10d
  312778:	41 f6 c2 01          	test   $0x1,%r10b
  31277c:	0f 85 2c 02 00 00    	jne    3129ae <<rexx_exec::Interp>::exec_parse+0x307e>
  312782:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  312788:	41 f6 c2 01          	test   $0x1,%r10b
  31278c:	0f 84 32 02 00 00    	je     3129c4 <<rexx_exec::Interp>::exec_parse+0x3094>
  312792:	42 c6 44 0f 18 3f    	movb   $0x3f,0x18(%rdi,%r9,1)
  312798:	41 c1 ea 08          	shr    $0x8,%r10d
  31279c:	41 f6 c2 01          	test   $0x1,%r10b
  3127a0:	0f 85 2c 02 00 00    	jne    3129d2 <<rexx_exec::Interp>::exec_parse+0x30a2>
  3127a6:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  3127ac:	41 f6 c2 01          	test   $0x1,%r10b
  3127b0:	0f 84 32 02 00 00    	je     3129e8 <<rexx_exec::Interp>::exec_parse+0x30b8>
  3127b6:	42 c6 44 0f 1a 3f    	movb   $0x3f,0x1a(%rdi,%r9,1)
  3127bc:	41 c1 ea 08          	shr    $0x8,%r10d
  3127c0:	41 f6 c2 01          	test   $0x1,%r10b
  3127c4:	0f 85 2c 02 00 00    	jne    3129f6 <<rexx_exec::Interp>::exec_parse+0x30c6>
  3127ca:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  3127d0:	41 f6 c2 01          	test   $0x1,%r10b
  3127d4:	0f 84 32 02 00 00    	je     312a0c <<rexx_exec::Interp>::exec_parse+0x30dc>
  3127da:	42 c6 44 0f 1c 3f    	movb   $0x3f,0x1c(%rdi,%r9,1)
  3127e0:	41 c1 ea 08          	shr    $0x8,%r10d
  3127e4:	41 f6 c2 01          	test   $0x1,%r10b
  3127e8:	0f 85 2c 02 00 00    	jne    312a1a <<rexx_exec::Interp>::exec_parse+0x30ea>
  3127ee:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  3127f4:	41 f6 c2 01          	test   $0x1,%r10b
  3127f8:	0f 84 32 02 00 00    	je     312a30 <<rexx_exec::Interp>::exec_parse+0x3100>
  3127fe:	42 c6 44 0f 1e 3f    	movb   $0x3f,0x1e(%rdi,%r9,1)
  312804:	41 c1 ea 08          	shr    $0x8,%r10d
  312808:	41 f6 c2 01          	test   $0x1,%r10b
  31280c:	0f 84 1e fd ff ff    	je     312530 <<rexx_exec::Interp>::exec_parse+0x2c00>
  312812:	e9 27 02 00 00       	jmp    312a3e <<rexx_exec::Interp>::exec_parse+0x310e>
  312817:	66 0f 1f 84 00 00 00 	nopw   0x0(%rax,%rax,1)
  31281e:	00 00 
  312820:	45 89 d3             	mov    %r10d,%r11d
  312823:	41 c1 eb 10          	shr    $0x10,%r11d
  312827:	41 f6 c3 01          	test   $0x1,%r11b
  31282b:	0f 84 9a fd ff ff    	je     3125cb <<rexx_exec::Interp>::exec_parse+0x2c9b>
  312831:	42 c6 44 0f 02 3f    	movb   $0x3f,0x2(%rdi,%r9,1)
  312837:	41 c1 ea 18          	shr    $0x18,%r10d
  31283b:	41 f6 c2 01          	test   $0x1,%r10b
  31283f:	0f 85 94 fd ff ff    	jne    3125d9 <<rexx_exec::Interp>::exec_parse+0x2ca9>
  312845:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  31284b:	41 f6 c2 01          	test   $0x1,%r10b
  31284f:	0f 84 9a fd ff ff    	je     3125ef <<rexx_exec::Interp>::exec_parse+0x2cbf>
  312855:	42 c6 44 0f 04 3f    	movb   $0x3f,0x4(%rdi,%r9,1)
  31285b:	41 c1 ea 08          	shr    $0x8,%r10d
  31285f:	41 f6 c2 01          	test   $0x1,%r10b
  312863:	0f 85 94 fd ff ff    	jne    3125fd <<rexx_exec::Interp>::exec_parse+0x2ccd>
  312869:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  31286f:	41 f6 c2 01          	test   $0x1,%r10b
  312873:	0f 84 9a fd ff ff    	je     312613 <<rexx_exec::Interp>::exec_parse+0x2ce3>
  312879:	42 c6 44 0f 06 3f    	movb   $0x3f,0x6(%rdi,%r9,1)
  31287f:	41 c1 ea 08          	shr    $0x8,%r10d
  312883:	41 f6 c2 01          	test   $0x1,%r10b
  312887:	0f 85 94 fd ff ff    	jne    312621 <<rexx_exec::Interp>::exec_parse+0x2cf1>
  31288d:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  312893:	41 f6 c2 01          	test   $0x1,%r10b
  312897:	0f 84 9a fd ff ff    	je     312637 <<rexx_exec::Interp>::exec_parse+0x2d07>
  31289d:	42 c6 44 0f 08 3f    	movb   $0x3f,0x8(%rdi,%r9,1)
  3128a3:	41 c1 ea 08          	shr    $0x8,%r10d
  3128a7:	41 f6 c2 01          	test   $0x1,%r10b
  3128ab:	0f 85 94 fd ff ff    	jne    312645 <<rexx_exec::Interp>::exec_parse+0x2d15>
  3128b1:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  3128b7:	41 f6 c2 01          	test   $0x1,%r10b
  3128bb:	0f 84 9a fd ff ff    	je     31265b <<rexx_exec::Interp>::exec_parse+0x2d2b>
  3128c1:	42 c6 44 0f 0a 3f    	movb   $0x3f,0xa(%rdi,%r9,1)
  3128c7:	41 c1 ea 08          	shr    $0x8,%r10d
  3128cb:	41 f6 c2 01          	test   $0x1,%r10b
  3128cf:	0f 85 94 fd ff ff    	jne    312669 <<rexx_exec::Interp>::exec_parse+0x2d39>
  3128d5:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  3128db:	41 f6 c2 01          	test   $0x1,%r10b
  3128df:	0f 84 9a fd ff ff    	je     31267f <<rexx_exec::Interp>::exec_parse+0x2d4f>
  3128e5:	42 c6 44 0f 0c 3f    	movb   $0x3f,0xc(%rdi,%r9,1)
  3128eb:	41 c1 ea 08          	shr    $0x8,%r10d
  3128ef:	41 f6 c2 01          	test   $0x1,%r10b
  3128f3:	0f 85 94 fd ff ff    	jne    31268d <<rexx_exec::Interp>::exec_parse+0x2d5d>
  3128f9:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  3128ff:	41 f6 c2 01          	test   $0x1,%r10b
  312903:	0f 84 9a fd ff ff    	je     3126a3 <<rexx_exec::Interp>::exec_parse+0x2d73>
  312909:	42 c6 44 0f 0e 3f    	movb   $0x3f,0xe(%rdi,%r9,1)
  31290f:	41 c1 ea 08          	shr    $0x8,%r10d
  312913:	41 f6 c2 01          	test   $0x1,%r10b
  312917:	0f 85 90 fd ff ff    	jne    3126ad <<rexx_exec::Interp>::exec_parse+0x2d7d>
  31291d:	e9 91 fd ff ff       	jmp    3126b3 <<rexx_exec::Interp>::exec_parse+0x2d83>
  312922:	66 66 66 66 66 2e 0f 	data16 data16 data16 data16 cs nopw 0x0(%rax,%rax,1)
  312929:	1f 84 00 00 00 00 00 
  312930:	45 89 d3             	mov    %r10d,%r11d
  312933:	41 c1 eb 08          	shr    $0x8,%r11d
  312937:	41 f6 c3 01          	test   $0x1,%r11b
  31293b:	0f 84 d4 fd ff ff    	je     312715 <<rexx_exec::Interp>::exec_parse+0x2de5>
  312941:	42 c6 44 0f 11 3f    	movb   $0x3f,0x11(%rdi,%r9,1)
  312947:	45 89 d3             	mov    %r10d,%r11d
  31294a:	41 c1 eb 10          	shr    $0x10,%r11d
  31294e:	41 f6 c3 01          	test   $0x1,%r11b
  312952:	0f 85 ce fd ff ff    	jne    312726 <<rexx_exec::Interp>::exec_parse+0x2df6>
  312958:	41 c1 ea 18          	shr    $0x18,%r10d
  31295c:	41 f6 c2 01          	test   $0x1,%r10b
  312960:	0f 84 d4 fd ff ff    	je     31273a <<rexx_exec::Interp>::exec_parse+0x2e0a>
  312966:	42 c6 44 0f 13 3f    	movb   $0x3f,0x13(%rdi,%r9,1)
  31296c:	66 44 0f c5 d1 02    	pextrw $0x2,%xmm1,%r10d
  312972:	41 f6 c2 01          	test   $0x1,%r10b
  312976:	0f 85 ce fd ff ff    	jne    31274a <<rexx_exec::Interp>::exec_parse+0x2e1a>
  31297c:	41 c1 ea 08          	shr    $0x8,%r10d
  312980:	41 f6 c2 01          	test   $0x1,%r10b
  312984:	0f 84 d4 fd ff ff    	je     31275e <<rexx_exec::Interp>::exec_parse+0x2e2e>
  31298a:	42 c6 44 0f 15 3f    	movb   $0x3f,0x15(%rdi,%r9,1)
  312990:	66 44 0f c5 d1 03    	pextrw $0x3,%xmm1,%r10d
  312996:	41 f6 c2 01          	test   $0x1,%r10b
  31299a:	0f 85 ce fd ff ff    	jne    31276e <<rexx_exec::Interp>::exec_parse+0x2e3e>
  3129a0:	41 c1 ea 08          	shr    $0x8,%r10d
  3129a4:	41 f6 c2 01          	test   $0x1,%r10b
  3129a8:	0f 84 d4 fd ff ff    	je     312782 <<rexx_exec::Interp>::exec_parse+0x2e52>
  3129ae:	42 c6 44 0f 17 3f    	movb   $0x3f,0x17(%rdi,%r9,1)
  3129b4:	66 44 0f c5 d1 04    	pextrw $0x4,%xmm1,%r10d
  3129ba:	41 f6 c2 01          	test   $0x1,%r10b
  3129be:	0f 85 ce fd ff ff    	jne    312792 <<rexx_exec::Interp>::exec_parse+0x2e62>
  3129c4:	41 c1 ea 08          	shr    $0x8,%r10d
  3129c8:	41 f6 c2 01          	test   $0x1,%r10b
  3129cc:	0f 84 d4 fd ff ff    	je     3127a6 <<rexx_exec::Interp>::exec_parse+0x2e76>
  3129d2:	42 c6 44 0f 19 3f    	movb   $0x3f,0x19(%rdi,%r9,1)
  3129d8:	66 44 0f c5 d1 05    	pextrw $0x5,%xmm1,%r10d
  3129de:	41 f6 c2 01          	test   $0x1,%r10b
  3129e2:	0f 85 ce fd ff ff    	jne    3127b6 <<rexx_exec::Interp>::exec_parse+0x2e86>
  3129e8:	41 c1 ea 08          	shr    $0x8,%r10d
  3129ec:	41 f6 c2 01          	test   $0x1,%r10b
  3129f0:	0f 84 d4 fd ff ff    	je     3127ca <<rexx_exec::Interp>::exec_parse+0x2e9a>
  3129f6:	42 c6 44 0f 1b 3f    	movb   $0x3f,0x1b(%rdi,%r9,1)
  3129fc:	66 44 0f c5 d1 06    	pextrw $0x6,%xmm1,%r10d
  312a02:	41 f6 c2 01          	test   $0x1,%r10b
  312a06:	0f 85 ce fd ff ff    	jne    3127da <<rexx_exec::Interp>::exec_parse+0x2eaa>
  312a0c:	41 c1 ea 08          	shr    $0x8,%r10d
  312a10:	41 f6 c2 01          	test   $0x1,%r10b
  312a14:	0f 84 d4 fd ff ff    	je     3127ee <<rexx_exec::Interp>::exec_parse+0x2ebe>
  312a1a:	42 c6 44 0f 1d 3f    	movb   $0x3f,0x1d(%rdi,%r9,1)
  312a20:	66 44 0f c5 d1 07    	pextrw $0x7,%xmm1,%r10d
  312a26:	41 f6 c2 01          	test   $0x1,%r10b
  312a2a:	0f 85 ce fd ff ff    	jne    3127fe <<rexx_exec::Interp>::exec_parse+0x2ece>
  312a30:	41 c1 ea 08          	shr    $0x8,%r10d
  312a34:	41 f6 c2 01          	test   $0x1,%r10b
  312a38:	0f 84 f2 fa ff ff    	je     312530 <<rexx_exec::Interp>::exec_parse+0x2c00>
  312a3e:	42 c6 44 0f 1f 3f    	movb   $0x3f,0x1f(%rdi,%r9,1)
  312a44:	e9 e7 fa ff ff       	jmp    312530 <<rexx_exec::Interp>::exec_parse+0x2c00>
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312a49:	4c 39 c6             	cmp    %r8,%rsi
  312a4c:	0f 84 33 f7 ff ff    	je     312185 <<rexx_exec::Interp>::exec_parse+0x2855>
rexx_exec::error::displayable:
  312a52:	40 f6 c6 10          	test   $0x10,%sil
  312a56:	0f 85 03 f8 ff ff    	jne    31225f <<rexx_exec::Interp>::exec_parse+0x292f>
  312a5c:	48 01 d0             	add    %rdx,%rax
  312a5f:	eb 1b                	jmp    312a7c <<rexx_exec::Interp>::exec_parse+0x314c>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2399
  312a61:	c6 01 3f             	movb   $0x3f,(%rcx)
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
  312a64:	66 66 66 2e 0f 1f 84 	data16 data16 cs nopw 0x0(%rax,%rax,1)
  312a6b:	00 00 00 00 00 
  312a70:	48 ff c1             	inc    %rcx
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312a73:	48 39 c1             	cmp    %rax,%rcx
<core::slice::iter::IterMut<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312a76:	0f 84 09 f7 ff ff    	je     312185 <<rexx_exec::Interp>::exec_parse+0x2855>
rexx_exec::error::displayable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:2398
  312a7c:	0f b6 11             	movzbl (%rcx),%edx
  312a7f:	83 fa 1f             	cmp    $0x1f,%edx
  312a82:	77 ec                	ja     312a70 <<rexx_exec::Interp>::exec_parse+0x3140>
  312a84:	83 fa 0d             	cmp    $0xd,%edx
  312a87:	77 d8                	ja     312a61 <<rexx_exec::Interp>::exec_parse+0x3131>
<rexx_exec::Interp>::assign_targets:
  312a89:	be 00 26 00 00       	mov    $0x2600,%esi
  312a8e:	0f a3 d6             	bt     %edx,%esi
  312a91:	72 dd                	jb     312a70 <<rexx_exec::Interp>::exec_parse+0x3140>
  312a93:	eb cc                	jmp    312a61 <<rexx_exec::Interp>::exec_parse+0x3131>
  312a95:	48 8b bc 24 10 02 00 	mov    0x210(%rsp),%rdi
  312a9c:	00 
  312a9d:	48 8b 8c 24 c0 01 00 	mov    0x1c0(%rsp),%rcx
  312aa4:	00 
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:311
  312aa5:	49 89 e8             	mov    %rbp,%r8
  312aa8:	e8 83 dd e4 ff       	call   160830 <<rexx_core::roots::RootSet>::set_frame_slot_of>
  312aad:	e9 53 f3 ff ff       	jmp    311e05 <<rexx_exec::Interp>::exec_parse+0x24d5>
<rexx_exec::Interp>::exec_parse:
  312ab2:	48 8b 74 24 40       	mov    0x40(%rsp),%rsi
  312ab7:	48 89 f7             	mov    %rsi,%rdi
  312aba:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312abd:	83 fa 01             	cmp    $0x1,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312ac0:	0f 84 d8 eb ff ff    	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312ac6:	41 be 01 00 00 00    	mov    $0x1,%r14d
  312acc:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312ad1:	38 4c 07 01          	cmp    %cl,0x1(%rdi,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312ad5:	0f 84 b4 eb ff ff    	je     31168f <<rexx_exec::Interp>::exec_parse+0x1d5f>
  312adb:	48 89 f7             	mov    %rsi,%rdi
  312ade:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312ae1:	83 fa 02             	cmp    $0x2,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312ae4:	0f 84 b4 eb ff ff    	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312aea:	41 be 02 00 00 00    	mov    $0x2,%r14d
  312af0:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312af5:	38 4c 07 02          	cmp    %cl,0x2(%rdi,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312af9:	0f 84 90 eb ff ff    	je     31168f <<rexx_exec::Interp>::exec_parse+0x1d5f>
  312aff:	48 89 f7             	mov    %rsi,%rdi
  312b02:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312b05:	83 fa 03             	cmp    $0x3,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312b08:	0f 84 90 eb ff ff    	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312b0e:	41 be 03 00 00 00    	mov    $0x3,%r14d
  312b14:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312b19:	38 4c 07 03          	cmp    %cl,0x3(%rdi,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312b1d:	0f 84 6c eb ff ff    	je     31168f <<rexx_exec::Interp>::exec_parse+0x1d5f>
  312b23:	48 89 f7             	mov    %rsi,%rdi
  312b26:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312b29:	83 fa 04             	cmp    $0x4,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312b2c:	0f 84 6c eb ff ff    	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312b32:	41 be 04 00 00 00    	mov    $0x4,%r14d
  312b38:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312b3d:	38 4c 07 04          	cmp    %cl,0x4(%rdi,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312b41:	0f 84 48 eb ff ff    	je     31168f <<rexx_exec::Interp>::exec_parse+0x1d5f>
  312b47:	48 89 f7             	mov    %rsi,%rdi
  312b4a:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312b4d:	83 fa 05             	cmp    $0x5,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312b50:	0f 84 48 eb ff ff    	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312b56:	41 be 05 00 00 00    	mov    $0x5,%r14d
  312b5c:	48 8b 7c 24 20       	mov    0x20(%rsp),%rdi
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312b61:	38 4c 07 05          	cmp    %cl,0x5(%rdi,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312b65:	0f 84 24 eb ff ff    	je     31168f <<rexx_exec::Interp>::exec_parse+0x1d5f>
  312b6b:	48 89 f7             	mov    %rsi,%rdi
  312b6e:	49 89 f0             	mov    %rsi,%r8
<core::ptr::non_null::NonNull<u8> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312b71:	83 fa 06             	cmp    $0x6,%edx
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312b74:	0f 84 24 eb ff ff    	je     31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
  312b7a:	41 be 06 00 00 00    	mov    $0x6,%r14d
  312b80:	48 89 f7             	mov    %rsi,%rdi
  312b83:	49 89 f0             	mov    %rsi,%r8
  312b86:	48 8b 54 24 20       	mov    0x20(%rsp),%rdx
rexx_exec::builtin::string::find_byte::{closure#0}:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/builtin/string.rs:169
  312b8b:	38 4c 02 06          	cmp    %cl,0x6(%rdx,%rax,1)
<core::slice::iter::Iter<u8> as core::iter::traits::iterator::Iterator>::position::<rexx_exec::builtin::string::find_byte::{closure#0}>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:379
  312b8f:	0f 84 fa ea ff ff    	je     31168f <<rexx_exec::Interp>::exec_parse+0x1d5f>
  312b95:	e9 04 eb ff ff       	jmp    31169e <<rexx_exec::Interp>::exec_parse+0x1d6e>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:56
  312b9a:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  312b9f:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  312ba4:	48 8b 54 24 30       	mov    0x30(%rsp),%rdx
  312ba9:	e8 fa b4 f6 ff       	call   27e0a8 <<rexx_exec::Interp>::traced_string_value_text>
<core::result::Result<alloc::vec::Vec<u8>, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312bae:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  312bb3:	4c 8b 6c 24 50       	mov    0x50(%rsp),%r13
  312bb8:	4c 8b 74 24 58       	mov    0x58(%rsp),%r14
  312bbd:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  312bc1:	74 41                	je     312c04 <<rexx_exec::Interp>::exec_parse+0x32d4>
<rexx_exec::Interp>::apply_trigger:
  312bc3:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  312bc8:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  312bcf:	75 1c                	jne    312bed <<rexx_exec::Interp>::exec_parse+0x32bd>
  312bd1:	f6 87 2d 05 00 00 01 	testb  $0x1,0x52d(%rdi)
  312bd8:	74 13                	je     312bed <<rexx_exec::Interp>::exec_parse+0x32bd>
<rexx_exec::Interp>::trace_result:
  312bda:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  312be1:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:225
  312be2:	4c 89 ea             	mov    %r13,%rdx
  312be5:	4c 89 f1             	mov    %r14,%rcx
  312be8:	e8 c3 b4 f6 ff       	call   27e0b0 <<rexx_exec::Interp>::trace_result_line>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312bed:	48 85 ed             	test   %rbp,%rbp
  312bf0:	0f 84 42 de ff ff    	je     310a38 <<rexx_exec::Interp>::exec_parse+0x1108>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312bf6:	4c 89 ef             	mov    %r13,%rdi
  312bf9:	ff 15 69 55 1c 00    	call   *0x1c5569(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  312bff:	e9 34 de ff ff       	jmp    310a38 <<rexx_exec::Interp>::exec_parse+0x1108>
<rexx_exec::Interp>::exec_parse:
  312c04:	4c 89 a4 24 c8 00 00 	mov    %r12,0xc8(%rsp)
  312c0b:	00 
  312c0c:	4c 89 f8             	mov    %r15,%rax
  312c0f:	49 89 df             	mov    %rbx,%r15
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312c12:	49 83 fd ff          	cmp    $0xffffffffffffffff,%r13
  312c16:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  312c1b:	48 8b 6c 24 20       	mov    0x20(%rsp),%rbp
  312c20:	0f 84 41 eb ff ff    	je     311767 <<rexx_exec::Interp>::exec_parse+0x1e37>
  312c26:	e9 28 04 00 00       	jmp    313053 <<rexx_exec::Interp>::exec_parse+0x3723>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  312c2b:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  312c30:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
  312c35:	4c 89 e6             	mov    %r12,%rsi
  312c38:	48 8d 94 24 e0 01 00 	lea    0x1e0(%rsp),%rdx
  312c3f:	00 
  312c40:	8b 8c 24 d4 01 00 00 	mov    0x1d4(%rsp),%ecx
  312c47:	44 8b 84 24 d8 01 00 	mov    0x1d8(%rsp),%r8d
  312c4e:	00 
  312c4f:	4c 8b 8c 24 c0 00 00 	mov    0xc0(%rsp),%r9
  312c56:	00 
  312c57:	e8 74 3d 04 00       	call   3569d0 <<rexx_exec::Interp>::next_template>
<core::result::Result<(rexx_exec::parse_template::SourceText, rexx_exec::parse_template::Cursor), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312c5c:	4c 8b 7c 24 48       	mov    0x48(%rsp),%r15
  312c61:	48 8d 44 24 49       	lea    0x49(%rsp),%rax
  312c66:	0f 10 40 07          	movups 0x7(%rax),%xmm0
  312c6a:	0f 29 84 24 00 01 00 	movaps %xmm0,0x100(%rsp)
  312c71:	00 
  312c72:	49 83 ff ff          	cmp    $0xffffffffffffffff,%r15
  312c76:	0f 84 80 09 00 00    	je     3135fc <<rexx_exec::Interp>::exec_parse+0x3ccc>
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2178
  312c7c:	48 8b 44 24 60       	mov    0x60(%rsp),%rax
  312c81:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
  312c86:	48 8b 44 24 70       	mov    0x70(%rsp),%rax
  312c8b:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  312c90:	48 8b 44 24 78       	mov    0x78(%rsp),%rax
  312c95:	48 89 84 24 c8 00 00 	mov    %rax,0xc8(%rsp)
  312c9c:	00 
  312c9d:	48 8b 84 24 80 00 00 	mov    0x80(%rsp),%rax
  312ca4:	00 
  312ca5:	48 89 84 24 b8 01 00 	mov    %rax,0x1b8(%rsp)
  312cac:	00 
  312cad:	48 8b 9c 24 88 00 00 	mov    0x88(%rsp),%rbx
  312cb4:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  312cb5:	66 0f 6f 84 24 00 01 	movdqa 0x100(%rsp),%xmm0
  312cbc:	00 00 
  312cbe:	66 0f 7f 84 24 50 02 	movdqa %xmm0,0x250(%rsp)
  312cc5:	00 00 
core::mem::replace::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:975
  312cc7:	4c 8b b4 24 d8 00 00 	mov    0xd8(%rsp),%r14
  312cce:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/mem/mod.rs:976
  312ccf:	4c 89 bc 24 d0 00 00 	mov    %r15,0xd0(%rsp)
  312cd6:	00 
  312cd7:	48 8d 84 24 d8 00 00 	lea    0xd8(%rsp),%rax
  312cde:	00 
  312cdf:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  312ce3:	48 8b 8c 24 28 01 00 	mov    0x128(%rsp),%rcx
  312cea:	00 
<rexx_exec::parse_template::SourceText>::into_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:97
  312ceb:	48 85 c9             	test   %rcx,%rcx
  312cee:	0f 88 8e 00 00 00    	js     312d82 <<rexx_exec::Interp>::exec_parse+0x3452>
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312cf4:	48 8d 81 ff ef ff ff 	lea    -0x1001(%rcx),%rax
  312cfb:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  312d01:	77 10                	ja     312d13 <<rexx_exec::Interp>::exec_parse+0x33e3>
  312d03:	48 85 c9             	test   %rcx,%rcx
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312d06:	74 7a                	je     312d82 <<rexx_exec::Interp>::exec_parse+0x3452>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312d08:	4c 89 f7             	mov    %r14,%rdi
  312d0b:	ff 15 57 54 1c 00    	call   *0x1c5457(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312d11:	eb 6f                	jmp    312d82 <<rexx_exec::Interp>::exec_parse+0x3452>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312d13:	4d 8b a4 24 e0 0b 00 	mov    0xbe0(%r12),%r12
  312d1a:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312d1b:	49 83 fc 08          	cmp    $0x8,%r12
  312d1f:	72 10                	jb     312d31 <<rexx_exec::Interp>::exec_parse+0x3401>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312d21:	4c 89 f7             	mov    %r14,%rdi
  312d24:	ff 15 3e 54 1c 00    	call   *0x1c543e(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  312d2a:	4c 8b 64 24 18       	mov    0x18(%rsp),%r12
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312d2f:	eb 51                	jmp    312d82 <<rexx_exec::Interp>::exec_parse+0x3452>
  312d31:	48 8b 84 24 38 01 00 	mov    0x138(%rsp),%rax
  312d38:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312d39:	4c 3b 20             	cmp    (%rax),%r12
  312d3c:	75 0d                	jne    312d4b <<rexx_exec::Interp>::exec_parse+0x341b>
  312d3e:	48 8b bc 24 38 01 00 	mov    0x138(%rsp),%rdi
  312d45:	00 
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312d46:	e8 b5 e3 17 00       	call   491100 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
  312d4b:	48 8b 54 24 18       	mov    0x18(%rsp),%rdx
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312d50:	48 8b 82 d8 0b 00 00 	mov    0xbd8(%rdx),%rax
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  312d57:	4b 8d 0c 64          	lea    (%r12,%r12,2),%rcx
  312d5b:	48 8b b4 24 28 01 00 	mov    0x128(%rsp),%rsi
  312d62:	00 
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312d63:	48 89 34 c8          	mov    %rsi,(%rax,%rcx,8)
  312d67:	4c 89 74 c8 08       	mov    %r14,0x8(%rax,%rcx,8)
  312d6c:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312d73:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312d75:	49 ff c4             	inc    %r12
  312d78:	4c 89 a2 e0 0b 00 00 	mov    %r12,0xbe0(%rdx)
  312d7f:	49 89 d4             	mov    %rdx,%r12
<rexx_exec::Interp>::exec_parse:
  312d82:	48 8b 84 24 48 01 00 	mov    0x148(%rsp),%rax
  312d89:	00 
<core::ptr::non_null::NonNull<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::cmp::PartialEq>::eq:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/non_null.rs:1663
  312d8a:	48 3b 84 24 f8 01 00 	cmp    0x1f8(%rsp),%rax
  312d91:	00 
  312d92:	48 b9 01 00 00 00 00 	movabs $0x8000000000000001,%rcx
  312d99:	00 00 80 
<core::slice::iter::Iter<core::option::Option<rexx_parse::ast::ParseTrigger>> as core::iter::traits::iterator::Iterator>::next:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/iter/macros.rs:180
  312d9c:	0f 85 0f db ff ff    	jne    3108b1 <<rexx_exec::Interp>::exec_parse+0xf81>
  312da2:	e9 f6 00 00 00       	jmp    312e9d <<rexx_exec::Interp>::exec_parse+0x356d>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:482
  312da7:	66 0f 6f 84 24 00 01 	movdqa 0x100(%rsp),%xmm0
  312dae:	00 00 
  312db0:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  312db7:	00 
  312db8:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  312dbc:	eb 44                	jmp    312e02 <<rexx_exec::Interp>::exec_parse+0x34d2>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  312dbe:	48 8b 5c 24 50       	mov    0x50(%rsp),%rbx
core::ptr::drop_glue::<core::option::Option<alloc::vec::Vec<u8>>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312dc3:	49 ff ce             	dec    %r14
  312dc6:	49 83 fe fd          	cmp    $0xfffffffffffffffd,%r14
  312dca:	77 0e                	ja     312dda <<rexx_exec::Interp>::exec_parse+0x34aa>
  312dcc:	48 8b bc 24 f0 00 00 	mov    0xf0(%rsp),%rdi
  312dd3:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312dd4:	ff 15 8e 53 1c 00    	call   *0x1c538e(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  312dda:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  312de1:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  312de2:	4c 89 38             	mov    %r15,(%rax)
  312de5:	48 89 58 08          	mov    %rbx,0x8(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312de9:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  312df0:	00 00 
  312df2:	7e 0e                	jle    312e02 <<rexx_exec::Interp>::exec_parse+0x34d2>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  312df4:	48 8b bc 24 d8 00 00 	mov    0xd8(%rsp),%rdi
  312dfb:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312dfc:	ff 15 66 53 1c 00    	call   *0x1c5366(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  312e02:	48 83 bc 24 e0 01 00 	cmpq   $0x0,0x1e0(%rsp)
  312e09:	00 00 
core::ptr::drop_glue::<rexx_exec::parse_template::ParseStrings>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  312e0b:	0f 8e b6 01 00 00    	jle    312fc7 <<rexx_exec::Interp>::exec_parse+0x3697>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  312e11:	48 8b bc 24 e8 01 00 	mov    0x1e8(%rsp),%rdi
  312e18:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312e19:	ff 15 49 53 1c 00    	call   *0x1c5349(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  312e1f:	e9 a3 01 00 00       	jmp    312fc7 <<rexx_exec::Interp>::exec_parse+0x3697>
<rexx_exec::Interp>::exec_parse:
  312e24:	48 8d 9c 24 10 01 00 	lea    0x110(%rsp),%rbx
  312e2b:	00 
<rexx_core::handle::ObjRef>::decode:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/handle.rs:164
  312e2c:	4c 89 f0             	mov    %r14,%rax
  312e2f:	48 c1 e8 02          	shr    $0x2,%rax
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  312e33:	89 c1                	mov    %eax,%ecx
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:184
  312e35:	48 39 8e 08 0a 00 00 	cmp    %rcx,0xa08(%rsi)
  312e3c:	0f 86 6c d8 ff ff    	jbe    3106ae <<rexx_exec::Interp>::exec_parse+0xd7e>
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:408
  312e42:	48 8b 86 00 0a 00 00 	mov    0xa00(%rsi),%rax
<usize as core::slice::index::SliceIndex<[rexx_core::heap::Slot]>>::get:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:186
  312e49:	48 8d 0c 49          	lea    (%rcx,%rcx,2),%rcx
  312e4d:	48 c1 e1 05          	shl    $0x5,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:641
  312e51:	83 3c 08 01          	cmpl   $0x1,(%rax,%rcx,1)
  312e55:	0f 85 53 d8 ff ff    	jne    3106ae <<rexx_exec::Interp>::exec_parse+0xd7e>
  312e5b:	48 01 c8             	add    %rcx,%rax
  312e5e:	80 78 08 00          	cmpb   $0x0,0x8(%rax)
  312e62:	0f 85 46 d8 ff ff    	jne    3106ae <<rexx_exec::Interp>::exec_parse+0xd7e>
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:408
  312e68:	4c 89 f1             	mov    %r14,%rcx
  312e6b:	48 c1 e9 22          	shr    $0x22,%rcx
<rexx_core::heap::Heap>::body_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/heap.rs:649
  312e6f:	39 48 04             	cmp    %ecx,0x4(%rax)
  312e72:	0f 85 36 d8 ff ff    	jne    3106ae <<rexx_exec::Interp>::exec_parse+0xd7e>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:83
  312e78:	80 78 10 01          	cmpb   $0x1,0x10(%rax)
  312e7c:	0f 85 c0 07 00 00    	jne    313642 <<rexx_exec::Interp>::exec_parse+0x3d12>
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  312e82:	4c 8b 60 28          	mov    0x28(%rax),%r12
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:100
  312e86:	e9 bc 07 00 00       	jmp    313647 <<rexx_exec::Interp>::exec_parse+0x3d17>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2179
  312e8b:	4c 8b 74 24 50       	mov    0x50(%rsp),%r14
<core::result::Result<rexx_core::handle::ObjRef, rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  312e90:	e9 be 01 00 00       	jmp    313053 <<rexx_exec::Interp>::exec_parse+0x3723>
<rexx_exec::Interp>::exec_parse:
  312e95:	4c 8b bc 24 28 01 00 	mov    0x128(%rsp),%r15
  312e9c:	00 
<rexx_exec::parse_template::SourceText>::into_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:97
  312e9d:	4d 85 ff             	test   %r15,%r15
  312ea0:	78 35                	js     312ed7 <<rexx_exec::Interp>::exec_parse+0x35a7>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:502
  312ea2:	48 8b 9c 24 d8 00 00 	mov    0xd8(%rsp),%rbx
  312ea9:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312eaa:	49 8d 87 ff ef ff ff 	lea    -0x1001(%r15),%rax
  312eb1:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  312eb7:	77 07                	ja     312ec0 <<rexx_exec::Interp>::exec_parse+0x3590>
  312eb9:	4d 85 ff             	test   %r15,%r15
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312ebc:	75 10                	jne    312ece <<rexx_exec::Interp>::exec_parse+0x359e>
  312ebe:	eb 17                	jmp    312ed7 <<rexx_exec::Interp>::exec_parse+0x35a7>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312ec0:	4d 8b b4 24 e0 0b 00 	mov    0xbe0(%r12),%r14
  312ec7:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312ec8:	49 83 fe 08          	cmp    $0x8,%r14
  312ecc:	72 57                	jb     312f25 <<rexx_exec::Interp>::exec_parse+0x35f5>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312ece:	48 89 df             	mov    %rbx,%rdi
  312ed1:	ff 15 91 52 1c 00    	call   *0x1c5291(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  312ed7:	4c 8b b4 24 e0 01 00 	mov    0x1e0(%rsp),%r14
  312ede:	00 
  312edf:	4d 85 f6             	test   %r14,%r14
<rexx_exec::Interp>::give_parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:389
  312ee2:	0f 88 d0 00 00 00    	js     312fb8 <<rexx_exec::Interp>::exec_parse+0x3688>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  312ee8:	48 8b 9c 24 e8 01 00 	mov    0x1e8(%rsp),%rbx
  312eef:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:370
  312ef0:	49 8d 86 ff ef ff ff 	lea    -0x1001(%r14),%rax
  312ef7:	48 3d ff ef ff ff    	cmp    $0xffffffffffffefff,%rax
  312efd:	77 0a                	ja     312f09 <<rexx_exec::Interp>::exec_parse+0x35d9>
  312eff:	4d 85 f6             	test   %r14,%r14
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312f02:	75 13                	jne    312f17 <<rexx_exec::Interp>::exec_parse+0x35e7>
  312f04:	e9 af 00 00 00       	jmp    312fb8 <<rexx_exec::Interp>::exec_parse+0x3688>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  312f09:	4d 8b bc 24 e0 0b 00 	mov    0xbe0(%r12),%r15
  312f10:	00 
<rexx_exec::Interp>::give_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:372
  312f11:	49 83 ff 08          	cmp    $0x8,%r15
  312f15:	72 61                	jb     312f78 <<rexx_exec::Interp>::exec_parse+0x3648>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  312f17:	48 89 df             	mov    %rbx,%rdi
  312f1a:	ff 15 48 52 1c 00    	call   *0x1c5248(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  312f20:	e9 93 00 00 00       	jmp    312fb8 <<rexx_exec::Interp>::exec_parse+0x3688>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312f25:	4d 3b b4 24 d0 0b 00 	cmp    0xbd0(%r12),%r14
  312f2c:	00 
  312f2d:	75 0d                	jne    312f3c <<rexx_exec::Interp>::exec_parse+0x360c>
<rexx_exec::Interp>::give_parse_buffer:
  312f2f:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  312f36:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312f37:	e8 c4 e1 17 00       	call   491100 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312f3c:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  312f43:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  312f44:	4b 8d 0c 76          	lea    (%r14,%r14,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312f48:	4c 89 3c c8          	mov    %r15,(%rax,%rcx,8)
  312f4c:	48 89 5c c8 08       	mov    %rbx,0x8(%rax,%rcx,8)
  312f51:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312f58:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312f5a:	49 ff c6             	inc    %r14
  312f5d:	4d 89 b4 24 e0 0b 00 	mov    %r14,0xbe0(%r12)
  312f64:	00 
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:503
  312f65:	4c 8b b4 24 e0 01 00 	mov    0x1e0(%rsp),%r14
  312f6c:	00 
  312f6d:	4d 85 f6             	test   %r14,%r14
<rexx_exec::Interp>::give_parse_strings:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:389
  312f70:	0f 89 72 ff ff ff    	jns    312ee8 <<rexx_exec::Interp>::exec_parse+0x35b8>
  312f76:	eb 40                	jmp    312fb8 <<rexx_exec::Interp>::exec_parse+0x3688>
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  312f78:	4d 3b bc 24 d0 0b 00 	cmp    0xbd0(%r12),%r15
  312f7f:	00 
  312f80:	75 0d                	jne    312f8f <<rexx_exec::Interp>::exec_parse+0x365f>
<rexx_exec::Interp>::give_parse_buffer:
  312f82:	49 8d bc 24 d0 0b 00 	lea    0xbd0(%r12),%rdi
  312f89:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  312f8a:	e8 71 e1 17 00       	call   491100 <<alloc::raw_vec::RawVec<alloc::vec::Vec<u8>>>::grow_one>
<alloc::raw_vec::RawVecInner>::non_null::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  312f8f:	49 8b 84 24 d8 0b 00 	mov    0xbd8(%r12),%rax
  312f96:	00 
<*mut alloc::vec::Vec<u8>>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  312f97:	4b 8d 0c 7f          	lea    (%r15,%r15,2),%rcx
core::ptr::write::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  312f9b:	4c 89 34 c8          	mov    %r14,(%rax,%rcx,8)
  312f9f:	48 89 5c c8 08       	mov    %rbx,0x8(%rax,%rcx,8)
  312fa4:	48 c7 44 c8 10 00 00 	movq   $0x0,0x10(%rax,%rcx,8)
  312fab:	00 00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  312fad:	49 ff c7             	inc    %r15
  312fb0:	4d 89 bc 24 e0 0b 00 	mov    %r15,0xbe0(%r12)
  312fb7:	00 
<rexx_exec::Interp>::exec_parse:
  312fb8:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  312fbf:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:504
  312fc0:	48 c7 00 ff ff ff ff 	movq   $0xffffffffffffffff,(%rax)
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  312fc7:	48 81 c4 68 02 00 00 	add    $0x268,%rsp
  312fce:	5b                   	pop    %rbx
  312fcf:	41 5c                	pop    %r12
  312fd1:	41 5d                	pop    %r13
  312fd3:	41 5e                	pop    %r14
  312fd5:	41 5f                	pop    %r15
  312fd7:	5d                   	pop    %rbp
  312fd8:	c3                   	ret
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  312fd9:	bf 34 00 00 00       	mov    $0x34,%edi
  312fde:	ff 15 a4 51 1c 00    	call   *0x1c51a4(%rip)        # 4d8188 <malloc@GLIBC_2.2.5>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  312fe4:	48 85 c0             	test   %rax,%rax
  312fe7:	0f 84 d0 08 00 00    	je     3138bd <<rexx_exec::Interp>::exec_parse+0x3f8d>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  312fed:	48 89 c3             	mov    %rax,%rbx
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  312ff0:	0f 10 05 d9 76 d5 ff 	movups -0x2a8927(%rip),%xmm0        # 6a6d0 <panic_unwind::imp::CANARY+0x5903>
  312ff7:	0f 11 40 20          	movups %xmm0,0x20(%rax)
  312ffb:	0f 10 05 be 76 d5 ff 	movups -0x2a8942(%rip),%xmm0        # 6a6c0 <panic_unwind::imp::CANARY+0x58f3>
  313002:	0f 11 40 10          	movups %xmm0,0x10(%rax)
  313006:	f3 0f 6f 05 a2 76 d5 	movdqu -0x2a895e(%rip),%xmm0        # 6a6b0 <panic_unwind::imp::CANARY+0x58e3>
  31300d:	ff 
  31300e:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
  313012:	c7 40 30 72 61 6e 64 	movl   $0x646e6172,0x30(%rax)
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  313019:	bf 18 00 00 00       	mov    $0x18,%edi
  31301e:	ff 15 64 51 1c 00    	call   *0x1c5164(%rip)        # 4d8188 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  313024:	48 85 c0             	test   %rax,%rax
  313027:	0f 84 97 07 00 00    	je     3137c4 <<rexx_exec::Interp>::exec_parse+0x3e94>
<alloc::boxed::Box<rexx_exec::Loud>>::new:
  31302d:	49 89 c6             	mov    %rax,%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  313030:	48 c7 00 34 00 00 00 	movq   $0x34,(%rax)
  313037:	48 89 58 08          	mov    %rbx,0x8(%rax)
  31303b:	48 c7 40 10 34 00 00 	movq   $0x34,0x10(%rax)
  313042:	00 
  313043:	41 bd 02 00 00 00    	mov    $0x2,%r13d
<rexx_exec::Interp>::apply_trigger:
  313049:	eb 08                	jmp    313053 <<rexx_exec::Interp>::exec_parse+0x3723>
<rexx_exec::Interp>::eval_in_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/eval.rs:134
  31304b:	4c 8b b4 24 08 01 00 	mov    0x108(%rsp),%r14
  313052:	00 
<rexx_exec::Interp>::exec_parse:
  313053:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  31305a:	00 
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::FromResidual<core::result::Result<core::convert::Infallible, rexx_exec::error::Failure>>>::from_residual:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2193
  31305b:	4c 89 28             	mov    %r13,(%rax)
  31305e:	4c 89 70 08          	mov    %r14,0x8(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313062:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  313069:	00 00 
  31306b:	0f 8f 83 fd ff ff    	jg     312df4 <<rexx_exec::Interp>::exec_parse+0x34c4>
  313071:	e9 8c fd ff ff       	jmp    312e02 <<rexx_exec::Interp>::exec_parse+0x34d2>
<rexx_exec::Interp>::traced_into_parse_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:353
  313076:	e8 b5 36 04 00       	call   356730 <<rexx_exec::Interp>::object_into_parse_buffer>
  31307b:	e9 ca d3 ff ff       	jmp    31044a <<rexx_exec::Interp>::exec_parse+0xb1a>
<rexx_exec::Interp>::parse_strings:
  313080:	48 89 94 24 f8 00 00 	mov    %rdx,0xf8(%rsp)
  313087:	00 
<rexx_exec::Code>::stem:
  313088:	48 01 c8             	add    %rcx,%rax
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/lib.rs:1287
  31308b:	48 8b 48 08          	mov    0x8(%rax),%rcx
  31308f:	48 89 8c 24 c8 01 00 	mov    %rcx,0x1c8(%rsp)
  313096:	00 
  313097:	48 8b 48 10          	mov    0x10(%rax),%rcx
  31309b:	48 89 4c 24 30       	mov    %rcx,0x30(%rsp)
  3130a0:	48 8b 40 18          	mov    0x18(%rax),%rax
  3130a4:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  3130a9:	e9 b0 d1 ff ff       	jmp    31025e <<rexx_exec::Interp>::exec_parse+0x92e>
<core::ops::range::Range<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:413
  3130ae:	48 8d 0d 63 a3 1a 00 	lea    0x1aa363(%rip),%rcx        # 4bd418 <rexx_api::ffi::CALL_CONTEXT+0x1eb8>
  3130b5:	4c 89 ff             	mov    %r15,%rdi
  3130b8:	4c 89 f6             	mov    %r14,%rsi
  3130bb:	48 89 c2             	mov    %rax,%rdx
  3130be:	e8 cd 96 e0 ff       	call   11c790 <core::slice::index::slice_index_fail>
  3130c3:	e9 15 08 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3130c8:	49 8d bf e0 0a 00 00 	lea    0xae0(%r15),%rdi
  3130cf:	e8 ac 8e e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  3130d4:	e9 0b d3 ff ff       	jmp    3103e4 <<rexx_exec::Interp>::exec_parse+0xab4>
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  3130d9:	bf 18 00 00 00       	mov    $0x18,%edi
  3130de:	ff 15 a4 50 1c 00    	call   *0x1c50a4(%rip)        # 4d8188 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  3130e4:	48 85 c0             	test   %rax,%rax
  3130e7:	0f 84 11 07 00 00    	je     3137fe <<rexx_exec::Interp>::exec_parse+0x3ece>
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:95
  3130ed:	49 89 c7             	mov    %rax,%r15
  3130f0:	48 8d bc 24 00 01 00 	lea    0x100(%rsp),%rdi
  3130f7:	00 
  3130f8:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
  3130fd:	4c 89 ea             	mov    %r13,%rdx
  313100:	e8 bb c8 e9 ff       	call   1af9c0 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  313105:	48 8b ac 24 00 01 00 	mov    0x100(%rsp),%rbp
  31310c:	00 
  31310d:	4c 8b b4 24 08 01 00 	mov    0x108(%rsp),%r14
  313114:	00 
  313115:	4c 8b a4 24 10 01 00 	mov    0x110(%rsp),%r12
  31311c:	00 
  31311d:	4d 85 e4             	test   %r12,%r12
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  313120:	0f 89 c5 00 00 00    	jns    3131eb <<rexx_exec::Interp>::exec_parse+0x38bb>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  313126:	45 31 ed             	xor    %r13d,%r13d
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  313129:	4c 89 ef             	mov    %r13,%rdi
  31312c:	4c 89 e6             	mov    %r12,%rsi
  31312f:	e8 40 ea df ff       	call   111b74 <alloc::raw_vec::handle_error>
  313134:	e9 a4 07 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  313139:	0f 84 1f 01 00 00    	je     31325e <<rexx_exec::Interp>::exec_parse+0x392e>
  31313f:	bd 01 00 00 00       	mov    $0x1,%ebp
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  313144:	be 01 00 00 00       	mov    $0x1,%esi
  313149:	48 89 df             	mov    %rbx,%rdi
  31314c:	e8 9f 46 e1 ff       	call   1277f0 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  313151:	48 85 c0             	test   %rax,%rax
  313154:	0f 84 06 d2 ff ff    	je     310360 <<rexx_exec::Interp>::exec_parse+0xa30>
<alloc::vec::Vec<u8>>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:969
  31315a:	48 89 5c 24 48       	mov    %rbx,0x48(%rsp)
  31315f:	48 89 44 24 50       	mov    %rax,0x50(%rsp)
  313164:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
  313169:	48 89 c7             	mov    %rax,%rdi
  31316c:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  313171:	48 89 da             	mov    %rbx,%rdx
  313174:	ff 15 1e 50 1c 00    	call   *0x1c501e(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<alloc::vec::Vec<u8>>::set_len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2224
  31317a:	48 89 5c 24 58       	mov    %rbx,0x58(%rsp)
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  31317f:	e9 ff 00 00 00       	jmp    313283 <<rexx_exec::Interp>::exec_parse+0x3953>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  313184:	0f 84 8a 02 00 00    	je     313414 <<rexx_exec::Interp>::exec_parse+0x3ae4>
  31318a:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  31318f:	be 01 00 00 00       	mov    $0x1,%esi
  313194:	48 89 ef             	mov    %rbp,%rdi
  313197:	e8 54 46 e1 ff       	call   1277f0 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  31319c:	48 85 c0             	test   %rax,%rax
  31319f:	0f 84 b8 cf ff ff    	je     31015d <<rexx_exec::Interp>::exec_parse+0x82d>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  3131a5:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  3131a8:	48 89 c7             	mov    %rax,%rdi
  3131ab:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
  3131b0:	48 89 ea             	mov    %rbp,%rdx
  3131b3:	ff 15 df 4f 1c 00    	call   *0x1c4fdf(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  3131b9:	e9 5c 02 00 00       	jmp    31341a <<rexx_exec::Interp>::exec_parse+0x3aea>
<rexx_exec::Interp>::assign_targets:
  3131be:	48 8d 0d c3 76 1a 00 	lea    0x1a76c3(%rip),%rcx        # 4ba888 <rexx_api::ffi::INSTANCE+0x5ef8>
  3131c5:	ba 07 00 00 00       	mov    $0x7,%edx
  3131ca:	31 ff                	xor    %edi,%edi
  3131cc:	48 89 c6             	mov    %rax,%rsi
  3131cf:	e8 bc 95 e0 ff       	call   11c790 <core::slice::index::slice_index_fail>
  3131d4:	e9 04 07 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  3131d9:	49 8d bc 24 e0 0a 00 	lea    0xae0(%r12),%rdi
  3131e0:	00 
  3131e1:	e8 9a 8d e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  3131e6:	e9 c3 d3 ff ff       	jmp    3105ae <<rexx_exec::Interp>::exec_parse+0xc7e>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  3131eb:	0f 84 d7 02 00 00    	je     3134c8 <<rexx_exec::Interp>::exec_parse+0x3b98>
  3131f1:	41 bd 01 00 00 00    	mov    $0x1,%r13d
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  3131f7:	be 01 00 00 00       	mov    $0x1,%esi
  3131fc:	4c 89 e7             	mov    %r12,%rdi
  3131ff:	e8 ec 45 e1 ff       	call   1277f0 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  313204:	48 85 c0             	test   %rax,%rax
  313207:	0f 84 1c ff ff ff    	je     313129 <<rexx_exec::Interp>::exec_parse+0x37f9>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  31320d:	48 89 c3             	mov    %rax,%rbx
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  313210:	48 89 c7             	mov    %rax,%rdi
  313213:	4c 89 f6             	mov    %r14,%rsi
  313216:	4c 89 e2             	mov    %r12,%rdx
  313219:	ff 15 79 4f 1c 00    	call   *0x1c4f79(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  31321f:	e9 a9 02 00 00       	jmp    3134cd <<rexx_exec::Interp>::exec_parse+0x3b9d>
<core::option::Option<&rexx_exec::activation::Activation>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  313224:	48 8d 3d 9f 6c d5 ff 	lea    -0x2a9361(%rip),%rdi        # 69eca <panic_unwind::imp::CANARY+0x50fd>
  31322b:	48 8d 15 46 d1 1a 00 	lea    0x1ad146(%rip),%rdx        # 4c0378 <rexx_api::ffi::CALL_CONTEXT+0x4e18>
  313232:	be 11 00 00 00       	mov    $0x11,%esi
  313237:	e8 44 be e0 ff       	call   11f080 <core::option::expect_failed>
  31323c:	e9 9c 06 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  313241:	48 8d 3d 5f 5e d5 ff 	lea    -0x2aa1a1(%rip),%rdi        # 690a7 <panic_unwind::imp::CANARY+0x42da>
  313248:	48 8d 15 11 a2 1a 00 	lea    0x1aa211(%rip),%rdx        # 4bd460 <rexx_api::ffi::CALL_CONTEXT+0x1f00>
  31324f:	be 36 00 00 00       	mov    $0x36,%esi
  313254:	e8 27 be e0 ff       	call   11f080 <core::option::expect_failed>
  313259:	e9 7f 06 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::vec::Vec<u8>>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:969
  31325e:	48 c7 44 24 48 00 00 	movq   $0x0,0x48(%rsp)
  313265:	00 00 
  313267:	48 c7 44 24 50 01 00 	movq   $0x1,0x50(%rsp)
  31326e:	00 00 
  313270:	48 c7 44 24 58 00 00 	movq   $0x0,0x58(%rsp)
  313277:	00 00 
  313279:	b8 01 00 00 00       	mov    $0x1,%eax
  31327e:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  313283:	48 8b ac 24 80 01 00 	mov    0x180(%rsp),%rbp
  31328a:	00 
<alloc::raw_vec::RawVecInner>::needs_to_grow:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:764
  31328b:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:670
  31328e:	0f 85 7e 05 00 00    	jne    313812 <<rexx_exec::Interp>::exec_parse+0x3ee2>
<alloc::vec::Vec<u8>>::append_elements_unreserved:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:2986
  313294:	48 01 eb             	add    %rbp,%rbx
  313297:	48 89 5c 24 58       	mov    %rbx,0x58(%rsp)
  31329c:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  3132a1:	41 80 be 35 05 00 00 	cmpb   $0x0,0x535(%r14)
  3132a8:	00 
  3132a9:	75 55                	jne    313300 <<rexx_exec::Interp>::exec_parse+0x39d0>
  3132ab:	41 f6 86 2e 05 00 00 	testb  $0x1,0x52e(%r14)
  3132b2:	01 
  3132b3:	74 4b                	je     313300 <<rexx_exec::Interp>::exec_parse+0x39d0>
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  3132b5:	49 8d be d8 10 00 00 	lea    0x10d8(%r14),%rdi
  3132bc:	49 8b ae e8 10 00 00 	mov    0x10e8(%r14),%rbp
<rexx_exec::Interp>::trace_compound_name:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:594
  3132c3:	48 89 5c 24 10       	mov    %rbx,0x10(%rsp)
  3132c8:	48 8b 44 24 40       	mov    0x40(%rsp),%rax
  3132cd:	48 89 44 24 08       	mov    %rax,0x8(%rsp)
  3132d2:	c7 04 24 20 3d 3e 20 	movl   $0x203e3d20,(%rsp)
  3132d9:	48 8d 35 35 79 d5 ff 	lea    -0x2a86cb(%rip),%rsi        # 6ac15 <panic_unwind::imp::CANARY+0x5e48>
  3132e0:	48 8b 94 24 c0 00 00 	mov    0xc0(%rsp),%rdx
  3132e7:	00 
  3132e8:	31 c9                	xor    %ecx,%ecx
  3132ea:	4d 89 e8             	mov    %r13,%r8
  3132ed:	4d 89 e1             	mov    %r12,%r9
  3132f0:	e8 6b d5 f6 ff       	call   280860 <rexx_exec::trace::format::push_tagged>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:595
  3132f5:	4c 89 f7             	mov    %r14,%rdi
  3132f8:	48 89 ee             	mov    %rbp,%rsi
  3132fb:	e8 c0 ae e8 ff       	call   19e1c0 <<rexx_exec::Interp>::route_trace_line>
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313300:	48 83 7c 24 48 00    	cmpq   $0x0,0x48(%rsp)
  313306:	74 0b                	je     313313 <<rexx_exec::Interp>::exec_parse+0x39e3>
<alloc::raw_vec::RawVecInner>::deallocate:
  313308:	48 8b 7c 24 40       	mov    0x40(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31330d:	ff 15 55 4e 1c 00    	call   *0x1c4e55(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  313313:	4c 8b 74 24 18       	mov    0x18(%rsp),%r14
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  313318:	4c 8b 8c 24 78 01 00 	mov    0x178(%rsp),%r9
  31331f:	00 
<alloc::vec::Vec<u8>>::as_slice:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1865
  313320:	48 8b 84 24 80 01 00 	mov    0x180(%rsp),%rax
  313327:	00 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:706
  313328:	48 89 04 24          	mov    %rax,(%rsp)
  31332c:	4c 89 f7             	mov    %r14,%rdi
  31332f:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
  313334:	48 8b 54 24 28       	mov    0x28(%rsp),%rdx
  313339:	48 8b 8c 24 f8 00 00 	mov    0xf8(%rsp),%rcx
  313340:	00 
  313341:	4c 8b 84 24 c8 01 00 	mov    0x1c8(%rsp),%r8
  313348:	00 
  313349:	e8 72 e1 fa ff       	call   2c14c0 <<rexx_exec::Interp>::stem_get_at>
  31334e:	49 89 c6             	mov    %rax,%r14
  313351:	89 d3                	mov    %edx,%ebx
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313353:	49 83 3f 00          	cmpq   $0x0,(%r15)
  313357:	74 12                	je     31336b <<rexx_exec::Interp>::exec_parse+0x3a3b>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
  313359:	48 8b 44 24 18       	mov    0x18(%rsp),%rax
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  31335e:	48 8b b8 c0 0b 00 00 	mov    0xbc0(%rax),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313365:	ff 15 fd 4d 1c 00    	call   *0x1c4dfd(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::give_key_buffer:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/stem.rs:64
  31336b:	48 8b 84 24 80 01 00 	mov    0x180(%rsp),%rax
  313372:	00 
  313373:	49 89 47 10          	mov    %rax,0x10(%r15)
  313377:	66 0f 6f 84 24 70 01 	movdqa 0x170(%rsp),%xmm0
  31337e:	00 00 
  313380:	f3 41 0f 7f 07       	movdqu %xmm0,(%r15)
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:315
  313385:	f6 c3 01             	test   $0x1,%bl
  313388:	0f 85 4a 04 00 00    	jne    3137d8 <<rexx_exec::Interp>::exec_parse+0x3ea8>
<rexx_exec::Interp>::read_parse_var:
  31338e:	4c 8b 7c 24 18       	mov    0x18(%rsp),%r15
<rexx_exec::trace::format::ChunkTrace>::intermediates:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:244
  313393:	41 f6 87 36 05 00 00 	testb  $0x4,0x536(%r15)
  31339a:	04 
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:709
  31339b:	0f 84 2f d0 ff ff    	je     3103d0 <<rexx_exec::Interp>::exec_parse+0xaa0>
  3133a1:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:710
  3133a6:	4c 89 fe             	mov    %r15,%rsi
  3133a9:	4c 89 f2             	mov    %r14,%rdx
  3133ac:	e8 0f c6 e9 ff       	call   1af9c0 <<rexx_exec::Interp>::to_text>
<alloc::borrow::Cow<[u8]> as core::ops::deref::Deref>::deref:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/borrow.rs:350
  3133b1:	48 8b 44 24 48       	mov    0x48(%rsp),%rax
  3133b6:	48 89 44 24 28       	mov    %rax,0x28(%rsp)
  3133bb:	48 8b 44 24 50       	mov    0x50(%rsp),%rax
  3133c0:	48 89 44 24 30       	mov    %rax,0x30(%rsp)
  3133c5:	48 8b 6c 24 58       	mov    0x58(%rsp),%rbp
  3133ca:	48 85 ed             	test   %rbp,%rbp
core::intrinsics::unlikely:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/intrinsics/mod.rs:478
  3133cd:	79 12                	jns    3133e1 <<rexx_exec::Interp>::exec_parse+0x3ab1>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
  3133cf:	31 db                	xor    %ebx,%ebx
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  3133d1:	48 89 df             	mov    %rbx,%rdi
  3133d4:	48 89 ee             	mov    %rbp,%rsi
  3133d7:	e8 98 e7 df ff       	call   111b74 <alloc::raw_vec::handle_error>
  3133dc:	e9 fc 04 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  3133e1:	0f 84 87 00 00 00    	je     31346e <<rexx_exec::Interp>::exec_parse+0x3b3e>
  3133e7:	bb 01 00 00 00       	mov    $0x1,%ebx
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  3133ec:	be 01 00 00 00       	mov    $0x1,%esi
  3133f1:	48 89 ef             	mov    %rbp,%rdi
  3133f4:	e8 f7 43 e1 ff       	call   1277f0 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  3133f9:	48 85 c0             	test   %rax,%rax
  3133fc:	74 d3                	je     3133d1 <<rexx_exec::Interp>::exec_parse+0x3aa1>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
  3133fe:	49 89 c7             	mov    %rax,%r15
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  313401:	48 89 c7             	mov    %rax,%rdi
  313404:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
  313409:	48 89 ea             	mov    %rbp,%rdx
  31340c:	ff 15 86 4d 1c 00    	call   *0x1c4d86(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
<u8 as <[_]>::to_vec_in::ConvertVec>::to_vec::<alloc::alloc::Global>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/slice.rs:452
  313412:	eb 60                	jmp    313474 <<rexx_exec::Interp>::exec_parse+0x3b44>
<rexx_exec::Interp>::read_parse_var:
  313414:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  31341a:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  313420:	7e 0b                	jle    31342d <<rexx_exec::Interp>::exec_parse+0x3afd>
  313422:	48 8b 7c 24 30       	mov    0x30(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313427:	ff 15 3b 4d 1c 00    	call   *0x1c4d3b(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  31342d:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  313432:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  313439:	0f 85 7e cf ff ff    	jne    3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
  31343f:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  313446:	0f 84 71 cf ff ff    	je     3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  31344c:	4c 89 34 24          	mov    %r14,(%rsp)
  313450:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  313457:	00 
  313458:	4c 89 ea             	mov    %r13,%rdx
  31345b:	4c 89 e1             	mov    %r12,%rcx
  31345e:	4d 89 f8             	mov    %r15,%r8
  313461:	49 89 e9             	mov    %rbp,%r9
  313464:	e8 f7 dd f6 ff       	call   281260 <<rexx_exec::Interp>::trace_variable_line>
  313469:	e9 4f cf ff ff       	jmp    3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
<rexx_exec::Interp>::read_parse_var:
  31346e:	41 bf 01 00 00 00    	mov    $0x1,%r15d
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313474:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  31347a:	7e 0b                	jle    313487 <<rexx_exec::Interp>::exec_parse+0x3b57>
  31347c:	48 8b 7c 24 30       	mov    0x30(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313481:	ff 15 e1 4c 1c 00    	call   *0x1c4ce1(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::read_parse_var:
  313487:	48 8b 7c 24 18       	mov    0x18(%rsp),%rdi
<rexx_exec::trace::format::TraceCache>::traced:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace/format.rs:306
  31348c:	80 bf 35 05 00 00 00 	cmpb   $0x0,0x535(%rdi)
  313493:	0f 85 24 cf ff ff    	jne    3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
  313499:	f6 87 2e 05 00 00 01 	testb  $0x1,0x52e(%rdi)
  3134a0:	0f 84 17 cf ff ff    	je     3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
<rexx_exec::Interp>::trace_variable:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/trace.rs:386
  3134a6:	4c 89 34 24          	mov    %r14,(%rsp)
  3134aa:	48 8b b4 24 c0 00 00 	mov    0xc0(%rsp),%rsi
  3134b1:	00 
  3134b2:	4c 89 ea             	mov    %r13,%rdx
  3134b5:	4c 89 e1             	mov    %r12,%rcx
  3134b8:	4d 89 f8             	mov    %r15,%r8
  3134bb:	49 89 e9             	mov    %rbp,%r9
  3134be:	e8 9d dd f6 ff       	call   281260 <<rexx_exec::Interp>::trace_variable_line>
  3134c3:	e9 f5 ce ff ff       	jmp    3103bd <<rexx_exec::Interp>::exec_parse+0xa8d>
<rexx_exec::Interp>::exec_parse:
  3134c8:	bb 01 00 00 00       	mov    $0x1,%ebx
  3134cd:	4d 89 f5             	mov    %r14,%r13
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:95
  3134d0:	4d 89 27             	mov    %r12,(%r15)
  3134d3:	49 89 5f 08          	mov    %rbx,0x8(%r15)
  3134d7:	4d 89 67 10          	mov    %r12,0x10(%r15)
alloc::alloc::alloc:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:130
  3134db:	bf 02 00 00 00       	mov    $0x2,%edi
  3134e0:	be 01 00 00 00       	mov    $0x1,%esi
  3134e5:	e8 06 43 e1 ff       	call   1277f0 <__rustc::__rust_alloc>
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:470
  3134ea:	48 85 c0             	test   %rax,%rax
  3134ed:	0f 84 db 03 00 00    	je     3138ce <<rexx_exec::Interp>::exec_parse+0x3f9e>
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  3134f3:	66 c7 00 32 36       	movw   $0x3632,(%rax)
<rexx_exec::error::Raised>::syntax:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/error.rs:139
  3134f8:	48 c7 44 24 60 ff ff 	movq   $0xffffffffffffffff,0x60(%rsp)
  3134ff:	ff ff 
  313501:	48 8d 0d 7f 37 d9 ff 	lea    -0x26c881(%rip),%rcx        # a6c87 <panic_unwind::imp::CANARY+0x41eba>
  313508:	48 89 4c 24 68       	mov    %rcx,0x68(%rsp)
  31350d:	48 c7 44 24 70 06 00 	movq   $0x6,0x70(%rsp)
  313514:	00 00 
  313516:	48 c7 44 24 48 01 00 	movq   $0x1,0x48(%rsp)
  31351d:	00 00 
  31351f:	4c 89 7c 24 50       	mov    %r15,0x50(%rsp)
  313524:	48 c7 44 24 58 01 00 	movq   $0x1,0x58(%rsp)
  31352b:	00 00 
  31352d:	48 c7 44 24 78 02 00 	movq   $0x2,0x78(%rsp)
  313534:	00 00 
  313536:	48 89 84 24 80 00 00 	mov    %rax,0x80(%rsp)
  31353d:	00 
  31353e:	48 c7 84 24 88 00 00 	movq   $0x2,0x88(%rsp)
  313545:	00 02 00 00 00 
  31354a:	48 c7 84 24 90 00 00 	movq   $0xffffffffffffffff,0x90(%rsp)
  313551:	00 ff ff ff ff 
  313556:	48 b8 00 00 00 00 1a 	movabs $0x4001a00000000,%rax
  31355d:	00 04 00 
  313560:	48 89 84 24 a8 00 00 	mov    %rax,0xa8(%rsp)
  313567:	00 
  313568:	c7 84 24 b0 00 00 00 	movl   $0x0,0xb0(%rsp)
  31356f:	00 00 00 00 
  313573:	66 c7 84 24 b4 00 00 	movw   $0x0,0xb4(%rsp)
  31357a:	00 00 00 
std::sys::alloc::unix::alloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:31
  31357d:	bf 70 00 00 00       	mov    $0x70,%edi
  313582:	ff 15 00 4c 1c 00    	call   *0x1c4c00(%rip)        # 4d8188 <malloc@GLIBC_2.2.5>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:251
  313588:	48 85 c0             	test   %rax,%rax
  31358b:	0f 84 d8 02 00 00    	je     313869 <<rexx_exec::Interp>::exec_parse+0x3f39>
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
  313591:	49 89 c6             	mov    %rax,%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:293
  313594:	0f 10 84 24 a8 00 00 	movups 0xa8(%rsp),%xmm0
  31359b:	00 
  31359c:	0f 11 40 60          	movups %xmm0,0x60(%rax)
  3135a0:	0f 10 84 24 98 00 00 	movups 0x98(%rsp),%xmm0
  3135a7:	00 
  3135a8:	0f 11 40 50          	movups %xmm0,0x50(%rax)
  3135ac:	0f 10 84 24 88 00 00 	movups 0x88(%rsp),%xmm0
  3135b3:	00 
  3135b4:	0f 11 40 40          	movups %xmm0,0x40(%rax)
  3135b8:	f3 0f 6f 44 24 48    	movdqu 0x48(%rsp),%xmm0
  3135be:	f3 0f 6f 4c 24 58    	movdqu 0x58(%rsp),%xmm1
  3135c4:	f3 0f 6f 54 24 68    	movdqu 0x68(%rsp),%xmm2
  3135ca:	f3 0f 6f 5c 24 78    	movdqu 0x78(%rsp),%xmm3
  3135d0:	f3 0f 7f 58 30       	movdqu %xmm3,0x30(%rax)
  3135d5:	f3 0f 7f 50 20       	movdqu %xmm2,0x20(%rax)
  3135da:	f3 0f 7f 48 10       	movdqu %xmm1,0x10(%rax)
  3135df:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  3135e3:	48 85 ed             	test   %rbp,%rbp
  3135e6:	7e 09                	jle    3135f1 <<rexx_exec::Interp>::exec_parse+0x3cc1>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3135e8:	4c 89 ef             	mov    %r13,%rdi
  3135eb:	ff 15 77 4b 1c 00    	call   *0x1c4b77(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<rexx_exec::Interp>::exec_parse:
  3135f1:	41 bd 03 00 00 00    	mov    $0x3,%r13d
  3135f7:	e9 57 fa ff ff       	jmp    313053 <<rexx_exec::Interp>::exec_parse+0x3723>
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:490
  3135fc:	66 0f 6f 84 24 00 01 	movdqa 0x100(%rsp),%xmm0
  313603:	00 00 
  313605:	48 8b 84 24 30 01 00 	mov    0x130(%rsp),%rax
  31360c:	00 
  31360d:	f3 0f 7f 00          	movdqu %xmm0,(%rax)
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313611:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  313618:	00 00 
  31361a:	0f 8f d4 f7 ff ff    	jg     312df4 <<rexx_exec::Interp>::exec_parse+0x34c4>
  313620:	e9 dd f7 ff ff       	jmp    312e02 <<rexx_exec::Interp>::exec_parse+0x34d2>
<rexx_core::roots::RootSet>::set_frame_slot:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/roots.rs:307
  313625:	48 8d 3d c2 76 d5 ff 	lea    -0x2a893e(%rip),%rdi        # 6acee <panic_unwind::imp::CANARY+0x5f21>
  31362c:	48 8d 15 65 d4 1a 00 	lea    0x1ad465(%rip),%rdx        # 4c0a98 <rexx_api::ffi::CALL_CONTEXT+0x5538>
  313633:	be 31 00 00 00       	mov    $0x31,%esi
  313638:	e8 23 92 e0 ff       	call   11c860 <core::panicking::panic>
  31363d:	e9 9b 02 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_core::bytes::Bytes>::as_slice:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-core/src/bytes.rs:85
  313642:	44 0f b6 60 11       	movzbl 0x11(%rax),%r12d
<rexx_exec::Interp>::source_text:
  313647:	48 8b 6c 24 18       	mov    0x18(%rsp),%rbp
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1029
  31364c:	4c 8b ad f0 0a 00 00 	mov    0xaf0(%rbp),%r13
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1032
  313653:	4c 3b ad e0 0a 00 00 	cmp    0xae0(%rbp),%r13
  31365a:	74 43                	je     31369f <<rexx_exec::Interp>::exec_parse+0x3d6f>
  31365c:	49 bf 01 00 00 00 00 	movabs $0x8000000000000001,%r15
  313663:	00 00 80 
<alloc::raw_vec::RawVecInner>::non_null::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  313666:	48 8b 85 e8 0a 00 00 	mov    0xae8(%rbp),%rax
core::ptr::write::<rexx_core::handle::ObjRef>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:1963
  31366d:	4e 89 34 e8          	mov    %r14,(%rax,%r13,8)
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1038
  313671:	49 ff c5             	inc    %r13
  313674:	4c 89 ad f0 0a 00 00 	mov    %r13,0xaf0(%rbp)
<rexx_exec::Interp>::source_text:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:414
  31367b:	4c 89 b4 24 08 01 00 	mov    %r14,0x108(%rsp)
  313682:	00 
  313683:	e9 9c d0 ff ff       	jmp    310724 <<rexx_exec::Interp>::exec_parse+0xdf4>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:558
  313688:	48 8d 0d 79 0d 1b 00 	lea    0x1b0d79(%rip),%rcx        # 4c4408 <rexx_api::ffi::CALL_CONTEXT+0x8ea8>
  31368f:	48 89 ef             	mov    %rbp,%rdi
  313692:	48 89 d6             	mov    %rdx,%rsi
  313695:	e8 f6 90 e0 ff       	call   11c790 <core::slice::index::slice_index_fail>
  31369a:	e9 3e 02 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::vec::Vec<rexx_core::handle::ObjRef>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1033
  31369f:	48 8d bd e0 0a 00 00 	lea    0xae0(%rbp),%rdi
  3136a6:	e8 d5 88 e4 ff       	call   15bf80 <<alloc::raw_vec::RawVec<core::ptr::non_null::NonNull<core::cell::Cell<rexx_core::handle::ObjRef>>>>::grow_one>
  3136ab:	eb af                	jmp    31365c <<rexx_exec::Interp>::exec_parse+0x3d2c>
<rexx_exec::Interp>::parse_strings:
  3136ad:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:318
  3136b2:	4c 89 fe             	mov    %r15,%rsi
  3136b5:	4c 89 f2             	mov    %r14,%rdx
  3136b8:	e8 03 f4 f6 ff       	call   282ac0 <<rexx_exec::Interp>::novalue_raised>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3136bd:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  3136c2:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  3136c6:	0f 85 13 c5 ff ff    	jne    30fbdf <<rexx_exec::Interp>::exec_parse+0x2af>
  3136cc:	e9 c1 ca ff ff       	jmp    310192 <<rexx_exec::Interp>::exec_parse+0x862>
<rexx_exec::Interp>::exec_parse:
  3136d1:	49 89 dc             	mov    %rbx,%r12
<rexx_exec::Interp>::apply_trigger:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:69
  3136d4:	48 8d 0d ad 71 1a 00 	lea    0x1a71ad(%rip),%rcx        # 4ba888 <rexx_api::ffi::INSTANCE+0x5ef8>
  3136db:	ba 07 00 00 00       	mov    $0x7,%edx
  3136e0:	31 ff                	xor    %edi,%edi
  3136e2:	4c 89 e6             	mov    %r12,%rsi
  3136e5:	e8 a6 90 e0 ff       	call   11c790 <core::slice::index::slice_index_fail>
  3136ea:	e9 ee 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
  3136ef:	49 89 dc             	mov    %rbx,%r12
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template/trigger.rs:79
  3136f2:	48 8d 0d 8f 71 1a 00 	lea    0x1a718f(%rip),%rcx        # 4ba888 <rexx_api::ffi::INSTANCE+0x5ef8>
  3136f9:	ba 07 00 00 00       	mov    $0x7,%edx
  3136fe:	31 ff                	xor    %edi,%edi
  313700:	4c 89 e6             	mov    %r12,%rsi
  313703:	e8 88 90 e0 ff       	call   11c790 <core::slice::index::slice_index_fail>
  313708:	e9 d0 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:545
  31370d:	48 8d 0d cc a3 1a 00 	lea    0x1aa3cc(%rip),%rcx        # 4bdae0 <rexx_api::ffi::CALL_CONTEXT+0x2580>
  313714:	be 14 00 00 00       	mov    $0x14,%esi
  313719:	ba 14 00 00 00       	mov    $0x14,%edx
  31371e:	4c 89 f7             	mov    %r14,%rdi
  313721:	e8 6a 90 e0 ff       	call   11c790 <core::slice::index::slice_index_fail>
  313726:	e9 b2 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<core::option::Option<&[u8]>>::expect:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs:969
  31372b:	48 8d 3d a5 5d d5 ff 	lea    -0x2aa25b(%rip),%rdi        # 694d7 <panic_unwind::imp::CANARY+0x470a>
  313732:	48 8d 15 8f a3 1a 00 	lea    0x1aa38f(%rip),%rdx        # 4bdac8 <rexx_api::ffi::CALL_CONTEXT+0x2568>
  313739:	be 35 00 00 00       	mov    $0x35,%esi
  31373e:	e8 3d b9 e0 ff       	call   11f080 <core::option::expect_failed>
  313743:	e9 95 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
  313748:	48 8d 3d 88 5d d5 ff 	lea    -0x2aa278(%rip),%rdi        # 694d7 <panic_unwind::imp::CANARY+0x470a>
  31374f:	48 8d 15 72 a3 1a 00 	lea    0x1aa372(%rip),%rdx        # 4bdac8 <rexx_api::ffi::CALL_CONTEXT+0x2568>
  313756:	be 35 00 00 00       	mov    $0x35,%esi
  31375b:	e8 20 b9 e0 ff       	call   11f080 <core::option::expect_failed>
  313760:	e9 78 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<core::ops::range::RangeFrom<usize> as core::slice::index::SliceIndex<[u8]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:545
  313765:	48 8d 0d 74 a3 1a 00 	lea    0x1aa374(%rip),%rcx        # 4bdae0 <rexx_api::ffi::CALL_CONTEXT+0x2580>
  31376c:	be 14 00 00 00       	mov    $0x14,%esi
  313771:	ba 14 00 00 00       	mov    $0x14,%edx
  313776:	4c 89 ef             	mov    %r13,%rdi
  313779:	e8 12 90 e0 ff       	call   11c790 <core::slice::index::slice_index_fail>
  31377e:	e9 5a 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::Interp>::assign_targets:
  313783:	48 8d 3d 12 4d d5 ff 	lea    -0x2ab2ee(%rip),%rdi        # 6849c <panic_unwind::imp::CANARY+0x36cf>
  31378a:	48 8d 15 47 93 1a 00 	lea    0x1a9347(%rip),%rdx        # 4bcad8 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  313791:	be ab 00 00 00       	mov    $0xab,%esi
  313796:	e8 15 72 e0 ff       	call   11a9b0 <core::panicking::panic_fmt>
  31379b:	e9 3d 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
  3137a0:	48 8d 0d 99 82 1a 00 	lea    0x1a8299(%rip),%rcx        # 4bba40 <rexx_api::ffi::CALL_CONTEXT+0x4e0>
  3137a7:	48 89 8c 24 b0 01 00 	mov    %rcx,0x1b0(%rsp)
  3137ae:	00 
  3137af:	48 89 c6             	mov    %rax,%rsi
  3137b2:	48 8b 94 24 b0 01 00 	mov    0x1b0(%rsp),%rdx
  3137b9:	00 
  3137ba:	e8 b2 90 e0 ff       	call   11c871 <core::panicking::panic_bounds_check>
  3137bf:	e9 19 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  3137c4:	bf 08 00 00 00       	mov    $0x8,%edi
  3137c9:	be 18 00 00 00       	mov    $0x18,%esi
  3137ce:	e8 b4 e3 df ff       	call   111b87 <alloc::alloc::handle_alloc_error>
  3137d3:	e9 05 01 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::Interp>::exec_parse:
  3137d8:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
  3137dd:	48 8b 74 24 18       	mov    0x18(%rsp),%rsi
<rexx_exec::Interp>::novalue_check:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/run/condition.rs:318
  3137e2:	4c 89 f2             	mov    %r14,%rdx
  3137e5:	e8 d6 f2 f6 ff       	call   282ac0 <<rexx_exec::Interp>::novalue_raised>
<core::result::Result<(), rexx_exec::error::Failure> as core::ops::try_trait::Try>::branch:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:2177
  3137ea:	48 8b 6c 24 48       	mov    0x48(%rsp),%rbp
  3137ef:	48 83 fd ff          	cmp    $0xffffffffffffffff,%rbp
  3137f3:	0f 85 e6 c3 ff ff    	jne    30fbdf <<rexx_exec::Interp>::exec_parse+0x2af>
  3137f9:	e9 90 fb ff ff       	jmp    31338e <<rexx_exec::Interp>::exec_parse+0x3a5e>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  3137fe:	bf 08 00 00 00       	mov    $0x8,%edi
  313803:	be 18 00 00 00       	mov    $0x18,%esi
  313808:	e8 7a e3 df ff       	call   111b87 <alloc::alloc::handle_alloc_error>
  31380d:	e9 cb 00 00 00       	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::Interp>::read_parse_var:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:703
  313812:	4c 8b b4 24 78 01 00 	mov    0x178(%rsp),%r14
  313819:	00 
  31381a:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<alloc::raw_vec::RawVecInner>::reserve:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:672
  31381f:	b9 01 00 00 00       	mov    $0x1,%ecx
  313824:	41 b8 01 00 00 00    	mov    $0x1,%r8d
  31382a:	48 8b 74 24 28       	mov    0x28(%rsp),%rsi
  31382f:	48 89 ea             	mov    %rbp,%rdx
  313832:	e8 99 ce e6 ff       	call   1806d0 <<alloc::raw_vec::RawVecInner<_>>::reserve::do_reserve_and_handle::<alloc::alloc::Global>>
<alloc::raw_vec::RawVecInner>::non_null::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:611
  313837:	48 8b 44 24 50       	mov    0x50(%rsp),%rax
<alloc::vec::Vec<u8>>::len:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:3101
  31383c:	48 8b 5c 24 58       	mov    0x58(%rsp),%rbx
  313841:	48 89 44 24 40       	mov    %rax,0x40(%rsp)
<*mut u8>::add:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mut_ptr.rs:970
  313846:	48 8d 3c 18          	lea    (%rax,%rbx,1),%rdi
core::ptr::copy_nonoverlapping::<u8>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:573
  31384a:	4c 89 f6             	mov    %r14,%rsi
  31384d:	48 89 ea             	mov    %rbp,%rdx
  313850:	ff 15 42 49 1c 00    	call   *0x1c4942(%rip)        # 4d8198 <memcpy@GLIBC_2.14>
  313856:	e9 39 fa ff ff       	jmp    313294 <<rexx_exec::Interp>::exec_parse+0x3964>
<usize as core::slice::index::SliceIndex<[alloc::boxed::Box<str>]>>::index:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/slice/index.rs:238
  31385b:	48 8d 15 7e fb 1b 00 	lea    0x1bfb7e(%rip),%rdx        # 4d33e0 <rexx_api::ffi::CALL_CONTEXT+0x17e80>
  313862:	e8 0a 90 e0 ff       	call   11c871 <core::panicking::panic_bounds_check>
  313867:	eb 74                	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
alloc::boxed::box_new_uninit:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:253
  313869:	bf 08 00 00 00       	mov    $0x8,%edi
  31386e:	be 70 00 00 00       	mov    $0x70,%esi
  313873:	e8 0f e3 df ff       	call   111b87 <alloc::alloc::handle_alloc_error>
  313878:	eb 63                	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  31387a:	bf 01 00 00 00       	mov    $0x1,%edi
  31387f:	4c 89 e6             	mov    %r12,%rsi
  313882:	e8 ed e2 df ff       	call   111b74 <alloc::raw_vec::handle_error>
  313887:	eb 54                	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<rexx_exec::parse_template::SourceText>::bytes:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:81
  313889:	48 8d 3d 0c 4c d5 ff 	lea    -0x2ab3f4(%rip),%rdi        # 6849c <panic_unwind::imp::CANARY+0x36cf>
  313890:	48 8d 15 41 92 1a 00 	lea    0x1a9241(%rip),%rdx        # 4bcad8 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  313897:	be ab 00 00 00       	mov    $0xab,%esi
  31389c:	e8 0f 71 e0 ff       	call   11a9b0 <core::panicking::panic_fmt>
  3138a1:	eb 3a                	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
  3138a3:	48 8d 3d f2 4b d5 ff 	lea    -0x2ab40e(%rip),%rdi        # 6849c <panic_unwind::imp::CANARY+0x36cf>
  3138aa:	48 8d 15 27 92 1a 00 	lea    0x1a9227(%rip),%rdx        # 4bcad8 <rexx_api::ffi::CALL_CONTEXT+0x1578>
  3138b1:	be ab 00 00 00       	mov    $0xab,%esi
  3138b6:	e8 f5 70 e0 ff       	call   11a9b0 <core::panicking::panic_fmt>
  3138bb:	eb 20                	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
<alloc::raw_vec::RawVecInner>::with_capacity_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:443
  3138bd:	bf 01 00 00 00       	mov    $0x1,%edi
  3138c2:	be 34 00 00 00       	mov    $0x34,%esi
  3138c7:	e8 a8 e2 df ff       	call   111b74 <alloc::raw_vec::handle_error>
  3138cc:	eb 0f                	jmp    3138dd <<rexx_exec::Interp>::exec_parse+0x3fad>
  3138ce:	bf 01 00 00 00       	mov    $0x1,%edi
  3138d3:	be 02 00 00 00       	mov    $0x2,%esi
  3138d8:	e8 97 e2 df ff       	call   111b74 <alloc::raw_vec::handle_error>
<rexx_exec::Interp>::apply_trigger:
  3138dd:	0f 0b                	ud2
  3138df:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3138e2:	48 85 ed             	test   %rbp,%rbp
  3138e5:	0f 84 bf 01 00 00    	je     313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3138eb:	4c 89 ef             	mov    %r13,%rdi
  3138ee:	ff 15 74 48 1c 00    	call   *0x1c4874(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3138f4:	e9 b1 01 00 00       	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::exec_parse:
  3138f9:	eb 67                	jmp    313962 <<rexx_exec::Interp>::exec_parse+0x4032>
  3138fb:	eb 65                	jmp    313962 <<rexx_exec::Interp>::exec_parse+0x4032>
  3138fd:	49 89 c4             	mov    %rax,%r12
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313900:	4c 89 f7             	mov    %r14,%rdi
  313903:	ff 15 5f 48 1c 00    	call   *0x1c485f(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  313909:	4c 89 bc 24 28 01 00 	mov    %r15,0x128(%rsp)
  313910:	00 
<alloc::vec::Vec<alloc::vec::Vec<u8>>>::push_mut:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:1027
  313911:	e9 94 01 00 00       	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::exec_parse:
  313916:	49 89 c6             	mov    %rax,%r14
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  313919:	4d 85 e4             	test   %r12,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31391c:	74 09                	je     313927 <<rexx_exec::Interp>::exec_parse+0x3ff7>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31391e:	48 89 df             	mov    %rbx,%rdi
  313921:	ff 15 41 48 1c 00    	call   *0x1c4841(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  313927:	4c 89 ff             	mov    %r15,%rdi
  31392a:	ff 15 38 48 1c 00    	call   *0x1c4838(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  313930:	4d 89 f4             	mov    %r14,%r12
  313933:	4d 89 ee             	mov    %r13,%r14
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  313936:	e9 22 01 00 00       	jmp    313a5d <<rexx_exec::Interp>::exec_parse+0x412d>
<rexx_exec::Interp>::exec_parse:
  31393b:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  31393e:	48 83 7c 24 48 00    	cmpq   $0x0,0x48(%rsp)
  313944:	0f 84 48 01 00 00    	je     313a92 <<rexx_exec::Interp>::exec_parse+0x4162>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  31394a:	48 8b 7c 24 50       	mov    0x50(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31394f:	ff 15 13 48 1c 00    	call   *0x1c4813(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313955:	e9 38 01 00 00       	jmp    313a92 <<rexx_exec::Interp>::exec_parse+0x4162>
<rexx_exec::Interp>::exec_parse:
  31395a:	49 89 c4             	mov    %rax,%r12
  31395d:	e9 e2 00 00 00       	jmp    313a44 <<rexx_exec::Interp>::exec_parse+0x4114>
  313962:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::try_allocate_in:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:461
  313965:	48 85 ed             	test   %rbp,%rbp
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313968:	0f 84 80 01 00 00    	je     313aee <<rexx_exec::Interp>::exec_parse+0x41be>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31396e:	4c 89 ff             	mov    %r15,%rdi
  313971:	e9 72 01 00 00       	jmp    313ae8 <<rexx_exec::Interp>::exec_parse+0x41b8>
<rexx_exec::Interp>::parse_strings:
  313976:	49 89 c4             	mov    %rax,%r12
  313979:	48 89 ef             	mov    %rbp,%rdi
  31397c:	e9 67 01 00 00       	jmp    313ae8 <<rexx_exec::Interp>::exec_parse+0x41b8>
<rexx_exec::Interp>::exec_parse:
  313981:	49 89 c4             	mov    %rax,%r12
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313984:	48 89 df             	mov    %rbx,%rdi
  313987:	e9 5c 01 00 00       	jmp    313ae8 <<rexx_exec::Interp>::exec_parse+0x41b8>
<rexx_exec::Interp>::exec_parse:
  31398c:	49 89 c4             	mov    %rax,%r12
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  31398f:	48 89 df             	mov    %rbx,%rdi
  313992:	e9 26 01 00 00       	jmp    313abd <<rexx_exec::Interp>::exec_parse+0x418d>
<rexx_exec::Interp>::exec_parse:
  313997:	e9 0b 01 00 00       	jmp    313aa7 <<rexx_exec::Interp>::exec_parse+0x4177>
  31399c:	eb 78                	jmp    313a16 <<rexx_exec::Interp>::exec_parse+0x40e6>
  31399e:	eb 76                	jmp    313a16 <<rexx_exec::Interp>::exec_parse+0x40e6>
  3139a0:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  3139a3:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  3139a9:	0f 85 fe c6 ff ff    	jne    3100ad <<rexx_exec::Interp>::exec_parse+0x77d>
  3139af:	e9 3a 01 00 00       	jmp    313aee <<rexx_exec::Interp>::exec_parse+0x41be>
<rexx_exec::Interp>::exec_parse:
  3139b4:	e9 ee 00 00 00       	jmp    313aa7 <<rexx_exec::Interp>::exec_parse+0x4177>
  3139b9:	49 89 c4             	mov    %rax,%r12
  3139bc:	e9 83 00 00 00       	jmp    313a44 <<rexx_exec::Interp>::exec_parse+0x4114>
  3139c1:	49 89 c4             	mov    %rax,%r12
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  3139c4:	e9 fa 00 00 00       	jmp    313ac3 <<rexx_exec::Interp>::exec_parse+0x4193>
  3139c9:	49 89 c4             	mov    %rax,%r12
  3139cc:	48 8b bc 24 f0 00 00 	mov    0xf0(%rsp),%rdi
  3139d3:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  3139d4:	ff 15 8e 47 1c 00    	call   *0x1c478e(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  3139da:	e9 cb 00 00 00       	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::exec_parse:
  3139df:	e9 c3 00 00 00       	jmp    313aa7 <<rexx_exec::Interp>::exec_parse+0x4177>
  3139e4:	49 89 c4             	mov    %rax,%r12
  3139e7:	48 8d 7c 24 48       	lea    0x48(%rsp),%rdi
<alloc::boxed::Box<rexx_exec::error::Raised>>::new:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:296
  3139ec:	e8 0f 95 e5 ff       	call   16cf00 <core::ptr::drop_glue::<rexx_exec::error::Raised>>
  3139f1:	4d 89 ee             	mov    %r13,%r14
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/boxed.rs:288
  3139f4:	eb 67                	jmp    313a5d <<rexx_exec::Interp>::exec_parse+0x412d>
<rexx_exec::Interp>::apply_trigger:
  3139f6:	eb 41                	jmp    313a39 <<rexx_exec::Interp>::exec_parse+0x4109>
  3139f8:	eb 73                	jmp    313a6d <<rexx_exec::Interp>::exec_parse+0x413d>
  3139fa:	e9 a8 00 00 00       	jmp    313aa7 <<rexx_exec::Interp>::exec_parse+0x4177>
  3139ff:	49 89 c4             	mov    %rax,%r12
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313a02:	48 89 df             	mov    %rbx,%rdi
  313a05:	ff 15 5d 47 1c 00    	call   *0x1c475d(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313a0b:	e9 9a 00 00 00       	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::exec_parse:
  313a10:	eb 04                	jmp    313a16 <<rexx_exec::Interp>::exec_parse+0x40e6>
  313a12:	eb 02                	jmp    313a16 <<rexx_exec::Interp>::exec_parse+0x40e6>
  313a14:	eb 00                	jmp    313a16 <<rexx_exec::Interp>::exec_parse+0x40e6>
  313a16:	49 89 c4             	mov    %rax,%r12
core::ptr::drop_glue::<rexx_exec::value::Carried>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313a19:	80 7c 24 48 01       	cmpb   $0x1,0x48(%rsp)
  313a1e:	0f 85 86 00 00 00    	jne    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
  313a24:	48 83 7c 24 50 00    	cmpq   $0x0,0x50(%rsp)
  313a2a:	74 7e                	je     313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313a2c:	48 8b 7c 24 58       	mov    0x58(%rsp),%rdi
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313a31:	ff 15 31 47 1c 00    	call   *0x1c4731(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313a37:	eb 71                	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::assign_targets:
  313a39:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313a3c:	48 83 7c 24 48 00    	cmpq   $0x0,0x48(%rsp)
  313a42:	74 66                	je     313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::exec_parse:
  313a44:	4c 89 ff             	mov    %r15,%rdi
  313a47:	ff 15 1b 47 1c 00    	call   *0x1c471b(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313a4d:	eb 5b                	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::exec_parse:
  313a4f:	eb 56                	jmp    313aa7 <<rexx_exec::Interp>::exec_parse+0x4177>
  313a51:	49 89 c4             	mov    %rax,%r12
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313a54:	4c 89 ff             	mov    %r15,%rdi
  313a57:	ff 15 0b 47 1c 00    	call   *0x1c470b(%rip)        # 4d8168 <free@GLIBC_2.2.5>
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313a5d:	48 85 ed             	test   %rbp,%rbp
  313a60:	7e 48                	jle    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313a62:	4c 89 f7             	mov    %r14,%rdi
  313a65:	ff 15 fd 46 1c 00    	call   *0x1c46fd(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313a6b:	eb 3d                	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::assign_targets:
  313a6d:	49 89 c4             	mov    %rax,%r12
  313a70:	48 8b bc 24 f0 00 00 	mov    0xf0(%rsp),%rdi
  313a77:	00 
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:792
  313a78:	49 ff ce             	dec    %r14
  313a7b:	49 83 fe fe          	cmp    $0xfffffffffffffffe,%r14
  313a7f:	73 29                	jae    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313a81:	ff 15 e1 46 1c 00    	call   *0x1c46e1(%rip)        # 4d8168 <free@GLIBC_2.2.5>
<alloc::alloc::Global>::deallocate_impl_runtime:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/alloc.rs:312
  313a87:	eb 21                	jmp    313aaa <<rexx_exec::Interp>::exec_parse+0x417a>
<rexx_exec::Interp>::exec_parse:
  313a89:	eb 1c                	jmp    313aa7 <<rexx_exec::Interp>::exec_parse+0x4177>
  313a8b:	eb 4b                	jmp    313ad8 <<rexx_exec::Interp>::exec_parse+0x41a8>
  313a8d:	eb 49                	jmp    313ad8 <<rexx_exec::Interp>::exec_parse+0x41a8>
  313a8f:	49 89 c4             	mov    %rax,%r12
<alloc::raw_vec::RawVecInner>::current_memory:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/raw_vec/mod.rs:631
  313a92:	48 83 bc 24 70 01 00 	cmpq   $0x0,0x170(%rsp)
  313a99:	00 00 
  313a9b:	74 51                	je     313aee <<rexx_exec::Interp>::exec_parse+0x41be>
core::ptr::drop_glue::<alloc::vec::Vec<u8>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313a9d:	48 8b bc 24 78 01 00 	mov    0x178(%rsp),%rdi
  313aa4:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313aa5:	eb 41                	jmp    313ae8 <<rexx_exec::Interp>::exec_parse+0x41b8>
<rexx_exec::Interp>::exec_parse:
  313aa7:	49 89 c4             	mov    %rax,%r12
core::ptr::drop_glue::<rexx_exec::parse_template::SourceText>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313aaa:	48 83 bc 24 28 01 00 	cmpq   $0x0,0x128(%rsp)
  313ab1:	00 00 
  313ab3:	7e 0e                	jle    313ac3 <<rexx_exec::Interp>::exec_parse+0x4193>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  313ab5:	48 8b bc 24 d8 00 00 	mov    0xd8(%rsp),%rdi
  313abc:	00 
  313abd:	ff 15 a5 46 1c 00    	call   *0x1c46a5(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  313ac3:	48 83 bc 24 e0 01 00 	cmpq   $0x0,0x1e0(%rsp)
  313aca:	00 00 
core::ptr::drop_glue::<rexx_exec::parse_template::ParseStrings>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313acc:	7e 20                	jle    313aee <<rexx_exec::Interp>::exec_parse+0x41be>
<rexx_exec::Interp>::exec_parse:
/home/moritz/dev/repos/ooRexx-rust-rewrite/rust/crates/rexx-exec/src/parse_template.rs:505
  313ace:	48 8b bc 24 e8 01 00 	mov    0x1e8(%rsp),%rdi
  313ad5:	00 
std::sys::alloc::unix::dealloc:
/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/std/src/sys/alloc/unix.rs:65
  313ad6:	eb 10                	jmp    313ae8 <<rexx_exec::Interp>::exec_parse+0x41b8>
<rexx_exec::Interp>::exec_parse:
  313ad8:	49 89 c4             	mov    %rax,%r12
core::ptr::drop_glue::<alloc::borrow::Cow<[u8]>>:
/home/moritz/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ptr/mod.rs:847
  313adb:	48 83 7c 24 28 00    	cmpq   $0x0,0x28(%rsp)
  313ae1:	7e 0b                	jle    313aee <<rexx_exec::Interp>::exec_parse+0x41be>
<rexx_exec::Interp>::exec_parse:
  313ae3:	48 8b 7c 24 30       	mov    0x30(%rsp),%rdi
  313ae8:	ff 15 7a 46 1c 00    	call   *0x1c467a(%rip)        # 4d8168 <free@GLIBC_2.2.5>
  313aee:	4c 89 e7             	mov    %r12,%rdi
  313af1:	e8 1a dd 19 00       	call   4b1810 <_Unwind_Resume@plt>

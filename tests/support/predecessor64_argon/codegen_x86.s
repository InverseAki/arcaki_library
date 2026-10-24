	.file	"predecessor64_codegen.8b35faa532ff1581-cgu.0"
	.section	.text.convert,"ax",@progbits
	.globl	convert
	.p2align	4
	.type	convert,@function
convert:
	.cfi_startproc
	movq	%rdi, %rax
	movups	(%rsi), %xmm0
	movq	16(%rsi), %rcx
	movups	%xmm0, (%rdi)
	movq	%rcx, 16(%rdi)
	retq
.Lfunc_end0:
	.size	convert, .Lfunc_end0-convert
	.cfi_endproc

	.section	.text.high_sub,"ax",@progbits
	.globl	high_sub
	.p2align	4
	.type	high_sub,@function
high_sub:
	.cfi_startproc
	movl	$127, %eax
	bsrq	%rdi, %rax
	xorl	$-64, %eax
	addl	$64, %eax
	retq
.Lfunc_end1:
	.size	high_sub, .Lfunc_end1-high_sub
	.cfi_endproc

	.section	.text.high_sub_nonzero,"ax",@progbits
	.globl	high_sub_nonzero
	.p2align	4
	.type	high_sub_nonzero,@function
high_sub_nonzero:
	.cfi_startproc
	bsrq	%rdi, %rcx
	xorl	%eax, %eax
	cmpq	$1, %rdi
	sbbl	%eax, %eax
	orl	%ecx, %eax
	retq
.Lfunc_end2:
	.size	high_sub_nonzero, .Lfunc_end2-high_sub_nonzero
	.cfi_endproc

	.section	.text.high_xor,"ax",@progbits
	.globl	high_xor
	.p2align	4
	.type	high_xor,@function
high_xor:
	.cfi_startproc
	movl	$127, %eax
	bsrq	%rdi, %rax
	retq
.Lfunc_end3:
	.size	high_xor, .Lfunc_end3-high_xor
	.cfi_endproc

	.globl	high_xor_nonzero
	.type	high_xor_nonzero,@function
high_xor_nonzero = high_sub_nonzero
	.ident	"rustc version 1.93.0 (254b59607 2026-01-19)"
	.section	".note.GNU-stack","",@progbits

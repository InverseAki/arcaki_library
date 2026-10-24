	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	_high_sub
	.p2align	2
_high_sub:
	.cfi_startproc
	clz	x8, x0
	mov	w9, #63
	sub	w0, w9, w8
	ret
	.cfi_endproc

	.globl	_high_xor
	.p2align	2
_high_xor:
	.cfi_startproc
	clz	x8, x0
	eor	w0, w8, #0x3f
	ret
	.cfi_endproc

	.globl	_high_sub_nonzero
	.p2align	2
_high_sub_nonzero:
	.cfi_startproc
	clz	x8, x0
	eor	w8, w8, #0x3f
	cmp	x0, #0
	csinv	w0, w8, wzr, ne
	ret
	.cfi_endproc

	.globl	_convert
	.p2align	2
_convert:
	.cfi_startproc
	ldr	q0, [x0]
	ldr	x9, [x0, #16]
	str	q0, [x8]
	str	x9, [x8, #16]
	ret
	.cfi_endproc

	.globl	_high_xor_nonzero
.set _high_xor_nonzero, _high_sub_nonzero
.subsections_via_symbols

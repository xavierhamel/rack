pub fn header() -> &'static str {
    "
bits 64
default rel
segment .bss
    _io@print_str_chars_written: resb 4
    _io@print_char_buffer:  resb 1
    _mem@mem: resb 256
    _mem@internal: resb 256
	_mem@ret_ptr: resb 6144
    _mem@variables: resb 24576

segment .text
    global _start
    extern GetStdHandle
    extern WriteConsoleA
    extern ExitProcess
    extern HeapAlloc
    extern HeapCreate
    extern HeapReAlloc
    extern HeapDestroy
    extern GetProcessHeap
    extern HeapFree
    extern printf"
}

pub fn ret_ptr() -> &'static str {
    "
_std@ret_ptr_addr:
	xor rax, rax
	mov rdx, 8
	mov ax, word [_mem@ret_ptr_idx]
	mul rdx
	lea rbx, [_mem@ret_ptr]
	add rax, rbx
	ret

_std@store_ret_ptr:
	pop r15
	pop r14
	call _std@ret_ptr_addr
	mov qword [rax], r14
	inc word [_mem@ret_ptr_idx]
	push r15
	ret

_std@load_ret_ptr:
    pop r15
	dec word [_mem@ret_ptr_idx]
	call _std@ret_ptr_addr
	mov r14, [rax]
	push r14
	push r15
	ret"
}

pub fn print_int() -> &'static str {
    " "
}
pub fn variables() -> &'static str {
    "
_std@put_variable:
    pop r15
    pop rax ;variable idx
    pop rbx ;variable
	mov rdx, 8
	mul rdx
	lea rcx, [_mem@variables]
	add rax, rcx
    mov qword [rax], rbx
    push r15
	ret

_std@fetch_variable:
    pop r15
    pop rax ;variable idx
	mov rdx, 8
	mul rdx
	lea rcx, [_mem@variables]
	add rax, rcx
    mov rbx, qword [rax]
    push rbx
    push r15
	ret"
}

pub fn exit() -> &'static str {
    "
_std@exit:
    lea rax, [_mem@internal]
    add rax, 32
    mov rcx, qword [rax]
    call HeapDestroy
    xor rcx, rcx
    call ExitProcess
    ret"
}
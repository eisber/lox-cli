EXTERN observe_fp:PROC
PUBLIC verify_sentinels

.data
ALIGN 16
simd_seed DQ 1122334455667788h, 8877665544332211h
fp_x DQ 03ff8000000000000h
fp_y DQ 04000000000000000h
x87_seed DQ 0400c000000000000h

.code
verify_sentinels PROC FRAME
    push rbx
    .pushreg rbx
    push rsi
    .pushreg rsi
    push rdi
    .pushreg rdi
    push r12
    .pushreg r12
    push r13
    .pushreg r13
    push r14
    .pushreg r14
    push r15
    .pushreg r15
    sub rsp, 300h
    .allocstack 300h
    .endprolog
    fxsave [rsp+40h]
    stmxcsr [rsp+268h]
    fld QWORD PTR [x87_seed]
    mov rbx, 1020304050607080h
    mov rsi, 1122334455667788h
    mov rdi, 2233445566778899h
    mov r12, 33445566778899aah
    mov r13, 445566778899aabbh
    mov r14, 5566778899aabbcch
    mov r15, 66778899aabbccddh
    movdqa xmm6, XMMWORD PTR [simd_seed]
    movdqa xmm7, xmm6
    movdqa xmm8, xmm6
    movdqa xmm9, xmm6
    movdqa xmm10, xmm6
    movdqa xmm11, xmm6
    movdqa xmm12, xmm6
    movdqa xmm13, xmm6
    movdqa xmm14, xmm6
    movdqa xmm15, xmm6
    movsd xmm0, QWORD PTR [fp_x]
    movsd xmm1, QWORD PTR [fp_y]
    call observe_fp
    movq rax, xmm0
    mov rcx, 4013000000000000h
    cmp rax, rcx
    jne failed
    stmxcsr [rsp+26ch]
    mov eax, DWORD PTR [rsp+268h]
    cmp eax, DWORD PTR [rsp+26ch]
    jne failed
    fstp QWORD PTR [rsp+260h]
    mov rax, QWORD PTR [rsp+260h]
    cmp rax, QWORD PTR [x87_seed]
    jne failed
    mov rax, 1020304050607080h
    cmp rbx, rax
    jne failed
    mov rax, 1122334455667788h
    cmp rsi, rax
    jne failed
    mov rax, 2233445566778899h
    cmp rdi, rax
    jne failed
    mov rax, 33445566778899aah
    cmp r12, rax
    jne failed
    mov rax, 445566778899aabbh
    cmp r13, rax
    jne failed
    mov rax, 5566778899aabbcch
    cmp r14, rax
    jne failed
    mov rax, 66778899aabbccddh
    cmp r15, rax
    jne failed
    movdqa xmm4, XMMWORD PTR [simd_seed]
    movdqa xmm5, xmm6
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm7
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm8
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm9
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm10
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm11
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm12
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm13
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm14
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    movdqa xmm5, xmm15
    pcmpeqb xmm5, xmm4
    pmovmskb ecx, xmm5
    cmp ecx, 0ffffh
    jne failed
    mov eax, 1
    jmp restored
failed:
    xor eax, eax
restored:
    fxrstor [rsp+40h]
    add rsp, 300h
    pop r15
    pop r14
    pop r13
    pop r12
    pop rdi
    pop rsi
    pop rbx
    ret
verify_sentinels ENDP
END

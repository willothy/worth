    ;; -- generated by the worth compiler --
segment .bss
mem:
    resb    640000
segment .text
global _start
_start:
    push    0
    push    97
    push    0
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    98
    push    1
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    99
    push    2
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    10
    push    4
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    13
    push    5
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    3
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    push    1
    push    1
    ;; -- syscall3 --
    pop     rax                         ;; Syscall number
    pop     rdi
    pop     rsi
    pop     rdx
    syscall
    push    0
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- load --
    pop     rax                         ;; Address to load from
    xor     rbx, rbx                    ;; Zero out rbx
    mov     bl, [rax]                   ;; Load low byte into rbx
    push    rbx
    push    1
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    push    0
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    1
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- load --
    pop     rax                         ;; Address to load from
    xor     rbx, rbx                    ;; Zero out rbx
    mov     bl, [rax]                   ;; Load low byte into rbx
    push    rbx
    push    1
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    push    1
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    2
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- load --
    pop     rax                         ;; Address to load from
    xor     rbx, rbx                    ;; Zero out rbx
    mov     bl, [rax]                   ;; Load low byte into rbx
    push    rbx
    push    1
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    push    2
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- add --
    pop     rax
    pop     rbx
    add     rax, rbx
    push    rax
    ;; -- intrinsic: swap --
    pop     rax
    pop     rbx
    push    rax
    push    rbx
    ;; -- end intrinsic --
    ;; -- store --
    pop     rbx                         ;; Value to store
    pop     rax                         ;; Address to store into
    mov     [rax], bl                   ;; Store low byte into address
    push    5
    ;; -- intrinsic: mem --
    push    mem
    ;; -- end intrinsic --
    push    1
    push    1
    ;; -- syscall3 --
    pop     rax                         ;; Syscall number
    pop     rdi
    pop     rsi
    pop     rdx
    syscall
    ;; -- syscall (1) 60: 60 --
    mov     rax, 60
    mov     rdi, 0
    syscall
intrinsic_dump:
    push    rbp
    mov     rbp, rsp
    sub     rsp, 64
    mov     qword [rbp - 8], rdi
    mov     qword [rbp - 56], 1
    mov     eax, 32
    sub     rax, qword [rbp - 56]
    mov     byte [rbp + rax - 48], 10
.intrinsic_dump_body:
    mov     rax, qword [rbp - 8]
    mov     ecx, 10
    xor     edx, edx
    div     rcx
    add     rdx, 48
    mov     cl, dl
    mov     eax, 32
    sub     rax, qword [rbp - 56]
    sub     rax, 1
    mov     byte [rbp + rax - 48], cl
    mov     rax, qword [rbp - 56]
    add     rax, 1
    mov     qword [rbp - 56], rax
    mov     rax, qword [rbp - 8]
    mov     ecx, 10
    xor     edx, edx
    div     rcx
    mov     qword [rbp - 8], rax
    cmp     qword [rbp - 8], 0
    jne     .intrinsic_dump_body
    mov     eax, 32
    sub     rax, qword [rbp - 56]
    lea     rsi, [rbp - 48]
    add     rsi, rax
    mov     rdx, qword [rbp - 56]
    mov     edi, 1
    mov     rax, 1
    syscall
    add     rsp, 64
    pop     rbp
    ret 
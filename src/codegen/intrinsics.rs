use crate::{asm, asm_line, comment, intrinsic_str, intrinsics, label, sys_exit, syscall};
use casey::lower;
use std::fmt::Display;

use super::builder::Builder;

intrinsics!(
    Print,
    Panic,
    Dup,
    Dup2 = "2dup",
    Swap,
    Mem,
    Drop,
    Drop2 = "2drop",
    Over,
    Argc,
    Argv,
    CastPtr = "cast(ptr)"
);

impl Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let intrinsic: &str = self.into();
        write!(f, "{}", intrinsic)
    }
}

pub fn castptr(asm: &mut Builder) {
    comment!(asm, "-- Pointer cast --");
}

pub fn argv(asm: &mut Builder) {
    asm!(
        asm,
        ("mov", "rax, [args_ptr]"),
        ("add", "rax, 8"),
        ("push", "rax")
    );
}

pub fn argc(asm: &mut Builder) {
    asm!(
        asm,
        ("mov", "rax, [args_ptr]"),
        ("mov rax, [rax]"),
        ("push", "rax")
    );
}

pub fn panic(asm: &mut Builder) {
    sys_exit!(asm, 1, "-- panic --");
}

pub fn print(asm: &mut Builder) {
    asm!(
        asm,
        ///Load argument to rdi
        ("pop", "rdi"),
        ("call", "intrinsic_print")
    );
}

pub fn dup(asm: &mut Builder) {
    asm!(asm, ("pop", "rax"), ("push", "rax"), ("push", "rax"));
}

pub fn dup2(asm: &mut Builder) {
    asm!(
        asm,
        ///( x1 x2 -> x1 x2 x1 x2 )
        ("pop", "rax"),
        ("pop", "rbx"),
        ("push", "rbx"),
        ("push", "rax"),
        ("push", "rbx"),
        ("push", "rax")
    );
}

pub fn drop(asm: &mut Builder) {
    asm!(asm, ("pop", "rax"));
}

pub fn drop2(asm: &mut Builder) {
    asm!(asm, ("pop", "rax"), ("pop", "rbx"));
}

pub fn over(asm: &mut Builder) {
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("push", "rbx"),
        ("push", "rax"),
        ("push", "rbx")
    );
}

pub fn swap(asm: &mut Builder) {
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("push", "rax"),
        ("push", "rbx")
    );
}

pub fn mem(asm: &mut Builder) {
    asm!(asm, ("push", "mem"));
}

pub fn gen_intrinsics(asm: &mut Builder) {
    // Print
    label!(asm, "intrinsic_print");
    asm!(
        asm,
        ("push", "rbp"),
        ("mov", "rbp, rsp"),
        ("sub", "rsp, 64"),
        ("mov", "qword [rbp - 8], rdi"),
        ("mov", "qword [rbp - 56], 1"),
        ("mov", "eax, 32"),
        ("sub", "rax, qword [rbp - 56]"),
        ("mov", "byte [rbp + rax - 48], 10")
    );
    label!(asm, ".intrinsic_print_body");
    asm!(
        asm,
        ("mov", "rax, qword [rbp - 8]"),
        ("mov", "ecx, 10"),
        ("xor", "edx, edx"),
        ("div", "rcx"),
        ("add", "rdx, 48"),
        ("mov", "cl, dl"),
        ("mov", "eax, 32"),
        ("sub", "rax, qword [rbp - 56]"),
        ("sub", "rax, 1"),
        ("mov", "byte [rbp + rax - 48], cl"),
        ("mov", "rax, qword [rbp - 56]"),
        ("add", "rax, 1"),
        ("mov", "qword [rbp - 56], rax"),
        ("mov", "rax, qword [rbp - 8]"),
        ("mov", "ecx, 10"),
        ("xor", "edx, edx"),
        ("div", "rcx"),
        ("mov", "qword [rbp - 8], rax"),
        ("cmp", "qword [rbp - 8], 0"),
        ("jne", ".intrinsic_print_body"),
        ("mov", "eax, 32"),
        ("sub", "rax, qword [rbp - 56]"),
        ("lea", "rsi, [rbp - 48]"),
        ("add", "rsi, rax"),
        ("mov", "rdx, qword [rbp - 56]"),
        ("mov", "edi, 1"),
        ("mov", "rax, 1"),
        ("syscall"),
        ("add", "rsp, 64"),
        ("pop", "rbp"),
        ("ret")
    );
}

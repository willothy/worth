use crate::{asm, asm_line, comment, intrinsics, label, sys_exit, syscall};
use std::fmt::Display;
use strum_macros::{EnumString, IntoStaticStr};

intrinsics!(Dump, Panic, Dup, Swap, Mem, Drop);

impl Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let intrinsic: &'static str = self.into();
        write!(f, "{}", intrinsic)
    }
}

pub fn panic(asm: &mut Vec<String>) {
    sys_exit!(asm, 1, "-- panic --");
}

pub fn dump(asm: &mut Vec<String>) {
    asm!(
        asm,
        ///Load argument to rdi
        ("pop", "rdi"),
        ("call", "intrinsic_dump")
    );
}

pub fn dup(asm: &mut Vec<String>) {
    asm!(asm, ("pop", "rax"), ("push", "rax"), ("push", "rax"));
}

pub fn drop(asm: &mut Vec<String>) {
    asm!(asm, ("pop", "rax"));
}

pub fn swap(asm: &mut Vec<String>) {
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("push", "rax"),
        ("push", "rbx")
    );
}

pub fn mem(asm: &mut Vec<String>) {
    asm!(asm, ("push", "mem"));
}

pub fn gen_intrinsics(asm: &mut Vec<String>) {
    // Dump
    label!(asm, "intrinsic_dump");
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
    label!(asm, ".intrinsic_dump_body");
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
        ("jne", ".intrinsic_dump_body"),
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

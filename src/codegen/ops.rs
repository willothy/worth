use crate::{asm, asm_line, comment};

use super::builder::Builder;

pub fn add(asm: &mut Builder) {
    comment!(asm, "-- add --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("add", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn sub(asm: &mut Builder) {
    comment!(asm, "-- sub --");
    asm!(
        asm,
        ("pop", "rbx"),
        ("pop", "rax"),
        ("sub", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn mul(asm: &mut Builder) {
    comment!(asm, "-- mul --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("mul", "rbx"),
        ("push", "rax")
    );
}

pub fn div(asm: &mut Builder) {
    comment!(asm, "-- div --");
    // TODO(#1): Test this
    asm!(
        asm,
        ("xor", "rdx, rdx"),
        ("pop", "rbx"),
        ("pop", "rax"),
        ("div", "rbx"),
        ("push", "rax")
    );
}

pub fn mod_(asm: &mut Builder) {
    comment!(asm, "-- divmod --");
    asm!(
        asm,
        ("xor", "rdx, rdx"),
        ("pop", "rbx"),
        ("pop", "rax"),
        ("div", "rbx"),
        ("push", "rdx")
    );
}

pub fn divmod(asm: &mut Builder) {
    comment!(asm, "-- divmod --");
    // TODO: Test this
    asm!(
        asm,
        ("xor", "rdx, rdx"),
        ("pop", "rbx"),
        ("pop", "rax"),
        ("div", "rbx"),
        ("push", "rax"),
        ("push", "rdx")
    );
}

pub fn not(asm: &mut Builder) {
    comment!(asm, "-- not --");
    asm!(asm, ("pop", "rax"), ("neg", "rax"), ("push", "rax"));
}

pub fn band(asm: &mut Builder) {
    comment!(asm, "-- and --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("and", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn bor(asm: &mut Builder) {
    comment!(asm, "-- or --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("or", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn xor(asm: &mut Builder) {
    comment!(asm, "-- xor --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("xor", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn shl(asm: &mut Builder) {
    comment!(asm, "-- shl --");
    asm!(
        asm,
        ("pop", "rcx"),
        ("pop", "rax"),
        ("shl", "rax, cl"),
        ("push", "rax")
    );
}

pub fn shr(asm: &mut Builder) {
    comment!(asm, "-- shr --");
    asm!(
        asm,
        ("pop", "rcx"),
        ("pop", "rax"),
        ("shr", "rax, cl"),
        ("push", "rax")
    );
}

pub fn eq(asm: &mut Builder) {
    comment!(asm, "-- eq --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rax"),
        ("pop", "rbx"),
        ("cmp", "rax, rbx"),
        ("cmove", "rcx, rdx"),
        ("push", "rcx")
    );
}

pub fn neq(asm: &mut Builder) {
    comment!(asm, "-- ne --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rax"),
        ("pop", "rbx"),
        ("cmp", "rax, rbx"),
        ("cmovne", "rcx, rdx"),
        ("push", "rcx")
    );
}

pub fn lt(asm: &mut Builder) {
    comment!(asm, "-- lt --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rbx"),
        ("pop", "rax"),
        ("cmp", "rax, rbx"),
        ("cmovl", "rcx, rdx"),
        ("push", "rcx")
    );
}

pub fn gt(asm: &mut Builder) {
    comment!(asm, "-- gt --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rbx"),
        ("pop", "rax"),
        ("cmp", "rax, rbx"),
        ("cmovg", "rcx, rdx"),
        ("push", "rcx")
    );
}

pub fn lte(asm: &mut Builder) {
    comment!(asm, "-- le --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rbx"),
        ("pop", "rax"),
        ("cmp", "rax, rbx"),
        ("cmovle", "rcx, rdx"),
        ("push", "rcx")
    );
}

pub fn gte(asm: &mut Builder) {
    comment!(asm, "-- ge --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rbx"),
        ("pop", "rax"),
        ("cmp", "rax, rbx"),
        ("cmovge", "rcx, rdx"),
        ("push", "rcx")
    );
}

pub fn load(asm: &mut Builder) {
    comment!(asm, "-- load --");
    asm!(
        asm,
        /// Address to load from
        ("pop", "rax"),
        /// Zero out rbx
        ("xor", "rbx, rbx"),
        /// Load low byte into rbx
        ("mov", "bl, [rax]"),
        ("push", "rbx")
    );
}

pub(crate) fn load64(asm: &mut Builder) {
    comment!(asm, "-- load64 --");
    asm!(
        asm,
        /// Address to load from
        ("pop", "rax"),
        /// Zero out rbx
        ("xor", "rbx, rbx"),
        /// Load low byte into rbx
        ("mov", "rbx, [rax]"),
        /// Push rbx
        ("push", "rbx")
    );
}

pub fn store(asm: &mut Builder) {
    comment!(asm, "-- store --");
    asm!(
        asm,
        /// Value to store
        ("pop", "rbx"),
        /// Address to store into
        ("pop", "rax"),
        /// Store low byte into address
        ("mov", "[rax], bl")
    );
}

pub fn store64(asm: &mut Builder) {
    comment!(asm, "-- store64 --");
    asm!(
        asm,
        /// Value to store
        ("pop", "rbx"),
        /// Address to store into
        ("pop", "rax"),
        /// Store low byte into address
        ("mov", "[rax], rbx")
    );
}

pub fn syscall0(asm: &mut Builder) {
    comment!(asm, "-- syscall0 --");
    asm!(
        asm,
        // Syscall number
        ("pop", "rax"),
        ("syscall"),
        ("push", "rax")
    );
}

pub fn syscall1(asm: &mut Builder) {
    comment!(asm, "-- syscall1 --");
    asm!(
        asm,
        /// Syscall number
        ("pop", "rax"),
        ("pop", "rdi"),
        ("syscall"),
        ("push", "rax")
    );
}

pub fn syscall2(asm: &mut Builder) {
    comment!(asm, "-- syscall2 --");
    asm!(
        asm,
        /// Syscall number
        ("pop", "rax"),
        ("pop", "rdi"),
        ("pop", "rsi"),
        ("syscall"),
        ("push", "rax")
    );
}

pub fn syscall3(asm: &mut Builder) {
    comment!(asm, "-- syscall3 --");
    asm!(
        asm,
        /// Syscall number
        ("pop", "rax"),
        ("pop", "rdi"),
        ("pop", "rsi"),
        ("pop", "rdx"),
        ("syscall"),
        ("push", "rax")
    );
}

pub fn syscall4(asm: &mut Builder) {
    comment!(asm, "-- syscall4 --");
    asm!(
        asm,
        /// Syscall number
        ("pop", "rax"),
        ("pop", "rdi"),
        ("pop", "rsi"),
        ("pop", "rdx"),
        ("pop", "r10"),
        ("syscall"),
        ("push", "rax")
    );
}

pub fn syscall5(asm: &mut Builder) {
    comment!(asm, "-- syscall5 --");
    asm!(
        asm,
        /// Syscall number
        ("pop", "rax"),
        ("pop", "rdi"),
        ("pop", "rsi"),
        ("pop", "rdx"),
        ("pop", "r10"),
        ("pop", "r8"),
        ("syscall"),
        ("push", "rax")
    );
}

pub fn syscall6(asm: &mut Builder) {
    comment!(asm, "-- syscall6 --");
    asm!(
        asm,
        /// Syscall number
        ("pop", "rax"),
        ("pop", "rdi"),
        ("pop", "rsi"),
        ("pop", "rdx"),
        ("pop", "r10"),
        ("pop", "r8"),
        ("pop", "r9"),
        ("syscall"),
        ("push", "rax")
    );
}

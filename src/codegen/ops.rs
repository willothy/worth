use crate::{asm, asm_line, comment};

pub fn add(asm: &mut Vec<String>) {
    comment!(asm, "-- add --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("add", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn sub(asm: &mut Vec<String>) {
    comment!(asm, "-- sub --");
    asm!(
        asm,
        ("pop", "rbx"),
        ("pop", "rax"),
        ("sub", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn mul(asm: &mut Vec<String>) {
    comment!(asm, "-- mul --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("imul", "rbx"),
        ("push", "rax")
    );
}

pub fn div(asm: &mut Vec<String>) {
    comment!(asm, "-- div --");
    // TODO: Test this
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("cqo"),
        ("idiv", "rbx"),
        ("push", "rax")
    );
}

pub fn rem(asm: &mut Vec<String>) {
    comment!(asm, "-- mod --");
    // TODO: Test this
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rcx"),
        ("cqo"),
        ("idiv", "rcx"),
        ("push", "rdx")
    );
}

pub fn not(asm: &mut Vec<String>) {
    comment!(asm, "-- not --");
    asm!(asm, ("pop", "rax"), ("neg", "rax"), ("push", "rax"));
}

pub fn and(asm: &mut Vec<String>) {
    comment!(asm, "-- and --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("and", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn or(asm: &mut Vec<String>) {
    comment!(asm, "-- or --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("or", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn xor(asm: &mut Vec<String>) {
    comment!(asm, "-- xor --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rbx"),
        ("xor", "rax, rbx"),
        ("push", "rax")
    );
}

pub fn shl(asm: &mut Vec<String>) {
    comment!(asm, "-- shl --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rcx"),
        ("shl", "rax, cl"),
        ("push", "rax")
    );
}

pub fn shr(asm: &mut Vec<String>) {
    comment!(asm, "-- shr --");
    asm!(
        asm,
        ("pop", "rax"),
        ("pop", "rcx"),
        ("shr", "rax, cl"),
        ("push", "rax")
    );
}

pub fn eq(asm: &mut Vec<String>) {
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

pub fn neq(asm: &mut Vec<String>) {
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

pub fn lt(asm: &mut Vec<String>) {
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

pub fn gt(asm: &mut Vec<String>) {
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

pub fn lte(asm: &mut Vec<String>) {
    comment!(asm, "-- le --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rax"),
        ("pop", "rbx"),
        ("cmp", "rax, rbx"),
        ("cmovle", "rcx, rdx"),
        ("push", "rcx")
    );
}

pub fn gte(asm: &mut Vec<String>) {
    comment!(asm, "-- ge --");
    asm!(
        asm,
        ("mov", "rcx, 0"),
        ("mov", "rdx, 1"),
        ("pop", "rax"),
        ("pop", "rbx"),
        ("cmp", "rax, rbx"),
        ("cmovge", "rcx, rdx"),
        ("push", "rcx")
    );
}

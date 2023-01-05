use crate::{asm, comment};

pub fn add(asm: &mut Vec<String>) {
    comment!(asm, "-- add --");
    asm!(
        asm,
        "pop     rax",
        "pop     rbx",
        "add     rax, rbx",
        "push    rax"
    );
}

pub fn sub(asm: &mut Vec<String>) {
    comment!(asm, "-- sub --");
    asm!(
        asm,
        "pop     rax",
        "pop     rbx",
        "sub     rax, rbx",
        "push    rax"
    );
}

pub fn mul(asm: &mut Vec<String>) {
    comment!(asm, "-- mul --");
    asm!(
        asm,
        "pop     rax",
        "pop     rbx",
        "imul    rbx",
        "push    rax"
    );
}

pub fn div(asm: &mut Vec<String>) {
    comment!(asm, "-- div --");
    todo!()
}

pub fn rem(asm: &mut Vec<String>) {
    comment!(asm, "-- mod --");
    todo!()
}

pub fn not(asm: &mut Vec<String>) {
    comment!(asm, "-- not --");
    todo!()
}

pub fn and(asm: &mut Vec<String>) {
    comment!(asm, "-- and --");
    todo!()
}

pub fn or(asm: &mut Vec<String>) {
    comment!(asm, "-- or --");
    todo!()
}

pub fn xor(asm: &mut Vec<String>) {
    comment!(asm, "-- xor --");
    todo!()
}

pub fn shl(asm: &mut Vec<String>) {
    comment!(asm, "-- shl --");
    todo!()
}

pub fn shr(asm: &mut Vec<String>) {
    comment!(asm, "-- shr --");
    todo!()
}

pub fn eq(asm: &mut Vec<String>) {
    comment!(asm, "-- eq --");
    asm!(
        asm,
        "mov     rcx, 0",
        "mov     rdx, 1",
        "pop     rax",
        "pop     rbx",
        "cmp     rax, rbx",
        "cmove   rcx, rdx",
        "push    rcx"
    );
}

pub fn neq(asm: &mut Vec<String>) {
    comment!(asm, "-- ne --");
    todo!()
}

pub fn lt(asm: &mut Vec<String>) {
    comment!(asm, "-- lt --");
    todo!()
}

pub fn gt(asm: &mut Vec<String>) {
    comment!(asm, "-- gt --");
    todo!()
}

pub fn lte(asm: &mut Vec<String>) {
    comment!(asm, "-- le --");
    todo!()
}

pub fn gte(asm: &mut Vec<String>) {
    comment!(asm, "-- ge --");
    todo!()
}

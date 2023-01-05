#[macro_export]
macro_rules! comment {
    ($asm:ident, $s:expr) => {
        $asm.push(format!(";; {}", $s));
    };
}

#[macro_export]
macro_rules! segment {
    ($asm:ident, $s:expr) => {
        $asm.push(format!("segment .{}", $s));
    };
}

#[macro_export]
macro_rules! global {
    ($asm:ident, $s:expr) => {
        $asm.push(format!("global {}", $s));
    };
}

#[macro_export]
macro_rules! label {
    ($asm:ident, $s:expr) => {
        $asm.push(format!("{}:", $s));
    };
}

#[macro_export]
macro_rules! asm {
    ($asm:ident, $($s:expr),*) => {
        $($asm.push("\t".to_string() + $s));*
    };
}

#[macro_export]
macro_rules! sys_exit {
    ($asm:ident, $code:expr) => {
        use crate::asm;
        asm!(
            $asm,
            "mov     rax, 60",
            &format!("mov     rdi, {}", $code),
            "syscall"
        );
    };
    ($asm:ident, $code:expr, $comment:expr) => {
        use crate::{asm, comment};
        comment!($asm, $comment);
        asm!(
            $asm,
            "mov rax, 60",
            &format!("mov rdi, {}", $code),
            "syscall"
        );
    };
}

#[macro_export]
macro_rules! comment {
    ($asm:ident, $s:expr) => {
        $asm.push(format!("{:4};; {}", " ", $s));
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

    ($asm:ident, $fmt:expr, $s:expr) => {
        $asm.push(format!("{}:", format!($fmt, $s)));
    };
}

#[macro_export]
macro_rules! asm {
    ($asm:ident, $($s:expr),*) => {
        $($asm.push("\t".to_string() + $s));*
    };

    ($asm:ident, $a:expr, $($s:expr),*, with $comment:expr) => {
        comment!($asm, $comment);
        $asm.push("    ".to_string() + $a);
        $($asm.push("    ".to_string() + $s));*
    };

    ($asm:ident, $s:expr, with $comment:expr) => {
        $asm.push(format!("    {:36}", $s) + "; " + $comment);
    };

    ($asm:ident, $fmt:expr, @ $s:expr) => {
        $asm.push(format!("    {}", format!($fmt, $s)));
    };

    ($asm:ident, $fmt:expr, @ $s:expr, with $comment:expr) => {
        $asm.push(format!("    {:36}; {}", format!($fmt, $s), $comment));
    };
}

#[macro_export]
macro_rules! intrinsics {
    (
        $($s:ident),*
    ) => {
        #[derive(Debug, IntoStaticStr, EnumString, Clone)]
        #[strum(ascii_case_insensitive)]
        pub enum Intrinsic {
            $($s),*
        }

        impl Intrinsic {
            pub fn compile(&self) -> fn(&mut Vec<String>) {
                use Intrinsic::*;
                use crate::codegen::intrinsics::*;
                match self {
                    $($s => casey::lower!($s)),*
                }
            }
        }
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

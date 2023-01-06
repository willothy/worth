#[macro_export]
macro_rules! comment {
    ($asm:ident, $s:expr) => {
        $asm.push(format!("{:4};; {}", " ", $s));
    };

    ($asm:ident, $fmt:expr, $s:expr) => {
        $asm.push(format!("{:4};; {}", " ", format!($fmt, $s)));
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
    ($asm:ident, $($(#[$cmt:meta])? ($($args:expr),+)),+) => {
        $(
            {
                $asm.push(asm_line!($($args),+$(; $cmt)?))
            }
        )*
    };
}

#[macro_export]
macro_rules! asm_line {
    ($op:expr) => {
        format!("{0:4}{1:4}", " ", $op)
    };
    ($op:expr; $comment:meta) => {
        format!("{0:4}{1:8}{0:28};; {2}", " ", $op, {
            let comment = stringify!($comment).to_string();
            comment[8..comment.len() - 1].to_string().trim()
        })
    };
    ($op:expr, $args:expr) => {
        format!("{0:4}{1:8}{2}", " ", $op, $args)
    };
    ($op:expr, $fmt:literal, $args:expr) => {
        format!("{0:4}{1:8}{2}", " ", $op, format!($fmt, $args))
    };
    ($op:expr, $args:expr; $comment:meta) => {
        format!("{0:4}{1:8}{2:28};; {3}", " ", $op, $args, {
            let comment = stringify!($comment).to_string();
            comment[8..comment.len() - 1].to_string().trim()
        })
    };
    ($op:expr, $fmt:literal, $args:expr; $comment:meta) => {
        format!("{0:4}{1:8}{2:28};; {3}", " ", $op, format!($fmt, $args), {
            let comment = stringify!($comment).to_string();
            comment[9..comment.len() - 1].to_string().trim()
        })
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
macro_rules! syscalls {
    (
        $($s:ident = $v:literal),*
    ) => {
        #[repr(i64)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Syscall {
            $($s = $v),*
        }
    };
}

/// Generates a syscall
/// Order:
/// rax, rbx, rcx, rdx, rsi, rdi, rbp
#[macro_export]
macro_rules! syscall {
    ($asm:ident, $op:ident) => {
        let _syscall_number = crate::codegen::syscalls::Syscall::$op as i64;
        comment!(
            $asm,
            "-- syscall (0) {}",
            &format!("{}{}", _syscall_op_, {
                let mut comment = stringify!($op).to_string();
                if comment.trim().is_empty() {
                    comment = " --".to_string();
                } else {
                    comment = comment[9..comment.len() - 1].to_string().trim().to_string();
                    comment = format!(": {} --", comment);
                }
                comment
            })
        );
        asm!($asm, ("mov", "rax, {}", _syscall_op_));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:ident, $arg1:expr) => {
        let _syscall_number = crate::codegen::intrinsics::Syscall::$op as i64;
        comment!(
            $asm,
            "-- syscall (1) {}",
            &format!("{}{}", _syscall_number, {
                let mut comment = stringify!($op).to_string();
                if comment.trim().is_empty() {
                    comment = " --".to_string();
                } else {
                    comment = format!(": {} --", comment);
                }
                comment
            })
        );
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("mov", "rbx, {}", $arg1));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:ident, $arg1:expr, $arg2:expr) => {
        let _syscall_number = crate::codegen::syscalls::Syscall::$op as i64;
        comment!(
            $asm,
            "-- syscall (2) {}",
            &format!("{}{}", _syscall_number, {
                let mut comment = stringify!($op).to_string();
                if comment.trim().is_empty() {
                    comment = " --".to_string();
                } else {
                    comment = format!(": {} --", comment);
                }
                comment
            })
        );
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("mov", "rbx, {}", $arg1));
        asm!($asm, ("mov", "rcx, {}", $arg2));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:ident, $arg1:expr, $arg2:expr, $arg3:expr) => {
        let _syscall_number = crate::codegen::syscalls::Syscall::$op as i64;
        comment!(
            $asm,
            "-- syscall (3) {}",
            &format!("{}{}", _syscall_number, {
                let mut comment = stringify!($op).to_string();
                if comment.trim().is_empty() {
                    comment = " --".to_string();
                } else {
                    comment = format!(": {} --", comment);
                }
                comment
            })
        );
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("mov", "rbx, {}", $arg1));
        asm!($asm, ("mov", "rcx, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:ident, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
        let _syscall_number = crate::codegen::syscalls::Syscall::$op as i64;
        comment!(
            $asm,
            "-- syscall (4) {}",
            &format!("{}{}", _syscall_number, {
                let mut comment = stringify!($op).to_string();
                if comment.trim().is_empty() {
                    comment = " --".to_string();
                } else {
                    comment = format!(": {} --", comment);
                }
                comment
            })
        );
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("mov", "rbx, {}", $arg1));
        asm!($asm, ("mov", "rcx, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "rsi, {}", $arg4));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:ident, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {
        let _syscall_number = crate::codegen::syscalls::Syscall::$op as i64;
        comment!(
            $asm,
            "-- syscall (5) {}",
            &format!("{}{}", _syscall_number, {
                let mut comment = stringify!($op).to_string();
                if comment.trim().is_empty() {
                    comment = " --".to_string();
                } else {
                    comment = format!(": {} --", comment);
                }
                comment
            })
        );
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("mov", "rbx, {}", $arg1));
        asm!($asm, ("mov", "rcx, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "rsi, {}", $arg4));
        asm!($asm, ("mov", "rdi, {}", $arg5));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:ident, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr) => {
        let _syscall_number = crate::codegen::syscalls::Syscall::$op as i64;
        comment!(
            $asm,
            "-- syscall (6) {}",
            &format!("{}{}", _syscall_number, {
                let mut comment = stringify!($op).to_string();
                if comment.trim().is_empty() {
                    comment = " --".to_string();
                } else {
                    comment = format!(": {} --", comment);
                }
                comment
            })
        );
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("mov", "rbx, {}", $arg1));
        asm!($asm, ("mov", "rcx, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "rsi, {}", $arg4));
        asm!($asm, ("mov", "rdi, {}", $arg5));
        asm!($asm, ("mov", "rbp, {}", $arg6));
        asm!($asm, ("syscall"));
    };
}

#[macro_export]
macro_rules! sys_exit {
    ($asm:ident, $code:expr) => {
        syscall!($asm, Exit, $code);
    };
    ($asm:ident, $code:expr, $comment:expr) => {
        comment!($asm, $comment);
        syscall!($asm, Exit, $code);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asm_macro_test() {
        let mut asm = Vec::new();
        asm!(
            asm,
            /// Test comment
            ("mov", "rax, 60"),
            ("mov", "rdi, {}", 0),
            ("syscall")
        );
        assert_eq!(
            asm[0],
            format!(
                "{0:4}{1:8}{2:28};; {3}",
                " ", "mov", "rax, 60", "Test comment"
            )
        );
        assert_eq!(asm[1], format!("{0:4}{1:8}{2}", " ", "mov", "rdi, 0"));
        assert_eq!(asm[2], "    syscall");
    }
}

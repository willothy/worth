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

syscalls! {
    Exit = 60
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
            &format!("{}{}", _syscall_number, {
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
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:ident, $arg1:expr) => {
        let _syscall_number = crate::codegen::syscalls::Syscall::$op as i64;
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "r10, {}", $arg4));
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "r10, {}", $arg4));
        asm!($asm, ("mov", "r8, {}", $arg5));
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "r10, {}", $arg4));
        asm!($asm, ("mov", "r8, {}", $arg5));
        asm!($asm, ("mov", "r9, {}", $arg6));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:literal) => {
        let _syscall_number = $op as i64;
        comment!(
            $asm,
            "-- syscall (0) {}",
            &format!("{}{}", _syscall_number, {
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
        asm!($asm, ("mov", "rax, {}", _syscall_number));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:literal, $arg1:expr) => {
        let _syscall_number = $op as i64;
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:literal, $arg1:expr, $arg2:expr) => {
        let _syscall_number = $op as i64;
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:literal, $arg1:expr, $arg2:expr, $arg3:expr) => {
        let _syscall_number = $op as i64;
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:literal, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr) => {
        let _syscall_number = $op as i64;
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "r10, {}", $arg4));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:literal, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr) => {
        let _syscall_number = $op as i64;
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "r10, {}", $arg4));
        asm!($asm, ("mov", "r8, {}", $arg5));
        asm!($asm, ("syscall"));
    };
    ($asm:ident, $op:literal, $arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr, $arg6:expr) => {
        let _syscall_number = $op as i64;
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
        asm!($asm, ("mov", "rdi, {}", $arg1));
        asm!($asm, ("mov", "rsi, {}", $arg2));
        asm!($asm, ("mov", "rdx, {}", $arg3));
        asm!($asm, ("mov", "r10, {}", $arg4));
        asm!($asm, ("mov", "r8, {}", $arg5));
        asm!($asm, ("mov", "r9, {}", $arg6));
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

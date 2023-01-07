pub use casey::lower;

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
        $($s:ident $(= $val:literal)?),*
    ) => {
        #[derive(Debug, Clone)]
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

        impl From<&Intrinsic> for &str {
            fn from(i: &Intrinsic) -> Self {
                use Intrinsic::*;
                match i {
                    $($s => intrinsic_str!(lower!, $s$(, $val)?)),*
                }
            }
        }

        impl FromStr for Intrinsic {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use Intrinsic::*;
                match s {
                    $(intrinsic_str!(lower!, $s$(, $val)?) => Ok($s),)*
                    _ => {
                        //println!("Unknown intrinsic: {}", s);
                        Err(())
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! intrinsic_str {
    ($lower:ident !, $s:ident, $val:literal) => {
        $val
    };
    ($lower:ident !, $s:ident) => {
        $lower!(stringify!($s))
    };
}

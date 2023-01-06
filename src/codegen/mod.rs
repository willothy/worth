mod compile;
pub mod intrinsics;
mod macros;
mod ops;
mod syscalls;

pub use compile::compile;
pub use compile::MEM_CAPACITY;

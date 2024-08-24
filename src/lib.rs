mod types;
#[macro_use]
mod from_args;
mod emitter;
mod macros;
mod utils;

mod async_emitter;

pub use async_emitter::AsyncEventEmitter;
pub use emitter::EventEmitter;
pub use macros::emit;

#[cfg(test)]
mod tests;

#[cfg(any(feature = "sync", feature = "async"))]
mod types;
#[cfg(any(feature = "sync", feature = "async"))]
#[macro_use]
mod from_args;
#[cfg(feature = "sync")]
mod emitter;
#[cfg(any(feature = "sync", feature = "async"))]
mod macros;
#[cfg(any(feature = "sync", feature = "async"))]
mod utils;

#[cfg(feature = "async")]
mod async_emitter;

#[cfg(feature = "sync")]
pub use emitter::EventEmitter;

#[cfg(feature = "async")]
pub use async_emitter::AsyncEventEmitter;

#[cfg(feature = "async")]
pub use emitter_runtime::*;

#[cfg(any(feature = "sync", feature = "async"))]
pub use macros::emit;

#[cfg(test)]
mod tests;

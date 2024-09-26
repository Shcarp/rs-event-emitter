#[cfg(any(feature = "sync", feature = "async", feature = "wasm"))]
mod types;

#[cfg(any(feature = "sync", feature = "async", feature = "wasm"))]
mod macros;
#[cfg(any(feature = "sync", feature = "async", feature = "wasm"))]
#[macro_use]
mod from_args;
#[cfg(feature = "sync")]
mod emitter;

#[cfg(any(feature = "sync", feature = "async", feature = "wasm"))]
mod utils;

#[cfg(feature = "wasm")]
mod wasm_emitter;

#[cfg(feature = "wasm")]
pub use wasm_emitter::*;

#[cfg(feature = "async")]
pub use async_emitter::AsyncEventEmitter;

#[cfg(feature = "async")]
pub use emitter_runtime::*;

#[cfg(feature = "async")]
mod async_emitter;

#[cfg(feature = "sync")]
pub use emitter::EventEmitter;


#[cfg(any(feature = "sync", feature = "async"))]
pub use macros::emit;

#[cfg(test)]
mod tests;

#![allow(unused_imports)]
use super::*;
#[cfg(feature = "async")]
mod test_async_event_base;
#[cfg(feature = "sync")]
mod test_basic_event_emission;
#[cfg(feature = "sync")]
mod test_complex_event_emitter;
#[cfg(feature = "sync")]
mod test_event_with_multiple_arguments;
#[cfg(feature = "sync")]
mod test_event_with_off_listening;
#[cfg(feature = "sync")]
mod test_event_with_off_multithreaded;
#[cfg(feature = "sync")]
mod test_multi_threaded_emission;
#[cfg(feature = "sync")]
mod test_multiple_event_types;
#[cfg(feature = "sync")]
mod test_multiple_handlers;
#[cfg(feature = "wasm")]
mod test_wasm_event;
#[cfg(feature = "async")]
use futures::future::BoxFuture;
use std::any::Any;
#[cfg(feature = "wasm")]
use std::cell::RefCell;
#[cfg(feature = "wasm")]
use std::rc::Rc;
#[cfg(any(feature = "sync", feature = "async"))]
use std::sync::Arc;
use uuid::Uuid;

#[cfg(any(feature = "sync", feature = "async"))]
pub type Param = Arc<dyn Any + Send + Sync>;

#[cfg(feature = "wasm")]
pub type Param = Rc<dyn Any>;

pub type HandlerId = Uuid;

#[cfg(feature = "sync")]
pub type BoxedHandler = (HandlerId, Arc<dyn Fn(&[Param]) + Send + Sync>);

#[cfg(feature = "wasm")]
pub type BoxedHandler = (HandlerId, Rc<RefCell<dyn Fn(&[Param])>>);

#[cfg(feature = "async")]
pub type BoxedAsyncHandler = (
    HandlerId,
    Arc<dyn Fn(&[Param]) -> BoxFuture<'static, ()> + Send + Sync>,
);

#[cfg(feature = "async")]
use futures::future::BoxFuture;

use std::any::Any;
use std::sync::Arc;
use uuid::Uuid;

pub type ArcAny = Arc<dyn Any + Send + Sync>;
pub type HandlerId = Uuid;

#[cfg(feature = "sync")]
pub type BoxedHandler = (HandlerId, Arc<dyn Fn(&[ArcAny]) + Send + Sync>);

#[cfg(feature = "async")]
pub type BoxedAsyncHandler = (
    HandlerId,
    Arc<dyn Fn(&[ArcAny]) -> BoxFuture<'static, ()> + Send + Sync>,
);

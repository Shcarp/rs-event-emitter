use std::any::Any;
use std::sync::Arc;

use uuid::Uuid;

pub type ArcAny = Arc<dyn Any + Send + Sync>;
pub type HandlerId = Uuid;

pub type BoxedHandler = (HandlerId, Arc<dyn Fn(&[ArcAny]) + Send + Sync>);
pub type BoxedAsyncHandler = (HandlerId, Arc<dyn Fn(&[ArcAny]) -> futures::future::BoxFuture<'static, ()> + Send + Sync>);
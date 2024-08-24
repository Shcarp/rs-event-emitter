use std::any::Any;
use std::sync::Arc;

pub type ArcAny = Arc<dyn Any + Send + Sync>;
pub type HandlerId = usize;
pub type BoxedHandler = (HandlerId, Arc<dyn Fn(&[ArcAny]) + Send + Sync>);
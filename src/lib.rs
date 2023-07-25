use dashmap::DashMap;
use uuid::Uuid;
use std::{any::Any, sync::Arc, fmt::Debug};

pub trait Handle {
    fn call(&self, data: &Box<dyn Any>);
    fn cmp(&self, other: &dyn Handle) -> bool;
    fn id(&self) -> Uuid;
}

#[derive(Clone)]
pub struct EventHandler<T: 'static> {
    uuid: Uuid,
    handler: fn(T),
}

unsafe impl<T> Send for EventHandler<T> {}
unsafe impl<T> Sync for EventHandler<T> {}

impl<T: Clone> EventHandler<T> {
    pub fn new(handler: fn(T)) -> Self
    {
        EventHandler {
            handler,
            uuid: Uuid::new_v4(),
        }
    }
}

impl<T: Clone + Debug> Handle for EventHandler<T> {
    fn call(&self, data: &Box<dyn Any>) {
        if let Some(value) = data.downcast_ref::<T>() {
            (self.handler)(value.clone());
        }
    }

    fn id(&self) -> Uuid {
        self.uuid
    }
  
    fn cmp(&self, other: &dyn Handle) -> bool {
        self.id() == other.id()
    }
}

#[derive(Clone)]
pub struct EventEmitter {
    event_handlers: Arc<DashMap<&'static str, Vec<Arc<dyn Handle>>>>,
}

unsafe impl Send for EventEmitter {}
unsafe impl Sync for EventEmitter {}

impl EventEmitter {
    pub fn new() -> Self {
        EventEmitter {
            event_handlers: Arc::new(DashMap::new()),
        }
    }

    pub fn on(&self, event: &'static str, handler: Arc<dyn Handle>) {
        self.event_handlers
            .entry(event)
            .or_insert(Vec::new())
            .push(handler);
    }

    pub fn off(&self, event: &str, handler: Arc<dyn Handle>) {
        if let Some(mut event_handlers) = self.event_handlers.get_mut(event) {
            event_handlers.retain(|h| !h.cmp(&*handler));
        }
    }

    pub fn emit(&self, event: &str, data: Box<dyn Any>) {
        if let Some(event_handlers) = self.event_handlers.get(event) {
            for handler in event_handlers.iter() {
                handler.call(&data);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Create a new EventEmitter
        let emitter = EventEmitter::new();

        // Define the event handlers
        let handler1 = EventHandler::new(|(a, b, c): (i32, &str, f64)| {
            println!("event1: {}, {}, {}", a, b, c);
            assert_eq!(a, 42);
            assert_eq!(b, "hello");
            assert_eq!(c, 3.14);
        });

        let handler2 = EventHandler::new(|(name, age): (String, u32)| {
            println!("event2: {}, {}", name, age);
            assert_eq!(name, "John");
            assert_eq!(age, 25);
        });

        // Register the event handlers
        emitter.on("event1", Arc::new(handler1.clone()));
        emitter.on("event2", Arc::new(handler2.clone()));

        // Emit events
        emitter.emit("event1", Box::new((42, "hello", 3.14)));
        emitter.emit("event2", Box::new(("John".to_string(), 25 as u32)));

        // Unregister event handlers
        emitter.off("event1", Arc::new(handler1));
        emitter.off("event2", Arc::new(handler2)); 

        // Emit events again, but handlers should not be called
        emitter.emit("event1", Box::new((41, "world", 2.71)));
        emitter.emit("event2", Box::new(("Alice".to_string(), 30 as u32)));
    }
}
 
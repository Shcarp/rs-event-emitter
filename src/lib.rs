use dashmap::DashMap;
use std::{any::Any, sync::Arc};

pub trait Handle {
    fn call(&self, data: Box<dyn Any>);
    fn cmp(&self, other: &dyn Handle) -> bool;
}

#[derive(Clone)]
pub struct EventHandler<T: 'static> {
    handler: Arc<Box<dyn Fn(T) + Send + Sync>>,
}

unsafe impl<T> Send for EventHandler<T> {}
unsafe impl<T> Sync for EventHandler<T> {}

impl<T: Clone> EventHandler<T> {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        EventHandler {
            handler: Arc::new(Box::new(handler)),
        }
    }
}

impl<T: Clone> Handle for EventHandler<T> {
    fn call(&self, data: Box<dyn Any>) {
        if let Some(value) = data.downcast_ref::<T>() {
            (self.handler)(value.clone());
        }
    }
    fn cmp(&self, other: &dyn Handle) -> bool {
        let o_ptr = other as *const dyn Handle;
        let ptr = self as *const dyn Handle;
        ptr == o_ptr
    }
}

#[derive(Clone)]
pub struct EventEmitter {
    event_handlers: Arc<DashMap<&'static str, Vec<Box<dyn Handle>>>>,
}

unsafe impl Send for EventEmitter {}
unsafe impl Sync for EventEmitter {}

impl EventEmitter {
    pub fn new() -> Self {
        EventEmitter {
            event_handlers: Arc::new(DashMap::new()),
        }
    }

    pub fn on(&self, event: &'static str, handler: impl Handle + 'static) {
        self.event_handlers
            .entry(event)
            .or_insert(Vec::new())
            .push(Box::new(handler));
    }

    pub fn off(&self, event: &str, handler: &(impl Handle + 'static)) {
        if let Some(mut event_handlers) = self.event_handlers.get_mut(event) {
            event_handlers.retain(|h| {
                h.cmp(handler)
            });
        }
    }

    pub fn emit<T: 'static + Clone>(&self, event: &str, data: T) {
        if let Some(event_handlers) = self.event_handlers.get(event) {
            for handler in event_handlers.iter() {
                handler.call(Box::new(data.clone()) as Box<dyn Any>);
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
        emitter.on("event1", handler1.clone());
        emitter.on("event2", handler2.clone());
        // Emit events
        emitter.emit("event1", (42, "hello", 3.14));
        emitter.emit("event2", ("John".to_string(), 25 as u32));

        // Unregister event handlers
        emitter.off("event1", &handler1);
        emitter.off("event2", &handler2);

        // Emit events again, but handlers should not be called
        emitter.emit("event1", (41, "world", 2.71));
        emitter.emit("event2", ("Alice".to_string(), 30));
    }
}

use dashmap::DashMap;
use std::{any::Any, sync::Arc};

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

    fn call(&self, data: Box<dyn Any>) {
        if let Some(value) = data.downcast_ref::<T>() {
            (self.handler)(value.clone());
        }
    }
}

#[derive(Clone)]
pub struct EventEmitter {
    event_handlers: Arc<DashMap<String, Vec<Box<dyn Any>>>>,
}

unsafe impl Send for EventEmitter {}
unsafe impl Sync for EventEmitter {}

impl EventEmitter {
    pub fn new() -> Self {
        EventEmitter {
            event_handlers: Arc::new(DashMap::new()),
        }
    }

    pub fn on<T>(&self, event: String, handler: EventHandler<T>) {
        self.event_handlers
            .entry(event)
            .or_insert(Vec::new())
            .push(Box::new(handler));
    }

    pub fn off<T>(&self, event: &str, handler: &EventHandler<T>) {
        if let Some(mut event_handlers) = self.event_handlers.get_mut(event) {
            event_handlers.retain(|h| {
                if let Some(value) = h.downcast_ref::<EventHandler<T>>() {
                    let ptr: *const EventHandler<T> = value as *const EventHandler<T>;
                    let other_ptr: *const EventHandler<T> =
                        handler as *const EventHandler<T>;
                    return ptr == other_ptr;
                }
                false
            });
        }
    }

    pub fn emit<T: 'static + Clone>(&self, event: &str, data: T) {
        if let Some(event_handlers) = self.event_handlers.get(&event.to_string()) {
            for handler in event_handlers.iter() {
                if let Some(handler) = handler.downcast_ref::<EventHandler<T>>() {
                    handler.call(Box::new(data.clone()) as Box<dyn Any>);
                }
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
            assert_eq!(a, 42);
            assert_eq!(b, "hello");
            assert_eq!(c, 3.14);
        });

        let handler2 = EventHandler::new(|(name, age): (String, u32)| {
            assert_eq!(name, "John");
            assert_eq!(age, 25);
        });

        // Register the event handlers
        emitter.on("event1".to_string(), handler1.clone());
        emitter.on("event2".to_string(), handler2.clone());
        // Emit events
        emitter.emit("event1", (42, "hello", 3.14));
        emitter.emit("event2", ("John".to_string(), 25));

        // Unregister event handlers
        emitter.off("event1", &handler1);
        emitter.off("event2", &handler2);

        // Emit events again, but handlers should not be called
        emitter.emit("event1", (41, "world", 2.71));
        emitter.emit("event2", ("Alice".to_string(), 30));
    }
}

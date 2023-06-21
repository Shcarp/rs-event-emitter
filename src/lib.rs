use dashmap::DashMap;
use std::{any::Any, ptr, sync::Arc};

#[derive(Clone)]
pub struct EventHandler<T: 'static> {
    handler: Arc<Box<dyn Fn(&T) + Send + Sync>>,
}

unsafe impl<T> Send for EventHandler<T> {}
unsafe impl<T> Sync for EventHandler<T> {}

impl<T: 'static> PartialEq for EventHandler<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other)
    }
}

impl<T: 'static> Eq for EventHandler<T> {}

impl<T: 'static> EventHandler<T> {
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        EventHandler {
            handler: Arc::new(Box::new(handler)),
        }
    }

    pub fn call(&self, data: &Box<dyn Any>) {
        if let Some(value) = data.downcast_ref::<T>() {
            (self.handler)(value);
        }
    }

    pub fn clone(&self) -> Self {
        EventHandler {
            handler: self.handler.clone(),
        }
    }

    pub fn cmp(&self, other: &EventHandler<T>) -> bool {
        ptr::eq(self.handler.as_ref(), other.handler.as_ref())
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

    pub fn emit<T: 'static>(&self, event: &str, data: Box<dyn Any>) {
        if let Some(event_handlers) = self.event_handlers.get(&event.to_string()) {
            for handler in event_handlers.iter() {
                if let Some(handler) = handler.downcast_ref::<EventHandler<T>>() {
                    handler.call(&data);
                    continue;
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
        let handler1 = EventHandler::<i32>::new(|data| {
            assert_eq!(*data, 42);
        });

        let handler2 = EventHandler::<String>::new(|data| {
            assert_eq!(data, "Hello");
        });

        // Register the event handlers
        emitter.on("event1".to_string(), handler1.clone());
        emitter.on("event2".to_string(), handler2.clone());

        // Emit events
        emitter.emit::<i32>("event1", Box::new(42));
        emitter.emit::<String>("event2", Box::new("Hello".to_string()));

        // Unregister event handlers
        emitter.off("event1", &handler1);
        emitter.off("event2", &handler2);

        // Emit events again, but handlers should not be called
        emitter.emit::<i32>("event1", Box::new(41));
        emitter.emit::<String>("event2", Box::new("Hello1".to_string()));
    }
}

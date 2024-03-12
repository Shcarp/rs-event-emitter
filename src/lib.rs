use dashmap::DashMap;
use std::{any::Any, fmt::Debug, sync::Arc};
use uuid::Uuid;

pub trait Handle {
    fn call(&self, data: &Box<dyn Any>);
    fn is_same(&self, other: &dyn Handle) -> bool;
    fn id(&self) -> Uuid;
    fn to_arc(&self) -> Arc<dyn Handle>;
}

#[derive(Clone)]
pub struct EventHandler<T: 'static> {
    uuid: Uuid,
    handler: Arc<Box<dyn Fn(T) -> ()>>,
}

unsafe impl<T> Send for EventHandler<T> {}
unsafe impl<T> Sync for EventHandler<T> {}

impl<T: Clone> EventHandler<T> {
    pub fn new(handler: Box<dyn Fn(T) -> ()>) -> Self {
        EventHandler {
            handler: Arc::new(handler),
            uuid: Uuid::new_v4(),
        }
    }
}

impl<T: Clone + Debug + 'static> Handle for EventHandler<T> {
    fn call(&self, data: &Box<dyn Any>) {
        if let Some(value) = data.downcast_ref::<T>() {
            (self.handler)(value.clone());
        }
    }

    fn id(&self) -> Uuid {
        self.uuid
    }

    fn is_same(&self, other: &dyn Handle) -> bool {
        self.id() == other.id()
    }

    fn to_arc(&self) -> Arc<dyn Handle> {
        Arc::new(self.clone())
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

    pub fn on(&self, event: &'static str, handler: &dyn Handle) {
        self.event_handlers
            .entry(event)
            .or_insert(Vec::new())
            .push(handler.to_arc());
    }

    pub fn off(&self, event: &str, handler: &dyn Handle) {
        if let Some(mut event_handlers) = self.event_handlers.get_mut(event) {
            event_handlers.retain(|h| !h.is_same(handler));
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
    use std::sync::{Arc, Mutex};
    use std::thread;

    struct TestData {
        called: bool,
        value: i32,
    }

    #[test]
    fn test_event_emitter() {
        let data = Arc::new(Mutex::new(TestData {
            called: false,
            value: 0,
        }));
        let data_clone = data.clone();

        let handler = EventHandler::new(Box::new(move |val: i32| {
            println!("Handler called with value: {}", val);
            let mut data = data_clone.lock().unwrap();
            data.called = true;
            data.value = val;
        }));

        let emitter = EventEmitter::new();

        // Register the event handler
        emitter.on("test_event", &handler);

        let clone_emitter = emitter.clone();

        let t_handler = thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(100));
            clone_emitter.emit("test_event", Box::new(42));
        });

        t_handler.join().unwrap();

        {
            let data = data.lock().unwrap();

            println!("{:?}", data.called);
            // Check if the handler was called and the value was set correctly
            assert!(data.called);
            assert_eq!(data.value, 42);
        }

        // Remove the event handler
        emitter.off("test_event", &handler);

        // Reset the test data
        {
            let mut data = data.lock().unwrap();
            data.called = false;
            data.value = 0;
        }

        // Emit the event again
        emitter.emit("test_event", Box::new(24));

        // Verify the handler was not called after being removed
        let data = data.lock().unwrap();
        assert!(!data.called);
        assert_eq!(data.value, 0);
    }
}

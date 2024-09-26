use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;

use crate::from_args::FromArgs;
use crate::types::{Param, BoxedHandler, HandlerId};
use crate::utils;

pub struct EventEmitter {
    handlers: Rc<RefCell<HashMap<String, Vec<BoxedHandler>>>>,
    once_handlers: Rc<RefCell<HashSet<HandlerId>>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        EventEmitter {
            handlers: Rc::new(RefCell::new(HashMap::new())),
            once_handlers: Rc::new(RefCell::new(HashSet::new())),
        }
    }

    pub fn on<F, Args>(&self, event: &str, handler: F) -> HandlerId
    where
        F: Fn(Args) + 'static,
        Args: FromArgs + 'static,
    {
        let boxed_handler: BoxedHandler = utils::create_wasm_handler(handler);
        let cloned_handler = boxed_handler.clone();

        let mut handlers = self.handlers.borrow_mut();
        handlers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(boxed_handler);

        cloned_handler.0
    }

    pub fn off(&self, event: &str, handler_id: HandlerId) {
        let mut handlers = self.handlers.borrow_mut();
        if let Some(event_handlers) = handlers.get_mut(event) {
            event_handlers.retain(|(id, _)| *id != handler_id);
        }
    }
    
    pub fn emit(&self, event: &str, args: Vec<Param>) {
        let mut handlers = self.handlers.borrow_mut();
        let mut once_handlers = self.once_handlers.borrow_mut();
        
        if let Some(event_handlers) = handlers.get_mut(event) {
            event_handlers.retain(|(id, handler)| {
                let func = handler.borrow();
                func(&args);
                
                if once_handlers.remove(id) {
                    false
                } else {
                    true
                }
            });
        }
    }

    pub fn clone(&self) -> Self {
        EventEmitter {
            handlers: Rc::clone(&self.handlers),
            once_handlers: Rc::clone(&self.once_handlers),
        }
    }

    pub fn once<F, Args>(&self, event: &str, handler: F) -> HandlerId
    where
        F: Fn(Args) + 'static,
        Args: FromArgs + 'static,
    {
        
        let handler_id = self.on(event, handler);
        self.once_handlers.borrow_mut().insert(handler_id);

        handler_id
    }

    pub fn remove_all_listeners(&self, event: &str) {
        let mut handlers = self.handlers.borrow_mut();
        handlers.remove(event);
    }

    pub fn listener_count(&self, event: &str) -> usize {
        self.handlers.borrow().get(event).map_or(0, |v| v.len())
    }

    pub fn event_names(&self) -> Vec<String> {
        self.handlers.borrow().keys().cloned().collect()
    }
}

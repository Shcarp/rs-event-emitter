use std::collections::HashMap;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

use crate::from_args::FromArgs;
use crate::types::{Param, BoxedHandler, HandlerId};
use crate::utils;

pub struct EventEmitter {
    handlers: Arc<Mutex<HashMap<String, Vec<BoxedHandler>>>>,
    sender: Sender<(String, Vec<Param>)>,
    receiver: Arc<Mutex<Receiver<(String, Vec<Param>)>>>,
    stop_sender: Sender<()>,
    stop_receiver: Arc<Mutex<Receiver<()>>>,
    thread_pool: Arc<ThreadPool>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self::with_thread_pool_size(num_cpus::get())
    }

    pub fn with_thread_pool_size(thread_pool_size: usize) -> Self {
        let (sender, receiver) = channel();
        let (stop_sender, stop_receiver) = channel();
        EventEmitter {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            receiver: Arc::new(Mutex::new(receiver)),
            sender,
            stop_sender,
            stop_receiver: Arc::new(Mutex::new(stop_receiver)),
            thread_pool: Arc::new(ThreadPool::new(thread_pool_size)),
        }
    }

    pub fn on<F, Args>(&self, event: &str, handler: F) -> HandlerId
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs + 'static,
    {
        let boxed_handler: BoxedHandler = utils::create_handler(handler);
        let cloned_handler = boxed_handler.clone();

        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(boxed_handler);

        cloned_handler.0
    }

    pub fn off(&self, event: &str, handler_id: HandlerId) {
        let mut handlers = self.handlers.lock().unwrap();
        if let Some(event_handlers) = handlers.get_mut(event) {
            event_handlers.retain(|(id, _)| *id != handler_id);
        }
    }

    pub fn emit(&self, event: &str, args: Vec<Param>) {
        let _ = self.sender.send((event.to_string(), args));
    }

    pub fn start_listening(&self) -> thread::JoinHandle<()> {
        let handlers = Arc::clone(&self.handlers);
        let receiver = Arc::clone(&self.receiver);
        let stop_receiver = Arc::clone(&self.stop_receiver);
        let thread_pool = Arc::clone(&self.thread_pool);

        thread::spawn(move || {
            loop {
                // 使用 select 来同时等待事件和停止信号
                if let Ok(_) = stop_receiver.lock().unwrap().try_recv() {
                    println!("Stop signal received, stopping listener");
                    break;
                }

                match receiver
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_millis(10))
                {
                    Ok((event, args)) => {
                        let event_handlers = {
                            let handlers = handlers.lock().unwrap();
                            handlers.get(&event).cloned()
                        };

                        if let Some(event_handlers) = event_handlers {
                            for (_, handler) in event_handlers {
                                let handler = Arc::clone(&handler);
                                let args = args.clone();
                                thread_pool.execute(move || {
                                    handler(&args);
                                });
                            }
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => {
                        println!("Channel disconnected, stopping listener");
                        break;
                    }
                }
            }
            println!("Event listener stopped");
            thread_pool.join();
        })
    }

    pub fn stop_listening(&self) {
        let _ = self.stop_sender.send(());
    }

    pub fn clone(&self) -> Self {
        EventEmitter {
            handlers: Arc::clone(&self.handlers),
            sender: self.sender.clone(),
            receiver: Arc::clone(&self.receiver),
            stop_sender: self.stop_sender.clone(),
            stop_receiver: Arc::clone(&self.stop_receiver),
            thread_pool: Arc::clone(&self.thread_pool),
        }
    }
}

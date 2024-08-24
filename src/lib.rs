mod types;
#[macro_use]
mod from_args;
mod macros;

use from_args::FromArgs;
pub use macros::emit;
pub use types::{ArcAny, HandlerId, BoxedHandler};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use threadpool::ThreadPool;


pub struct EventEmitter {
    handlers: Arc<Mutex<HashMap<String, Vec<BoxedHandler>>>>,
    sender: Sender<(String, Vec<ArcAny>)>,
    receiver: Arc<Mutex<Receiver<(String, Vec<ArcAny>)>>>,
    next_id: Arc<Mutex<HandlerId>>,
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
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn on<F, Args>(&self, event: &str, handler: F) -> HandlerId
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs + 'static,
    {
        let handler_id = {
            let mut next_id = self.next_id.lock().unwrap();
            *next_id += 1;
            *next_id
        };
        let boxed_handler: BoxedHandler = (handler_id, Arc::new(move |args: &[ArcAny]| {
            if let Some(typed_args) = Args::from_args(args) {
                handler(typed_args);
            }
        }));

        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(boxed_handler);

        handler_id
    }

    pub fn off(&self, event: &str, handler_id: HandlerId)
    {
        let mut handlers = self.handlers.lock().unwrap();
        if let Some(event_handlers) = handlers.get_mut(event) {
            event_handlers.retain(|(id, _)| {
                *id != handler_id
            });
        }
    }

    pub fn emit(&self, event: &str, args: Vec<ArcAny>) {
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

                match receiver.lock().unwrap().recv_timeout(Duration::from_millis(10)) {
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
                    },
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
            next_id: Arc::clone(&self.next_id),
            stop_sender: self.stop_sender.clone(),
            stop_receiver: Arc::clone(&self.stop_receiver),
            thread_pool: Arc::clone(&self.thread_pool),
        }
    }
}

#[cfg(test)]
mod tests;

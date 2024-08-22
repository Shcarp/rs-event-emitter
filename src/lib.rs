use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;
use threadpool::ThreadPool;

type ArcAny = Arc<dyn Any + Send + Sync>;
type HandlerId = usize;
type BoxedHandler = (HandlerId, Arc<dyn Fn(&[ArcAny]) + Send + Sync>);

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

pub trait FromArgs: Sized {
    fn from_args(args: &[ArcAny]) -> Option<Self>;
}

macro_rules! to_lowercase {
    (A) => { "a" };
    (B) => { "b" };
    (C) => { "c" };
    (D) => { "d" };
    (E) => { "e" };
    (F) => { "f" };
    (G) => { "g" };
    (H) => { "h" };
    (I) => { "i" };
    (J) => { "j" };
    (K) => { "k" };
    (L) => { "l" };
    (M) => { "m" };
    (N) => { "n" };
    (O) => { "o" };
    (P) => { "p" };
    (Q) => { "q" };
    (R) => { "r" };
    (S) => { "s" };
    (T) => { "t" };
    (U) => { "u" };
    (V) => { "v" };
    (W) => { "w" };
    (X) => { "x" };
    (Y) => { "y" };
    (Z) => { "z" };
    ($other:ident) => { stringify!($other) };
}

macro_rules! impl_from_args {
    ($($ty:ident),*) => {
        impl<$($ty: 'static + Clone),*> FromArgs for ($($ty,)*) {
            fn from_args(args: &[ArcAny]) -> Option<Self> {
                let expected_len = count_tts!($($ty)*);
                if args.len() != expected_len {
                    return None;
                }
                Some((
                    $(
                        args.get(idx_from_args!($ty))?.downcast_ref::<$ty>()?.clone(),
                    )*
                ))
            }
        }
    };
}

macro_rules! idx_from_args {
    ($ty:ident) => {{
        const LOWERCASE: &str = to_lowercase!($ty);
        let idx = LOWERCASE.as_bytes()[0].wrapping_sub(b'a') as usize;
        if idx >= 26 { panic!("Invalid type name for FromArgs: {}", stringify!($ty)); }
        idx
    }};
}
macro_rules! count_tts {
    () => {0};
    ($head:tt $($tail:tt)*) => {1 + count_tts!($($tail)*)};
}

impl_from_args!(A);
impl_from_args!(A, B);
impl_from_args!(A, B, C);
impl_from_args!(A, B, C, D);
impl_from_args!(A, B, C, D, E);
impl_from_args!(A, B, C, D, E, F);
impl_from_args!(A, B, C, D, E, F, G);
impl_from_args!(A, B, C, D, E, F, G, H);
impl_from_args!(A, B, C, D, E, F, G, H, I);
impl_from_args!(A, B, C, D, E, F, G, H, I, J);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_from_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

#[macro_export]
macro_rules! emit {
    ($emitter:expr, $event:expr, $($arg:expr),*) => {
        $emitter.emit($event, vec![$(Arc::new($arg) as ArcAny),*])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::time::Duration;

    #[derive(Clone, Debug, PartialEq)]
    struct ComplexData {
        id: usize,
        name: String,
        data: Vec<f64>,
        metadata: HashMap<String, String>,
    }

    #[test]
    fn test_basic_event_emission() {
        let emitter = EventEmitter::new();
        let _listener = emitter.start_listening();

        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        emitter.on("increment", move |(_,): (i32,)| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        emit!(emitter, "increment", 1);
        emit!(emitter, "increment", 1);

        // 给一些时间让事件被处理
        thread::sleep(Duration::from_millis(100));

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_multiple_event_types() {
        let emitter = EventEmitter::new();
        let _listener = emitter.start_listening();

        let result = Arc::new(Mutex::new(String::new()));
        let result_clone1 = Arc::clone(&result);
        let result_clone2 = Arc::clone(&result);

        emitter.on("greet", move |(name,): (String,)| {
            let mut result = result_clone1.lock().unwrap();
            *result = format!("Hello, {}!", name);
        });

        emitter.on("farewell", move |(name,): (String,)| {
            let mut result = result_clone2.lock().unwrap();
            *result = format!("Goodbye, {}!", name);
        });

        emit!(emitter, "greet", "Alice".to_string());
        thread::sleep(Duration::from_millis(50));
        assert_eq!(*result.lock().unwrap(), "Hello, Alice!");

        emit!(emitter, "farewell", "Bob".to_string());
        thread::sleep(Duration::from_millis(50));
        assert_eq!(*result.lock().unwrap(), "Goodbye, Bob!");
    }

    #[test]
    fn test_multiple_handlers() {
        let emitter = EventEmitter::new();
        let _listener = emitter.start_listening();

        let counter1 = Arc::new(AtomicI32::new(0));
        let counter2 = Arc::new(AtomicI32::new(0));
        let counter1_clone = Arc::clone(&counter1);
        let counter2_clone = Arc::clone(&counter2);

        emitter.on("increment", move |(_,): (i32,)| {
            counter1_clone.fetch_add(1, Ordering::SeqCst);
        });

        emitter.on("increment", move |(_,): (i32,)| {
            counter2_clone.fetch_add(2, Ordering::SeqCst);
        });

        emit!(emitter, "increment", 1);

        thread::sleep(Duration::from_millis(100));

        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_multi_threaded_emission() {
        let emitter = EventEmitter::with_thread_pool_size(4);
        let _listener = emitter.start_listening();

        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        emitter.on("increment", move |(_,): (i32,)| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let emitter_clone = emitter.clone();
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                emit!(emitter_clone, "increment", 1);
            }
        });

        for _ in 0..50 {
            emit!(emitter, "increment", 1);
        }

        handle.join().unwrap();

        thread::sleep(Duration::from_millis(200));

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_event_with_multiple_arguments() {
        let emitter = EventEmitter::new();
        let _listener = emitter.start_listening();

        let result = Arc::new(Mutex::new(String::new()));
        let result_clone = Arc::clone(&result);

        emitter.on("person_info", move |(name, age, city, is_student, gpa): (String, i32, String, bool, f32)| {
            let mut result = result_clone.lock().unwrap();
            *result = format!("{} is {} years old, lives in {}, student status: {}, GPA: {:.2}", 
                            name, age, city, if is_student { "Yes" } else { "No" }, gpa);
        });

        emit!(emitter, "person_info", 
            "Alice".to_string(), 
            30, 
            "New York".to_string(), 
            true, 
            3.75f32
        );

        // 给一些时间让事件被处理
        thread::sleep(Duration::from_millis(100));

        assert_eq!(*result.lock().unwrap(), 
                "Alice is 30 years old, lives in New York, student status: Yes, GPA: 3.75");
    }

    #[test]
    fn test_event_with_off_multithreaded() {
        let emitter = Arc::new(EventEmitter::new());
        let _listener = emitter.start_listening();
    
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);
    
        let handler_id = emitter.on("increment", move |(value,): (i32,)| {
            counter_clone.fetch_add(value, Ordering::SeqCst);
        });

        let total_events = 100; // 5 threads * 20 events each
        let events_before_off = 50; // We'll remove the handler halfway through
    
        // Spawn multiple threads to emit events
        let mut handles = vec![];
        for _ in 0..5 {
            let emitter_clone = Arc::clone(&emitter);
            let handle = thread::spawn(move || {
                for i in 0..20 {
                    emit!(emitter_clone, "increment", 1);
                    if i == 9 { // Pause halfway through to allow for handler removal
                        thread::sleep(Duration::from_millis(50));
                    }
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }
    
        // Wait a bit, then remove the handler
        thread::sleep(Duration::from_millis(50));
        emitter.off("increment", handler_id);
    
        // Wait for all threads to finishhandler
        for handle in handles {
            handle.join().unwrap();
        }
    
        // Give some time for any remaining events to be processed
        thread::sleep(Duration::from_millis(100));
    
        let final_count = counter.load(Ordering::SeqCst);
        println!("Final count: {}", final_count);
    
        // The final count should be greater than events_before_off but less than total_events
        assert!(
            final_count >= events_before_off && final_count < total_events,
            "Final count was {}, expected between {} and {}",
            final_count,
            events_before_off,
            total_events
        );
    
            // Emit more events after removing the handler
        for _ in 0..20 {
            emit!(emitter, "increment", 1);
        }

        thread::sleep(Duration::from_millis(100));

        // The count should not have changed
        assert_eq!(
            counter.load(Ordering::SeqCst),
            final_count,
            "Counter changed after handler was removed"
        );
    }

    #[test]
    fn test_event_with_off_listening() {
        let emitter = Arc::new(EventEmitter::new());
        let listener = emitter.start_listening();
    
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let handler_id = emitter.on("increment", move |(value,): (i32,)| {
            counter_clone.fetch_add(value, Ordering::SeqCst);
        });
    
        // Emit some events
        for _ in 0..20 {
            emit!(emitter, "increment", 1);
        }
    
        // Wait a bit, then stop listening
        thread::sleep(Duration::from_millis(50));

        println!("Stopping listening {}", counter.load(Ordering::SeqCst));

        emitter.stop_listening();

        // Give some time for the listener thread to stop
        thread::sleep(Duration::from_millis(10));

        // Emit more events after stopping listening
        for _ in 0..20 {
            emit!(emitter, "increment", 1);
        }
    
        // Give some time for any remaining events to be processed
        thread::sleep(Duration::from_millis(100));
    
        // The count should not have changed
        assert_eq!(
            counter.load(Ordering::SeqCst),
            20,
            "Counter changed after stopping listening"
        );
    
        // Emit more events after stopping listening
        for _ in 0..20 {
            emit!(emitter, "increment", 1);
        }
    
        // The count should not have changed
        assert_eq!(
            counter.load(Ordering::SeqCst),
            20,
            "Counter changed after stopping listening"
        );
    
        // Remove the handler
        emitter.off("increment", handler_id);
    
        // Emit more events after removing the handler
        for _ in 0..20 {
            emit!(emitter, "increment", 1);
        }
    
        // The count should not have changed
        assert_eq!(
            counter.load(Ordering::SeqCst),
            20,
            "Counter changed after handler was removed"
        );
    
        // Wait for the listener thread to finish
        listener.join().unwrap();
    }


    #[test]
    fn test_complex_event_emitter() {
        let emitter = Arc::new(EventEmitter::new()); // 使用4个线程的线程池
        let listener = emitter.start_listening();

        // 用于收集结果的共享数据结构
        let results = Arc::new(Mutex::new(Vec::new()));

        // 测试简单类型
        {
            let results_clone = Arc::clone(&results);
            emitter.on("simple", move |(x, y): (i32, String)| {
                let mut results = results_clone.lock().unwrap();
                results.push(format!("Simple: {} - {}", x, y));
            });
        }

        // 测试复杂类型
        {
            let results_clone = Arc::clone(&results);
            emitter.on("complex", move |(data,): (ComplexData,)| {
                let mut results = results_clone.lock().unwrap();
                results.push(format!("Complex: {} - {}", data.id, data.name));
            });
        }

        // 测试多参数
        {
            let results_clone = Arc::clone(&results);
            emitter.on("multi", move |(a, b, c, d): (i32, String, f64, bool)| {
                let mut results = results_clone.lock().unwrap();
                results.push(format!("Multi: {} {} {} {}", a, b, c, d));
            });
        }

        // 发射事件
        emit!(emitter, "simple", 42, "Hello".to_string());

        let complex_data = ComplexData {
            id: 1,
            name: "Test".to_string(),
            data: vec![1.0, 2.0, 3.0],
            metadata: {
                let mut map = HashMap::new();
                map.insert("key".to_string(), "value".to_string());
                map
            },
        };
        emit!(emitter, "complex", complex_data);

        emit!(emitter, "multi", 10, "Test".to_string(), 3.14, true);

        // 测试多线程发射
        let emitter_clone = Arc::clone(&emitter);
        let handle = thread::spawn(move || {
            for i in 0..5 {
                emit!(emitter_clone, "simple", i, format!("Thread {}", i));
                thread::sleep(Duration::from_millis(10));
            }
        });

        // 等待所有事件处理完成
        handle.join().unwrap();
        thread::sleep(Duration::from_millis(100));

        // 停止监听
        emitter.stop_listening();
        listener.join().unwrap();

        // 验证结果
        let results = results.lock().unwrap();
        assert_eq!(results.len(), 8); // 3 initial events + 5 from thread

        assert!(results.contains(&"Simple: 42 - Hello".to_string()));
        assert!(results.contains(&"Complex: 1 - Test".to_string()));
        assert!(results.contains(&"Multi: 10 Test 3.14 true".to_string()));

        for i in 0..5 {
            assert!(results.contains(&format!("Simple: {} - Thread {}", i, i)));
        }

        // 测试 off 方法
        let emitter = EventEmitter::new();
        let _listener = emitter.start_listening();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        let handler_id = emitter.on("increment", move |(_,): (i32,)| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        emit!(emitter, "increment", 1);
        emit!(emitter, "increment", 1);
        thread::sleep(Duration::from_millis(10));

        emitter.off("increment", handler_id);

        emit!(emitter, "increment", 1);

        thread::sleep(Duration::from_millis(100));

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}

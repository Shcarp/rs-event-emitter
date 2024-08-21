use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

type ArcAny = Arc<dyn Any + Send + Sync>;
type BoxedHandler = Arc<dyn Fn(&[ArcAny]) + Send + Sync>;

pub struct EventEmitter {
    handlers: Arc<Mutex<HashMap<String, Vec<BoxedHandler>>>>,
    sender: Sender<(String, Vec<ArcAny>)>,
    receiver: Arc<Mutex<Receiver<(String, Vec<ArcAny>)>>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        EventEmitter {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn on<F, Args>(&self, event: &str, handler: F)
    where
        F: Fn(Args) + Send + Sync + 'static,
        Args: FromArgs + 'static,
    {
        let boxed_handler: BoxedHandler = Arc::new(move |args: &[ArcAny]| {
            if let Some(typed_args) = Args::from_args(args) {
                handler(typed_args);
            }
        });

        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(boxed_handler);
    }
    pub fn emit(&self, event: &str, args: Vec<ArcAny>) {
        let _ = self.sender.send((event.to_string(), args));
    }

    pub fn start_listening(&self) -> thread::JoinHandle<()> {
        let handlers = Arc::clone(&self.handlers);
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            loop {
                let (event, args) = receiver.lock().unwrap().recv().unwrap();
                let handlers = handlers.lock().unwrap();
                if let Some(event_handlers) = handlers.get(&event) {
                    for handler in event_handlers {
                        let handler = Arc::clone(handler);
                        let args = args.clone();
                        thread::spawn(move || {
                            handler(&args);
                        });
                    }
                }
            }
        })
    }

    pub fn clone(&self) -> Self {
        EventEmitter {
            handlers: Arc::clone(&self.handlers),
            sender: self.sender.clone(),
            receiver: Arc::clone(&self.receiver),
        }
    }
}

pub trait FromArgs: Sized {
    fn from_args(args: &[ArcAny]) -> Option<Self>;
}

macro_rules! impl_from_args {
    ($($ty:ident),*) => {
        impl<$($ty: 'static + Clone),*> FromArgs for ($($ty,)*) {
            fn from_args(args: &[ArcAny]) -> Option<Self> {
                let expected_len = count_tts!($($ty)*);
                if args.len() != expected_len {
                    return None;
                }
                let mut index = 0;
                (
                    $(
                        {
                            let arg = args[index].downcast_ref::<$ty>()?;
                            index += 1;
                            arg.clone()
                        },
                    )*
                ).into()
            }
        }
    };
}

// Helper macro to count the number of type parameters
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
        let emitter = EventEmitter::new();
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
}

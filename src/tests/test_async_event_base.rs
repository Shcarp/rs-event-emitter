use crate::{async_emit, AsyncEventEmitter};

use futures::future::join_all;
use runtime::TokioRuntime;

use std::{
    panic,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
};

use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_on_and_emit() {
    let rt = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(rt);

    let counter = Arc::new(AtomicI32::new(0));

    let counter_clone = counter.clone();

    emitter
        .on("test_event", move |_args: ()| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        })
        .await;

    emitter.emit("test_event", vec![]).await;
    emitter.emit("test_event", vec![]).await;

    assert_eq!(counter.load(Ordering::SeqCst), 2);

    println!("done");
}

#[tokio::test]
async fn test_multiple_handlers() {
    let runtime = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(runtime);

    let counter1 = Arc::new(AtomicI32::new(0));
    let counter2 = Arc::new(AtomicI32::new(0));

    let counter1_clone = counter1.clone();
    let counter2_clone = counter2.clone();

    emitter
        .on("test_event", move |_args: ()| {
            let counter = counter1_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        })
        .await;

    emitter
        .on("test_event", move |_args: ()| {
            let counter = counter2_clone.clone();
            Box::pin(async move {
                counter.fetch_add(2, Ordering::SeqCst);
            })
        })
        .await;

    emitter.emit("test_event", vec![]).await;

    sleep(Duration::from_millis(50)).await;
    assert_eq!(counter1.load(Ordering::SeqCst), 1);
    assert_eq!(counter2.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_off() {
    let runtime = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(runtime);

    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = counter.clone();

    let handler_id = emitter
        .on("test_event", move |_args: ()| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        })
        .await;

    emitter.emit("test_event", vec![]).await;
    sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    emitter.off("test_event", handler_id).await;
    emitter.emit("test_event", vec![]).await;
    sleep(Duration::from_millis(50)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_emit_with_args() {
    let runtime = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(runtime);

    let result = Arc::new(tokio::sync::Mutex::new(String::new()));
    let result_clone = result.clone();

    emitter
        .on("greet", move |args: (String,)| {
            let result = result_clone.clone();
            Box::pin(async move {
                let mut guard = result.lock().await;
                *guard = format!("Hello, {}!", args.0);
            })
        })
        .await;

    async_emit!(emitter, "greet", "Alice".to_string());

    sleep(Duration::from_millis(50)).await;
    assert_eq!(*result.lock().await, "Hello, Alice!");
}

#[tokio::test]
async fn test_concurrent_emits() {
    let runtime = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(runtime);

    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = counter.clone();

    emitter
        .on("test_event", move |_args: ()| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                sleep(Duration::from_millis(10)).await; // Simulate some work
                counter.fetch_add(1, Ordering::SeqCst);
            })
        })
        .await;

    let futures = (0..100).map(|_| emitter.emit("test_event", vec![]));
    join_all(futures).await;

    sleep(Duration::from_millis(200)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 100);
}

#[tokio::test]
async fn test_error_handling() {
    fn custom_panic_hook(panic_info: &panic::PanicInfo<'_>) {
        let location = panic_info
            .location()
            .unwrap_or_else(|| panic::Location::caller());
        let message = match panic_info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match panic_info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<dyn Any>",
            },
        };
        eprintln!(
            "Panic occurred in file '{}' at line {}: {}",
            location.file(),
            location.line(),
            message
        );
    }

    panic::set_hook(Box::new(custom_panic_hook));

    let runtime = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(runtime);

    emitter
        .on("error_event", |_args: ()| {
            Box::pin(async {
                panic!("This handler should panic");
            })
        })
        .await;

    // This should not panic the whole test
    emitter.emit("error_event", vec![]).await;

    // If we reached here, the test passed
    assert!(true);
}

#[tokio::test]
async fn test_multiple_events() {
    let runtime = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(runtime);

    let counter1 = Arc::new(AtomicI32::new(0));
    let counter2 = Arc::new(AtomicI32::new(0));

    let counter1_clone = counter1.clone();
    let counter2_clone = counter2.clone();

    emitter
        .on("event1", move |_args: ()| {
            let counter = counter1_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        })
        .await;

    emitter
        .on("event2", move |_args: ()| {
            let counter = counter2_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
            })
        })
        .await;

    emitter.emit("event1", vec![]).await;
    emitter.emit("event2", vec![]).await;
    emitter.emit("event1", vec![]).await;

    sleep(Duration::from_millis(50)).await;
    assert_eq!(counter1.load(Ordering::SeqCst), 2);
    assert_eq!(counter2.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_performance() {
    let runtime = Arc::new(TokioRuntime::new());
    let emitter = AsyncEventEmitter::new(runtime);

    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = counter.clone();

    emitter
        .on("perf_event", move |_args: ()| {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::Relaxed);
            })
        })
        .await;

    let start = std::time::Instant::now();
    let futures = (0..10_000).map(|_| emitter.emit("perf_event", vec![]));
    join_all(futures).await;

    let duration = start.elapsed();
    println!("Time taken for 10,000 emits: {:?}", duration);

    sleep(Duration::from_millis(100)).await;
    assert_eq!(counter.load(Ordering::SeqCst), 10_000);
    assert!(
        duration < Duration::from_secs(5),
        "Performance test took too long"
    );
}

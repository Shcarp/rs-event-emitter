use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::*;

#[derive(Clone, Debug, PartialEq)]
struct ComplexData {
    id: usize,
    name: String,
    data: Vec<f64>,
    metadata: HashMap<String, String>,
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

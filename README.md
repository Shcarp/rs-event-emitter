# Rust Event Emitter

A thread-safe, flexible event emitter implementation in Rust.

## Features

- Thread-safe event emission and handling
- Support for multiple event types and handlers
- Typed event arguments
- Asynchronous event processing
- Easy-to-use macros for event emission

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
event_emitter = "2.0.2"  # Replace with the actual version
```

## Usage

Here's a more comprehensive example demonstrating various features of the `EventEmitter`:

```rust
use event_emitter::{EventEmitter, emit};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Create a new EventEmitter
    let emitter = EventEmitter::new();
    
    // Start the event listener in a separate thread
    let listener = emitter.start_listening();

    // Create a shared state to demonstrate thread-safe updates
    let shared_state = Arc::new(Mutex::new(Vec::new()));

    // Handler for a simple greeting event
    emitter.on("greet", |(name,): (String,)| {
        println!("Hello, {}!", name);
    });

    // Handler for an event with multiple arguments
    {
        let state = Arc::clone(&shared_state);
        emitter.on("user_action", move |(user, action, value): (String, String, i32)| {
            println!("User {} performed action: {} with value: {}", user, action, value);
            let mut data = state.lock().unwrap();
            data.push((user, action, value));
        });
    }

    // Handler for a more complex event
    {
        let state = Arc::clone(&shared_state);
        emitter.on("process_data", move |(data, callback): (Vec<i32>, Box<dyn Fn(i32) + Send + 'static>)| {
            let sum: i32 = data.iter().sum();
            callback(sum);
            let mut shared_data = state.lock().unwrap();
            shared_data.push(("System".to_string(), "process_data".to_string(), sum));
        });
    }

    // Emit events
    emit!(emitter, "greet", "Alice".to_string());
    
    emit!(emitter, "user_action", "Bob".to_string(), "click".to_string(), 5);
    
    let callback = Box::new(|result: i32| {
        println!("Processing result: {}", result);
    });
    emit!(emitter, "process_data", vec![1, 2, 3, 4, 5], callback);

    // Demonstrate emitting events from another thread
    let emitter_clone = emitter.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        emit!(emitter_clone, "greet", "Thread".to_string());
    });

    // Wait a bit for all events to be processed
    thread::sleep(Duration::from_millis(200));

    // Print the final state
    let final_state = shared_state.lock().unwrap();
    println!("Final state: {:?}", *final_state);

    // Stop the event listener
    emitter.stop_listening();
    listener.join().unwrap();
}
```

This example demonstrates:

1. Creating and starting an `EventEmitter`
2. Registering handlers for different event types
3. Handling events with multiple arguments
4. Using shared state across event handlers
5. Emitting events with the `emit!` macro
6. Passing callbacks as event arguments
7. Cloning and using the `EventEmitter` in different threads

Remember to handle potential errors and use appropriate synchronization primitives in your actual implementation.

## API

### `EventEmitter`

The main struct for managing events.

#### Methods

- `new()`: Create a new `EventEmitter`.
- `with_thread_pool_size(thread_pool_size: usize)`: Create a new `EventEmitter` with a specified capacity.
- `on<F, Args>(&self, event: &str, handler: F) -> HandlerId`: Register an event handler.
- `off(&self, event: &str, handler_id: HandlerId)`: Remove all handlers for an event.
- `emit(&self, event: &str, args: Vec<ArcAny>)`: Emit an event.
- `start_listening(&self) -> JoinHandle<()>`: Start the event processing loop.
- `stop_listening(&self)`: Stop the event processing loop.
- `clone(&self) -> Self`: Create a clone of the `EventEmitter`.

### Event Parameters

The `EventEmitter` supports up to 16 parameters for each event. This limit is implemented through the `FromArgs` trait and the `impl_from_args!` macro. If you need to pass more than 16 parameters, consider grouping related data into structs or using a Vec or HashMap to pass collections of data.

### Macros

- `emit!`: A convenient macro for emitting events with typed arguments.

## Threading

The `EventEmitter` is designed to be thread-safe. Event handlers are executed in separate threads, allowing for concurrent processing of events.

## Testing

The library includes a comprehensive test suite. Run the tests using:

```
cargo test
```

## License

This project is licensed under [LICENSE NAME]. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
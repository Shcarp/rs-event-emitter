### rs-event-emitter

### Usage

```rs
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
```
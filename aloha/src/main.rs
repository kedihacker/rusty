use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Notify;

struct QueueProcessor<T> {
    sender: Sender<T>,
    start_signal: Arc<Notify>,
}

impl<T: Clone + Send + 'static> QueueProcessor<T> {
    fn new() -> Self {
        let (sender, _) = channel(100);
        Self {
            sender,
            start_signal: Arc::new(Notify::new()),
        }
    }

    fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }

    fn signal_start(&self) {
        self.start_signal.notify_waiters();
    }

    fn spawn_processor<F>(&self, handler: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'static,
    {
        let mut receiver = self.sender.subscribe();
        let signal = Arc::clone(&self.start_signal);

        tokio::spawn(async move {
            signal.notified().await;

            while let Ok(item) = receiver.recv().await {
                handler(item).await;
            }
        })
    }
}

#[tokio::main]
async fn main() {

    let processor: QueueProcessor<u32> = QueueProcessor::new();

    let sender1 = processor.sender();
    let sender2 = processor.sender();

    let producer1 = tokio::spawn(async move {
        for i in 0..5 {
            sender1.send(i).unwrap();
            println!("Producer 1 sent: {}", i);
        }
    });

    let producer2 = tokio::spawn(async move {
        for i in 100..105 {
            sender2.send(i).unwrap();
            println!("Producer 2 sent: {}", i);
        }
    });

    let consumer1 = processor.spawn_processor(|item| {
        Box::pin(async move {
            println!("Consumer 1 processed: {}", item);
        })
    });

    let consumer2 = processor.spawn_processor(|item| {
        Box::pin(async move {
            println!("Consumer 2 processed: {}", item);
        })
    });

    producer1.await.unwrap();
    producer2.await.unwrap();

    println!("Arbitrary condition met (producers done), starting processing...");
    processor.signal_start();

    drop(processor);

    consumer1.await.unwrap();
    consumer2.await.unwrap();

    println!("All done!");
}

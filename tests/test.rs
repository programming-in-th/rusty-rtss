use std::task::Poll;

use futures::channel::mpsc::UnboundedSender;
use futures::prelude::*;
use futures_util::poll;
use rusty_rtss::app::App;

mod mock {
    use std::sync::Mutex;

    use futures::prelude::*;

    use dashmap::DashMap;
    use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
    use futures_util::stream::BoxStream;
    use rusty_rtss::{listener::Listener, publisher::Publisher};

    type Writer = UnboundedSender<Payload>;
    type Reader = UnboundedReceiver<Payload>;

    #[derive(Debug, PartialEq, Eq)]
    pub struct Payload {
        data: [u8; 1024],
    }

    impl Payload {
        pub fn new(data: [u8; 1024]) -> Self {
            Self { data }
        }
    }

    pub struct MockSubscriber {
        writer: Writer,
    }

    impl MockSubscriber {
        pub fn new(writer: Writer) -> Self {
            Self { writer }
        }
    }

    pub struct MockFanoutPublisher {
        writers: DashMap<usize, Writer>,
        size: Mutex<usize>,
    }

    #[async_trait::async_trait]
    impl Publisher for MockFanoutPublisher {
        type PublishData = Payload;
        type Subscriber = MockSubscriber;

        fn add_subscriber(&self, subscriber: Self::Subscriber) {
            let mut lock = self.size.lock().unwrap();
            let value = *lock;
            *lock += 1;
            drop(lock);
            self.writers.insert(value, subscriber.writer);
        }

        async fn publish(&self, data: Self::PublishData) {
            for connection in self.writers.iter() {
                let data = Payload::new(data.data.clone());
                let mut writer = connection.value().clone();
                writer.send(data).await.unwrap();
            }
        }
    }

    impl MockFanoutPublisher {
        pub fn new() -> Self {
            Self {
                writers: DashMap::default(),
                size: Mutex::new(0),
            }
        }
    }

    pub struct MockListener {
        rx: Reader,
    }

    impl Listener for MockListener {
        type Data = Payload;
        type S = BoxStream<'static, Self::Data>;

        fn into_stream(self) -> Self::S {
            Box::pin(self.rx)
        }
    }

    impl MockListener {
        pub fn new(rx: Reader) -> Self {
            Self { rx }
        }
    }
}

fn get_app() -> (
    App<mock::MockFanoutPublisher>,
    UnboundedSender<mock::Payload>,
) {
    let (input, rx) = futures::channel::mpsc::unbounded();
    let listener = mock::MockListener::new(rx);
    let publisher = mock::MockFanoutPublisher::new();

    (
        App::new(listener, publisher).expect("unable to create app"),
        input,
    )
}

#[tokio::test]
async fn test_one_subscriber() {
    let (app, mut input) = get_app();

    let (tx, mut rx) = futures::channel::mpsc::unbounded();
    let subscriber = mock::MockSubscriber::new(tx);

    app.add_subscriber(subscriber).await.unwrap();

    let d = poll!(rx.next());

    assert_eq!(d, Poll::Pending);

    input.send(mock::Payload::new([41; 1024])).await.unwrap();

    // Give execution back to other task.
    tokio::task::yield_now().await;

    let d = poll!(rx.next());

    assert_eq!(d, Poll::Ready(Some(mock::Payload::new([41; 1024]))));
}

#[tokio::test]
async fn test_many_subscriber() {
    let (app, mut input) = get_app();

    let (tx1, mut rx1) = futures::channel::mpsc::unbounded();
    let subscriber1 = mock::MockSubscriber::new(tx1);

    let (tx2, mut rx2) = futures::channel::mpsc::unbounded();
    let subscriber2 = mock::MockSubscriber::new(tx2);

    let (tx3, mut rx3) = futures::channel::mpsc::unbounded();
    let subscriber3 = mock::MockSubscriber::new(tx3);

    app.add_subscriber(subscriber1).await.unwrap();
    app.add_subscriber(subscriber2).await.unwrap();
    app.add_subscriber(subscriber3).await.unwrap();

    let d1 = poll!(rx1.next());
    let d2 = poll!(rx2.next());
    let d3 = poll!(rx3.next());

    assert_eq!(d1, Poll::Pending);
    assert_eq!(d2, Poll::Pending);
    assert_eq!(d3, Poll::Pending);

    input.send(mock::Payload::new([41; 1024])).await.unwrap();

    // Give execution back to other task.
    tokio::task::yield_now().await;

    let d1 = poll!(rx1.next());
    let d2 = poll!(rx2.next());
    let d3 = poll!(rx3.next());

    assert_eq!(d1, Poll::Ready(Some(mock::Payload::new([41; 1024]))));
    assert_eq!(d2, Poll::Ready(Some(mock::Payload::new([41; 1024]))));
    assert_eq!(d3, Poll::Ready(Some(mock::Payload::new([41; 1024]))));
}

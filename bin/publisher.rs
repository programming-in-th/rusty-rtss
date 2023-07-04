use rusty_rtss::sse::SsePublisher;

pub fn get_publisher() -> SsePublisher<super::Identifier, super::Payload> {
    SsePublisher::new()
}

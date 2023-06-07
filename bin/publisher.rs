use rusty_rtss::sse::SsePublisher;

pub fn get_publisher() -> SsePublisher<super::Identifier, super::payload::Payload> {
    SsePublisher::new()
}

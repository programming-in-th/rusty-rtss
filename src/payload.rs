use axum::response::sse::Event;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Payload {
    pub id: i32,
    pub groups: Vec<Group>,
    pub score: i32,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Group {
    score: f64,
    full_score: f64,
    submission_id: String,
    group_index: i32,
    run_result: Vec<RunResult>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RunResult {
    submission_id: String,
    test_index: i32,
    status: String,
    time_usage: f64,
    memory_usage: i32,
    score: f64,
    message: String,
}

impl std::fmt::Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Payload {}\tstatus: `{}`\tscore: {}", self.id, self.status, self.score))
    }
}

impl From<Payload> for Result<Event, serde_json::Error> {
    fn from(value: Payload) -> Self {
        Event::default().json_data(value)
    }
}

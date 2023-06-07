use axum::response::sse::Event;
use rusty_rtss::sse::Identifiable;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{
    postgres::{PgNotification, PgRow},
    types::JsonValue,
    FromRow, Row,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    pub id: i32,
    pub groups: Vec<Group>,
    pub score: i32,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    score: f64,
    full_score: f64,
    submission_id: String,
    group_index: i32,
    run_result: Vec<RunResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunResult {
    submission_id: String,
    test_index: i32,
    status: String,
    time_usage: f64,
    memory_usage: i32,
    score: f64,
    message: String,
}

impl Identifiable for Payload {
    type Identifier = i32;

    fn id(&self) -> Self::Identifier {
        self.id
    }
}

impl From<Payload> for Event {
    fn from(value: Payload) -> Self {
        Event::default()
            .json_data(value)
            .expect("unable to serialize payload")
    }
}

impl From<PgNotification> for Payload {
    fn from(value: PgNotification) -> Self {
        serde_json::from_str(value.payload()).unwrap()
    }
}

impl FromRow<'_, PgRow> for Payload {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Payload {
            id: row.try_get("id")?,
            groups: row.try_get("groups").and_then(json_value_to_vec)?,
            score: row.try_get("score")?,
            status: row.try_get("status")?,
        })
    }
}

fn json_value_to_vec<T>(json: JsonValue) -> std::result::Result<Vec<T>, sqlx::Error>
where
    T: DeserializeOwned,
{
    let vec = match json {
        JsonValue::Array(vec) => vec,
        _ => {
            return Err(sqlx::Error::ColumnNotFound(
                "Json object is not array".to_string(),
            ))
        }
    };

    vec.into_iter()
        .map(serde_json::from_value)
        .collect::<Result<Vec<T>, _>>()
        .map_err(|_| sqlx::Error::ColumnNotFound("Unable to deserialize object".to_string()))
}

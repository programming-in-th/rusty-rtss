use futures_util::{stream::BoxStream, StreamExt};
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};

use crate::cfg::RmqConfig;

pub async fn get_stream(
    rmq_config: RmqConfig,
) -> Result<BoxStream<'static, crate::Payload>, Box<dyn std::error::Error>> {
    let addr = if rmq_config.vhost == "/" {
        format!(
            "amqp://{username}:{password}@{host}:{port}",
            username = rmq_config.username,
            password = rmq_config.password,
            host = rmq_config.host,
            port = rmq_config.port
        )
    } else {
        format!(
            "amqp://{username}:{password}@{host}:{port}/{vhost}",
            username = rmq_config.username,
            password = rmq_config.password,
            host = rmq_config.host,
            port = rmq_config.port,
            vhost = rmq_config.vhost
        )
    };

    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;

    let channel = conn.create_channel().await?;

    let queue_name = format!("submission.update.{}", rmq_config.queue_env);

    let _queue = channel
        .queue_declare(
            &queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            &queue_name,
            "rtss",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    Ok(Box::pin(async_stream::stream! {
        while let Some(Ok(delivery)) = consumer.next().await {
            if let Ok(data) = serde_json::from_slice(&delivery.data) {
                yield data;
            } else {
                tracing::warn!("Fail to deserialize message");
                if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                    tracing::warn!("Fail to ack message: {e}");
                }
            }
        }

        tracing::info!("consumer closed");
    }))
}

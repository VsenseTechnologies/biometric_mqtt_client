mod db;
mod logger;
mod models;
mod mqtt;
mod redis;

use dotenv::dotenv;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::db::db_service::Database;
use crate::mqtt::mqtt_client::MqttClient;
use crate::mqtt::mqtt_handler::MessageHandler;
use crate::redis::redis_service::RedisService;
use logger::init_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //initializing the logger
    init_logger().expect("failed to initialize the logger");

    //loading the enviroment variables from the .env file
    dotenv().ok();

    let (tx, rx) = mpsc::channel::<(String, String)>(100);

    let database = Arc::new(Database::new().await?);
    let redis_service = Arc::new(RedisService::new().await?);

    let (mqtt_client, mqtt_instance) = MqttClient::new(tx).await?;
    let mqtt_client = Arc::new(mqtt_client);

    let mut message_handler = MessageHandler::new(
        rx,
        Arc::clone(&mqtt_client),
        Arc::clone(&database),
        Arc::clone(&redis_service),
    );

    tokio::spawn(async move {
        mqtt_instance.run().await;
    });

    message_handler.run().await;

    Ok(())
}

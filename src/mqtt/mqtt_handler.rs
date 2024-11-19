use log::error;
use rumqttc::{AsyncClient, QoS};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

use crate::db::db_service::Database;
use crate::redis::redis_service::RedisService;

use crate::models::InsertSyncAckRequest;
use crate::models::InsertedNewStudent;
use crate::models::UnitConnectedRequest;
use crate::models::UnitDisconnectedRequest;

#[derive(Debug)]
pub struct MessageHandler {
    rx: Receiver<(String, String)>,
    mqtt_client: Arc<AsyncClient>,
    db: Arc<Database>,
    redis_service: Arc<RedisService>,
}

fn convert_to_data_packete(data: &String) -> &str {
    if data.len() >= 30 {
        return &data[..30];
    }
    return &data[..data.len()];
}

impl MessageHandler {
    pub fn new(
        rx: Receiver<(String, String)>,
        mqtt_client: Arc<AsyncClient>,
        db: Arc<Database>,
        redis_service: Arc<RedisService>,
    ) -> Self {
        Self {
            rx,
            mqtt_client,
            db,
            redis_service,
        }
    }

    pub async fn run(self: &mut Self) {
        while let Some(payload) = self.rx.recv().await {
            let (topic, message) = payload;

            let db = Arc::clone(&self.db);

            let redis_service = Arc::clone(&self.redis_service);

            let mqtt_client = Arc::clone(&self.mqtt_client);

            tokio::task::spawn(async move {
                let topic = topic.as_str();

                let topic_parts: Vec<&str> = topic.split("/").collect();

                let unit_subscribe_topic = topic_parts[0];
                let unit_publish_topic = topic_parts[1];

                match unit_publish_topic {
                    "deletesync" => {
                        if let Ok(json_response) = redis_service
                            .get_user_from_deletes(unit_subscribe_topic.to_string())
                            .await
                        {
                            if let Ok(result) = serde_json::from_str::<Vec<u32>>(&json_response) {
                                if result.len() == 0 {
                                    let mqtt_json_response =
                                        json!({"mty":"1","error_status":"0","ste":"1"});

                                    if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                        let _ = mqtt_client
                                            .publish(
                                                unit_subscribe_topic,
                                                QoS::AtLeastOnce,
                                                false,
                                                payload,
                                            )
                                            .await;
                                    }
                                } else {
                                    let mqtt_json_response =
                                        json!({"mty":"1","est":"0","ste":"0","suid":result[0]});

                                    if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                        let _ = mqtt_client
                                            .publish(
                                                unit_subscribe_topic,
                                                QoS::AtLeastOnce,
                                                false,
                                                payload,
                                            )
                                            .await;
                                    }
                                }
                            } else {
                                let mqtt_json_response = json!({"mty":"1","est":"1","ety":"0"});

                                if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                    let _ = mqtt_client
                                        .publish(
                                            unit_subscribe_topic,
                                            QoS::AtLeastOnce,
                                            false,
                                            payload,
                                        )
                                        .await;
                                }

                                error!("erorr occured while parsing the deletes response from redis -> {}",unit_subscribe_topic);
                            }
                        } else {
                            let mqtt_josn_response = json!({"mty":"1","est":"1","ety":"1"});

                            if let Ok(payload) = serde_json::to_vec(&mqtt_josn_response) {
                                let _ = mqtt_client
                                    .publish(unit_subscribe_topic, QoS::AtLeastOnce, false, payload)
                                    .await;
                            }

                            error!("error occurred with redis while getting the users from deletes -> {}",unit_subscribe_topic);
                        }
                    }

                    "insertsync" => {
                        if let Ok(json_response) = redis_service
                            .get_user_from_inserts(unit_subscribe_topic.to_string())
                            .await
                        {
                            if let Ok(result) =
                                serde_json::from_str::<Vec<InsertedNewStudent>>(&json_response)
                            {
                                if result.len() == 0 {
                                    let mqtt_json_response = json!({"mty":"2","est":"0","ste":"1"});

                                    if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                        let _ = mqtt_client
                                            .publish(
                                                unit_subscribe_topic,
                                                QoS::AtLeastOnce,
                                                false,
                                                payload,
                                            )
                                            .await;
                                    }
                                } else {
                                    if result[0].fingerprint_data.len() == 0 {
                                        if let Err(_) = redis_service
                                            .remove_user_from_inserts(
                                                unit_subscribe_topic.to_string(),
                                            )
                                            .await
                                        {
                                            let mqtt_json_response =
                                                json!({"mty":"2","est":"1","ety":"1"});

                                            if let Ok(payload) =
                                                serde_json::to_vec(&mqtt_json_response)
                                            {
                                                let _ = mqtt_client
                                                    .publish(
                                                        unit_subscribe_topic,
                                                        QoS::AtLeastOnce,
                                                        false,
                                                        payload,
                                                    )
                                                    .await;
                                            }

                                            error!("error occurred with redis while deleting the user from inserts in inserts ack -> {}",unit_subscribe_topic);
                                        }

                                        let mqtt_json_response = json!({"mty":"3","est":"0"});

                                        if let Ok(payload) = serde_json::to_vec(&mqtt_json_response)
                                        {
                                            let _ = mqtt_client
                                                .publish(
                                                    unit_subscribe_topic,
                                                    QoS::AtLeastOnce,
                                                    false,
                                                    payload,
                                                )
                                                .await;
                                        }

                                        return;
                                    }
                                    let fingerprint_data =
                                        convert_to_data_packete(&result[0].fingerprint_data);

                                    let mqtt_json_response = json!({"mty":"2","est":"0","ste":"0","suid":result[0].student_unit_id,"fpd":fingerprint_data});

                                    if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                        let _ = mqtt_client
                                            .publish(
                                                unit_subscribe_topic,
                                                QoS::AtLeastOnce,
                                                false,
                                                payload,
                                            )
                                            .await;
                                    }
                                }
                            } else {
                                let mqtt_json_response = json!({"mty":"2","est":"1","ety":"0"});

                                if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                    let _ = mqtt_client
                                        .publish(
                                            unit_subscribe_topic,
                                            QoS::AtLeastOnce,
                                            false,
                                            payload,
                                        )
                                        .await;
                                }

                                error!("erorr occured while parsing the inserts response from redis -> {}",unit_subscribe_topic);
                            }
                        } else {
                            let mqtt_json_response = json!({"mty":"2","est":"1","ety":"1"});

                            if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                let _ = mqtt_client
                                    .publish(unit_subscribe_topic, QoS::AtLeastOnce, false, payload)
                                    .await;
                            }

                            error!("error occurred with redis while getting the user from inerts -> {}",unit_subscribe_topic);
                        }
                    }

                    "deletesyncack" => {
                        if let Err(_) = redis_service
                            .remove_user_from_deletes(unit_subscribe_topic.to_string())
                            .await
                        {
                            error!("error occurred with redis while deleting the users from deletes -> {}",unit_subscribe_topic);
                        }
                    }

                    "insertsyncack" => {
                        if let Ok(insert_sync_ack) =
                            serde_json::from_str::<InsertSyncAckRequest>(&message)
                        {
                            if let Ok(json_response) = redis_service
                                .get_user_from_inserts(unit_subscribe_topic.to_string())
                                .await
                            {
                                if let Ok(result) =
                                    serde_json::from_str::<Vec<InsertedNewStudent>>(&json_response)
                                {
                                    let fingerprint_data = &result[0].fingerprint_data
                                        [insert_sync_ack.dlen as usize..];

                                    if let Err(err) = redis_service
                                        .update_user_fingerprint_data_in_inserts(
                                            unit_subscribe_topic.to_string(),
                                            fingerprint_data,
                                        )
                                        .await
                                    {
                                        println!("{err:?}");
                                        error!("error occurred with redis while updating the user in inserts ack -> {}",unit_subscribe_topic);
                                    }
                                } else {
                                    error!("error occurred while parsing the inserts response from redis -> {}",unit_subscribe_topic);
                                }
                            } else {
                                error!("error occurred with redis while getting user from redis in inserts ack -> {}",unit_subscribe_topic);
                            }
                        } else {
                            error!(
                                "error occurred while parsing the inserts ack request -> {}",
                                unit_publish_topic
                            )
                        }
                    }

                    "attendence" => {
                        if let Ok(_) = redis_service.insert_attendence_log(message).await {
                            let mqtt_json_response = json!({"mty":"4","est":"0"});

                            if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                let _ = mqtt_client
                                    .publish(unit_subscribe_topic, QoS::AtLeastOnce, false, payload)
                                    .await;
                            }
                        } else {
                            let mqtt_json_response = json!({"mty":"4","est":"1","ety":"0"});

                            if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                let _ = mqtt_client
                                    .publish(unit_subscribe_topic, QoS::AtLeastOnce, false, payload)
                                    .await;
                            }
                        }
                    }

                    "connected" => {
                        if let Ok(unit_connected_request) =
                            serde_json::from_str::<UnitConnectedRequest>(&message)
                        {
                            if let Ok(is_unit_valid) = db
                                .check_unit_is_valid(unit_connected_request.uid.clone())
                                .await
                            {
                                if is_unit_valid {
                                    if let Ok(_) =
                                        db.update_unit_state(unit_connected_request.uid, true).await
                                    {
                                        let mqtt_json_response = json!({"mty":"0","est":"0"});

                                        if let Ok(payload) = serde_json::to_vec(&mqtt_json_response)
                                        {
                                            let _ = mqtt_client
                                                .publish(
                                                    unit_subscribe_topic,
                                                    QoS::AtLeastOnce,
                                                    false,
                                                    payload,
                                                )
                                                .await;
                                        }
                                    } else {
                                        let mqtt_json_response =
                                            json!({"mty":"0","est":"1","ety":"1"});

                                        if let Ok(payload) = serde_json::to_vec(&mqtt_json_response)
                                        {
                                            let _ = mqtt_client
                                                .publish(
                                                    unit_subscribe_topic,
                                                    QoS::AtLeastOnce,
                                                    false,
                                                    payload,
                                                )
                                                .await;
                                        }

                                        error!("error occured with database while updating the unit connected state -> {}",unit_subscribe_topic);
                                    }
                                } else {
                                    let mqtt_json_response = json!({"mty":"0","est":"1","ety":"2"});

                                    if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                        let _ = mqtt_client
                                            .publish(
                                                unit_subscribe_topic,
                                                QoS::AtLeastOnce,
                                                false,
                                                payload,
                                            )
                                            .await;
                                    }

                                    error!(
                                        "connection request from invalid unit -> {}",
                                        unit_subscribe_topic
                                    );
                                }
                            } else {
                                let mqtt_json_response = json!({"mty":"0","est":"1","ety":"1"});

                                if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                    let _ = mqtt_client
                                        .publish(
                                            unit_subscribe_topic,
                                            QoS::AtLeastOnce,
                                            false,
                                            payload,
                                        )
                                        .await;
                                }

                                error!("error occured with database while checking if unit is valid or not -> {}",unit_subscribe_topic);
                            }
                        } else {
                            let mqtt_json_response = json!({"mty":"0","est":"1","ety":"0"});

                            if let Ok(payload) = serde_json::to_vec(&mqtt_json_response) {
                                let _ = mqtt_client
                                    .publish(unit_subscribe_topic, QoS::AtLeastOnce, false, payload)
                                    .await;
                            }

                            error!("failed to parse the client message request to unit connected request -> {}",unit_subscribe_topic);
                        }
                    }

                    "disconnected" => {
                        if let Ok(unit_disconnected_request) =
                            serde_json::from_str::<UnitDisconnectedRequest>(&message)
                        {
                            if let Ok(is_unit_valid) = db
                                .check_unit_is_valid(unit_disconnected_request.uid.clone())
                                .await
                            {
                                if is_unit_valid {
                                    if let Ok(_) = db
                                        .update_unit_state(unit_disconnected_request.uid, false)
                                        .await
                                    {
                                        error!(
                                            "unit disconnected from the mqtt broker -> {}",
                                            unit_subscribe_topic
                                        );
                                    } else {
                                        error!("error occured with the database while updating the unit disconnected state -> {}",unit_subscribe_topic);
                                    }
                                } else {
                                    error!(
                                        "disconnection request from the invalid unit -> {}",
                                        unit_subscribe_topic
                                    );
                                }
                            } else {
                                error!("error occured with database while checking if the unit is valid or not -> {}",unit_subscribe_topic);
                            }
                        } else {
                            error!("failed to parse the client message request to unit disconnected request -> {}",unit_subscribe_topic);
                        }
                    }
                    _ => (),
                }
            });
        }
    }
}

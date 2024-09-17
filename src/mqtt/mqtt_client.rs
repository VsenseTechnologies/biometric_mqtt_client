use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, Packet, QoS};
use std::env;
use std::time::Duration;
use tokio::sync::mpsc::Sender;


pub struct MqttClient {
    pub tx:Sender<(String,String)>,
    pub event_loop:EventLoop
}

impl MqttClient {
    pub async fn new(tx:Sender<(String,String)>) -> Result<(AsyncClient,Self),rumqttc::ClientError> {
        //loading the values from environment variables
        let client_id=env::var("MQTT_CLIENT_ID").expect("failed to read the MQTT_CLIENT env variable");
        let host=env::var("MQTT_HOST").expect("failed to read the MQTT_HOST env variable");
        let port=env::var("MQTT_PORT").expect("failed to read the MQTT_PORT env variable");

        //parsing the port variable
        let port:u16=port.trim().parse().expect("failed to parse port variable");


        let mut mqtt_options=MqttOptions::new(client_id, host, port);
        mqtt_options.set_keep_alive(Duration::from_secs(10));

        let (client,event_loop)=AsyncClient::new(mqtt_options, 10);

        client.subscribe("+/attendence", QoS::AtLeastOnce).await?;
        client.subscribe("+/connected", QoS::AtLeastOnce).await?;
        client.subscribe("+/disconnected",QoS::AtLeastOnce).await?;
        client.subscribe("+/deletesync", QoS::AtLeastOnce).await?;
        client.subscribe("+/insertsync",QoS::AtLeastOnce).await?;
       
        client.subscribe("+/deletesyncack", QoS::AtLeastOnce).await?;
        client.subscribe("+/insertsyncack", QoS::AtLeastOnce).await?;

        Ok((client,Self {
            tx,
            event_loop
        }))
    }

    pub async fn run(mut self:Self) {
        loop {
            if let Ok(event)=self.event_loop.poll().await {
                match event {
                    Event::Incoming(Incoming::ConnAck(_)) => {
                        println!("connected to mqtt broker");
                    },
                    Event::Incoming(Packet::Publish(publish)) => {
                        let topic=publish.topic;
                        if let Ok(payload)=String::from_utf8(publish.payload.to_vec()) {
                            self.tx.send((topic,payload)).await.unwrap();
                        }
                    }
                    _=>()
                }
            }else {
                println!("disconnected from the mqtt broker");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }
}
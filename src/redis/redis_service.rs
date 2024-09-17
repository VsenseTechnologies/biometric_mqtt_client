use redis::{Client,RedisError};
use redis::aio::MultiplexedConnection;
use std::env;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct RedisService {
    pub attendence_list_name:String,
    pub insert_json_name:String,
    pub delete_json_name:String,
    pub client:Mutex<MultiplexedConnection>
}

impl RedisService {
    pub async fn new() -> Result<Self,RedisError> {
        
        let redis_uri=env::var("REDIS_URI").expect("failed to read the REDIS_URI env variable");

        let attendence_list_name=env::var("REDIS_ATTENDENCE_LIST_NAME").expect("failed to read the REDIS_ATTENDENCE_LIST_NAME");

        let insert_json_name=env::var("REDIS_INSERT_JSON_NAME").expect("failed to read the REDIS_INSERT_JSON_NAME");

        let delete_json_name=env::var("REDIS_DELETE_JSON_NAME").expect("failed to read the REDIS_DELETE_JSON_NAME");

        let client=Client::open(redis_uri).expect("invalid redis uri");

        let mut connecton=client.get_multiplexed_async_connection().await?;

        redis::cmd("PING").exec_async(&mut connecton).await?;

        println!("connected to redis");

        Ok(Self {
            attendence_list_name,
            insert_json_name,
            delete_json_name,
            client:Mutex::new(connecton)
        })
    }
    
    pub async fn get_user_from_deletes(self:&Self,unit_id:String)-> Result<String, redis::RedisError>{

        let json_name=&self.delete_json_name;
        let command="JSON.GET";
        let json_arr_path=format!("$.{}[0]",unit_id.to_uppercase());
        let arguements=[json_name,&json_arr_path];

        let mut redis_client=self.client.lock().await;

        let json_response=redis::cmd(command).arg(&arguements).query_async::<String>(&mut *redis_client).await?;

        Ok(json_response)        
    }

    pub async fn get_user_from_inserts(self:&Self,unit_id:String) -> Result<String,redis::RedisError> {
        
        let json_name=&self.insert_json_name;

        let command="JSON.GET";
        
        let json_arr_path=format!("$.{}[0]",unit_id.to_uppercase());

        let arguements=[json_name,&json_arr_path];

        let mut redis_client=self.client.lock().await;

        let json_response=redis::cmd(command).arg(&arguements).query_async::<String>(&mut *redis_client).await?;

        Ok(json_response)    
    }

    pub async fn remove_user_from_inserts(self:&Self,unit_id:String) -> Result<(),redis::RedisError> {
        
        let json_name=&self.insert_json_name;

        let command="JSON.DEL";

        let json_arr_path=format!("$.{}[0]",unit_id.to_uppercase());

        let arguements=[json_name,&json_arr_path];

        let mut redis_client=self.client.lock().await;

        redis::cmd(&command).arg(&arguements).exec_async(&mut *redis_client).await?;

        Ok(())
    }

    pub async fn remove_user_from_deletes(self:&Self,unit_id:String) -> Result<(),redis::RedisError> {
        
        let json_name=&self.delete_json_name;

        let command="JSON.DEL";

        let json_arr_path=format!("$.{}[0]",unit_id.to_uppercase());

        let arguements=[json_name,&json_arr_path];

        let mut redis_client=self.client.lock().await;

        redis::cmd(command).arg(&arguements).exec_async(&mut *redis_client).await?;

        Ok(())
    }

    pub async fn insert_attendence_log(self:&Self,log:String)->Result<(),RedisError> {

        let list_name=&self.attendence_list_name;

        let command="LPUSH";

        let arguments=[list_name,&log];

        let mut redis_client=self.client.lock().await;

        redis::cmd(command).arg(&arguments).exec_async(&mut *redis_client).await?;

        Ok(())
    }

}
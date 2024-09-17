use serde::Deserialize;


#[derive(Debug,Deserialize)]
pub struct UnitConnectedRequest {
    pub unit_id:String
}

#[derive(Debug,Deserialize)]
pub struct UnitDisconnectedRequest {
    pub unit_id:String
}


#[derive(Debug,Deserialize)]
pub struct InsertedNewStudent {
    pub student_unit_id:String,
    pub fingerprint_data:String
}
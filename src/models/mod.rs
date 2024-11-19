use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UnitConnectedRequest {
    pub uid: String,
}

#[derive(Debug, Deserialize)]
pub struct UnitDisconnectedRequest {
    pub uid: String,
}

#[derive(Debug, Deserialize)]
pub struct InsertSyncAckRequest {
    pub dlen: i32,
}

#[derive(Debug, Deserialize)]
pub struct InsertedNewStudent {
    pub student_unit_id: String,
    pub fingerprint_data: String,
}

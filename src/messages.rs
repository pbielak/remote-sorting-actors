/*
Messages used in system
*/

/// Supervisor -> SortingActor request
#[derive(Serialize, Deserialize, Debug, Message)]
pub struct SortingRequest {
    pub values: Vec<i64>,
}

impl SortingRequest {
    pub fn new(values: Vec<i64>) -> SortingRequest {
        SortingRequest{
            values
        }
    }
}


/// SortingActor -> Supervisor response
#[derive(Serialize, Deserialize, Debug, Message)]
pub struct SortingResponse {
    pub values: Vec<i64>,
    pub duration: i64
}

impl SortingResponse {
    pub fn new(values: Vec<i64>, duration: i64) -> SortingResponse {
        SortingResponse{
            values,
            duration
        }
    }
}

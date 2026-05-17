use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonBuffer(pub Vec<u8>);

uniffi::custom_newtype!(JsonBuffer, Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZenEngineTrace {
    pub id: String,
    pub value: JsonBuffer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZenEngineResponse {
    pub performance: String,
    pub result: JsonBuffer,
    pub trace: Option<HashMap<String, ZenEngineTrace>>,
}

pub fn get_zen_engine_response() -> ZenEngineResponse {
    ZenEngineResponse {
        performance: "ready".to_string(),
        result: JsonBuffer(vec![1, 2, 3]),
        trace: Some(HashMap::from([(
            "primary".to_string(),
            ZenEngineTrace {
                id: "primary".to_string(),
                value: JsonBuffer(vec![4, 5, 6]),
            },
        )])),
    }
}

pub fn return_zen_engine_response(response: ZenEngineResponse) -> ZenEngineResponse {
    response
}

uniffi::include_scaffolding!("api");

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct AddResponse {
    pub result: bool
}

#[derive(Deserialize)]
pub struct RemoveResponse {
    pub result: bool
}

#[derive(Deserialize)]
pub struct ModifyResponse {
    pub result: bool
}

#[derive(Deserialize)]
pub struct SetProxyEnabledResponse {
    pub result: bool
}

#[derive(Deserialize)]
pub struct IsProxyEnabledResponse {
    pub result: bool
}
use kube::api::DynamicObject;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct WatchEvent{
    pub resource_url:  String,
    pub event_type: String,
    pub dynamic_object: Option<DynamicObject>,
}

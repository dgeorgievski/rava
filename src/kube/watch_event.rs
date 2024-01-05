use kube::api::DynamicObject;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct WatchEvent{
    pub resource_url:  String,
    pub dynamic_object: Option<DynamicObject>,
}

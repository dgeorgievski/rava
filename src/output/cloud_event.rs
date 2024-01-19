use kube::core::{TypeMeta, DynamicObject};
use kube::api::ResourceExt;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use cloudevents::binding::reqwest::RequestBuilderExt;
use cloudevents::{EventBuilder, EventBuilderV10};
use uuid::Uuid;
use actix_web::http::StatusCode;
use crate::kube::WatchEvent;
use crate::output::parse_type_meta;
use crate::output::utils;
use crate::configuration::Settings;


pub async fn setup_cloud_event_output(conf: &Settings, 
                        rx_we: Receiver<WatchEvent>) -> Result<bool, regex::Error> {
    let (tx_api, rx_api): (Sender<String>, Receiver<String>) = channel(32);
    let (tx_type, rx_type): (Sender<Option<TypeMeta>>, Receiver<Option<TypeMeta>>) = channel(32);

    //todo improve error handling and passing
    let result = match parse_type_meta(rx_api, tx_type).await {
        Ok(_) => {
            let _result = http_post_ce(conf.nats.proxy_url.clone(), 
                                        rx_we, tx_api, 
                                        rx_type).await;
            true
        },
        Err(why) => {
            tracing::error!("CloudEvent output setup failed {:?}", why);
            false
        },
    };

    Ok(result)
}

pub async fn http_post_ce(target: String,
    mut rx_we: Receiver<WatchEvent>, 
    tx_api: Sender<String>,
    mut rx_type: Receiver<Option<TypeMeta>>) -> std::io::Result<()> {
    
    tokio::spawn(async move { 
        println!("{0:<20} {1:<20} {2:<20} {3:<width$}", "KIND", "NAMESPACE", "AGE", "NAME", width = 63);
        while let Some(we) = rx_we.recv().await {
            let res_url = we.resource_url;
            let Some(mut obj) = we.dynamic_object else { continue };
            let age = utils::format_creation_since(obj.creation_timestamp());
            let ns = obj.namespace().unwrap_or("".to_owned());
            let tm_kind: String;
            
            if let Some(type_meta) = &obj.types {
                tm_kind = type_meta.kind.clone();
            }else{
                //todo parse k8s API to create the TypeMeta struct.
                // println!("\nMissing type_meta for {:?}", res_url);
                match tx_api.send(res_url).await{
                    Err(why) => {
                        tracing::error!("Failed sending k8s api URL: {:?}", why);
                        continue;
                    },
                    Ok(_) => { 
                        if let Some(res) = rx_type.recv().await {
                            if let Some(tm) = res {
                                tm_kind = tm.kind.clone();
                                // Add missing TypeMeta
                                obj = DynamicObject {
                                    types: Some(tm),
                                    metadata: obj.metadata,
                                    data: obj.data,
                                };
                            }else{
                                // TypeMeta extraction error
                                continue;
                            }
                        }else{
                            //channel error
                            continue;
                        };
                    },
                };
            };

            // convert DynamicObject to JSON string
            let obj_json = match serde_json::to_string(&obj) {
                                Ok(j) => j,
                                Err(_) => String::from("{}"),
                            };

            // println!("\n{:?}\n", obj_json);
            let res_ce = match EventBuilderV10::new()
                        .id(&Uuid::new_v4().hyphenated().to_string())
                        .ty("healthcat.event.v1")
                        .source("http://localhost")
                        .data("application/json", obj_json)
                        .build() 
            {
                Ok(event) => {
                    match reqwest::Client::new()
                                .post(&target)
                                .event(event)
                    {
                        Ok(request) => {
                            match request
                                            .header("Access-Control-Allow-Origin", "*")
                                            .send()
                                            .await {
                                Ok(res) => {
                                    res.status()
                                },
                                Err(why) => {
                                    println!("Post event failed {:?}", why);
                                    if let Some(st) = why.status() {
                                        st
                                    }else{
                                        StatusCode::INTERNAL_SERVER_ERROR
                                    }
                                },
                            }
                        },
                        Err(why) => {
                            println!("Building event failed {:?}", why);
                            StatusCode::BAD_REQUEST
                        },
                    }
                },
                Err(why) => {
                    println!("Failed building CloudEvent {:?}", why);
                    continue;
                },
            };

            println!("{0:<20} [{1:<20}] {2:<20} {3:<20} {4:<width$}", 
                                                tm_kind, 
                                                res_ce,
                                                ns, 
                                                age, 
                                                obj.name_any(), 
                                                width = 80);
        }
    });
    
    Ok(())
}

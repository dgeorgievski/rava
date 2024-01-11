pub mod cloud_event;
mod utils;

use regex::Regex;
// use serde::Serialize;
// use serde_json::Result as serde_Result;
use crate::kube::WatchEvent;
// use kube::api::{DynamicObject, ResourceExt};
use kube::core::{TypeMeta, DynamicObject};
use kube::api::ResourceExt;
use tokio::sync::mpsc::{Sender, Receiver, channel};

pub async fn setup_output(rx_we: Receiver<WatchEvent>) -> Result<bool, regex::Error> {
    let (tx_api, rx_api): (Sender<String>, Receiver<String>) = channel(32);
    let (tx_type, rx_type): (Sender<Option<TypeMeta>>, Receiver<Option<TypeMeta>>) = channel(32);

    //todo improve error handling and passing
    let result = match parse_type_meta(rx_api, tx_type).await {
        Ok(_) => {
            let _result = simple_print_process(rx_we, tx_api, rx_type).await;
            true
        },
        Err(why) => {
            tracing::error!("output setup failed {:?}", why);
            false
        },
    };

    Ok(result)
}

pub async fn simple_print_process(mut rx_we: Receiver<WatchEvent>, 
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

            println!("{0:<20} {1:<20} {2:<20} {3:<width$}", 
                                                        tm_kind, 
                                                        ns, 
                                                        age, 
                                                        obj.name_any(), 
                                                        width = 63);

            // convert DynamicObject to JSON string
            match serde_json::to_string(&obj) {
                Ok(_j) => {
                    // println!("\n{:?}\n", j);
                    continue;
                },
                Err(why) => {
                    println!("Err converting to JSON {:?}", why);
                },
            }
        }
    });
    
    Ok(())
}

// Receives a k8s API path and returns a TypeMeta structure.
// rx(/api/v1/events) -> parse -> sn(TypeMeta{ api_version: v1, kind: Event}) 
// todo add caching for a given path to avoid repeated regex matching.
pub async fn parse_type_meta(mut rx: Receiver<String>, 
                                tx: Sender<Option<TypeMeta>>) -> Result<bool, regex::Error> {
    let k8s_api_pattern = vec![
        r"/api/(?<ver>[a-z0-9]*)/(?<resource>[a-zA-Z0-9-]*)s$",
        r"/api/(?<ver>[a-z0-9]*)/namespaces/(?<ns>[a-zA-Z0-9-]*)/(?<resource>[a-zA-Z0-9]*)s$",
        r"/apis/(?<apigroup>[a-z0-9\.]*)/(?<ver>[a-z0-9]*)/(?<resource>[a-zA-Z0-9]*)s$",
        r"/apis/(?<apigroup>[a-z0-9\.]*)/(?<ver>[a-z0-9]*)/namespaces/(?<ns>[a-zA-Z0-9-]*)/(?<resource>[a-zA-Z0-9]*)s$",
    ];

    let mut k8s_api_rex: Vec<Regex> = Vec::new();

    for p in k8s_api_pattern {
        match Regex::new(p) {
            Ok(r) => {
                k8s_api_rex.push(r);
            },
            Err(err) => { 
                return Err(err)
            },
        }
    }

    tokio::spawn(async move {
        while let Some(hay) = rx.recv().await {
            let mut result: Option<TypeMeta> = None;

            'k8sapi: for r in &k8s_api_rex {
                if let Some(caps) = r.captures(&hay) {
                    result = Some(TypeMeta{
                                api_version: caps["ver"].to_string(),
                                kind: utils::capitalize(&caps["resource"]),
                            });          
                    // skip the remaining patterns
                    break 'k8sapi;
                };
            };

            if let Err(why) = tx.send(result).await{
                tracing::error!("Failed to send TypeMeta: {:?}", why);
            };
        };
    });

    Ok(true)
}
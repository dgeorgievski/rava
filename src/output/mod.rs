mod cloud_event;
mod utils;

use regex::{Regex, Error};
use crate::kube::WatchEvent;
// use kube::api::{DynamicObject, ResourceExt};
use kube::core::TypeMeta;
use kube::api::ResourceExt;
use tokio::sync::mpsc::{Sender, Receiver};

pub async fn simple_print_process(mut rx: Receiver<WatchEvent>) -> std::io::Result<()> {
    tokio::spawn(async move { 
        println!("{0:<20} {1:<20} {2:<20} {3:<width$}", "KIND", "NAMESPACE", "AGE", "NAME", width = 63);
        while let Some(we) = rx.recv().await {
            let res_url = we.resource_url;
            if let Some(obj) = we.dynamic_object {
                let age = utils::format_creation_since(obj.creation_timestamp());
                let ns = obj.namespace().unwrap_or("".to_owned());
                let mut tm_kind = String::from("");
                if let Some(type_meta) = &obj.types {
                    tm_kind = type_meta.kind.clone();
                }else{
                    println!("\nMissing type_meta for {:?}", res_url)
                }

                println!("{0:<20} {1:<20} {2:<20} {3:<width$}", tm_kind, ns, age, obj.name_any(), width = 63);
            }
        }
    });
    
    Ok(())
}

// rx(/api/v1/events) -> parse -> sn(TypeMeta{ api_version: v1, kind: Event}) 
pub async fn parse_type_meta(mut rx: Receiver<String>, tx: Sender<Option<TypeMeta>>) -> Result<bool, Error> {
    let re_core: Regex;
    let re_core_ns: Regex;
    let re_apis: Regex; 
    let re_apis_ns: Regex;

    // compile RegEx only once
    match Regex::new(r"/api/(?<ver>[a-z0-9]*)/(?<resource>[a-zA-Z0-9-]*)s\s+") {
        Ok(ex) => {re_core = ex},
        Err(err) => { return Err(err)},
    };

    match Regex::new(r"/api/(?<ver>[a-z0-9]*)/namespaces/(?<ns>[a-zA-Z0-9-]*)/(?<resource>[a-zA-Z0-9]*)s\s+") {
        Ok(ex) => {re_core_ns = ex},
        Err(err) => { return Err(err)},
    };

    match Regex::new(r"/apis/(?<apigroup>[a-z0-9\.]*)/(?<ver>[a-z0-9]*)/(?<resource>[a-zA-Z0-9]*)s\s+"){
        Ok(ex) => {re_apis = ex},
        Err(err) => { return Err(err)},
    };
    
    match Regex::new(r"/apis/(?<apigroup>[a-z0-9\.]*)/(?<ver>[a-z0-9]*)/namespaces/(?<ns>[a-zA-Z0-9-]*)/(?<resource>[a-zA-Z0-9]*)s\s+"){
        Ok(ex) => {re_apis_ns = ex},
        Err(err) => { return Err(err)},
    };

    tokio::spawn(async move {
        while let Some(hay) = rx.recv().await {
            let mut result = if let Some(caps) = re_core.captures(&hay) {
                    // println!("/api/{}/{}", &caps["ver"], &caps["resource"]);
                    Some(TypeMeta{
                        api_version: caps["ver"].to_string(),
                        kind: utils::capitalize(&caps["resource"]),
                    })
                }else{
                    None
                };
        
            result = if let Some(caps) = re_core_ns.captures(&hay) {
                        // println!("/api/{}/namespaces/{}/{}", &caps["ver"], 
                        //                                     &caps["ns"], 
                        //                                     &caps["resource"]);
                        Some(TypeMeta{
                            api_version: caps["ver"].to_string(),
                            kind: utils::capitalize(&caps["resource"]),
                        })
                    }else { 
                        None
                    };
            
            result = if let Some(caps) = re_apis.captures(&hay) {
                        // println!("/api/{}/{}/{}", &caps["apigroup"], 
                        //                             &caps["ver"], 
                        //                             &caps["resource"]);
                            Some(TypeMeta{
                                api_version: format!("{}/{}", &caps["apigroup"],
                                                            &caps["ver"]),
                                kind: utils::capitalize(&caps["resource"]),
                            })
                        }else { 
                            None
                        };
        
            result = if let Some(caps) = re_apis_ns.captures(&hay) {
                        // println!("/api/{}/{}/namespaces/{}/{}", &caps["apigroup"], 
                        //                             &caps["ver"], 
                        //                             &caps["ns"], 
                        //                             &caps["resource"]);
                            Some(TypeMeta{
                                api_version: format!("{}/{}", &caps["apigroup"],
                                                        &caps["ver"]),
                                kind: utils::capitalize(&caps["resource"]),
                            })
                        }else { 
                            None
                        };

            match tx.send(result).await{
                Err(why) => {
                    tracing::error!("Failed to send TypeMeta: {:?}", why);
                },
                _ => { continue },
            };
        };
    });

    Ok(true)
}
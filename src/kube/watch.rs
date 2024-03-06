use crate::kube::{client, discovery, WatchEvent};

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};

// use kube::{
//     api::{DynamicObject, ListParams},
//     discovery::Scope,
//     runtime::{watcher, WatchStreamExt},
// };

use kube::core::DynamicObject;
use kube::runtime::{watcher, WatchStreamExt};

use kube::ResourceExt;
use tokio::sync::mpsc::{channel, Receiver, Sender};
// use tracing::field;
use crate::configuration::Settings;
use crate::kube::tekton_metrics::{PodMetrics, poll_pod_metrics};

pub async fn watch(conf: &Settings) -> Result<Receiver<WatchEvent>> {
    let (tx, rx): (Sender<WatchEvent>, Receiver<WatchEvent>) = channel(32);

    let cli = match client::client(conf.kube.use_tls).await {
        Err(why) => {
            tracing::error!("k8s Client failed {:?}", why);
            return Err(why.into())
        }
        Ok(cli) => {
            tracing::info!("Succesfully connected to k8s");
            cli
        }
    };

    let discovery = discovery::new(&cli).await?;
    // Common discovery, parameters, and api configuration for a single resource
    let api_res = discovery::resolve_api_resources( 
                        &discovery, 
                        &conf.kube.resources);

    // Open PodMetrics channel
    // let tx_pm: Option<Sender<PodMetrics>> = match get_pod_metrics(&conf).await{
        let tx_pm: Option<Sender<PodMetrics>> = match poll_pod_metrics(&conf).await{    
        Ok(tx) => {
            tracing::info!("PodMetrics channel OK");
            Some(tx)
        },
        Err(why) => {
            tracing::error!("Failed setting PodMetrics channel {}", why);
            None
        },
    };

    for (ares, caps) in api_res {
        println!("\n\n ApiRes {:?}\n CAP: {:?}", ares, caps);
        
        let dyn_apis = discovery::dynamic_api(
                                            ares, 
                                            caps,
                                            cli.clone(), 
                                            &conf.kube.resources);


        for apisel in dyn_apis { 
            let tx2 = tx.clone();
            let resource_url: String = apisel.api_dyn.resource_url().to_owned();
            // dbg!(&apisel);

            let tx_pm2 = tx_pm.clone();
            // start watching API Resource in a dedicated thread
            tokio::spawn(async move {
                let mut wc = watcher::Config::default();
                if let Some(sel) = apisel.field_selectors {
                    if sel.len() > 0 {
                        wc.field_selector = Some(sel.join(","));
                        println!("Added field selectors {:?} url: {}", 
                            wc.field_selector, 
                            resource_url);
                    }
                }

                if let Some(sel) = apisel.label_selectors {
                    if sel.len() > 0 {
                        wc.label_selector = Some(sel.join(","));
                        println!("Added label selectors {:?} url: {}", 
                            wc.label_selector, 
                            resource_url);
                    }
                }


                let mut stream = watcher(apisel.api_dyn, wc).
                                        applied_objects().
                                        boxed();
                loop {
                    let obj = match stream.try_next().await {
                        Ok(obj) => Some(obj.unwrap()),
                        Err(_error) => {
                            // tracing::error!("failed to get stream response: {:?}", error);
                            None
                        }
                    };

                    // TaskRun? Check its PodMetrics
                    if let Some(tx_pm3) = &tx_pm2 {
                        if let Some(dyn_obj) = &obj {
                            if let Some(type_meta) = &dyn_obj.types {
                                if type_meta.kind == "TaskRun" {
                                    if let Some(ns) = &dyn_obj.namespace(){
                                        let status= get_task_run_data(&dyn_obj);
                                        
                                        tx_pm3.send(PodMetrics{
                                            name: dyn_obj.name_any(),
                                            namespace: ns.to_string(),
                                            status: status.trim_matches('"').to_string(),
                                        }).await.unwrap();
                                    }
                                }
                            }
                        };
                    }

                    let we = WatchEvent{
                        resource_url: resource_url.clone(),
                        event_type: apisel.event_type.clone(),
                        dynamic_object: obj,
                    };

                    tx2.send(we).await.unwrap();
                }
            });
        }
    }

    return Ok(rx);
}

// Returns TaskRun reason of status condition and name of k8s node
fn get_task_run_data(dynobj: &DynamicObject) -> String {
    let mut ret_status = "Unknown".to_string();

    if let Some(status) = dynobj.data.get("status") {
        if let Some(cond) = status.get("conditions") {
            if cond.is_array() {
                if let Some(reason) = cond[0].get("reason") {
                    ret_status = reason.to_string();
                }
            }
        }
    }

    ret_status
}
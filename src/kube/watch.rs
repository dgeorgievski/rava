use crate::kube::{client, discovery, WatchEvent};

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};

// use kube::{
//     api::{DynamicObject, ListParams},
//     discovery::Scope,
//     runtime::{watcher, WatchStreamExt},
// };

use kube::runtime::{watcher, WatchStreamExt};

use tokio::sync::mpsc::{channel, Receiver, Sender};
// use tracing::field;
use crate::configuration::Settings;

pub async fn watch(conf: &Settings) -> Result<Receiver<WatchEvent>> {
    tracing::debug!("resources {:?}", conf.kube.resources);
    
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
    

    let (tx, rx): (Sender<WatchEvent>, Receiver<WatchEvent>) = channel(32);

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

            tokio::spawn(async move {
                let mut wc = watcher::Config::default();
                if let Some(sel) = apisel.field_selectors {
                    if sel.len() > 0 {
                        wc.field_selector = Some(sel.join(","));
                        println!("Added field selectors {:?} url: {}", wc.field_selector, resource_url);
                    }
                }

                if let Some(sel) = apisel.label_selectors {
                    if sel.len() > 0 {
                        wc.label_selector = Some(sel.join(","));
                        println!("Added label selectors {:?} url: {}", wc.label_selector, resource_url);
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
                    let we = WatchEvent{
                        resource_url: resource_url.clone(),
                        dynamic_object: obj,
                    };

                    tx2.send(we).await.unwrap();
                }
            });
        }
    }

    return Ok(rx);
}

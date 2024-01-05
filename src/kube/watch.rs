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
    tracing::debug!("resources {:?}, namespaces {:?}", conf.kube.resources, conf.kube.namespaces);
    
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
    
     // TODO: add label filtering
    // let mut lp = ListParams::default();
    // if let Some(label) = app.selector.clone() {
    //     lp = lp.labels(label.as_str());
    // }

    // if let Some(name) = app.name.clone() {
    //     lp = lp.fields(&format!("metadata.name={}", name));
    // }

    // TODO: add labels later to watcher
    // let wc = watcher::Config::default();
    //  .labels("kubernetes.io/lifecycle=spot");

    // let (tx, rx): (Sender<DynamicObject>, Receiver<DynamicObject>) = channel(32);
    let (tx, rx): (Sender<WatchEvent>, Receiver<WatchEvent>) = channel(32);

    for (ares, caps) in api_res {
        println!("\n\n ApiRes {:?}\n CAP: {:?}", ares, caps);

        let mut field_event_selector = "";
        if ares.kind.clone().as_str() == "Event" {
            field_event_selector = "type!=Normal";
        }
        
        let dyn_apis = discovery::dynamic_api(
            ares, 
            caps,
            cli.clone(), 
            &conf.kube.namespaces);

        for api in dyn_apis {
            let tx2 = tx.clone();
            let resource_url: String = api.resource_url().to_owned();
            // dbg!(&api);

            tokio::spawn(async move {
                // present a dumb table for it for now. kubectl does not do this anymore.
                let mut wc = watcher::Config::default();
                if field_event_selector.len() > 0 {
                    wc.field_selector = Some(field_event_selector.into());
                }

                let mut stream = watcher(api, wc).
                                        applied_objects().
                                        boxed();
                loop {
                    let obj = match stream.try_next().await {
                        Ok(obj) => Some(obj.unwrap()),
                        Err(error) => {
                            tracing::error!("failed to get stream response: {:?}", error);
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

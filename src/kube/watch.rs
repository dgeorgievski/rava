use crate::kube::{client, discovery};

use anyhow::{Context, Result};
// use futures::{StreamExt, TryStreamExt};

// use kube::{
//     api::{DynamicObject, ListParams},
//     discovery::Scope,
//     runtime::{watcher, WatchStreamExt},
// };

// use tokio::sync::mpsc::{channel, Receiver, Sender};
use crate::configuration::Settings;

pub async fn watch(conf: &Settings) -> Result<()> {
    tracing::info!("watching");
    let cli = client::client(conf.kube.use_tls).await?;
    let discovery = discovery::new(&cli).await?;
    
    // TODO: make resources configurable and possible to watch them 
    let resources = conf.kube.resources.join(",");

    tracing::info!("resources {:?}, namespaces {:?}", conf.kube.resources, conf.kube.namespaces);
    
    // Common discovery, parameters, and api configuration for a single resource
    let (ar, caps) = discovery::resolve_api_resource(
                            &discovery, 
                            &conf.kube.resources)
                        .with_context(|| 
                            format!("resource {:?} not found in cluster", resources))?;

    tracing::info!("watching 2");
    // TODO: add label filtering
    // let mut lp = ListParams::default();
    // if let Some(label) = app.selector.clone() {
    //     lp = lp.labels(label.as_str());
    // }

    // if let Some(name) = app.name.clone() {
    //     lp = lp.fields(&format!("metadata.name={}", name));
    // }
    let _api = discovery::dynamic_api(
                        ar, 
                        caps, 
                        cli, 
                        &conf.kube.namespaces);

    

    // let (tx, rx): (Sender<DynamicObject>, Receiver<DynamicObject>) = channel(32);

    // tokio::spawn(async move {
    //     // present a dumb table for it for now. kubectl does not do this anymore.
    //     let mut stream = watcher(api, lp).applied_objects().boxed();
    //     loop {
    //         let obj = match stream.try_next().await {
    //             Ok(obj) => obj.unwrap(),
    //             Err(error) => {
    //                 panic!("failed to get stream response: {:?}", error)
    //             }
    //         };
    //         tx.send(obj).await.unwrap();
    //     }
    // });

    return Ok(());
}

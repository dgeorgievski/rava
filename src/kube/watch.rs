use crate::kube::{client, discovery};

use anyhow::Result;
// use futures::{StreamExt, TryStreamExt};

// use kube::{
//     api::{DynamicObject, ListParams},
//     discovery::Scope,
//     runtime::{watcher, WatchStreamExt},
// };

// use tokio::sync::mpsc::{channel, Receiver, Sender};
use crate::configuration::Settings;

pub async fn watch(conf: &Settings) -> Result<()> {
    tracing::info!("resources {:?}, namespaces {:?}", conf.kube.resources, conf.kube.namespaces);
    
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

    for (res, caps) in api_res {
        println!("\n\n ApiRes {:?} \n\n CAP: {:?}", res, caps);
    }

                    
    // TODO: add label filtering
    // let mut lp = ListParams::default();
    // if let Some(label) = app.selector.clone() {
    //     lp = lp.labels(label.as_str());
    // }

    // if let Some(name) = app.name.clone() {
    //     lp = lp.fields(&format!("metadata.name={}", name));
    // }
    // let _api = discovery::dynamic_api(
    //                     ar, 
    //                     caps, 
    //                     cli, 
    //                     &conf.kube.namespaces);

    

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

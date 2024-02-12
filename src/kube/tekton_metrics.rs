use kube::{core::ApiResource,
    api::{Api, DynamicObject},
    discovery::Discovery,
};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::{self, Duration}
};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use chrono::prelude::*;

use crate::kube::client;
use crate::configuration::Settings;
use crate::output::cloud_event::http_post_dynobj;

#[derive(Debug)]
enum Command {
    Poll,
    _Ping,
    _Get(String),
    Add(PodMetrics),
    Update(PodMetricsRecord),
    _Remove(String),
    PublishAll,
}

#[derive(Debug, Clone)]
struct PodMetricsRecord {
    pub name: String,
    pub namespace: String,
    pub count_add: i32,
    pub updated_at: DateTime<Utc>,
    pub metric: Option<DynamicObject>,
}

impl Default for PodMetricsRecord {
    fn default() -> Self {
        PodMetricsRecord{
            name: String::from(""),
            namespace: String::from(""),
            count_add: 0,
            updated_at: Utc::now(),
            metric: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PodMetrics {
    pub name: String,
    pub namespace: String,
}

// Poll PodMetrics in regular time intervals
pub async fn poll_pod_metrics(conf: &Settings) -> Result<Sender<PodMetrics>> {
    let (tx, mut rx): (Sender<PodMetrics>, Receiver<PodMetrics>) = channel(32);
    let (tx_cmd, rx_cmd): (Sender<Command>, Receiver<Command>) = channel(32);
    let mut ipoll = time::interval(Duration::from_secs(30)); 
    let mut ipub = time::interval(Duration::from_secs(300)); 

    // thread 1: receive TaskRun name
    let tx_cmd2 = tx_cmd.clone();
    tokio::spawn(async move{
        loop {
            tokio::select!{
                // Receive PodMetrics event 
                we = async {
                    match rx.recv().await {
                        Some(we) => we,
                        None => {
                            //channel failure
                            PodMetrics{
                                name: String::from("n/a"),
                                namespace: String::from("n/a"),
                            }
                        }
                    }
                } => {
                    if we.name == "n/a" {
                        continue;
                    }

                    match tx_cmd2.send(Command::Add(we)).await{
                        Ok(_) => continue,
                        Err(why) => {
                            tracing::error!("Failed cmd: {:?}", why);
                        }
                    }
                }

                // Scan available PodMetrics and update DB
                _ = async {
                    ipoll.tick().await;
                }=>{
                    match tx_cmd2.send(Command::Poll).await{
                        Ok(_) => continue,
                        Err(why) => {
                            tracing::error!("Failed cmd: {:?}", why);
                        }
                    }
                }

                // Publish DB to the HTTP CloudEvents sink
                _ = async {
                    ipub.tick().await;
                }=>{
                    match tx_cmd2.send(Command::PublishAll).await{
                        Ok(_) => continue,
                        Err(why) => {
                            tracing::error!("Failed cmd: {:?}", why);
                        }
                    }
                }
            }
        }
    });

    // thread 2: Sample PodMetrics resources in k8s in regular intervals
    let tx_cmd3 = tx_cmd.clone();
    let tx_get_pm: Option<Sender<PodMetricsRecord>> = match get_pod_metrics(tx_cmd3, &conf).await{   
        Ok(tx) => {
            tracing::info!("PodMetrics channel OK");
            Some(tx)
        },
        Err(why) => {
            tracing::error!("Failed setting PodMetrics channel {}", why);
            None
        },
    };

    // thread 2: manage PodMetrics DB
    manage_pod_metrics_db(tx_get_pm, rx_cmd, &conf).await;

    Ok(tx)
}

// Start a thread to manage the PodMetrics DB
// Process Commands to manage the state of the DB
async fn manage_pod_metrics_db(tx_pm: Option<Sender<PodMetricsRecord>>, 
    mut rx_cmd: Receiver<Command>,
    conf: &Settings) {
    let mut db:BTreeMap<String, PodMetricsRecord> = BTreeMap::new();

    let conf2 = conf.clone();
    tokio::spawn(async move{
        // process PodMetrics command
        while let Some(cmd) = rx_cmd.recv().await {
            match cmd {
                // TektonRun stream events.
                // Add a new record if missing, or 
                // update updated_at if present.
                Command::Add(pm) => {
                    println!(">> Cmd::Add -> {:?}", pm);
                    let key = &format!("{}/{}", pm.namespace, pm.name);
                    if db.contains_key(key) {
                        if let Some(pmr) = db.get_mut(key){
                            pmr.updated_at = Utc::now();
                            pmr.count_add += 1;
                        }else{
                            tracing::error!("Failed to get val for key: {}", key);
                        }
                    }else{
                        db.insert(key.to_string(), PodMetricsRecord { 
                            name: pm.name, 
                            namespace: pm.namespace, 
                            count_add: 0, 
                            updated_at: Utc::now(), 
                            metric: None });
                    }
                }

                // Iterate over DB, and update PodMetrics
                Command::Poll => {
                    println!(">> Cmd::Polling");
                    // remove stale records first - last updated >5m.
                    db. 
                    retain(|_, v| {
                        let tm_diff = Utc::now() - v.updated_at;
                        if tm_diff.num_seconds() <= 300 {
                            true
                        }else{
                            false
                        }
                    });

                    // poll PodMetrics.
                    if let Some(tx) = &tx_pm {
                        for (key, pmr) in db.iter() {
                            let t_pm = pmr.clone();
                            match tx.send(t_pm).await {
                                Ok(_) => continue,
                                Err(why) => {
                                    tracing::error!("Polling failed {}: {:?}", key, why);            
                                }
                            }
                        }

                        // dump DB contents
                        print_pod_metrics_db(&db).await;

                    }else{
                        tracing::error!("Polling channel closed");
                    }

                }

                // Update or delete a PodMetricsRecord
                Command::Update(pmr) => {
                    let key = &format!("{}/{}", pmr.namespace, pmr.name);
                    println!(">> Cmd::Update {}", key);
                    db.insert(key.to_string(), pmr);
                }

                // Post PodMetrics as CloudEvents to nats-events-proxy HTTP sink
                Command::PublishAll => {
                    println!(">> Cmd::PublishAll");
                    let _ = publish_all_metrics(&db, &conf2).await;
                },

                _ => {
                    println!("Cmd not implemented: {:?}", cmd);
                }
            }
        }
    }); 
}

// start a thread that monitors PodMetrics
async fn get_pod_metrics(tx_cmd: Sender<Command>, conf: &Settings) -> Result<Sender<PodMetricsRecord>> {
    let (tx, mut rx): (Sender<PodMetricsRecord>, Receiver<PodMetricsRecord>) = channel(128);
    
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

    // Do the auto discovery of PodMetrics
    let mut is_pm_ar_found = false;

    // default PodMetrics version
    let mut pm_ar =  ApiResource {
        group: String::from("metrics.k8s.io"),
        version: String::from("v1beta1"),
        api_version: String::from("metrics.k8s.io/v1beta1"),
        kind: String::from("PodMetrics"),
        plural: String::from("pods"),
    };
    
    let discovery: Discovery;
    match Discovery::new(cli.clone()).run().await {
        Ok(d) => {
            discovery = d;
        },
        Err(why) => {
            tracing::error!("PodMetrics discovery failed: {:?}", why);
            return Err(why.into())
        },
    };

    'outer: for group in discovery.groups() {
        for (ar, _caps) in group.recommended_resources() {
            if ar.kind == "PodMetrics" {
                tracing::info!("PodMetrics API found {:?}", ar);
                is_pm_ar_found = true;
                pm_ar = ar;
                break 'outer;
            }
        }
    }

    if !is_pm_ar_found {
        tracing::error!("PodMetrics API not found");
        return Err(anyhow!("PodMetrics API not found"));
    };
 
    // PodMetrics thread
    // Get Pod name of TaskRun and namespace, create PodMetrics APi Resource
    // and collect the resource state.
    // let conf2 = conf.clone();
    tokio::spawn(async move{
        while let Some(pm) = rx.recv().await {
            let pod_name = &format!("{}-pod", pm.name);
            // tracing::info!("checking {}/{}", pm.namespace, pod_name);
            // println!(">> checking {}/{}", pm.namespace, pod_name);

            let api: Api<DynamicObject> = Api::namespaced_with(
                cli.clone(), 
                &pm.namespace,
                &pm_ar);
    
            let pmo = match api.get_opt(&pod_name.as_str()).await {
                Ok(pmo) => {
                    pmo
                },
                Err(why) => {
                    tracing::error!("Failed to get PodMetrics {:?}", why);
                    None
                }
            };
            
            let record = match pmo {
                // PodMetrics not found, skip message
                None => continue,

                Some(_) => PodMetricsRecord{
                        name: pm.name.clone(),
                        namespace: pm.namespace.clone(),
                        count_add: pm.count_add + 1,
                        updated_at: Utc::now(),
                        metric: pmo,
                    }
            };

            let _ = tx_cmd.send(Command::Update(record)).await;
           
            // let event_type = String::from("healthcat.tekton.podmetrics.v1");
            // let _ = http_post_dynobj(obj, event_type, &conf2).await;
        }
    });

    Ok(tx)
} 

// Iterate over the PodMetrics DB and post them to nats-events-proxy
async fn publish_all_metrics(db: &BTreeMap<String, PodMetricsRecord>, conf: &Settings) {
    let event_type = String::from("healthcat.tekton.podmetrics.v1");

    for (_key, pmr) in db.iter() {
        let obj = match &pmr.metric {
            Some(metric) => {
                metric
            },
            None => {
                continue;
            },
        }; 

        let _ = http_post_dynobj(obj.clone(), event_type.clone(), &conf).await;
    }
}

// print to stdout the contents of DB
async fn print_pod_metrics_db(db: &BTreeMap<String, PodMetricsRecord>) {
    println!("\n>> Printing PodMetrics DB");
    for (key, pmr) in db.iter() {
        let val = match &pmr.metric {
            Some(met) => {
                met.data.to_string()
            },
            None => {
                String::from("")
            },
        };
        println!(">> {0:<20} cnt: {1} at: {2:<20} obj: {3:<60}",
            key,
            pmr.count_add,
            pmr.updated_at,
            val);
    }
    println!("");
}
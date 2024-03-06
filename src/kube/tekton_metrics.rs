use kube::{api::{Api, DynamicObject, ListParams}, core::ApiResource, discovery::Discovery, Client, ResourceExt
};
use k8s_openapi::{
    api::core::v1::Node, apimachinery::pkg::api::resource::Quantity};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    time::{self, Duration}
};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use chrono::prelude::*;

use crate::kube::client;
use crate::configuration::Settings;
use crate::output::cloud_event::http_post_pod_metrics_record;

#[derive(Debug)]
enum Command {
    Poll,
    _Ping,
    _Get(String),
    Add(PodMetrics),
    Update(PodMetricsRecord),
    Remove(PodMetrics),
    PublishAll,
}

#[derive(Debug, Clone)]
pub struct PodMetricsRecord {
    pub name: String,
    pub namespace: String,
    pub count_add: i32,
    pub updated_at: DateTime<Utc>,
    pub metric: Option<DynamicObject>,
    pub node_metric: Option<WorkerNode>,
}

impl Default for PodMetricsRecord {
    fn default() -> Self {
        PodMetricsRecord{
            name: String::from(""),
            namespace: String::from(""),
            count_add: 0,
            updated_at: Utc::now(),
            metric: None,
            node_metric: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkerNode {
    pub status: WorkerNodeStatus
}

impl Default for WorkerNode {
    fn default() -> Self {
        WorkerNode { status: WorkerNodeStatus{
            allocatable: AllocatableResources{
                cpu: Quantity("0".to_string()),
                memory: Quantity("0".to_string()),
                }
            }
         }
    }
}

impl WorkerNode {
    fn new(cpu: Quantity, memory: Quantity) -> WorkerNode {
        WorkerNode { status: WorkerNodeStatus{
            allocatable: AllocatableResources{
                    cpu,
                    memory,
                }
            }
         }
    }
}


#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkerNodeStatus  {
    pub allocatable: AllocatableResources,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocatableResources {
    pub cpu: Quantity,
    pub memory: Quantity,
}

#[derive(Debug, Clone)]
pub struct PodMetrics {
    pub name: String,
    pub namespace: String,
    pub status: String, 
}

// Poll PodMetrics in regular time intervals
pub async fn poll_pod_metrics(conf: &Settings) -> Result<Sender<PodMetrics>> {
    let (tx, mut rx): (Sender<PodMetrics>, Receiver<PodMetrics>) = 
        channel(conf.pod_metrics.def_channel_size);
    let (tx_cmd, rx_cmd): (Sender<Command>, Receiver<Command>) = 
        channel(conf.pod_metrics.def_channel_size);
    let mut ipoll = time::interval(Duration::from_secs(conf.pod_metrics.poll_interval)); 
    let mut ipub = time::interval(Duration::from_secs(conf.pod_metrics.publish_interval)); 

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
                                status: String::from("n/a"),
                            }
                        }
                    }
                } => {
                    if we.name == "n/a" {
                        continue;
                    }

                    // \"Succeeded\"
                    let cmd = match we.status.as_str().trim_matches('"') {
                        "Succeeded" => Command::Remove(we.clone()),
                        "Failed" => Command::Remove(we.clone()),
                        _ => Command::Add(we.clone()),
                    };

                    // TODO add or remove command
                    match tx_cmd2.send(cmd).await{
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
                            metric: None,
                            node_metric: None, 
                        });
                    }
                }

                Command::Remove(pm) => {
                    println!(">> Cmd::Remove -> {:?}", pm);
                    db. 
                    retain(|_, v| {
                        if v.name == pm.name && v.namespace == pm.namespace {
                            false
                        }else{
                            true
                        }
                    });
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
    let (tx, mut rx): (Sender<PodMetricsRecord>, Receiver<PodMetricsRecord>) = 
        channel(conf.pod_metrics.record_channel_size);
    
    let mut db_workers:BTreeMap<String, WorkerNode> = BTreeMap::new();

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
    
    let mut pod_ar =  ApiResource {
        group: String::from(""),
        version: String::from("v1"),
        api_version: String::from("v1"),
        kind: String::from("Pod"),
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

    let mut cnt_found = 0;
    'outer: for group in discovery.groups() {
        for (ar, _caps) in group.recommended_resources() {
            
            match ar.kind.as_str() {
                "PodMetrics" => {
                    tracing::info!("PodMetrics API found {:?}", ar);
                    is_pm_ar_found = true;
                    pm_ar = ar;
                    cnt_found += 1;
                }

                "Pod" => {
                    tracing::info!("Pods API found {:?}", ar);
                    pod_ar = ar;
                    cnt_found += 1;
                }

                &_ => {
                    if cnt_found == 2 {
                        break 'outer
                    }
                }
            }
        }
    }

    if !is_pm_ar_found {
        tracing::error!("PodMetrics API not found");
        return Err(anyhow!("PodMetrics API not found"));
    };
 
    // get nodes Allocatable CPU and Mem thresholds
    match get_worker_nodes_allocations(cli.clone(), &mut db_workers).await {
        Ok(_) => {
            println!(" >> Worker nodes allocation loaded successfully");
        },
        Err(why) => {
            println!(" >> Worker nodes allocations loading failed {}", why);
        }
    }

    // PodMetrics thread
    // Get Pod name of TaskRun and namespace, create PodMetrics APi Resource
    // and collect the resource state.
    // let conf2 = conf.clone();
    tokio::spawn(async move{
        while let Some(pm) = rx.recv().await {
            let pod_name = &format!("{}-pod", pm.name);
            // tracing::info!("checking {}/{}", pm.namespace, pod_name);
            // println!(">> checking {}/{}", pm.namespace, pod_name);

            //get PodMetrics 
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
                //
                // PodMetrics not found, skip message
                //
                None => continue,

                Some(_) => {
                    // Extract worker nodeName from TaskRun pod
                    let api_pod: Api<DynamicObject> = Api::namespaced_with(
                        cli.clone(), 
                        &pm.namespace,
                        &pod_ar);
            
                    let pod_node_name = match api_pod.get_opt(&pod_name.as_str()).await {
                        Ok(podo) => {
                            let mut node_name = "Unknown".to_string();
                            if let Some(pod) = podo {
                                if let Some(spec) = pod.data.get("spec") {
                                    if let Some(worker) = spec.get("nodeName") {
                                        node_name = worker.to_string();
                                    }
                                }
                            }

                            node_name.trim_matches('"').to_string()
                        },
                        Err(why) => {
                            tracing::error!("Failed to get Pod[{}] {:?}", pod_name, why);
                            "Unknown".to_string()
                        }
                    };

                    //
                    // Get NodeMetrics for the worker
                    //
                    let cpu = match db_workers.clone().get(&pod_node_name) {
                        Some(c) => c.status.allocatable.cpu.clone(),
                        None => Quantity("0".to_string()),
                    };

                    let memory = match db_workers.clone().get(&pod_node_name) {
                        Some(c) => c.status.allocatable.memory.clone(),
                        None => Quantity("0".to_string()),
                    };

                    let node_usage = WorkerNode::new(cpu, memory);

                    PodMetricsRecord{
                        name: pm.name.clone(),
                        namespace: pm.namespace.clone(),
                        count_add: pm.count_add + 1,
                        updated_at: Utc::now(),
                        metric: pmo,
                        node_metric: Some(node_usage),
                    }
                }
            };

            let _ = tx_cmd.send(Command::Update(record)).await;
           
        }
    });

    Ok(tx)
} 

// Iterate over the PodMetrics DB and post them to nats-events-proxy
async fn publish_all_metrics(db: &BTreeMap<String, PodMetricsRecord>, conf: &Settings) {
    let event_type = String::from("healthcat.tekton.podmetrics.v1");

    for (_key, pmr) in db.iter() { 
        let pod_metrics = PodMetricsRecord {
            name: pmr.name.clone(),
            namespace: pmr.namespace.clone(),
            count_add: pmr.count_add.clone(),
            updated_at: pmr.updated_at,
            metric: pmr.metric.clone(),
            node_metric: pmr.node_metric.clone(),
        };
        let _ = http_post_pod_metrics_record(pod_metrics, event_type.clone(), &conf).await;
        // let _ = http_post_dynobj(obj.clone(), event_type.clone(), &conf).await;
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
        let node_metrics = match &pmr.node_metric {
            Some(m) => m.clone(),
            None => WorkerNode::default(),
        };

        println!(">> {0:<20} cnt: {1} at: {2:<20} node[cpu:{3:?} mem: {4:?}] obj: {5:<60}",
            key,
            pmr.count_add,
            pmr.updated_at,
            node_metrics.status.allocatable.cpu,
            node_metrics.status.allocatable.memory,
            val);
    }
    println!("");
}

// Retrieve and store in-mem all Worker nodes allocations
async fn get_worker_nodes_allocations(cli: Client,
    db: &mut BTreeMap<String, WorkerNode>) ->  anyhow::Result<()> {

    //get worker Nodes allocations 
    let nodes_api: Api<Node> = Api::all(cli.clone());

    // select workers used for performance
    let lp = ListParams::default().
    labels("node-role.kubernetes.io/worker=,node-role.kubernetes.io/performance=");

    let nodes = match nodes_api.list(&lp).await {
        Ok(n) => n,
        Err(why) => {
            println!(" >>>> get_worker_nodes_allocations nodes API: {}", why);
            tracing::error!("Listing Nodes failed: {}", why);
            return Err(anyhow!("Listing Nodes failed"));
        },
    };

    if nodes.items.len() == 0 {
        tracing::error!("Missing worker Nodes");
        return Err(anyhow!("Missing worker Nodes"));
    };

    for node in nodes {
        let node_name = node.name_any();
        if let Some(ns) = node.status {
            if let Some(aloc) = ns.allocatable {
                let cpu = match aloc.get("cpu") {
                    Some(c) => c.clone(),
                    None => Quantity("0".to_string()),
                };

                let mem = match aloc.get("memory") {
                    Some(m) => m.clone(),
                    None => Quantity("0".to_string()),
                };

                println!(" >> Worker[{0}] cpu: {1:?} mem: {2:?}", node_name,
                                                            cpu,
                                                            mem);
                db.insert(node_name, WorkerNode { 
                        status: WorkerNodeStatus { 
                            allocatable: AllocatableResources { 
                                cpu: cpu.clone(), 
                                memory: mem.clone() 
                            }}});
            }
        }
    }

    Ok(())
}
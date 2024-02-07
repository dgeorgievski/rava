# Metrics 

# K8S Metrics API

## k8s issues
### Metrics API is missing a watcher. 
Work is in progress. Polling is the only option at the moment.
- Discussion: https://groups.google.com/g/kubernetes-sig-instrumentation/c/_b6c0oyPLJA/m/Y4rMQTBDAgAJ
- Metrics SIG: https://github.com/kubernetes/enhancements/tree/master/keps/sig-instrumentation/1013-metrics-watch-api
- kube-rs issue: https://github.com/kube-rs/kube/issues/1092

## CLI examples
```sh
$ kubectl get --raw /apis/metrics.k8s.io/v1beta1/namespaces/tenant-smf/pods
$ kubectl get --raw "/apis/metrics.k8s.io/v1beta1/namespaces/tenant-smf/pods/tenant-smf-regression-4-sbqf7-run-tests-pod" | jq 
```
Result
```json
{
  "kind": "PodMetrics",
  "apiVersion": "metrics.k8s.io/v1beta1",
  "metadata": {
    "name": "tenant-smf-regression-4-sbqf7-run-tests-pod",
    "namespace": "tenant-smf",
    "creationTimestamp": "2024-01-31T16:51:39Z",
    "labels": {
      "5g-core.casa-systems.com/app": "smf",
      "5g-core.casa-systems.com/cluster": "ci-cd",
      "5g-core.casa-systems.com/pipelineType": "3bt",
      "5g-core.casa-systems.com/taskType": "test-management",
      "app": "testmanager-pipeline",
      "app.kubernetes.io/managed-by": "tekton-pipelines",
      "tekton.dev/memberOf": "tasks",
      "tekton.dev/pipeline": "tenant-smf-regression-4",
      "tekton.dev/pipelineRun": "tenant-smf-regression-4-sbqf7",
      "tekton.dev/pipelineTask": "run-tests",
      "tekton.dev/taskRun": "tenant-smf-regression-4-sbqf7-run-tests",
      "testing.casa-systems.com/type": "helper",
      "triggers.tekton.dev/eventlistener": "regression",
      "triggers.tekton.dev/trigger": "regression-trigger",
      "triggers.tekton.dev/triggers-eventid": "ef2ead9f-ba97-4c34-b018-92d26f8e8526",
      "vendor": "casa"
    }
  },
  "timestamp": "2024-01-31T16:51:39Z",
  "window": "5m0s",
  "containers": [
    {
      "name": "step-run-tests",
      "usage": {
        "cpu": "0",
        "memory": "16836Ki"
      }
    },
    {
      "name": "POD",
      "usage": {
        "cpu": "0",
        "memory": "0"
      }
    }
  ]
}
```


Delete 
```json
ApiWithSelectors {
    label_selectors: Some(
        [],
    ),
    field_selectors: Some(
        [],
    ),
    event_type: "healthcat.tekton.podmetrics.v1",
    api_dyn: Api {
        request: Request {
            url_path: "/apis/metrics.k8s.io/v1beta1/pods",
        },
        client: "...",
        namespace: None,
    },
}

```

TaskRun
```json
&apisel = ApiWithSelectors {
    label_selectors: Some(
        [
            "tekton.dev/pipeline=vzw-tekton-server-e2e",
        ],
    ),
    field_selectors: Some(
        [],
    ),
    event_type: "healthcat.tekton.taskrun.v1",
    api_dyn: Api {
        request: Request {
            url_path: "/apis/tekton.dev/v1/namespaces/casa-platform-ci/taskruns",
        },
        client: "...",
        namespace: Some(
            "casa-platform-ci",
        ),
    },
}
```
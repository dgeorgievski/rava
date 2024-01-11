# Handling errors and missing info

## Missimg TypeMeta
Examples of requests URLs used by kube-rs client to watch over resources.
```sh
# Core v1 events
/api/v1/events
/api/v1/namespaces/tenant-upf/events

# events.k8s.io events
/apis/events.k8s.io/v1/events
/apis/events.k8s.io/v1/namespaces/tenant-upf/event
```

Routes
```sh
/apis/route.openshift.io/v1/routes
/apis/route.openshift.io/v1/namespaces/openshift-gitops/routes
```

Pattern
```sh
# core
## All namespaces
/api/v1/{resource_name}s
## Some namespace
/api/v1/namespaces/{namespace}/{resource}s

# apis
## All namespaces
/apis/{group_name}/{group_version}/{resource_name}s
# Some namespace
/apis/{group_name}/{group_version}/namespaces/{namespace}/{resource_name}s

```
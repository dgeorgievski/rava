# healthcat-rs 

Observe Kubernetes resources in a given cluster and report their state over HTTP as CloudEvents.

## Benefits of using `kuber-rs` client

This version of healthcat takes advantage of Rust `kube-rs` dynamic capabilities to read resources in Kubernetes with addresses some challenges with `client-go` based clients:
1. Read any API resource. Kubernetes or CRD based, in a given cluster.
   With `client-go` the biggest challenge is trying to use the same client for varieties of API resources. Often, the upstream projects providing informers libraries have conflicting dependencies or worse, in case of RedHat OpenShift, they use forked `client-go` for development of their API objects.
2. Use static data types for tracked API resources. 
   With dynamic informers in client-go one need to resort to reflection to cast data types, which makes the code management harder.
3. Better streaming for k8s Watchers/Informers
   K8s streams could break or stall, that's my experience for example with Tekton related API resources like PipelineRuns,  after which they need to be reset. `kube-rs`, according to its developers, resets streams automatically.



Run PipelineRun
```json

 "status": Object {},

 "startTime": String("2024-01-22T15:22:30Z"),
"conditions": Array [
Object {
    "lastTransitionTime": String("2024-01-22T15:22:30Z"),
    "message": String("Tasks Completed: 0 (Failed: 0, Cancelled 0), Incomplete: 2, Skipped: 0"),
    "reason": String("Running"),
    "status": String("Unknown"),
    "type": String("Succeeded"),
},
],

"startTime": String("2024-01-22T15:22:30Z"),
"conditions": Array [
    Object {
        "lastTransitionTime": String("2024-01-22T15:22:48Z"),
        "message": String("Tasks Completed: 1 (Failed: 0, Cancelled 0), Incomplete: 1, Skipped: 0"),
        "reason": String("Running"),
        "status": String("Unknown"),
        "type": String("Succeeded"),
    },
    ],
 
"completionTime": String("2024-01-22T15:23:45Z"),
"conditions": Array [
    Object {
        "lastTransitionTime": String("2024-01-22T15:23:45Z"),
        "message": String("Tasks Completed: 2 (Failed: 0, Cancelled 0), Skipped: 0"),
        "reason": String("Succeeded"),
        "status": String("True"),
        "type": String("Succeeded"),
    },
],


"completionTime": String("2024-01-22T15:23:45Z"),
"startTime": String("2024-01-22T15:22:30Z"),
"conditions": Array [
    Object {
        "lastTransitionTime": String("2024-01-22T15:23:45Z"),
        "message": String("Tasks Completed: 2 (Failed: 0, Cancelled 0), Skipped: 0"),
        "reason": String("Succeeded"),
        "status": String("True"),
        "type": String("Succeeded"),
    },
],
```

Delete PipelineRun
```json

managed_fields: Some(
    [
         ManagedFieldsEntry {
             operation: Some(
                        "Update",
                    ),
"startTime": String("2024-01-22T15:22:30Z"),
"completionTime": String("2024-01-22T15:23:45Z"),
"conditions": Array [
    Object {
        "lastTransitionTime": String("2024-01-22T15:23:45Z"),
        "message": String("Tasks Completed: 2 (Failed: 0, Cancelled 0), Skipped: 0"),
        "reason": String("Succeeded"),
        "status": String("True"),
        "type": String("Succeeded"),
    },
],


```

TaskRuns
```json

"status": {
      "conditions": [
          {
              "lastTransitionTime": "2024-01-22T16:31:36Z",
              "message": "pod status \"Initialized\":\"False\"; message: \"containers with incomplete status: [prepare place-scripts working-dir-initializer]\"",
              "reason": "Pending",
              "status": "Unknown",
              "type": "Succeeded"
          }
      ],
      "podName": "gw-tekton-events-push-tag-52jzh-build-push-pod",
      "startTime": "2024-01-22T16:31:35Z",
      "steps": [
          {
              "container": "step-build-and-push",
              "name": "build-and-push",
              "waiting": {
                  "reason": "PodInitializing"
              }
          },
          {
              "container": "step-write-url",
              "name": "write-url",
              "waiting": {
                  "reason": "PodInitializing"
              }
          }
      ],

"status": {
      "completionTime": "2024-01-22T16:32:36Z",
      "conditions": [
          {
              "lastTransitionTime": "2024-01-22T16:32:36Z",
              "message": "All Steps have completed executing",
              "reason": "Succeeded",
              "status": "True",
              "type": "Succeeded"
          }
      ],
      "podName": "gw-tekton-events-push-tag-52jzh-build-push-pod",
      "results": [
          {
              "name": "IMAGE_DIGEST",
              "type": "string",
              "value": "sha256:154ba65aa73a05cf38b41b770a2e8e1acbf0b1c38f0c0a457cb96ce090159ebd"
          },
          {
              "name": "IMAGE_URL",
              "type": "string",
              "value": "registry.gitlab.casa-systems.com/dimitar.georgievski/gw-tekton-events:master"
          }
      ],
      "startTime": "2024-01-22T16:31:36Z",
      "steps": [
          {
              "container": "step-build-and-push",
              "imageID": "gcr.io/kaniko-project/executor@sha256:68bb272f681f691254acfbdcef00962f22efe2f0c1e287e6a837b0abe07fb94b",
              "name": "build-and-push",
              "terminated": {
                  "containerID": "cri-o://b9fbf230f37b9dfa49b4ba97e2978a01b0296f7ed71f9995ce36e1189829f135",
                  "exitCode": 0,
                  "finishedAt": "2024-01-22T16:32:34Z",
                  "message": "[{\"key\":\"IMAGE_DIGEST\",\"value\":\"sha256:154ba65aa73a05cf38b41b770a2e8e1acbf0b1c38f0c0a457cb96ce090159ebd\",\"type\":1}]",    
                  "reason": "Completed",
                  "startedAt": "2024-01-22T16:31:46Z"
              }
          },
          {
              "container": "step-write-url",
              "imageID": "docker.artifactory.casa-systems.com/library/bash@sha256:c523c636b722339f41b6a431b44588ab2f762c5de5ec3bd7964420ff982fb1d9",
              "name": "write-url",
              "terminated": {
                  "containerID": "cri-o://fd35d71d0d52cecc4b7a6acd8972a63490073481ef8e3f2382f23c4a46a84a1a",
                  "exitCode": 0,
                  "finishedAt": "2024-01-22T16:32:35Z",
                  "message": "[{\"key\":\"IMAGE_DIGEST\",\"value\":\"sha256:154ba65aa73a05cf38b41b770a2e8e1acbf0b1c38f0c0a457cb96ce090159ebd\",\"type\":1},{\"key\":\"IMAGE_URL\",\"value\":\"registry.gitlab.casa-systems.com/dimitar.georgievski/gw-tekton-events:master\",\"type\":1}]",
                  "reason": "Completed",
                  "startedAt": "2024-01-22T16:32:34Z"
              }
          }
      ],

 "status": {
      "conditions": [
          {
              "lastTransitionTime": "2024-01-22T16:31:36Z",
              "message": "pod status \"Initialized\":\"False\"; message: \"containers with incomplete status: [prepare place-scripts working-dir-initializer]\"",
              "reason": "Pending",
              "status": "Unknown",
              "type": "Succeeded"
          }
      ],
      "podName": "gw-tekton-events-push-tag-52jzh-build-push-pod",
      "startTime": "2024-01-22T16:31:36Z",
      "steps": [
          {
              "container": "step-build-and-push",
              "name": "build-and-push",
              "waiting": {
                  "reason": "PodInitializing"
              }
          },
          {
              "container": "step-write-url",
              "name": "write-url",
              "waiting": {
                  "reason": "PodInitializing"
              }
          }
      ],      

```

Pods used by steps in TaskRuns
```sh
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     Pending   0             0s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     Pending   0             0s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     Init:0/2   0             0s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     Init:0/2   0             12s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     Init:1/2   0             13s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     PodInitializing   0             14s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   1/1     Running           0             15s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   1/1     Running           0             15s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     Completed         0             18s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Pending           0             0s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Pending           0             0s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Init:0/3          0             0s
gw-tekton-events-push-tag-52jzh-fetch-source-pod   0/1     Completed         0             21s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Init:0/3          0             3s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Init:1/3          0             5s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Init:2/3          0             6s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     PodInitializing   0             7s
gw-tekton-events-push-tag-52jzh-build-push-pod     2/2     Running           0             9s
gw-tekton-events-push-tag-52jzh-build-push-pod     2/2     Running           0             9s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Completed         0             61s
gw-tekton-events-push-tag-52jzh-build-push-pod     0/2     Completed         0             65s
```
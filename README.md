# distributed-lecture-example
Example worker pool implementation for a lecture on distributed computing.

## Prerequisites
This example uses `tonic` and thus requires `protoc` to be installed. See https://grpc.io/docs/protoc-installation/ for instructions for your system.

## Components
This example is made up of four parts: an injector for adding work, a coordinator for tracking work, a worker for processing work, and a watcher for monitoring the coordinator.

### Coordinator
The Coordinator receives new work and retains a queue of waiting work. Workers can request a new piece of work to perform from the Coordinator.

### Injector
An Injector repeatedly adds work to a Coordinator to be queued for eventual processing.

### Worker
A Worker requests new work from the Coordinator and sleeps for a period of time determined by the work parameters. Any number of Work instances can connect to a Coordinator.

### Watcher
Monitoring service for the Coordinator.

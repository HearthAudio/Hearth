# Architecture
Hearth is made up by a collection of tokio taks the primary tasks that 
run for the duration of the program are:
1. The Scheduler (If enabled in config)
2. Thw worker (If enabled in config)

## Scheduler

The Scheduler is the simplest it only handles taking in jobs queued from the client and distributing them across connected workers. 
It also handles identifying workers on startup via a ping-pong system.

## Worker
Thw worker receives messages from the client and the scheduler.
When a new job is started on the worker that job is run inside of a Tokio task
that listens on an IPC channel for messages from the main worker thread that listens to the kafka broker.

Messages received from the client or DirectWorkerCommunication messages
are for things like pausing, seeking, resuming, etc...
thing that happen once the job has been scheduled on the worker. 

Messages received from the scheduler are for ping-pong identification and for scheduling jobs.

## Job Scheduling Process
To schedule a job a number of steps must first take place in the below order:
1. Client sends ExternalQueueJobRequest
2. Scheduler receives request and uses Round-Robin to pick worker to schedule on
3. Scheduler sends InternalWorkerQueueJob to worker that includes information for scheduling the job
4. Worker receives InternalWorkerQueueJob and schedules the job
5. Worker sends ExternalQueueJobResponse that includes information like the job ID so the client can properly address future messages

## Error Reporting
If Hearth encounters an error an ErrorReport message is sent to the client whcih contains the:
* GuildID
* Error message
* JobID
* RequestID

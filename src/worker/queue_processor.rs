// Processing streaming jobs from kafka queue

// Parallelize with tokio

pub fn process_job() {
    let worker = tokio::spawn(async move {
        return initialize_worker_internal(worker_config).await;
    });
    let futures = vec![
        scheduler,
        worker,
    ];
    futures::future::join_all(futures).await;
}
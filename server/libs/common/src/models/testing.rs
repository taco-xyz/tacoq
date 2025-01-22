use crate::models::{TaskKind, Worker, WorkerKind};

pub fn setup_worker_kinds() -> Vec<WorkerKind> {
    vec![
        WorkerKind::new(
            "worker1".to_string(),
            "task1".to_string(),
            "task1".to_string(),
        ),
        WorkerKind::new(
            "worker2".to_string(),
            "task2".to_string(),
            "task2".to_string(),
        ),
        WorkerKind::new(
            "worker3".to_string(),
            "task2".to_string(),
            "task2".to_string(),
        ),
    ]
}

pub fn setup_task_kinds(worker_kinds: &Vec<WorkerKind>) -> Vec<TaskKind> {
    vec![
        TaskKind::new("task1".to_string(), worker_kinds[0].name.clone()),
        TaskKind::new("task2".to_string(), worker_kinds[1].name.clone()),
        TaskKind::new("task3".to_string(), worker_kinds[2].name.clone()),
    ]
}

pub fn setup_workers(worker_kinds: &Vec<WorkerKind>) -> Vec<Worker> {
    vec![
        Worker::new("worker1".to_string(), worker_kinds[0].name.clone()),
        Worker::new("worker2".to_string(), worker_kinds[1].name.clone()),
        Worker::new("worker3".to_string(), worker_kinds[2].name.clone()),
    ]
}

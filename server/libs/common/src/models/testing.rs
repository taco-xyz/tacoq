#[cfg(test)]
pub mod test {
    use crate::models::{TaskKind, Worker, WorkerKind};

    pub fn setup_worker_kinds() -> Vec<WorkerKind> {
        vec![
            WorkerKind::new("kind1", "task1", "task1"),
            WorkerKind::new("kind2", "task2", "task2"),
            WorkerKind::new("kind3", "task3", "task3"),
        ]
    }

    pub fn setup_task_kinds(worker_kinds: &Vec<WorkerKind>) -> Vec<TaskKind> {
        vec![
            TaskKind::new("task1", &worker_kinds[0].name),
            TaskKind::new("task2", &worker_kinds[1].name),
            TaskKind::new("task3", &worker_kinds[2].name),
        ]
    }

    pub fn setup_workers(worker_kinds: &Vec<WorkerKind>) -> Vec<Worker> {
        vec![
            Worker::new("worker1", &worker_kinds[0].name),
            Worker::new("worker2", &worker_kinds[1].name),
            Worker::new("worker3", &worker_kinds[2].name),
        ]
    }
}

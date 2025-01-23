#[cfg(test)]
pub mod test {
    use crate::models::{TaskKind, Worker, WorkerKind};

    pub fn setup_worker_kinds() -> Vec<WorkerKind> {
        vec![
            WorkerKind::new("worker1", "task1", "task1"),
            WorkerKind::new("worker2", "task2", "task2"),
            WorkerKind::new("worker3", "task2", "task2"),
        ]
    }

    pub fn setup_task_kinds(worker_kinds: &Vec<WorkerKind>) -> Vec<TaskKind> {
        vec![
            TaskKind::new("task1", worker_kinds[0].name.clone()),
            TaskKind::new("task2", worker_kinds[1].name.clone()),
            TaskKind::new("task3", worker_kinds[2].name.clone()),
        ]
    }

    pub fn setup_workers(worker_kinds: &Vec<WorkerKind>) -> Vec<Worker> {
        vec![
            Worker::new("worker1", worker_kinds[0].name.clone()),
            Worker::new("worker2", worker_kinds[1].name.clone()),
            Worker::new("worker3", worker_kinds[2].name.clone()),
        ]
    }
}

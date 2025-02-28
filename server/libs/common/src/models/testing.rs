#[cfg(test)]
pub mod test {
    use uuid::Uuid;

    use crate::models::{Worker, WorkerKind};

    pub fn get_worker_kinds() -> Vec<WorkerKind> {
        vec![
            WorkerKind::new("kind1", "task1", "task1"),
            WorkerKind::new("kind2", "task2", "task2"),
            WorkerKind::new("kind3", "task3", "task3"),
        ]
    }

    pub fn get_workers(worker_kinds: &[WorkerKind]) -> Vec<Worker> {
        vec![
            Worker::new(Uuid::new_v4(), &worker_kinds[0].name),
            Worker::new(Uuid::new_v4(), &worker_kinds[1].name),
            Worker::new(Uuid::new_v4(), &worker_kinds[2].name),
        ]
    }
}

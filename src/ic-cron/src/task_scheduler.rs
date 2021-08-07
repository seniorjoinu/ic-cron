use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ic_cdk::export::candid::utils::ArgumentEncoder;
use ic_cdk::export::candid::{Principal, Result as CandidResult};
use union_utils::RemoteCallEndpoint;

use crate::types::{
    Iterations, SchedulingInterval, Task, TaskExecutionQueue, TaskId, TaskTimestamp,
};

#[derive(Default)]
pub struct TaskScheduler {
    pub tasks: HashMap<TaskId, Task>,
    pub task_id_counter: TaskId,
    pub queue: TaskExecutionQueue,
}

impl TaskScheduler {
    pub fn enqueue<Tuple: ArgumentEncoder>(
        &mut self,
        endpoint: RemoteCallEndpoint,
        args: Tuple,
        cycles: u64,
        scheduling_interval: SchedulingInterval,
        timestamp: u64,
    ) -> CandidResult<TaskId> {
        let id = self.generate_task_id();
        let task = Task::new(
            id,
            endpoint,
            args,
            cycles,
            timestamp,
            None,
            scheduling_interval,
        )?;

        match task.scheduling_interval.iterations {
            Iterations::Exact(times) => {
                if times > 0 {
                    self.queue.push(TaskTimestamp {
                        task_id: id,
                        timestamp: timestamp + task.scheduling_interval.duration_nano,
                    })
                }
            }
            Iterations::Infinite => self.queue.push(TaskTimestamp {
                task_id: id,
                timestamp: timestamp + task.scheduling_interval.duration_nano,
            }),
        };

        self.tasks.insert(id, task);

        Ok(id)
    }

    pub fn iterate(&mut self, timestamp: u64) -> Vec<Task> {
        let mut tasks = vec![];

        for task_id in self
            .queue
            .pop_ready(timestamp)
            .into_iter()
            .map(|it| it.task_id)
        {
            let mut should_remove = false;

            match self.tasks.entry(task_id) {
                Entry::Occupied(mut entry) => {
                    let task = entry.get_mut();

                    match task.scheduling_interval.iterations {
                        Iterations::Infinite => {
                            let new_rescheduled_at =
                                if let Some(rescheduled_at) = task.rescheduled_at {
                                    rescheduled_at + task.scheduling_interval.duration_nano
                                } else {
                                    task.scheduled_at + task.scheduling_interval.duration_nano
                                };

                            task.rescheduled_at = Some(new_rescheduled_at);

                            self.queue.push(TaskTimestamp {
                                task_id,
                                timestamp: new_rescheduled_at
                                    + task.scheduling_interval.duration_nano,
                            });
                        }
                        Iterations::Exact(times_left) => {
                            if times_left > 1 {
                                let new_rescheduled_at =
                                    if let Some(rescheduled_at) = task.rescheduled_at {
                                        rescheduled_at + task.scheduling_interval.duration_nano
                                    } else {
                                        task.scheduled_at + task.scheduling_interval.duration_nano
                                    };

                                task.rescheduled_at = Some(new_rescheduled_at);

                                self.queue.push(TaskTimestamp {
                                    task_id,
                                    timestamp: new_rescheduled_at
                                        + task.scheduling_interval.duration_nano,
                                });

                                task.scheduling_interval.iterations =
                                    Iterations::Exact(times_left - 1);
                            } else {
                                should_remove = true;
                            }
                        }
                    };

                    tasks.push(task.clone());
                }
                Entry::Vacant(_) => {}
            }

            if should_remove {
                self.tasks.remove(&task_id);
            }
        }

        tasks
    }

    pub fn dequeue(&mut self, task_id: TaskId) -> Option<Task> {
        self.tasks.remove(&task_id)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn get_task_by_id(&self, task_id: &TaskId) -> Option<Task> {
        self.tasks.get(task_id).cloned()
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    fn generate_task_id(&mut self) -> TaskId {
        let res = self.task_id_counter;
        self.task_id_counter += 1;

        res
    }
}

#[cfg(test)]
mod tests {
    use union_utils::{random_principal_test, RemoteCallEndpoint};

    use crate::task_scheduler::TaskScheduler;
    use crate::types::{Iterations, SchedulingInterval};

    #[test]
    fn queue_works_fine() {
        let mut scheduler = TaskScheduler::default();

        let task_id_1 = scheduler
            .enqueue(
                RemoteCallEndpoint {
                    canister_id: random_principal_test(),
                    method_name: "test".into(),
                },
                (10, "abc"),
                0,
                SchedulingInterval {
                    duration_nano: 10,
                    iterations: Iterations::Exact(1),
                },
                0,
            )
            .ok()
            .unwrap();

        let task_id_2 = scheduler
            .enqueue(
                RemoteCallEndpoint {
                    canister_id: random_principal_test(),
                    method_name: "test".into(),
                },
                (10, "abc"),
                0,
                SchedulingInterval {
                    duration_nano: 10,
                    iterations: Iterations::Infinite,
                },
                0,
            )
            .ok()
            .unwrap();

        let task_id_3 = scheduler
            .enqueue(
                RemoteCallEndpoint {
                    canister_id: random_principal_test(),
                    method_name: "test".into(),
                },
                (),
                0,
                SchedulingInterval {
                    duration_nano: 20,
                    iterations: Iterations::Exact(2),
                },
                0,
            )
            .ok()
            .unwrap();

        assert!(!scheduler.is_empty(), "Scheduler is not empty");

        let tasks_emp = scheduler.iterate(5);
        assert!(
            tasks_emp.is_empty(),
            "There should not be any tasks at timestamp 5"
        );

        let tasks_1_2 = scheduler.iterate(10);
        assert_eq!(
            tasks_1_2.len(),
            2,
            "At timestamp 10 there should be 2 tasks"
        );
        assert!(
            tasks_1_2.iter().any(|t| t.id == task_id_1),
            "Should contain task 1"
        );
        assert!(
            tasks_1_2.iter().any(|t| t.id == task_id_2),
            "Should contain task 2"
        );

        let tasks_emp = scheduler.iterate(15);
        assert!(
            tasks_emp.is_empty(),
            "There should not be any tasks at timestamp 15"
        );

        let tasks_2_3 = scheduler.iterate(20);
        assert_eq!(
            tasks_2_3.len(),
            2,
            "At timestamp 20 there should be 2 tasks"
        );
        assert!(
            tasks_2_3.iter().any(|t| t.id == task_id_2),
            "Should contain task 2"
        );
        assert!(
            tasks_2_3.iter().any(|t| t.id == task_id_3),
            "Should contain task 3"
        );

        let tasks_2 = scheduler.iterate(30);
        assert_eq!(
            tasks_2.len(),
            1,
            "There should be a single task at timestamp 30"
        );
        assert_eq!(tasks_2[0].id, task_id_2, "Should contain task 2");

        let tasks_2_3 = scheduler.iterate(42);
        assert_eq!(
            tasks_2_3.len(),
            2,
            "At timestamp 40 there should be 2 tasks"
        );
        assert!(
            tasks_2_3.iter().any(|t| t.id == task_id_2),
            "Should contain task 2"
        );
        assert!(
            tasks_2_3.iter().any(|t| t.id == task_id_3),
            "Should contain task 3"
        );

        let tasks_2 = scheduler.iterate(55);
        assert_eq!(
            tasks_2.len(),
            1,
            "There should be a single task at timestamp 60"
        );
        assert_eq!(tasks_2[0].id, task_id_2, "Should contain task 2");

        let tasks_2 = scheduler.iterate(60);
        assert_eq!(
            tasks_2.len(),
            1,
            "There should be a single task at timestamp 60"
        );
        assert_eq!(tasks_2[0].id, task_id_2, "Should contain task 2");
    }
}

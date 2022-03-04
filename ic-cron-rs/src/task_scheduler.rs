use std::collections::hash_map::Entry;
use std::collections::HashMap;

use ic_cdk::export::candid::{CandidType, Deserialize, Result as CandidResult};

use crate::types::{
    Iterations, ScheduledTask, SchedulingOptions, TaskExecutionQueue, TaskId, TaskTimestamp,
};

#[derive(Default, CandidType, Deserialize, Clone)]
pub struct TaskScheduler {
    pub tasks: HashMap<TaskId, ScheduledTask>,
    pub task_id_counter: TaskId,

    pub queue: TaskExecutionQueue,
}

impl TaskScheduler {
    pub fn enqueue<TaskPayload: CandidType>(
        &mut self,
        payload: TaskPayload,
        scheduling_interval: SchedulingOptions,
        timestamp: u64,
    ) -> CandidResult<TaskId> {
        let id = self.generate_task_id();
        let task = ScheduledTask::new(id, payload, timestamp, None, scheduling_interval)?;

        match task.scheduling_options.iterations {
            Iterations::Exact(times) => {
                if times > 0 {
                    self.queue.push(TaskTimestamp {
                        task_id: id,
                        timestamp: timestamp + task.scheduling_options.delay_nano,
                    })
                }
            }
            Iterations::Infinite => self.queue.push(TaskTimestamp {
                task_id: id,
                timestamp: timestamp + task.scheduling_options.delay_nano,
            }),
        };

        self.tasks.insert(id, task);

        Ok(id)
    }

    pub fn iterate(&mut self, timestamp: u64) -> Vec<ScheduledTask> {
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

                    match task.scheduling_options.iterations {
                        Iterations::Infinite => {
                            let new_rescheduled_at = if task.delay_passed {
                                if let Some(rescheduled_at) = task.rescheduled_at {
                                    rescheduled_at + task.scheduling_options.interval_nano
                                } else {
                                    task.scheduled_at + task.scheduling_options.interval_nano
                                }
                            } else {
                                task.delay_passed = true;

                                if let Some(rescheduled_at) = task.rescheduled_at {
                                    rescheduled_at + task.scheduling_options.delay_nano
                                } else {
                                    task.scheduled_at + task.scheduling_options.delay_nano
                                }
                            };

                            task.rescheduled_at = Some(new_rescheduled_at);

                            self.queue.push(TaskTimestamp {
                                task_id,
                                timestamp: new_rescheduled_at
                                    + task.scheduling_options.interval_nano,
                            });
                        }
                        Iterations::Exact(times_left) => {
                            if times_left > 1 {
                                let new_rescheduled_at = if task.delay_passed {
                                    if let Some(rescheduled_at) = task.rescheduled_at {
                                        rescheduled_at + task.scheduling_options.interval_nano
                                    } else {
                                        task.scheduled_at + task.scheduling_options.interval_nano
                                    }
                                } else {
                                    task.delay_passed = true;

                                    if let Some(rescheduled_at) = task.rescheduled_at {
                                        rescheduled_at + task.scheduling_options.delay_nano
                                    } else {
                                        task.scheduled_at + task.scheduling_options.delay_nano
                                    }
                                };

                                task.rescheduled_at = Some(new_rescheduled_at);

                                self.queue.push(TaskTimestamp {
                                    task_id,
                                    timestamp: new_rescheduled_at
                                        + task.scheduling_options.interval_nano,
                                });

                                task.scheduling_options.iterations =
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

    pub fn dequeue(&mut self, task_id: TaskId) -> Option<ScheduledTask> {
        self.tasks.remove(&task_id)
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn get_task(&self, task_id: &TaskId) -> Option<&ScheduledTask> {
        self.tasks.get(task_id)
    }

    pub fn get_task_mut(&mut self, task_id: &TaskId) -> Option<&mut ScheduledTask> {
        self.tasks.get_mut(task_id)
    }

    pub fn get_task_by_id_cloned(&self, task_id: &TaskId) -> Option<ScheduledTask> {
        self.get_task(task_id).cloned()
    }

    pub fn get_tasks_cloned(&self) -> Vec<ScheduledTask> {
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
    use ic_cdk::export::candid::{decode_one, encode_one};
    use ic_cdk::export::candid::{CandidType, Deserialize};

    use crate::task_scheduler::TaskScheduler;
    use crate::types::{Iterations, SchedulingOptions};

    #[derive(CandidType, Deserialize)]
    pub struct TestPayload {
        pub a: bool,
    }

    #[test]
    fn main_flow_works_fine() {
        let mut scheduler = TaskScheduler::default();

        let task_id_1 = scheduler
            .enqueue(
                TestPayload { a: true },
                SchedulingOptions {
                    delay_nano: 10,
                    interval_nano: 10,
                    iterations: Iterations::Exact(1),
                },
                0,
            )
            .ok()
            .unwrap();

        let task_id_2 = scheduler
            .enqueue(
                TestPayload { a: true },
                SchedulingOptions {
                    delay_nano: 10,
                    interval_nano: 10,
                    iterations: Iterations::Infinite,
                },
                0,
            )
            .ok()
            .unwrap();

        let task_id_3 = scheduler
            .enqueue(
                TestPayload { a: false },
                SchedulingOptions {
                    delay_nano: 20,
                    interval_nano: 20,
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

        scheduler.dequeue(task_id_2).unwrap();

        scheduler
            .enqueue(
                TestPayload { a: true },
                SchedulingOptions {
                    delay_nano: 10,
                    interval_nano: 10,
                    iterations: Iterations::Exact(1),
                },
                0,
            )
            .ok()
            .unwrap();
    }

    #[test]
    fn delay_works_fine() {
        let mut scheduler = TaskScheduler::default();

        let task_id_1 = scheduler
            .enqueue(
                TestPayload { a: true },
                SchedulingOptions {
                    delay_nano: 10,
                    interval_nano: 20,
                    iterations: Iterations::Infinite,
                },
                0,
            )
            .ok()
            .unwrap();

        let tasks = scheduler.iterate(5);

        assert!(
            tasks.is_empty(),
            "There shouldn't be any task at this timestamp (5)"
        );

        let tasks = scheduler.iterate(10);
        assert_eq!(
            tasks.len(),
            1,
            "There should be a task that was triggered by a delay at this timestamp (10)"
        );

        let tasks = scheduler.iterate(20);
        assert!(
            tasks.is_empty(),
            "There shouldn't be any task at this timestamp (20)"
        );

        let tasks = scheduler.iterate(30);
        assert_eq!(
            tasks.len(),
            1,
            "There should be a task that was triggered by an interval at this timestamp (30)"
        );

        let tasks = scheduler.iterate(50);
        assert_eq!(
            tasks.len(),
            1,
            "There should be a task that was triggered by an interval at this timestamp (50)"
        );
    }

    #[test]
    fn ser_de_works_fine() {
        let mut scheduler = TaskScheduler::default();

        scheduler
            .enqueue(
                TestPayload { a: true },
                SchedulingOptions {
                    delay_nano: 10,
                    interval_nano: 20,
                    iterations: Iterations::Infinite,
                },
                0,
            )
            .ok()
            .unwrap();

        let bytes = encode_one(scheduler).expect("Should be able to encode task scheduler");
        let mut scheduler: TaskScheduler =
            decode_one(&bytes).expect("Should be able to decode task scheduler");

        let tasks = scheduler.iterate(10);

        assert_eq!(
            tasks.len(),
            1,
            "There should be a task that was triggered by a delay at this timestamp (10)"
        );
    }
}

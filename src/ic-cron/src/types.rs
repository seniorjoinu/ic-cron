use std::cmp::{max, min, Ordering};
use std::collections::BinaryHeap;

use ic_cdk::export::candid::utils::ArgumentEncoder;
use ic_cdk::export::candid::{
    encode_args, CandidType, Deserialize, Principal, Result as CandidResult,
};
use union_utils::{RemoteCallEndpoint, RemoteCallPayload};

pub type TaskId = u64;

#[derive(Clone, CandidType, Deserialize)]
pub enum Iterations {
    Infinite,
    Exact(u64),
}

#[derive(Clone, CandidType, Deserialize)]
pub struct SchedulingInterval {
    pub duration_nano: u64,
    pub iterations: Iterations,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub payload: RemoteCallPayload,
    pub scheduled_at: u64,
    pub rescheduled_at: Option<u64>,
    pub scheduling_interval: SchedulingInterval,
}

impl Task {
    pub fn new<Tuple: ArgumentEncoder>(
        id: TaskId,
        endpoint: RemoteCallEndpoint,
        args: Tuple,
        cycles: u64,
        scheduled_at: u64,
        rescheduled_at: Option<u64>,
        scheduling_interval: SchedulingInterval,
    ) -> CandidResult<Self> {
        let payload = RemoteCallPayload {
            endpoint,
            cycles,
            args_raw: encode_args(args)?,
        };

        Ok(Self {
            id,
            payload,
            scheduled_at,
            rescheduled_at,
            scheduling_interval,
        })
    }
}

pub struct TaskTimestamp {
    pub task_id: TaskId,
    pub timestamp: u64,
}

impl PartialEq for TaskTimestamp {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.eq(&other.timestamp) && self.task_id.eq(&other.task_id)
    }
}

impl Eq for TaskTimestamp {}

impl PartialOrd for TaskTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp
            .partial_cmp(&other.timestamp)
            .map(|it| it.reverse())
    }

    fn lt(&self, other: &Self) -> bool {
        self.timestamp.gt(&other.timestamp)
    }

    fn le(&self, other: &Self) -> bool {
        self.timestamp.ge(&other.timestamp)
    }

    fn gt(&self, other: &Self) -> bool {
        self.timestamp.lt(&other.timestamp)
    }

    fn ge(&self, other: &Self) -> bool {
        self.timestamp.le(&other.timestamp)
    }
}

impl Ord for TaskTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp).reverse()
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        max(self, other)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        min(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        if self.timestamp < max.timestamp {
            max
        } else if self.timestamp > min.timestamp {
            min
        } else {
            self
        }
    }
}

#[derive(Default)]
pub struct TaskExecutionQueue(BinaryHeap<TaskTimestamp>);

impl TaskExecutionQueue {
    pub fn push(&mut self, task: TaskTimestamp) {
        self.0.push(task);
    }

    pub fn pop_ready(&mut self, timestamp: u64) -> Vec<TaskTimestamp> {
        let mut cur = self.0.peek();
        if cur.is_none() {
            return Vec::new();
        }

        let mut result = vec![];

        while cur.unwrap().timestamp <= timestamp {
            result.push(self.0.pop().unwrap());

            cur = self.0.peek();
            if cur.is_none() {
                break;
            }
        }

        result
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

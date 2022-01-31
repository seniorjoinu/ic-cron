use std::cmp::{max, min, Ordering};
use std::collections::BinaryHeap;

use ic_cdk::export::candid::types::{Serializer, Type};
use ic_cdk::export::candid::{
    decode_one, encode_one, CandidType, Deserialize, Result as CandidResult,
};

pub type TaskId = u64;

#[derive(Clone, CandidType, Deserialize)]
pub struct Task {
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, CandidType, Deserialize)]
pub enum Iterations {
    Infinite,
    Exact(u64),
}

#[derive(Clone, Copy, CandidType, Deserialize)]
pub struct SchedulingInterval {
    pub start_at_nano: u64,
    pub interval_nano: u64,
    pub iterations: Iterations,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct ScheduledTask {
    pub id: TaskId,
    pub payload: Task,
    pub scheduled_at: u64,
    pub rescheduled_at: Option<u64>,
    pub scheduling_interval: SchedulingInterval,
    pub delay_passed: bool,
}

impl ScheduledTask {
    pub fn new<TaskPayload: CandidType>(
        id: TaskId,
        payload: TaskPayload,
        scheduled_at: u64,
        rescheduled_at: Option<u64>,
        scheduling_interval: SchedulingInterval,
    ) -> CandidResult<Self> {
        let task = Task {
            data: encode_one(payload).unwrap(),
        };

        Ok(Self {
            id,
            payload: task,
            scheduled_at,
            rescheduled_at,
            scheduling_interval,
            delay_passed: false,
        })
    }

    pub fn get_payload<'a, T>(&'a self) -> CandidResult<T>
    where
        T: Deserialize<'a> + CandidType,
    {
        decode_one(&self.payload.data)
    }
}

#[derive(CandidType, Deserialize, Clone, Copy)]
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

#[derive(Default, Deserialize, Clone)]
pub struct TaskExecutionQueue(BinaryHeap<TaskTimestamp>);

impl TaskExecutionQueue {
    #[inline(always)]
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

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl CandidType for TaskExecutionQueue {
    fn _ty() -> Type {
        Type::Vec(Box::new(TaskTimestamp::_ty()))
    }

    fn ty() -> Type {
        Self::_ty()
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        self.clone().0.into_sorted_vec().idl_serialize(serializer)
    }
}

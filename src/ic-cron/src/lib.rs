use ic_cdk::api::call::call_raw;

use crate::task_scheduler::TaskScheduler;
use crate::types::Task;

pub mod macros;
pub mod task_scheduler;
pub mod types;

#[allow(unused_must_use)]
pub fn exec_cron_task(task: Task) {
    call_raw(
        task.payload.endpoint.canister_id,
        task.payload.endpoint.method_name.as_str(),
        task.payload.args_raw,
        task.payload.cycles,
    );
}

#[derive(Default)]
pub struct Cron {
    pub is_running: bool,
    pub scheduler: TaskScheduler,
}

#[macro_export]
macro_rules! implement_cron {
    () => {
        pub static mut _CRON_STATE: Option<ic_cron::task_scheduler::TaskScheduler> = None;

        pub fn get_cron_state() -> &'static mut ic_cron::task_scheduler::TaskScheduler {
            unsafe {
                match _CRON_STATE.as_mut() {
                    Some(cron) => cron,
                    None => {
                        _put_cron_state(Some(ic_cron::task_scheduler::TaskScheduler::default()));
                        get_cron_state()
                    }
                }
            }
        }

        pub fn _take_cron_state() -> Option<ic_cron::task_scheduler::TaskScheduler> {
            unsafe { _CRON_STATE.take() }
        }

        pub fn _put_cron_state(state: Option<ic_cron::task_scheduler::TaskScheduler>) {
            unsafe {
                _CRON_STATE = state;
            }
        }

        pub fn cron_enqueue<Payload: ic_cdk::export::candid::CandidType>(
            payload: Payload,
            scheduling_options: ic_cron::types::SchedulingOptions,
        ) -> ic_cdk::export::candid::Result<ic_cron::types::TaskId> {
            let cron = get_cron_state();

            let id = cron.enqueue(payload, scheduling_options, ic_cdk::api::time())?;

            Ok(id)
        }

        pub fn cron_dequeue(
            task_id: ic_cron::types::TaskId,
        ) -> Option<ic_cron::types::ScheduledTask> {
            get_cron_state().dequeue(task_id)
        }

        pub fn cron_ready_tasks() -> Vec<ic_cron::types::ScheduledTask> {
            get_cron_state().iterate(ic_cdk::api::time())
        }
    };
}

#[cfg(test)]
mod tests {
    use crate as ic_cron;
    use crate::implement_cron;
    use crate::task_scheduler::TaskScheduler;
    use ic_cdk::storage::{stable_restore, stable_save};
    use ic_cdk_macros::{heartbeat, post_upgrade, pre_upgrade};

    implement_cron!();

    #[pre_upgrade]
    fn pre_upgrade_hook() {
        let cron_state = _take_cron_state();

        stable_save((cron_state,)).expect("Unable to save the state to stable memory");
    }

    #[post_upgrade]
    fn post_upgrade_hook() {
        let (cron_state,): (Option<TaskScheduler>,) =
            stable_restore().expect("Unable to restore the state from stable memory");

        _put_cron_state(cron_state);
    }

    #[heartbeat]
    fn tick() {
        let tasks = cron_ready_tasks();
    }

    #[test]
    fn no_op() {
        assert!(true);
    }
}

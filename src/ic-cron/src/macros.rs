#[macro_export]
macro_rules! implement_cron {
    () => {
        static mut CRON: Option<ic_cron::Cron> = None;

        pub fn get_cron_state() -> &'static mut ic_cron::Cron {
            unsafe {
                match CRON.as_mut() {
                    Some(cron) => cron,
                    None => {
                        CRON = Some(ic_cron::Cron::default());
                        get_cron_state()
                    }
                }
            }
        }

        pub fn cron_enqueue<Tuple: ic_cdk::export::candid::utils::ArgumentEncoder>(
            endpoint: union_utils::RemoteCallEndpoint,
            args: Tuple,
            cycles: u64,
            scheduling_interval: ic_cron::types::SchedulingInterval,
        ) -> ic_cdk::export::candid::Result<ic_cron::types::TaskId> {
            let cron = get_cron_state();
            let task = cron.scheduler.enqueue(
                endpoint,
                args,
                cycles,
                scheduling_interval,
                ic_cdk::api::time(),
            );

            if !cron.is_running {
                cron.is_running = true;

                _call_cron_pulse();
            }

            task
        }

        pub fn cron_dequeue(task_id: ic_cron::types::TaskId) -> Option<ic_cron::types::Task> {
            get_cron_state().scheduler.dequeue(task_id)
        }

        #[allow(unused_must_use)]
        pub fn _call_cron_pulse() {
            if get_cron_state().is_running {
                ic_cdk::call::<(), ()>(ic_cdk::id(), "_cron_pulse", ());
            }
        }

        #[allow(unused_must_use)]
        #[ic_cdk_macros::update]
        fn _cron_pulse() {
            let cron = get_cron_state();

            cron.scheduler
                .iterate(ic_cdk::api::time())
                .into_iter()
                .for_each(ic_cron::exec_cron_task);

            if cron.scheduler.is_empty() {
                cron.is_running = false;
            }

            _call_cron_pulse();
        }
    };
}

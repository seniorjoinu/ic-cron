#[macro_export]
macro_rules! implement_cron {
    () => {
        static mut CRON: Option<ic_cron::types::Cron> = None;

        #[inline(always)]
        pub fn get_cron_state() -> &'static mut ic_cron::types::Cron {
            unsafe {
                match CRON.as_mut() {
                    Some(cron) => cron,
                    None => {
                        CRON = Some(ic_cron::types::Cron::default());
                        get_cron_state()
                    }
                }
            }
        }

        pub fn cron_enqueue<Payload: ic_cdk::export::candid::CandidType>(
            kind: u8,
            payload: Payload,
            scheduling_interval: ic_cron::types::SchedulingInterval,
        ) -> ic_cdk::export::candid::Result<ic_cron::types::TaskId> {
            let cron = get_cron_state();
            let task =
                cron.scheduler
                    .enqueue(kind, payload, scheduling_interval, ic_cdk::api::time());

            if !cron.is_running {
                cron.is_running = true;

                _call_cron_pulse();
            }

            task
        }

        #[inline(always)]
        pub fn cron_dequeue(
            task_id: ic_cron::types::TaskId,
        ) -> Option<ic_cron::types::ScheduledTask> {
            get_cron_state().scheduler.dequeue(task_id)
        }

        #[allow(unused_must_use)]
        #[inline(always)]
        pub fn _call_cron_pulse() {
            if get_cron_state().is_running {
                ic_cdk::block_on(async {
                    ic_cdk::call::<(), ()>(ic_cdk::id(), "_cron_pulse", ()).await;
                });
            };
        }

        #[ic_cdk_macros::update]
        fn _cron_pulse() {
            union_utils::log("ic_cron._cron_pulse()");

            let cron = get_cron_state();

            cron.scheduler
                .iterate(ic_cdk::api::time())
                .into_iter()
                .for_each(_cron_task_handler);

            if cron.scheduler.is_empty() {
                cron.is_running = false;
            }

            _call_cron_pulse();
        }
    };
}

#[macro_export]
macro_rules! u8_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl std::convert::TryFrom<u8> for $name {
            type Error = ();

            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as u8 => Ok($name::$vname),)*
                    _ => Err(()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as ic_cron;
    use crate::implement_cron;
    use crate::types::ScheduledTask;

    fn _cron_task_handler(task: ScheduledTask) {
        match task.get_kind() {
            0u8 => {}
            1u8 => {}
            _ => {}
        }
    }

    implement_cron!();
}

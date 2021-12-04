#[macro_export]
macro_rules! implement_cron {
    () => {
        static mut CRON: Option<ic_cron::task_scheduler::TaskScheduler> = None;

        pub fn get_cron_state() -> &'static mut ic_cron::task_scheduler::TaskScheduler {
            unsafe {
                match CRON.as_mut() {
                    Some(cron) => cron,
                    None => {
                        CRON = Some(ic_cron::task_scheduler::TaskScheduler::default());
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

            let id = cron.enqueue(kind, payload, scheduling_interval, ic_cdk::api::time())?;

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
    use core::convert::TryInto;

    implement_cron!();

    u8_enum! {
        pub enum HanlderKind {
            First,
            Second,
        }
    }

    #[export_name = "canister_heartbeat"]
    fn heartbeat() {
        for task in cron_ready_tasks() {
            match task.get_kind().try_into() {
                Ok(HanlderKind::First) => {}
                Ok(HanlderKind::Second) => {}
                Err(_) => {}
            }
        }
    }
}

use std::convert::TryInto;

use ic_cdk::export::candid::export_service;
use ic_cdk::trap;
use ic_cdk_macros::{query, update};
use union_utils::log;

use ic_cron::types::{Iterations, SchedulingInterval, TaskId};
use ic_cron::{implement_cron, u8_enum};

// ------------- MAIN LOGIC -------------------

#[derive(Default)]
pub struct AutomaticCounter {
    pub counter_1: u64,
    pub counter_1_started: bool,

    pub counter_2: u64,
    pub counter_2_started: bool,
}

u8_enum! {
    pub enum CronTaskKind {
        One,
        Two,
    }
}

#[update]
fn start_counter_1(duration_nano: u64) -> TaskId {
    log("Start counter 1");

    let state = get_state();

    if state.counter_1_started {
        trap("Counter 1 already started");
    }

    let res = cron_enqueue(
        CronTaskKind::One as u8,
        String::from("Hello from task 1!"),
        SchedulingInterval {
            duration_nano,
            iterations: Iterations::Infinite,
        },
    );

    state.counter_1_started = true;

    res.unwrap()
}

#[query]
fn get_counter_1() -> u64 {
    get_state().counter_1
}

#[update]
fn start_counter_2(duration_nano: u64, step: u64) -> TaskId {
    log("Start counter 1");

    let state = get_state();

    if state.counter_2_started {
        trap("Counter 2 already started");
    }

    let res = cron_enqueue(
        CronTaskKind::Two as u8,
        step,
        SchedulingInterval {
            duration_nano,
            iterations: Iterations::Infinite,
        },
    );

    state.counter_2_started = true;

    res.unwrap()
}

#[query]
fn get_counter_2() -> u64 {
    get_state().counter_2
}

// --------------- RECURRENCE ------------------

implement_cron!();

#[export_name = "canister_heartbeat"]
fn heartbeat() {
    for task in cron_ready_tasks() {
        match task.get_kind().try_into() {
            Ok(CronTaskKind::One) => {
                let message = task.get_payload::<String>().unwrap();

                log(format!("Task One executed: {}", message.as_str()).as_str());

                get_state().counter_1 += 1;
            }
            Ok(CronTaskKind::Two) => {
                log("Task Two executed");

                let step = task.get_payload::<u64>().unwrap();

                get_state().counter_2 += step;
            }
            Err(_) => log("Invalid cron task handler"),
        }
    }
}

// ------------------ STATE ----------------------

static mut STATE: Option<AutomaticCounter> = None;

pub fn get_state() -> &'static mut AutomaticCounter {
    unsafe {
        match STATE.as_mut() {
            Some(s) => s,
            None => {
                STATE = Some(AutomaticCounter::default());
                get_state()
            }
        }
    }
}

// ---------------- CANDID -----------------------

export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}

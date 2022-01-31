use ic_cdk::export::candid::{export_service, CandidType, Deserialize};
use ic_cdk::trap;
use ic_cdk_macros::{heartbeat, init, query, update};

use ic_cron::implement_cron;
use ic_cron::types::{Iterations, SchedulingInterval, TaskId};

// ------------- MAIN LOGIC -------------------

#[derive(Default)]
pub struct AutomaticCounter {
    pub counter_1: u64,
    pub counter_1_started: bool,

    pub counter_2: u64,
    pub counter_2_started: bool,
}

#[derive(CandidType, Deserialize)]
pub enum CronTaskKind {
    One(String),
    Two(u64),
}

#[update]
fn start_counter_1(duration_nano: u64) -> TaskId {
    ic_cdk::print("Start counter 1");

    let state = get_state();

    if state.counter_1_started {
        trap("Counter 1 already started");
    }

    let res = cron_enqueue(
        CronTaskKind::One(String::from("Hello from task 1!")),
        SchedulingInterval {
            start_at_nano: duration_nano,
            interval_nano: duration_nano,
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
    ic_cdk::print("Start counter 2");

    let state = get_state();

    if state.counter_2_started {
        trap("Counter 2 already started");
    }

    let res = cron_enqueue(
        CronTaskKind::Two(step),
        SchedulingInterval {
            start_at_nano: duration_nano,
            interval_nano: duration_nano,
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

#[init]
fn init() {
    ic_cdk::print("INIT");
}

#[heartbeat]
fn tick() {
    for task in cron_ready_tasks() {
        let kind = task
            .get_payload::<CronTaskKind>()
            .expect("Unable to deserialize cron task kind");

        match kind {
            CronTaskKind::One(message) => {
                ic_cdk::print(format!("Task One executed: {}", message.as_str()).as_str());

                get_state().counter_1 += 1;
            }
            CronTaskKind::Two(step) => {
                ic_cdk::print("Task Two executed");

                get_state().counter_2 += step;
            }
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

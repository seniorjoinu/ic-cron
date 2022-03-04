## IC Cron

Task scheduler rust library for the Internet Computer

### Motivation

The IC provides built-in "heartbeat" functionality which is basically a special function that gets executed each time
consensus ticks. But this is not enough for a comprehensive task scheduling - you still have to implement scheduling
logic by yourself. This rust library does exactly that - provides you with simple APIs for complex background scheduling 
scenarios to execute your code at any specific time, as many times as you want.

### Installation

Make sure you're using `dfx 0.8.4` or higher.

```toml
# Cargo.toml

[dependencies]
ic-cron = "0.6"
```

### Usage

```rust
// somewhere in your canister's code
ic_cron::implement_cron!();

#[derive(CandidType, Deserialize)]
enum TaskKind {
    SendGoodMorning(String),
    DoSomethingElse,
}

// enqueue a task
#[ic_cdk_macros::update]
pub fn enqueue_task_1() {
    cron_enqueue(
        // set a task payload - any CandidType is supported
        TaskKind::SendGoodMorning(String::from("sweetie")),
        // set a scheduling interval (how often and how many times to execute)
        ic_cron::types::SchedulingOptions {
            1_000_000_000 * 60 * 5, // after waiting for 5 minutes delay once
            1_000_000_000 * 10, // each 10 seconds
            iterations: Iterations::Exact(20), // until executed 20 times
        },
    );
}

// enqueue another task
#[ic_cdk_macros::update]
pub fn enqueue_task_2() {
    cron_enqueue(
        TaskKind::DoSomethingElse,
        ic_cron::types::SchedulingOptions {
            0, // start immediately
            1_000_000_000 * 60 * 5, // each 5 minutes
            iterations: Iterations::Infinite, // repeat infinitely
        },
    );
}

// in a canister heartbeat function get all tasks ready for execution at this exact moment and use it
#[ic_cdk_macros::heartbeat]
fn heartbeat() {
    // cron_ready_tasks will only return tasks which should be executed right now
    for task in cron_ready_tasks() {
        let kind = task.get_payload::<TaskKind>().expect("Serialization error");
      
        match kind {
            TaskKind::SendGoodMorning(name) => {
                // will print "Good morning, sweetie!"      
                println!("Good morning, {}!", name);
            },
            TaskKind::DoSomethingElse => {
                ...
            },
        };   
    }
}
```

### How many cycles does it consume?

Since this library is just a fancy task queue, there is no significant overhead in terms of cycles.

## How does it work?

This library uses built-in canister heartbeat functionality. Each time you enqueue a task it gets added to the task
queue. Tasks could be scheduled in different ways - they can be executed some exact number of times or infinitely. It is
very similar to how you use `setTimeout()` and `setInterval()` in javascript, but more flexible. Each
time `canister_heartbeat` function is called, you have to call `cron_ready_tasks()` function which efficiently iterates
over the task queue and pops tasks which scheduled execution timestamp is <= current timestamp. Rescheduled tasks get
their next execution timestamp relative to their previous planned execution timestamp - this way the scheduler
compensates an error caused by unstable consensus intervals.

## Limitations

Since `ic-cron` can't pulse faster than the consensus ticks, it has an error of ~2s. 

## Tutorials
* [Introduction To ic-cron Library](https://dev.to/seniorjoinu/introduction-to-ic-cron-library-17g1)
* [Extending Sonic With Limit Orders Using ic-cron Library](https://hackernoon.com/tutorial-extending-sonic-with-limit-orders-using-ic-cron-library)
* [How to Execute Background Tasks on Particular Weekdays with IC-Cron and Chrono](https://hackernoon.com/how-to-execute-background-tasks-on-particular-weekdays-with-ic-cron-and-chrono)
* [How To Build A Token With Recurrent Payments On The Internet Computer Using ic-cron Library](https://dev.to/seniorjoinu/tutorial-how-to-build-a-token-with-recurrent-payments-on-the-internet-computer-using-ic-cron-library-3l2h)

## API

See the [example](./example) project for better understanding.

### implement_cron!()

This macro will implement all the functions you will use: `get_cron_state()`, `cron_enqueue()`, `cron_dequeue()`
and `cron_ready_tasks()`.

Basically, this macro implements an inheritance pattern. Just like in a regular object-oriented programming language.
Check the [source code](ic-cron-rs/src/macros.rs) for further info.

### cron_enqueue()

Schedules a new task. Returns task id, which then can be used in `cron_dequeue()` to de-schedule the task.

Params:

* `payload: CandidType` - the data you want to provide with the task
* `scheduling_interval: SchedulingInterval` - how often your task should be executed and how many times it should be
  rescheduled

Returns:

* `ic_cdk::export::candid::Result<u64>` - `Ok(task id)` if everything is fine, and `Err` if there is a serialization
  issue with your `payload`

### cron_dequeue()

Deschedules the task, removing it from the queue.

Params:

* `task_id: u64` - an id of the task you want to delete from the queue

Returns:

* `Option<ScheduledTask>` - `Some(task)`, if the operation was a success; `None`, if there was no such task.

### cron_ready_tasks()

Returns a vec of tasks ready to be executed right now.

Returns:

* `Vec<ScheduledTask>` - vec of tasks to handle

### get_cron_state()

Returns a static mutable reference to object which can be used to observe scheduler's state and modify it. Mostly 
intended for advanced users who want to extend `ic-cron`. See the [source code](ic-cron-rs/src/task_scheduler.rs) for 
further info.

### set_cron_state()

Sets the global state of the task scheduler, so this new state is accessible from `get_cron_state()` function.

Params:

* `TaskScheduler` - state object you can get from `get_cron_state()` function

These two functions could be used to persist scheduled tasks between canister upgrades:
```rust
#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade_hook() {
    let cron_state = get_cron_state().clone();

    stable_save((cron_state,)).expect("Unable to save the state to stable memory");
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade_hook() {
    let (cron_state,): (TaskScheduler,) =
          stable_restore().expect("Unable to restore the state from stable memory");

    set_cron_state(cron_state);
}
```

## Candid

You don't need to modify your `.did` file for this library to work.

## Contribution

You can reach me out here on Github opening an issue, or you could start a thread on Dfinity developer forum.

You're also welcome to suggest new features and open PR's.

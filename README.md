## IC Cron

Task scheduler rust library for the Internet Computer

### Motivation

The IC provides built-in "heartbeat" functionality which is basically a special function that gets executed each time
consensus ticks. But this is not enough for a comprehensive task scheduling - you still have to implement scheduling
logic by yourself. This rust library does exactly that.

Moreover, this library can help optimize scenarios when your canister sends a lot of messages, when there are more than
one message per single recipient. In that case you could, instead of sending them all immediately, store them in some
buffer and schedule an `ic-cron` task to batch-send them, once the task is executed, consuming only one message per
block, per unique recipient.

### Installation

```toml
# Cargo.toml

[dependencies]
ic-cron = "0.4"
```

### Usage

```rust
// somewhere in your canister's code

ic_cron::implement_cron!();

// this step is optional - you can use simple u8's to differ between task handlers
ic_cron::u8_enum! {
    pub enum TaskKind {
        SendGoodMorning,
        TransferTokens,
    }
}

// in a canister heartbeat function get all tasks ready for execution at this exact moment and use it
#[export_name = "canister_heartbeat"]
fn heartbeat() {
    for task in cron_ready_tasks() {
        match task.get_kind().try_into() {
            Ok(TaskKind::SendGoodMorning) => {
                let name = task.get_payload::<String>().unwrap();
          
                // will print "Good morning, sweetie!"      
                say(format!("Good morning, {}!", name));
            },
            Ok(TaskKind::TransferTokens) => {
                ...
            },
            Err(e) => trap(e),
        };   
    }
}

...

// inside any #[update] function
// enqueue a task
cron_enqueue(
    // set a task kind so later you could decide how to handle it's execution
    TaskKind::SendGoodMorning as u8,
    // set a task payload - any CandidType is supported, so custom types would also work fine
    String::from("sweetie"), 
    // set a scheduling interval (how often and how many times to execute)
    ic_cron::types::SchedulingInterval {
        1_000_000_000 * 10, // each 10 seconds
        iterations: Iterations::Exact(20), // until executed 20 times
    },
);
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

1. Right now `ic-cron` doesn't support canister upgrades, so all your queued tasks will be lost. This is due to a
limitation in `ic-cdk`, which doesn't support multiple stable variables at this moment. Once they do, I'll update this
library, so it will handle canister upgrades gracefully.
If you really want this functionality right now, you may try to serialize the state manually using `get_cron_state()`
function.

2. Since `ic-cron` can't pulse faster than the consensus ticks, it has an error of ~2s. So make sure you're not using a
`duration_nano` interval less than 3s, otherwise it won't work as expected.

## API

See the [example](./example) project for better understanding.

### implement_cron!()

This macro will implement all the functions you will use: `get_cron_state()`, `cron_enqueue()`, `cron_dequeue()`
and `cron_ready_tasks()`.

Basically this macros implements an inheritance pattern. Just like in a regular object-oriented programming language.
Check the [source code](ic-cron-rs/src/macros.rs) for further info.

### cron_enqueue()

Schedules a new task. Returns task id, which then can be used in `cron_dequeue()` to de-schedule the task.

Params:

* `kind: u8` - used to differentiate the way you want to process this task once it's executed
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

### u8_enum!()

Helper macro which will automatically derive `TryInto<u8>` for your enum.

### get_cron_state()

Returns an object which can be used to observe scheduler's state and modify it. Mostly intended for advanced users who
want to extend `ic-cron`. See the [source code](ic-cron-rs/src/task_scheduler.rs) for further info.

## Candid

You don't need to modify your `.did` file for this library to work.

## Contribution

You can reach me out here on github opening an issue, or you could start a thread on dfinity's developer forum.

You're also welcome to suggest new features and open PR's.

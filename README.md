## IC Cron

Makes your IC canister proactive

### Abstract

Canisters are reactive by their nature - they only do something when they're asked by a client or another canister. But
what if you need your canister to do something automatically after some time passes? For example, what if you want your
canister to transfer some tokens on your behalf each month? Or maybe you want your canister to send you a "Good
morning!"
message through OpenChat each morning?

The only way to achieve such a behaviour before was to introduce an off-chain component, that will wait the time you
need and then call canister's functions you want. This component could be either an edge device (such as user's
smartphone) or some centralized cloud instance like AWS.

**But not anymore.** With `ic-cron` you can do all of this stuff completely on-chain for a reasonable price. No more
centralized "clock-bots", no more complex Uniswap-like patterns when each user helps with recurrent task execution. Just
schedule a task and your good to go.

And it is just a rust library.

### Installation

```toml
# Cargo.toml

[dependencies]
ic-cron = "0.2.5"
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

// define a task handler function
// it will be automatically invoked each time a task is ready to be executed
fn _cron_task_handler(task: ScheduledTask) {
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

### How this repository is organized

There are three directories:

* [ic-cron](./ic-cron) - the library code itself
* [example](./example) - example auto-counter canister which can help you understand how to use this library in your
  project
* [example-e2e-test](./example-e2e-test) - ts-mocha test that proves it is all real

I strongly suggest you to visit all of them for better understanding.

### How many cycles does it consume?

I did not run any benchmarking at this moment, but it is pretty efficient. Simple math says it should add around **
$2/mo** overhead considering your canister always having a scheduled task in queue. If the scheduling is eventual (
sometimes you have a pending task, sometimes you don't) - it should consume even less.

> Q: Does complexity of my tasks adds another overhead to cycles consumption?
>
> A: No! You only pay for what you've coded. No additional cycles are wasted.

> Q: What if I have multiple canisters each of which needs this behaviour?
>
> A: In this case you can encapsulate ic-cron into a single separate clock-canister and ask it to schedule
> tasks for your other canisters.

## Limitations

Right now `ic-cron` doesn't support canister upgrades, so all your queued tasks will be lost. This is due to a
limitation in `ic-cdk`, which doesn't support multiple stable variables at this moment. Once they do, I'll update this
library, so it will handle canister upgrades gracefully.

If you really want this functionality right now, you may try to serialize the state manually using `get_cron_state()`
function.

## How does it work?

It is pretty simple. It abuses the IC's messaging mechanics so your canister starts sending a wake-up message to itself.
Once this message is received, it checks a list of tasks if there are any of them which could be executed at this exact
moment. If there are some, it passes them to the `_cron_task_handler()` function one by one, and then sends the special
message once again. If no more enqueued tasks left, it stops sending the message. Once a new task is enqueued, it starts
to send the message again.

So, basically it uses a weird infinite loop to eventually wake the canister up to do some work.

## API

See the [example](./example) project for better understanding.

### implement_cron!()

This macro will implement all the functions you will use: `get_cron_state()`, `cron_enqueue()`, `cron_dequeue()` as well
as a new `#[update]` endpoint for your canister - `_cron_pulse()`, on which your canister will send the wake-up message.

Basically this macros implements an inheritance pattern. Just like in a regular object-oriented programming language.
Check the [source code](./ic-cron/src/macros.rs) for further info.

### cron_enqueue()

Schedules a new task. Returns task id, which then can be used in `cron_dequeue()` to deschedule the task.

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

### u8_enum!()

Helper macro which will automatically derive `TryInto<u8>` for your enum.

### get_cron_state()

Returns an object which can be used to observe scheduler's state and modify it. Mostly intended for advanced users who
want to implement custom logic on top of `ic-cron`. See the [source code](./ic-cron/src/task_scheduler.rs) for further
info.

## Candid

You don't need to modify your `.did` file for this library to work.

## Contribution

You can reach me out here on github opening an issue, or you could start a thread on dfinity's developer forum.

You're also welcome to suggest new features and open PR's.

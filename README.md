## IC Cron

Makes your canister pro-active

```rust
ic_cron::implement_cron!();

...

// in any function excluding #[init]
cron_enqueue(...);
```
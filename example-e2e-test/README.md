### E2E test for ic-cron

* make sure you have `rust` installed
* make sure `wasm` target (`rustup target add wasm32-unknown-unknown`) and `ic-cdk-optimizer` (`cargo install ic-cdk-optimizer`) are also installed
* `yarn install` - install dependencies
* `yarn start` - start a replica in a separate terminal
* `yarn test` - start the test
* observe replicas logs

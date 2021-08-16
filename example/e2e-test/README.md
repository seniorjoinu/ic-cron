### E2E test for ic-cron

This directory contains tests for automatic counter canister located at [example directory](..)

#### Requirements

* `rust`
* `wasm32-unknown-unknown` target
* `dfx 0.8.0`
* `ic-cdk-optimizer`

#### Local development

* `yarn install` - install dependencies
* `yarn start` - start a replica in a separate terminal
* `yarn test` - start the test
* observe replicas logs
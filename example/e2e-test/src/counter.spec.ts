import {delay, getSecsNano, ISetup, setup} from "./utils";
import {Ed25519KeyIdentity} from "@dfinity/identity";
import {assert} from 'chai';
import {deployExample} from "./deploy";

describe('automatic counter', () => {
    let counter: ISetup;

    before(async () => {
        await deployExample();

        counter = await setup(Ed25519KeyIdentity.generate());

        await counter.agent.fetchRootKey();
    });

    // this test checks for canister 'ic-cron-example' to work nicely
    // it may fail sometimes for the first counter, since consensus may take more time, than the counter ticks
    it("flow works fine", async () => {
        // checking that before any interactions counters are set to zero
        let c1Before = await counter.counterClient.get_counter_1();
        let c2Before = await counter.counterClient.get_counter_2();

        assert.equal(c1Before, 0n);
        assert.equal(c2Before, 0n);

        // start a counter 1 (increments by 1 each 3s) and waiting for some ticks to pass
        await Promise.all([
            counter.counterClient.start_counter_1(getSecsNano(3)),

            // waiting a little bit more than we need, since consensus may be slow
            delay(1000 * 13),
        ]);

        // checking that the value of counter 1 was incremented exactly how we expect
        const c1After = await counter.counterClient.get_counter_1();
        assert.equal(c1After, 4n);

        // start a counter 2 (increments by 10 each 10s) and waiting for some ticks to pass
        await Promise.all([
            counter.counterClient.start_counter_2(getSecsNano(10), 10n),

            // waiting a little bit more than we need, since consensus may be slow
            delay(1000 * 21),
        ]);

        // checking that the value of counter 2 was incremented exactly how we expect
        const c2After = await counter.counterClient.get_counter_2();
        assert.equal(c2After, 20n);
    });
});
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

    it("flow works fine", async () => {
        let c1Before = await counter.counterClient.get_counter_1();
        let c2Before = await counter.counterClient.get_counter_2();

        assert.equal(c1Before, 0n);
        assert.equal(c2Before, 0n);

        await counter.counterClient.start_counter_1(getSecsNano(5));
        await delay(1000 * 10);

        const c1After = await counter.counterClient.get_counter_1();
        assert.equal(c1After, 2n);

        await counter.counterClient.start_counter_2(getSecsNano(10), 10n);
        await delay(1000 * 20);

        const c2After = await counter.counterClient.get_counter_2();
        assert.equal(c2After, 20n);
    });
});
import {Actor, HttpAgent, Identity} from "@dfinity/agent";
import fetch from 'node-fetch';
import {exec} from 'child_process';
import {expect} from "chai";

import ICounterClient from 'dfx-type/ic-cron-example/ic-cron-example';


export interface ISetup {
    agent: HttpAgent;
    counterClient: ICounterClient;
}

export async function setup(identity: Identity): Promise<ISetup> {
    const agent = new HttpAgent({
        host: 'http://localhost:8000/',
        // @ts-ignore
        fetch,
        identity
    });

    const {canisterId, idlFactory} = await import('dfx/ic-cron-example/ic-cron-example');

    const client: ICounterClient = Actor.createActor(idlFactory, {
        agent,
        canisterId
    });

    return {
        agent,
        counterClient: client
    };
}

export function getTimeNano(): bigint {
    return BigInt(new Date().getTime() * 1000_000)
}

export function getHoursNano(h: number): bigint {
    return BigInt(1000_000_000 * 60 * 60 * h);
}

export function getSecsNano(s: number): bigint {
    return BigInt(1000_000_000 * s);
}

export function getMinsNano(m: number): bigint {
    return BigInt(1000_000_000 * 60 * m);
}

export async function execAsync(command: string) {
    return new Promise((res, rej) => {
        exec(command, (err, stderr, stdout) => {
            if (err) {
                rej(err);
            } else if (stderr) {
                rej(stderr);
            } else if (stdout) {
                res(stdout);
            } else {
                res("No error");
            }
        })
    })
}

export const expectThrowsAsync = async (method: Promise<any>, errorMessage?: string) => {
    let error = null
    try {
        await method
    } catch (err) {
        error = err
    }

    expect(error).to.be.an('Error', errorMessage);
}

export async function delay(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}
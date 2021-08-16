import {execAsync} from "./utils";

export async function deployExample() {
    const command = `dfx deploy ic-cron-example`;

    console.log(command);
    console.log(await execAsync(command));
}
import { createActor, canisterId } from 'declarations/{project_name}_backend';
import { building } from '$app/environment';

function dummyActor() {
    return new Proxy({}, { get() { throw new Error("Canister invoked while building"); } });
}

export const backend = building ? dummyActor() : createActor(canisterId);

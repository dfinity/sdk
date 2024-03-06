import { createActor, canisterId } from 'declarations/{backend_name}';
import { building } from '$app/environment';

function dummyActor() {
    return new Proxy({}, { get() { throw new Error("Canister invoked while building"); } });
}

const buildingOrTesting = building || process.env.NODE_ENV === "test";

export const backend = buildingOrTesting
    ? dummyActor()
    : createActor(canisterId);

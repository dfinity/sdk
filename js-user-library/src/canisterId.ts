import BigNumber from "bignumber.js";
import { Hex } from "./hex";

// Canister IDs are represented as u64 in the HTTP handler of the client.
export type CanisterId = BigNumber & { __canisterID__: void };

export const fromHex = (hex: Hex): CanisterId => {
  return new BigNumber(`0x${hex}`) as CanisterId;
};

export const toHex = (id: CanisterId): Hex => {
  return id.toString(16) as Hex;
};

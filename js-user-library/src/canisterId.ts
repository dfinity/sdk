import { Hex } from "./hex";
import { Int } from "./int";
import * as int from "./int";

// FIXME
// The current implementation of the client expects canister IDs to be
// represented as u64. This `Int` type will not be sufficient for u64 but may be
// good enough for now.

// export type CanisterId = Buffer & { __canisterId__: void };
export type CanisterId = Int & { __canisterId__: void };

// export const fromHex = (hex: Hex): CanisterId => {
//   return buffer.fromHex(hex) as CanisterId;
// };
export const fromHex = (hex: Hex): CanisterId => int.fromHex(hex) as CanisterId;

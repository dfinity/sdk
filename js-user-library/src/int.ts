import { Hex } from "./hex";

export type Int = number & { __int__: void };

export const fromHex = (hex: Hex) => parseInt(hex, 16);

export const toHex = (int: Int): Hex => int.toString(16) as Hex;

import { Buffer } from "safe-buffer";
import { Text } from "./Text";
import { Type } from "./Type";

export class TypeTable {
  private typs: Array<Buffer> = [];
  private idx: Map<Type, number> = new Map();

  hasType(obj: Type) {
    return this.idx.has(obj);
  }

  addType(obj: Type, buf: Buffer) {
    if (this.hasType(obj)) {
      throw new Error(`duplicate type name: ${obj}`);
    }
    const idx = this.typs.length;
    this.idx.set(obj, idx);
    this.typs.push(buf);
  }

  mergeType(obj: Type, knot: Type) {
    if (!this.hasType(obj)) {
      throw new Error(`Missing type index for ${obj}`);
    }
    if (!this.hasType(knot)) {
      throw new Error(`Missing type index for ${knot}`);
    }
    const idx = this.idx.get(obj);
    const knot_idx = this.idx.get(knot);
    this.typs[idx] = this.typs[knot_idx];
    this.typs.splice(knot_idx, 1);
    this.idx.delete(knot);
  }

  getTypeIdx(obj: Type): Buffer {
    if (!this.hasType(obj)) {
      throw new Error(`Missing type index for ${obj}`);
    }
    return sleb.encode(this.idx.get(obj));
  }

  encodeTable(): Buffer {
    const len = leb.encode(this.typs.length);
    const buf = Buffer.concat(this.typs);
    return Buffer.concat([len, buf]);
  }
};

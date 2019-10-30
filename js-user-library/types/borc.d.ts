declare module "borc" {
  class Decoder {
    constructor(opts: {
      size: Number,
      tags: Record<number, (val: any) => any>,
    })

    decodeFirst(input: ArrayBuffer): any
  }

  export function encode(o: any): ArrayBuffer

  class Tagged {
    constructor(tag: Number, value: any)
  }
}

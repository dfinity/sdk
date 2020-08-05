declare module 'base32.js' {
  type Ret = {
    finalize: () => any;
  };

  interface DecoderConfig {
    type?: 'rfc4648' | 'crockford' | 'base32hex';
    alphabet?: string;
    lc?: boolean;
  }
  class Decoder {
    constructor(options?: DecoderConfig);
    write(str: string): this;
    finalize(str?: string): ArrayBuffer;
  }
  class Encoder {
    private buf: ArrayBuffer;
    private charmap: { [key: number]: number };
    constructor(options?: DecoderConfig);
    write(buf: ArrayBuffer): this;
    finalize(str?: ArrayBuffer): string;
  }
}

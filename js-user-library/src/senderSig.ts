import { BinaryBlob } from './blob';

export type SenderSig = BinaryBlob & { __senderSig__: void };

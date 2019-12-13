import { BinaryBlob } from './blob';

export type SenderPubKey = BinaryBlob & { __senderPubKey__: void };

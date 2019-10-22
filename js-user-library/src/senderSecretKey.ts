import { BinaryBlob } from "./blob";

export type SenderSecretKey = BinaryBlob & { __senderSecretKey__: void };

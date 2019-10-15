import tweetnacl from "tweetnacl";
import { RequestId } from "./requestId";
import { SenderSecretKey } from "./senderSecretKey";
import { SenderSig } from "./senderSig";

export const sign = (
  secretKey: SenderSecretKey,
) => (
  requestId: RequestId,
): SenderSig => {
  return tweetnacl.sign(requestId, secretKey) as SenderSig;
};

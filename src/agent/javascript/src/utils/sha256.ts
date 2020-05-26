import { BinaryBlob, blobFromUint8Array } from '../types';

export async function sha256(data: ArrayBuffer): Promise<BinaryBlob> {
  const digest = await crypto.subtle.digest({ name: 'SHA-256' }, new Uint8Array(data));
  return blobFromUint8Array(new Uint8Array(digest));
}

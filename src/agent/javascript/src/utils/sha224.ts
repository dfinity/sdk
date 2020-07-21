import { sha224 as jsSha224 } from 'js-sha256';
import { BinaryBlob, blobFromUint8Array } from '../types';

export function sha224(data: ArrayBuffer): BinaryBlob {
  const shaObj = jsSha224.create();
  shaObj.update(data);
  return blobFromUint8Array(new Uint8Array(shaObj.array()));
}

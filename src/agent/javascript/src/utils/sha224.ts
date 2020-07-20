import jsSHA from 'jssha';
import { BinaryBlob, blobFromUint8Array } from '../types';

export function sha224(data: ArrayBuffer): BinaryBlob {
  const shaObj = new jsSHA('SHA-224', 'ARRAYBUFFER');
  shaObj.update(data);
  return blobFromUint8Array(shaObj.getHash('UINT8ARRAY'));
}

import { crc32 } from 'crc';

export function getCrc32(hex: string): number {
  return crc32(Buffer.from(hex, 'hex'));
}

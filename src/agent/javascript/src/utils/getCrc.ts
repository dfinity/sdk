import { crc8 } from 'crc';
export function getCrc(hex: string): string {
  return crc8(Buffer.from(hex, 'hex')).toString(16).toUpperCase().padStart(2, '0');
}

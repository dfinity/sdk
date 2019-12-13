/**
 * Hashes a string to a number. Algorithm can be found here:
 * https://caml.inria.fr/pub/papers/garrigue-polymorphic_variants-ml98.pdf
 * @param s
 */
export function idlHash(s: string): number {
  const utf8encoder = new TextEncoder();
  const array = utf8encoder.encode(s);

  let h = 0;
  for (const c of array) {
    h = (h * 223 + c) % 2 ** 32;
  }
  return h;
}

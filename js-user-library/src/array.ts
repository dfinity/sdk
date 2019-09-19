export const zipWith = <A, B, C>(
  xs: Array<A>,
  ys: Array<B>,
  f: (x: A, y: B) => C,
): Array<C> => {
  return xs.map((x, i) => f(x, ys[i]));
};

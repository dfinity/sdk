// tslint:disable-next-line: max-line-length
// https://www.typescriptlang.org/docs/handbook/advanced-types.html#exhaustiveness-checking
export const assertNever = (x: never): never => {
  throw new Error(`Unexpected object: ${x}`);
};

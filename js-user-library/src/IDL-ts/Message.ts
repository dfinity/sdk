import { Type } from "./Type";

export class Message {
  argTypes: Array<Type>
  returnTypes: Array<Type>;

  constructor(argTypes: Array<Type>, returnTypes: Array<Type>) {
    this.argTypes = argTypes;
    this.returnTypes = returnTypes;
  }
};

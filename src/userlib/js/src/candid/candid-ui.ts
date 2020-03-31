import BigNumber from 'bignumber.js';
import { CanisterId } from '../canisterId';
import * as IDL from '../idl';
import * as UI from './candid-core';

// tslint:disable:max-classes-per-file
type InputBox = UI.InputBox;

const InputConfig: UI.UIConfig = { parse: parsePrimitive };
const FormConfig: UI.FormConfig = { render: renderInput, container: 'div' };

const inputBox = (t: IDL.Type, config: Partial<UI.UIConfig>) => {
  return new UI.InputBox(t, { ...InputConfig, ...config });
};
const recordForm = (fields: Array<[string, IDL.Type]>, config: Partial<UI.FormConfig>) => {
  return new UI.RecordForm(fields, { ...FormConfig, ...config });
};
const variantForm = (fields: Array<[string, IDL.Type]>, config: Partial<UI.FormConfig>) => {
  return new UI.VariantForm(fields, { ...FormConfig, ...config });
};
const optForm = (ty: IDL.Type, config: Partial<UI.FormConfig>) => {
  return new UI.OptionForm(ty, { ...FormConfig, ...config });
};
const vecForm = (ty: IDL.Type, config: Partial<UI.FormConfig>) => {
  return new UI.VecForm(ty, { ...FormConfig, ...config });
};

class Render extends IDL.Visitor<null, InputBox> {
  public visitType<T>(t: IDL.Type<T>, d: null): InputBox {
    const input = document.createElement('input');
    input.classList.add('argument');
    input.placeholder = t.display();
    return inputBox(t, { input });
  }
  public visitNull(t: IDL.NullClass, d: null): InputBox {
    return inputBox(t, {});
  }
  public visitRecord(t: IDL.RecordClass, fields: Array<[string, IDL.Type]>, d: null): InputBox {
    const form = recordForm(fields, {});
    return inputBox(t, { form });
  }
  public visitVariant(t: IDL.VariantClass, fields: Array<[string, IDL.Type]>, d: null): InputBox {
    const select = document.createElement('select');
    for (const [key, type] of fields) {
      const option = document.createElement('option');
      option.innerText = key;
      select.appendChild(option);
    }
    select.selectedIndex = -1;
    select.classList.add('open');
    const form = variantForm(fields, { open: select, event: 'change' });
    return inputBox(t, { form });
  }
  public visitOpt<T>(t: IDL.OptClass<T>, ty: IDL.Type<T>, d: null): InputBox {
    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.classList.add('open');
    const form = optForm(ty, { open: checkbox, event: 'change' });
    return inputBox(t, { form });
  }
  public visitVec<T>(t: IDL.VecClass<T>, ty: IDL.Type<T>, d: null): InputBox {
    const len = document.createElement('input');
    len.type = 'number';
    len.min = '0';
    len.max = '100';
    len.style.width = '3em';
    len.placeholder = 'len';
    len.classList.add('open');
    const form = vecForm(ty, { open: len, event: 'change' });
    return inputBox(t, { form });
  }
  public visitRec<T>(t: IDL.RecClass<T>, ty: IDL.ConstructType<T>, d: null): InputBox {
    return renderInput(ty);
  }
}

class Parse extends IDL.Visitor<string, any> {
  public visitNull(t: IDL.NullClass, v: string): null {
    return null;
  }
  public visitBool(t: IDL.BoolClass, v: string): boolean {
    if (v === 'true') {
      return true;
    }
    if (v === 'false') {
      return false;
    }
    throw new Error(`Cannot parse ${v} as boolean`);
  }
  public visitText(t: IDL.TextClass, v: string): string {
    return v;
  }
  public visitInt(t: IDL.IntClass, v: string): BigNumber {
    return new BigNumber(v);
  }
  public visitNat(t: IDL.NatClass, v: string): BigNumber {
    return new BigNumber(v);
  }
  public visitFixedInt(t: IDL.FixedIntClass, v: string): BigNumber {
    return new BigNumber(v);
  }
  public visitFixedNat(t: IDL.FixedNatClass, v: string): BigNumber {
    return new BigNumber(v);
  }
  public visitPrincipal(t: IDL.PrincipalClass, v: string): CanisterId {
    return CanisterId.fromText(v);
  }
  public visitService(t: IDL.ServiceClass, v: string): CanisterId {
    return CanisterId.fromText(v);
  }
  public visitFunc(t: IDL.FuncClass, v: string): [CanisterId, string] {
    const x = v.split('.', 2);
    return [CanisterId.fromText(x[0]), x[1]];
  }
}

class Random extends IDL.Visitor<string, any> {
  public visitNull(t: IDL.NullClass, v: string): null {
    return null;
  }
  public visitBool(t: IDL.BoolClass, v: string): boolean {
    return Math.random() < 0.5;
  }
  public visitText(t: IDL.TextClass, v: string): string {
    return Math.random()
      .toString(36)
      .substring(6);
  }
  public visitInt(t: IDL.IntClass, v: string): BigNumber {
    return new BigNumber(this.generateNumber(true));
  }
  public visitNat(t: IDL.NatClass, v: string): BigNumber {
    return new BigNumber(this.generateNumber(false));
  }
  public visitFixedInt(t: IDL.FixedIntClass, v: string): BigNumber {
    return new BigNumber(this.generateNumber(true));
  }
  public visitFixedNat(t: IDL.FixedNatClass, v: string): BigNumber {
    return new BigNumber(this.generateNumber(false));
  }
  private generateNumber(signed: boolean): number {
    const num = Math.floor(Math.random() * 100);
    if (signed && Math.random() < 0.5) {
      return -num;
    } else {
      return num;
    }
  }
}

export function renderInput(t: IDL.Type): InputBox {
  return t.accept(new Render(), null);
}

function parsePrimitive(t: IDL.Type, config: UI.ParseConfig, d: string) {
  if (config.random && d === '') {
    return t.accept(new Random(), d);
  } else {
    return t.accept(new Parse(), d);
  }
}

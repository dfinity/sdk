import BigNumber from 'bignumber.js';
import { CanisterId } from '../canisterId';
import * as IDL from '../idl';
import * as UI from './candid-core';

// tslint:disable:max-classes-per-file
type InputBox = UI.InputBox;

const InputConfig: UI.UIConfig = { parse: parsePrimitive };
const FormConfig: UI.FormConfig = { render: renderInput };

export const inputBox = (t: IDL.Type, config: Partial<UI.UIConfig>) => {
  return new UI.InputBox(t, { ...InputConfig, ...config });
};
export const recordForm = (fields: Array<[string, IDL.Type]>, config: Partial<UI.FormConfig>) => {
  return new UI.RecordForm(fields, { ...FormConfig, ...config });
};
export const variantForm = (fields: Array<[string, IDL.Type]>, config: Partial<UI.FormConfig>) => {
  return new UI.VariantForm(fields, { ...FormConfig, ...config });
};
export const optForm = (ty: IDL.Type, config: Partial<UI.FormConfig>) => {
  return new UI.OptionForm(ty, { ...FormConfig, ...config });
};
export const vecForm = (ty: IDL.Type, config: Partial<UI.FormConfig>) => {
  return new UI.VecForm(ty, { ...FormConfig, ...config });
};

export class Render extends IDL.Visitor<null, InputBox> {
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
    let config = {};
    if (fields.length > 1) {
      const container = document.createElement('div');
      container.classList.add('popup-form');
      config = { container };
    }
    const form = recordForm(fields, config);
    return inputBox(t, { form });
  }
  public visitVariant(t: IDL.VariantClass, fields: Array<[string, IDL.Type]>, d: null): InputBox {
    const select = document.createElement('select');
    for (const [key, type] of fields) {
      const option = new Option(key);
      select.add(option);
    }
    select.selectedIndex = -1;
    select.classList.add('open');
    const config: Partial<UI.FormConfig> = { open: select, event: 'change' };
    const form = variantForm(fields, config);
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
    const container = document.createElement('div');
    container.classList.add('popup-form');
    const form = vecForm(ty, { open: len, event: 'change', container });
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
  public visitFloat(t: IDL.FloatClass, v: string): number {
    return parseFloat(v);
  }
  public visitNumber(t: IDL.PrimitiveType, v: string): BigNumber {
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
  public visitFloat(t: IDL.FloatClass, v: string): number {
    return Math.random();
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

function parsePrimitive(t: IDL.Type, config: UI.ParseConfig, d: string) {
  if (config.random && d === '') {
    return t.accept(new Random(), d);
  } else {
    return t.accept(new Parse(), d);
  }
}

export function renderInput(t: IDL.Type): InputBox {
  return t.accept(new Render(), null);
}

interface ValueConfig {
  input: InputBox;
  value: any;
}

export function renderValue(t: IDL.Type, input: InputBox, value: any) {
  return t.accept(new RenderValue(), { input, value });
}

class RenderValue extends IDL.Visitor<ValueConfig, void> {
  public visitType<T>(t: IDL.Type<T>, d: ValueConfig) {
    (d.input.ui.input as HTMLInputElement).value = t.valueToString(d.value);
  }
  public visitNull(t: IDL.NullClass, d: ValueConfig) {}
  public visitText(t: IDL.TextClass, d: ValueConfig) {
    (d.input.ui.input as HTMLInputElement).value = d.value;
  }
  public visitRec<T>(t: IDL.RecClass<T>, ty: IDL.ConstructType<T>, d: ValueConfig) {
    renderValue(ty, d.input, d.value);
  }
  public visitOpt<T>(t: IDL.OptClass<T>, ty: IDL.Type<T>, d: ValueConfig) {
    if (d.value.length === 0) {
      return;
    } else {
      const form = d.input.ui.form!;
      const open = form.ui.open as HTMLInputElement;
      open.checked = true;
      open.dispatchEvent(new Event(form.ui.event!));
      renderValue(ty, form.form[0], d.value[0]);
    }
  }
  public visitRecord(t: IDL.RecordClass, fields: Array<[string, IDL.Type]>, d: ValueConfig) {
    const form = d.input.ui.form!;
    fields.forEach(([key, type], i) => {
      renderValue(type, form.form[i], d.value[key]);
    });
  }
  public visitVariant(t: IDL.VariantClass, fields: Array<[string, IDL.Type]>, d: ValueConfig) {
    const form = d.input.ui.form!;
    const selected = Object.entries(d.value)[0];
    fields.forEach(([key, type], i) => {
      if (key === selected[0]) {
        const open = form.ui.open as HTMLSelectElement;
        open.selectedIndex = i;
        open.dispatchEvent(new Event(form.ui.event!));
        renderValue(type, form.form[0], selected[1]);
      }
    });
  }
  public visitVec<T>(t: IDL.VecClass<T>, ty: IDL.Type<T>, d: ValueConfig) {
    const form = d.input.ui.form!;
    const len = d.value.length;
    const open = form.ui.open as HTMLInputElement;
    open.value = len;
    open.dispatchEvent(new Event(form.ui.event!));
    d.value.forEach((v: T, i: number) => {
      renderValue(ty, form.form[i], v);
    });
  }
}

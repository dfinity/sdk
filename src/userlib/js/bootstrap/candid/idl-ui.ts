import { CanisterId, IDL } from '@internet-computer/userlib';
import BigNumber from 'bignumber.js';

// tslint:disable:max-classes-per-file

class Render extends IDL.Visitor<null, InputBox> {
  public visitPrimitive<T>(t: IDL.PrimitiveType<T>, d: null): InputBox {
    return new InputBox(t, null);
  }
  public visitNull(t: IDL.NullClass, d: null): InputBox {
    const input = new InputBox(t, null);
    input.input.type = 'hidden';
    return input;
  }
  public visitPrincipal(t: IDL.PrincipalClass, d: null): InputBox {
    return new InputBox(t, null);
  }
  public visitRecord(t: IDL.RecordClass, fields: Array<[string, IDL.Type]>, d: null): InputBox {
    const form = new RecordForm(fields);
    return new InputBox(t, form);
  }
  public visitVariant(t: IDL.VariantClass, fields: Array<[string, IDL.Type]>, d: null): InputBox {
    const form = new VariantForm(fields);
    return new InputBox(t, form);
  }
  public visitOpt<T>(t: IDL.OptClass<T>, ty: IDL.Type<T>, d: null): InputBox {
    const form = new OptionForm(ty);
    return new InputBox(t, form);
  }
  public visitVec<T>(t: IDL.VecClass<T>, ty: IDL.Type<T>, d: null): InputBox {
    const form = new VecForm(ty);
    return new InputBox(t, form);
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
}

export function renderInput(t: IDL.Type): InputBox {
  return t.accept(new Render(), null);
}

function parsePrimitive(t: IDL.Type, d: string) {
  return t.accept(new Parse(), d);
}

class InputBox {
  public input: HTMLInputElement;
  public status: HTMLElement;
  public label: string | null = null;
  public value: any = undefined;

  constructor(public idl: IDL.Type, public form: InputForm | null = null) {
    const status = document.createElement('div');
    status.className = 'status';
    this.status = status;

    const input = document.createElement('input');
    input.className = 'argument';
    input.placeholder = idl.display();
    this.input = input;

    input.addEventListener('blur', () => {
      if (input.value === '') {
        return;
      }
      this.parse();
    });
    input.addEventListener('focus', () => {
      input.className = 'argument';
    });
  }
  public isRejected(): boolean {
    return this.value === undefined;
  }
  public parse(): any {
    if (this.form) {
      const value = this.form.parse();
      this.value = value;
      return value;
    }

    try {
      const value = parsePrimitive(this.idl, this.input.value);
      if (!this.idl.covariant(value)) {
        throw new Error(`${this.input.value} is not of type ${this.idl.display()}`);
      }
      this.status.style.display = 'none';
      this.value = value;
      return value;
    } catch (err) {
      this.input.className += ' reject';
      this.status.style.display = 'block';
      this.status.innerHTML = 'InputError: ' + err.message;
      this.value = undefined;
      return undefined;
    }
  }
  public render(dom: HTMLElement): void {
    const container = document.createElement('span');
    if (this.label) {
      const label = document.createElement('label');
      label.innerText = this.label;
      container.appendChild(label);
    }
    container.appendChild(this.input);
    container.appendChild(this.status);

    if (this.form) {
      this.input.type = 'hidden';
      this.form.render(container);
      const input = this.input;
    }
    dom.appendChild(container);
  }
}

abstract class InputForm {
  public form: InputBox[] = [];
  public open: HTMLElement = document.createElement('button');
  public event: string = 'change';

  public abstract parse(): any;
  public abstract generateForm(): any;
  public renderForm(dom: HTMLElement): void {
    if (this.form.length === 0) {
      return;
    }
    if (this.form.length === 1) {
      this.form[0].render(dom);
      return;
    }
    const form = document.createElement('div');
    form.className = 'popup-form';
    this.form.forEach(e => e.render(form));
    dom.appendChild(form);
  }
  public render(dom: HTMLElement): void {
    dom.appendChild(this.open);
    const form = this;
    form.open.addEventListener(form.event, () => {
      while (dom.lastElementChild) {
        if (dom.lastElementChild !== form.open) {
          dom.removeChild(dom.lastElementChild);
        } else {
          break;
        }
      }
      // Render form
      form.generateForm();
      form.renderForm(dom);
    });
  }
}

class RecordForm extends InputForm {
  constructor(public fields: Array<[string, IDL.Type]>) {
    super();
    this.open.innerText = '...';
    this.event = 'click';
  }
  public generateForm(): void {
    this.form = this.fields.map(([key, type]) => {
      const input = renderInput(type);
      input.label = key + ' ';
      return input;
    });
  }
  public render(dom: HTMLElement): void {
    // No open button for record
    this.generateForm();
    this.renderForm(dom);
  }
  public parse(): Record<string, any> | undefined {
    const v: Record<string, any> = {};
    this.fields.forEach(([key, _], i) => {
      const value = this.form[i].parse();
      v[key] = value;
    });
    if (this.form.some(input => input.isRejected())) {
      return undefined;
    }
    return v;
  }
}

class VariantForm extends InputForm {
  constructor(public fields: Array<[string, IDL.Type]>) {
    super();
    const select = document.createElement('select');
    for (const [key, type] of fields) {
      const option = document.createElement('option');
      option.innerText = key;
      select.appendChild(option);
    }
    select.selectedIndex = -1;
    select.className = 'open';
    this.open = select;
    this.event = 'change';
  }
  public generateForm(): void {
    const index = (this.open as HTMLSelectElement).selectedIndex;
    const [_, type] = this.fields[index];
    const variant = renderInput(type);
    this.form = [variant];
  }
  public parse(): Record<string, any> | undefined {
    const select = this.open as HTMLSelectElement;
    const selected = select.options[select.selectedIndex].text;
    const value = this.form[0].parse();
    if (value === undefined) {
      return undefined;
    }
    const v: Record<string, any> = {};
    v[selected] = value;
    return v;
  }
}

class OptionForm extends InputForm {
  constructor(public ty: IDL.Type) {
    super();
    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.className = 'open';
    this.open = checkbox;
    this.event = 'change';
  }
  public generateForm(): void {
    if ((this.open as HTMLInputElement).checked) {
      const opt = renderInput(this.ty);
      this.form = [opt];
    } else {
      this.form = [];
    }
  }
  public parse<T>(): [T] | [] | undefined {
    if (this.form.length === 0) {
      return [];
    } else {
      const value = this.form[0].parse();
      if (value === undefined) {
        return undefined;
      }
      return [value];
    }
  }
}

class VecForm extends InputForm {
  constructor(public ty: IDL.Type) {
    super();
    const len = document.createElement('input');
    len.type = 'number';
    len.min = '0';
    len.max = '100';
    len.style.width = '3em';
    len.placeholder = 'length';
    len.className = 'open';
    this.open = len;
    this.event = 'change';
  }
  public generateForm(): void {
    const len = (this.open as HTMLInputElement).valueAsNumber;
    this.form = [];
    for (let i = 0; i < len; i++) {
      const t = renderInput(this.ty);
      this.form.push(t);
    }
  }
  public renderForm(dom: HTMLElement): void {
    // Same code as parent class except the single length optimization
    if (this.form.length === 0) {
      return;
    }
    const form = document.createElement('div');
    form.className = 'popup-form';
    this.form.forEach(e => e.render(form));
    dom.appendChild(form);
  }
  public parse<T>(): T[] | undefined {
    const value = this.form.map(input => {
      return input.parse();
    });
    if (this.form.some(input => input.isRejected())) {
      return undefined;
    }
    return value;
  }
}

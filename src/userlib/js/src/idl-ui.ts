import * as IDL from './idl';

// tslint:disable:max-classes-per-file
export class Format {
  constructor(public showInput: boolean, public form: InputForm | null, public hasClose: boolean) {}
}

class Render extends IDL.Visitor<null, InputBox> {
  public visitPrimitive<T>(t: IDL.PrimitiveType<T>, d: null): InputBox {
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
    return new InputBox(t, null);
  }
  public visitRec<T>(t: IDL.RecClass<T>, ty: IDL.ConstructType<T>, d: null): InputBox {
    return renderInput(ty);
  }
}

class Default extends IDL.Visitor<null, string | null> {
  public visitType<T>(t: IDL.Type<T>, d: null): string | null {
    return null;
  }
  public visitUnit(t: IDL.UnitClass, d: null): string | null {
    return 'null';
  }
  public visitOpt<T>(t: IDL.OptClass<T>, ty: IDL.Type<T>, d: null): string | null {
    return '[]';
  }
  public visitRec<T>(t: IDL.RecClass<T>, ty: IDL.ConstructType<T>, d: null): string | null {
    return defaultString(ty);
  }
}

export function renderInput(t: IDL.Type): InputBox {
  return t.accept(new Render(), null);
}

function defaultString(t: IDL.Type): string | null {
  return t.accept(new Default(), null);
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
    if (this.input.disabled && this.form) {
      const value = this.form.parse();
      this.value = value;
      if (value !== undefined) {
        this.input.value = this.idl.valueToString(value);
      }
      return value;
    }
    if (this.input.value === '') {
      const str = defaultString(this.idl);
      if (str) {
        this.input.value = str;
      }
    }
    try {
      const value = this.idl.stringToValue(this.input.value);
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
      this.form.render(container);
      const input = this.input;
      this.form.open.addEventListener(this.form.event, () => {
        input.setAttribute('disabled', '');
      });
    }
    dom.appendChild(container);
  }
}

abstract class InputForm {
  public form: InputBox[] = [];
  public open: HTMLElement = document.createElement('button');
  public event: string = 'click';

  public abstract parse(): any;
  public abstract generateForm(): any;
  public renderForm(dom: HTMLElement): void {
    if (!this.form.length) {
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
    this.open = document.createElement('input');
    (this.open as HTMLInputElement).type = 'checkbox';
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
/*
function renderComposite(
  dom: HTMLElement,
  idl: IDL.Type,
  open: HTMLElement,
  event: string,
  render: (dom: HTMLElement) => HTMLInputElement[],
  parse: (args: HTMLInputElement[]) => string,
): HTMLInputElement {
  const container = document.createElement('span');
  const input = renderPrimitive(container, idl);
  input.className = 'composite';
  container.appendChild(open);

  open.addEventListener(event, () => {
    const form = document.createElement('div');
    form.className = 'popup-form';
    const args = render(form);
    if (!args || !args.length) {
      input.value = parse(args);
      input.focus();
      return;
    }

    input.setAttribute('disabled', '');
    open.setAttribute('disabled', '');
    const close = document.createElement('button');
    close.innerText = 'X';
    form.appendChild(close);
    open.insertAdjacentElement('afterend', form);

    close.addEventListener('click', () => {
      args.forEach(arg => arg.dispatchEvent(parseEvent));
      const isReject = args.some(arg => arg.classList.contains('reject'));
      if (isReject) {
        return;
      }
      const result = parse(args);
      input.removeAttribute('disabled');
      open.removeAttribute('disabled');
      input.value = result;
      (form.parentNode as Node).removeChild(form);
      input.focus();
    });
  });
  dom.appendChild(container);
  return input;
}
*/

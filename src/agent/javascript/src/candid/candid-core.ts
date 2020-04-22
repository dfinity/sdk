import * as IDL from '../idl';

// tslint:disable:max-classes-per-file

export interface ParseConfig {
  random?: boolean;
}

export interface UIConfig {
  input?: HTMLElement;
  form?: InputForm;
  parse(t: IDL.Type, config: ParseConfig, v: string): any;
}

export interface FormConfig {
  open?: HTMLElement;
  event?: string;
  labelMap?: Record<string, string>;
  container?: HTMLElement;
  render(t: IDL.Type): InputBox;
}

export class InputBox {
  public status: HTMLElement;
  public label: string | null = null;
  public value: any = undefined;

  constructor(public idl: IDL.Type, public ui: UIConfig) {
    const status = document.createElement('div');
    status.className = 'status';
    this.status = status;

    if (ui.input) {
      ui.input.addEventListener('blur', () => {
        if ((ui.input as HTMLInputElement).value === '') {
          return;
        }
        this.parse();
      });
      ui.input.addEventListener('focus', () => {
        ui.input!.classList.remove('reject');
      });
    }
  }
  public isRejected(): boolean {
    return this.value === undefined;
  }

  public parse(config: ParseConfig = {}): any {
    if (this.ui.form) {
      const value = this.ui.form.parse(config);
      this.value = value;
      return value;
    }

    if (this.ui.input) {
      const input = this.ui.input as HTMLInputElement;
      try {
        const value = this.ui.parse(this.idl, config, input.value);
        if (!this.idl.covariant(value)) {
          throw new Error(`${input.value} is not of type ${this.idl.display()}`);
        }
        this.status.style.display = 'none';
        this.value = value;
        return value;
      } catch (err) {
        input.classList.add('reject');
        this.status.style.display = 'block';
        this.status.innerHTML = 'InputError: ' + err.message;
        this.value = undefined;
        return undefined;
      }
    }
    return null;
  }
  public render(dom: HTMLElement): void {
    const container = document.createElement('span');
    if (this.label) {
      const label = document.createElement('label');
      label.innerText = this.label;
      container.appendChild(label);
    }
    if (this.ui.input) {
      container.appendChild(this.ui.input);
      container.appendChild(this.status);
    }

    if (this.ui.form) {
      this.ui.form.render(container);
    }
    dom.appendChild(container);
  }
}

export abstract class InputForm {
  public form: InputBox[] = [];
  constructor(public ui: FormConfig) {}

  public abstract parse(config: ParseConfig): any;
  public abstract generateForm(): any;
  public renderForm(dom: HTMLElement): void {
    if (this.ui.container) {
      this.form.forEach(e => e.render(this.ui.container!));
      dom.appendChild(this.ui.container);
    } else {
      this.form.forEach(e => e.render(dom));
    }
  }
  public render(dom: HTMLElement): void {
    if (this.ui.open && this.ui.event) {
      dom.appendChild(this.ui.open);
      const form = this;
      form.ui.open!.addEventListener(form.ui.event!, () => {
        // Remove old form
        if (form.ui.container) {
          form.ui.container.innerHTML = '';
        } else {
          const oldContainer = form.ui.open!.nextElementSibling;
          if (oldContainer) {
            oldContainer.parentNode!.removeChild(oldContainer);
          }
        }
        // Render form
        form.generateForm();
        form.renderForm(dom);
      });
    } else {
      this.generateForm();
      this.renderForm(dom);
    }
  }
}

export class RecordForm extends InputForm {
  constructor(public fields: Array<[string, IDL.Type]>, public ui: FormConfig) {
    super(ui);
  }
  public generateForm(): void {
    this.form = this.fields.map(([key, type]) => {
      const input = this.ui.render(type);
      if (this.ui.labelMap && this.ui.labelMap.hasOwnProperty(key)) {
        input.label = this.ui.labelMap[key] + ' ';
      } else {
        input.label = key + ' ';
      }
      return input;
    });
  }
  public parse(config: ParseConfig): Record<string, any> | undefined {
    const v: Record<string, any> = {};
    this.fields.forEach(([key, _], i) => {
      const value = this.form[i].parse(config);
      v[key] = value;
    });
    if (this.form.some(input => input.isRejected())) {
      return undefined;
    }
    return v;
  }
}

export class VariantForm extends InputForm {
  constructor(public fields: Array<[string, IDL.Type]>, public ui: FormConfig) {
    super(ui);
  }
  public generateForm(): void {
    const index = (this.ui.open as HTMLSelectElement).selectedIndex;
    const [_, type] = this.fields[index];
    const variant = this.ui.render(type);
    this.form = [variant];
  }
  public parse(config: ParseConfig): Record<string, any> | undefined {
    const select = this.ui.open as HTMLSelectElement;
    const selected = select.options[select.selectedIndex].value;
    const value = this.form[0].parse(config);
    if (value === undefined) {
      return undefined;
    }
    const v: Record<string, any> = {};
    v[selected] = value;
    return v;
  }
}

export class OptionForm extends InputForm {
  constructor(public ty: IDL.Type, public ui: FormConfig) {
    super(ui);
  }
  public generateForm(): void {
    if ((this.ui.open as HTMLInputElement).checked) {
      const opt = this.ui.render(this.ty);
      this.form = [opt];
    } else {
      this.form = [];
    }
  }
  public parse<T>(config: ParseConfig): [T] | [] | undefined {
    if (this.form.length === 0) {
      return [];
    } else {
      const value = this.form[0].parse(config);
      if (value === undefined) {
        return undefined;
      }
      return [value];
    }
  }
}

export class VecForm extends InputForm {
  constructor(public ty: IDL.Type, public ui: FormConfig) {
    super(ui);
  }
  public generateForm(): void {
    const len = +(this.ui.open as HTMLInputElement).value;
    this.form = [];
    for (let i = 0; i < len; i++) {
      const t = this.ui.render(this.ty);
      this.form.push(t);
    }
  }
  public parse<T>(config: ParseConfig): T[] | undefined {
    const value = this.form.map(input => {
      return input.parse(config);
    });
    if (this.form.some(input => input.isRejected())) {
      return undefined;
    }
    return value;
  }
}

import * as IDL from './idl';

class Render implements IDL.Visitor<HTMLElement, HTMLInputElement> {
  public visitEmpty(t: IDL.EmptyClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitBool(t: IDL.BoolClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitUnit(t: IDL.UnitClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitText(t: IDL.TextClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitInt(t: IDL.IntClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitNat(t: IDL.NatClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitFixedInt(t: IDL.FixedIntClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitFixedNat(t: IDL.FixedNatClass, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitVec<T>(t: IDL.VecClass<T>, d: HTMLElement): HTMLInputElement {
    return renderPrimitive(d, t);
  }
  public visitOpt<T>(t: IDL.OptClass<T>, d: HTMLElement): HTMLInputElement {
    return renderOption(d, t);
  }
  public visitRecord(t: IDL.RecordClass, d: HTMLElement): HTMLInputElement {
    return renderRecord(d, t);
  }
  public visitVariant(t: IDL.VariantClass, d: HTMLElement): HTMLInputElement {
    return renderVariant(d, t);
  }
  public visitRec<T>(t: IDL.RecClass<T>, d: HTMLElement): HTMLInputElement {
    // @ts-ignore
    return renderInput(t._type as IDL.Type, d);
  }
}

class Default implements IDL.Visitor<null, string | null> {
  public visitEmpty(t: IDL.EmptyClass, d: null): string | null {
    return null;
  }
  public visitBool(t: IDL.BoolClass, d: null): string | null {
    return null;
  }
  public visitUnit(t: IDL.UnitClass, d: null): string | null {
    return 'null';
  }
  public visitText(t: IDL.TextClass, d: null): string | null {
    return null;
  }
  public visitInt(t: IDL.IntClass, d: null): string | null {
    return null;
  }
  public visitNat(t: IDL.NatClass, d: null): string | null {
    return null;
  }
  public visitFixedInt(t: IDL.FixedIntClass, d: null): string | null {
    return null;
  }
  public visitFixedNat(t: IDL.FixedNatClass, d: null): string | null {
    return null;
  }
  public visitVec<T>(t: IDL.VecClass<T>, d: null): string | null {
    return null;
  }
  public visitOpt<T>(t: IDL.OptClass<T>, d: null): string | null {
    return '[]';
  }
  public visitRecord(t: IDL.RecordClass, d: null): string | null {
    return null;
  }
  public visitVariant(t: IDL.VariantClass, d: null): string | null {
    return null;
  }
  public visitRec<T>(t: IDL.RecClass<T>, d: null): string | null {
    // @ts-ignore
    return defaultString(t._type as IDL.Type);
  }
}

export function renderInput(t: IDL.Type, dom: HTMLElement): HTMLInputElement {
  return t.accept(new Render(), dom);
}

function defaultString(t: IDL.Type): string | null {
  return t.accept(new Default(), null);
}

// tslint:disable:no-shadowed-variable

function validate(idl: IDL.Type, arg: HTMLInputElement) {
  const value = idl.stringToValue(arg.value);
  if (!idl.covariant(value)) {
    throw new Error(`${arg.value} is not of type ${idl.display()}`);
  }
  return value;
}

const parseEvent = new Event('parse');

function renderPrimitive(dom: HTMLElement, idl: IDL.Type): HTMLInputElement {
  const container = document.createElement('span');
  const status = document.createElement('div');
  status.className = 'status';
  const arg = document.createElement('input');
  arg.className = 'argument';
  arg.placeholder = idl.display();
  const val = defaultString(idl);
  if (val) {
    arg.value = val;
  }

  arg.addEventListener('parse', () => {
    try {
      const value = validate(idl, arg);
      status.style.display = 'none';
    } catch (err) {
      arg.className += ' reject';
      status.style.display = 'block';
      status.innerHTML = 'InputError: ' + err.message;
    }
  });
  arg.addEventListener('blur', () => {
    if (arg.value === '') {
      return;
    }
    arg.dispatchEvent(parseEvent);
  });
  arg.addEventListener('focus', () => {
    arg.className = 'argument';
  });

  container.appendChild(arg);
  container.appendChild(status);
  dom.appendChild(container);
  return arg;
}

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

function renderRecord(dom: HTMLElement, idl: IDL.RecordClass): HTMLInputElement {
  const open = document.createElement('button');
  open.innerText = '...';

  const render = (dom: HTMLElement): HTMLInputElement[] => {
    const args = [];
    // @ts-ignore
    for (const [key, type] of idl._fields) {
      const label = document.createElement('label');
      label.innerText = key + ' ';
      dom.appendChild(label);
      const arg = renderInput(type, dom);
      args.push(arg);
    }
    return args;
  };
  const parse = (args: HTMLInputElement[]): string => {
    const values: string[] = [];
    // @ts-ignore
    idl._fields.forEach(([key, _], i) => {
      const val = '"' + key + '":' + args[i].value;
      values.push(val);
    });
    return `{${values.join(', ')}}`;
  };
  return renderComposite(dom, idl, open, 'click', render, parse);
}

function renderOption<T>(dom: HTMLElement, idl: IDL.OptClass<T>): HTMLInputElement {
  const checkbox = document.createElement('input');
  checkbox.type = 'checkbox';
  checkbox.checked = false;

  const render = (dom: HTMLElement): HTMLInputElement[] => {
    if (checkbox.checked) {
      // @ts-ignore
      const opt = renderInput(idl._type, dom);
      return [opt];
    } else {
      return [];
    }
  };
  const parse = (arg: HTMLInputElement[]): string => {
    if (!arg || !arg.length) {
      return '[]';
    } else {
      return '[' + arg[0].value + ']';
    }
  };
  return renderComposite(dom, idl, checkbox, 'change', render, parse);
}

function renderVariant(dom: HTMLElement, idl: IDL.VariantClass): HTMLInputElement {
  const select = document.createElement('select');
  // @ts-ignore
  for (const [key, type] of idl._fields) {
    const option = document.createElement('option');
    option.innerText = key;
    select.appendChild(option);
  }
  select.selectedIndex = -1;

  const render = (dom: HTMLElement): HTMLInputElement[] => {
    const index = select.selectedIndex;
    // @ts-ignore
    const [_, type] = idl._fields[index];
    const variant = renderInput(type, dom);
    return [variant];
  };
  const parse = (arg: HTMLInputElement[]): string => {
    const selected = select.options[select.selectedIndex].text;
    return `{"${selected}":${arg[0].value}}`;
  };
  return renderComposite(dom, idl, select, 'change', render, parse);
}

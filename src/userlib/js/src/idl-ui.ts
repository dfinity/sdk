import { FuncClass, Type } from './idl';

// tslint:disable:no-shadowed-variable

export function renderPrimitive(dom: HTMLElement, id: string, idl: Type): HTMLInputElement {
  const status = document.createElement('div');
  status.className = 'status';
  const arg = document.createElement('input');
  arg.className = 'argument';
  arg.id = id;
  arg.placeholder = idl.display();

  arg.addEventListener('focus', () => {
    arg.className = 'argument';
  });
  arg.addEventListener('blur', () => {
    try {
      if (arg.value === '') {
        return;
      }
      const value = idl.stringToValue(arg.value);
      if (!idl.covariant(value)) {
        throw new Error(`${arg.value} is not of type ${idl.display()}`);
      }
      status.style.display = 'none';
    } catch (err) {
      arg.className += ' reject';
      status.style.display = 'block';
      status.innerHTML = 'InputError: ' + err.message;
    }
  });
  dom.appendChild(arg);
  dom.appendChild(status);
  return arg;
}

export function renderComposite(
  dom: HTMLElement,
  id: string,
  idl: Type,
  open: HTMLElement,
  event: string,
  render: any,
  parse: any,
): HTMLInputElement {
  const input = renderPrimitive(dom, id, idl);
  input.className = 'composite';
  dom.appendChild(open);

  open.addEventListener(event, () => {
    input.setAttribute('disabled', '');
    open.setAttribute('disabled', '');

    const form = document.createElement('div');
    form.className = 'popup-form';
    const args = render(form, id);
    const close = document.createElement('button');
    close.innerText = 'X';
    form.appendChild(close);
    open.insertAdjacentElement('afterend', form);

    close.addEventListener('click', () => {
      // TODO validate args
      const result = parse(args);
      input.value = result;
      input.removeAttribute('disabled');
      open.removeAttribute('disabled');
      (form.parentNode as Node).removeChild(form);
      input.focus();
    });
  });
  return input;
}

export function renderRecord(dom: HTMLElement, id: string, idl: any): HTMLInputElement {
  const open = document.createElement('button');
  open.innerText = '...';

  const render = (dom: HTMLElement, id: string): HTMLInputElement[] => {
    const args = [];
    for (const [key, type] of idl._fields) {
      const label = document.createElement('label');
      const keyId = id + '_' + key;
      label.innerText = key + ' ';
      dom.appendChild(label);
      const arg = type.renderInput(dom, keyId);
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
  return renderComposite(dom, id, idl, open, 'click', render, parse);
}

export function renderOption(dom: HTMLElement, id: string, idl: any): HTMLInputElement {
  const checkbox = document.createElement('input');
  checkbox.type = 'checkbox';
  checkbox.checked = false;

  const render = (dom: HTMLElement, id: string): HTMLInputElement => {
    const opt = idl._type.renderInput(dom, id + '_opt');
    if (!checkbox.checked) {
      opt.style.display = 'none';
    } else {
      opt.style.display = 'block';
    }
    return opt;
  };
  const parse = (arg: HTMLInputElement): string => {
    if (!checkbox.checked || arg.value === '') {
      return 'null';
    } else {
      return arg.value;
    }
  };
  return renderComposite(dom, id, idl, checkbox, 'change', render, parse);
}

export function renderVariant(dom: HTMLElement, id: string, idl: any): HTMLInputElement {
  const select = document.createElement('select');
  for (const [key, type] of idl._fields) {
    const option = document.createElement('option');
    option.innerText = key;
    select.appendChild(option);
  }
  select.selectedIndex = -1;

  const render = (dom: HTMLElement, id: string): HTMLInputElement => {
    const index = select.selectedIndex;
    const [_, type] = idl._fields[index];
    const variant = type.renderInput(dom, id + '_' + index);
    return variant;
  };
  const parse = (arg: HTMLInputElement): string => {
    const selected = select.options[select.selectedIndex].text;
    return `{"${selected}":${arg.value}}`;
  };
  return renderComposite(dom, id, idl, select, 'change', render, parse);
}

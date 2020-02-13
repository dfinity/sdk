import { Type } from './idl';

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
      const value = JSON.parse(arg.value);
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
  render: any,
  parse: any,
): HTMLInputElement {
  const input = renderPrimitive(dom, id, idl);
  const open = document.createElement('button');
  open.innerText = '...';
  dom.appendChild(open);

  open.addEventListener('click', () => {
    open.disabled = true;
    const form = document.createElement('div');
    form.className = 'popup-form';
    const args = render(form, id);
    const close = document.createElement('button');
    close.innerText = 'X';
    form.appendChild(close);
    open.insertAdjacentElement('afterend', form);

    close.addEventListener('click', () => {
      open.disabled = false;
      const result = parse(args);
      input.value = result;
      (form.parentNode as Node).removeChild(form);
    });
  });
  return input;
}

export function renderRecord(dom: HTMLElement, id: string, idl: any): HTMLInputElement {
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
  return renderComposite(dom, id, idl, render, parse);
}

export function renderOption(dom: HTMLElement, id: string, idl: any): HTMLInputElement {
  const render = (dom: HTMLElement, id: string): HTMLInputElement[] => {
    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.checked = true;
    dom.appendChild(checkbox);
    const opt = idl._type.renderInput(dom, id + '_opt');

    checkbox.addEventListener('click', () => {
      if (checkbox.checked) {
        opt.style.display = 'block';
      } else {
        opt.style.display = 'none';
      }
    });
    return [checkbox, opt];
  };
  const parse = (args: HTMLInputElement[]): string => {
    if (args[0].checked) {
      return args[1].value;
    } else {
      return 'null';
    }
  };
  return renderComposite(dom, id, idl, render, parse);
}

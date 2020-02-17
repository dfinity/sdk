import { FuncClass, Type } from './idl';

// tslint:disable:no-shadowed-variable

function validate(idl: Type, arg: HTMLInputElement): any {
  const value = idl.stringToValue(arg.value);
  if (!idl.covariant(value)) {
    throw new Error(`${arg.value} is not of type ${idl.display()}`);
  }
  return value;
}

const parseEvent = new Event('parse');

export function renderPrimitive(dom: HTMLElement, id: string, idl: Type): HTMLInputElement {
  const container = document.createElement('span');    
  const status = document.createElement('div');
  status.className = 'status';
  const arg = document.createElement('input');
  arg.className = 'argument';
  arg.id = id;
  arg.placeholder = idl.display();

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

export function renderComposite(
  dom: HTMLElement,
  id: string,
  idl: Type,
  open: HTMLElement,
  event: string,
  render: (dom: HTMLElement, id: string) => HTMLInputElement[],
  parse: (args: HTMLInputElement[]) => string,
): HTMLInputElement {
  const container = document.createElement('span');
  const input = renderPrimitive(container, id, idl);
  // input.className = 'composite';
  container.appendChild(open);

  open.addEventListener(event, () => {
    input.setAttribute('disabled', '');
    open.setAttribute('disabled', '');

    const form = document.createElement('div');
    form.className = 'popup-form';
    const args = render(form, id);
    if (!args || !args.length) {
      input.value = parse(args);
      input.focus();
      return;
    }
    
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
  /*
  const container = document.createElement('span');
  const checkbox = document.createElement('input');
  checkbox.type = 'checkbox';
  checkbox.checked = false;
  checkbox.value = '[]';
  container.appendChild(checkbox);

  checkbox.addEventListener('change', () => {
    if (checkbox.checked) {
      const opt = idl._type.renderInput(container, id + '_opt');
      opt.addEventListener('input', () => {
        checkbox.value = '[' + opt.value + ']';
      });
    } else {
      const remove = checkbox.nextElementSibling as Node;
      (remove.parentNode as Node).removeChild(remove);
      checkbox.value = '[]';
    }
  });
  dom.appendChild(container);
  return checkbox;
  */
  const checkbox = document.createElement('input');
  checkbox.type = 'checkbox';
  checkbox.checked = false;

  const render = (dom: HTMLElement, id: string): HTMLInputElement[] => {
    if (checkbox.checked) {
      const opt = idl._type.renderInput(dom, id + '_opt');
      return [opt];
    } else {
      return [];
    }
  };
  const parse = (arg: HTMLInputElement[]): string => {
    if (!arg || !arg.length) {
      return 'null';
    } else {
      return arg[0].value;
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

  const render = (dom: HTMLElement, id: string): HTMLInputElement[] => {
    const index = select.selectedIndex;
    const [_, type] = idl._fields[index];
    const variant = type.renderInput(dom, id + '_' + index);
    return [variant];
  };
  const parse = (arg: HTMLInputElement[]): string => {
    const selected = select.options[select.selectedIndex].text;
    return `{"${selected}":${arg[0].value}}`;
  };
  return renderComposite(dom, id, idl, select, 'change', render, parse);
}

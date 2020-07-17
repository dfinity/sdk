import './candid.css';
// tslint:disable-next-line:ordered-imports
import { Actor, CanisterId, IDL, InputBox, UI } from '@dfinity/agent';

class CanisterActor extends Actor {
  [x: string]: (...args: unknown[]) => Promise<unknown>;
}

export function render(id: CanisterId, canister: CanisterActor) {
  document.getElementById('title')!.innerText = `Service ${id}`;
  for (const [name, func] of Actor.interfaceOf(canister)._fields) {
    renderMethod(canister, name, func);
  }
  const consoleEl = document.createElement('div');
  consoleEl.className = 'console';
  document.body.appendChild(consoleEl);
}

function renderMethod(canister: CanisterActor, name: string, idlFunc: IDL.FuncClass) {
  const item = document.createElement('li');

  const sig = document.createElement('div');
  sig.className = 'signature';
  sig.innerHTML = `${name}: ${idlFunc.display()}`;
  item.appendChild(sig);

  const inputs: InputBox[] = [];
  idlFunc.argTypes.forEach((arg, i) => {
    const inputbox = UI.renderInput(arg);
    inputs.push(inputbox);
    inputbox.render(item);
  });

  const button = document.createElement('button');
  button.className = 'btn';
  if (idlFunc.annotations.includes('query')) {
    button.innerText = 'Query';
  } else {
    button.innerText = 'Call';
  }
  item.appendChild(button);

  const random = document.createElement('button');
  random.className = 'btn';
  random.innerText = 'Lucky';
  item.appendChild(random);

  const resultDiv = document.createElement('div');
  resultDiv.className = 'result';
  const left = document.createElement('span');
  left.className = 'left';
  const right = document.createElement('span');
  right.className = 'right';
  resultDiv.appendChild(left);
  resultDiv.appendChild(right);
  item.appendChild(resultDiv);

  const list = document.getElementById('methods')!;
  list.append(item);

  async function call(actor: Actor, args: any[]) {
    left.className = 'left';
    left.innerText = 'Waiting...';
    right.innerText = '';
    resultDiv.style.display = 'block';

    const tStart = Date.now();
    const result = canister[name].apply(actor, args);
    const duration = (Date.now() - tStart) / 1000;
    right.innerText = `(${duration}s)`;
    return result;
  }

  function callAndRender(actor: Actor, args: any[]) {
    (async () => {
      const callResult = await call(actor, args);
      let result: any;
      if (idlFunc.retTypes.length === 0) {
        result = [];
      } else if (idlFunc.retTypes.length === 1) {
        result = [callResult];
      } else {
        result = callResult;
      }
      left.innerHTML = '';

      const containers: HTMLDivElement[] = [];
      const textContainer = document.createElement('div');
      containers.push(textContainer);
      left.appendChild(textContainer);
      const text = encodeStr(IDL.FuncClass.argsToString(idlFunc.retTypes, result));
      textContainer.innerHTML = text;
      const showArgs = encodeStr(IDL.FuncClass.argsToString(idlFunc.argTypes, args));
      log(`› ${name}${showArgs}`);
      log(text);

      const uiContainer = document.createElement('div');
      containers.push(uiContainer);
      uiContainer.style.display = 'none';
      left.appendChild(uiContainer);
      idlFunc.retTypes.forEach((arg, ind) => {
        const box = UI.renderInput(arg);
        box.render(uiContainer);
        UI.renderValue(arg, box, result[ind]);
      });

      const jsonContainer = document.createElement('div');
      containers.push(jsonContainer);
      jsonContainer.style.display = 'none';
      left.appendChild(jsonContainer);
      jsonContainer.innerText = JSON.stringify(callResult);

      let i = 0;
      left.addEventListener('click', () => {
        containers[i].style.display = 'none';
        i = (i + 1) % 3;
        containers[i].style.display = 'block';
      });
    })().catch(err => {
      left.className += ' error';
      left.innerText = err.message;
      throw err;
    });
  }

  random.addEventListener('click', () => {
    const args = inputs.map(arg => arg.parse({ random: true }));
    const isReject = inputs.some(arg => arg.isRejected());
    if (isReject) {
      return;
    }
    callAndRender(canister, args);
  });

  button.addEventListener('click', () => {
    const args = inputs.map(arg => arg.parse());
    const isReject = inputs.some(arg => arg.isRejected());
    if (isReject) {
      return;
    }
    callAndRender(canister, args);
  });
}

function encodeStr(str: string) {
  const escapeChars: Record<string, string> = {
    ' ': '&nbsp;',
    '<': '&lt;',
    '>': '&gt;',
    '\n': '<br>',
  };
  const regex = new RegExp('[ <>\n]', 'g');
  return str.replace(regex, m => {
    return escapeChars[m];
  });
}

function log(content: Element | string) {
  const consoleEl = document.getElementsByClassName('console')[0];
  const line = document.createElement('div');
  line.className = 'console-line';
  if (content instanceof Element) {
    line.appendChild(content);
  } else {
    line.innerHTML = content;
  }
  consoleEl.appendChild(line);
}

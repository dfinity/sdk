import { UI } from '../../out';

export function render(id, actor, canister) {
  document.getElementById('title').innerText = `Service ${id}`;
  for (const [name, func] of Object.entries(actor._fields)) {
    renderMethod(name, func, canister[name]);
  }
  const console = document.createElement("div");
  console.className = 'console';
  document.body.appendChild(console);
}

function renderMethod(name, idl_func, f) {
  const item = document.createElement("li");

  const sig = document.createElement("div");
  sig.className = 'signature';
  sig.innerHTML = `${name}: ${idl_func.display()}`;
  item.appendChild(sig);

  const button = document.createElement("button");
  button.className = 'btn';
  if (idl_func.annotations.includes('query')) {
    button.innerText = 'Query';
  } else {
    button.innerText = 'Call';
  }

  const inputs = [];
  idl_func.argTypes.forEach((arg, i) => {
    const input = UI.renderInput(arg, item);
    inputs.push(input);
  });

  item.appendChild(button);

  const result = document.createElement("div");
  result.className = 'result';
  const left = document.createElement("span");
  left.className = 'left';
  const right = document.createElement("span");
  right.className = 'right';
  result.appendChild(left);
  result.appendChild(right);
  item.appendChild(result);

  const list = document.getElementById("methods");
  list.append(item);

  button.addEventListener("click", function() {
    inputs.forEach(arg => arg.dispatchEvent(UI.parseEvent));
    const isReject = inputs.some(arg => arg.classList.contains('reject'));
    if (isReject) {
      return;
    }
    
    left.className = 'left';
    left.innerText = 'Waiting...';
    right.innerText = ''
    result.style.display = 'block';
    (async function () {
      const args = inputs.map((arg, i) => idl_func.argTypes[i].stringToValue(arg.value));
      const t_before = Date.now();
      const result = await f.apply(null, args);
      const duration = (Date.now() - t_before)/1000;
      var show_result = '';
      if (idl_func.retTypes.length === 1) {
        show_result = idl_func.retTypes[0].valueToString(result);
      } else {
        show_result = valuesToString(idl_func.retTypes, result);
      }
      left.innerText = show_result;
      right.innerText = `(${duration}s)`;

      const show_args = valuesToString(idl_func.argTypes, args);
      log(`› ${name}${show_args}`);
      log(show_result);
    })().catch(err => {
      left.className += ' error';
      left.innerText = err.name + ': ' + err.message;
    });
  });
};

function zipWith(xs, ys, f) {
  return xs.map((x, i) => f(x, ys[i]));
}

function valuesToString(types, values) {
  return '(' + zipWith(types, values, ((t, v) => t.valueToString(v))).join(', ') + ')';
}

function log(content) {
  const console = document.getElementsByClassName("console")[0];
  const line = document.createElement("div");
  line.className = 'console-line';
  if (content instanceof Element) {
    line.appendChild(content);
  } else {
    line.innerText = content;
  }
  console.appendChild(line);
}

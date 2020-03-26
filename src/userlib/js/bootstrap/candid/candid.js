import * as UI from './candid-ui';
import './candid.css';

export function render(id, canister) {
  document.getElementById('title').innerText = `Service ${id}`;
  for (const [name, func] of Object.entries(canister.__actorInterface())) {
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

  const inputs = [];
  idl_func.argTypes.forEach((arg, i) => {
    const inputbox = UI.renderInput(arg);
    inputs.push(inputbox);
    inputbox.render(item);
  });

  const button = document.createElement("button");
  button.className = 'btn';
  if (idl_func.annotations.includes('query')) {
    button.innerText = 'Query';
  } else {
    button.innerText = 'Call';
  }  
  item.appendChild(button);

  const random = document.createElement("button");
  random.className = 'btn';
  random.innerText = 'Lucky';
  item.appendChild(random);

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

  function call(args) {
    left.className = 'left';
    left.innerText = 'Waiting...';
    right.innerText = ''
    result.style.display = 'block';
    (async function () {
      const t_before = Date.now();
      const result = await f.apply(null, args);
      const duration = (Date.now() - t_before)/1000;
      var show_result = '';
      if (idl_func.retTypes.length === 1) {
        show_result = idl_func.retTypes[0].valueToString(result);
      } else {
        show_result = valuesToString(idl_func.retTypes, result);
      }
      show_result = encodeStr(show_result);
      left.innerHTML = show_result;
      right.innerText = `(${duration}s)`;

      const show_args = encodeStr(valuesToString(idl_func.argTypes, args));
      log(`› ${name}${show_args}`);
      log(show_result);
    })().catch(err => {
      left.className += ' error';
      left.innerText = err.message;
    });    
  }
  
  random.addEventListener("click", function() {
    const args = inputs.map(arg => arg.parse({ random: true }));
    const isReject = inputs.some(arg => arg.isRejected());
    if (isReject) {
      return;
    }    
    call(args);
  });
  
  button.addEventListener("click", function() {
    const args = inputs.map(arg => arg.parse());
    const isReject = inputs.some(arg => arg.isRejected());
    if (isReject) {
      return;
    }
    call(args);
  });
};

function zipWith(xs, ys, f) {
  return xs.map((x, i) => f(x, ys[i]));
}

function encodeStr(str) {
  const escapeChars = {
    ' ' : '&nbsp;',
    '<' : '&lt;',
    '>' : '&gt;',
    '\n' : '<br>',
  };
  const regex = new RegExp('[ <>\n]', 'g');
  return str.replace(regex, m => {
    return escapeChars[m];
  });
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
    line.innerHTML = content;
  }
  console.appendChild(line);
}

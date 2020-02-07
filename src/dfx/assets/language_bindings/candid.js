import canister from 'ic:canisters/{project_name}';
import candid from 'ic:idl/{project_name}';
import { IDL } from 'ic:userlib';

document.getElementById('title').innerText = 'Service {project_name}';

const actor = candid({IDL});
for (let [name, func] of Object.entries(actor._fields)) {
  renderMethod(name, func, canister[name]);
}
const console = document.createElement("div");
console.className = 'console';
document.body.appendChild(console);

function renderMethod(name, idl_func, f) {
  const status = document.createElement("div");
  status.className = 'status';

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

  const arg_length = idl_func.argTypes.length;
  for (var i = 0; i < arg_length; i++) {
    const t = idl_func.argTypes[i];
    const arg = document.createElement("input");
    arg.className = 'argument';
    arg.id = `${name}_arg${i}`;
    item.appendChild(arg);

    arg.addEventListener("focus", function () {
      arg.className = 'argument';
    });
    arg.addEventListener("blur", function() {
      try {
        if (arg.value === '') {
          return;
        }
        const value = JSON.parse(arg.value);
        if (!t.covariant(value)) {
          throw new Error(`Expect ${arg.value} to be of type ${t.display()}`);
        }
        status.style.display = 'none';
        button.disabled = false;
      } catch(err) {
        arg.className += ' reject';        
        status.style.display = 'block';
        button.disabled = true;        
        status.innerHTML = 'ParseError: ' + err.message;
      };
    });
  }

  item.appendChild(button);
  item.appendChild(status);

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
    left.className = 'left';
    left.innerText = 'Waiting...';
    right.innerText = ''
    result.style.display = 'block';
    (async function () {
      var args = [];
      for (var i = 0; i < arg_length; i++) {
        const arg = document.getElementById(`${name}_arg${i}`).value;
        args.push(JSON.parse(arg));
      }
      const t_before = Date.now();
      const result = await f.apply(null, args);
      const duration = (Date.now() - t_before)/1000;
      left.innerText = JSON.stringify(result);
      right.innerText = `(${duration}s)`;

      const show_args = args.map(arg => JSON.stringify(arg)).join(', ');
      log(`› ${name}(${show_args})`);
      log(JSON.stringify(result));
    })().catch(err => {
      left.className += ' error';
      left.innerText = err.name + ': ' + err.message;
    });
  });
};

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

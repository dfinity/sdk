import canister from 'ic:canisters/{project_name}';
import candid from 'ic:idl/{project_name}';
import { IDL } from 'ic:userlib';

document.getElementById('title').innerText = 'Service {project_name}';

const actor = candid({IDL});
for (let [name, func] of Object.entries(actor._fields)) {
  const sig = showArgs(func.argTypes) + " &rarr; " + showArgs(func.retTypes);
  renderMethod(canister[name], name, sig, func.argTypes.length);
}


// The following functions can go into userlib.
// But keeping it here also has benefits: user can change the code to fit
// whatever style we like.

function showArgs(args) {
  return '('.concat(args.map(arg => arg.name)) + ')';
}

function renderMethod(f, method, sig, arg_length) {
  const list = document.getElementById("methods");
  const item = document.createElement("li");
  var html = `<div class="signature">${method}: ${sig}</div>`;
  for (var i = 0; i < arg_length; i++) {
    html += `<input class='argument' id='${method}_arg${i}'></input> `;
  };
  html += `<button class='btn' id='${method}'>Call</button>`;
  html += `<div class='result' id='${method}_result'><span class='left'></span><span class='right'></span></div>`;
  item.innerHTML = html;
  list.append(item);
  
  document.getElementById(method).addEventListener("click", function() {
    const field = `${method}_result`;
    const dom = document.getElementById(field);
    const left = dom.getElementsByClassName('left')[0];
    const right = dom.getElementsByClassName('right')[0];
    left.className = 'left';
    left.innerText = 'Waiting...';
    right.innerText = '';
    dom.style.display = 'block';
    (async function () {
      var args = [];
      for (var i = 0; i < arg_length; i++) {
        const arg = document.getElementById(`${method}_arg${i}`).value;
        args.push(JSON.parse(arg));
      }
      const t_before = Date.now();
      const result = await f.apply(null, args);
      const duration = (Date.now() - t_before)/1000;
      left.innerText = JSON.stringify(result);
      right.innerText = `(${duration}s)`;
    })().catch(err => {
      left.className += ' error';
      left.innerText = err.name + ': ' + err.message;
    });
  });
};

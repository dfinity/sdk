// Runs tape tests in browser and writes results to the page's body

const tape = require('tape')

document.body.innerText = ''
tape.createStream({ objectMode: true }).on('data', data => {
  document.body.innerText += JSON.stringify(data) + '\n\n'
});

// Tape automatically runs these
require('../unit-tests/idl')
require('../unit-tests/serialisation')

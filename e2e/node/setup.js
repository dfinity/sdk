module.exports = function() {
  // Run the Replica by using `dfx start`.
  const { spawn } = require('child_process');
  global.replicaProcess = spawn('dfx', ['replica', '--port=8080'], { stdio: 'inherit' });

  return new Promise(resolve => setTimeout(resolve, 5000));
};

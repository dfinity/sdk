module.exports = function(config, a, b, c) {
  // Sending SIGINT here since the Replica process isn't actually the replica
  // but `dfx replica`, which should be interupted to clean up properly.
  // (ie. if directly killed it will keep the replica process running in
  // the background).
  global.replicaProcess.kill('SIGTERM');

  // Give the replica a second to gather its things and quit.
  // We unfortunately cannot exit our own process here because we don't know
  // the status of the tests (fail/success).
  return new Promise(resolve => setTimeout(resolve, 1000));
};

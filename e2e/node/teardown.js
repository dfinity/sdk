module.exports = function() {
  // Sending SIGINT here since the Replica process isn't actually the replica
  // but `dfx replica`, which should be interupted to clean up properly.
  // (ie. if directly killed it will keep the replica process running in
  // the background).
  global.replicaProcess.kill('SIGTERM');

  // Just process.exit() after a second if nothing else. If we're here,
  // the tests succeeded anyway.
  return new Promise(resolve => setTimeout(resolve, 1000)).then(() => {
    process.exit(0);
  });
};

{ napalm }:

napalm.buildPackage ./. {
  npmCommands = [
    "npm install"
    "npm test"
  ];
}

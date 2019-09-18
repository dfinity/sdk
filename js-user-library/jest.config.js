module.exports = {
  bail: false,
  setupFiles: [
    "./src/test-setup",
    "whatwg-fetch",
  ],
  testEnvironment: "jsdom",
  testPathIgnorePatterns: [
    "/node_modules/",
    "/out/",
  ]
};

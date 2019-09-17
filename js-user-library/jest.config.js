module.exports = {
  bail: false,
  setupFiles: [
    "whatwg-fetch",
  ],
  testEnvironment: "jsdom",
  testPathIgnorePatterns: [
    "/node_modules/",
    "/out/",
  ]
};

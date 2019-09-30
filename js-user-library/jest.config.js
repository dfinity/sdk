module.exports = {
  bail: false,
  setupFiles: [
    "./src/test-setup",
  ],
  setupFilesAfterEnv: [
    "jest-expect-message",
  ],
  testEnvironment: "jsdom",
  testPathIgnorePatterns: [
    "/node_modules/",
    "/out/",
    "/src/IDL-ts/",
  ]
};

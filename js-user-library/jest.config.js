module.exports = {
  bail: false,
  setupFiles: [
    "./test-setup",
  ],
  setupFilesAfterEnv: [
    "jest-expect-message",
  ],
  testEnvironment: "jsdom",
  testPathIgnorePatterns: [
    "/node_modules/",
    "/out/",
  ]
};

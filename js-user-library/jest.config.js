module.exports = {
  bail: false,
  setupFilesAfterEnv: [
    "jest-expect-message",
  ],
  testEnvironment: "jsdom"
};

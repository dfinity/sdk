module.exports = {
  bail: false,
  testTimeout: 60001,
  globalSetup: './setup',
  globalTeardown: './teardown',
  setupFiles: [
    "./test-setup",
  ],
  setupFilesAfterEnv: [
    "jest-expect-message",
  ],
  // Since we're running e2e tests, ALL typescript files are up for grab.
  testMatch: [
    "**/*.ts"
  ],
  testPathIgnorePatterns: [
    "/node_modules/",
    "/utils/",
  ],
  transform: {
    "^.+\\.ts$": "ts-jest"
  }
};

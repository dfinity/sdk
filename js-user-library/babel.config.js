module.exports = {
  presets: [
    [
      "@babel/preset-env",
      {
        targets: {
          browsers: "defaults",
          node: "current",
        },
      },
    ],
  ],
};

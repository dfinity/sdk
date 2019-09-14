module.exports = {
  presets: [
    [
      "@babel/preset-env",
      {
        corejs: 3,
        targets: {
          // browsers: "defaults", // FIXME: we may need to add regenerator-runtime
          node: "current",
        },
        "useBuiltIns": "usage",
      },
    ],
  ],
};

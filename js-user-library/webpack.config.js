const webpack = require("webpack");

const config = {
  mode: "development",
  entry: [
    "./out/index.js",
  ],
  devtool: "inline-source-map",
  output: {
    libraryTarget: "umd",
  },
};

const nodeConfig = {
  ...config,
  target: "node",
  output: {
    ...config.output,
    filename: "lib.node.js",
  },
  plugins: [
    new webpack.ProvidePlugin({
      crypto: "@trust/webcrypto",
      fetch: "node-fetch",
      TextEncoder: ["text-encoding", "TextEncoder"],
    }),
  ],
};

const webConfig = {
  ...config,
  target: "web",
  output: {
    ...config.output,
    filename: "lib.web.js",
  },
};

module.exports = [
  nodeConfig,
  webConfig,
];

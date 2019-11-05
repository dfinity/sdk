const webpack = require("webpack");

const config = {
  mode: "development",
  entry: [
    "./src/index",
  ],
  devtool: "inline-source-map",
  output: {
    libraryTarget: "umd",
  },
  module: {
    rules: [
      {
        test: /\.(ts)$/,
        exclude: /node_modules/,
        loader: "babel-loader",
      },
    ],
  },
  resolve: {
    extensions: [
      ".js",
      ".ts",
    ],
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

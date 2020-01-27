const path = require("path");
const webpack = require("webpack");
const TerserPlugin = require('terser-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

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

const prodConfig = {
  ...webConfig,
  target: "web",
  output: {
    ...webConfig.output,
    filename: "lib.prod.js",
  },
  devtool: "source-map",
  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        cache: true,
        parallel: true,
        sourceMap: true, // Must be set to true if using source-maps in production
        terserOptions: {
          ecma: 8,
          minimize: true,
          comments: false
          // https://github.com/webpack-contrib/terser-webpack-plugin#terseroptions
        }
      }),
    ],
  }
};

const bootstrapConfig = {
  ...webConfig,
  entry: "./bootstrap/index.js",
  target: "web",
  output: {
    ...webConfig.output,
    path: path.resolve(__dirname, "./dist/bootstrap"),
    filename: "index.js",
  },
  devtool: "none",
  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        cache: true,
        parallel: true,
        sourceMap: true, // Must be set to true if using source-maps in production
        terserOptions: {
          ecma: 8,
          minimize: true,
          comments: false
          // https://github.com/webpack-contrib/terser-webpack-plugin#terseroptions
        }
      }),
    ],
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'bootstrap/index.html',
      filename: 'index.html'
    }),
    new CopyWebpackPlugin([{
        from: 'bootstrap/dfinity.png',
        to: 'favicon.ico',
      }]),
  ]
};

module.exports = [
  nodeConfig,
  webConfig,
  prodConfig,
  bootstrapConfig,
];

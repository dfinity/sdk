const fs = require("fs");
const path = require("path");
const TerserPlugin = require('terser-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

// If we're in Nix, we need to let the resolution works normally.
const agentPath = path.join(__dirname, '../agent/javascript/src');
const resolve = fs.existsSync(agentPath) ? {
  alias: {
    '@dfinity/agent': agentPath,
  }
} : {};

module.exports = {
  mode: "production",
  entry: "./src/index.js",
  target: "web",
  output: {
    path: path.resolve(__dirname, "./dist"),
    filename: "index.js",
  },
  resolve,
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
  module: {
    rules: [{
      test: /\.css$/,
      use: ['style-loader', 'css-loader']
    }]
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'src/index.html',
      filename: 'index.html'
    }),
    new HtmlWebpackPlugin({
      template: 'src/candid/index.html',
      filename: 'candid/index.html'
    }),
    new CopyWebpackPlugin([{
        from: 'src/dfinity.png',
        to: 'favicon.ico',
      }]),
  ]
};

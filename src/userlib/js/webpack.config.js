const path = require("path");
const TerserPlugin = require('terser-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const bootstrapConfig = {
  mode: "production",
  entry: "./bootstrap/index.js",
  target: "web",
  output: {
    libraryTarget: "umd",
    path: path.resolve(__dirname, "./dist/bootstrap"),
    filename: "index.js",
  },
  resolve: {
    alias: {
      '@internet-computer/userlib': path.resolve('src'),
    },
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
  module: {
    rules: [{
      test: /\.css$/,
      use: ['style-loader', 'css-loader']
    }]
  },  
  plugins: [
    new HtmlWebpackPlugin({
      template: 'bootstrap/index.html',
      filename: 'index.html'
    }),
    new HtmlWebpackPlugin({
      template: 'bootstrap/candid/index.html',
      filename: 'candid/index.html'
    }),
    new CopyWebpackPlugin([{
        from: 'bootstrap/dfinity.png',
        to: 'favicon.ico',
      }]),
  ]
};

module.exports = [
  bootstrapConfig,
];

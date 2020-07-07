const fs = require('fs');
const path = require('path');
const TerserPlugin = require('terser-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');

module.exports = {
  mode: 'production',
  entry: {
    bootstrap: './src/index.ts',
    candid: './src/candid/candid.js',
    worker: './src/worker.ts',
  },
  target: 'web',
  output: {
    // This is necessary to allow internal apps to bundle their own code with
    // webpack which may conflict with us.
    jsonpFunction: '__dfinityJsonp',
    path: path.resolve(__dirname, './dist'),
    filename: '[name].js',
  },
  resolve: {
    plugins: [new TsconfigPathsPlugin({ configFile: './tsconfig.json' })],
    extensions: ['.tsx', '.ts', '.js'],
  },
  devtool: 'source-map',
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
          comments: false,
          // https://github.com/webpack-contrib/terser-webpack-plugin#terseroptions
        },
      }),
    ],
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      },
      {
        test: /\.tsx?$/,
        use: ['ts-loader'],
      },
    ],
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: 'src/index.html',
      filename: 'index.html',
      chunks: ['bootstrap'],
    }),
    new HtmlWebpackPlugin({
      template: 'src/worker.html',
      filename: 'worker.html',
      chunks: ['worker'],
    }),
    new HtmlWebpackPlugin({
      template: 'src/candid/index.html',
      filename: 'candid/index.html',
      // TODO: change candid.js to candid.ts, and make it a proper bootstrap, and
      //       change this chunk to candid.
      chunks: ['bootstrap'],
    }),
    new CopyWebpackPlugin([
      {
        from: 'src/dfinity.png',
        to: 'favicon.ico',
      },
    ]),
  ],
};

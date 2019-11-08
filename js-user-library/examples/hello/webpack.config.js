const path = require("path");

module.exports = {
  mode: "development",
  entry: "./src/hello/index.js",
  devtool: 'inline-source-map',
  output: { path: path.resolve(__dirname, "public") },
};

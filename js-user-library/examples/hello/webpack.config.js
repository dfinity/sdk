const path = require("path");

module.exports = {
  mode: "development",
  entry: "./src/hello/index.js",
  devtool: 'inline-source-map',
  output: {
    filename: "index.js",
    path: path.resolve(__dirname, "public"),
  },
};

const path = require("path");
const TerserPlugin = require("terser-webpack-plugin");
const dfxJson = require("./dfx.json");

// List of all aliases for canisters. This creates the module alias for
// the `import ... from "ic:canisters/xyz"` where xyz is the name of a
// canister.
const aliases = Object.entries(dfxJson.canisters)
  .reduce((acc, [name,value]) => {
    const outputRoot = path.join(__dirname, dfxJson.defaults.build.output, name);
    return {
      ...acc,
      ["ic:canisters/" + name]: path.join(outputRoot, name + ".js"),
      ["ic:idl/" + name]: path.join(outputRoot, name + ".did.js"),
    };
  }, {});

/**
 * Generate a webpack configuration for a canister.
 */
function generateWebpackConfigForCanister(name, info) {
  if (typeof info.frontend !== 'object') {
    return;
  }

  const inputRoot = __dirname;

  return {
    mode: "production",
    entry: {
      index: path.join(inputRoot, info.frontend.entrypoint),
    },
    devtool: "source-map",
    optimization: {
      minimize: true,
      minimizer: [new TerserPlugin()],
    },
    resolve: {
      alias: aliases,
    },
    output: {
      filename: "[name].js",
      path: path.join(__dirname, info.frontend.output),
    },

// Depending in the language or framework you are using for
// front-end development, add module loaders to the default 
// webpack configuration. For example, if following the
// "Adding a stylesheet" tutorial, uncomment the following lines:
// module: {
//  rules: [
//    { test: /\.(js|ts)x?$/, loader: "ts-loader" },
//    { test: /\.css$/, use: ['style-loader','css-loader'] }
//  ]
// },
    plugins: [
    ],
  };
}

// If you have additional webpack configurations you want to build
//  as part of this configuration, add them to the section below.
module.exports = [
  ...Object.entries(dfxJson.canisters).map(([name, info]) => {
    return generateWebpackConfigForCanister(name, info);
  }).filter(x => !!x),
];

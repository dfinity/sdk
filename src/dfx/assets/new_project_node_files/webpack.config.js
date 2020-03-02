const path = require("path");
const TerserPlugin = require("terser-webpack-plugin");
const dfxJson = require("./dfx.json");

// List of all aliases for canisters. This creates the module alias for
// the `import ... from "ic:canisters/xyz"` where xyz is the name of a
// canister.
const aliases = Object.entries(dfxJson.canisters).reduce((acc, [name,value]) => {
  const outputRoot = path.join(__dirname, dfxJson.defaults.build.output, name);
  const filename = path.basename(value.main, ".mo");
  return {
    ...acc,
    ["ic:canisters/" + name]: path.join(outputRoot, filename + ".js"),
    ["ic:idl/" + name]: path.join(outputRoot, filename + ".did.js"),
  };
}, {
  // This will later point to the userlib from npm, when we publish the userlib.
  "ic:userlib": path.join(
    process.env["HOME"],
    ".cache/dfinity/versions",
    dfxJson.dfx || process.env["DFX_VERSION"],
    "js-user-library/",
  ),
});

/**
 * Generate a webpack configuration for a canister.
 */
function generateWebpackConfigForCanister(name, info) {
  if (typeof info.frontend !== 'object') {
    return;
  }

  const outputRoot = path.join(__dirname, dfxJson.defaults.build.output, name);
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
      path: path.join(outputRoot, "assets"),
    },
    plugins: [
    ],
  };
}

// If you have webpack configurations you want to build as part of this
// config, add them here.
module.exports = [
  ...Object.entries(dfxJson.canisters).map(([name, info]) => {
    return generateWebpackConfigForCanister(name, info);
  }).filter(x => !!x),
];

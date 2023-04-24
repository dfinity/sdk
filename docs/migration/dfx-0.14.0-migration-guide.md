# 0.14.0 Migration Guide - Environment Variables

This is not an immediate breaking change, but there will be a standardization around environment variables. Backwards compatibility will be supported until 0.16.0, but we recommend updating your projects to use the new environment variables.

## Context

Previously, `webpack.config.js` included an `initCanisterEnv` that parsed canister\*ids.json files and mapped them to environment variables. The pattern used there was `<CANISTER_NAME_UPPERCASE>_CANISTER_ID`. This was not consistent with the pattern used in the `dfx` internals, which used `CANISTER_ID_<canister
_name_case_agnostic>`. This was confusing for users, and since uppercase environment variables is a best practice, we have decided to standardize on the `CANISTER_ID_<CANISTER_NAME_UPPERCASE>` pattern across the board.

## Changes

Starting in `dfx 0.14.0`, the auto-generated `index.js` from `dfx generate` will use the new `CANISTER_ID_<CANISTER_NAME_UPPERCASE>`, with a fallback to the old `<CANISTER_NAME_UPPERCASE>_CANISTER_ID` pattern. This will look like this:

```js
/* CANISTER_ID is replaced by webpack based on node environment
 * Note: canister environment variable will be standardized as
 * process.env.CANISTER_ID_<CANISTER_NAME_UPPERCASE>
 * beginning in dfx 0.16.0
 */
export const canisterId =
  process.env.CANISTER_ID_HELLO_BACKEND ||
  process.env.HELLO_BACKEND_CANISTER_ID;
```

This will not break an existing application, but if you are using the environment variables elsewhere throughout your application, you should switch to the new pattern.

## Webpack / Bundler Migration

If your frontend project is using the old environment variables, you will need to update your `webpack.config.js` file to use the new `CANISTER_ID_<CANISTER_NAME_UPPERCASE>` variables, or to use the new webpack config provided in the starter project. We recommend using the new, simplified webpack config, which is provided below:

```js
require("dotenv").config();
const path = require("path");
const webpack = require("webpack");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const TerserPlugin = require("terser-webpack-plugin");
const CopyPlugin = require("copy-webpack-plugin");

const isDevelopment = process.env.NODE_ENV !== "production";

const frontendDirectory = "<your_frontend_directory>";

const frontend_entry = path.join("src", frontendDirectory, "src", "index.html");

module.exports = {
  target: "web",
  mode: isDevelopment ? "development" : "production",
  entry: {
    // The frontend.entrypoint points to the HTML file for this build, so we need
    // to replace the extension to `.js`.
    index: path.join(__dirname, frontend_entry).replace(/\.html$/, ".js"),
  },
  devtool: isDevelopment ? "source-map" : false,
  optimization: {
    minimize: !isDevelopment,
    minimizer: [new TerserPlugin()],
  },
  resolve: {
    extensions: [".js", ".ts", ".jsx", ".tsx"],
    fallback: {
      assert: require.resolve("assert/"),
      buffer: require.resolve("buffer/"),
      events: require.resolve("events/"),
      stream: require.resolve("stream-browserify/"),
      util: require.resolve("util/"),
    },
  },
  output: {
    filename: "index.js",
    path: path.join(__dirname, "dist", frontendDirectory),
  },

  // Depending in the language or framework you are using for
  // front-end development, add module loaders to the default
  // webpack configuration. For example, if you are using React
  // modules and CSS as described in the "Adding a stylesheet"
  // tutorial, uncomment the following lines:
  // module: {
  //  rules: [
  //    { test: /\.(ts|tsx|jsx)$/, loader: "ts-loader" },
  //    { test: /\.css$/, use: ['style-loader','css-loader'] }
  //  ]
  // },
  plugins: [
    new HtmlWebpackPlugin({
      template: path.join(__dirname, frontend_entry),
      cache: false,
    }),
    new webpack.EnvironmentPlugin([
      ...Object.keys(process.env).filter((key) => {
        if (key.includes("CANISTER")) return true;
        if (key.includes("DFX")) return true;
        return false;
      }),
    ]),
    new webpack.ProvidePlugin({
      Buffer: [require.resolve("buffer/"), "Buffer"],
      process: require.resolve("process/browser"),
    }),
    new CopyPlugin({
      patterns: [
        {
          from: `src/${frontendDirectory}/src/.ic-assets.json*`,
          to: ".ic-assets.json5",
          noErrorOnMissing: true,
        },
      ],
    }),
  ],
  // proxy /api to port 4943 during development.
  // if you edit dfx.json to define a project-specific local network, change the port to match.
  devServer: {
    proxy: {
      "/api": {
        target: "http://127.0.0.1:4943",
        changeOrigin: true,
        pathRewrite: {
          "^/api": "/api",
        },
      },
    },
    static: path.resolve(__dirname, "src", frontendDirectory, "assets"),
    hot: true,
    watchFiles: [path.resolve(__dirname, "src", frontendDirectory)],
    liveReload: true,
  },
};
```

If you prefer to modify an existing webpack config, you will need to update the `initCanisterEnv` function to use the new `CANISTER_ID_<CANISTER_NAME_UPPERCASE>` variables.

For example, if your frontend project is using the following mapping of environment variables inside of `initCanisterEnv`:

```js
prev[canisterName.toUpperCase() + "_CANISTER_ID"] = canisterDetails[network];
````

You should update your `webpack.config.js` file to use the new `CANISTER_ID_<CANISTER_NAME_UPPERCASE>` variables:

```js
prev[`CANISTER_ID_${canisterName.toUpperCase()}`] = canisterDetails[network];
```

Alternately, if you are using the new `.env` file support, you can install `dotenv` and imitate the new webpack config provided above.

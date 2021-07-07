const path = require("path");
const webpack = require("webpack");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const TerserPlugin = require("terser-webpack-plugin");

const canisters = require(path.resolve(".dfx", "local", "canister_ids.json"));
let prodCanisters;

try {
  prodCanisters = require(path.resolve("canister_ids.json"));
  for (const canister in prodCanisters) {
    if (Object.hasOwnProperty.call(prodCanisters, canister)) {
      const canisterId = prodCanisters[canister];
    }
  }
} catch (error) {
  console.log("No production canister_ids.json found. Continuing with local");
}

const canisterMap = new Map();
for (const canister in canisters) {
  if (Object.hasOwnProperty.call(canisters, canister)) {
    const canisterId = canisters[canister];
    const networkName = process.env["DFX_NETWORK"] || "local";

    canisterMap.set(`${canister.toUpperCase()}_CANISTER_ID`, canisterId[networkName]);
  }
}

const isDevelopment = process.env.NODE_ENV !== "production";
const asset_entry = path.join("src", "{project_name}_assets", 'src', "index.html");

module.exports = {
  mode: isDevelopment ? "development" : "production",
  entry: {
    // The frontend.entrypoint points to the HTML file for this build, so we need
    // to replace the extension to `.js`.
    index: path
      .join(__dirname, asset_entry)
      .replace(/\.html$/, ".js"),
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
    path: path.join(__dirname, "dist", "{project_name}_assets"),
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
      template: path.join(__dirname, asset_entry),
      filename: "index.html",
      chunks: ["index"],
    }),
    new webpack.DefinePlugin({
      "process.env": {
        "NODE_ENV": `${process.env.node_env}`,
        {project_name_uppercase}_CANISTER_ID: `"${canisterMap.get("{project_name_uppercase}_CANISTER_ID")}"`
      },
    }),
    new webpack.ProvidePlugin({
      Buffer: [require.resolve("buffer/"), "Buffer"],
      process: require.resolve("process/browser"),
    }),
  ],
  // proxy /api to port 8000 during development
  devServer: {
    proxy: {
      "/api": {
        target: "http://localhost:8000",
        changeOrigin: true,
        pathRewrite: {
          "^/api": "/api",
        },
      },
    },
  },
};

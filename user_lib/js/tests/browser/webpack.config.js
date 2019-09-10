// Webpackifies the unit tests

module.exports = {
  entry: './tests/browser/index.js',
  output: {
    filename: 'index.min.js',
  },
  mode: 'development',
  node: {
    fs: 'empty'
  },
}

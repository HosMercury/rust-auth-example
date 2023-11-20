const path = require('path');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const TerserPlugin = require('terser-webpack-plugin');
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin');

module.exports = {
  watch: true,
  watchOptions: {
    ignored: ['**/src', '**/node_modules'],
  },
  entry: path.resolve(__dirname, 'assets/main.js'),
  output: {
    path: path.resolve(__dirname, 'public'),
    filename: 'script-[chunkhash:7].js',
    clean: true,
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: [MiniCssExtractPlugin.loader, 'css-loader'],
      },
    ],
  },
  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        extractComments: false,
        terserOptions: {
          format: {
            comments: false,
          },
        },
      }),
      new CssMinimizerPlugin(),
    ],
  },
  plugins: [
    new MiniCssExtractPlugin({ filename: 'style-[chunkhash:7].css' }),
    new CssMinimizerPlugin(),
  ],
};

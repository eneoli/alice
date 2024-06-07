const path = require('path');

const HtmlWebpackPlugin = require('html-webpack-plugin');
const EsLintWebpackPlugin = require("eslint-webpack-plugin");
const ForkTsCheckerWebpackPlugin = require('fork-ts-checker-webpack-plugin');

const isProduction = process.env.NODE_ENV === 'production';

const config = {
    entry: {
        app: './src/index.tsx',
    },
    output: {
        path: path.resolve(__dirname, '..', 'dist'),
        filename: '[name].[contenthash].js',
        clean: true,
    },
    devServer: {
        host: '0.0.0.0',
        hot: true,
    },
    plugins: [
        new EsLintWebpackPlugin({
            extensions: ['js', 'jsx', 'ts', 'tsx'],
            configType: 'flat',
            eslintPath: 'eslint/use-at-your-own-risk',
        }),
        new ForkTsCheckerWebpackPlugin(),
        new HtmlWebpackPlugin({ template: 'index.html' }),
    ],
    module: {
        rules: [
            {
                test: /\.(ts|tsx)$/i,
                loader: 'babel-loader',
                exclude: ['/node_modules/'],
            },
        ],
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.jsx', '.js', 'mjs', 'cjs'],
    },
    experiments: {
        asyncWebAssembly: true,
    },
    performance: {
        hints: false
    },
    stats: {
        assets: false,
        moduleAssets: false,
        nestedModules: false,
        cachedModules: false,
        runtimeModules: false,
        dependentModules: false,
        cachedAssets: false,
        children: false,
        chunks: false,
        chunkGroups: false,
        chunkModules: false,
        chunkOrigins: false,
        hash: false,
        optimizationBailout: false,
        performance: false,
        modules: false,

        builtAt: true,
        errors: true,
        errorDetails: true,
        errorStack: true,
        logging: 'warn',
    },
};

module.exports = () => {
    if (isProduction) {
        config.mode = 'production';
    } else {
        config.mode = 'development';
    }

    return config;
};

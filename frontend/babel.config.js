module.exports = {
    presets: [
        [
            "@babel/preset-env",
            {
                modules: 'auto',
                useBuiltIns: "usage",
                targets: "> 0.25%, not dead",
                corejs: 3,
            }
        ],
        "@babel/preset-react",
        "@babel/typescript",
    ],
    plugins: [
        "babel-plugin-transform-typescript-metadata",
        [
            "@babel/plugin-transform-typescript",
            {
                allowDeclareFields: true,
                isTSX: true,
            }
        ],
        ["@babel/plugin-proposal-decorators", {"legacy": true}],
        "@babel/plugin-transform-runtime",
        "@babel/plugin-transform-react-jsx",
    ]
};

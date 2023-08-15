module.exports = function override(config, env) {
    config.resolve.fallback = {
        ...config.resolve.fallback,
        "crypto": require.resolve("crypto-js"),
        "buffer": require.resolve("buffer"),
        "stream": require.resolve("stream-browserify")
    };
    return config;
};
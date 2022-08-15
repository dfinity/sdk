jq '.defaults.build.packtool="echo --package describe ./vessel/describe/v1.0.1/src --package rate ./vessel/rate/v1.0.0/src"' dfx.json | sponge dfx.json

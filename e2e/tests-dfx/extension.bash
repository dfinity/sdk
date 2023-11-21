#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
  use_test_specific_cache_root
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "extension install with an empty cache does not create a corrupt cache" {
  dfx cache delete
  dfx extension install nns --version 0.2.1
  dfx_start
}

@test "install extension from official registry" {
  assert_command_fail dfx snsx

  assert_command dfx extension list
  assert_match 'No extensions installed'

  assert_command dfx extension install sns --install-as snsx --version 0.2.1
  # TODO: how to capture spinner message?
  # assert_match 'Successfully installed extension'

  assert_command dfx extension list
  assert_match 'snsx'

  assert_command dfx --help
  assert_match 'snsx.*Toolkit for'

  assert_command dfx snsx --help

  assert_command dfx extension uninstall snsx
  # TODO: how to capture spinner message?
  # assert_match 'Successfully uninstalled extension'

  assert_command dfx extension list
  assert_match 'No extensions installed'
}

@test "manually create extension" {
  assert_command dfx extension list
  assert_match 'No extensions installed'

  CACHE_DIR=$(dfx cache show)
  mkdir -p "$CACHE_DIR"/extensions/test_extension
  echo '#!/usr/bin/env bash

echo testoutput' > "$CACHE_DIR"/extensions/test_extension/test_extension
  chmod +x "$CACHE_DIR"/extensions/test_extension/test_extension

  assert_command_fail dfx extension list
  assert_match "Error.*Cannot load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx extension run test_extension
  assert_match "Error.*Cannot load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx test_extension
  assert_match "Error.*Cannot load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx --help
  assert_match "Error.*Cannot load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx test_extension --help
  assert_match "Error.*Cannot load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  echo "{}" > "$CACHE_DIR"/extensions/test_extension/extension.json

  assert_command_fail dfx extension list
  assert_match "Error.*Cannot load extension manifest.*Failed to parse contents of .*extensions/test_extension/extension.json as json.* missing field .* at line .* column .*"

  assert_command_fail dfx extension run test_extension
  assert_match "Error.*Cannot load extension manifest.*Failed to parse contents of .*extensions/test_extension/extension.json as json.* missing field .* at line .* column .*"

  assert_command_fail dfx test_extension
  assert_match "Error.*Cannot load extension manifest.*Failed to parse contents of .*extensions/test_extension/extension.json as json.* missing field .* at line .* column .*"

  assert_command_fail dfx --help
  assert_match "Error.*Cannot load extension manifest.*Failed to parse contents of .*extensions/test_extension/extension.json as json.* missing field .* at line .* column .*"

  assert_command_fail dfx test_extension --help
  assert_match "Error.*Cannot load extension manifest.*Failed to parse contents of .*extensions/test_extension/extension.json as json.* missing field .* at line .* column .*"

  echo '{
  "name": "test_extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": []
}' > "$CACHE_DIR"/extensions/test_extension/extension.json

  assert_command dfx --help
  assert_match "test_extension.*Test extension for e2e purposes."

  assert_command dfx test_extension --help
  assert_match "Test extension for e2e purposes..*Usage: dfx test_extension"

  assert_command dfx extension list
  assert_match "test_extension"

  assert_command dfx extension run test_extension
  assert_match "testoutput"

  assert_command dfx test_extension
  assert_match "testoutput"

  assert_command dfx extension uninstall test_extension
  # TODO: how to capture spinner message?
  # assert_match 'Successfully uninstalled extension'

  assert_command dfx extension list
  assert_match 'No extensions installed'
}


@test "run with hyphened parameters" {
  CACHE_DIR=$(dfx cache show)
  mkdir -p "$CACHE_DIR"/extensions/test_extension

  cat > "$CACHE_DIR"/extensions/test_extension/test_extension << "EOF"
#!/usr/bin/env bash

if [ "$2" == "--the-param" ]; then
  echo "pamparam the param is $3"
fi
EOF

  chmod +x "$CACHE_DIR"/extensions/test_extension/test_extension

  cat > "$CACHE_DIR"/extensions/test_extension/extension.json <<EOF
{
  "name": "test_extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "subcommands": {
  "abc": {
  "about": "something something",
  "args": {
    "the_param": {
      "about": "this is the param",
      "long": "the-param"
    }
  }
  }
  }
}
EOF

  assert_command dfx test_extension abc --the-param 123
  assert_eq "pamparam the param is 123"
  assert_command dfx extension run test_extension abc --the-param 123
  assert_eq "pamparam the param is 123"
}

@test "run with multiple values for the same parameter" {
  CACHE_DIR=$(dfx cache show)
  mkdir -p "$CACHE_DIR"/extensions/test_extension

  cat > "$CACHE_DIR"/extensions/test_extension/test_extension << "EOF"
#!/usr/bin/env bash

echo $@
EOF

  chmod +x "$CACHE_DIR"/extensions/test_extension/test_extension

  cat > "$CACHE_DIR"/extensions/test_extension/extension.json <<EOF
{
  "name": "test_extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "subcommands": {
  "abc": {
  "about": "something something",
  "args": {
    "the_param": {
      "about": "this is the param",
      "long": "the-param",
      "multiple": true
    },
    "another_param": {
      "about": "this is the param",
      "long": "the-another-param",
      "multiple": true
    }
  }
  }
  }
}
EOF

  assert_command dfx test_extension abc --the-param 123 456 789 --the-another-param 464646
  assert_eq "abc --the-param 123 456 789 --the-another-param 464646 --dfx-cache-path $CACHE_DIR"
  assert_command dfx test_extension abc --the-another-param 464646 --the-param 123 456 789
  assert_eq "abc --the-another-param 464646 --the-param 123 456 789 --dfx-cache-path $CACHE_DIR"
  assert_command dfx extension run test_extension abc --the-param 123 456 789 --the-another-param 464646
  assert_eq "abc --the-param 123 456 789 --the-another-param 464646 --dfx-cache-path $CACHE_DIR"
  assert_command dfx extension run test_extension abc --the-another-param 464646 --the-param 123 456 789
  assert_eq "abc --the-another-param 464646 --the-param 123 456 789 --dfx-cache-path $CACHE_DIR"
}

@test "custom canister types" {
    dfx cache install

    CACHE_DIR=$(dfx cache show)
    mkdir -p "$CACHE_DIR"/extensions/playground
    cat > "$CACHE_DIR"/extensions/playground/extension.json <<EOF
{
  "name": "playground",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/playground",
  "authors": "DFINITY",
  "summary": "Motoko playground for the Internet Computer",
  "categories": [],
  "keywords": [],
  "subcommands": {},
  "canister_types": {
    "playground": {
      "type": "custom",
      "build": "echo the wasm-utils canister is prebuilt",
      "candid": "{{canister_name}}.did",
      "wasm": "{{canister_name}}.wasm",
      "gzip": true
    }
  }
}
EOF
    cat > "$CACHE_DIR"/extensions/playground/playground <<EOF
#!/usr/bin/env bash
echo testoutput
EOF
    chmod +x "$CACHE_DIR"/extensions/playground/playground

    assert_command dfx extension list
    assert_match "playground"

    dfx_new hello
    create_networks_json
    install_asset playground_backend

    cat > dfx.json <<EOF
{
  "canisters": {
      "wasm-utils": {
          "type": "playground"
      }
  },
  "defaults": {
    "build": {
      "args": "",
      "packtool": ""
    }
  },
  "output_env_file": ".env",
  "version": 1
}
EOF

    dfx_start
    assert_command dfx deploy -v
    assert_match 'Backend canister via Candid interface'
}

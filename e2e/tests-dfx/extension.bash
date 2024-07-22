#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
  use_test_specific_cache_root
}

teardown() {
  stop_webserver
  dfx_stop

  standard_teardown
}

@test "extension canister type" {
  dfx_start

  install_asset wasm/identity
  CACHE_DIR=$(dfx cache show)
  mkdir -p "$CACHE_DIR"/extensions/embera
  cat > "$CACHE_DIR"/extensions/embera/extension.json <<EOF
  {
      "name": "embera",
      "version": "0.1.0",
      "homepage": "https://github.com/dfinity/dfx-extensions",
      "authors": "DFINITY",
      "summary": "Test extension for e2e purposes.",
      "categories": [],
      "keywords": [],
      "canister_type": {
       "evaluation_order": [ "wasm" ],
       "defaults": {
         "type": "custom",
         "build": [
           "echo the embera build step for canister {{canister_name}} with candid {{canister.candid}} and main file {{canister.main}} and gzip is {{canister.gzip}}",
           "mkdir -p .embera/{{canister_name}}",
           "cp main.wasm {{canister.wasm}}"
         ],
         "gzip": true,
         "wasm": ".embera/{{canister_name}}/{{canister_name}}.wasm",
         "post_install": [
           "echo the embera post-install step for canister {{canister_name}} with candid {{canister.candid}} and main file {{canister.main}} and gzip is {{canister.gzip}}"
         ],
         "metadata": [
           {
             "name": "metadata-from-extension",
             "content": "the content (from extension definition), gzip is {{canister.gzip}}"
           },
           {
             "name": "metadata-in-extension-overwritten-by-dfx-json",
             "content": "the content to be overwritten (from extension definition)"
           },
           {
             "name": "extension-limits-to-local-network",
             "networks": [ "local" ],
             "content": "this only applies to the local network"
           },
           {
             "name": "extension-limits-to-ic-network",
             "networks": [ "ic" ],
             "content": "this only applies to the ic network"
           }
         ],
         "tech_stack": {
           "cdk": {
             "embera": {
               "version": "1.5.2"
             },
             "ic-cdk": {
               "version": "2.14.4"
             }
           },
           "language": {
             "kotlin": {}
           }
         }
       }
      }
  }
EOF
  cat > dfx.json <<EOF
  {
    "canisters": {
      "c1": {
        "type": "embera",
        "candid": "main.did",
        "main": "main-file.embera",
        "metadata": [
          {
            "name": "metadata-in-extension-overwritten-by-dfx-json",
            "content": "the overwritten content (from dfx.json)"
          }
        ]
      },
      "c2": {
        "type": "embera",
        "candid": "main.did",
        "gzip": false,
        "main": "main-file.embera"
      },
      "c3": {
        "type": "embera",
        "candid": "main.did",
        "main": "main-file.embera",
        "metadata": [
          {
            "name": "extension-limits-to-local-network",
            "networks": [ "ic" ],
            "content": "this only applies to the ic network"
          },
          {
            "name": "extension-limits-to-ic-network",
            "networks": [ "local" ],
            "content": "dfx.json configuration applies only to local network"
          }
        ]
      },
      "c4": {
        "type": "embera",
        "candid": "main.did",
        "main": "main-file.embera",
        "tech_stack": {
          "cdk": {
            "ic-cdk": {
              "version": "1.16.6"
            }
          },
          "language": {
            "java": {
              "version": "25.0.6"
            }
          }
        }
      }
    }
  }
EOF

  assert_command dfx extension list
  assert_contains embera
  assert_command dfx deploy
  assert_contains "the embera build step for canister c1 with candid main.did and main file main-file.embera and gzip is true"
  assert_contains "the embera post-install step for canister c1 with candid main.did and main file main-file.embera and gzip is true"
  assert_contains "the embera build step for canister c2 with candid main.did and main file main-file.embera and gzip is false"
  assert_contains "the embera post-install step for canister c2 with candid main.did and main file main-file.embera and gzip is false"

  assert_command dfx canister metadata c1 metadata-in-extension-overwritten-by-dfx-json
  assert_eq "the overwritten content (from dfx.json)"
  assert_command dfx canister metadata c1 metadata-from-extension
  assert_eq "the content (from extension definition), gzip is true"

  assert_command dfx canister metadata c2 metadata-in-extension-overwritten-by-dfx-json
  assert_eq "the content to be overwritten (from extension definition)"
  assert_command dfx canister metadata c2 metadata-from-extension
  assert_eq "the content (from extension definition), gzip is false"

  assert_command dfx canister metadata c3 extension-limits-to-local-network
  assert_eq "this only applies to the local network"

  assert_command dfx canister metadata c3 extension-limits-to-ic-network
  assert_eq "dfx.json configuration applies only to local network"

  assert_command dfx canister metadata c4 dfx
  # shellcheck disable=SC2154
  echo "$stdout" > f.json
  assert_command jq -r '.tech_stack.cdk | keys | sort | join(",")' f.json
  assert_eq "embera,ic-cdk"
  assert_command jq -r '.tech_stack.cdk."ic-cdk".version' f.json
  assert_eq "1.16.6"
  assert_command jq -r '.tech_stack.language | keys | sort | join(",")' f.json
  assert_eq "java,kotlin"
}

@test "extension install with an empty cache does not create a corrupt cache" {
  dfx cache delete
  dfx extension install nns --version 0.2.1
  dfx_start
}

install_extension_from_official_registry() {
  EXTENSION=$1

  assert_command_fail dfx snsx

  assert_command dfx extension list
  assert_match 'No extensions installed'

  assert_command dfx extension install "$EXTENSION" --install-as snsx --version 0.2.1
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

@test "install extension by name from official registry" {
  install_extension_from_official_registry sns
}

@test "install extension by url from official registry" {
  install_extension_from_official_registry https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/sns/extension.json
}

get_extension_architecture() {
  _cputype="$(uname -m)"
  case "$_cputype" in

    x86_64 | x86-64 | x64 | amd64)
      _cputype=x86_64
      ;;

    arm64 | aarch64)
      _cputype=aarch64
      ;;

    *)
      err "unknown CPU type: $_cputype"
      ;;

  esac
  echo "$_cputype"
}

@test "install extension by url from elsewhere" {
  start_webserver --directory www
  EXTENSION_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/extension.json"
  mkdir -p  www/arbitrary/downloads

  cat > www/arbitrary/extension.json <<EOF
{
  "name": "an-extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "download_url_template": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/downloads/{{tag}}/{{basename}}.{{archive-format}}"
}
EOF
  cat www/arbitrary/extension.json
  cat > www/arbitrary/an-extension <<EOF
#!/usr/bin/env bash

echo "an extension output"
EOF
  chmod +x www/arbitrary/an-extension

  cat > www/arbitrary/dependencies.json <<EOF
{
  "0.1.0": {
    "dfx": {
      "version": ">=0.8.0"
    }
  }
}
EOF

  arch=$(get_extension_architecture)

  if [ "$(uname)" == "Darwin" ]; then
    ARCHIVE_BASENAME="an-extension-$arch-apple-darwin"
  else
    ARCHIVE_BASENAME="an-extension-$arch-unknown-linux-gnu"
  fi

  mkdir "$ARCHIVE_BASENAME"
  cp www/arbitrary/extension.json "$ARCHIVE_BASENAME"
  cp www/arbitrary/an-extension "$ARCHIVE_BASENAME"
  tar -czf "$ARCHIVE_BASENAME".tar.gz "$ARCHIVE_BASENAME"
  rm -rf "$ARCHIVE_BASENAME"

  mkdir -p www/arbitrary/downloads/an-extension-v0.1.0
  mv "$ARCHIVE_BASENAME".tar.gz www/arbitrary/downloads/an-extension-v0.1.0/

  assert_command dfx extension install "$EXTENSION_URL"
}

@test "install extension with non-platform-specific archive" {
  start_webserver --directory www
  EXTENSION_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/extension.json"
  mkdir -p  www/arbitrary/downloads

  cat > www/arbitrary/extension.json <<EOF
{
  "name": "an-extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "download_url_template": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/downloads/{{tag}}.{{archive-format}}"
}
EOF
  cat www/arbitrary/extension.json
  cat > www/arbitrary/an-extension <<EOF
#!/usr/bin/env bash

echo "an extension output"
EOF
  chmod +x www/arbitrary/an-extension

  cat > www/arbitrary/dependencies.json <<EOF
{
  "0.1.0": {
    "dfx": {
      "version": ">=0.8.0"
    }
  }
}
EOF


  ARCHIVE_BASENAME="an-extension-v0.1.0"

  mkdir "$ARCHIVE_BASENAME"
  cp www/arbitrary/extension.json "$ARCHIVE_BASENAME"
  cp www/arbitrary/an-extension "$ARCHIVE_BASENAME"
  tar -czf "$ARCHIVE_BASENAME".tar.gz "$ARCHIVE_BASENAME"
  rm -rf "$ARCHIVE_BASENAME"

  mv "$ARCHIVE_BASENAME".tar.gz www/arbitrary/downloads/

  assert_command dfx extension install "$EXTENSION_URL"
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
  assert_match "Error.*Failed to load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx extension run test_extension
  assert_match "Error.*Failed to load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx test_extension
  assert_match "Error.*Failed to load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx --help
  assert_match "Error.*Failed to load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx test_extension --help
  assert_match "Error.*Failed to load extension manifest.*Failed to read JSON file.*Failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  echo "{}" > "$CACHE_DIR"/extensions/test_extension/extension.json

  assert_command_fail dfx extension list
  assert_contains "Failed to load extension manifest"
  assert_match "Failed to parse contents of .*extensions/test_extension/extension.json as json"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx extension run test_extension
  assert_contains "Failed to load extension manifest"
  assert_match "Failed to parse contents of .*extensions/test_extension/extension.json as json.*"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx test_extension
  assert_contains "Failed to load extension manifest"
  assert_match "Failed to parse contents of .*extensions/test_extension/extension.json as json.*"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx --help
  assert_contains "Failed to load extension manifest"
  assert_match "Failed to parse contents of .*extensions/test_extension/extension.json as json.*"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx test_extension --help
  assert_contains "Failed to load extension manifest"
  assert_match "Failed to parse contents of .*extensions/test_extension/extension.json as json.*"
  assert_match "missing field .* at line .* column .*"

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

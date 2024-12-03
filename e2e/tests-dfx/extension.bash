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

@test "extension-defined project template" {
  start_webserver --directory www
  EXTENSION_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/extension.json"
  mkdir -p  www/arbitrary/downloads www/arbitrary/project_templates/a-template

  cat > www/arbitrary/extension.json <<EOF
{
  "name": "an-extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "project_templates": {
    "rust-by-extension": {
      "category": "backend",
      "display": "rust by extension",
      "requirements": [],
      "post_create": "cargo update",
      "port_create_failure_warning": "You will need to run it yourself (or a similar command like 'cargo vendor'), because 'dfx build' will use the --locked flag with Cargo."
    }
  },
  "download_url_template": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/downloads/{{tag}}.{{archive-format}}"
}
EOF

  cat > www/arbitrary/dependencies.json <<EOF
{
  "0.1.0": {
    "dfx": {
      "version": ">=0.8.0"
    }
  }
}
EOF

  cp -R "${BATS_TEST_DIRNAME}/../../src/dfx/assets/project_templates/rust" www/arbitrary/project_templates/rust-by-extension

  ARCHIVE_BASENAME="an-extension-v0.1.0"

  mkdir "$ARCHIVE_BASENAME"
  cp www/arbitrary/extension.json "$ARCHIVE_BASENAME"
  cp -R www/arbitrary/project_templates "$ARCHIVE_BASENAME"
  tar -czf "$ARCHIVE_BASENAME".tar.gz "$ARCHIVE_BASENAME"
  rm -rf "$ARCHIVE_BASENAME"

  mv "$ARCHIVE_BASENAME".tar.gz www/arbitrary/downloads/

  assert_command dfx extension install "$EXTENSION_URL"

  setup_rust

  dfx new rbe --type rust-by-extension --no-frontend
  cd rbe || exit

  dfx_start
  assert_command dfx deploy
  assert_command dfx canister call rbe_backend greet '("Rust By Extension")'
  assert_contains "Hello, Rust By Extension!"
}

@test "extension-defined project template replaces built-in type" {
  start_webserver --directory www
  EXTENSION_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/extension.json"
  mkdir -p  www/arbitrary/downloads www/arbitrary/project_templates/a-template

  cat > www/arbitrary/extension.json <<EOF
{
  "name": "an-extension",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "project_templates": {
    "rust": {
      "category": "backend",
      "display": "rust by extension",
      "requirements": [],
      "post_create": "cargo update"
    }
  },
  "download_url_template": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/downloads/{{tag}}.{{archive-format}}"
}
EOF

  cat > www/arbitrary/dependencies.json <<EOF
{
  "0.1.0": {
    "dfx": {
      "version": ">=0.8.0"
    }
  }
}
EOF

  cp -R "${BATS_TEST_DIRNAME}/../../src/dfx/assets/project_templates/rust" www/arbitrary/project_templates/rust
  echo "just-proves-it-used-the-project-template" > www/arbitrary/project_templates/rust/proof.txt

  ARCHIVE_BASENAME="an-extension-v0.1.0"

  mkdir "$ARCHIVE_BASENAME"
  cp www/arbitrary/extension.json "$ARCHIVE_BASENAME"
  cp -R www/arbitrary/project_templates "$ARCHIVE_BASENAME"
  tar -czf "$ARCHIVE_BASENAME".tar.gz "$ARCHIVE_BASENAME"
  rm -rf "$ARCHIVE_BASENAME"

  mv "$ARCHIVE_BASENAME".tar.gz www/arbitrary/downloads/

  assert_command dfx extension install "$EXTENSION_URL"

  setup_rust

  dfx new rbe --type rust --no-frontend
  assert_command cat rbe/proof.txt
  assert_eq "just-proves-it-used-the-project-template"

  cd rbe || exit

  dfx_start
  assert_command dfx deploy
  assert_command dfx canister call rbe_backend greet '("Rust By Extension")'
  assert_contains "Hello, Rust By Extension!"
}

@test "run an extension command with a canister type defined by another extension" {
  install_shared_asset subnet_type/shared_network_settings/system
  dfx_start_for_nns_install

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
         "wasm": ".embera/{{canister_name}}/{{canister_name}}.wasm"
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
        "main": "main-file.embera"
      }
    }
  }
EOF

  dfx extension install nns --version 0.4.7
  dfx nns install
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

install_extension_from_dfx_extensions_repo() {
  EXTENSION=$1

  assert_command_fail dfx snsx

  assert_command dfx extension list
  assert_match 'No extensions installed'

  assert_command dfx extension install "$EXTENSION" --install-as snsx --version 0.4.7
  assert_contains "Extension 'sns' version 0.4.7 installed successfully, and is available as 'snsx'"

  assert_command dfx extension list
  assert_match 'snsx'

  assert_command dfx --help
  assert_match 'snsx.*Initialize, deploy and interact with an SNS'

  assert_command dfx snsx --help

  assert_command dfx extension uninstall snsx
  # TODO: how to capture spinner message?
  # assert_match 'Successfully uninstalled extension'

  assert_command dfx extension list
  assert_match 'No extensions installed'
}

@test "install extension by name from official catalog" {
  install_extension_from_dfx_extensions_repo sns
}

@test "install hosted extension by url" {
  install_extension_from_dfx_extensions_repo https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/sns/extension.json
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

@test "install extension from catalog" {
  start_webserver --directory www

  CATALOG_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-1/catalog.json"
  mkdir -p www/arbitrary-1
  cat > www/arbitrary-1/catalog.json <<EOF
{
  "foo": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/foo/extension.json",
  "bar": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/bar/extension.json"
}
EOF


  EXTENSION_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/foo/extension.json"
  mkdir -p  www/arbitrary-2/foo/downloads

  cat > www/arbitrary-2/foo/extension.json <<EOF
{
  "name": "foo",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "download_url_template": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/downloads/{{tag}}/{{basename}}.{{archive-format}}"
}
EOF
  cat www/arbitrary-2/foo/extension.json

  cat > www/arbitrary-2/foo/dependencies.json <<EOF
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
    ARCHIVE_BASENAME="foo-$arch-apple-darwin"
  else
    ARCHIVE_BASENAME="foo-$arch-unknown-linux-gnu"
  fi

  mkdir "$ARCHIVE_BASENAME"
  cp www/arbitrary-2/foo/extension.json "$ARCHIVE_BASENAME"
  tar -czf "$ARCHIVE_BASENAME".tar.gz "$ARCHIVE_BASENAME"
  rm -rf "$ARCHIVE_BASENAME"

  mkdir -p www/arbitrary/downloads/foo-v0.1.0
  mv "$ARCHIVE_BASENAME".tar.gz www/arbitrary/downloads/foo-v0.1.0/

  assert_command dfx extension install foo --catalog-url "$CATALOG_URL"
}

@test "install extension with no subcommands" {
  start_webserver --directory www

  CATALOG_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-1/catalog.json"
  mkdir -p www/arbitrary-1
  cat > www/arbitrary-1/catalog.json <<EOF
{
  "foo": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/foo/extension.json",
  "bar": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/bar/extension.json"
}
EOF


  EXTENSION_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/foo/extension.json"
  mkdir -p  www/arbitrary-2/foo/downloads

  cat > www/arbitrary-2/foo/extension.json <<EOF
{
  "name": "foo",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "download_url_template": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/downloads/{{tag}}/{{basename}}.{{archive-format}}"
}
EOF
  cat www/arbitrary-2/foo/extension.json

  cat > www/arbitrary-2/foo/dependencies.json <<EOF
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
    ARCHIVE_BASENAME="foo-$arch-apple-darwin"
  else
    ARCHIVE_BASENAME="foo-$arch-unknown-linux-gnu"
  fi

  mkdir "$ARCHIVE_BASENAME"
  cp www/arbitrary-2/foo/extension.json "$ARCHIVE_BASENAME"
  tar -czf "$ARCHIVE_BASENAME".tar.gz "$ARCHIVE_BASENAME"
  rm -rf "$ARCHIVE_BASENAME"

  mkdir -p www/arbitrary/downloads/foo-v0.1.0
  mv "$ARCHIVE_BASENAME".tar.gz www/arbitrary/downloads/foo-v0.1.0/

  assert_command dfx extension install foo --catalog-url "$CATALOG_URL"
}

@test "install extension with empty subcommands" {
  start_webserver --directory www

  CATALOG_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-1/catalog.json"
  mkdir -p www/arbitrary-1
  cat > www/arbitrary-1/catalog.json <<EOF
{
  "foo": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/foo/extension.json",
  "bar": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/bar/extension.json"
}
EOF


  EXTENSION_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary-2/foo/extension.json"
  mkdir -p  www/arbitrary-2/foo/downloads

  cat > www/arbitrary-2/foo/extension.json <<EOF
{
  "name": "foo",
  "version": "0.1.0",
  "homepage": "https://github.com/dfinity/dfx-extensions",
  "authors": "DFINITY",
  "summary": "Test extension for e2e purposes.",
  "categories": [],
  "keywords": [],
  "subcommands": {},
  "download_url_template": "http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/downloads/{{tag}}/{{basename}}.{{archive-format}}"
}
EOF
  cat www/arbitrary-2/foo/extension.json

  cat > www/arbitrary-2/foo/dependencies.json <<EOF
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
    ARCHIVE_BASENAME="foo-$arch-apple-darwin"
  else
    ARCHIVE_BASENAME="foo-$arch-unknown-linux-gnu"
  fi

  mkdir "$ARCHIVE_BASENAME"
  cp www/arbitrary-2/foo/extension.json "$ARCHIVE_BASENAME"
  tar -czf "$ARCHIVE_BASENAME".tar.gz "$ARCHIVE_BASENAME"
  rm -rf "$ARCHIVE_BASENAME"

  mkdir -p www/arbitrary/downloads/foo-v0.1.0
  mv "$ARCHIVE_BASENAME".tar.gz www/arbitrary/downloads/foo-v0.1.0/

  assert_command dfx extension install foo --catalog-url "$CATALOG_URL"
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

@test "install is not an error if already installed" {
  assert_command_fail dfx nns --help
  assert_command dfx extension install nns --version 0.4.1
  assert_command dfx extension install nns --version 0.4.1
  # shellcheck disable=SC2154
  assert_eq "WARN: Extension 'nns' version 0.4.1 is already installed" "$stderr"
  assert_command dfx nns --help
}

@test "install is not an error if an older version is already installed and no version was specified" {
  assert_command_fail dfx nns --help
  assert_command dfx extension install nns --version 0.3.1
  assert_command dfx extension install nns
  # shellcheck disable=SC2154
  assert_eq "WARN: Extension 'nns' version 0.3.1 is already installed" "$stderr"
  assert_command dfx nns --help
}

@test "reports error if older version already installed and specific version requested" {
  assert_command_fail dfx nns --help
  assert_command dfx extension install nns --version 0.3.1
  assert_command_fail dfx extension install nns --version 0.4.1
  # shellcheck disable=SC2154
  assert_contains "ERROR: Extension 'nns' is already installed at version 0.3.1" "$stderr"
  # shellcheck disable=SC2154
  assert_contains 'ERROR: To upgrade, run "dfx extension uninstall nns" and then re-run the dfx extension install command' "$stderr"
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
  assert_match "Error.*Failed to load extension manifest.*failed to read JSON file.*failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx extension run test_extension
  assert_match "Error.*Failed to load extension manifest.*failed to read JSON file.*failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx test_extension
  assert_match "Error.*Failed to load extension manifest.*failed to read JSON file.*failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx --help
  assert_match "Error.*Failed to load extension manifest.*failed to read JSON file.*failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  assert_command_fail dfx test_extension --help
  assert_match "Error.*Failed to load extension manifest.*failed to read JSON file.*failed to read .*extensions/test_extension/extension.json.*No such file or directory"

  echo "{}" > "$CACHE_DIR"/extensions/test_extension/extension.json

  assert_command_fail dfx extension list
  assert_contains "Failed to load extension manifest"
  assert_match "failed to parse contents of .*extensions/test_extension/extension.json as json"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx extension run test_extension
  assert_contains "Failed to load extension manifest"
  assert_match "failed to parse contents of .*extensions/test_extension/extension.json as json.*"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx test_extension
  assert_contains "Failed to load extension manifest"
  assert_match "failed to parse contents of .*extensions/test_extension/extension.json as json.*"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx --help
  assert_contains "Failed to load extension manifest"
  assert_match "failed to parse contents of .*extensions/test_extension/extension.json as json.*"
  assert_match "missing field .* at line .* column .*"

  assert_command_fail dfx test_extension --help
  assert_contains "Failed to load extension manifest"
  assert_match "failed to parse contents of .*extensions/test_extension/extension.json as json.*"
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

@test "extension run uses project root" {
  CACHE_DIR=$(dfx cache show)
  mkdir -p "$CACHE_DIR"/extensions/test_extension

  cat > "$CACHE_DIR"/extensions/test_extension/test_extension << "EOF"
#!/usr/bin/env bash

echo "the current directory is '$(pwd)'"

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
  }
  }
  }
}
EOF

  mkdir -p project || exit
  cd project || exit
  echo "{}" >dfx.json
  mkdir -p subdir || exit
  cd subdir || exit

  assert_command dfx test_extension abc
  assert_match "the current directory is '.*/working-dir/project'"
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

@test "list available extensions from official catalog" {
  assert_command dfx extension list --available
  assert_contains "sns"
  assert_contains "nns"
}

@test "list available extensions from customized catalog" {
  start_webserver --directory www
  CATALOG_URL_URL="http://localhost:$E2E_WEB_SERVER_PORT/arbitrary/catalog.json"
  mkdir -p  www/arbitrary

    cat > www/arbitrary/catalog.json <<EOF
{
  "nns": "https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/nns/extension.json",
  "sns": "https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/sns/extension.json",
  "test": "https://raw.githubusercontent.com/dfinity/dfx-extensions/main/extensions/sns/extension.json"
}
EOF

  assert_command dfx extension list --catalog-url="$CATALOG_URL_URL"
  assert_contains "sns"
  assert_contains "nns"
  assert_contains "test"
}

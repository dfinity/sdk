#!/bin/bash

set -ex

export

# Enter temporary directory.
pushd /tmp

# Install Bats.
sudo apt-get install --yes bats

# Install Bats support.
version=0.3.0
wget https://github.com/ztombol/bats-support/archive/v$version.tar.gz
sudo mkdir /usr/local/lib/bats-support
sudo tar --directory /usr/local/lib/bats-support --extract --file v$version.tar.gz --strip-components 1
rm v$version.tar.gz

# Packages needed for some tests
if [ "$E2E_TEST" = "tests-dfx/certificate.bash" ]; then
    sudo apt-get install --yes mitmproxy python3-pip
    # pyparsing 3.x renamed something, resulting in this error:
    # # AttributeError: module 'pyparsing' has no attribute 'operatorPrecedence'
    pip3 uninstall --yes pyparsing
    pip3 install pyparsing==2.4.6
fi

echo "What is happening with pyparsing"
python3 --version
python3 -c "
import pyparsing as pp
print(pp.__version__)
"

# Set environment variables.
BATS_SUPPORT="/usr/local/lib/bats-support"
echo "BATSLIB=${BATS_SUPPORT}" >> "$GITHUB_ENV"
echo "$HOME/bin" >> "$GITHUB_PATH"

# Exit temporary directory.
popd

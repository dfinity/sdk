# Print the latest dfx version available on GitHub releases
get_latest_dfx_version() {
    latest_version=$(curl -sL "https://api.github.com/repos/dfinity/sdk/releases/latest" | jq -r ".tag_name")
    echo "$latest_version"
}

# Extract a particular file from the latest release tarball, and save it to the specified destination
# Usage: get_from_latest_release_tarball <file_path> <destination>
get_from_latest_release_tarball() {
    local -r file_path=$1
    local -r destination=$2
    local -r tarball_url=$(curl -sL "https://api.github.com/repos/dfinity/sdk/releases/latest" | jq -r ".tarball_url")

    local -r temp_dir=$(mktemp -d)
    curl -sL "$tarball_url" -o "${temp_dir}/release.tar.gz"

    tar -xzf "$temp_dir/release.tar.gz" -C "$temp_dir" "*/$file_path"
    local -r file_name=$(basename $file_path)
    local -r extracted_file=$(find "$temp_dir" -type f -name "$file_name")
    mv "$extracted_file" "$destination"
}

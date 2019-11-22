## install/manifest.sh
#   Functions useful for dealing with the manifest (which is JSON).

# Get the version of a tag from the manifest JSON file.
# Arguments:
#   $1 - The tag to get.
#   STDIN - The manifest file.
# Returns:
#   0 if the tag was found, 1 if it wasn't.
#   Prints out the version number.
get_tag_from_manifest_json() {
    # Find the tag in the file. Then get the last digits.
    # The first grep returns `"tag_name": "1.2.3` (without the last quote).
    cat \
        | tr -d '\n' \
        | grep -o "\"$1\":[[:space:]]*\"[a-zA-Z0-9.]*" \
        | grep -o "[0-9.]*$"
}

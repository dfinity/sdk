main() {
  log "Installing dfx cache"
  dfx cache install
}

main "$@" || exit $?
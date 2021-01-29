# The cargo audit job for known vulnerabilities. This generally runs
# against the advisory database pinned in sources.json; on Hydra
# (master) however the latest advisory database is fetched from
# RustSec/advisory-db. This means that whenever a new security
# vulnerability is published or when Cargo.lock has been changed `cargo
# audit` will run.
{ pkgs ? import ./nix { inherit system; }
, system ? builtins.currentSystem
, RustSec-advisory-db ? pkgs.sources.advisory-db
}:
pkgs.lib.cargo-security-audit {
  name = "dfinity-sdk";
  cargoLock = ./Cargo.lock;
  db = RustSec-advisory-db;
	# Ignore this vulnerability until candid updates to a logos with the beef patch
	# ID:       RUSTSEC-2020-0122
	# Crate:    beef
	# Version:  0.4.4
	# Date:     2020-10-28
	# URL:      https://rustsec.org/advisories/RUSTSEC-2020-0122
	# Title:    beef::Cow lacks a Sync bound on its Send trait allowing for data races
	# Solution:  upgrade to >= 0.5.0
	# Dependency tree:
	# beef 0.4.4
	# └── logos-derive 0.11.5
	#     └── logos 0.11.4
	#         └── candid 0.6.13
	#             ├── ic-utils 0.1.0
	#             │   └── dfx 0.6.21
	#             └── dfx 0.6.21
  ignores = [ "RUSTSEC-2020-0122" ];
}

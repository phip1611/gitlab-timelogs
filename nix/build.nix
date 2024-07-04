{ crane
, darwin
, lib
, nix-gitignore
, openssl
, pkg-config
, rust-bin
, stdenv
}:

let
  # Toolchain from Rust overlay.
  rustToolchain = rust-bin.stable.latest.default;
  craneLib = crane.overrideToolchain rustToolchain;

  commonArgs = {
    src = nix-gitignore.gitignoreSource [ ] ../.;
    # Not using this, as this removes the ".graphql" file.
    # src = craneLib.cleanCargoSource ./..;
    nativeBuildInputs = [
      pkg-config
    ];
    buildInputs = [
      openssl
    ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
      SystemConfiguration
    ]);
    # Fix build. Reference:
    # - https://github.com/sfackler/rust-openssl/issues/1430
    # - https://docs.rs/openssl/latest/openssl/
    OPENSSL_NO_VENDOR = true;
  };

  # Downloaded and compiled dependencies.
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  cargoPackage = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
  });
in
cargoPackage

{ craneLib
, darwin
, lib
, nix-gitignore
, openssl
, pkg-config
, stdenv
}:

let
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

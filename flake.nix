{
  description = "gitlab-timelogs";

  inputs = {
    crane.url = "github:ipetkov/crane/master";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, flake-parts, ... }@inputs:
    let
      pkgs = import inputs.nixpkgs {
        system = "x86_64-linux";
        overlays = [
          (inputs.rust-overlay.overlays.default)
        ];
      };
    in
    flake-parts.lib.mkFlake { inherit inputs; }
      {
        flake = {
          nixosModules.default = (
            { pkgs, ... }:
            {
              environment.systemPackages = [
                self.packages.${pkgs.system}.gitlab-timelogs
              ];
            }
          );
        };
        # Don't artificially limit users at this point. If the build fails,
        # they will see soon enough.
        systems = inputs.nixpkgs.lib.systems.flakeExposed;
        perSystem = { system, self', ... }:
          let
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                (inputs.rust-overlay.overlays.default)
              ];
            };
          in
          {
            devShells = {
              default = pkgs.mkShell {
                inputsFrom = [ self'.packages.default ];
              };
            };
            formatter = pkgs.nixpkgs-fmt;
            packages = rec {
              default = gitlab-timelogs;
              gitlab-timelogs = pkgs.callPackage ./nix/build.nix {
                crane = inputs.crane.mkLib pkgs;
              };
            };
          };
      };
}

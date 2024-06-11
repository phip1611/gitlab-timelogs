{
  description = "gitlab-timelogs";

  inputs = {
    crane.url = "github:ipetkov/crane/master";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, ... }@inputs:
    let
      pkgs = import inputs.nixpkgs {
        system = "x86_64-linux";
        overlays = [
          (inputs.rust-overlay.overlays.default)
        ];
      };
    in
    {
      devShells.x86_64-linux = {
        default = import ./shell.nix { inherit pkgs; };
      };
      formatter.x86_64-linux = pkgs.nixpkgs-fmt;
      nixosModules.default = (
        { pkgs, ... }:
        {
          environment.systemPackages = [
            self.packages.${pkgs.system}.gitlab-timelogs
          ];
        }
      );
      packages.x86_64-linux = rec {
        default = gitlab-timelogs;
        gitlab-timelogs = import ./nix/build.nix {
          inherit pkgs;
          crane = inputs.crane.mkLib pkgs;
        };
      };
    };
}


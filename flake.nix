{
  description = "gitlab-timelogs";

  inputs = {
    crane.url = "github:ipetkov/crane/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs, ... }@inputs:
    let
      systems = nixpkgs.lib.systems.flakeExposed;
      forAllSystems =
        function: nixpkgs.lib.genAttrs systems (system: function nixpkgs.legacyPackages.${system});
    in
    {
      nixosModules = {
        default = (
          { pkgs, ... }:
          {
            environment.systemPackages = [
              self.packages.${pkgs.system}.gitlab-timelogs
            ];
          }
        );
        home-manager = (
          {
            config,
            pkgs,
            lib,
            ...
          }:
          let
            cfg = config.gitlab-timelogs;
          in
          {
            options.gitlab-timelogs = {
              enable = lib.mkEnableOption "gitlab-timelogs";
              package = lib.mkOption {
                type = lib.types.package;
                default = self.packages.${pkgs.system}.gitlab-timelogs;
              };
              config = lib.mkOption {
                description = "The values in your config file.";
                type = lib.types.submodule {
                  options = {
                    gitlabHost = lib.mkOption {
                      type = lib.types.str;
                      description = "Gitlab host you want to query.";
                      example = "gitlab.example.com";
                    };
                    gitlabUsername = lib.mkOption {
                      type = lib.types.str;
                      description = "Your gitlab username";
                      example = "exampleuser";
                    };
                    gitlabToken = lib.mkOption {
                      type = lib.types.str;
                      description = "A gitlab token with read access to the given host.";
                      example = "glpat-XXXXXXXXXXXXXXXXXXXX";
                    };
                  };
                };
              };
            };
            config = lib.mkIf cfg.enable {
              home.packages = [
                cfg.package
              ];

              home.file.".config/gitlab-timelogs/config.toml".text = ''
                gitlab_host = "${cfg.config.gitlabHost}"
                gitlab_username = "${cfg.config.gitlabUsername}"
                gitlab_token = "${cfg.config.gitlabToken}"
              '';
            };
          }
        );
      };
      devShells = forAllSystems (
        pkgs:
        let
          system = pkgs.stdenv.hostPlatform.system;
        in
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ];
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [
              pkgs.openssl
            ];
          };
        }
      );
      formatter = forAllSystems (pkgs: pkgs.nixfmt-tree);
      packages = forAllSystems (
        pkgs:
        let
          gitlab-timelogs = pkgs.callPackage ./nix/build.nix {
            craneLib = inputs.crane.mkLib pkgs;
          };
        in
        {
          inherit gitlab-timelogs;
          default = gitlab-timelogs;
        }
      );
    };
}

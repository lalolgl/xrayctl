{
  description = "A command-line manager for Xray subscriptions and connections";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;

    in {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "xrayctl";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [
              pkgs.makeWrapper
            ];

            postFixup = ''
              wrapProgram $out/bin/xrayctl \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.xray ]}
            '';

            meta = {
              description = "A command-line manager for Xray subscriptions and connections";
              homepage = "https://github.com/lalolgl/xrayctl";
              license = pkgs.lib.licenses.mit;
              mainProgram = "xrayctl";
            };
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              rustfmt
              clippy
            ];
          };
        });
    };
}

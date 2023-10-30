{
  description = "panorama - a system status notification daemon for Linux desktop systems";

  inputs = {
    flakeutils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flakeutils }:
    flakeutils.lib.eachDefaultSystem (system:
      let
        NAME = "panorama";
        VERSION = "0.1.0";

        pkgs = import nixpkgs {
          inherit system;
        };

      in
      rec {
        packages.${NAME} = pkgs.rustPlatform.buildRustPackage rec {
          pname = NAME;
          version = VERSION;

          src = pkgs.lib.cleanSource ./.;

          cargoLock.lockFile = ./Cargo.lock;

          runtimeDependencies = with pkgs; [
            # Needed for the notify-send command.
            libnotify
          ];

          # meta = with pkgs.stdenv.lib; {
          #   description = "Notification daemon for Linux desktop systems.";
          #   homepage = "https://github.com/theduke/panorama";
          #   license = licenses.mit;
          #   maintainers = [ maintainers.theduke ];
          # };
        };

        defaultPackage = packages.${NAME};

        # For `nix run`.
        apps.${NAME} = flakeutils.lib.mkApp {
          drv = packages.${NAME};
        };
        defaultApp = apps.${NAME};

        devShell = pkgs.stdenv.mkDerivation {
          name = NAME;
          src = self;
          nativeBuildInputs = with pkgs; [
            pkg-config
            
          ];
          buildInputs = with pkgs; [
            pkg-config
            # Required for libudev, which is included in the systemd package
            systemd
            git-cliff
            libnotify

            cargo-udeps
            cargo-deny
          ];
        };
      }
    );
}

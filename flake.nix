{
  description = "bundle environment for hook101";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    # nixpkgs.url = "https://mirrors.tencent.com/github.com/nixos/nixpkgs/archive/nixos-unstable.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    let
      javaVersion = 21;

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [ 
                inputs.self.overlays.default
                inputs.rust-overlay.overlays.default
                ];
              config.allowUnfree = true;
              config.android_sdk.accept_license = true;
            };
          }
        );
    in
    {
      overlays.default =
        final: prev:
        let
          jdk = prev."jdk${toString javaVersion}";
          androidComposition = prev.androidenv.composeAndroidPackages {
            includeNDK = true;
            platformVersions = [ "36" ];
            buildToolsVersions = [ "35.0.0" ];
            abiVersions = [ "armeabi-v7a" "arm64-v8a" ];

            # Uncommon for compiling
            useGoogleAPIs = false;
            useGoogleTVAddOns = false;
            includeEmulator = false;
            includeSystemImages = false;
            includeSources = false;
          };
          androidSdk = androidComposition.androidsdk;
        in
        {
          inherit jdk;
          inherit androidSdk;
          gradle = prev.gradle.override { java = jdk; };
        };
      devShells = forEachSupportedSystem (
        { pkgs }:
        let
          rustPkgs = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in 
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              jdk
              androidSdk
              gradle
              rustPkgs
              cargo
              rustfmt
              clippy
              rust-analyzer
              cargo-ndk
            ];

            JAVA_HOME = pkgs.jdk;
            ANDROID_HOME= "${pkgs.androidSdk}/libexec/android-sdk";
            ANDROID_NDK_HOME= "${pkgs.androidSdk}/libexec/android-sdk/ndk-bundle";
          };
        }
      );
    };
}

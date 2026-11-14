{
  description = "Oxevu server, web and Android development";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
    in {
      devShells = nixpkgs.lib.genAttrs systems (system:
        let
          pkgs = import nixpkgs { inherit system; config = { allowUnfree = true; android_sdk.accept_license = true; }; };
          android = pkgs.androidenv.composeAndroidPackages {
            platformVersions = [ "35" ];
            buildToolsVersions = [ "35.0.0" ];
            includeEmulator = true;
            includeSystemImages = true;
            systemImageTypes = [ "google_apis" ];
            abiVersions = [ (if pkgs.stdenv.hostPlatform.isAarch64 then "arm64-v8a" else "x86_64") ];
          };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc cargo rustfmt clippy rust-analyzer pkg-config openssl
              nodejs_22 pnpm ffmpeg sqlite just git curl jq nixfmt direnv python3
              jdk17 android.androidsdk
            ];
            ANDROID_HOME = "${android.androidsdk}/libexec/android-sdk";
            ANDROID_SDK_ROOT = "${android.androidsdk}/libexec/android-sdk";
            JAVA_HOME = "${pkgs.jdk17}";
            shellHook = ''
              export PATH="$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
              echo "Oxevu: Rust, web, FFmpeg and Android SDK ready. See docs/development.md."
            '';
          };
        });
    };
}

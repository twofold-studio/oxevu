{
  lib,
  stdenv,
  rustPlatform,
  nodejs,
  pnpm,
  pnpmConfigHook,
  fetchPnpmDeps,
  pkg-config,
  openssl,
  aws-lc,
  sqlite,
  cmake,
  makeWrapper,
  ffmpeg,
}:
let
  source = lib.cleanSource ../.;
  web = stdenv.mkDerivation (final: {
    pname = "oxevu-web";
    version = lib.strings.trim (builtins.readFile ../VERSION);
    src = source;
    sourceRoot = "source/apps/web";
    nativeBuildInputs = [
      nodejs
      pnpm
      pnpmConfigHook
    ];
    pnpmDeps = fetchPnpmDeps {
      pname = "oxevu-web";
      inherit (final) version src sourceRoot;
      fetcherVersion = 5;
      hash = "sha256-fmb4eF0VpCEhRQKO+AEU3ybNWcTehbiMUTvN15jy79k=";
    };
    buildPhase = "pnpm build";
    installPhase = "cp -r dist $out";
  });
in
rustPlatform.buildRustPackage {
  pname = "oxevu";
  version = lib.strings.trim (builtins.readFile ../VERSION);
  src = source;
  cargoLock.lockFile = ../Cargo.lock;
  nativeBuildInputs = [
    pkg-config
    cmake
    makeWrapper
  ];
  buildInputs = [
    openssl
    aws-lc
    sqlite
  ];
  preBuild = "cp -r ${web} apps/web/dist";
  doCheck = false;
  postInstall = ''
    wrapProgram $out/bin/oxevu-server --prefix PATH : ${lib.makeBinPath [ ffmpeg ]}
  '';
  meta.mainProgram = "oxevu-server";
}

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
      hash = "sha256-pE1ca+eKNuv57yr3evprJLgnc/1ATJSbybFlogWXQkY=";
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
  cargoBuildFlags = [
    "-p"
    "oxevu-server"
    "-p"
    "oxevu-api"
    "--bin"
    "oxevu-server"
    "--bin"
    "oxevu-export-api"
  ];
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
  postBuild = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    contract_exporter=$(find target -path '*/release/oxevu-export-api' -type f -executable -print -quit)
    test -n "$contract_exporter"
    "$contract_exporter" > "$TMPDIR/openapi.json"
    diff -u apps/web/src/lib/api/openapi.json "$TMPDIR/openapi.json"
  '';
  doCheck = false;
  postInstall = ''
    rm -f "$out/bin/oxevu-export-api"
    wrapProgram $out/bin/oxevu-server --prefix PATH : ${lib.makeBinPath [ ffmpeg ]}
  '';
  meta.mainProgram = "oxevu-server";
}

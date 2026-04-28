{
  cmake,
  fetchurl,
  llvmPackages,
  openssl,
  libcap ? null,
  rustPlatform,
  pkg-config,
  lib,
  stdenv,
  packageName ? "codex-cli",
  binaryName ? "codex",
  derivationName ? "codex-rs",
  description ? "OpenAI Codex command-line interface rust implementation",
  enableRustyV8 ? true,
  version ? "0.0.0",
  ...
}:
let
  rustyV8Archive =
    if enableRustyV8 then
      let
        rustyV8Artifacts =
          {
            x86_64-linux = {
              file = "librusty_v8_release_x86_64-unknown-linux-gnu.a.gz";
              sha256 = "e64b4d99e4ae293a2e846244a89b80178ba10382c13fb591c1fa6968f5291153";
            };
            aarch64-linux = {
              file = "librusty_v8_release_aarch64-unknown-linux-gnu.a.gz";
              sha256 = "dbf165b07c81bdb054bc046b43d23e69fcf7bcc1a4c1b5b4776983a71062ecd8";
            };
            x86_64-darwin = {
              file = "librusty_v8_release_x86_64-apple-darwin.a.gz";
              sha256 = "630cd240f1bbecdb071417dc18387ab81cf67c549c1c515a0b4fcf9eba647bb7";
            };
            aarch64-darwin = {
              file = "librusty_v8_release_aarch64-apple-darwin.a.gz";
              sha256 = "bfe2c9be32a56c28546f0f965825ee68fbf606405f310cc4e17b448a568cf98a";
            };
          }
          .${stdenv.hostPlatform.system}
            or (throw "Unsupported system for rusty_v8 prebuilt archive: ${stdenv.hostPlatform.system}");
      in
      fetchurl {
        url = "https://github.com/denoland/rusty_v8/releases/download/v146.4.0/${rustyV8Artifacts.file}";
        inherit (rustyV8Artifacts) sha256;
      }
    else
      null;
in
rustPlatform.buildRustPackage (_: {
  env =
    {
      PKG_CONFIG_PATH = lib.makeSearchPathOutput "dev" "lib/pkgconfig" (
        [ openssl ] ++ lib.optionals stdenv.isLinux [ libcap ]
      );
    }
    // lib.optionalAttrs enableRustyV8 {
      RUSTY_V8_ARCHIVE = rustyV8Archive;
    };
  pname = derivationName;
  inherit version;
  cargoLock.lockFile = ./Cargo.lock;
  cargoBuildFlags = [
    "-p"
    packageName
    "--bin"
    binaryName
  ];
  cargoInstallFlags = [
    "-p"
    packageName
    "--bin"
    binaryName
  ];
  doCheck = false;
  src = ./.;

  # Patch the workspace Cargo.toml so that cargo embeds the correct version in
  # CARGO_PKG_VERSION (which the binary reads via env!("CARGO_PKG_VERSION")).
  # On release commits the Cargo.toml already contains the real version and
  # this sed is a no-op.
  postPatch = ''
    sed -i 's/^version = "0\.0\.0"$/version = "${version}"/' Cargo.toml
  '';
  nativeBuildInputs = [
    cmake
    llvmPackages.clang
    llvmPackages.libclang.lib
    openssl
    pkg-config
  ] ++ lib.optionals stdenv.isLinux [
    libcap
  ];

  cargoLock.outputHashes = {
    "libwebrtc-0.3.26" = "sha256-0HPuwaGcqpuG+Pp6z79bCuDu/DyE858VZSYr3DKZD9o=";
    "ratatui-0.29.0" = "sha256-HBvT5c8GsiCxMffNjJGLmHnvG77A6cqEL+1ARurBXho=";
    "crossterm-0.28.1" = "sha256-6qCtfSMuXACKFb9ATID39XyFDIEMFDmbx6SSmNe+728=";
    "nucleo-0.5.0" = "sha256-Hm4SxtTSBrcWpXrtSqeO0TACbUxq3gizg1zD/6Yw/sI=";
    "nucleo-matcher-0.3.1" = "sha256-Hm4SxtTSBrcWpXrtSqeO0TACbUxq3gizg1zD/6Yw/sI=";
    "runfiles-0.1.0" = "sha256-uJpVLcQh8wWZA3GPv9D8Nt43EOirajfDJ7eq/FB+tek=";
    "tokio-tungstenite-0.28.0" = "sha256-hJAkvWxDjB9A9GqansahWhTmj/ekcelslLUTtwqI7lw=";
    "tungstenite-0.27.0" = "sha256-AN5wql2X2yJnQ7lnDxpljNw0Jua40GtmT+w3wjER010=";
  };

  meta = with lib; {
    inherit description;
    license = licenses.asl20;
    homepage = "https://github.com/openai/codex";
    mainProgram = binaryName;
  };
})

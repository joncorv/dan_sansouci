{
  lib,
  stdenv,
  rustPlatform,
  openssl,
  pkg-config,
}:

rustPlatform.buildRustPackage rec {
  pname = "bedrock";
  version = "0.1.10";

  src = ./.;

  # cargoHash = "sha256-Ak8oQ8YNwWIY5X7t5L9e2MiUYgUMNwvZXp0zm8E6PGM=";
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    openssl
  ];

  cargoRoot = "./.";
  buildAndTestSubdir = cargoRoot;

  meta = with lib; {
    description = "Bedrock: The backend for Big-Sky";
    homepage = "https://github.com/joncorv/dan_sansouci";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "bedrock";
    platforms = platforms.linux ++ platforms.darwin;
  };
}

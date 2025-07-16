{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  darwin,
  stdenv,
  typescript,
}:

let
  workspace = import ../workspace.nix;
in
rustPlatform.buildRustPackage {
  pname = "hakanai-cli";
  version = workspace.version;

  src = lib.cleanSource ../.;

  cargoBuildCommand = "cargo build --release --package cli";
  cargoTestCommand = "cargo test --release --package cli";
  cargoHash = workspace.cargoHash;

  nativeBuildInputs = [
    pkg-config
    typescript
  ];

  buildInputs =
    [
      openssl
    ]
    ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.Security
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  meta = with lib; {
    description = "A minimalist one-time secret sharing service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist.";
    homepage = "https://github.com/czerwonk/hakanai";
    license = licenses.mit;
  };
}

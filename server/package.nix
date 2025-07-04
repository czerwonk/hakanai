{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  darwin,
  stdenv,
}:

rustPlatform.buildRustPackage {
  pname = "hakanai-server";
  version = "1.0.0";

  src = lib.cleanSource ../.;

  cargoBuildCommand = "cargo build --release --package server";
  cargoTestCommand = "cargo test --release --package server";
  cargoHash = "sha256-Ka6J0QQWe9fhqClADH8An18FmoQsYiGI20LtFla5TYQ=";

  nativeBuildInputs = [
    pkg-config
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

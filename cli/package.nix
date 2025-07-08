{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  darwin,
  stdenv,
}:

rustPlatform.buildRustPackage {
  pname = "hakanai-cli";
  version = "1.2.2";

  src = lib.cleanSource ../.;

  cargoBuildCommand = "cargo build --release --package cli";
  cargoTestCommand = "cargo test --release --package cli";
  cargoHash = "sha256-+JU8vSMRiUIj3FuZ/xR7ydYXDyxbdbsr48WI8o4FuRw=";

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

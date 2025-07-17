{
  lib,
  rustPlatform,
  pkg-config,
  openssl,
  darwin,
  stdenv,
}:

let
  workspace = import ../workspace.nix;
in
rustPlatform.buildRustPackage {
  pname = "hakanai-cli";
  version = workspace.version;

  src = lib.cleanSource ../.;
  postPatch = ''
    # Create a minimal workspace that only includes lib and cli
    cat > Cargo.toml << EOF
    [workspace]
    resolver = "3"
    members = [
      "lib",
      "cli"
    ]
    EOF
  '';
  cargoBuildCommand = "cargo build --release --package hakanai";
  cargoTestCommand = "cargo test --release --package hakanai";
  cargoHash = workspace.cargoHash;

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

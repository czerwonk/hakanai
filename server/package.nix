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
  pname = "hakanai-server";
  version = workspace.version;

  src = lib.cleanSource ../.;
  postPatch = ''
    # Create a minimal workspace that only includes lib and cli
    cat > Cargo.toml << EOF
    [workspace]
    resolver = "3"
    members = [
      "lib",
      "server"
    ]
    EOF
  '';

  cargoBuildCommand = "cargo build --release --package hakanai-server";
  cargoTestCommand = "cargo test --release --package hakanai-server";
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

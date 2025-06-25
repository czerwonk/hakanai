{
  description = "A minimalist one-time secret sharing service. Share sensitive data through ephemeral links that self-destruct after a single view. No accounts, no tracking, just a simple way to transmit secrets that vanish like morning mist.";

  outputs =
    { self, nixpkgs }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      pkgsForSystem =
        system:
        (import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        });
    in
    {
      overlays.default =
        _final: prev:
        let
          inherit (prev) rustPlatform callPackage lib;
        in
        {
          hakanai-cli = callPackage ./cli/package.nix { inherit rustPlatform lib; };
          hakanai-server = callPackage ./server/package.nix { inherit rustPlatform lib; };
        };

      packages = forAllSystems (system: rec {
        inherit (pkgsForSystem system) hakanai-cli hakanai-server;
        default = hakanai-cli;
      });
    };
}

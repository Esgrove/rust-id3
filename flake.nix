{
  inputs = {
    # Pinned to Rust v1.70
    nixpkgs.url = "github:nixos/nixpkgs?ref=50a7139fbd1acd4a3d4cfa695e694c529dd26f3a";
  };

  outputs =
    { self, nixpkgs }:
    {
      devShells = builtins.mapAttrs (system: pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            commitizen
            ffmpeg
            gcc
            rust-analyzer
            rustfmt
          ];
        };
      }) nixpkgs.legacyPackages;
    };
}

{
  inputs = {
    # Pinned to Rust v1.85
    nixpkgs.url = "github:nixos/nixpkgs?ref=dd613136ee91f67e5dba3f3f41ac99ae89c5406b";
    nixpkgs-cz.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixpkgs-cz,
    }:
    {
      devShells = builtins.mapAttrs (system: pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            ffmpeg
            gcc
            rust-analyzer
            rustc
            rustfmt
            nixpkgs-cz.legacyPackages.${system}.commitizen
          ];
        };
      }) nixpkgs.legacyPackages;
    };
}

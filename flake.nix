{
  description = "Durop development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/8c50a710ddca43d7a530fb805ad55bde8d0141c5";
    rust-overlay.url = "github:oxalica/rust-overlay/74cebce67ef6671ea48fda353a5be2d229bd2920";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "aarch64-darwin" "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; overlays = [ (import rust-overlay) ]; };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              git jq opentofu python312 uv nodejs_24 tailscale pkg-config
            ] ++ lib.optionals stdenv.isLinux [ xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr libxkbcommon ];
            LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux
              (pkgs.lib.makeLibraryPath (with pkgs; [ xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr libxkbcommon ]));
          };
        });
    };
}

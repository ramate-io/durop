{
  description = "Durop development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/8c50a710ddca43d7a530fb805ad55bde8d0141c5";
    # Only Node uses this newer snapshot; retain the existing Rust/Python/provider tools.
    nixpkgs-node.url = "github:NixOS/nixpkgs/4fe25e12eb9f4c744966dd3de5a05246cf098601";
    rust-overlay.url = "github:oxalica/rust-overlay/74cebce67ef6671ea48fda353a5be2d229bd2920";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, nixpkgs-node, rust-overlay, ... }:
    let
      systems = [ "aarch64-darwin" "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; overlays = [ (import rust-overlay) ]; };
          nodePkgs = import nixpkgs-node { inherit system; };
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              # Node 24 + Darwin LLVM 21 corrupts JSII worker fd tracking (#536039).
              git jq opentofu python312 uv nodePkgs.nodejs_22 tailscale pkg-config
            ] ++ lib.optionals stdenv.isLinux [ xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr libxkbcommon ];
            LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.isLinux
              (pkgs.lib.makeLibraryPath (with pkgs; [ xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr libxkbcommon ]));
          };
        });
    };
}

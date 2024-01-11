{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
	  rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
	    extensions = [ "rust-src" "rust-analyzer" "rust-std" ];
	    targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown"];
	  });
      buildInputs = with pkgs; [
        libxkbcommon
        libdrm
        libinput
        libevdev
        wayland
        wayland-protocols
        egl-wayland
        vulkan-loader
        vulkan-validation-layers
        xorg.libX11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXinerama
        xorg.libXi
        xorg.libxcb
        xorg.libXext
        udev
    		bacon
    		wasm-pack
    		rust
        python311
        #lib.getDev alsa
        pkg-config
        (pkgs.lib.getLib alsa-lib)
        (pkgs.lib.getDev alsa-lib)
        gtk2
        gtk3
        swt
        xorg.libXtst
        openssl

        stdenv.cc.cc.lib
    ];
    in
    {
      devShells.${system}.default = pkgs.mkShell rec {
        inherit buildInputs;
        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
      };
    };
}

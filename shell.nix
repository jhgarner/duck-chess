with import <nixpkgs> {};
mkShell rec {
  buildInputs = [ rustup openssl pkg-config udev alsa-lib xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi wayland libGL ];
  shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath (buildInputs ++ [
    udev alsaLib vulkan-loader
    libxkbcommon wayland # To use wayland feature
  ])}"'';
}

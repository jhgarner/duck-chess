with import <nixpkgs> {};
mkShell rec {
  buildInputs = [ openssl pkg-config udev alsa-lib xorg.libX11 xorg.libXcursor xorg.libXrandr xorg.libXi wayland libGL at-spi2-atk
    atkmm
    cairo
    gobject-introspection
    gdk-pixbuf
    glib
    gtk3
    harfbuzz
    librsvg
    libsoup_3
    pango
    webkitgtk_4_1
    openssl ];
  shellHook = ''export PATH="/home/jack/.cargo/bin/:$PATH"'';
  # shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath (buildInputs ++ [
  #   udev alsa-lib vulkan-loader
  #   libxkbcommon wayland # To use wayland feature
  # ])}"'';
}

with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";

  nativeBuildInputs = [
    rustup
    pkgconfig
  ];

  buildInputs = [
    freetype
    expat
    xorg.libX11
    fontconfig

    # for dialogs
    gnome3.zenity

    # fuse
    fuse
  ];

  LD_LIBRARY_PATH = stdenv.lib.makeLibraryPath [
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    vulkan-loader
  ];
}

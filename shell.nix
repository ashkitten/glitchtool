# https://github.com/maps4print/azul/issues/226#issuecomment-569630892

with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  nativeBuildInputs = [
    rustup

    # build-time additional dependencies
    pkgconfig
    python3
  ];

  buildInputs = [
    # run-time additional dependencies
    freetype
    expat
    xorg.libxcb
    fontconfig

    # for dialogs
    gnome3.zenity

    # fuse
    fuse
  ];

  LD_LIBRARY_PATH = stdenv.lib.makeLibraryPath [
    # for azul
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libglvnd
  ];
}

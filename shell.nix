# https://github.com/maps4print/azul/issues/226#issuecomment-569630892

with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  nativeBuildInputs = [
    rustup

    # Build-time Additional Dependencies
    pkgconfig
    python3
  ];

  buildInputs = [
    # Run-time Additional Dependencies
    freetype
    expat
    xorg.libxcb
    fontconfig
  ];

  # LD_LIBRARY_PATH = ''
  #   ${xorg.libXcursor}/lib:${xorg.libX11}/lib
  # '';
  LD_LIBRARY_PATH = stdenv.lib.makeLibraryPath [
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libglvnd
  ];
}

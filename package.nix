{
  lib,
  stdenv,
  rustPlatform,
  glib-networking,
  pkg-config,
  wrapGAppsHook4,
  libxkbcommon,
  wayland,
  glib,
  gobject-introspection,
  gtk4,
  libadwaita,
}:

rustPlatform.buildRustPackage {
  pname = "nmrs";
  version = "1.1.0-stable";

  src = ./.;

  cargoHash = "sha256-hmkUlcw/D38ASqo7GcNK8p6IqfnFIAQWRrjpWymzKFY=";

  nativeBuildInputs = [
    pkg-config
  ]
  ++ lib.optionals stdenv.hostPlatform.isLinux [ wrapGAppsHook4 ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    glib-networking
    libxkbcommon
    wayland
    glib
    gobject-introspection
    gtk4
    libadwaita
  ];

  doCheck = false;
  doInstallCheck = true;

  meta = with lib; {
    description = "Wayland-native frontend for NetworkManager. ";
    homepage = "https://github.com/cachebag/nmrs";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "nmrs";
    platforms = platforms.linux ++ platforms.darwin;
  };
}

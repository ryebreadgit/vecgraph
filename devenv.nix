{
  pkgs,
  lib,
  ...
}:

{
  # https://devenv.sh/basics/

  # https://devenv.sh/packages/
  packages = with pkgs; [
    protobuf
    protolint
    cargo-dist
    git-cliff
  ];

  # https://devenv.sh/variables/
  env = {
    LD_LIBRARY_PATH = lib.makeLibraryPath (
      with pkgs;
      [
      ]
    );
    PKG_CONFIG_PATH = lib.makeSearchPathOutput "dev" "lib/pkgconfig" (
      with pkgs;
      [
      ]
    );
  };

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "stable";
  };

  # https://devenv.sh/scripts/
  scripts.run.exec = "cargo run";
  scripts.build.exec = "cargo build";
  scripts.build-release.exec = "cargo build --release";

  enterShell = ''
    echo "ðŸ¦€ Rust development environment loaded!"
    echo ""
    echo "Available commands:"
    echo "  run           - cargo run"
    echo "  build         - cargo build"  
    echo "  build-release - cargo build --release"
    echo ""
  '';
}

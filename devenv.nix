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
  scripts.changelog-gen.exec = ''
    VERSION=$1
    if [ -z "$VERSION" ]; then
      echo "Usage: changelog-gen <version>"
      exit 1
    fi

    git cliff --unreleased --tag "v$VERSION" --output /tmp/changelog-draft.md

    ''${EDITOR:-nano} /tmp/changelog-draft.md

    if [ -f CHANGELOG.md ]; then
      printf "# Changelog\n\n" > /tmp/changelog-merged.md
      grep -v "^# Changelog" /tmp/changelog-draft.md >> /tmp/changelog-merged.md
      grep -v "^# Changelog" CHANGELOG.md >> /tmp/changelog-merged.md
      mv /tmp/changelog-merged.md CHANGELOG.md
    else
      cp /tmp/changelog-draft.md CHANGELOG.md
    fi

    echo "CHANGELOG.md updated with new version $VERSION"
  '';

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

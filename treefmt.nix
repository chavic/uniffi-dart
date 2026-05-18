{ pkgs, ... }:
{
  projectRootFile = "flake.nix";

  programs = {
    dart-format.enable = true;
    nixfmt.enable = true;
    rustfmt = {
      enable = true;
      package = pkgs.rustToolchains.stable;
      edition = "2021";
    };
  };

  settings = {
    excludes = [
      "experiments/**"
    ];
    formatter = {
      dart-format.excludes = [
        "fixtures/**/test/*.dart"
      ];
      rustfmt.options = [
        "--config-path"
        "./rustfmt.toml"
      ];
    };
  };
}

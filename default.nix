{ pkgs ? import
    (fetchTarball {
      name = "jpetrucciani-2024-08-04";
      url = "https://github.com/jpetrucciani/nix/archive/5d293e2449312b4fef796dcd0836bf6bc1fad684.tar.gz";
      sha256 = "0z8dyf6d9mdka8gq3zchyy5ywzz91bj18xzdjnm0gpwify0pk312";
    })
    { }
}:
let
  name = "argus";

  tools = with pkgs; {
    cli = [
      coreutils
      nixpkgs-fmt
    ];
    rust = [
      cargo
      clang
      rust-analyzer
      rustc
      rustfmt
    ];
    scripts = pkgs.lib.attrsets.attrValues scripts;
  };

  scripts = with pkgs; { };
  paths = pkgs.lib.flatten [ (builtins.attrValues tools) ];
  env = pkgs.buildEnv {
    inherit name paths; buildInputs = paths;
  };
in
(env.overrideAttrs (_: {
  inherit name;
  NIXUP = "0.0.7";
})) // {
  inherit scripts;
}

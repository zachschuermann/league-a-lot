let
   moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
   nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
   ruststable = (nixpkgs.latest.rustChannels.stable.rust.override {
     extensions = [ "rust-src" ];}
   );
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "rust-env";
    nativeBuildInputs = [
        ruststable
        pkg-config
        clang
        llvm
        llvmPackages.libclang
        mold

	rust-analyzer
    ];
    buildInputs = [
        openssl
    ];
  }

{
  description = "Development shell for baseband_fuzz";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = {
    self,
    nixpkgs,
  }: {
    devShells.x86_64-linux.default = let
      pkgs = import nixpkgs {system = "x86_64-linux";};
      llvmPkgs = pkgs.llvmPackages;
      deps = with pkgs; [
        rustc
        cargo
        pixman
        glib
        meson
        ninja
        pkg-config
        flex
        bison
        llvm
        llvmPkgs.clang
        llvmPkgs.libclang
        llvmPkgs.libcxxClang
      ];
    in
      pkgs.mkShell {
        buildInputs = deps;
        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath deps)}";
          export LIBCLANG_PATH="${builtins.toString llvmPkgs.libclang.lib}/lib";
          export PATH="$PATH:${pkgs.rustc}/bin:${pkgs.cargo}/bin";
        '';
      };
  };
}

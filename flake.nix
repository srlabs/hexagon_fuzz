{
  description = "Development shell for baseband_fuzz";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = {
    self,
    nixpkgs,
  }: {
    devShells.x86_64-linux.default = let
      pkgs = import nixpkgs {system = "x86_64-linux";};
      llvmPkgs = pkgs.llvmPackages_20;
      deps = with pkgs; [
        pixman
        glib
        meson
        ninja
        pkg-config
        flex
        bison
        llvm
        llvmPkgs.clangUseLLVM
        llvmPkgs.libcxxClang
      ];
    in
      pkgs.mkShell {
        buildInputs = deps;
        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath deps)}";
        '';
      };
  };
}

{
  description = "Development shell for baseband_fuzz";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = {
    self,
    nixpkgs,
  }: {
    devShells.x86_64-linux.default = let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        config = {
          # This is the key part: add the explicitly allowed insecure package
          permittedInsecurePackages = [
            "python-2.7.18.8" # Replace with the exact version from the error message if it differs
          ];
        };
      };
      llvmPkgs = pkgs.llvmPackages;
      deps = with pkgs; [
        rustc
        cargo
        ncurses5
        libcxx
        pixman
        glib
        meson
        python2
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

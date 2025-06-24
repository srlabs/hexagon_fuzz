{
  description = "Development shell for hexagon_fuzz";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs";
  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config = {
              permittedInsecurePackages = [
                "openssl-1.1.1w"
                "python-2.7.18.8"
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
            socat
            glib
            meson
            python27Full
            ninja
            pkg-config
            flex
            bison
            openssl_1_1
            llvm
            llvmPkgs.clang
            llvmPkgs.libclang
            llvmPkgs.libcxxClang
          ];
        in
        {
          default = pkgs.mkShell {
            buildInputs = deps;
            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath deps)}";
              export LIBCLANG_PATH="${builtins.toString llvmPkgs.libclang.lib}/lib";
              export PATH="$PATH:${pkgs.rustc}/bin:${pkgs.cargo}/bin";
            '';
          };
        }
      );
    };
}

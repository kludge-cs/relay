{
	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

		hooks = {
			url = "github:cachix/git-hooks.nix";
			inputs.nixpkgs.follows = "nixpkgs";
		};

		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = {
		self,
		hooks,
		fenix,
		nixpkgs,
	}: let
		inherit (nixpkgs) lib;

		systems = [
			"aarch64-linux"
			"i686-linux"
			"x86_64-linux"
			"aarch64-darwin"
			"x86_64-darwin"
		];

		forAllSystems = f:
			lib.genAttrs systems (system:
					f rec {
						pkgs =
							import nixpkgs {
								inherit system;
								overlays = [self.overlays.default];
							};

						inherit system;
					});
	in {
		overlays.default = final: prev: {
			rustToolchain = let
				pkgs = fenix.packages.${prev.stdenv.hostPlatform.system};
			in
				pkgs.combine (with pkgs.stable; [
						rustc
						cargo
						clippy
						rust-src
						pkgs.default.rustfmt
					]);
		};

		devShells =
			forAllSystems ({
					pkgs,
					system,
				}: let
					check = self.checks.${system}.pre-commit-check;
				in {
					default =
						pkgs.mkShell {
							inherit (check) shellHook;

							packages =
								check.enabledPackages
								++ (builtins.attrValues {
										inherit
											(pkgs)
											rustToolchain
											cargo-edit
											cargo-watch
											rust-analyzer
											openssl
											pkg-config
											;
									});

							env = {
								RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
								LD_LIBRARY_PATH = "${pkgs.openssl.out}/lib";
								PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
							};
						};
				});

		packages =
			forAllSystems ({pkgs, ...}: {
					default =
						(pkgs.makeRustPlatform {
								cargo = pkgs.rustToolchain;
								rustc = pkgs.rustToolchain;
							}).buildRustPackage {
							pname = "relay";
							version = "0.1.0";
							src = ./.;
							cargoLock.lockFile = ./Cargo.lock;
							buildInputs = [pkgs.openssl];
							RUSTFLAGS = "--cfg=openssl";
						};
				});

		checks =
			forAllSystems ({
					system,
					pkgs,
					...
				}: {
					pre-commit-check =
						hooks.lib.${system}.run {
							src = ./.;
							package = pkgs.prek;
							hooks = {
								convco.enable = true;
								alejandra.enable = true;
								clippy = {
									enable = true;
									package = fenix.packages.${system}.stable.clippy;
								};
								rustfmt = {
									enable = true;
									package = fenix.packages.${system}.default.rustfmt;
								};
								statix = {
									enable = true;
									settings.ignore = ["/.direnv"];
								};
							};
						};
				});

		formatter = forAllSystems ({pkgs, ...}: pkgs.alejandra);
	};
}

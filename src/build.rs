#![feature(if_let)]
extern crate "pkg-config" as pkg_config;
use std::io::fs::PathExtensions;
use std::io::process::Command;
use std::path::Path;

fn main() {
	match pkg_config::find_library("jit") {
		Ok(()) => return,
		Err(..) => {}
	}
	let ref final_lib_dir = Path::new("native/jit/.libs");
	let ref submod_path = Path::new("native");
	if !final_lib_dir.join("libjit.a").exists() {
		if !submod_path.exists() {
			run(Command::new("git").arg("submodule").arg("init"));
		}
		run(Command::new("git").arg("submodule").arg("update"));
		if !submod_path.exists() {
			run(Command::new("git").arg("clone").arg("git://git.savannah.gnu.org/libjit.git").arg("native"));
		}
		run(Command::new("sh").arg("auto_gen.sh").cwd(submod_path));
		run(Command::new("sh").arg("configure").arg("--enable-static").arg("--disable-shared").arg("CFLAGS=-fPIC").cwd(submod_path));
		run(Command::new("make").cwd(submod_path));
	}
    println!("cargo:rustc-flags=-l jit:static");
	println!("cargo:rustc-flags=-L {}", final_lib_dir.display());
}
fn run(command: &mut Command) {
	println!("{}" , command);
	match command.output() {
		Ok(ref process) if !process.status.success() => {
			panic!("failed with output: \n{}\n{}", String::from_utf8_lossy(process.output[]), String::from_utf8_lossy(process.error[]));
		},
		Ok(_) => (),
		Err(err) => panic!("failed due to {}", err)
	}
}
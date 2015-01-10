#![allow(unstable)]
#[cfg(not(windows))]
extern crate "pkg-config" as pkg_config;
use std::io::fs::PathExtensions;
use std::io::process::Command;
use std::path::Path;

#[cfg(windows)]
static FINAL_LIB:&'static str = "libjit.dll";

#[cfg(not(windows))]
static FINAL_LIB:&'static str = "libjit.a";

static MINGW:&'static str = "c:/mingw";

fn main() {
	if cfg!(windows) && !Path::new(MINGW).exists() {
		panic!("LibJIT build requires MinGW and MSYS to be installed");
	} else if pkg_config::find_library("jit").is_ok() {
		return;
	}
	let ref submod_path = Path::new("libjit");
	let ref final_lib_dir = submod_path.join("jit/.libs");
	if !final_lib_dir.join(FINAL_LIB).exists() {
		if !submod_path.exists() {
			run(Command::new("git").arg("submodule").arg("init"));
		}
		run(Command::new("git").arg("submodule").arg("update"));
		if !submod_path.exists() {
			run(Command::new("git").arg("clone").arg("git://git.savannah.gnu.org/libjit.git").arg(submod_path.display().to_string().as_slice()));
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
			panic!("failed with output: \n{}\n{}", String::from_utf8_lossy(&*process.output), String::from_utf8_lossy(&*process.error));
		},
		Ok(_) => (),
		Err(err) => panic!("failed due to {}", err)
	}
}
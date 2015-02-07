#![feature(env, io, path)]
#[cfg(not(windows))]
extern crate "pkg-config" as pkg_config;
use std::old_io::process::Command;
use std::old_io::fs;
use std::old_path::Path as OPath;
use std::env;
use std::path::Path;

#[cfg(windows)]
static FINAL_LIB:&'static str = "libjit.dll";

#[cfg(not(windows))]
static FINAL_LIB:&'static str = "libjit.a";

static MINGW:&'static str = "c:/mingw";

fn exists(path: &Path) -> bool {
	use std::old_io::fs::{PathExtensions};
	OPath::new(path.to_str().unwrap()).exists()
}
fn path(path: &str) -> &Path {
	&Path::new(path)
}
fn main() {
	if cfg!(windows) && !exists(path(MINGW)) {
		panic!("LibJIT build requires MinGW and MSYS to be installed");
	} else if pkg_config::find_library("jit").is_ok() {
		return;
	}
	let out_dir = env::var("OUT_DIR").unwrap();
	let out_dir = Path::new(&*out_dir);
	let submod_path = Path::new("libjit");
	let final_lib_dir = submod_path.join("jit/.libs");
	if !exists(&*final_lib_dir.join(FINAL_LIB)) {
		if !exists(submod_path) {
			run(Command::new("git").arg("submodule").arg("init"));
		}
		run(Command::new("git").arg("submodule").arg("update"));
		if !exists(submod_path) {
			run(Command::new("git").arg("clone").arg("git://git.savannah.gnu.org/libjit.git").arg(&*submod_path.display().to_string()));
		}
		let submod_path = OPath::new(submod_path.to_str().unwrap());
		run(Command::new("sh").arg("auto_gen.sh").cwd(&submod_path));
		run(Command::new("sh").arg("configure").arg("--enable-static").arg("--disable-shared").arg("CFLAGS=-fPIC").cwd(&submod_path));
		run(Command::new("make").cwd(&submod_path));
	}
	let ref from = OPath::new(final_lib_dir.join(FINAL_LIB).to_str().unwrap());
	let ref to = OPath::new(out_dir.join(FINAL_LIB).to_str().unwrap());
	fs::copy(from, to).unwrap();
    println!("cargo:rustc-flags=-l jit:static -L {}", out_dir.display());
}
fn run(command: &mut Command) {
	println!("{:?}" , command);
	match command.output() {
		Ok(ref process) if !process.status.success() => {
			panic!("failed with output: \n{}\n{}", String::from_utf8_lossy(&*process.output), String::from_utf8_lossy(&*process.error));
		},
		Ok(_) => (),
		Err(err) => panic!("failed due to {}", err)
	}
}
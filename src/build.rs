use std::io;
use std::io::fs;
use std::io::fs::PathExtensions;
use std::io::process::{Process, Command};
use std::path::Path;
use std::os;

fn main() {

	io::println("Searching for LibJIT");
	let ref lib_path = Path::new("/usr/lib/libjit.so");
	let ref dest_path = Path::new(os::getenv("OUT_DIR").expect("No output directory given")).join(Path::new("libjit.so"));
	if lib_path.exists() {
		fs::copy(lib_path, dest_path).unwrap();
	} else if !dest_path.exists() {
		io::println("No LibJIT found in /usr/lib/ so updating LibJIT");
		let mut git = Command::new("git");
		run(git.clone().arg("submodule init"));
		run(git.arg("submodule update"));
		let old_cwd = os::getcwd();
		os::change_dir(&Path::new("native"));
		io::println("Configuring LibJIT");
		run(Command::new("sh").arg("autogen.sh"));
		run_cmd("./configure");

		io::println("Building LibJIT");
		run_cmd("make");

		run(Command::new("make").arg("DESTDIR=root").arg("install"));
		os::change_dir(&old_cwd);

		let ref lib_path = Path::new("native/jit/.libs/libjit.so");
		fs::copy(lib_path, dest_path).unwrap();
	}
}

fn run_cmd(command: &str) -> Process {
	run(&mut Command::new(command))
}
fn run(command: &mut Command) -> Process {
	command.spawn().ok().expect(format!("Error while running command `{}`", command).as_slice())
}
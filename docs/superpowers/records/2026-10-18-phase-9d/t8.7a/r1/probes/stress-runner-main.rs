use std::io::Write;
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let text = std::fs::read(&path).unwrap();
    let o = rexx_exec::run_program_collect_every_alloc(&path, text, rexx_exec::Invocation::none());
    std::io::stdout().write_all(&o.stdout).unwrap();
    std::io::stderr().write_all(&o.stderr).unwrap();
    eprintln!("[collections {}]", o.collections);
    std::process::exit(o.exit_code);
}

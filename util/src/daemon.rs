use std::process;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {

        use std::os::windows::process::CommandExt;

        ///
        /// Running in Background
        /// 
        /// # Panics
        /// 
        /// paniced when start a new process failed.
        /// 
        pub fn daemon() {
            std::process::Command::new("cmd.exe").
                creation_flags(0x08000000).
                arg("/c").
                args(std::env::args().filter(|arg| arg.ne("--daemon")).collect::<Vec<String>>()).
                stdin(std::process::Stdio::null()).
                stdout(std::process::Stdio::null()).
                stderr(std::process::Stdio::null()).
                spawn().
                expect("running in background failed");
            process::exit(1);
        }

    } else {

        extern crate daemonize;

        use std::fs::File;

        use daemonize::Daemonize;

        ///
        /// unix-like Platform Running in Background
        /// 
        /// # Notes
        /// 
        /// not test certificates available
        pub fn daemon() {
            let stdout = File::create("/dev/null").unwrap();
            let stderr = File::create("/dev/null").unwrap();

            Daemonize::new().
                umask(0o777).
                stdout(stdout).
                stderr(stderr).
                expect("running in background failed");
            process::exit(1);
        }
    }
}
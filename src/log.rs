pub fn info<S: AsRef<str>>(msg : S) {
    let msg = msg.as_ref();
    println!(" [.] {}", msg);
}

pub fn continue_info<S: AsRef<str>>(msg : S) {
    let msg = msg.as_ref();
    println!(" | - {}", msg);
}

pub fn lf() {
    println!("");
}

pub fn pending<S: AsRef<str>>(msg : S) {
    let msg = msg.as_ref();
    println!(" [*] {}", msg);
}

pub fn error<S: AsRef<str>>(msg : S) {
    let msg = msg.as_ref();
    println!(" [X] {}", msg);
}

pub fn warning<S: AsRef<str>>(msg : S) {
    let msg = msg.as_ref();
    println!(" [!] {}", msg);
}

pub fn positive<S: AsRef<str>>(msg : S) {
    let msg = msg.as_ref();
    println!(" [+] {}", msg);
}

pub fn negative<S: AsRef<str>>(msg : S) {
    let msg = msg.as_ref();
    println!(" [-] {}", msg);
}
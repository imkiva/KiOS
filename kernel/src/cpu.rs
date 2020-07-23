pub fn forever_hlt() -> ! {
    loop {
        hlt();
    }
}

pub fn hlt() {
    x86_64::instructions::hlt();
}

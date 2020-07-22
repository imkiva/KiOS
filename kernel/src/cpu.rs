pub fn forever_hlt() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

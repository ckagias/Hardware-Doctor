fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        res.compile().expect("embed icon.ico into the exe");
    }
}

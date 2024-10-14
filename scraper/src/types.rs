pub trait UrlTyped {
    fn get_path(&self) -> &'static str;
}

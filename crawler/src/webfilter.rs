use crate::page::Page;
use whatlang::{detect, Lang};

pub fn pass(page: &Page) -> bool {
     match detect(&page.extract) {
        Some(info) => match info.lang() {
            Lang::Eng => info.is_reliable(),
            _ => false,
        },
        None => false,
    }
}

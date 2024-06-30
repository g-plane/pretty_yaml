use crate::config::LanguageOptions;

pub(crate) struct Ctx<'a> {
    pub indent_width: usize,
    pub options: &'a LanguageOptions,
}

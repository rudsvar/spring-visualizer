#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Autowired {
    name: String,
    class: String,
}

impl Autowired {
    pub fn new(class: String, name: String) -> Self {
        Autowired { class, name }
    }
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn class(&self) -> &str {
        self.class.as_ref()
    }
}

use strum::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum ComponentType {
    SpringBootApplication,
    Configuration,
    Controller,
    Service,
    Repository,
    Component,
}

impl ComponentType {
    pub fn color_code(&self) -> &'static str {
        match self {
            ComponentType::SpringBootApplication => "#2c9162",
            ComponentType::Configuration => "#28a9e0",
            ComponentType::Controller => "#7050bf",
            ComponentType::Service => "#a81347",
            ComponentType::Repository => "#e06907",
            ComponentType::Component => "#ffc400",
        }
    }
}

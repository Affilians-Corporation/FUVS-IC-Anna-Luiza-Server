use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Theme {

    pub name: String,

    pub sub_themes: Vec<SubTheme>
}

impl Theme {
    pub fn new(name: &str) -> Self {
        return Theme {
            name: name.to_string(), 
            sub_themes: Vec::new()
        }
    }
    pub fn add_subtheme(&mut self, sub_theme: SubTheme) {
        self.sub_themes.push(sub_theme);
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct SubTheme {

    name: String,

    difficulties: Vec<Difficulty>
}

impl SubTheme {
    pub fn new(name: &str) -> Self {
        return SubTheme {
            name: name.to_string(),
            difficulties: Vec::new()
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum DifficultyLevel {

    Easy,

    Medium,

    Hard
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difficulty {
    level: DifficultyLevel,

    questions: Vec<Question>
}

impl Difficulty {
    pub fn new(level: DifficultyLevel) -> Self {
        return Difficulty {
            level,
            questions: Vec::new()
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    id: u32,

    resources: Vec<Resource>
}

impl Question {
    pub fn new(id: u32, resources: Vec<Resource>) -> Self {
        return Question {
            id,
            resources
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource{

    resource_name: String,
    resource_type: ResourceType,

    resource_uri: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum ResourceType {

    Text,

    Image,

    Scene,

    Audio,

    Video
}

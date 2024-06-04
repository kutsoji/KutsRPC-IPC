use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Activity<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamps: Option<Timestamps>,
    assets: Assets<'a>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    buttons: Vec<Button<'a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Timestamps {
    pub start: i64,
    pub end: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Assets<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<&'a str>,
}

impl<'a> Assets<'a> {
    pub fn new() -> Self {
        Self {
            large_image: None,
            large_text: None,
            small_image: None,
            small_text: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Button<'a> {
    pub label: &'a str,
    pub url: &'a str,
}

impl<'a> Activity<'a> {
    pub fn new() -> Self {
        Self {
            state: None,
            details: None,
            timestamps: None,
            assets: Assets::new(),
            buttons: Vec::new(),
        }
    }

    pub fn set_state(&mut self, state: &'a str) -> Self {
        self.state = Some(state);
        self.clone()
    }

    pub fn set_details(&mut self, details: &'a str) -> Self {
        self.details = Some(details);
        self.clone()
    }

    pub fn set_timestamps(&mut self, start: i64, end: i64) -> Self {
        self.timestamps = Some(Timestamps { start, end });
        self.clone()
    }

    pub fn set_large_image(&mut self, large_image: &'a str) -> Self {
        self.assets.large_image = Some(large_image);
        self.clone()
    }

    pub fn set_large_text(&mut self, large_text: &'a str) -> Self {
        self.assets.large_text = Some(large_text);
        self.clone()
    }

    pub fn set_small_image(&mut self, small_image: &'a str) -> Self {
        self.assets.small_image = Some(small_image);
        self.clone()
    }

    pub fn set_small_text(&mut self, small_text: &'a str) -> Self {
        self.assets.small_text = Some(small_text);
        self.clone()
    }

    pub fn set_buttons(&mut self, buttons: Vec<(&'a str, &'a str)>) -> Self {
        for button in buttons {
            self.buttons.push(Button {
                label: button.0,
                url: button.1,
            })
        }
        self.clone()
    }
}

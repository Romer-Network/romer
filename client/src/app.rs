#[derive(PartialEq, Clone, Debug)]
pub enum OrgType {
    MarketMaker,
    StablecoinIssuer,
}

#[derive(PartialEq, Debug)]
pub enum Screen {
    Welcome,
    Registration,
}

pub struct App {
    pub org_type: Option<OrgType>,
    pub screen: Screen,
    pub organization_name: String,
    pub sender_comp_id: String,
    pub selected_field: usize,
    pub show_success: bool,
}

impl App {
    pub fn new() -> App {
        App {
            org_type: None,
            screen: Screen::Welcome,
            organization_name: String::new(),
            sender_comp_id: String::new(),
            selected_field: 0,
            show_success: false,
        }
    }

    pub fn get_help_text(&self) -> String {
        match self.screen {
            Screen::Welcome => String::from("Press 'M' for Market Maker, 'S' for Stablecoin Issuer, ESC to exit"),
            Screen::Registration => String::from("TAB to switch fields, ESC to go back, ENTER to submit"),
        }
    }

    pub fn debug_state(&self) -> String {
        format!(
            "Screen: {:?}, Field: {}, Org Type: {:?}",
            self.screen,
            self.selected_field,
            self.org_type
        )
    }
}
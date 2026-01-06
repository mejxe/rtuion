#[derive(Debug, Clone, Copy)]
pub enum Tabs {
    TimerTab,
    SettingsTab,
    StatsTab,
}
impl Tabs {
    pub fn next(&mut self) {
        *self = match self {
            Tabs::TimerTab => Tabs::SettingsTab,
            Tabs::SettingsTab => Tabs::StatsTab,
            Tabs::StatsTab => Tabs::TimerTab,
        };
    }
}
impl From<Tabs> for usize {
    fn from(value: Tabs) -> Self {
        match value {
            Tabs::TimerTab => 0,
            Tabs::SettingsTab => 1,
            Tabs::StatsTab => 2,
        }
    }
}
impl From<&Tabs> for usize {
    fn from(value: &Tabs) -> Self {
        value.clone().into()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Tabs {
    TimerTab,
    SettingsTab,
    StatsTab,
    NoTab,

}
impl Tabs {
    pub fn next(&mut self) {
        *self = match self {
            Tabs::TimerTab => Tabs::SettingsTab,
            Tabs::SettingsTab => Tabs::StatsTab,
            Tabs::StatsTab => Tabs::TimerTab,
            Tabs::NoTab => Tabs::NoTab,
        };
    }
    pub fn prev(&mut self) {
        *self = match self {
            Tabs::TimerTab => Tabs::StatsTab,
            Tabs::SettingsTab => Tabs::TimerTab,
            Tabs::StatsTab => Tabs::SettingsTab,
            Tabs::NoTab => Tabs::NoTab,
        };
    }
}
impl From<Tabs> for usize {
    fn from(value: Tabs) -> Self {
        match value {
            Tabs::TimerTab => 0,
            Tabs::SettingsTab => 1,
            Tabs::StatsTab => 2,
            Tabs::NoTab => 3,
        }
    }
}
impl From<&Tabs> for usize {
    fn from(value: &Tabs) -> Self {
        value.clone().into()
    }
}

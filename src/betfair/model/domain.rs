#[derive(Clone, Copy, Debug)]
pub enum BetfairDomain {
    Com,
    It,
    Es,
}

impl BetfairDomain {
    pub fn host(self) -> &'static str {
        match self {
            BetfairDomain::Com => "api.betfair.com",
            BetfairDomain::It => "api.betfair.it",
            BetfairDomain::Es => "api.betfair.es",
        }
    }
}
